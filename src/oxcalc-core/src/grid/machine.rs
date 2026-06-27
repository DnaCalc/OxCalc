#![forbid(unsafe_code)]

//! Strict Excel-grid reference machines for W061.
//!
//! This module is deliberately boring storage: finite BTreeMap support for the
//! authored/computed grid surface plus a separate scalar dirty-closure oracle.
//! Optimized grid storage, template coalescing, spill repair, and structural
//! edit transforms prove themselves against this floor; they do not live here.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

use crate::grid::ast::{
    ExcelGridFormulaAnchor, ExcelGridReference, ExcelGridReferenceTransformPayload,
    ExcelGridStructuralEdit,
};
use crate::grid::coords::{ExcelGridBounds, ExcelGridCellAddress};
use crate::grid::geometry::{ExcelGridStructuredTable, ExcelGridStructuredTableColumn};
use crate::grid::reference_engine::{
    EXCEL_GRID_PROFILE_ID, ExcelGridReferenceSystemProvider, StrictExcelGridReferenceProfile,
    decode_excel_grid_reference_payload, excel_grid_defined_name_key,
    excel_grid_reference_like_from_profile_record,
};
use oxfml_core::binding::{
    BindContext, BindRequest, BoundFormula, NormalizedReference, ReferenceBindProfile,
    ReferenceTransformKind, ReferenceTransformOutcome, ReferenceTransformRequest,
};
use oxfml_core::consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest};
use oxfml_core::interface::{
    TableCallerRegion, TableColumnDescriptor, TableDescriptor, TableRef, TableRegionKind,
    TypedContextQueryBundle,
};
use oxfml_core::red::project_red_view;
use oxfml_core::seam::Locus;
use oxfml_core::source::{
    FormulaChannelKind, FormulaSourceRecord, FormulaToken, StructureContextVersion,
};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};
use oxfml_core::{EvaluationBackend, bind_formula};
use oxfunc_core::host_info::{
    AggregateCellContext, AggregateReferenceContext, HostInfoError, HostInfoProvider,
};
use oxfunc_core::resolver::{
    CallerContext, ReferenceComposeRequest, ReferenceDereferenceRequest,
    ReferenceEnumerationRequest, ReferenceFacts, ReferenceFactsRequest, ReferenceResolutionError,
    ReferenceSystemError, ReferenceSystemOperation, ReferenceSystemProvider,
    ReferenceTextResolveRequest, ReferenceTransformKind as EvalTransformKind,
    ReferenceTransformRequest as EvalTransformRequest, ResolvedReferenceCell,
    ResolvedReferenceExtent, ResolvedReferenceValues, materialize_resolved_reference_values,
    reference_facts,
};
use oxfunc_core::value::{
    ArrayShape, CalcArray, CalcValue, CoreValue, ExcelText, ReferenceLike, WorksheetErrorCode,
};

use crate::grid::authored::{GridAuthoredCell, GridFormulaCell};
use crate::grid::error::GridRefError;
use crate::grid::geometry::GridRect;

mod axis_state;
mod calc_ref_sheet;
mod differential;
mod host_info;
mod invalidation;
mod optimized_provider;
mod optimized_sheet;
mod optimized_storage;
mod optimized_valuation;
mod overlay;
mod r1c1_plan;
mod spill_ledger;
mod warm_no_op;
pub use axis_state::*;
pub use calc_ref_sheet::*;
pub use differential::*;
pub use host_info::*;
pub use invalidation::*;
pub use optimized_provider::*;
pub use optimized_sheet::*;
pub use optimized_storage::*;
pub use optimized_valuation::*;
pub use overlay::*;
pub use r1c1_plan::*;
pub use spill_ledger::*;
pub use warm_no_op::*;

// Recalc-phase spill publication tallies; defined here (not in spill_ledger) so
// both the recalc paths and the spill_ledger helpers can touch its fields.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct GridSpillPublicationCounters {
    facts_published: usize,
    facts_blocked: usize,
    ghost_cells_published: usize,
}

pub const GRID_CALC_REF_DEFAULT_MATERIALIZATION_LIMIT: u64 = 100_000;
pub const GRID_INVALIDATION_REF_DEFAULT_SCALARIZATION_LIMIT: u64 = 100_000;
pub const GRID_INVALIDATION_COMPRESSED_RANGE_INDEX_BLOCK_SIZE: u32 = 1_024;
pub const GRID_OPTIMIZED_PROVIDER_MATERIALIZATION_LIMIT: usize = 100_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridAggregateContextQueryReport {
    pub resolved_rect: GridRect,
    pub rows: usize,
    pub cols: usize,
    pub declared_cell_count: usize,
    pub row_context_runs: usize,
    pub explicit_axis_row_entries_visited: usize,
    pub default_row_runs: usize,
    pub axis_run_probe_count: usize,
    pub per_cell_context_expansion_count: usize,
    pub manually_hidden_rows: u32,
    pub filtered_hidden_rows: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GridAggregateRowContextRun {
    first_row: u32,
    last_row: u32,
    props: GridAxisProps,
}

impl GridAggregateRowContextRun {
    #[must_use]
    const fn row_count(&self) -> u32 {
        self.last_row - self.first_row + 1
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GridAggregateContextQueryPlan {
    resolved_rect: GridRect,
    rows: usize,
    cols: usize,
    declared_cell_count: usize,
    row_runs: Vec<GridAggregateRowContextRun>,
    explicit_axis_row_entries_visited: usize,
    default_row_runs: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridCalcRefCellReadout {
    pub address: ExcelGridCellAddress,
    pub authored: Option<GridAuthoredCell>,
    pub computed: CalcValue,
    pub spill_anchor: Option<ExcelGridCellAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridCalcRefRecalcReport {
    pub occupied_cells: usize,
    pub literal_cells: usize,
    pub formula_cells: usize,
    pub cells_evaluated: usize,
    pub formula_evaluations: usize,
    pub spill_repair_passes: usize,
    pub spill_repair_formula_evaluations: usize,
    pub spill_repair_converged: bool,
    pub spill_facts_published: usize,
    pub spill_facts_blocked: usize,
    pub spill_ghost_cells_published: usize,
    pub visited_cells: Vec<ExcelGridCellAddress>,
}

impl GridCalcRefRecalcReport {
    #[must_use]
    pub fn p00_non_spill_exact_once_holds(&self) -> bool {
        self.cells_evaluated == self.occupied_cells
            && self.visited_cells.len() == self.occupied_cells
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridRegionMaterializationReport {
    pub cells_written: usize,
    pub rect: GridRect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridOptimizedCellSource {
    SparsePoint,
    DenseValueRegion { region_index: usize },
    RepeatedFormulaRegion { region_index: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridOptimizedCellReadout {
    pub address: ExcelGridCellAddress,
    pub authored: Option<GridAuthoredCell>,
    pub source: Option<GridOptimizedCellSource>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridOptimizedComputedReadout {
    pub address: ExcelGridCellAddress,
    pub computed: CalcValue,
    pub source: Option<GridOptimizedCellSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedStorageStats {
    pub sparse_point_cells: usize,
    pub dense_value_regions: usize,
    pub dense_value_cells: u64,
    pub repeated_formula_regions: usize,
    pub repeated_formula_cells: u64,
    pub distinct_repeated_formula_templates: usize,
    pub spill_facts: usize,
    pub authored_cells_upper_bound: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedPartitionWitnessReport {
    pub sparse_point_cells: usize,
    pub dense_value_regions: usize,
    pub repeated_formula_regions: usize,
    pub dense_value_pair_checks: u64,
    pub repeated_formula_pair_checks: u64,
    pub dense_value_overlap_count: usize,
    pub repeated_formula_overlap_count: usize,
    pub max_parallelism_bound: u64,
}

impl GridOptimizedPartitionWitnessReport {
    #[must_use]
    pub const fn p18_partition_witness_holds(&self) -> bool {
        self.dense_value_overlap_count == 0 && self.repeated_formula_overlap_count == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedStorageByteReport {
    pub accounting_model: &'static str,
    pub authored_storage_bytes: u64,
    pub sparse_point_bytes: u64,
    pub dense_value_region_bytes: u64,
    pub repeated_formula_region_bytes: u64,
    pub metadata_bytes: u64,
    pub authored_cells_upper_bound: u64,
    pub dense_value_cells: u64,
    pub dense_numeric_packed_cells: u64,
    pub repeated_formula_cells: u64,
    pub sparse_point_cells: u64,
    pub grid_cell_capacity: u64,
    pub blank_cells: u64,
    pub blank_cell_bytes: u64,
}

impl GridOptimizedStorageByteReport {
    #[must_use]
    pub fn authored_bytes_per_cell_micros(&self) -> u64 {
        bytes_per_cell_micros(self.authored_storage_bytes, self.authored_cells_upper_bound)
    }

    #[must_use]
    pub fn dense_bytes_per_cell_micros(&self) -> u64 {
        bytes_per_cell_micros(self.dense_value_region_bytes, self.dense_value_cells)
    }

    #[must_use]
    pub fn repeated_formula_bytes_per_cell_micros(&self) -> u64 {
        bytes_per_cell_micros(
            self.repeated_formula_region_bytes,
            self.repeated_formula_cells,
        )
    }

    #[must_use]
    pub fn sparse_point_bytes_per_cell_micros(&self) -> u64 {
        bytes_per_cell_micros(self.sparse_point_bytes, self.sparse_point_cells)
    }

    #[must_use]
    pub fn p10_dense_value_budget_holds(&self) -> bool {
        self.dense_value_cells == 0 || self.dense_bytes_per_cell_micros() <= 17_000_000
    }

    #[must_use]
    pub fn p10_repeated_formula_budget_holds(&self) -> bool {
        self.repeated_formula_cells == 0
            || self.repeated_formula_bytes_per_cell_micros() <= 17_000_000
    }

    #[must_use]
    pub fn p10_sparse_singleton_budget_holds(&self) -> bool {
        self.sparse_point_cells == 0 || self.sparse_point_bytes_per_cell_micros() <= 85_000_000
    }

    #[must_use]
    pub const fn p10_blank_cells_zero_bytes_holds(&self) -> bool {
        self.blank_cell_bytes == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedCowRetentionReport {
    pub retained_revision_count: usize,
    pub unique_dense_payloads: usize,
    pub unique_dense_payload_bytes: u64,
    pub dense_region_metadata_bytes: u64,
    pub repeated_formula_region_bytes: u64,
    pub sparse_point_bytes: u64,
    pub sheet_root_metadata_bytes: u64,
    pub retained_compact_regions: usize,
    pub cow_retained_bytes: u64,
    pub naive_full_snapshot_retention_bytes_floor: u64,
    pub retained_to_naive_ratio_micros: u64,
}

impl GridOptimizedCowRetentionReport {
    #[must_use]
    pub const fn p21_cow_retention_holds(&self) -> bool {
        self.retained_revision_count > 1
            && self.cow_retained_bytes < self.naive_full_snapshot_retention_bytes_floor
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedStructuralEditReport {
    pub edit: GridAxisEdit,
    pub sparse_points_kept: usize,
    pub sparse_points_dropped: usize,
    pub sparse_formula_cells_transformed: usize,
    pub sparse_formula_reference_transforms: usize,
    pub dense_value_regions_before: usize,
    pub dense_value_regions_after: usize,
    pub dense_value_regions_dropped: usize,
    pub dense_value_cells_before: u64,
    pub dense_value_cells_after: u64,
    pub repeated_formula_regions_before: usize,
    pub repeated_formula_regions_after: usize,
    pub repeated_formula_regions_dropped: usize,
    pub repeated_formula_cells_before: u64,
    pub repeated_formula_cells_after: u64,
    pub repeated_formula_segments_transformed: usize,
    pub repeated_formula_reference_transforms: usize,
    pub spill_facts_kept: usize,
    pub spill_facts_dropped: usize,
    pub merged_regions_kept: usize,
    pub merged_regions_dropped: usize,
    pub feature_regions_kept: usize,
    pub feature_regions_dropped: usize,
    pub feature_regions_marked_needs_refresh: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct GridCellCoord {
    row: u32,
    col: u32,
}

impl GridCellCoord {
    #[must_use]
    const fn new(row: u32, col: u32) -> Self {
        Self { row, col }
    }

    #[must_use]
    fn from_address(address: &ExcelGridCellAddress) -> Self {
        Self::new(address.row, address.col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxfunc_core::resolver::ReferenceTextResolutionMode;

    fn bounds() -> ExcelGridBounds {
        ExcelGridBounds {
            max_rows: 10,
            max_cols: 5,
        }
    }

    fn sheet() -> GridCalcRefSheet {
        GridCalcRefSheet::new("book:default", "sheet:default", bounds())
    }

    fn address(row: u32, col: u32) -> ExcelGridCellAddress {
        ExcelGridCellAddress::new("book:default", "sheet:default", row, col)
    }

    fn set(
        addresses: impl IntoIterator<Item = ExcelGridCellAddress>,
    ) -> BTreeSet<ExcelGridCellAddress> {
        addresses.into_iter().collect()
    }

    fn optimized_sheet() -> GridOptimizedSheet {
        GridOptimizedSheet::new("book:default", "sheet:default", bounds())
    }

    fn array_col(values: impl IntoIterator<Item = f64>) -> CalcValue {
        CalcValue::array(
            oxfunc_core::value::CalcArray::from_rows(
                values
                    .into_iter()
                    .map(|value| vec![CalcValue::number(value)])
                    .collect(),
            )
            .unwrap(),
        )
    }

    fn strict_grid_reference_from_text(
        text: impl Into<String>,
        address: &ExcelGridCellAddress,
        bounds: ExcelGridBounds,
    ) -> ReferenceLike {
        ExcelGridReferenceSystemProvider::new(
            address.workbook_id.clone(),
            address.sheet_id.clone(),
            address.row,
            address.col,
        )
        .with_bounds(bounds)
        .resolve_text(&ReferenceTextResolveRequest {
            text: text.into(),
            mode: ReferenceTextResolutionMode::Indirect,
            a1_style: Some(true),
            caller_context: None,
        })
        .expect("strict grid reference text should resolve")
    }

    #[test]
    fn optimized_grid_keeps_sparse_singletons_without_region_storage() {
        let mut sheet = optimized_sheet();
        sheet
            .set_literal(address(1, 1), CalcValue::number(7.0))
            .unwrap();
        sheet
            .set_formula(
                address(10, 5),
                GridFormulaCell::new("=A1", "excel.grid.v1:cell:R[-9]C[-4]"),
            )
            .unwrap();

        let stats = sheet.storage_stats();
        assert_eq!(stats.sparse_point_cells, 2);
        assert_eq!(stats.dense_value_regions, 0);
        assert_eq!(stats.repeated_formula_regions, 0);
        assert_eq!(stats.authored_cells_upper_bound, 2);

        let readout =
            sheet.sampled_authored_readout([address(1, 1), address(10, 5), address(5, 5)]);
        assert_eq!(
            readout[0].authored,
            Some(GridAuthoredCell::Literal(CalcValue::number(7.0)))
        );
        assert_eq!(
            readout[0].source,
            Some(GridOptimizedCellSource::SparsePoint)
        );
        assert!(matches!(
            readout[1].authored,
            Some(GridAuthoredCell::Formula(_))
        ));
        assert_eq!(
            readout[1].source,
            Some(GridOptimizedCellSource::SparsePoint)
        );
        assert!(readout[2].authored.is_none());
        assert!(readout[2].source.is_none());
    }

    #[test]
    fn optimized_grid_stores_dense_values_as_region_and_projects_to_reference() {
        let mut sheet = optimized_sheet();
        let rect = GridRect::new("book:default", "sheet:default", 1, 1, 4, 4, bounds()).unwrap();

        let report = sheet
            .put_dense_literal_region_with(rect.clone(), |address| {
                CalcValue::number(f64::from((address.row * 100) + address.col))
            })
            .unwrap();

        assert_eq!(report.cells_written, 16);
        assert_eq!(sheet.dense_value_regions().len(), 1);
        assert_eq!(sheet.dense_value_regions()[0].row_major_values().len(), 16);
        assert_eq!(
            sheet.storage_stats(),
            GridOptimizedStorageStats {
                sparse_point_cells: 0,
                dense_value_regions: 1,
                dense_value_cells: 16,
                repeated_formula_regions: 0,
                repeated_formula_cells: 0,
                distinct_repeated_formula_templates: 0,
                spill_facts: 0,
                authored_cells_upper_bound: 16,
            }
        );
        let bytes = sheet.storage_byte_report();
        assert_eq!(bytes.dense_numeric_packed_cells, 16);
        assert_eq!(bytes.blank_cell_bytes, 0);
        assert!(bytes.p10_dense_value_budget_holds());
        assert!(bytes.p10_blank_cells_zero_bytes_holds());

        let readout = sheet.sampled_authored_readout([address(4, 4)]);
        assert_eq!(
            readout[0].authored,
            Some(GridAuthoredCell::Literal(CalcValue::number(404.0)))
        );
        assert_eq!(
            readout[0].source,
            Some(GridOptimizedCellSource::DenseValueRegion { region_index: 0 })
        );

        let reference = sheet.project_authored_to_reference(100).unwrap();
        assert_eq!(reference.authored().len(), 16);
        assert_eq!(
            reference.authored().get(&address(4, 4)),
            Some(&GridAuthoredCell::Literal(CalcValue::number(404.0)))
        );
    }

    #[test]
    fn optimized_grid_stores_repeated_r1c1_formula_once_with_punch_through_override() {
        let mut sheet = optimized_sheet();
        let rect = GridRect::new("book:default", "sheet:default", 1, 2, 3, 2, bounds()).unwrap();
        let formula = GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1);

        sheet
            .put_repeated_formula_region(rect, formula.clone())
            .unwrap();
        sheet
            .set_literal(address(2, 2), CalcValue::number(99.0))
            .unwrap();

        let stats = sheet.storage_stats();
        assert_eq!(stats.sparse_point_cells, 1);
        assert_eq!(stats.repeated_formula_regions, 1);
        assert_eq!(stats.repeated_formula_cells, 3);
        assert_eq!(stats.distinct_repeated_formula_templates, 1);

        let readout = sheet.sampled_authored_readout([address(1, 2), address(2, 2), address(3, 2)]);
        assert_eq!(
            readout[0].source,
            Some(GridOptimizedCellSource::RepeatedFormulaRegion { region_index: 0 })
        );
        assert!(matches!(
            readout[0].authored,
            Some(GridAuthoredCell::Formula(_))
        ));
        assert_eq!(
            readout[1].authored,
            Some(GridAuthoredCell::Literal(CalcValue::number(99.0)))
        );
        assert_eq!(
            readout[1].source,
            Some(GridOptimizedCellSource::SparsePoint)
        );
        assert_eq!(
            readout[2].source,
            Some(GridOptimizedCellSource::RepeatedFormulaRegion { region_index: 0 })
        );

        let mut reference = sheet.project_authored_to_reference(100).unwrap();
        let report = reference.recalculate_mark_all_dirty(|request| {
            assert_eq!(request.formula.normal_form_key, formula.normal_form_key);
            CalcValue::number(f64::from(request.address.row * 10))
        });

        assert_eq!(reference.authored().len(), 3);
        assert_eq!(report.formula_cells, 2);
        assert_eq!(report.literal_cells, 1);
        assert_eq!(report.formula_evaluations, 2);
        assert_eq!(reference.read_cell(&address(2, 2)), CalcValue::number(99.0));
    }

    #[test]
    fn optimized_grid_later_dense_region_overwrites_sparse_point_for_fill_semantics() {
        let mut sheet = optimized_sheet();
        sheet
            .set_literal(address(2, 2), CalcValue::number(1.0))
            .unwrap();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 2, 2, 2, 3, bounds()).unwrap(),
                vec![CalcValue::number(20.0), CalcValue::number(30.0)],
            )
            .unwrap();

        let readout = sheet.sampled_authored_readout([address(2, 2), address(2, 3)]);
        assert_eq!(
            readout[0].authored,
            Some(GridAuthoredCell::Literal(CalcValue::number(20.0)))
        );
        assert_eq!(
            readout[0].source,
            Some(GridOptimizedCellSource::DenseValueRegion { region_index: 0 })
        );
        assert_eq!(
            readout[1].authored,
            Some(GridAuthoredCell::Literal(CalcValue::number(30.0)))
        );
    }

    #[test]
    fn optimized_grid_rejects_mismatched_dense_region_payload() {
        let mut sheet = optimized_sheet();

        assert_eq!(
            sheet.put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 2, 2, bounds()).unwrap(),
                vec![CalcValue::number(1.0)]
            ),
            Err(GridRefError::DenseRegionValueCountMismatch {
                cells: 4,
                values: 1,
            })
        );
    }

    #[test]
    fn optimized_grid_partition_witness_reports_disjoint_compact_regions() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap(),
                vec![
                    CalcValue::number(1.0),
                    CalcValue::number(2.0),
                    CalcValue::number(3.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 2, 3, 2, bounds()).unwrap(),
                GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let witness = sheet.partition_witness_report();

        assert!(witness.p18_partition_witness_holds());
        assert_eq!(witness.sparse_point_cells, 0);
        assert_eq!(witness.dense_value_regions, 1);
        assert_eq!(witness.repeated_formula_regions, 1);
        assert_eq!(witness.max_parallelism_bound, 2);
    }

    #[test]
    fn optimized_grid_partition_witness_detects_same_level_overlap() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap(),
                vec![
                    CalcValue::number(1.0),
                    CalcValue::number(2.0),
                    CalcValue::number(3.0),
                ],
            )
            .unwrap();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 2, 1, 4, 1, bounds()).unwrap(),
                vec![
                    CalcValue::number(20.0),
                    CalcValue::number(30.0),
                    CalcValue::number(40.0),
                ],
            )
            .unwrap();

        let witness = sheet.partition_witness_report();

        assert!(!witness.p18_partition_witness_holds());
        assert_eq!(witness.dense_value_pair_checks, 1);
        assert_eq!(witness.dense_value_overlap_count, 1);
    }

    #[test]
    fn optimized_publication_delta_counts_dense_region_entries_not_cells() {
        let large_bounds = ExcelGridBounds {
            max_rows: 10,
            max_cols: 2,
        };
        let rect =
            GridRect::new("book:default", "sheet:default", 1, 1, 10, 1, large_bounds).unwrap();
        let values = (1..=10).map(f64::from).collect::<Vec<_>>();
        let mut previous =
            GridOptimizedValuation::new("book:default", "sheet:default", large_bounds);
        previous.push_dense_value_payload(
            rect.clone(),
            GridDenseValuePayload::from_numbers(values.clone()),
            1,
            GridOptimizedCellSource::DenseValueRegion { region_index: 0 },
        );

        let mut revision_only =
            GridOptimizedValuation::new("book:default", "sheet:default", large_bounds);
        revision_only.push_dense_value_payload(
            rect.clone(),
            GridDenseValuePayload::from_numbers(values.clone()),
            99,
            GridOptimizedCellSource::DenseValueRegion { region_index: 0 },
        );
        let revision_only_report = revision_only.publication_delta_report_since(&previous);
        assert_eq!(revision_only_report.publication_entries_total(), 0);
        assert_eq!(revision_only_report.dense_region_entries_unchanged, 1);

        let mut changed_values = values;
        changed_values[4] = 500.0;
        let mut current =
            GridOptimizedValuation::new("book:default", "sheet:default", large_bounds);
        current.push_dense_value_payload(
            rect,
            GridDenseValuePayload::from_numbers(changed_values),
            100,
            GridOptimizedCellSource::DenseValueRegion { region_index: 0 },
        );

        let report = current.publication_delta_report_since(&previous);

        assert!(report.same_grid_identity);
        assert_eq!(report.previous_dense_region_entries, 1);
        assert_eq!(report.current_dense_region_entries, 1);
        assert_eq!(report.previous_dense_cells, 10);
        assert_eq!(report.current_dense_cells, 10);
        assert_eq!(report.dense_region_entries_changed, 1);
        assert_eq!(report.dense_region_cells_changed, 10);
        assert_eq!(report.publication_entries_total(), 1);
        assert_eq!(report.naive_current_computed_cell_publication_floor, 10);
        assert_eq!(report.naive_full_grid_publication_floor, 20);
        assert!(
            u64::try_from(report.publication_entries_total()).unwrap()
                < report.naive_current_computed_cell_publication_floor
        );
    }

    #[test]
    fn optimized_tile_snapshot_reports_visible_tile_without_unrelated_sparse_scan() {
        let large_bounds = ExcelGridBounds {
            max_rows: 100,
            max_cols: 20,
        };
        let tile =
            GridRect::new("book:default", "sheet:default", 20, 1, 29, 10, large_bounds).unwrap();
        let mut valuation =
            GridOptimizedValuation::new("book:default", "sheet:default", large_bounds);
        valuation.push_dense_value_payload(
            tile.clone(),
            GridDenseValuePayload::from_numbers((1..=100).map(f64::from).collect()),
            1,
            GridOptimizedCellSource::DenseValueRegion { region_index: 0 },
        );
        valuation
            .insert_sparse_computed_value(
                ExcelGridCellAddress::new("book:default", "sheet:default", 90, 20),
                2,
                CalcValue::number(900.0),
                GridOptimizedCellSource::SparsePoint,
            )
            .unwrap();

        let report = valuation
            .tile_snapshot_report(tile)
            .expect("tile snapshot should be reportable");

        assert_eq!(report.subscribed_cell_count, 100);
        assert_eq!(report.defined_cell_count, 100);
        assert_eq!(report.blank_cell_count, 0);
        assert_eq!(report.dense_value_cells_visited, 100);
        assert_eq!(report.sparse_value_cells_visited, 0);
        assert_eq!(report.compact_regions_intersected, 1);
        assert_eq!(report.estimated_value_payload_bytes, 800);
        assert!(report.p15_tile_streaming_holds(64));
        assert!(report.estimated_frame_bytes < report.full_grid_dense_numeric_bytes_floor);
    }

    #[test]
    fn optimized_visible_first_recalc_projects_only_viewport_upstream_cone() {
        let bounds = ExcelGridBounds {
            max_rows: 1_000,
            max_cols: 10,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", bounds);
        let dense_rect =
            GridRect::new("book:default", "sheet:default", 1, 1, 1_000, 8, bounds).unwrap();
        sheet
            .put_dense_number_region_with(dense_rect, |address| {
                f64::from(address.row) * 1000.0 + f64::from(address.col)
            })
            .unwrap();
        let formula_rect =
            GridRect::new("book:default", "sheet:default", 1, 9, 1_000, 10, bounds).unwrap();
        sheet
            .put_repeated_formula_region(
                formula_rect,
                GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let visible_rect =
            GridRect::new("book:default", "sheet:default", 200, 10, 299, 10, bounds).unwrap();
        let (valuation, report) = sheet
            .recalculate_visible_rect_compact_with_oxfml(visible_rect.clone(), 1_000)
            .expect("visible viewport should be evaluable");
        let visible = valuation
            .tile_snapshot_report(visible_rect)
            .expect("visible viewport should be reportable");

        assert_eq!(report.visible_cell_count, 100);
        assert_eq!(report.visible_upstream_cell_count, 300);
        assert_eq!(report.dense_value_cells_projected, 100);
        assert_eq!(report.repeated_formula_cells_projected, 200);
        assert_eq!(report.cells_evaluated_before_visible_complete, 300);
        assert_eq!(report.formula_evaluations_before_visible_complete, 200);
        assert_eq!(report.full_recalc_occupied_cell_floor, 10_000);
        assert!(report.p16_visible_first_holds());
        assert_eq!(visible.subscribed_cell_count, 100);
        assert_eq!(visible.defined_cell_count, 100);
        assert_eq!(visible.dense_value_cells_visited, 100);
        assert_eq!(visible.sparse_value_cells_visited, 0);
        let row_250 = valuation.read_cell(&ExcelGridCellAddress::new(
            "book:default",
            "sheet:default",
            250,
            10,
        ));
        assert_eq!(number_from_calc_value(&row_250.computed), Some(1_000_032.0));
    }

    #[test]
    fn optimized_grid_spill_blockage_probe_skips_empty_extent_cells() {
        let large_bounds = ExcelGridBounds {
            max_rows: 3_000,
            max_cols: 3,
        };
        let sheet = GridOptimizedSheet::new("book:default", "sheet:default", large_bounds);
        let anchor = ExcelGridCellAddress::new("book:default", "sheet:default", 1, 1);
        let extent = GridRect::new(
            "book:default",
            "sheet:default",
            1,
            1,
            3_000,
            1,
            large_bounds,
        )
        .unwrap();

        let report = sheet
            .optimized_spill_blockage_probe_report(&anchor, &extent)
            .expect("empty spill extent should be probeable");

        assert!(!report.blocked);
        assert_eq!(report.extent_cell_count, 3_000);
        assert_eq!(report.naive_extent_cell_probe_floor, 3_000);
        assert_eq!(report.compact_blocker_probe_count(), 0);
    }

    #[test]
    fn optimized_grid_spill_blockage_probe_finds_sparse_blocker_without_extent_scan() {
        let large_bounds = ExcelGridBounds {
            max_rows: 3_000,
            max_cols: 3,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", large_bounds);
        let anchor = ExcelGridCellAddress::new("book:default", "sheet:default", 1, 1);
        let extent = GridRect::new(
            "book:default",
            "sheet:default",
            1,
            1,
            3_000,
            1,
            large_bounds,
        )
        .unwrap();
        sheet
            .set_literal(
                ExcelGridCellAddress::new("book:default", "sheet:default", 3_000, 1),
                CalcValue::number(99.0),
            )
            .unwrap();

        let report = sheet
            .optimized_spill_blockage_probe_report(&anchor, &extent)
            .expect("sparse blocker should be probeable");

        assert!(report.blocked);
        assert_eq!(report.extent_cell_count, 3_000);
        assert_eq!(report.sparse_point_candidates, 1);
        assert_eq!(report.compact_blocker_probe_count(), 1);
        assert!(
            report.compact_blocker_probe_count() < report.naive_extent_cell_probe_floor as usize
        );
    }

    #[test]
    fn overlay_set_blockage_probe_matches_legacy_probe() {
        // OVL-2 equivalence proof: the unified overlay-set probe produces a
        // byte-identical blockage report to the legacy per-type probe across a
        // corpus exercising every candidate category (table feature, merged,
        // own-spill skip, blocked-formula anchor-containment, unblocked spill,
        // sparse payload, and an unblocked extent).
        use crate::grid::authored::GridFormulaCell;

        let bounds = ExcelGridBounds {
            max_rows: 100,
            max_cols: 12,
        };
        let addr = |row, col| ExcelGridCellAddress::new("book:default", "sheet:default", row, col);
        let rect = |top, left, bottom, right| {
            GridRect::new(
                "book:default",
                "sheet:default",
                top,
                left,
                bottom,
                right,
                bounds,
            )
            .unwrap()
        };

        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", bounds);
        // A table A1:C5 (set_table_overlay also installs a "table-overlay"
        // feature-rendered region over the range).
        sheet
            .set_table_overlay(
                GridTableOverlay::new(
                    "t",
                    "T",
                    rect(1, 1, 5, 3),
                    vec![GridTableColumn::new("t:c", "C", 1, rect(2, 1, 5, 3))],
                )
                .with_header_rect(rect(1, 1, 1, 3)),
            )
            .unwrap();
        // A merged region E1:F2 and a sparse literal at E10.
        sheet.add_merged_region(rect(1, 5, 2, 6)).unwrap();
        sheet
            .set_literal(addr(10, 5), CalcValue::number(1.0))
            .unwrap();
        // A blocked spill anchored at a formula cell J1 (extent J1:J4), and an
        // unblocked spill anchored at H1 (extent H1:H4).
        sheet
            .set_formula(
                addr(1, 10),
                GridFormulaCell::new("=A1", "excel.grid.v1:cell:R[0]C[-9]"),
            )
            .unwrap();
        sheet
            .set_spill_fact(GridSpillFact {
                anchor: addr(1, 10),
                extent: rect(1, 10, 4, 10),
                blocked: true,
            })
            .unwrap();
        sheet
            .set_spill_fact(GridSpillFact {
                anchor: addr(1, 8),
                extent: rect(1, 8, 4, 8),
                blocked: false,
            })
            .unwrap();

        let facts = sheet.spill_facts().clone();
        let probes = [
            (addr(20, 1), rect(20, 1, 24, 3)), // unblocked
            (addr(1, 1), rect(1, 1, 10, 3)),   // table feature region
            (addr(5, 5), rect(1, 5, 5, 6)),    // merged region
            (addr(1, 8), rect(1, 8, 4, 8)),    // own unblocked spill -> skipped
            (addr(2, 10), rect(2, 10, 3, 10)), // inside blocked spill -> pre-pass
            (addr(10, 8), rect(1, 8, 8, 8)),   // unblocked spill blocks
            (addr(8, 5), rect(8, 5, 12, 6)),   // sparse payload at E10
        ];

        let mut saw_blocked = false;
        let mut saw_unblocked = false;
        for (anchor, extent) in &probes {
            let legacy = sheet
                .optimized_spill_blockage_probe_report_with_facts(anchor, extent, &facts)
                .unwrap();
            let unified = sheet
                .overlay_set_blockage_probe(anchor, extent, &facts)
                .unwrap();
            assert_eq!(
                legacy.blocked, unified.blocked,
                "blocked verdict diverged at anchor {anchor:?}"
            );
            assert_eq!(
                legacy.sparse_point_candidates,
                unified.sparse_point_candidates
            );
            assert_eq!(
                legacy.dense_value_region_candidates,
                unified.dense_value_region_candidates
            );
            assert_eq!(
                legacy.repeated_formula_region_candidates,
                unified.repeated_formula_region_candidates
            );
            assert_eq!(
                legacy.merged_region_candidates,
                unified.merged_region_candidates
            );
            assert_eq!(
                legacy.feature_rendered_region_candidates,
                unified.feature_rendered_region_candidates
            );
            assert_eq!(
                legacy.blocked_formula_spill_fact_candidates,
                unified.blocked_formula_spill_fact_candidates
            );
            assert_eq!(
                legacy.unblocked_spill_fact_candidates,
                unified.unblocked_spill_fact_candidates
            );
            assert_eq!(legacy.extent_cell_count, unified.extent_cell_count);
            assert_eq!(
                legacy.naive_extent_cell_probe_floor,
                unified.naive_extent_cell_probe_floor
            );
            saw_blocked |= unified.blocked;
            saw_unblocked |= !unified.blocked;
        }
        assert!(
            saw_blocked && saw_unblocked,
            "the corpus must exercise both blocked and unblocked outcomes"
        );
    }

    #[test]
    fn grid_overlay_methods_forward_to_legacy_per_type_behaviour() {
        let bounds = ExcelGridBounds {
            max_rows: 100,
            max_cols: 12,
        };
        let addr = |row, col| ExcelGridCellAddress::new("book:default", "sheet:default", row, col);
        let rect = |top, left, bottom, right| {
            GridRect::new(
                "book:default",
                "sheet:default",
                top,
                left,
                bottom,
                right,
                bounds,
            )
            .unwrap()
        };
        // Insert 2 rows before row 3: rows 1-2 hold, rows 3+ shift down by 2.
        let edit = GridAxisEdit::insert_rows(3, 2);

        // Table: the range, header, and column bands all transform.
        let table = GridTableOverlay::new(
            "t",
            "T",
            rect(1, 1, 5, 3),
            vec![GridTableColumn::new("t:c", "C", 1, rect(2, 1, 5, 3))],
        )
        .with_header_rect(rect(1, 1, 1, 3));
        let GridOverlay::Table(transformed_table) = GridOverlay::Table(table.clone())
            .transform_for_axis_edit(edit, bounds)
            .unwrap()
            .expect("table survives the insert")
        else {
            panic!("expected a table overlay");
        };
        assert_eq!(transformed_table.table_range, rect(1, 1, 7, 3));
        assert_eq!(transformed_table.header_rect, Some(rect(1, 1, 1, 3)));
        assert_eq!(transformed_table.columns[0].data_rect, rect(2, 1, 7, 3));

        // Merged region wholly above the insertion point is unchanged.
        let GridOverlay::Merged(transformed_merged) = GridOverlay::Merged(GridMergedRegion {
            rect: rect(1, 5, 2, 6),
        })
        .transform_for_axis_edit(edit, bounds)
        .unwrap()
        .expect("merge survives") else {
            panic!("expected a merged overlay");
        };
        assert_eq!(transformed_merged.rect, rect(1, 5, 2, 6));

        // Spill: anchor above the insert holds; the extent grows by the inserted rows.
        let GridOverlay::Spill(transformed_spill) = GridOverlay::Spill(GridSpillFact {
            anchor: addr(1, 8),
            extent: rect(1, 8, 4, 8),
            blocked: false,
        })
        .transform_for_axis_edit(edit, bounds)
        .unwrap()
        .expect("spill survives") else {
            panic!("expected a spill overlay");
        };
        assert_eq!(transformed_spill.anchor, addr(1, 8));
        assert_eq!(transformed_spill.extent, rect(1, 8, 6, 8));
        assert!(!transformed_spill.blocked);

        // admit_axis_edit: a pivot feature refuses an intersecting edit (with the
        // legacy detail string); a table-overlay feature admits it.
        let pivot = GridOverlay::FeatureRendered(FeatureRenderedRegion {
            rect: rect(1, 1, 5, 3),
            feature_kind: "pivot".to_string(),
            needs_refresh: false,
        });
        match pivot.admit_axis_edit(edit).unwrap() {
            EditAdmission::Refuse { detail } => {
                assert!(detail.contains("edit intersects claimed region R1C1:R5C3"));
            }
            EditAdmission::Allow => panic!("a pivot feature must refuse an intersecting edit"),
        }
        let table_feature = GridOverlay::FeatureRendered(FeatureRenderedRegion {
            rect: rect(1, 1, 5, 3),
            feature_kind: "table-overlay".to_string(),
            needs_refresh: false,
        });
        assert_eq!(
            table_feature.admit_axis_edit(edit).unwrap(),
            EditAdmission::Allow
        );

        // Kind discriminants and blocks_spill semantics.
        assert_eq!(
            GridOverlay::Merged(GridMergedRegion {
                rect: rect(1, 5, 2, 6)
            })
            .kind(),
            OverlayKind::Merged
        );
        assert_eq!(GridOverlay::Table(table).blocks_spill(), SpillBlock::None);
        assert_eq!(
            GridOverlay::Merged(GridMergedRegion {
                rect: rect(1, 5, 2, 6)
            })
            .blocks_spill(),
            SpillBlock::Hard
        );
        assert_eq!(
            GridOverlay::Spill(GridSpillFact {
                anchor: addr(1, 8),
                extent: rect(1, 8, 4, 8),
                blocked: true,
            })
            .blocks_spill(),
            SpillBlock::None
        );
    }

    #[test]
    fn apply_axis_edit_unified_loop_preserves_counters_and_fail_fast() {
        let bounds = ExcelGridBounds {
            max_rows: 100,
            max_cols: 12,
        };
        let addr = |row, col| ExcelGridCellAddress::new("book:default", "sheet:default", row, col);
        let rect = |top, left, bottom, right| {
            GridRect::new(
                "book:default",
                "sheet:default",
                top,
                left,
                bottom,
                right,
                bounds,
            )
            .unwrap()
        };
        // A sheet carrying every overlay kind: a table A1:C5 (which also installs
        // a "table-overlay" feature region), a merged region E1:F2, a refusing
        // "pivot" feature H1:J3, and a spill anchored at L1.
        let build_sheet = || {
            let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", bounds);
            sheet
                .set_table_overlay(GridTableOverlay::new(
                    "t",
                    "T",
                    rect(1, 1, 5, 3),
                    vec![GridTableColumn::new("t:c", "C", 1, rect(2, 1, 5, 3))],
                ))
                .unwrap();
            sheet.add_merged_region(rect(1, 5, 2, 6)).unwrap();
            sheet
                .add_feature_rendered_region(rect(1, 8, 3, 10), "pivot", false)
                .unwrap();
            sheet
                .set_spill_fact(GridSpillFact {
                    anchor: addr(1, 12),
                    extent: rect(1, 12, 4, 12),
                    blocked: false,
                })
                .unwrap();
            sheet
        };

        // Insert a row before row 1: every overlay shifts down, and only the pivot
        // (which marks-refresh-on-transform) is counted as marked.
        let mut sheet = build_sheet();
        let report = sheet
            .apply_axis_edit(GridAxisEdit::insert_rows(1, 1))
            .unwrap();
        assert_eq!(report.feature_regions_kept, 2);
        assert_eq!(report.feature_regions_dropped, 0);
        assert_eq!(report.feature_regions_marked_needs_refresh, 1);
        assert_eq!(report.merged_regions_kept, 1);
        assert_eq!(report.merged_regions_dropped, 0);
        assert_eq!(report.spill_facts_kept, 1);
        assert_eq!(report.spill_facts_dropped, 0);
        assert_eq!(sheet.table_overlays().len(), 1);
        assert_eq!(
            sheet.table_overlays().values().next().unwrap().table_range,
            rect(2, 1, 6, 3),
            "the table shifts down a row through the unified loop"
        );

        // An insert before row 2 intersects the pivot (rows 1-3): the edit is
        // refused fail-fast, before any overlay is mutated.
        let mut sheet = build_sheet();
        let before_tables = sheet.table_overlays().clone();
        let before_merged = sheet.merged_regions().to_vec();
        let before_features = sheet.feature_rendered_regions().to_vec();
        let err = sheet
            .apply_axis_edit(GridAxisEdit::insert_rows(2, 1))
            .unwrap_err();
        assert!(matches!(
            err,
            GridRefError::FeatureRenderedRegionEditRefused { .. }
        ));
        assert_eq!(
            sheet.table_overlays(),
            &before_tables,
            "a refused edit must not mutate tables"
        );
        assert_eq!(
            sheet.merged_regions(),
            before_merged.as_slice(),
            "a refused edit must not mutate merged regions"
        );
        assert_eq!(
            sheet.feature_rendered_regions(),
            before_features.as_slice(),
            "a refused edit must not mutate feature regions"
        );
    }

    #[test]
    fn compare_grid_overlay_blockage_detects_spill_fact_divergence() {
        let bounds = ExcelGridBounds {
            max_rows: 100,
            max_cols: 12,
        };
        let addr = |row, col| ExcelGridCellAddress::new("book:default", "sheet:default", row, col);
        let rect = |top, left, bottom, right| {
            GridRect::new(
                "book:default",
                "sheet:default",
                top,
                left,
                bottom,
                right,
                bounds,
            )
            .unwrap()
        };

        // Identical spill facts: the engines agree, no mismatch.
        let shared = GridSpillFact {
            anchor: addr(1, 1),
            extent: rect(1, 1, 4, 1),
            blocked: false,
        };
        assert!(compare_grid_overlay_blockage(&[shared.clone()], &[shared.clone()]).is_empty());

        // Same anchor, divergent blocked flag: one mismatch carrying both facts.
        let reference_fact = GridSpillFact {
            anchor: addr(1, 1),
            extent: rect(1, 1, 4, 1),
            blocked: false,
        };
        let optimized_fact = GridSpillFact {
            anchor: addr(1, 1),
            extent: rect(1, 1, 4, 1),
            blocked: true,
        };
        let mismatches =
            compare_grid_overlay_blockage(&[reference_fact.clone()], &[optimized_fact.clone()]);
        assert_eq!(mismatches.len(), 1);
        assert_eq!(mismatches[0].anchor, addr(1, 1));
        assert_eq!(mismatches[0].reference, Some(reference_fact));
        assert_eq!(mismatches[0].optimized, Some(optimized_fact));

        // Present in only one engine: a mismatch with the other side `None`.
        let only_optimized = GridSpillFact {
            anchor: addr(2, 1),
            extent: rect(2, 1, 3, 1),
            blocked: false,
        };
        let mismatches = compare_grid_overlay_blockage(&[], &[only_optimized.clone()]);
        assert_eq!(mismatches.len(), 1);
        assert_eq!(mismatches[0].reference, None);
        assert_eq!(mismatches[0].optimized, Some(only_optimized));
    }

    #[test]
    fn run_engine_mode_both_carries_empty_overlay_blockage_mismatch_when_engines_agree() {
        use crate::grid::authored::GridFormulaCell;

        let bounds = ExcelGridBounds {
            max_rows: 100,
            max_cols: 12,
        };
        let addr = |row, col| ExcelGridCellAddress::new("book:default", "sheet:default", row, col);
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", bounds);
        sheet
            .set_literal(addr(1, 1), CalcValue::number(7.0))
            .unwrap();
        sheet
            .set_formula(
                addr(1, 2),
                GridFormulaCell::new("=A1*3", "excel.grid.v1:cell:R[0]C[-1]*3"),
            )
            .unwrap();

        let report = sheet
            .run_engine_mode_with_oxfml(GridEngineMode::Both, [addr(1, 1), addr(1, 2)], 1_000_000)
            .unwrap();
        assert!(report.mismatches.is_empty(), "{:?}", report.mismatches);
        assert!(
            report.overlay_blockage_mismatches.is_empty(),
            "the two engines must agree on overlay blockage: {:?}",
            report.overlay_blockage_mismatches
        );
    }

    #[test]
    fn optimized_grid_spill_clear_uses_sparse_index_for_old_extent() {
        let large_bounds = ExcelGridBounds {
            max_rows: 3_000,
            max_cols: 3,
        };
        let anchor = ExcelGridCellAddress::new("book:default", "sheet:default", 1, 1);
        let extent = GridRect::new(
            "book:default",
            "sheet:default",
            1,
            1,
            3_000,
            1,
            large_bounds,
        )
        .unwrap();
        let mut valuation =
            GridOptimizedValuation::new("book:default", "sheet:default", large_bounds);
        valuation
            .set_spill_fact(GridSpillFact {
                anchor: anchor.clone(),
                extent: extent.clone(),
                blocked: false,
            })
            .unwrap();

        for row in [1, 1_500, 3_000] {
            valuation
                .insert_sparse_computed_value(
                    ExcelGridCellAddress::new("book:default", "sheet:default", row, 1),
                    u64::from(row),
                    CalcValue::number(f64::from(row)),
                    GridOptimizedCellSource::SparsePoint,
                )
                .unwrap();
        }
        for row in 1..=10 {
            valuation
                .insert_sparse_computed_value(
                    ExcelGridCellAddress::new("book:default", "sheet:default", row, 2),
                    u64::from(row),
                    CalcValue::number(f64::from(row * 10)),
                    GridOptimizedCellSource::SparsePoint,
                )
                .unwrap();
        }

        let report = valuation
            .clear_formula_output_for_anchor_report(&anchor)
            .expect("valid old spill anchor should be clearable");

        assert!(report.had_spill_fact);
        assert_eq!(report.old_extent, extent);
        assert_eq!(report.old_extent_cell_count, 3_000);
        assert_eq!(report.naive_sparse_value_scan_floor, 13);
        assert_eq!(report.indexed_candidate_count, 3);
        assert_eq!(report.sparse_values_removed, 3);
        assert_eq!(report.dense_value_regions_removed, 0);
        assert_eq!(report.dense_value_cells_removed, 0);
        assert_eq!(valuation.spill_facts().len(), 0);
        assert_eq!(valuation.sparse_computed_cells(), 10);
        assert_eq!(
            valuation.read_cell(&address(1, 1)).computed,
            CalcValue::empty()
        );
        assert_eq!(
            valuation
                .read_cell(&ExcelGridCellAddress::new(
                    "book:default",
                    "sheet:default",
                    10,
                    2,
                ))
                .computed,
            CalcValue::number(100.0)
        );
    }

    #[test]
    fn optimized_grid_spill_clear_removes_dense_old_output_region() {
        let mut valuation = GridOptimizedValuation::new("book:default", "sheet:default", bounds());
        let anchor = address(1, 1);
        let old_extent =
            GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap();
        let unrelated_extent =
            GridRect::new("book:default", "sheet:default", 1, 2, 3, 2, bounds()).unwrap();
        valuation
            .set_spill_fact(GridSpillFact {
                anchor: anchor.clone(),
                extent: old_extent.clone(),
                blocked: false,
            })
            .unwrap();
        valuation.push_dense_value_payload(
            old_extent.clone(),
            GridDenseValuePayload::from_numbers(vec![1.0, 2.0, 3.0]),
            1,
            GridOptimizedCellSource::SparsePoint,
        );
        valuation.push_dense_value_payload(
            unrelated_extent,
            GridDenseValuePayload::from_numbers(vec![10.0, 20.0, 30.0]),
            1,
            GridOptimizedCellSource::SparsePoint,
        );

        let report = valuation
            .clear_formula_output_for_anchor_report(&anchor)
            .expect("valid old dense spill output should be clearable");

        assert!(report.had_spill_fact);
        assert_eq!(report.dense_value_regions_removed, 1);
        assert_eq!(report.dense_value_cells_removed, 3);
        assert_eq!(valuation.dense_value_regions().len(), 1);
        assert_eq!(valuation.dense_computed_cells(), 3);
        assert_eq!(
            valuation.read_cell(&address(2, 1)).computed,
            CalcValue::empty()
        );
        assert_eq!(
            valuation.read_cell(&address(2, 2)).computed,
            CalcValue::number(20.0)
        );
    }

    #[test]
    fn optimized_grid_compact_recalc_keeps_dense_values_region_backed() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region_with(
                GridRect::new("book:default", "sheet:default", 1, 1, 4, 4, bounds()).unwrap(),
                |address| CalcValue::number(f64::from((address.row * 100) + address.col)),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact(100, |_| {
                panic!("dense literal-only compact recalc should not evaluate formulas")
            })
            .unwrap();

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 16);
        assert_eq!(report.literal_cells, 16);
        assert_eq!(report.formula_cells, 0);
        assert_eq!(report.formula_plan_cache_lookups(), 0);
        assert_eq!(report.dense_value_region_cells, 16);
        assert_eq!(report.computed_dense_value_regions, 1);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(valuation.dense_value_regions().len(), 1);
        assert_eq!(
            valuation.read_cell(&address(4, 4)).computed,
            CalcValue::number(404.0)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 4)).source,
            Some(GridOptimizedCellSource::DenseValueRegion { region_index: 0 })
        );
    }

    #[test]
    fn optimized_grid_compact_recalc_prepares_repeated_formula_template_once() {
        let mut sheet = optimized_sheet();
        let formula = GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1);
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 2, 3, 2, bounds()).unwrap(),
                formula.clone(),
            )
            .unwrap();
        sheet
            .set_literal(address(2, 2), CalcValue::number(99.0))
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact(100, |request| {
                assert_eq!(request.formula.normal_form_key, formula.normal_form_key);
                assert_eq!(
                    request.source,
                    GridOptimizedCellSource::RepeatedFormulaRegion { region_index: 0 }
                );
                CalcValue::number(f64::from(request.address.row * 10))
            })
            .unwrap();

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 3);
        assert_eq!(report.formula_cells, 2);
        assert_eq!(report.literal_cells, 1);
        assert_eq!(report.formula_evaluations, 2);
        assert_eq!(report.repeated_formula_region_cells, 2);
        assert_eq!(report.formula_templates_prepared, 1);
        assert_eq!(report.distinct_formula_templates, 1);
        assert_eq!(report.formula_plan_cache_misses, 1);
        assert_eq!(report.formula_plan_cache_hits, 1);
        assert_eq!(report.formula_plan_cache_hit_rate_micros(), 500_000);
        assert_eq!(report.computed_sparse_cells, 3);
        assert_eq!(
            valuation.read_cell(&address(1, 2)).computed,
            CalcValue::number(10.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 2)).computed,
            CalcValue::number(99.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 2)).computed,
            CalcValue::number(30.0)
        );
    }

    #[test]
    fn optimized_grid_compact_recalc_layers_sparse_override_over_dense_region() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 1, 3, bounds()).unwrap(),
                vec![
                    CalcValue::number(1.0),
                    CalcValue::number(2.0),
                    CalcValue::number(3.0),
                ],
            )
            .unwrap();
        sheet
            .set_literal(address(1, 2), CalcValue::number(99.0))
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact(100, |_| {
                panic!("literal-only compact recalc should not evaluate formulas")
            })
            .unwrap();

        assert!(report.p00_primary_exact_once_holds());
        assert_eq!(report.occupied_cells, 3);
        assert_eq!(report.literal_cells, 3);
        assert_eq!(report.dense_value_region_cells, 2);
        assert_eq!(report.sparse_literal_cells, 1);
        assert_eq!(report.computed_dense_value_regions, 1);
        assert_eq!(report.computed_sparse_cells, 1);
        assert_eq!(
            valuation.read_cell(&address(1, 1)).computed,
            CalcValue::number(1.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 1)).source,
            Some(GridOptimizedCellSource::DenseValueRegion { region_index: 0 })
        );
        assert_eq!(
            valuation.read_cell(&address(1, 2)).computed,
            CalcValue::number(99.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 2)).source,
            Some(GridOptimizedCellSource::SparsePoint)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_reads_dense_region_without_projection() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap(),
                vec![
                    CalcValue::number(2.0),
                    CalcValue::number(3.0),
                    CalcValue::number(5.0),
                ],
            )
            .unwrap();
        sheet
            .set_formula(
                address(1, 2),
                GridFormulaCell::new("=SUM(A1:A3)", "excel.grid.v1:sum:R[0]C[-1]:R[2]C[-1]"),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should evaluate A1 ranges through its provider");

        assert!(report.p00_primary_exact_once_holds());
        assert_eq!(report.occupied_cells, 4);
        assert_eq!(report.literal_cells, 3);
        assert_eq!(report.formula_cells, 1);
        assert_eq!(report.computed_dense_value_regions, 1);
        assert_eq!(report.computed_sparse_cells, 1);
        assert_eq!(
            valuation.read_cell(&address(1, 2)).computed,
            CalcValue::number(10.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_publishes_sequence_spill() {
        let mut sheet = optimized_sheet();
        sheet
            .set_formula(
                address(1, 1),
                GridFormulaCell::new("=SEQUENCE(3)", "excel.grid.v1:sequence:R1C1#"),
            )
            .unwrap();
        sheet
            .set_formula(
                address(1, 2),
                GridFormulaCell::new("=SUM(A1#)", "excel.grid.v1:sum-spill:R[0]C[-1]#"),
            )
            .unwrap();

        let report = sheet
            .run_engine_mode_with_oxfml(
                GridEngineMode::Both,
                [address(1, 1), address(2, 1), address(3, 1), address(1, 2)],
                100,
            )
            .expect("SEQUENCE spill should publish and feed A1# in both engines");

        assert!(report.mismatches.is_empty());
        let optimized = report.optimized.as_ref().unwrap();
        assert_eq!(optimized.readout[0].computed, CalcValue::number(1.0));
        assert_eq!(optimized.readout[1].computed, CalcValue::number(2.0));
        assert_eq!(optimized.readout[2].computed, CalcValue::number(3.0));
        assert_eq!(optimized.readout[3].computed, CalcValue::number(6.0));
        assert!(matches!(
            &optimized.recalc,
            GridEngineRecalcReport::Optimized(recalc)
                if recalc.spill_facts_published == 1
                    && recalc.spill_ghost_cells_published == 2
                    && recalc.computed_dense_value_regions == 1
                    && recalc.computed_sparse_cells == 1
        ));
    }

    #[test]
    fn optimized_grid_recalc_and_commit_publishes_spill_state_to_sheet() {
        let mut sheet = optimized_sheet();
        sheet
            .set_formula(
                address(1, 1),
                GridFormulaCell::new("=SEQUENCE(3)", "excel.grid.v1:sequence:R1C1#"),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml_and_commit_spill_publication(100)
            .expect("optimized recalc should commit published spill state");

        assert_eq!(report.recalc.spill_facts_published, 1);
        assert_eq!(valuation.spill_facts().len(), 1);
        assert_eq!(report.spill_commit.previous_spill_fact_entries, 0);
        assert_eq!(report.spill_commit.committed_spill_fact_entries, 1);
        assert_eq!(report.spill_commit.ledger_update.anchors_added, 1);
        assert_eq!(report.spill_commit.committed_epoch_anchors, 1);
        let committed_extent =
            GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap();
        assert_eq!(
            sheet.spill_facts().get(&address(1, 1)).unwrap().extent,
            committed_extent
        );
        assert_eq!(
            sheet
                .spill_epoch_ledger()
                .snapshot_for(&address(1, 1))
                .unwrap()
                .value_epoch,
            1
        );

        let (_, unchanged_report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml_and_commit_spill_publication(100)
            .expect("unchanged optimized spill recalc should preserve committed epoch");

        assert_eq!(
            unchanged_report.spill_commit.ledger_update.epochs_preserved,
            1
        );
        assert_eq!(
            sheet
                .spill_epoch_ledger()
                .snapshot_for(&address(1, 1))
                .unwrap()
                .value_epoch,
            1
        );
    }

    #[test]
    fn optimized_grid_recalc_and_commit_publishes_filter_spill_state_to_sheet() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 2, bounds()).unwrap(),
                vec![
                    CalcValue::number(101.0),
                    CalcValue::number(102.0),
                    CalcValue::number(201.0),
                    CalcValue::number(202.0),
                    CalcValue::number(301.0),
                    CalcValue::number(302.0),
                ],
            )
            .unwrap();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 3, 3, bounds()).unwrap(),
                vec![
                    CalcValue::logical(true),
                    CalcValue::logical(false),
                    CalcValue::logical(true),
                ],
            )
            .unwrap();
        sheet
            .set_formula(
                address(1, 4),
                GridFormulaCell::new(
                    "=FILTER(A1:B3,C1:C3)",
                    "excel.grid.v1:filter-spill:R1C1:R3C2:R1C3:R3C3",
                ),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml_and_commit_spill_publication(100)
            .expect("optimized FILTER recalc should commit its published spill state");

        assert_eq!(report.recalc.spill_facts_published, 1);
        assert_eq!(report.recalc.spill_ghost_cells_published, 3);
        assert_eq!(report.spill_commit.previous_spill_fact_entries, 0);
        assert_eq!(report.spill_commit.committed_spill_fact_entries, 1);
        assert_eq!(report.spill_commit.ledger_update.anchors_added, 1);
        let expected_extent =
            GridRect::new("book:default", "sheet:default", 1, 4, 2, 5, bounds()).unwrap();
        assert_eq!(
            sheet.spill_facts().get(&address(1, 4)).unwrap().extent,
            expected_extent
        );
        assert_eq!(
            sheet
                .spill_epoch_ledger()
                .snapshot_for(&address(1, 4))
                .unwrap()
                .value_epoch,
            1
        );
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(101.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 5)).computed,
            CalcValue::number(102.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 4)).computed,
            CalcValue::number(301.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 5)).computed,
            CalcValue::number(302.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 4)).computed,
            CalcValue::empty()
        );
    }

    #[test]
    fn optimized_grid_recalc_and_commit_updates_filter_spill_epoch_on_shape_change() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 4, 2, bounds()).unwrap(),
                vec![
                    CalcValue::number(101.0),
                    CalcValue::number(102.0),
                    CalcValue::number(201.0),
                    CalcValue::number(202.0),
                    CalcValue::number(301.0),
                    CalcValue::number(302.0),
                    CalcValue::number(401.0),
                    CalcValue::number(402.0),
                ],
            )
            .unwrap();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 4, 3, bounds()).unwrap(),
                vec![
                    CalcValue::logical(true),
                    CalcValue::logical(true),
                    CalcValue::logical(true),
                    CalcValue::logical(true),
                ],
            )
            .unwrap();
        sheet
            .set_formula(
                address(1, 4),
                GridFormulaCell::new(
                    "=FILTER(A1:B4,C1:C4)",
                    "excel.grid.v1:filter-spill-lifecycle:R1C1:R4C2:R1C3:R4C3",
                ),
            )
            .unwrap();

        let (first_valuation, first_report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml_and_commit_spill_publication(100)
            .expect("initial optimized FILTER recalc should commit the full spill shape");

        let full_extent =
            GridRect::new("book:default", "sheet:default", 1, 4, 4, 5, bounds()).unwrap();
        assert_eq!(first_report.spill_commit.previous_spill_fact_entries, 0);
        assert_eq!(first_report.spill_commit.committed_spill_fact_entries, 1);
        assert_eq!(first_report.spill_commit.ledger_update.anchors_added, 1);
        assert_eq!(first_report.spill_commit.ledger_update.anchors_changed, 1);
        assert_eq!(
            sheet.spill_facts().get(&address(1, 4)).unwrap().extent,
            full_extent
        );
        assert_eq!(
            sheet
                .spill_epoch_ledger()
                .snapshot_for(&address(1, 4))
                .unwrap()
                .value_epoch,
            1
        );
        assert_eq!(
            first_valuation.read_cell(&address(4, 5)).computed,
            CalcValue::number(402.0)
        );

        sheet
            .set_literal(address(2, 3), CalcValue::logical(false))
            .unwrap();
        sheet
            .set_literal(address(4, 3), CalcValue::logical(false))
            .unwrap();

        let (second_valuation, second_report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml_and_commit_spill_publication(100)
            .expect("changed FILTER mask should commit the shrunken spill shape");

        let shrunken_extent =
            GridRect::new("book:default", "sheet:default", 1, 4, 2, 5, bounds()).unwrap();
        assert_eq!(second_report.spill_commit.previous_spill_fact_entries, 1);
        assert_eq!(second_report.spill_commit.committed_spill_fact_entries, 1);
        assert_eq!(second_report.spill_commit.ledger_update.anchors_added, 0);
        assert_eq!(second_report.spill_commit.ledger_update.anchors_changed, 1);
        assert_eq!(
            second_report
                .spill_commit
                .ledger_update
                .extent_changed_anchors,
            1
        );
        assert_eq!(
            second_report
                .spill_commit
                .ledger_update
                .value_changed_anchors,
            1
        );
        assert_eq!(
            second_report
                .spill_commit
                .ledger_update
                .blocked_changed_anchors,
            0
        );
        assert_eq!(
            sheet.spill_facts().get(&address(1, 4)).unwrap().extent,
            shrunken_extent
        );
        assert_eq!(
            sheet
                .spill_epoch_ledger()
                .snapshot_for(&address(1, 4))
                .unwrap()
                .value_epoch,
            2
        );
        assert_eq!(
            second_valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(101.0)
        );
        assert_eq!(
            second_valuation.read_cell(&address(1, 5)).computed,
            CalcValue::number(102.0)
        );
        assert_eq!(
            second_valuation.read_cell(&address(2, 4)).computed,
            CalcValue::number(301.0)
        );
        assert_eq!(
            second_valuation.read_cell(&address(2, 5)).computed,
            CalcValue::number(302.0)
        );
        assert_eq!(
            second_valuation.read_cell(&address(3, 4)).computed,
            CalcValue::empty()
        );
    }

    #[test]
    fn optimized_grid_recalc_and_commit_publishes_column_filter_spill_state_to_sheet() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 2, 3, bounds()).unwrap(),
                vec![
                    CalcValue::number(101.0),
                    CalcValue::number(102.0),
                    CalcValue::number(103.0),
                    CalcValue::number(201.0),
                    CalcValue::number(202.0),
                    CalcValue::number(203.0),
                ],
            )
            .unwrap();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 3, 1, 3, 3, bounds()).unwrap(),
                vec![
                    CalcValue::logical(true),
                    CalcValue::logical(false),
                    CalcValue::logical(true),
                ],
            )
            .unwrap();
        sheet
            .set_formula(
                address(1, 4),
                GridFormulaCell::new(
                    "=FILTER(A1:C2,A3:C3)",
                    "excel.grid.v1:filter-spill-columns:R1C1:R2C3:R3C1:R3C3",
                ),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml_and_commit_spill_publication(100)
            .expect("optimized column FILTER recalc should commit its published spill state");

        assert_eq!(report.recalc.spill_facts_published, 1);
        assert_eq!(report.recalc.spill_ghost_cells_published, 3);
        assert_eq!(report.spill_commit.committed_spill_fact_entries, 1);
        assert_eq!(report.spill_commit.ledger_update.anchors_added, 1);
        let expected_extent =
            GridRect::new("book:default", "sheet:default", 1, 4, 2, 5, bounds()).unwrap();
        assert_eq!(
            sheet.spill_facts().get(&address(1, 4)).unwrap().extent,
            expected_extent
        );
        assert_eq!(
            sheet
                .spill_epoch_ledger()
                .snapshot_for(&address(1, 4))
                .unwrap()
                .value_epoch,
            1
        );
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(101.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 5)).computed,
            CalcValue::number(103.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 4)).computed,
            CalcValue::number(201.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 5)).computed,
            CalcValue::number(203.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_repairs_out_of_order_sequence_spill_consumer() {
        let mut sheet = optimized_sheet();
        sheet
            .set_formula(
                address(1, 1),
                GridFormulaCell::new("=SUM(B1#)", "excel.grid.v1:sum-late-spill:RC[1]#"),
            )
            .unwrap();
        sheet
            .set_formula(
                address(1, 2),
                GridFormulaCell::new("=SEQUENCE(3)", "excel.grid.v1:sequence:R1C2#"),
            )
            .unwrap();

        let report = sheet
            .run_engine_mode_with_oxfml(
                GridEngineMode::Both,
                [address(1, 1), address(1, 2), address(2, 2), address(3, 2)],
                100,
            )
            .expect("late B1# publication should repair its earlier consumer");

        assert!(report.mismatches.is_empty());
        let reference = report.reference.as_ref().unwrap();
        assert_eq!(reference.readout[0].computed, CalcValue::number(6.0));
        assert_eq!(reference.readout[1].computed, CalcValue::number(1.0));
        assert_eq!(reference.readout[2].computed, CalcValue::number(2.0));
        assert_eq!(reference.readout[3].computed, CalcValue::number(3.0));
        assert!(matches!(
            &reference.recalc,
            GridEngineRecalcReport::Reference(recalc)
                if recalc.p00_non_spill_exact_once_holds()
                    && recalc.spill_repair_passes == 1
                    && recalc.spill_repair_formula_evaluations == 2
                    && recalc.spill_repair_converged
                    && recalc.spill_facts_published == 1
                    && recalc.spill_ghost_cells_published == 2
        ));

        let optimized = report.optimized.as_ref().unwrap();
        assert_eq!(optimized.readout[0].computed, CalcValue::number(6.0));
        assert_eq!(optimized.readout[1].computed, CalcValue::number(1.0));
        assert_eq!(optimized.readout[2].computed, CalcValue::number(2.0));
        assert_eq!(optimized.readout[3].computed, CalcValue::number(3.0));
        assert!(matches!(
            &optimized.recalc,
            GridEngineRecalcReport::Optimized(recalc)
                if recalc.p00_primary_exact_once_holds()
                    && recalc.spill_repair_passes == 1
                    && recalc.spill_repair_formula_evaluations == 2
                    && recalc.spill_repair_converged
                    && recalc.spill_facts_published == 1
                    && recalc.spill_ghost_cells_published == 2
                    && recalc.computed_dense_value_regions == 1
                    && recalc.computed_sparse_cells == 1
        ));
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_blocks_sequence_spill_on_table_overlay() {
        let mut sheet = optimized_sheet();
        sheet
            .set_formula(
                address(1, 1),
                GridFormulaCell::new("=SEQUENCE(3)", "excel.grid.v1:sequence:R1C1#"),
            )
            .unwrap();
        sheet
            .add_feature_rendered_region(
                GridRect::new("book:default", "sheet:default", 2, 1, 3, 1, bounds()).unwrap(),
                "table-overlay",
                false,
            )
            .unwrap();

        let report = sheet
            .run_engine_mode_with_oxfml(
                GridEngineMode::Both,
                [address(1, 1), address(2, 1), address(3, 1)],
                100,
            )
            .expect("table overlays should block spilling arrays in both engines");

        assert!(report.mismatches.is_empty());
        let optimized = report.optimized.as_ref().unwrap();
        assert_eq!(
            optimized.readout[0].computed,
            CalcValue::error(WorksheetErrorCode::Spill)
        );
        assert_eq!(optimized.readout[1].computed, CalcValue::empty());
        assert_eq!(optimized.readout[2].computed, CalcValue::empty());
        assert!(matches!(
            &optimized.recalc,
            GridEngineRecalcReport::Optimized(recalc)
                if recalc.spill_facts_blocked == 1
                    && recalc.spill_facts_published == 0
                    && recalc.spill_ghost_cells_published == 0
                    && recalc.spill_repair_passes == 0
        ));

        let reference = report.reference.as_ref().unwrap();
        assert_eq!(
            reference.readout[0].computed,
            CalcValue::error(WorksheetErrorCode::Spill)
        );
        assert!(matches!(
            &reference.recalc,
            GridEngineRecalcReport::Reference(recalc)
                if recalc.spill_facts_blocked == 1
                    && recalc.spill_facts_published == 0
                    && recalc.spill_ghost_cells_published == 0
                    && recalc.spill_repair_passes == 0
        ));
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_blocks_sequence_spill_on_merged_region() {
        let mut sheet = optimized_sheet();
        sheet
            .set_formula(
                address(1, 1),
                GridFormulaCell::new("=SEQUENCE(3)", "excel.grid.v1:sequence:R1C1#"),
            )
            .unwrap();
        sheet
            .add_merged_region(
                GridRect::new("book:default", "sheet:default", 2, 1, 3, 1, bounds()).unwrap(),
            )
            .unwrap();

        let report = sheet
            .run_engine_mode_with_oxfml(
                GridEngineMode::Both,
                [address(1, 1), address(2, 1), address(3, 1)],
                100,
            )
            .expect("merged regions should block spilling arrays in both engines");

        assert!(report.mismatches.is_empty());
        let optimized = report.optimized.as_ref().unwrap();
        assert_eq!(
            optimized.readout[0].computed,
            CalcValue::error(WorksheetErrorCode::Spill)
        );
        assert_eq!(optimized.readout[1].computed, CalcValue::empty());
        assert_eq!(optimized.readout[2].computed, CalcValue::empty());
        assert!(matches!(
            &optimized.recalc,
            GridEngineRecalcReport::Optimized(recalc)
                if recalc.spill_facts_blocked == 1
                    && recalc.spill_facts_published == 0
                    && recalc.spill_ghost_cells_published == 0
                    && recalc.spill_repair_passes == 0
        ));

        let reference = report.reference.as_ref().unwrap();
        assert_eq!(
            reference.readout[0].computed,
            CalcValue::error(WorksheetErrorCode::Spill)
        );
        assert!(matches!(
            &reference.recalc,
            GridEngineRecalcReport::Reference(recalc)
                if recalc.spill_facts_blocked == 1
                    && recalc.spill_facts_published == 0
                    && recalc.spill_ghost_cells_published == 0
                    && recalc.spill_repair_passes == 0
        ));
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_blocks_mutual_sequence_spill_anchors() {
        let mut sheet = optimized_sheet();
        sheet
            .set_formula(
                address(1, 1),
                GridFormulaCell::new("=SEQUENCE(3)", "excel.grid.v1:sequence:R1C1#"),
            )
            .unwrap();
        sheet
            .set_formula(
                address(2, 1),
                GridFormulaCell::new("=SEQUENCE(3)", "excel.grid.v1:sequence:R2C1#"),
            )
            .unwrap();

        let report = sheet
            .run_engine_mode_with_oxfml(
                GridEngineMode::Both,
                [address(1, 1), address(2, 1), address(3, 1), address(4, 1)],
                100,
            )
            .expect("neighboring sequence spill anchors should mutually block");

        assert!(report.mismatches.is_empty());
        let optimized = report.optimized.as_ref().unwrap();
        assert_eq!(
            optimized.readout[0].computed,
            CalcValue::error(WorksheetErrorCode::Spill)
        );
        assert_eq!(
            optimized.readout[1].computed,
            CalcValue::error(WorksheetErrorCode::Spill)
        );
        assert_eq!(optimized.readout[2].computed, CalcValue::empty());
        assert_eq!(optimized.readout[3].computed, CalcValue::empty());
        assert!(matches!(
            &optimized.recalc,
            GridEngineRecalcReport::Optimized(recalc)
                if recalc.spill_facts_blocked == 2
                    && recalc.spill_facts_published == 0
                    && recalc.spill_ghost_cells_published == 0
                    && recalc.spill_repair_passes == 0
        ));

        let reference = report.reference.as_ref().unwrap();
        assert_eq!(
            reference.readout[0].computed,
            CalcValue::error(WorksheetErrorCode::Spill)
        );
        assert_eq!(
            reference.readout[1].computed,
            CalcValue::error(WorksheetErrorCode::Spill)
        );
        assert!(matches!(
            &reference.recalc,
            GridEngineRecalcReport::Reference(recalc)
                if recalc.spill_facts_blocked == 2
                    && recalc.spill_facts_published == 0
                    && recalc.spill_ghost_cells_published == 0
                    && recalc.spill_repair_passes == 0
        ));
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_repeated_r1c1_against_dense_region() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap(),
                vec![
                    CalcValue::number(10.0),
                    CalcValue::number(20.0),
                    CalcValue::number(30.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 2, 3, 2, bounds()).unwrap(),
                GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should evaluate R1C1 formulas through its provider");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 6);
        assert_eq!(report.literal_cells, 3);
        assert_eq!(report.formula_cells, 3);
        assert_eq!(report.formula_templates_prepared, 1);
        assert_eq!(report.formula_plan_cache_misses, 1);
        assert_eq!(report.formula_plan_cache_hits, 2);
        assert_eq!(report.computed_dense_value_regions, 2);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 2)).computed,
            CalcValue::number(20.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 2)).computed,
            CalcValue::number(40.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 2)).computed,
            CalcValue::number(60.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_direct_scalar_r1c1_templates() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap(),
                vec![
                    CalcValue::number(10.0),
                    CalcValue::number(20.0),
                    CalcValue::number(30.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 2, 3, 2, bounds()).unwrap(),
                GridFormulaCell::new("=RC[-1]", "excel.grid.v1:r1c1-template:RC[-1]")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 3, 3, bounds()).unwrap(),
                GridFormulaCell::new("=(RC[-2])", "excel.grid.v1:r1c1-template:(RC[-2])")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile direct scalar R1C1 templates");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 9);
        assert_eq!(report.literal_cells, 3);
        assert_eq!(report.formula_cells, 6);
        assert_eq!(report.formula_templates_prepared, 2);
        assert_eq!(report.compiled_formula_plan_cache_misses, 2);
        assert_eq!(report.compiled_formula_plans_cached, 2);
        assert_eq!(report.computed_dense_value_regions, 3);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 2)).computed,
            CalcValue::number(10.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 2)).computed,
            CalcValue::number(30.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 3)).computed,
            CalcValue::number(10.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 3)).computed,
            CalcValue::number(30.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_unary_minus_r1c1_templates() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 2, bounds()).unwrap(),
                vec![
                    CalcValue::number(10.0),
                    CalcValue::number(1.0),
                    CalcValue::number(20.0),
                    CalcValue::number(-2.0),
                    CalcValue::number(30.0),
                    CalcValue::number(3.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 3, 3, bounds()).unwrap(),
                GridFormulaCell::new("=-RC[-2]", "excel.grid.v1:r1c1-template:-RC[-2]")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 3, 4, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=-(RC[-3]+RC[-2])",
                    "excel.grid.v1:r1c1-template:-(RC[-3]+RC[-2])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 5, 3, 5, bounds()).unwrap(),
                GridFormulaCell::new("=-RC[-4]*2+1", "excel.grid.v1:r1c1-template:-RC[-4]*2+1")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile unary-minus R1C1 templates");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 15);
        assert_eq!(report.literal_cells, 6);
        assert_eq!(report.formula_cells, 9);
        assert_eq!(report.formula_templates_prepared, 3);
        assert_eq!(report.compiled_formula_plan_cache_misses, 3);
        assert_eq!(report.compiled_formula_plans_cached, 3);
        assert_eq!(report.computed_dense_value_regions, 4);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 3)).computed,
            CalcValue::number(-10.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 3)).computed,
            CalcValue::number(-30.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(-11.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 4)).computed,
            CalcValue::number(-18.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 4)).computed,
            CalcValue::number(-33.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 5)).computed,
            CalcValue::number(-19.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 5)).computed,
            CalcValue::number(-59.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_r1c1_argument_aggregate_templates() {
        let wide_bounds = ExcelGridBounds {
            max_rows: 10,
            max_cols: 9,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", wide_bounds);
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 2, wide_bounds).unwrap(),
                vec![
                    CalcValue::number(10.0),
                    CalcValue::number(1.0),
                    CalcValue::number(20.0),
                    CalcValue::number(2.0),
                    CalcValue::number(30.0),
                    CalcValue::number(3.0),
                ],
            )
            .unwrap();
        for (col, source, normal_form) in [
            (
                3,
                "=SUM(RC[-2],RC[-1],5)",
                "excel.grid.v1:r1c1-template:SUM(RC[-2],RC[-1],5)",
            ),
            (
                4,
                "=COUNT(RC[-3],RC[-2],5)",
                "excel.grid.v1:r1c1-template:COUNT(RC[-3],RC[-2],5)",
            ),
            (
                5,
                "=PRODUCT(RC[-4],RC[-3],2)",
                "excel.grid.v1:r1c1-template:PRODUCT(RC[-4],RC[-3],2)",
            ),
            (
                6,
                "=AVERAGE(RC[-5],RC[-4],5)",
                "excel.grid.v1:r1c1-template:AVERAGE(RC[-5],RC[-4],5)",
            ),
            (
                7,
                "=MIN(RC[-6],RC[-5],5)",
                "excel.grid.v1:r1c1-template:MIN(RC[-6],RC[-5],5)",
            ),
            (
                8,
                "=MAX(RC[-7],RC[-6],5)",
                "excel.grid.v1:r1c1-template:MAX(RC[-7],RC[-6],5)",
            ),
            (
                9,
                "=SUMSQ(RC[-8],RC[-7],5)",
                "excel.grid.v1:r1c1-template:SUMSQ(RC[-8],RC[-7],5)",
            ),
        ] {
            sheet
                .put_repeated_formula_region(
                    GridRect::new("book:default", "sheet:default", 1, col, 3, col, wide_bounds)
                        .unwrap(),
                    GridFormulaCell::new(source, normal_form)
                        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
                )
                .unwrap();
        }

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile R1C1 argument aggregate templates");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 27);
        assert_eq!(report.literal_cells, 6);
        assert_eq!(report.formula_cells, 21);
        assert_eq!(report.formula_templates_prepared, 7);
        assert_eq!(report.compiled_formula_plan_cache_misses, 7);
        assert_eq!(report.compiled_formula_plans_cached, 7);
        assert_eq!(report.computed_dense_value_regions, 8);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 3)).computed,
            CalcValue::number(16.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 3)).computed,
            CalcValue::number(38.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 4)).computed,
            CalcValue::number(3.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 5)).computed,
            CalcValue::number(180.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 6)).computed,
            CalcValue::number((10.0 + 1.0 + 5.0) / 3.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 7)).computed,
            CalcValue::number(3.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 8)).computed,
            CalcValue::number(20.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 9)).computed,
            CalcValue::number(126.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 9)).computed,
            CalcValue::number(934.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_r1c1_scalar_function_templates() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 2, bounds()).unwrap(),
                vec![
                    CalcValue::number(-3.0),
                    CalcValue::number(4.0),
                    CalcValue::number(0.0),
                    CalcValue::number(9.0),
                    CalcValue::number(5.0),
                    CalcValue::number(-1.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 3, 3, bounds()).unwrap(),
                GridFormulaCell::new("=ABS(RC[-2])", "excel.grid.v1:r1c1-template:ABS(RC[-2])")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 3, 4, bounds()).unwrap(),
                GridFormulaCell::new("=SQRT(RC[-2])", "excel.grid.v1:r1c1-template:SQRT(RC[-2])")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 5, 3, 5, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=POWER(ABS(RC[-4]),2)",
                    "excel.grid.v1:r1c1-template:POWER(ABS(RC[-4]),2)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile R1C1 scalar function templates");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 15);
        assert_eq!(report.literal_cells, 6);
        assert_eq!(report.formula_cells, 9);
        assert_eq!(report.formula_templates_prepared, 3);
        assert_eq!(report.compiled_formula_plan_cache_misses, 3);
        assert_eq!(report.compiled_formula_plans_cached, 3);
        assert_eq!(report.computed_dense_value_regions, 4);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 3)).computed,
            CalcValue::number(3.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 3)).computed,
            CalcValue::number(5.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(2.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 4)).computed,
            CalcValue::number(3.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 4)).computed,
            CalcValue::error(WorksheetErrorCode::Num)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 5)).computed,
            CalcValue::number(9.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 5)).computed,
            CalcValue::number(25.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_r1c1_reference_function_templates() {
        let wide_bounds = ExcelGridBounds {
            max_rows: 10,
            max_cols: 7,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", wide_bounds);
        for (col, source, normal_form) in [
            (1, "=ROW()", "excel.grid.v1:r1c1-template:ROW()"),
            (2, "=COLUMN()", "excel.grid.v1:r1c1-template:COLUMN()"),
            (
                3,
                "=ROW(R[2]C[-2])",
                "excel.grid.v1:r1c1-template:ROW(R[2]C[-2])",
            ),
            (
                4,
                "=COLUMN(RC[-3])",
                "excel.grid.v1:r1c1-template:COLUMN(RC[-3])",
            ),
            (
                5,
                "=ROWS(R1C1:R3C1)",
                "excel.grid.v1:r1c1-template:ROWS(R1C1:R3C1)",
            ),
            (
                6,
                "=COLUMNS(RC[-5]:RC[-3])",
                "excel.grid.v1:r1c1-template:COLUMNS(RC[-5]:RC[-3])",
            ),
            (
                7,
                "=ROW(RC[-6])+COLUMN(RC[-6])",
                "excel.grid.v1:r1c1-template:ROW(RC[-6])+COLUMN(RC[-6])",
            ),
        ] {
            sheet
                .put_repeated_formula_region(
                    GridRect::new("book:default", "sheet:default", 1, col, 3, col, wide_bounds)
                        .unwrap(),
                    GridFormulaCell::new(source, normal_form)
                        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
                )
                .unwrap();
        }

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile R1C1 reference function templates");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 21);
        assert_eq!(report.literal_cells, 0);
        assert_eq!(report.formula_cells, 21);
        assert_eq!(report.formula_templates_prepared, 7);
        assert_eq!(report.compiled_formula_plan_cache_misses, 7);
        assert_eq!(report.compiled_formula_plans_cached, 7);
        assert_eq!(report.computed_dense_value_regions, 7);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 1)).computed,
            CalcValue::number(1.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 1)).computed,
            CalcValue::number(3.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 2)).computed,
            CalcValue::number(2.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 3)).computed,
            CalcValue::number(5.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 4)).computed,
            CalcValue::number(1.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 5)).computed,
            CalcValue::number(3.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 6)).computed,
            CalcValue::number(3.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 7)).computed,
            CalcValue::number(4.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_r1c1_mod_function_templates() {
        let test_bounds = ExcelGridBounds {
            max_rows: 10,
            max_cols: 5,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", test_bounds);
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 4, 2, test_bounds).unwrap(),
                vec![
                    CalcValue::number(10.0),
                    CalcValue::number(3.0),
                    CalcValue::number(11.0),
                    CalcValue::number(2.0),
                    CalcValue::number(-10.0),
                    CalcValue::number(3.0),
                    CalcValue::number(10.0),
                    CalcValue::number(0.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 4, 3, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=MOD(RC[-2],RC[-1])",
                    "excel.grid.v1:r1c1-template:MOD(RC[-2],RC[-1])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 4, 4, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=IF(MOD(RC[-3],2)=0,RC[-3]/2,RC[-3]*3)",
                    "excel.grid.v1:r1c1-template:IF(MOD(RC[-3],2)=0,RC[-3]/2,RC[-3]*3)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 5, 4, 5, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=MOD(POWER(RC[-4],2),RC[-3])",
                    "excel.grid.v1:r1c1-template:MOD(POWER(RC[-4],2),RC[-3])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile R1C1 MOD function templates");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 20);
        assert_eq!(report.literal_cells, 8);
        assert_eq!(report.formula_cells, 12);
        assert_eq!(report.formula_templates_prepared, 3);
        assert_eq!(report.compiled_formula_plan_cache_misses, 3);
        assert_eq!(report.compiled_formula_plans_cached, 3);
        assert_eq!(report.computed_dense_value_regions, 4);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 3)).computed,
            CalcValue::number(1.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 3)).computed,
            CalcValue::number(1.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 3)).computed,
            CalcValue::number(2.0)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 3)).computed,
            CalcValue::error(WorksheetErrorCode::Div0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(5.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 4)).computed,
            CalcValue::number(33.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 4)).computed,
            CalcValue::number(-5.0)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 4)).computed,
            CalcValue::number(5.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 5)).computed,
            CalcValue::number(1.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 5)).computed,
            CalcValue::number(1.0)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 5)).computed,
            CalcValue::error(WorksheetErrorCode::Div0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_r1c1_rounding_function_templates() {
        let test_bounds = ExcelGridBounds {
            max_rows: 10,
            max_cols: 5,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", test_bounds);
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 4, 2, test_bounds).unwrap(),
                vec![
                    CalcValue::number(1.5),
                    CalcValue::number(0.0),
                    CalcValue::number(-1.5),
                    CalcValue::number(0.0),
                    CalcValue::number(125.0),
                    CalcValue::number(-1.0),
                    CalcValue::number(-125.0),
                    CalcValue::number(-1.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 4, 3, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=ROUND(RC[-2],RC[-1])",
                    "excel.grid.v1:r1c1-template:ROUND(RC[-2],RC[-1])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 4, 4, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=ROUNDUP(RC[-3],RC[-2])",
                    "excel.grid.v1:r1c1-template:ROUNDUP(RC[-3],RC[-2])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 5, 4, 5, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=ROUNDDOWN(RC[-4],RC[-3])",
                    "excel.grid.v1:r1c1-template:ROUNDDOWN(RC[-4],RC[-3])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile R1C1 ROUND-family templates");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 20);
        assert_eq!(report.literal_cells, 8);
        assert_eq!(report.formula_cells, 12);
        assert_eq!(report.formula_templates_prepared, 3);
        assert_eq!(report.compiled_formula_plan_cache_misses, 3);
        assert_eq!(report.compiled_formula_plans_cached, 3);
        assert_eq!(report.computed_dense_value_regions, 4);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 3)).computed,
            CalcValue::number(2.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 3)).computed,
            CalcValue::number(-2.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 3)).computed,
            CalcValue::number(130.0)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 3)).computed,
            CalcValue::number(-130.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(2.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 4)).computed,
            CalcValue::number(-2.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 4)).computed,
            CalcValue::number(130.0)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 4)).computed,
            CalcValue::number(-130.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 5)).computed,
            CalcValue::number(1.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 5)).computed,
            CalcValue::number(-1.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 5)).computed,
            CalcValue::number(120.0)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 5)).computed,
            CalcValue::number(-120.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_r1c1_integer_function_templates() {
        let test_bounds = ExcelGridBounds {
            max_rows: 10,
            max_cols: 5,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", test_bounds);
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 4, 2, test_bounds).unwrap(),
                vec![
                    CalcValue::number(1.9),
                    CalcValue::number(0.0),
                    CalcValue::number(-1.9),
                    CalcValue::number(0.0),
                    CalcValue::number(125.9),
                    CalcValue::number(-1.0),
                    CalcValue::number(-125.9),
                    CalcValue::number(-1.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 4, 3, test_bounds).unwrap(),
                GridFormulaCell::new("=INT(RC[-2])", "excel.grid.v1:r1c1-template:INT(RC[-2])")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 4, 4, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=TRUNC(RC[-3])",
                    "excel.grid.v1:r1c1-template:TRUNC(RC[-3])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 5, 4, 5, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=TRUNC(RC[-4],RC[-3])",
                    "excel.grid.v1:r1c1-template:TRUNC(RC[-4],RC[-3])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile R1C1 INT/TRUNC templates");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 20);
        assert_eq!(report.literal_cells, 8);
        assert_eq!(report.formula_cells, 12);
        assert_eq!(report.formula_templates_prepared, 3);
        assert_eq!(report.compiled_formula_plan_cache_misses, 3);
        assert_eq!(report.compiled_formula_plans_cached, 3);
        assert_eq!(report.computed_dense_value_regions, 4);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 3)).computed,
            CalcValue::number(1.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 3)).computed,
            CalcValue::number(-2.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 3)).computed,
            CalcValue::number(125.0)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 3)).computed,
            CalcValue::number(-126.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(1.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 4)).computed,
            CalcValue::number(-1.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 4)).computed,
            CalcValue::number(125.0)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 4)).computed,
            CalcValue::number(-125.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 5)).computed,
            CalcValue::number(1.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 5)).computed,
            CalcValue::number(-1.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 5)).computed,
            CalcValue::number(120.0)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 5)).computed,
            CalcValue::number(-120.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_r1c1_log_function_templates() {
        let test_bounds = ExcelGridBounds {
            max_rows: 10,
            max_cols: 6,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", test_bounds);
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 4, 2, test_bounds).unwrap(),
                vec![
                    CalcValue::number(1.0),
                    CalcValue::number(0.0),
                    CalcValue::number(100.0),
                    CalcValue::number(10.0),
                    CalcValue::number(8.0),
                    CalcValue::number(2.0),
                    CalcValue::number(-1.0),
                    CalcValue::number(1.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 4, 3, test_bounds).unwrap(),
                GridFormulaCell::new("=EXP(RC[-1])", "excel.grid.v1:r1c1-template:EXP(RC[-1])")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 4, 4, test_bounds).unwrap(),
                GridFormulaCell::new("=LN(RC[-3])", "excel.grid.v1:r1c1-template:LN(RC[-3])")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 5, 4, 5, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=LOG10(RC[-4])",
                    "excel.grid.v1:r1c1-template:LOG10(RC[-4])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 6, 4, 6, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=LOG(RC[-5],RC[-4])",
                    "excel.grid.v1:r1c1-template:LOG(RC[-5],RC[-4])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile R1C1 EXP/LN/LOG templates");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 24);
        assert_eq!(report.literal_cells, 8);
        assert_eq!(report.formula_cells, 16);
        assert_eq!(report.formula_templates_prepared, 4);
        assert_eq!(report.compiled_formula_plan_cache_misses, 4);
        assert_eq!(report.compiled_formula_plans_cached, 4);
        assert_eq!(report.computed_dense_value_regions, 5);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 3)).computed,
            CalcValue::number(1.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(0.0)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 4)).computed,
            CalcValue::error(WorksheetErrorCode::Num)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 5)).computed,
            CalcValue::number(2.0)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 5)).computed,
            CalcValue::error(WorksheetErrorCode::Num)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 6)).computed,
            CalcValue::error(WorksheetErrorCode::Num)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 6)).computed,
            CalcValue::number(2.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 6)).computed,
            CalcValue::number(3.0)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 6)).computed,
            CalcValue::error(WorksheetErrorCode::Num)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_r1c1_trig_function_templates() {
        let test_bounds = ExcelGridBounds {
            max_rows: 10,
            max_cols: 4,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", test_bounds);
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, test_bounds).unwrap(),
                vec![
                    CalcValue::number(0.0),
                    CalcValue::number(std::f64::consts::FRAC_PI_6),
                    CalcValue::number(std::f64::consts::FRAC_PI_4),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 2, 3, 2, test_bounds).unwrap(),
                GridFormulaCell::new("=SIN(RC[-1])", "excel.grid.v1:r1c1-template:SIN(RC[-1])")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 3, 3, test_bounds).unwrap(),
                GridFormulaCell::new("=COS(RC[-2])", "excel.grid.v1:r1c1-template:COS(RC[-2])")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 3, 4, test_bounds).unwrap(),
                GridFormulaCell::new("=TAN(RC[-3])", "excel.grid.v1:r1c1-template:TAN(RC[-3])")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile R1C1 SIN/COS/TAN templates");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 12);
        assert_eq!(report.literal_cells, 3);
        assert_eq!(report.formula_cells, 9);
        assert_eq!(report.formula_templates_prepared, 3);
        assert_eq!(report.compiled_formula_plan_cache_misses, 3);
        assert_eq!(report.compiled_formula_plans_cached, 3);
        assert_eq!(report.computed_dense_value_regions, 4);
        assert_eq!(report.computed_sparse_cells, 0);

        let assert_close = |row: u32, col: u32, expected: f64| {
            let actual = number_from_calc_value(&valuation.read_cell(&address(row, col)).computed)
                .expect("cell should hold a numeric trig result");
            assert!(
                (actual - expected).abs() < 1.0e-12,
                "expected ({row},{col}) to be close to {expected}, got {actual}"
            );
        };
        assert_close(1, 2, 0.0);
        assert_close(1, 3, 1.0);
        assert_close(1, 4, 0.0);
        assert_close(2, 2, 0.5);
        assert_close(2, 3, 3.0_f64.sqrt() / 2.0);
        assert_close(2, 4, 1.0 / 3.0_f64.sqrt());
        assert_close(3, 2, 2.0_f64.sqrt() / 2.0);
        assert_close(3, 3, 2.0_f64.sqrt() / 2.0);
        assert_close(3, 4, 1.0);
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_r1c1_angle_function_templates() {
        let test_bounds = ExcelGridBounds {
            max_rows: 10,
            max_cols: 6,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", test_bounds);
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 2, test_bounds).unwrap(),
                vec![
                    CalcValue::number(0.0),
                    CalcValue::number(0.0),
                    CalcValue::number(90.0),
                    CalcValue::number(std::f64::consts::FRAC_PI_2),
                    CalcValue::number(180.0),
                    CalcValue::number(std::f64::consts::PI),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 3, 3, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=RADIANS(RC[-2])",
                    "excel.grid.v1:r1c1-template:RADIANS(RC[-2])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 3, 4, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=DEGREES(RC[-2])",
                    "excel.grid.v1:r1c1-template:DEGREES(RC[-2])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 5, 3, 5, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=SIN(RADIANS(RC[-4]))",
                    "excel.grid.v1:r1c1-template:SIN(RADIANS(RC[-4]))",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 6, 3, 6, test_bounds).unwrap(),
                GridFormulaCell::new("=PI()", "excel.grid.v1:r1c1-template:PI()")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile R1C1 RADIANS/DEGREES/PI templates");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 18);
        assert_eq!(report.literal_cells, 6);
        assert_eq!(report.formula_cells, 12);
        assert_eq!(report.formula_templates_prepared, 4);
        assert_eq!(report.compiled_formula_plan_cache_misses, 4);
        assert_eq!(report.compiled_formula_plans_cached, 4);
        assert_eq!(report.computed_dense_value_regions, 5);
        assert_eq!(report.computed_sparse_cells, 0);

        let assert_close = |row: u32, col: u32, expected: f64| {
            let actual = number_from_calc_value(&valuation.read_cell(&address(row, col)).computed)
                .expect("cell should hold a numeric angle result");
            assert!(
                (actual - expected).abs() < 1.0e-12,
                "expected ({row},{col}) to be close to {expected}, got {actual}"
            );
        };
        assert_close(1, 3, 0.0);
        assert_close(2, 3, std::f64::consts::FRAC_PI_2);
        assert_close(3, 3, std::f64::consts::PI);
        assert_close(1, 4, 0.0);
        assert_close(2, 4, 90.0);
        assert_close(3, 4, 180.0);
        assert_close(1, 5, 0.0);
        assert_close(2, 5, 1.0);
        assert_close(3, 5, 0.0);
        assert_close(1, 6, std::f64::consts::PI);
        assert_close(2, 6, std::f64::consts::PI);
        assert_close(3, 6, std::f64::consts::PI);
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_two_left_r1c1_as_dense_output() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 2, bounds()).unwrap(),
                vec![
                    CalcValue::number(10.0),
                    CalcValue::number(1.0),
                    CalcValue::number(20.0),
                    CalcValue::number(2.0),
                    CalcValue::number(30.0),
                    CalcValue::number(3.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 3, 4, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=RC[-2]+RC[-1]",
                    "excel.grid.v1:r1c1-template:RC[-2]+RC[-1]",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile two-left R1C1 formulas");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 12);
        assert_eq!(report.literal_cells, 6);
        assert_eq!(report.formula_cells, 6);
        assert_eq!(report.formula_templates_prepared, 1);
        assert_eq!(report.compiled_formula_plan_cache_misses, 1);
        assert_eq!(report.compiled_formula_plans_cached, 1);
        assert_eq!(report.computed_dense_value_regions, 2);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 3)).computed,
            CalcValue::number(11.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(12.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 4)).computed,
            CalcValue::number(36.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_general_binary_r1c1_templates() {
        let test_bounds = ExcelGridBounds {
            max_rows: 10,
            max_cols: 12,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", test_bounds);
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 3, test_bounds).unwrap(),
                vec![
                    CalcValue::number(10.0),
                    CalcValue::number(100.0),
                    CalcValue::number(1.0),
                    CalcValue::number(20.0),
                    CalcValue::number(200.0),
                    CalcValue::number(2.0),
                    CalcValue::number(30.0),
                    CalcValue::number(300.0),
                    CalcValue::number(3.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 3, 4, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=RC[-3]+RC[-1]",
                    "excel.grid.v1:r1c1-template:RC[-3]+RC[-1]",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 5, 3, 5, test_bounds).unwrap(),
                GridFormulaCell::new("=RC[-2]*3", "excel.grid.v1:r1c1-template:RC[-2]*3")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 6, 3, 6, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=RC[-1]-RC[-3]",
                    "excel.grid.v1:r1c1-template:RC[-1]-RC[-3]",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 7, 3, 7, test_bounds).unwrap(),
                GridFormulaCell::new("=RC[-6]/2", "excel.grid.v1:r1c1-template:RC[-6]/2")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 8, 3, 8, test_bounds).unwrap(),
                GridFormulaCell::new("=RC[-7]/0", "excel.grid.v1:r1c1-template:RC[-7]/0")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 9, 3, 9, test_bounds).unwrap(),
                GridFormulaCell::new("=RC[-1]+1", "excel.grid.v1:r1c1-template:RC[-1]+1")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 10, 3, 10, test_bounds).unwrap(),
                GridFormulaCell::new("=RC[-9]*0.5", "excel.grid.v1:r1c1-template:RC[-9]*0.5")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 11, 3, 11, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=RC[-10]+RC[-9]*RC[-8]",
                    "excel.grid.v1:r1c1-template:RC[-10]+RC[-9]*RC[-8]",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 12, 3, 12, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=(RC[-11]+RC[-10])*RC[-9]",
                    "excel.grid.v1:r1c1-template:(RC[-11]+RC[-10])*RC[-9]",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile generic binary R1C1 formulas");

        assert!(report.p00_primary_exact_once_holds());
        assert_eq!(report.occupied_cells, 36);
        assert_eq!(report.literal_cells, 9);
        assert_eq!(report.formula_cells, 27);
        assert_eq!(report.formula_templates_prepared, 9);
        assert_eq!(report.compiled_formula_plan_cache_misses, 9);
        assert_eq!(report.compiled_formula_plans_cached, 9);
        assert_eq!(report.computed_dense_value_regions, 10);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(11.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 4)).computed,
            CalcValue::number(33.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 5)).computed,
            CalcValue::number(3.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 5)).computed,
            CalcValue::number(9.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 6)).computed,
            CalcValue::number(2.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 6)).computed,
            CalcValue::number(6.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 7)).computed,
            CalcValue::number(5.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 7)).computed,
            CalcValue::number(15.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 8)).computed,
            CalcValue::error(WorksheetErrorCode::Div0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 8)).computed,
            CalcValue::error(WorksheetErrorCode::Div0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 9)).computed,
            CalcValue::error(WorksheetErrorCode::Div0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 9)).computed,
            CalcValue::error(WorksheetErrorCode::Div0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 10)).computed,
            CalcValue::number(5.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 10)).computed,
            CalcValue::number(15.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 11)).computed,
            CalcValue::number(110.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 11)).computed,
            CalcValue::number(930.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 12)).computed,
            CalcValue::number(110.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 12)).computed,
            CalcValue::number(990.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_r1c1_if_templates() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 4, 1, bounds()).unwrap(),
                vec![
                    CalcValue::number(-2.0),
                    CalcValue::number(0.0),
                    CalcValue::number(3.0),
                    CalcValue::number(5.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 2, 4, 2, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=IF(RC[-1]>0,RC[-1],0)",
                    "excel.grid.v1:r1c1-template:IF(RC[-1]>0,RC[-1],0)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 4, 3, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=IF(RC[-2]>=3,RC[-1],RC[-2])",
                    "excel.grid.v1:r1c1-template:IF(RC[-2]>=3,RC[-1],RC[-2])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 4, 4, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=IF(RC[-3]>0,RC[-3]*2,RC[-3]/2)",
                    "excel.grid.v1:r1c1-template:IF(RC[-3]>0,RC[-3]*2,RC[-3]/2)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile numeric R1C1 IF templates");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 16);
        assert_eq!(report.literal_cells, 4);
        assert_eq!(report.formula_cells, 12);
        assert_eq!(report.formula_templates_prepared, 3);
        assert_eq!(report.compiled_formula_plan_cache_misses, 3);
        assert_eq!(report.compiled_formula_plans_cached, 3);
        assert_eq!(report.computed_dense_value_regions, 4);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 2)).computed,
            CalcValue::number(0.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 2)).computed,
            CalcValue::number(3.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 3)).computed,
            CalcValue::number(-2.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 3)).computed,
            CalcValue::number(3.0)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 3)).computed,
            CalcValue::number(5.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(-1.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 4)).computed,
            CalcValue::number(0.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 4)).computed,
            CalcValue::number(6.0)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 4)).computed,
            CalcValue::number(10.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_nested_r1c1_if_templates() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 4, 2, bounds()).unwrap(),
                vec![
                    CalcValue::number(-2.0),
                    CalcValue::number(-1.0),
                    CalcValue::number(0.0),
                    CalcValue::number(1.0),
                    CalcValue::number(3.0),
                    CalcValue::number(-1.0),
                    CalcValue::number(5.0),
                    CalcValue::number(1.0),
                ],
            )
            .unwrap();
        let nested_if =
            "IF(RC[-2]>0,IF(RC[-1]>0,RC[-2]*2,RC[-2]*3),IF(RC[-1]>0,RC[-2]+10,RC[-2]-10))";
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 4, 3, bounds()).unwrap(),
                GridFormulaCell::new(
                    format!("={nested_if}"),
                    format!("excel.grid.v1:r1c1-template:{nested_if}"),
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 4, 4, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=IF(RC[-3]>0,IF(RC[-2]>0,RC[-3]*2,RC[-3]*3),IF(RC[-2]>0,RC[-3]+10,RC[-3]-10))+RC[-3]",
                    "excel.grid.v1:r1c1-template:IF(RC[-3]>0,IF(RC[-2]>0,RC[-3]*2,RC[-3]*3),IF(RC[-2]>0,RC[-3]+10,RC[-3]-10))+RC[-3]",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile nested R1C1 IF templates");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 16);
        assert_eq!(report.literal_cells, 8);
        assert_eq!(report.formula_cells, 8);
        assert_eq!(report.formula_templates_prepared, 2);
        assert_eq!(report.compiled_formula_plan_cache_misses, 2);
        assert_eq!(report.compiled_formula_plans_cached, 2);
        assert_eq!(report.computed_dense_value_regions, 3);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 3)).computed,
            CalcValue::number(-12.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 3)).computed,
            CalcValue::number(10.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 3)).computed,
            CalcValue::number(9.0)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 3)).computed,
            CalcValue::number(10.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(-14.0)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 4)).computed,
            CalcValue::number(15.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_r1c1_if_propagates_condition_and_branch_errors() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 2, 3, bounds()).unwrap(),
                vec![
                    CalcValue::error(WorksheetErrorCode::Div0),
                    CalcValue::number(100.0),
                    CalcValue::number(200.0),
                    CalcValue::number(1.0),
                    CalcValue::error(WorksheetErrorCode::Ref),
                    CalcValue::number(300.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 2, 4, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=IF(RC[-3]>0,RC[-2],RC[-1])",
                    "excel.grid.v1:r1c1-template:IF(RC[-3]>0,RC[-2],RC[-1])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile R1C1 IF error paths");

        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.compiled_formula_plan_cache_misses, 1);
        assert_eq!(report.compiled_formula_plans_cached, 1);
        assert_eq!(report.computed_dense_value_regions, 2);
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::error(WorksheetErrorCode::Div0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 4)).computed,
            CalcValue::error(WorksheetErrorCode::Ref)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_r1c1_iferror_templates() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 4, 2, bounds()).unwrap(),
                vec![
                    CalcValue::number(10.0),
                    CalcValue::number(2.0),
                    CalcValue::number(20.0),
                    CalcValue::number(0.0),
                    CalcValue::number(30.0),
                    CalcValue::number(5.0),
                    CalcValue::number(40.0),
                    CalcValue::number(0.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 4, 3, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=IFERROR(RC[-2]/RC[-1],0)",
                    "excel.grid.v1:r1c1-template:IFERROR(RC[-2]/RC[-1],0)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 4, 4, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=IFERROR(RC[-3],RC[-3]/0)",
                    "excel.grid.v1:r1c1-template:IFERROR(RC[-3],RC[-3]/0)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 5, 4, 5, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=IFERROR(RC[-4]/RC[-3],RC[-4]+1)",
                    "excel.grid.v1:r1c1-template:IFERROR(RC[-4]/RC[-3],RC[-4]+1)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile R1C1 IFERROR templates");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 20);
        assert_eq!(report.literal_cells, 8);
        assert_eq!(report.formula_cells, 12);
        assert_eq!(report.formula_templates_prepared, 3);
        assert_eq!(report.compiled_formula_plan_cache_misses, 3);
        assert_eq!(report.compiled_formula_plans_cached, 3);
        assert_eq!(report.computed_dense_value_regions, 4);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 3)).computed,
            CalcValue::number(5.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 3)).computed,
            CalcValue::number(0.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 3)).computed,
            CalcValue::number(6.0)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 3)).computed,
            CalcValue::number(0.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 4)).computed,
            CalcValue::number(20.0)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 4)).computed,
            CalcValue::number(40.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 5)).computed,
            CalcValue::number(5.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 5)).computed,
            CalcValue::number(21.0)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 5)).computed,
            CalcValue::number(41.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_r1c1_comparison_templates() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 4, 2, bounds()).unwrap(),
                vec![
                    CalcValue::number(-2.0),
                    CalcValue::number(1.0),
                    CalcValue::number(0.0),
                    CalcValue::number(0.0),
                    CalcValue::number(3.0),
                    CalcValue::number(3.0),
                    CalcValue::number(5.0),
                    CalcValue::number(4.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 4, 3, bounds()).unwrap(),
                GridFormulaCell::new("=RC[-2]>0", "excel.grid.v1:r1c1-template:RC[-2]>0")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 4, 4, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=RC[-3]*2>RC[-2]+1",
                    "excel.grid.v1:r1c1-template:RC[-3]*2>RC[-2]+1",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 5, 4, 5, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=IFERROR(RC[-4]/RC[-3],99)>0",
                    "excel.grid.v1:r1c1-template:IFERROR(RC[-4]/RC[-3],99)>0",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile R1C1 comparison templates");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 20);
        assert_eq!(report.literal_cells, 8);
        assert_eq!(report.formula_cells, 12);
        assert_eq!(report.formula_templates_prepared, 3);
        assert_eq!(report.compiled_formula_plan_cache_misses, 3);
        assert_eq!(report.compiled_formula_plans_cached, 3);
        assert_eq!(report.computed_dense_value_regions, 4);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(valuation.dense_computed_cells(), 20);
        assert_eq!(valuation.dense_computed_numeric_packed_cells(), 8);
        assert_eq!(valuation.dense_computed_logical_packed_cells(), 12);
        assert_eq!(
            valuation.read_cell(&address(1, 3)).computed,
            CalcValue::logical(false)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 3)).computed,
            CalcValue::logical(true)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::logical(false)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 4)).computed,
            CalcValue::logical(true)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 5)).computed,
            CalcValue::logical(false)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 5)).computed,
            CalcValue::logical(true)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 5)).computed,
            CalcValue::logical(true)
        );
        assert_eq!(
            valuation.read_cell(&address(4, 5)).computed,
            CalcValue::logical(true)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_r1c1_logical_function_templates() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 2, bounds()).unwrap(),
                vec![
                    CalcValue::number(1.0),
                    CalcValue::number(1.0),
                    CalcValue::number(-2.0),
                    CalcValue::number(3.0),
                    CalcValue::number(-3.0),
                    CalcValue::number(-4.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 3, 3, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=AND(RC[-2]>0,RC[-1]>0)",
                    "excel.grid.v1:r1c1-template:AND(RC[-2]>0,RC[-1]>0)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 3, 4, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=OR(RC[-3]>0,RC[-2]>0)",
                    "excel.grid.v1:r1c1-template:OR(RC[-3]>0,RC[-2]>0)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 5, 3, 5, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=NOT(AND(RC[-4]>0,RC[-3]>0))",
                    "excel.grid.v1:r1c1-template:NOT(AND(RC[-4]>0,RC[-3]>0))",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile R1C1 logical function templates");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 15);
        assert_eq!(report.literal_cells, 6);
        assert_eq!(report.formula_cells, 9);
        assert_eq!(report.formula_templates_prepared, 3);
        assert_eq!(report.compiled_formula_plan_cache_misses, 3);
        assert_eq!(report.compiled_formula_plans_cached, 3);
        assert_eq!(report.computed_dense_value_regions, 4);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(valuation.dense_computed_cells(), 15);
        assert_eq!(valuation.dense_computed_numeric_packed_cells(), 6);
        assert_eq!(valuation.dense_computed_logical_packed_cells(), 9);
        assert_eq!(
            valuation.read_cell(&address(1, 3)).computed,
            CalcValue::logical(true)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 3)).computed,
            CalcValue::logical(false)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 3)).computed,
            CalcValue::logical(false)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::logical(true)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 4)).computed,
            CalcValue::logical(true)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 4)).computed,
            CalcValue::logical(false)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 5)).computed,
            CalcValue::logical(false)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 5)).computed,
            CalcValue::logical(true)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 5)).computed,
            CalcValue::logical(true)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_r1c1_if_logical_condition_templates() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 2, bounds()).unwrap(),
                vec![
                    CalcValue::number(1.0),
                    CalcValue::number(-1.0),
                    CalcValue::number(-2.0),
                    CalcValue::number(3.0),
                    CalcValue::number(3.0),
                    CalcValue::number(3.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 3, 3, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=IF(AND(RC[-2]>0,RC[-1]>0),RC[-2]+RC[-1],0)",
                    "excel.grid.v1:r1c1-template:IF(AND(RC[-2]>0,RC[-1]>0),RC[-2]+RC[-1],0)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 3, 4, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=IF(OR(RC[-3]>0,RC[-2]>0),RC[-3]-RC[-2],0)",
                    "excel.grid.v1:r1c1-template:IF(OR(RC[-3]>0,RC[-2]>0),RC[-3]-RC[-2],0)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 5, 3, 5, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=IF(NOT(AND(RC[-4]>0,RC[-3]>0)),ABS(RC[-4]),0)",
                    "excel.grid.v1:r1c1-template:IF(NOT(AND(RC[-4]>0,RC[-3]>0)),ABS(RC[-4]),0)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile R1C1 IF logical condition templates");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 15);
        assert_eq!(report.literal_cells, 6);
        assert_eq!(report.formula_cells, 9);
        assert_eq!(report.formula_templates_prepared, 3);
        assert_eq!(report.compiled_formula_plan_cache_misses, 3);
        assert_eq!(report.compiled_formula_plans_cached, 3);
        assert_eq!(report.computed_dense_value_regions, 4);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(valuation.dense_computed_cells(), 15);
        assert_eq!(valuation.dense_computed_numeric_packed_cells(), 15);
        assert_eq!(valuation.dense_computed_logical_packed_cells(), 0);
        assert_eq!(
            valuation.read_cell(&address(1, 3)).computed,
            CalcValue::number(0.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 3)).computed,
            CalcValue::number(0.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 3)).computed,
            CalcValue::number(6.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(2.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 4)).computed,
            CalcValue::number(-5.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 4)).computed,
            CalcValue::number(0.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 5)).computed,
            CalcValue::number(1.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 5)).computed,
            CalcValue::number(2.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 5)).computed,
            CalcValue::number(0.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_r1c1_comparison_propagates_operand_errors() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 2, 2, bounds()).unwrap(),
                vec![
                    CalcValue::error(WorksheetErrorCode::Div0),
                    CalcValue::number(1.0),
                    CalcValue::number(2.0),
                    CalcValue::error(WorksheetErrorCode::Ref),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 2, 3, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=RC[-2]>RC[-1]",
                    "excel.grid.v1:r1c1-template:RC[-2]>RC[-1]",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile R1C1 comparison error paths");

        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.compiled_formula_plan_cache_misses, 1);
        assert_eq!(report.compiled_formula_plans_cached, 1);
        assert_eq!(report.computed_dense_value_regions, 2);
        assert_eq!(
            valuation.read_cell(&address(1, 3)).computed,
            CalcValue::error(WorksheetErrorCode::Div0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 3)).computed,
            CalcValue::error(WorksheetErrorCode::Ref)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_r1c1_range_aggregate_propagates_errors() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_number_region_with(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap(),
                |address| f64::from(address.row) * 2.0,
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 2, 3, 2, bounds()).unwrap(),
                GridFormulaCell::new("=RC[-1]/0", "excel.grid.v1:r1c1-template:RC[-1]/0")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 3, 3, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=SUM(RC[-2]:RC[-1])",
                    "excel.grid.v1:r1c1-template:SUM(RC[-2]:RC[-1])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 3, 4, bounds()).unwrap(),
                GridFormulaCell::new(
                    "=IFERROR(SUM(RC[-3]:RC[-2]),RC[-3])",
                    "excel.grid.v1:r1c1-template:IFERROR(SUM(RC[-3]:RC[-2]),RC[-3])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should propagate R1C1 range aggregate errors");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 12);
        assert_eq!(report.literal_cells, 3);
        assert_eq!(report.formula_cells, 9);
        assert_eq!(report.formula_templates_prepared, 3);
        assert_eq!(report.compiled_formula_plan_cache_misses, 3);
        assert_eq!(report.compiled_formula_plans_cached, 3);
        assert_eq!(report.computed_dense_value_regions, 4);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(valuation.dense_computed_cells(), 12);
        assert_eq!(valuation.dense_computed_numeric_packed_cells(), 6);
        assert_eq!(valuation.dense_computed_logical_packed_cells(), 0);
        assert_eq!(
            valuation.read_cell(&address(1, 2)).computed,
            CalcValue::error(WorksheetErrorCode::Div0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 3)).computed,
            CalcValue::error(WorksheetErrorCode::Div0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 3)).computed,
            CalcValue::error(WorksheetErrorCode::Div0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(2.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 4)).computed,
            CalcValue::number(6.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_r1c1_text_functions_stay_dense() {
        let test_bounds = ExcelGridBounds {
            max_rows: 10,
            max_cols: 5,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", test_bounds);
        sheet
            .put_dense_literal_region_with(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, test_bounds).unwrap(),
                |_address| CalcValue::text(ExcelText::from_interop_assignment("RowGrid")),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 2, 3, 2, test_bounds).unwrap(),
                GridFormulaCell::new("=LEN(RC[-1])", "excel.grid.v1:r1c1-template:LEN(RC[-1])")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 3, 3, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=LEFT(RC[-2],3)",
                    "excel.grid.v1:r1c1-template:LEFT(RC[-2],3)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 3, 4, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=RIGHT(RC[-3],4)",
                    "excel.grid.v1:r1c1-template:RIGHT(RC[-3],4)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 5, 3, 5, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=CONCAT(RC[-2],RC[-1])",
                    "excel.grid.v1:r1c1-template:CONCAT(RC[-2],RC[-1])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should evaluate R1C1 text functions");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 15);
        assert_eq!(report.literal_cells, 3);
        assert_eq!(report.formula_cells, 12);
        assert_eq!(report.formula_templates_prepared, 4);
        assert_eq!(report.compiled_formula_plan_cache_misses, 4);
        assert_eq!(report.compiled_formula_plans_cached, 4);
        assert_eq!(report.computed_dense_value_regions, 5);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(valuation.dense_computed_cells(), 15);
        assert_eq!(valuation.dense_computed_numeric_packed_cells(), 3);
        assert_eq!(valuation.dense_computed_logical_packed_cells(), 0);
        assert_eq!(
            valuation.read_cell(&address(1, 2)).computed,
            CalcValue::number(7.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 3)).computed,
            CalcValue::text(ExcelText::from_interop_assignment("Row"))
        );
        assert_eq!(
            valuation.read_cell(&address(2, 4)).computed,
            CalcValue::text(ExcelText::from_interop_assignment("Grid"))
        );
        assert_eq!(
            valuation.read_cell(&address(3, 5)).computed,
            CalcValue::text(ExcelText::from_interop_assignment("RowGrid"))
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_r1c1_index_function_stays_dense() {
        let test_bounds = ExcelGridBounds {
            max_rows: 10,
            max_cols: 6,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", test_bounds);
        sheet
            .put_dense_number_region_with(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, test_bounds).unwrap(),
                |address| f64::from(address.row) * 10.0,
            )
            .unwrap();
        sheet
            .put_dense_literal_region_with(
                GridRect::new("book:default", "sheet:default", 1, 2, 3, 2, test_bounds).unwrap(),
                |_address| CalcValue::text(ExcelText::from_interop_assignment("Index")),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 3, 3, 3, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=INDEX(RC[-2]:RC[-1],1,1)",
                    "excel.grid.v1:r1c1-template:INDEX(RC[-2]:RC[-1],1,1)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 3, 4, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=INDEX(RC[-3]:RC[-2],1,2)",
                    "excel.grid.v1:r1c1-template:INDEX(RC[-3]:RC[-2],1,2)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 5, 3, 5, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=INDEX(R1C1:RC1,ROW(),1)",
                    "excel.grid.v1:r1c1-template:INDEX(R1C1:RC1,ROW(),1)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 6, 3, 6, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=INDEX(RC[-5]:RC[-4],2,1)",
                    "excel.grid.v1:r1c1-template:INDEX(RC[-5]:RC[-4],2,1)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should evaluate R1C1 INDEX formulas");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 18);
        assert_eq!(report.literal_cells, 6);
        assert_eq!(report.formula_cells, 12);
        assert_eq!(report.formula_templates_prepared, 4);
        assert_eq!(report.compiled_formula_plan_cache_misses, 4);
        assert_eq!(report.compiled_formula_plans_cached, 4);
        assert_eq!(report.computed_dense_value_regions, 6);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(valuation.dense_computed_cells(), 18);
        assert_eq!(valuation.dense_computed_numeric_packed_cells(), 9);
        assert_eq!(valuation.dense_computed_logical_packed_cells(), 0);
        assert_eq!(
            valuation.read_cell(&address(1, 3)).computed,
            CalcValue::number(10.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 4)).computed,
            CalcValue::text(ExcelText::from_interop_assignment("Index"))
        );
        assert_eq!(
            valuation.read_cell(&address(3, 5)).computed,
            CalcValue::number(30.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 6)).computed,
            CalcValue::error(WorksheetErrorCode::Ref)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_r1c1_match_function_stays_dense() {
        let test_bounds = ExcelGridBounds {
            max_rows: 10,
            max_cols: 6,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", test_bounds);
        sheet
            .put_dense_number_region_with(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 3, test_bounds).unwrap(),
                |address| f64::from(address.row) * 10.0 + f64::from(address.col) - 1.0,
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 3, 4, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=MATCH(RC[-2],RC[-3]:RC[-1],0)",
                    "excel.grid.v1:r1c1-template:MATCH(RC[-2],RC[-3]:RC[-1],0)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 5, 3, 5, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=INDEX(RC[-4]:RC[-2],1,MATCH(RC[-3],RC[-4]:RC[-2],0))",
                    "excel.grid.v1:r1c1-template:INDEX(RC[-4]:RC[-2],1,MATCH(RC[-3],RC[-4]:RC[-2],0))",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 6, 3, 6, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=MATCH(999999999,RC[-5]:RC[-3],0)",
                    "excel.grid.v1:r1c1-template:MATCH(999999999,RC[-5]:RC[-3],0)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should evaluate R1C1 MATCH formulas");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 18);
        assert_eq!(report.literal_cells, 9);
        assert_eq!(report.formula_cells, 9);
        assert_eq!(report.formula_templates_prepared, 3);
        assert_eq!(report.compiled_formula_plan_cache_misses, 3);
        assert_eq!(report.compiled_formula_plans_cached, 3);
        assert_eq!(report.computed_dense_value_regions, 4);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(valuation.dense_computed_cells(), 18);
        assert_eq!(valuation.dense_computed_numeric_packed_cells(), 15);
        assert_eq!(valuation.dense_computed_logical_packed_cells(), 0);
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(2.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 5)).computed,
            CalcValue::number(31.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 6)).computed,
            CalcValue::error(WorksheetErrorCode::NA)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_r1c1_vlookup_function_stays_dense() {
        let test_bounds = ExcelGridBounds {
            max_rows: 10,
            max_cols: 7,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", test_bounds);
        sheet
            .put_dense_number_region_with(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, test_bounds).unwrap(),
                |address| f64::from(address.row) * 10.0,
            )
            .unwrap();
        sheet
            .put_dense_literal_region_with(
                GridRect::new("book:default", "sheet:default", 1, 2, 3, 2, test_bounds).unwrap(),
                |_address| CalcValue::text(ExcelText::from_interop_assignment("Lookup")),
            )
            .unwrap();
        sheet
            .put_dense_number_region_with(
                GridRect::new("book:default", "sheet:default", 1, 3, 3, 3, test_bounds).unwrap(),
                |address| f64::from(address.row) * 100.0,
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 3, 4, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=VLOOKUP(RC[-3],RC[-3]:RC[-1],2,FALSE)",
                    "excel.grid.v1:r1c1-template:VLOOKUP(RC[-3],RC[-3]:RC[-1],2,FALSE)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 5, 3, 5, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=VLOOKUP(RC[-4],RC[-4]:RC[-2],3,0)",
                    "excel.grid.v1:r1c1-template:VLOOKUP(RC[-4],RC[-4]:RC[-2],3,0)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 6, 3, 6, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=VLOOKUP(999999999,RC[-5]:RC[-3],2,FALSE)",
                    "excel.grid.v1:r1c1-template:VLOOKUP(999999999,RC[-5]:RC[-3],2,FALSE)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 7, 3, 7, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=VLOOKUP(RC[-6],RC[-6]:RC[-4],4,FALSE)",
                    "excel.grid.v1:r1c1-template:VLOOKUP(RC[-6],RC[-6]:RC[-4],4,FALSE)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should evaluate R1C1 VLOOKUP formulas");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 21);
        assert_eq!(report.literal_cells, 9);
        assert_eq!(report.formula_cells, 12);
        assert_eq!(report.formula_templates_prepared, 4);
        assert_eq!(report.compiled_formula_plan_cache_misses, 4);
        assert_eq!(report.compiled_formula_plans_cached, 4);
        assert_eq!(report.computed_dense_value_regions, 7);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(valuation.dense_computed_cells(), 21);
        assert_eq!(valuation.dense_computed_numeric_packed_cells(), 9);
        assert_eq!(valuation.dense_computed_logical_packed_cells(), 0);
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::text(ExcelText::from_interop_assignment("Lookup"))
        );
        assert_eq!(
            valuation.read_cell(&address(3, 5)).computed,
            CalcValue::number(300.0)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 6)).computed,
            CalcValue::error(WorksheetErrorCode::NA)
        );
        assert_eq!(
            valuation.read_cell(&address(2, 7)).computed,
            CalcValue::error(WorksheetErrorCode::Ref)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_absolute_r1c1_templates() {
        let test_bounds = ExcelGridBounds {
            max_rows: 10,
            max_cols: 5,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", test_bounds);
        sheet
            .put_dense_number_region_with(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 3, test_bounds).unwrap(),
                |address| f64::from(address.row) * f64::from(address.col),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 3, 4, test_bounds).unwrap(),
                GridFormulaCell::new("=RC[-1]+R1C1", "excel.grid.v1:r1c1-template:RC[-1]+R1C1")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 5, 3, 5, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=SUM(R1C1:R1C3)",
                    "excel.grid.v1:r1c1-template:SUM(R1C1:R1C3)",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile absolute R1C1 references");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 15);
        assert_eq!(report.literal_cells, 9);
        assert_eq!(report.formula_cells, 6);
        assert_eq!(report.formula_templates_prepared, 2);
        assert_eq!(report.compiled_formula_plan_cache_misses, 2);
        assert_eq!(report.compiled_formula_plans_cached, 2);
        assert_eq!(report.computed_dense_value_regions, 3);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(4.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 4)).computed,
            CalcValue::number(10.0)
        );
        assert_eq!(
            valuation.read_cell(&address(1, 5)).computed,
            CalcValue::number(6.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 5)).computed,
            CalcValue::number(6.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_r1c1_sum_range_templates() {
        let test_bounds = ExcelGridBounds {
            max_rows: 10,
            max_cols: 5,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", test_bounds);
        sheet
            .put_dense_number_region_with(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 3, test_bounds).unwrap(),
                |address| f64::from(address.row) * f64::from(address.col),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 3, 4, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=SUM(RC[-3]:RC[-1])",
                    "excel.grid.v1:r1c1-template:SUM(RC[-3]:RC[-1])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile R1C1 SUM range formulas");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 12);
        assert_eq!(report.literal_cells, 9);
        assert_eq!(report.formula_cells, 3);
        assert_eq!(report.formula_templates_prepared, 1);
        assert_eq!(report.compiled_formula_plan_cache_misses, 1);
        assert_eq!(report.compiled_formula_plans_cached, 1);
        assert_eq!(report.computed_dense_value_regions, 2);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(6.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 4)).computed,
            CalcValue::number(18.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_r1c1_sumsq_range_templates() {
        let test_bounds = ExcelGridBounds {
            max_rows: 10,
            max_cols: 5,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", test_bounds);
        sheet
            .put_dense_number_region_with(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 3, test_bounds).unwrap(),
                |address| f64::from(address.row) * f64::from(address.col),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 3, 4, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=SUMSQ(RC[-3]:RC[-1])",
                    "excel.grid.v1:r1c1-template:SUMSQ(RC[-3]:RC[-1])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile R1C1 SUMSQ range formulas");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 12);
        assert_eq!(report.literal_cells, 9);
        assert_eq!(report.formula_cells, 3);
        assert_eq!(report.formula_templates_prepared, 1);
        assert_eq!(report.compiled_formula_plan_cache_misses, 1);
        assert_eq!(report.compiled_formula_plans_cached, 1);
        assert_eq!(report.computed_dense_value_regions, 2);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(14.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 4)).computed,
            CalcValue::number(126.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_r1c1_count_range_templates() {
        let test_bounds = ExcelGridBounds {
            max_rows: 10,
            max_cols: 5,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", test_bounds);
        sheet
            .put_dense_number_region_with(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 3, test_bounds).unwrap(),
                |address| f64::from(address.row) * f64::from(address.col),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 3, 4, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=COUNT(RC[-3]:RC[-1])",
                    "excel.grid.v1:r1c1-template:COUNT(RC[-3]:RC[-1])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile R1C1 COUNT range formulas");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 12);
        assert_eq!(report.literal_cells, 9);
        assert_eq!(report.formula_cells, 3);
        assert_eq!(report.formula_templates_prepared, 1);
        assert_eq!(report.compiled_formula_plan_cache_misses, 1);
        assert_eq!(report.compiled_formula_plans_cached, 1);
        assert_eq!(report.computed_dense_value_regions, 2);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(3.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 4)).computed,
            CalcValue::number(3.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_r1c1_product_range_templates() {
        let test_bounds = ExcelGridBounds {
            max_rows: 10,
            max_cols: 5,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", test_bounds);
        sheet
            .put_dense_number_region_with(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 3, test_bounds).unwrap(),
                |address| f64::from(address.row) * f64::from(address.col),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 3, 4, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=PRODUCT(RC[-3]:RC[-1])",
                    "excel.grid.v1:r1c1-template:PRODUCT(RC[-3]:RC[-1])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile R1C1 PRODUCT range formulas");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 12);
        assert_eq!(report.literal_cells, 9);
        assert_eq!(report.formula_cells, 3);
        assert_eq!(report.formula_templates_prepared, 1);
        assert_eq!(report.compiled_formula_plan_cache_misses, 1);
        assert_eq!(report.compiled_formula_plans_cached, 1);
        assert_eq!(report.computed_dense_value_regions, 2);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(6.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 4)).computed,
            CalcValue::number(162.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_r1c1_average_range_templates() {
        let test_bounds = ExcelGridBounds {
            max_rows: 10,
            max_cols: 5,
        };
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", test_bounds);
        sheet
            .put_dense_number_region_with(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 3, test_bounds).unwrap(),
                |address| f64::from(address.row) * f64::from(address.col),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 4, 3, 4, test_bounds).unwrap(),
                GridFormulaCell::new(
                    "=AVERAGE(RC[-3]:RC[-1])",
                    "excel.grid.v1:r1c1-template:AVERAGE(RC[-3]:RC[-1])",
                )
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml(100)
            .expect("optimized compact recalc should compile R1C1 AVERAGE range formulas");

        assert!(report.p00_primary_exact_once_holds());
        assert!(report.p11_template_prepare_once_holds());
        assert!(report.p14_plan_cache_hit_floor_holds());
        assert_eq!(report.occupied_cells, 12);
        assert_eq!(report.literal_cells, 9);
        assert_eq!(report.formula_cells, 3);
        assert_eq!(report.formula_templates_prepared, 1);
        assert_eq!(report.compiled_formula_plan_cache_misses, 1);
        assert_eq!(report.compiled_formula_plans_cached, 1);
        assert_eq!(report.computed_dense_value_regions, 2);
        assert_eq!(report.computed_sparse_cells, 0);
        assert_eq!(
            valuation.read_cell(&address(1, 4)).computed,
            CalcValue::number(2.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 4)).computed,
            CalcValue::number(6.0)
        );
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_r1c1_min_max_range_templates() {
        for (formula, normal_form_key, first_expected, last_expected) in [
            (
                "=MIN(RC[-3]:RC[-1])",
                "excel.grid.v1:r1c1-template:MIN(RC[-3]:RC[-1])",
                1.0,
                3.0,
            ),
            (
                "=MAX(RC[-3]:RC[-1])",
                "excel.grid.v1:r1c1-template:MAX(RC[-3]:RC[-1])",
                3.0,
                9.0,
            ),
        ] {
            let test_bounds = ExcelGridBounds {
                max_rows: 10,
                max_cols: 5,
            };
            let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", test_bounds);
            sheet
                .put_dense_number_region_with(
                    GridRect::new("book:default", "sheet:default", 1, 1, 3, 3, test_bounds)
                        .unwrap(),
                    |address| f64::from(address.row) * f64::from(address.col),
                )
                .unwrap();
            sheet
                .put_repeated_formula_region(
                    GridRect::new("book:default", "sheet:default", 1, 4, 3, 4, test_bounds)
                        .unwrap(),
                    GridFormulaCell::new(formula, normal_form_key)
                        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
                )
                .unwrap();

            let (valuation, report) = sheet
                .recalculate_mark_all_dirty_compact_with_oxfml(100)
                .expect("optimized compact recalc should compile R1C1 MIN/MAX range formulas");

            assert!(report.p00_primary_exact_once_holds());
            assert!(report.p11_template_prepare_once_holds());
            assert!(report.p14_plan_cache_hit_floor_holds());
            assert_eq!(report.occupied_cells, 12);
            assert_eq!(report.literal_cells, 9);
            assert_eq!(report.formula_cells, 3);
            assert_eq!(report.formula_templates_prepared, 1);
            assert_eq!(report.compiled_formula_plan_cache_misses, 1);
            assert_eq!(report.compiled_formula_plans_cached, 1);
            assert_eq!(report.computed_dense_value_regions, 2);
            assert_eq!(report.computed_sparse_cells, 0);
            assert_eq!(
                valuation.read_cell(&address(1, 4)).computed,
                CalcValue::number(first_expected)
            );
            assert_eq!(
                valuation.read_cell(&address(3, 4)).computed,
                CalcValue::number(last_expected)
            );
        }
    }

    #[test]
    fn optimized_grid_persistent_formula_plan_cache_survives_recalc_rounds() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap(),
                vec![
                    CalcValue::number(10.0),
                    CalcValue::number(20.0),
                    CalcValue::number(30.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 2, 3, 2, bounds()).unwrap(),
                GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let mut formula_plan_cache = GridOptimizedFormulaPlanCache::default();
        let (_, cold_report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml_using_formula_plan_cache(
                100,
                &mut formula_plan_cache,
            )
            .unwrap();
        assert_eq!(cold_report.formula_plan_cache_misses, 1);
        assert_eq!(cold_report.formula_plan_cache_hits, 2);
        assert_eq!(cold_report.compiled_formula_plan_cache_misses, 1);
        assert_eq!(cold_report.compiled_formula_plan_cache_hits, 0);
        assert_eq!(formula_plan_cache.cached_template_count(), 1);
        assert_eq!(formula_plan_cache.cached_compiled_plan_count(), 1);
        assert!(formula_plan_cache.contains_template("excel.grid.v1:r1c1-template:RC[-1]*2"));
        assert!(formula_plan_cache.contains_compiled_plan("excel.grid.v1:r1c1-template:RC[-1]*2"));

        let (valuation, hot_report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml_using_formula_plan_cache(
                100,
                &mut formula_plan_cache,
            )
            .unwrap();
        assert_eq!(hot_report.formula_cells, 3);
        assert_eq!(hot_report.formula_plan_cache_misses, 0);
        assert_eq!(hot_report.formula_plan_cache_hits, 3);
        assert_eq!(hot_report.compiled_formula_plan_cache_misses, 0);
        assert_eq!(hot_report.compiled_formula_plan_cache_hits, 1);
        assert_eq!(
            valuation.read_cell(&address(3, 2)).computed,
            CalcValue::number(60.0)
        );

        let report = sheet.persistent_formula_plan_cache_report(3, 100).unwrap();
        assert!(report.p14_persistent_plan_cache_holds());
        assert_eq!(report.rounds, 3);
        assert_eq!(report.formula_cells_per_round, 3);
        assert_eq!(report.first_round_misses, 1);
        assert_eq!(report.later_round_misses, 0);
        assert_eq!(report.total_misses, 1);
        assert_eq!(report.total_hits, 8);
        assert_eq!(report.total_compiled_plan_misses, 1);
        assert_eq!(report.total_compiled_plan_hits, 2);
        assert_eq!(report.cached_compiled_plan_count, 1);
    }

    #[test]
    fn optimized_grid_formula_plan_cache_recompiles_stale_fingerprint_and_prunes_unused_plans() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap(),
                vec![
                    CalcValue::number(10.0),
                    CalcValue::number(20.0),
                    CalcValue::number(30.0),
                ],
            )
            .unwrap();
        sheet
            .set_literal(address(1, 2), CalcValue::number(5.0))
            .unwrap();
        let normal_key = "excel.grid.v1:r1c1-template:volatile-key-for-test";
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 2, 2, 3, 2, bounds()).unwrap(),
                GridFormulaCell::new("=RC[-1]*2", normal_key)
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let mut formula_plan_cache = GridOptimizedFormulaPlanCache::default();
        let (_, cold_report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml_using_formula_plan_cache(
                100,
                &mut formula_plan_cache,
            )
            .unwrap();
        assert_eq!(cold_report.compiled_formula_plan_cache_misses, 1);
        assert_eq!(formula_plan_cache.cached_compiled_plan_count(), 1);
        assert_eq!(
            formula_plan_cache.compiled_plan_for_formula(
                &GridFormulaCell::new("=RC[-1]*2", normal_key)
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1)
            ),
            Some(GridOptimizedCompiledFormulaPlan::r1c1_double_left())
        );

        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 2, 2, 3, 2, bounds()).unwrap(),
                GridFormulaCell::new("=RC[-1]+R[-1]C", normal_key)
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();
        let (valuation, recompiled_report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml_using_formula_plan_cache(
                100,
                &mut formula_plan_cache,
            )
            .unwrap();
        assert_eq!(recompiled_report.formula_plan_cache_misses, 0);
        assert_eq!(recompiled_report.compiled_formula_plan_cache_misses, 1);
        assert_eq!(recompiled_report.compiled_formula_plan_cache_hits, 0);
        assert_eq!(formula_plan_cache.cached_compiled_plan_count(), 1);
        let left_plus_above = GridFormulaCell::new("=RC[-1]+R[-1]C", normal_key)
            .with_source_channel(FormulaChannelKind::WorksheetR1C1);
        assert_eq!(
            formula_plan_cache.compiled_plan_for_formula(&left_plus_above),
            GridOptimizedCompiledFormulaPlan::compile(&left_plus_above)
        );
        assert_ne!(
            formula_plan_cache.compiled_plan_for_formula(&left_plus_above),
            Some(GridOptimizedCompiledFormulaPlan::r1c1_double_left())
        );
        assert_eq!(
            valuation.read_cell(&address(2, 2)).computed,
            CalcValue::number(25.0)
        );
        assert_eq!(
            valuation.read_cell(&address(3, 2)).computed,
            CalcValue::number(55.0)
        );

        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 2, 2, 3, 2, bounds()).unwrap(),
                vec![CalcValue::number(7.0), CalcValue::number(8.0)],
            )
            .unwrap();
        let (_, pruned_report) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml_using_formula_plan_cache(
                100,
                &mut formula_plan_cache,
            )
            .unwrap();
        assert_eq!(pruned_report.formula_cells, 0);
        assert_eq!(formula_plan_cache.cached_template_count(), 0);
        assert_eq!(formula_plan_cache.cached_compiled_plan_count(), 0);
    }

    #[test]
    fn optimized_grid_warm_noop_reuses_cached_valuation_until_sheet_changes() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap(),
                vec![
                    CalcValue::number(2.0),
                    CalcValue::number(4.0),
                    CalcValue::number(6.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 2, 3, 2, bounds()).unwrap(),
                GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let (valuation, report, cache) = sheet
            .recalculate_mark_all_dirty_compact_with_oxfml_cached(100)
            .unwrap();
        assert_eq!(report.occupied_cells, 6);
        assert_eq!(report.cells_evaluated, 6);
        assert_eq!(report.formula_evaluations, 3);
        assert_eq!(
            valuation.read_cell(&address(3, 2)).computed,
            CalcValue::number(12.0)
        );

        let (warm_valuation, warm_report) = sheet
            .recalculate_warm_noop_compact_with_oxfml(&cache)
            .expect("unchanged sheet should reuse optimized valuation");
        assert!(warm_report.p19_warm_noop_holds());
        assert_eq!(warm_report.cached_occupied_cells, 6);
        assert_eq!(warm_report.cached_formula_cells, 3);
        assert_eq!(warm_report.cells_visited, 0);
        assert_eq!(
            warm_valuation.read_cell(&address(3, 2)).computed,
            CalcValue::number(12.0)
        );

        sheet
            .set_literal(address(3, 1), CalcValue::number(10.0))
            .unwrap();
        assert!(
            sheet
                .recalculate_warm_noop_compact_with_oxfml(&cache)
                .is_none()
        );
    }

    #[test]
    fn optimized_grid_whole_column_enumeration_visits_occupied_slots_not_extent() {
        let bounds = ExcelGridBounds::strict_excel();
        let mut sheet = GridOptimizedSheet::new("book:default", "sheet:default", bounds);
        sheet
            .set_literal(address(1, 1), CalcValue::number(5.0))
            .unwrap();
        sheet
            .set_literal(address(bounds.max_rows, 2), CalcValue::number(7.0))
            .unwrap();
        sheet
            .set_literal(address(1, 3), CalcValue::number(99.0))
            .unwrap();
        sheet
            .set_formula(
                address(1, 4),
                GridFormulaCell::new("=SUM(A:B)", "excel.grid.v1:sum-whole-column:C1:C2"),
            )
            .unwrap();

        let reports = sheet
            .optimized_formula_reference_enumeration_reports(&address(1, 4), 100_000)
            .unwrap();

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].reference_source_text, "A:B");
        assert_eq!(
            reports[0].declared_cell_count,
            usize::try_from(bounds.max_rows).unwrap() * 2
        );
        assert_eq!(reports[0].defined_cell_count, 2);
        assert_eq!(reports[0].sparse_value_cells_visited, 2);
        assert_eq!(reports[0].slots_visited(), 2);
        assert!(reports[0].p20_occupied_slots_holds());
    }

    #[test]
    fn optimized_grid_provider_materializes_large_dense_reference_within_budget() {
        let rows = u32::try_from(GRID_OPTIMIZED_PROVIDER_MATERIALIZATION_LIMIT)
            .unwrap()
            .saturating_add(1);
        let large_bounds = ExcelGridBounds {
            max_rows: rows,
            max_cols: 2,
        };
        let mut valuation =
            GridOptimizedValuation::new("book:default", "sheet:default", large_bounds);
        let rect =
            GridRect::new("book:default", "sheet:default", 1, 1, rows, 1, large_bounds).unwrap();
        valuation.push_dense_value_payload(
            rect,
            GridDenseValuePayload::from_numbers((1..=rows).map(f64::from).collect()),
            1,
            GridOptimizedCellSource::DenseValueRegion { region_index: 0 },
        );
        let provider = valuation.reference_system_provider_with_dense_materialization_limit(
            1,
            2,
            u64::from(rows),
        );
        let reference = strict_grid_reference_from_text(
            format!("A1:A{rows}"),
            &ExcelGridCellAddress::new("book:default", "sheet:default", 1, 2),
            large_bounds,
        );

        let value = provider
            .dereference(&ReferenceDereferenceRequest { reference })
            .expect("large dense reference should dereference within the explicit budget");
        let CoreValue::Array(array) = value.core() else {
            panic!("large dense reference should materialize as an array");
        };

        assert_eq!(
            array.shape(),
            ArrayShape {
                rows: usize::try_from(rows).unwrap(),
                cols: 1
            }
        );
        assert_eq!(array.get(0, 0), Some(&CalcValue::number(1.0)));
        assert_eq!(
            array.get(usize::try_from(rows).unwrap() - 1, 0),
            Some(&CalcValue::number(f64::from(rows)))
        );
    }

    #[test]
    fn optimized_grid_provider_materializes_large_dense_reference_with_sparse_overlay() {
        let rows = u32::try_from(GRID_OPTIMIZED_PROVIDER_MATERIALIZATION_LIMIT)
            .unwrap()
            .saturating_add(1);
        let large_bounds = ExcelGridBounds {
            max_rows: rows,
            max_cols: 2,
        };
        let mut valuation =
            GridOptimizedValuation::new("book:default", "sheet:default", large_bounds);
        let rect =
            GridRect::new("book:default", "sheet:default", 1, 1, rows, 1, large_bounds).unwrap();
        valuation.push_dense_value_payload(
            rect,
            GridDenseValuePayload::from_numbers((1..=rows).map(f64::from).collect()),
            1,
            GridOptimizedCellSource::DenseValueRegion { region_index: 0 },
        );
        let override_row = rows / 2;
        valuation
            .insert_sparse_computed_value(
                ExcelGridCellAddress::new("book:default", "sheet:default", override_row, 1),
                2,
                CalcValue::number(99_999.0),
                GridOptimizedCellSource::SparsePoint,
            )
            .unwrap();
        let provider = valuation.reference_system_provider_with_dense_materialization_limit(
            1,
            2,
            u64::from(rows),
        );
        let reference = strict_grid_reference_from_text(
            format!("A1:A{rows}"),
            &ExcelGridCellAddress::new("book:default", "sheet:default", 1, 2),
            large_bounds,
        );

        let value = provider
            .dereference(&ReferenceDereferenceRequest { reference })
            .expect("large dense reference with sparse overlay should dereference within budget");
        let CoreValue::Array(array) = value.core() else {
            panic!("large dense reference should materialize as an array");
        };

        assert_eq!(
            array.shape(),
            ArrayShape {
                rows: usize::try_from(rows).unwrap(),
                cols: 1
            }
        );
        assert_eq!(array.get(0, 0), Some(&CalcValue::number(1.0)));
        assert_eq!(
            array.get(usize::try_from(override_row - 1).unwrap(), 0),
            Some(&CalcValue::number(99_999.0))
        );
        assert_eq!(
            array.get(usize::try_from(rows).unwrap() - 1, 0),
            Some(&CalcValue::number(f64::from(rows)))
        );
    }

    #[test]
    fn optimized_grid_provider_keeps_large_sparse_reference_on_enumeration_path() {
        let rows = u32::try_from(GRID_OPTIMIZED_PROVIDER_MATERIALIZATION_LIMIT)
            .unwrap()
            .saturating_add(1);
        let large_bounds = ExcelGridBounds {
            max_rows: rows,
            max_cols: 2,
        };
        let mut valuation =
            GridOptimizedValuation::new("book:default", "sheet:default", large_bounds);
        valuation
            .insert_sparse_computed_value(
                ExcelGridCellAddress::new("book:default", "sheet:default", rows, 1),
                1,
                CalcValue::number(7.0),
                GridOptimizedCellSource::SparsePoint,
            )
            .unwrap();
        let provider = valuation.reference_system_provider_with_dense_materialization_limit(
            1,
            2,
            u64::from(rows),
        );
        let reference = strict_grid_reference_from_text(
            format!("A1:A{rows}"),
            &ExcelGridCellAddress::new("book:default", "sheet:default", 1, 2),
            large_bounds,
        );

        let error = provider
            .dereference(&ReferenceDereferenceRequest { reference })
            .expect_err("large sparse reference should require sparse enumeration");

        assert!(matches!(
            error,
            ReferenceResolutionError::ProviderFailure { detail }
                if detail == "optimized_grid_reference_requires_sparse_enumeration"
        ));
    }

    #[test]
    fn optimized_grid_compact_oxfml_recalc_evaluates_subtotal_with_axis_visibility_context() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap(),
                vec![
                    CalcValue::number(10.0),
                    CalcValue::number(20.0),
                    CalcValue::number(30.0),
                ],
            )
            .unwrap();
        sheet.axis_state_mut().set_row(
            2,
            GridAxisProps {
                hidden_manual: true,
                ..GridAxisProps::visible()
            },
        );
        sheet.axis_state_mut().set_row(
            3,
            GridAxisProps {
                hidden_filter: true,
                ..GridAxisProps::visible()
            },
        );
        sheet
            .set_formula(
                address(1, 2),
                GridFormulaCell::new(
                    "=SUBTOTAL(9,A1:A3)",
                    "excel.grid.v1:subtotal9:R[0]C[-1]:R[2]C[-1]",
                ),
            )
            .unwrap();
        sheet
            .set_formula(
                address(1, 3),
                GridFormulaCell::new(
                    "=SUBTOTAL(109,A1:A3)",
                    "excel.grid.v1:subtotal109:R[0]C[-2]:R[2]C[-2]",
                ),
            )
            .unwrap();

        let report = sheet
            .run_engine_mode_with_oxfml(GridEngineMode::Both, [address(1, 2), address(1, 3)], 100)
            .expect("optimized host info should match reference hidden-row subtotal semantics");

        assert!(report.mismatches.is_empty());
        assert_eq!(
            report.optimized.as_ref().unwrap().readout[0].computed,
            CalcValue::number(30.0)
        );
        assert_eq!(
            report.optimized.as_ref().unwrap().readout[1].computed,
            CalcValue::number(10.0)
        );
    }

    #[test]
    fn optimized_grid_structural_insert_splits_dense_value_region_without_authoring_gap() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap(),
                vec![
                    CalcValue::number(10.0),
                    CalcValue::number(20.0),
                    CalcValue::number(30.0),
                ],
            )
            .unwrap();
        sheet
            .set_defined_name(
                "InputRange",
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap(),
            )
            .unwrap();

        let report = sheet
            .apply_axis_edit(GridAxisEdit::insert_rows(2, 1))
            .expect("optimized dense region should split around inserted blank row");

        assert_eq!(report.dense_value_regions_before, 1);
        assert_eq!(report.dense_value_regions_after, 2);
        assert_eq!(report.dense_value_cells_before, 3);
        assert_eq!(report.dense_value_cells_after, 3);
        assert_eq!(
            sheet.authored_cell_at(&address(1, 1)).unwrap().authored,
            Some(GridAuthoredCell::Literal(CalcValue::number(10.0)))
        );
        assert_eq!(
            sheet.authored_cell_at(&address(2, 1)).unwrap().authored,
            None
        );
        assert_eq!(
            sheet.authored_cell_at(&address(3, 1)).unwrap().authored,
            Some(GridAuthoredCell::Literal(CalcValue::number(20.0)))
        );
        assert_eq!(
            sheet.authored_cell_at(&address(4, 1)).unwrap().authored,
            Some(GridAuthoredCell::Literal(CalcValue::number(30.0)))
        );
        assert_eq!(
            sheet.defined_names().get("INPUTRANGE").unwrap(),
            &GridRect::new("book:default", "sheet:default", 1, 1, 4, 1, bounds()).unwrap()
        );
    }

    #[test]
    fn optimized_grid_structural_insert_transforms_feature_rendered_regions() {
        let mut sheet = optimized_sheet();
        sheet
            .add_feature_rendered_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap(),
                "table-overlay",
                false,
            )
            .unwrap();

        let report = sheet
            .apply_axis_edit(GridAxisEdit::insert_rows(2, 1))
            .expect("optimized feature regions should transform with row insertion");

        assert_eq!(report.feature_regions_kept, 1);
        assert_eq!(report.feature_regions_dropped, 0);
        assert_eq!(report.feature_regions_marked_needs_refresh, 0);
        assert_eq!(
            sheet.feature_rendered_regions()[0].rect,
            GridRect::new("book:default", "sheet:default", 1, 1, 4, 1, bounds()).unwrap()
        );
    }

    #[test]
    fn optimized_grid_structural_insert_before_pivot_feature_marks_refresh() {
        let mut sheet = optimized_sheet();
        sheet
            .add_feature_rendered_region(
                GridRect::new("book:default", "sheet:default", 2, 1, 4, 2, bounds()).unwrap(),
                "pivot-report",
                false,
            )
            .unwrap();

        let report = sheet
            .apply_axis_edit(GridAxisEdit::insert_rows(2, 1))
            .expect("pivot feature region should shift when edit is before it");

        assert_eq!(report.feature_regions_kept, 1);
        assert_eq!(report.feature_regions_dropped, 0);
        assert_eq!(report.feature_regions_marked_needs_refresh, 1);
        assert_eq!(
            sheet.feature_rendered_regions()[0],
            FeatureRenderedRegion {
                rect: GridRect::new("book:default", "sheet:default", 3, 1, 5, 2, bounds()).unwrap(),
                feature_kind: "pivot-report".to_string(),
                needs_refresh: true,
            }
        );
    }

    #[test]
    fn optimized_grid_structural_insert_inside_pivot_feature_is_refused_without_mutation() {
        let mut sheet = optimized_sheet();
        sheet
            .set_literal(address(5, 1), CalcValue::number(55.0))
            .unwrap();
        sheet
            .add_feature_rendered_region(
                GridRect::new("book:default", "sheet:default", 2, 1, 4, 2, bounds()).unwrap(),
                "pivot-report",
                false,
            )
            .unwrap();

        let error = sheet
            .apply_axis_edit(GridAxisEdit::insert_rows(3, 1))
            .expect_err("pivot feature region should refuse an inside row insertion");

        assert!(matches!(
            error,
            GridRefError::FeatureRenderedRegionEditRefused { feature_kind, .. }
                if feature_kind == "pivot-report"
        ));
        assert_eq!(
            sheet.authored_cell_at(&address(5, 1)).unwrap().authored,
            Some(GridAuthoredCell::Literal(CalcValue::number(55.0)))
        );
        assert_eq!(
            sheet.feature_rendered_regions()[0],
            FeatureRenderedRegion {
                rect: GridRect::new("book:default", "sheet:default", 2, 1, 4, 2, bounds()).unwrap(),
                feature_kind: "pivot-report".to_string(),
                needs_refresh: false,
            }
        );
    }

    #[test]
    fn optimized_grid_structural_insert_transforms_merged_regions() {
        let mut sheet = optimized_sheet();
        sheet
            .add_merged_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap(),
            )
            .unwrap();

        let report = sheet
            .apply_axis_edit(GridAxisEdit::insert_rows(2, 1))
            .expect("optimized merged regions should transform with row insertion");

        assert_eq!(report.merged_regions_kept, 1);
        assert_eq!(report.merged_regions_dropped, 0);
        assert_eq!(
            sheet.merged_regions()[0].rect,
            GridRect::new("book:default", "sheet:default", 1, 1, 4, 1, bounds()).unwrap()
        );
    }

    #[test]
    fn optimized_grid_structural_insert_splits_repeated_r1c1_region_and_still_matches_reference() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 4, 1, bounds()).unwrap(),
                vec![
                    CalcValue::number(10.0),
                    CalcValue::number(20.0),
                    CalcValue::number(30.0),
                    CalcValue::number(40.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 2, 4, 2, bounds()).unwrap(),
                GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let edit_report = sheet
            .apply_axis_edit(GridAxisEdit::insert_rows(3, 1))
            .expect("optimized repeated formula region should split around inserted blank row");

        assert_eq!(edit_report.dense_value_regions_after, 2);
        assert_eq!(edit_report.repeated_formula_regions_before, 1);
        assert_eq!(edit_report.repeated_formula_regions_after, 2);
        assert_eq!(edit_report.repeated_formula_cells_after, 4);
        assert_eq!(edit_report.repeated_formula_reference_transforms, 1);

        let report = sheet
            .run_engine_mode_with_oxfml(
                GridEngineMode::Both,
                [
                    address(1, 2),
                    address(2, 2),
                    address(3, 2),
                    address(4, 2),
                    address(5, 2),
                ],
                100,
            )
            .expect("optimized structural edit should stay equivalent to reference projection");

        assert!(report.mismatches.is_empty());
        let optimized = &report.optimized.as_ref().unwrap().readout;
        assert_eq!(optimized[0].computed, CalcValue::number(20.0));
        assert_eq!(optimized[1].computed, CalcValue::number(40.0));
        assert_eq!(optimized[2].computed, CalcValue::empty());
        assert_eq!(optimized[3].computed, CalcValue::number(60.0));
        assert_eq!(optimized[4].computed, CalcValue::number(80.0));
    }

    #[test]
    fn optimized_grid_structural_insert_rewrites_sparse_formula_reference() {
        let mut sheet = optimized_sheet();
        sheet
            .set_literal(address(1, 1), CalcValue::number(2.0))
            .unwrap();
        sheet
            .set_formula(
                address(1, 2),
                GridFormulaCell::new("=A1+1", "stale-before-transform"),
            )
            .unwrap();

        let edit_report = sheet
            .apply_axis_edit(GridAxisEdit::insert_columns(1, 1))
            .expect("optimized sparse formula should transform with structural edit");

        assert_eq!(edit_report.sparse_points_kept, 2);
        assert_eq!(edit_report.sparse_points_dropped, 0);
        assert_eq!(edit_report.sparse_formula_cells_transformed, 1);
        assert_eq!(edit_report.sparse_formula_reference_transforms, 1);
        let readout = sheet.authored_cell_at(&address(1, 3)).unwrap();
        match readout.authored.unwrap() {
            GridAuthoredCell::Formula(formula) => assert_eq!(formula.source_text, "=B1+1"),
            other => panic!("expected sparse transformed formula, got {other:?}"),
        }

        let report = sheet
            .run_engine_mode_with_oxfml(GridEngineMode::Both, [address(1, 3)], 100)
            .expect("optimized sparse formula edit should stay equivalent to reference");
        assert!(report.mismatches.is_empty());
        assert_eq!(
            report.optimized.as_ref().unwrap().readout[0].computed,
            CalcValue::number(3.0)
        );
    }

    #[test]
    fn grid_engine_mode_both_matches_dense_values_and_sparse_sum_formula() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap(),
                vec![
                    CalcValue::number(2.0),
                    CalcValue::number(3.0),
                    CalcValue::number(5.0),
                ],
            )
            .unwrap();
        sheet
            .set_formula(
                address(1, 2),
                GridFormulaCell::new("=SUM(A1:A3)", "excel.grid.v1:sum:R[0]C[-1]:R[2]C[-1]"),
            )
            .unwrap();

        let report = sheet
            .run_engine_mode_with_oxfml(
                GridEngineMode::Both,
                [address(1, 1), address(2, 1), address(3, 1), address(1, 2)],
                100,
            )
            .expect("both-mode harness should run reference and optimized engines");

        assert!(report.mismatches.is_empty());
        assert!(report.reference.is_some());
        assert!(report.optimized.is_some());
        assert_eq!(
            report.reference.as_ref().unwrap().readout[3].computed,
            CalcValue::number(10.0)
        );
        assert_eq!(
            report.optimized.as_ref().unwrap().readout[3].computed,
            CalcValue::number(10.0)
        );
        assert!(matches!(
            &report.reference.as_ref().unwrap().recalc,
            GridEngineRecalcReport::Reference(recalc)
                if recalc.p00_non_spill_exact_once_holds()
                    && recalc.occupied_cells == 4
                    && recalc.formula_cells == 1
        ));
        assert!(matches!(
            &report.optimized.as_ref().unwrap().recalc,
            GridEngineRecalcReport::Optimized(recalc)
                if recalc.p00_primary_exact_once_holds()
                    && recalc.occupied_cells == 4
                    && recalc.computed_dense_value_regions == 1
                    && recalc.computed_sparse_cells == 1
        ));
    }

    #[test]
    fn grid_reference_functions_evaluate_through_strict_excel_grid_provider() {
        // Reference-taking functions that were migrated out of OxFunc's bare-adapter
        // corpus (OxFml core is grid-agnostic) are exercised here against the REAL
        // strict-excel-grid profile + provider, end to end through OxFml. OxFunc keeps
        // a minimal same-sheet version (oxfml_minimal_reference_functions); this is the
        // grid-owning home.

        // COLUMNS reports the column span of a range.
        let mut columns_sheet = optimized_sheet();
        columns_sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 1, 3, bounds()).unwrap(),
                vec![
                    CalcValue::number(1.0),
                    CalcValue::number(2.0),
                    CalcValue::number(3.0),
                ],
            )
            .unwrap();
        columns_sheet
            .set_formula(
                address(1, 5),
                GridFormulaCell::new("=COLUMNS(A1:C1)", "test:grid-ref:columns"),
            )
            .unwrap();
        let columns_report = columns_sheet
            .run_engine_mode_with_oxfml(
                GridEngineMode::Both,
                [address(1, 1), address(1, 2), address(1, 3), address(1, 5)],
                100,
            )
            .expect("columns harness runs");
        assert!(columns_report.mismatches.is_empty());
        assert_eq!(
            columns_report.reference.as_ref().unwrap().readout[3].computed,
            CalcValue::number(3.0)
        );

        // INDEX selects a cell from an area (A1:B3 row-major -> B2 == 40).
        let mut index_sheet = optimized_sheet();
        index_sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 2, bounds()).unwrap(),
                vec![
                    CalcValue::number(10.0),
                    CalcValue::number(20.0),
                    CalcValue::number(30.0),
                    CalcValue::number(40.0),
                    CalcValue::number(50.0),
                    CalcValue::number(60.0),
                ],
            )
            .unwrap();
        index_sheet
            .set_formula(
                address(1, 4),
                GridFormulaCell::new("=INDEX(A1:B3,2,2)", "test:grid-ref:index"),
            )
            .unwrap();
        let index_report = index_sheet
            .run_engine_mode_with_oxfml(
                GridEngineMode::Both,
                [
                    address(1, 1),
                    address(1, 2),
                    address(2, 1),
                    address(2, 2),
                    address(3, 1),
                    address(3, 2),
                    address(1, 4),
                ],
                100,
            )
            .expect("index harness runs");
        assert!(index_report.mismatches.is_empty());
        assert_eq!(
            index_report.reference.as_ref().unwrap().readout[6].computed,
            CalcValue::number(40.0)
        );

        // COUNTIF aggregates over a range.
        let mut countif_sheet = optimized_sheet();
        countif_sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 5, 1, bounds()).unwrap(),
                vec![
                    CalcValue::number(1.0),
                    CalcValue::number(2.0),
                    CalcValue::number(4.0),
                    CalcValue::number(5.0),
                    CalcValue::number(6.0),
                ],
            )
            .unwrap();
        countif_sheet
            .set_formula(
                address(1, 3),
                GridFormulaCell::new("=COUNTIF(A1:A5,\">3\")", "test:grid-ref:countif"),
            )
            .unwrap();
        let countif_report = countif_sheet
            .run_engine_mode_with_oxfml(
                GridEngineMode::Both,
                [
                    address(1, 1),
                    address(2, 1),
                    address(3, 1),
                    address(4, 1),
                    address(5, 1),
                    address(1, 3),
                ],
                100,
            )
            .expect("countif harness runs");
        assert!(countif_report.mismatches.is_empty());
        assert_eq!(
            countif_report.reference.as_ref().unwrap().readout[5].computed,
            CalcValue::number(3.0)
        );

        // VLOOKUP over an area (lookup family; deferred w050 A05/A06).
        let mut vlookup_sheet = optimized_sheet();
        vlookup_sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 2, bounds()).unwrap(),
                vec![
                    CalcValue::number(1.0),
                    CalcValue::number(10.0),
                    CalcValue::number(2.0),
                    CalcValue::number(20.0),
                    CalcValue::number(3.0),
                    CalcValue::number(30.0),
                ],
            )
            .unwrap();
        vlookup_sheet
            .set_formula(
                address(1, 4),
                GridFormulaCell::new("=VLOOKUP(2,A1:B3,2,FALSE)", "test:grid-ref:vlookup"),
            )
            .unwrap();
        let vlookup_report = vlookup_sheet
            .run_engine_mode_with_oxfml(
                GridEngineMode::Both,
                [
                    address(1, 1),
                    address(1, 2),
                    address(2, 1),
                    address(2, 2),
                    address(3, 1),
                    address(3, 2),
                    address(1, 4),
                ],
                100,
            )
            .expect("vlookup harness runs");
        assert!(vlookup_report.mismatches.is_empty());
        assert_eq!(
            vlookup_report.reference.as_ref().unwrap().readout[6].computed,
            CalcValue::number(20.0)
        );

        // Implicit intersection over a single-cell reference (the `@` operator /
        // _xlfn.SINGLE): resolves to the cell value through the real provider.
        let mut at_sheet = optimized_sheet();
        at_sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 1, 1, bounds()).unwrap(),
                vec![CalcValue::number(42.0)],
            )
            .unwrap();
        at_sheet
            .set_formula(
                address(1, 3),
                GridFormulaCell::new("=@A1", "test:grid-ref:at"),
            )
            .unwrap();
        let at_report = at_sheet
            .run_engine_mode_with_oxfml(GridEngineMode::Both, [address(1, 1), address(1, 3)], 100)
            .expect("implicit-intersection harness runs");
        assert!(at_report.mismatches.is_empty());
        assert_eq!(
            at_report.reference.as_ref().unwrap().readout[1].computed,
            CalcValue::number(42.0)
        );

        // OFFSET returns a reference (via the provider's Offset transform),
        // dereferenced to the cell value.
        let mut offset_sheet = optimized_sheet();
        offset_sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 2, 1, bounds()).unwrap(),
                vec![CalcValue::number(42.0), CalcValue::number(7.0)],
            )
            .unwrap();
        offset_sheet
            .set_formula(
                address(1, 3),
                GridFormulaCell::new("=OFFSET(A1,1,0)", "test:grid-ref:offset"),
            )
            .unwrap();
        let offset_report = offset_sheet
            .run_engine_mode_with_oxfml(
                GridEngineMode::Both,
                [address(1, 1), address(2, 1), address(1, 3)],
                100,
            )
            .expect("offset harness runs");
        assert!(offset_report.mismatches.is_empty());
        assert_eq!(
            offset_report.reference.as_ref().unwrap().readout[2].computed,
            CalcValue::number(7.0)
        );
    }

    #[test]
    fn grid_engine_mode_both_resolves_defined_name_and_indirect_text() {
        let mut sheet = optimized_sheet();
        let input_range =
            GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap();
        sheet
            .put_dense_literal_region(
                input_range.clone(),
                vec![
                    CalcValue::number(2.0),
                    CalcValue::number(4.0),
                    CalcValue::number(6.0),
                ],
            )
            .unwrap();
        sheet
            .set_defined_name("InputRange", input_range.clone())
            .unwrap();
        sheet
            .set_formula(
                address(1, 2),
                GridFormulaCell::new("=SUM(InputRange)", "excel.grid.v1:sum-name:InputRange"),
            )
            .unwrap();
        sheet
            .set_formula(
                address(1, 3),
                GridFormulaCell::new(
                    "=SUM(INDIRECT(\"InputRange\"))",
                    "excel.grid.v1:sum-indirect-name:InputRange",
                ),
            )
            .unwrap();

        let reference = sheet.project_authored_to_reference(100).unwrap();
        assert_eq!(
            reference.defined_names().get("INPUTRANGE"),
            Some(&input_range)
        );

        let report = sheet
            .run_engine_mode_with_oxfml(GridEngineMode::Both, [address(1, 2), address(1, 3)], 100)
            .expect("defined names should resolve through the grid provider in both engines");

        assert!(report.mismatches.is_empty());
        assert_eq!(
            report.reference.as_ref().unwrap().readout[0].computed,
            CalcValue::number(12.0)
        );
        assert_eq!(
            report.reference.as_ref().unwrap().readout[1].computed,
            CalcValue::number(12.0)
        );
        assert_eq!(
            report.optimized.as_ref().unwrap().readout[0].computed,
            CalcValue::number(12.0)
        );
        assert_eq!(
            report.optimized.as_ref().unwrap().readout[1].computed,
            CalcValue::number(12.0)
        );
    }

    #[test]
    fn optimized_grid_defined_name_lifecycle_renames_direct_formula_and_deletes_namespace() {
        let mut sheet = optimized_sheet();
        let input_range =
            GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap();
        sheet
            .put_dense_literal_region(
                input_range.clone(),
                vec![
                    CalcValue::number(2.0),
                    CalcValue::number(4.0),
                    CalcValue::number(6.0),
                ],
            )
            .unwrap();
        sheet
            .set_defined_name("InputRange", input_range.clone())
            .unwrap();
        sheet
            .set_formula(
                address(1, 2),
                GridFormulaCell::new("=SUM(InputRange)", "excel.grid.v1:sum-name:InputRange"),
            )
            .unwrap();
        sheet
            .set_formula(
                address(1, 3),
                GridFormulaCell::new(
                    "=SUM(INDIRECT(\"InputRange\"))",
                    "excel.grid.v1:sum-indirect-name:InputRange",
                ),
            )
            .unwrap();

        let rename = sheet
            .rename_defined_name("InputRange", "DataRange")
            .expect("defined-name rename should move namespace entry and rewrite direct formulas");
        assert_eq!(rename.operation, GridNameLifecycleOperation::Rename);
        assert_eq!(rename.old_name_key.as_deref(), Some("INPUTRANGE"));
        assert_eq!(rename.new_name_key.as_deref(), Some("DATARANGE"));
        assert_eq!(rename.formula_cells_transformed, 1);
        assert_eq!(rename.formula_reference_transforms, 1);
        assert_eq!(sheet.defined_names().get("DATARANGE"), Some(&input_range));
        assert!(!sheet.defined_names().contains_key("INPUTRANGE"));
        match sheet
            .authored_cell_at(&address(1, 2))
            .unwrap()
            .authored
            .unwrap()
        {
            GridAuthoredCell::Formula(formula) => {
                assert_eq!(formula.source_text, "=SUM(DataRange)");
            }
            other => panic!("expected renamed defined-name formula, got {other:?}"),
        }
        match sheet
            .authored_cell_at(&address(1, 3))
            .unwrap()
            .authored
            .unwrap()
        {
            GridAuthoredCell::Formula(formula) => {
                assert_eq!(formula.source_text, "=SUM(INDIRECT(\"InputRange\"))");
            }
            other => panic!("expected indirect defined-name formula, got {other:?}"),
        }

        let renamed_report = sheet
            .run_engine_mode_with_oxfml(GridEngineMode::Both, [address(1, 2), address(1, 3)], 100)
            .expect("defined-name lifecycle should evaluate through both grid engines");
        assert!(renamed_report.mismatches.is_empty());
        assert_eq!(
            renamed_report.reference.as_ref().unwrap().readout[0].computed,
            CalcValue::number(12.0)
        );
        assert_eq!(
            renamed_report.reference.as_ref().unwrap().readout[1].computed,
            CalcValue::error(WorksheetErrorCode::Ref)
        );

        let delete = sheet
            .delete_defined_name("DataRange")
            .expect("defined-name delete should remove namespace entry");
        assert_eq!(delete.operation, GridNameLifecycleOperation::Delete);
        assert_eq!(delete.old_name_key.as_deref(), Some("DATARANGE"));
        assert_eq!(delete.new_name_key, None);
        assert_eq!(delete.formula_cells_transformed, 1);
        assert_eq!(delete.formula_reference_transforms, 1);
        assert!(sheet.defined_names().is_empty());
        match sheet
            .authored_cell_at(&address(1, 2))
            .unwrap()
            .authored
            .unwrap()
        {
            GridAuthoredCell::Formula(formula) => {
                assert_eq!(formula.source_text, "=SUM(#NAME?)");
            }
            other => panic!("expected deleted defined-name formula, got {other:?}"),
        }

        let deleted_report = sheet
            .run_engine_mode_with_oxfml(GridEngineMode::Both, [address(1, 2), address(1, 3)], 100)
            .expect("deleted defined names should become worksheet errors, not engine failures");
        assert!(deleted_report.mismatches.is_empty());
        assert_eq!(
            deleted_report.reference.as_ref().unwrap().readout[0].computed,
            CalcValue::error(WorksheetErrorCode::Name)
        );
        assert_eq!(
            deleted_report.reference.as_ref().unwrap().readout[1].computed,
            CalcValue::error(WorksheetErrorCode::Ref)
        );
        assert!(matches!(
            sheet.delete_defined_name("DataRange"),
            Err(GridRefError::DefinedNameNotFound { name }) if name == "DataRange"
        ));
    }

    #[test]
    fn grid_calc_ref_defined_name_lifecycle_renames_formula_and_deletes_namespace() {
        let mut sheet = sheet();
        let input_range =
            GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap();
        sheet
            .set_literal(address(1, 1), CalcValue::number(2.0))
            .unwrap();
        sheet
            .set_literal(address(2, 1), CalcValue::number(4.0))
            .unwrap();
        sheet
            .set_literal(address(3, 1), CalcValue::number(6.0))
            .unwrap();
        sheet
            .set_defined_name("InputRange", input_range.clone())
            .unwrap();
        sheet
            .set_formula(
                address(1, 2),
                GridFormulaCell::new("=SUM(InputRange)", "excel.grid.v1:sum-name:InputRange"),
            )
            .unwrap();

        let rename = sheet
            .rename_defined_name("InputRange", "DataRange")
            .unwrap();
        assert_eq!(rename.old_name_key.as_deref(), Some("INPUTRANGE"));
        assert_eq!(rename.new_name_key.as_deref(), Some("DATARANGE"));
        assert_eq!(rename.formula_cells_transformed, 1);
        assert_eq!(rename.formula_reference_transforms, 1);
        assert_eq!(sheet.defined_names().get("DATARANGE"), Some(&input_range));
        match sheet.authored.get(&address(1, 2)).unwrap() {
            GridAuthoredCell::Formula(formula) => {
                assert_eq!(formula.source_text, "=SUM(DataRange)");
            }
            other => panic!("expected renamed defined-name formula, got {other:?}"),
        }

        sheet.recalculate_mark_all_dirty_with_oxfml().unwrap();
        assert_eq!(sheet.read_cell(&address(1, 2)), CalcValue::number(12.0));

        let delete = sheet.delete_defined_name("DataRange").unwrap();
        assert_eq!(delete.old_name_key.as_deref(), Some("DATARANGE"));
        assert_eq!(delete.formula_cells_transformed, 1);
        assert_eq!(delete.formula_reference_transforms, 1);
        assert!(sheet.defined_names().is_empty());
        match sheet.authored.get(&address(1, 2)).unwrap() {
            GridAuthoredCell::Formula(formula) => {
                assert_eq!(formula.source_text, "=SUM(#NAME?)");
            }
            other => panic!("expected deleted defined-name formula, got {other:?}"),
        }
        sheet.recalculate_mark_all_dirty_with_oxfml().unwrap();
        assert_eq!(
            sheet.read_cell(&address(1, 2)),
            CalcValue::error(WorksheetErrorCode::Name)
        );
    }

    #[test]
    fn optimized_grid_table_lifecycle_resize_rename_and_delete_updates_provider_shape() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 4, 2, bounds()).unwrap(),
                vec![
                    CalcValue::empty(),
                    CalcValue::empty(),
                    CalcValue::empty(),
                    CalcValue::number(2.0),
                    CalcValue::empty(),
                    CalcValue::number(4.0),
                    CalcValue::empty(),
                    CalcValue::number(6.0),
                ],
            )
            .unwrap();
        sheet
            .set_table_overlay(GridTableOverlay::new(
                "table:lifecycle:input",
                "Table1",
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 2, bounds()).unwrap(),
                vec![
                    GridTableColumn::new(
                        "table1:label",
                        "Label",
                        1,
                        GridRect::new("book:default", "sheet:default", 2, 1, 3, 1, bounds())
                            .unwrap(),
                    ),
                    GridTableColumn::new(
                        "table1:amount",
                        "Amount",
                        2,
                        GridRect::new("book:default", "sheet:default", 2, 2, 3, 2, bounds())
                            .unwrap(),
                    ),
                ],
            ))
            .unwrap();
        assert_eq!(sheet.feature_rendered_regions().len(), 1);

        let resize = sheet
            .resize_table_overlay(GridTableOverlay::new(
                "table:lifecycle:input",
                "Table1",
                GridRect::new("book:default", "sheet:default", 1, 1, 4, 2, bounds()).unwrap(),
                vec![
                    GridTableColumn::new(
                        "table1:label",
                        "Label",
                        1,
                        GridRect::new("book:default", "sheet:default", 2, 1, 4, 1, bounds())
                            .unwrap(),
                    ),
                    GridTableColumn::new(
                        "table1:amount",
                        "Amount",
                        2,
                        GridRect::new("book:default", "sheet:default", 2, 2, 4, 2, bounds())
                            .unwrap(),
                    ),
                ],
            ))
            .unwrap();
        assert_eq!(resize.operation, GridTableLifecycleOperation::Resize);
        assert_eq!(resize.feature_regions_removed, 1);
        assert_eq!(resize.feature_regions_added, 1);
        assert_eq!(sheet.feature_rendered_regions().len(), 1);
        assert_eq!(
            sheet.feature_rendered_regions()[0].rect,
            GridRect::new("book:default", "sheet:default", 1, 1, 4, 2, bounds()).unwrap()
        );

        sheet
            .set_formula(
                address(1, 3),
                GridFormulaCell::new("=SUM(Table1[Amount])", "excel.grid.v1:sum-table:Table1"),
            )
            .unwrap();
        let rename = sheet.rename_table_overlay("Table1", "Sales").unwrap();
        assert_eq!(rename.operation, GridTableLifecycleOperation::Rename);
        assert_eq!(rename.formula_cells_transformed, 1);
        assert_eq!(rename.formula_reference_transforms, 1);
        assert!(sheet.table_overlays().contains_key("SALES"));
        assert!(!sheet.table_overlays().contains_key("TABLE1"));
        assert_eq!(sheet.feature_rendered_regions().len(), 1);
        match sheet
            .authored_cell_at(&address(1, 3))
            .unwrap()
            .authored
            .unwrap()
        {
            GridAuthoredCell::Formula(formula) => {
                assert_eq!(formula.source_text, "=SUM(Sales[Amount])");
            }
            other => panic!("expected renamed table formula, got {other:?}"),
        }
        let report = sheet
            .run_engine_mode_with_oxfml(GridEngineMode::Both, [address(1, 3)], 100)
            .expect("renamed and resized table should resolve through both grid engines");
        assert!(report.mismatches.is_empty());
        assert_eq!(
            report.optimized.as_ref().unwrap().readout[0].computed,
            CalcValue::number(12.0)
        );

        let delete = sheet.delete_table_overlay("Sales").unwrap();
        assert_eq!(delete.operation, GridTableLifecycleOperation::Delete);
        assert_eq!(delete.feature_regions_removed, 1);
        assert_eq!(delete.formula_cells_transformed, 1);
        assert_eq!(delete.formula_reference_transforms, 1);
        assert!(sheet.table_overlays().is_empty());
        assert!(sheet.feature_rendered_regions().is_empty());
        match sheet
            .authored_cell_at(&address(1, 3))
            .unwrap()
            .authored
            .unwrap()
        {
            GridAuthoredCell::Formula(formula) => {
                assert_eq!(formula.source_text, "=SUM(#REF!)");
            }
            other => panic!("expected deleted table formula to become #REF!, got {other:?}"),
        }
        assert!(matches!(
            sheet.delete_table_overlay("Sales"),
            Err(GridRefError::TableOverlayNotFound { name }) if name == "Sales"
        ));
    }

    #[test]
    fn grid_calc_ref_table_lifecycle_replaces_and_deletes_feature_claims() {
        let mut sheet = sheet();
        sheet
            .set_table_overlay(GridTableOverlay::new(
                "table:lifecycle:ref",
                "Table1",
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 2, bounds()).unwrap(),
                vec![
                    GridTableColumn::new(
                        "table1:label",
                        "Label",
                        1,
                        GridRect::new("book:default", "sheet:default", 2, 1, 3, 1, bounds())
                            .unwrap(),
                    ),
                    GridTableColumn::new(
                        "table1:amount",
                        "Amount",
                        2,
                        GridRect::new("book:default", "sheet:default", 2, 2, 3, 2, bounds())
                            .unwrap(),
                    ),
                ],
            ))
            .unwrap();
        sheet
            .set_table_overlay(GridTableOverlay::new(
                "table:lifecycle:ref",
                "Table1",
                GridRect::new("book:default", "sheet:default", 1, 1, 4, 2, bounds()).unwrap(),
                vec![
                    GridTableColumn::new(
                        "table1:label",
                        "Label",
                        1,
                        GridRect::new("book:default", "sheet:default", 2, 1, 4, 1, bounds())
                            .unwrap(),
                    ),
                    GridTableColumn::new(
                        "table1:amount",
                        "Amount",
                        2,
                        GridRect::new("book:default", "sheet:default", 2, 2, 4, 2, bounds())
                            .unwrap(),
                    ),
                ],
            ))
            .unwrap();

        assert_eq!(
            sheet.feature_rendered_regions(),
            &[FeatureRenderedRegion {
                rect: GridRect::new("book:default", "sheet:default", 1, 1, 4, 2, bounds()).unwrap(),
                feature_kind: "table-overlay".to_string(),
                needs_refresh: false,
            }]
        );

        sheet
            .set_formula(
                address(1, 3),
                GridFormulaCell::new("=SUM(Table1[Amount])", "excel.grid.v1:sum-table:Table1"),
            )
            .unwrap();
        let rename = sheet.rename_table_overlay("Table1", "Sales").unwrap();
        assert_eq!(rename.old_table_key.as_deref(), Some("TABLE1"));
        assert_eq!(rename.new_table_key.as_deref(), Some("SALES"));
        assert_eq!(rename.formula_cells_transformed, 1);
        assert_eq!(rename.formula_reference_transforms, 1);
        assert!(sheet.table_overlays().contains_key("SALES"));
        match sheet.authored.get(&address(1, 3)).unwrap() {
            GridAuthoredCell::Formula(formula) => {
                assert_eq!(formula.source_text, "=SUM(Sales[Amount])");
            }
            other => panic!("expected renamed table formula, got {other:?}"),
        }

        let delete = sheet.delete_table_overlay("Sales").unwrap();
        assert_eq!(delete.feature_regions_removed, 1);
        assert_eq!(delete.formula_cells_transformed, 1);
        assert_eq!(delete.formula_reference_transforms, 1);
        assert!(sheet.table_overlays().is_empty());
        assert!(sheet.feature_rendered_regions().is_empty());
        match sheet.authored.get(&address(1, 3)).unwrap() {
            GridAuthoredCell::Formula(formula) => {
                assert_eq!(formula.source_text, "=SUM(#REF!)");
            }
            other => panic!("expected deleted table formula to become #REF!, got {other:?}"),
        }
    }

    #[test]
    fn grid_engine_mode_both_resolves_table_overlay_structured_reference_and_indirect_text() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 4, 2, bounds()).unwrap(),
                vec![
                    CalcValue::empty(),
                    CalcValue::empty(),
                    CalcValue::empty(),
                    CalcValue::number(2.0),
                    CalcValue::empty(),
                    CalcValue::number(4.0),
                    CalcValue::empty(),
                    CalcValue::number(6.0),
                ],
            )
            .unwrap();
        let amount_data =
            GridRect::new("book:default", "sheet:default", 2, 2, 4, 2, bounds()).unwrap();
        sheet
            .set_table_overlay(
                GridTableOverlay::new(
                    "table:default:table1",
                    "Table1",
                    GridRect::new("book:default", "sheet:default", 1, 1, 4, 2, bounds()).unwrap(),
                    vec![
                        GridTableColumn::new(
                            "table1:label",
                            "Label",
                            1,
                            GridRect::new("book:default", "sheet:default", 2, 1, 4, 1, bounds())
                                .unwrap(),
                        ),
                        GridTableColumn::new("table1:amount", "Amount", 2, amount_data.clone()),
                    ],
                )
                .with_header_rect(
                    GridRect::new("book:default", "sheet:default", 1, 1, 1, 2, bounds()).unwrap(),
                ),
            )
            .unwrap();
        sheet
            .set_formula(
                address(1, 3),
                GridFormulaCell::new(
                    "=SUM(Table1[Amount])",
                    "excel.grid.v1:sum-table:Table1[Amount]",
                ),
            )
            .unwrap();
        sheet
            .set_formula(
                address(1, 4),
                GridFormulaCell::new(
                    "=SUM(INDIRECT(\"Table1[Amount]\"))",
                    "excel.grid.v1:sum-indirect-table:Table1[Amount]",
                ),
            )
            .unwrap();

        let reference = sheet.project_authored_to_reference(100).unwrap();
        assert!(reference.table_overlays().contains_key("TABLE1"));

        let report = sheet
            .run_engine_mode_with_oxfml(GridEngineMode::Both, [address(1, 3), address(1, 4)], 100)
            .expect("table structured refs should resolve through the grid provider");

        assert!(report.mismatches.is_empty());
        assert_eq!(
            report.reference.as_ref().unwrap().readout[0].computed,
            CalcValue::number(12.0)
        );
        assert_eq!(
            report.reference.as_ref().unwrap().readout[1].computed,
            CalcValue::number(12.0)
        );
        assert_eq!(
            report.optimized.as_ref().unwrap().readout[0].computed,
            CalcValue::number(12.0)
        );
        assert_eq!(
            report.optimized.as_ref().unwrap().readout[1].computed,
            CalcValue::number(12.0)
        );
    }

    #[test]
    fn grid_engine_mode_both_resolves_table_caller_context_references() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 4, 2, bounds()).unwrap(),
                vec![
                    CalcValue::empty(),
                    CalcValue::empty(),
                    CalcValue::empty(),
                    CalcValue::number(2.0),
                    CalcValue::empty(),
                    CalcValue::number(4.0),
                    CalcValue::empty(),
                    CalcValue::number(6.0),
                ],
            )
            .unwrap();
        sheet
            .set_table_overlay(
                GridTableOverlay::new(
                    "table:default:table1",
                    "Table1",
                    GridRect::new("book:default", "sheet:default", 1, 1, 4, 4, bounds()).unwrap(),
                    vec![
                        GridTableColumn::new(
                            "table1:label",
                            "Label",
                            1,
                            GridRect::new("book:default", "sheet:default", 2, 1, 4, 1, bounds())
                                .unwrap(),
                        ),
                        GridTableColumn::new(
                            "table1:amount",
                            "Amount",
                            2,
                            GridRect::new("book:default", "sheet:default", 2, 2, 4, 2, bounds())
                                .unwrap(),
                        ),
                        GridTableColumn::new(
                            "table1:double",
                            "Double",
                            3,
                            GridRect::new("book:default", "sheet:default", 2, 3, 4, 3, bounds())
                                .unwrap(),
                        ),
                        GridTableColumn::new(
                            "table1:amount-total",
                            "AmountTotal",
                            4,
                            GridRect::new("book:default", "sheet:default", 2, 4, 4, 4, bounds())
                                .unwrap(),
                        ),
                    ],
                )
                .with_header_rect(
                    GridRect::new("book:default", "sheet:default", 1, 1, 1, 4, bounds()).unwrap(),
                ),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 2, 3, 4, 3, bounds()).unwrap(),
                GridFormulaCell::new("=[@Amount]*2", "excel.grid.v1:table-this-row:Amount*2"),
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 2, 4, 4, 4, bounds()).unwrap(),
                GridFormulaCell::new("=SUM([Amount])", "excel.grid.v1:table-omitted:sum:Amount"),
            )
            .unwrap();

        let report = sheet
            .run_engine_mode_with_oxfml(
                GridEngineMode::Both,
                [
                    address(2, 3),
                    address(3, 3),
                    address(4, 3),
                    address(2, 4),
                    address(3, 4),
                    address(4, 4),
                ],
                100,
            )
            .expect("table caller context should resolve omitted structured refs");

        assert!(report.mismatches.is_empty());
        assert_eq!(
            report.reference.as_ref().unwrap().readout,
            vec![
                GridEngineCellReadout {
                    address: address(2, 3),
                    computed: CalcValue::number(4.0),
                },
                GridEngineCellReadout {
                    address: address(3, 3),
                    computed: CalcValue::number(8.0),
                },
                GridEngineCellReadout {
                    address: address(4, 3),
                    computed: CalcValue::number(12.0),
                },
                GridEngineCellReadout {
                    address: address(2, 4),
                    computed: CalcValue::number(12.0),
                },
                GridEngineCellReadout {
                    address: address(3, 4),
                    computed: CalcValue::number(12.0),
                },
                GridEngineCellReadout {
                    address: address(4, 4),
                    computed: CalcValue::number(12.0),
                },
            ]
        );
        assert_eq!(
            report.reference.as_ref().unwrap().readout,
            report.optimized.as_ref().unwrap().readout
        );
    }

    #[test]
    fn grid_engine_mode_both_resolves_section_qualified_and_escaped_table_references() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 5, 3, bounds()).unwrap(),
                vec![
                    CalcValue::empty(),
                    CalcValue::empty(),
                    CalcValue::empty(),
                    CalcValue::empty(),
                    CalcValue::number(2.0),
                    CalcValue::number(1.0),
                    CalcValue::empty(),
                    CalcValue::number(4.0),
                    CalcValue::number(2.0),
                    CalcValue::empty(),
                    CalcValue::number(6.0),
                    CalcValue::number(3.0),
                    CalcValue::empty(),
                    CalcValue::number(12.0),
                    CalcValue::number(6.0),
                ],
            )
            .unwrap();
        sheet
            .set_table_overlay(
                GridTableOverlay::new(
                    "table:default:table1",
                    "Table1",
                    GridRect::new("book:default", "sheet:default", 1, 1, 5, 3, bounds()).unwrap(),
                    vec![
                        GridTableColumn::new(
                            "table1:label",
                            "Label",
                            1,
                            GridRect::new("book:default", "sheet:default", 2, 1, 4, 1, bounds())
                                .unwrap(),
                        ),
                        GridTableColumn::new(
                            "table1:amount",
                            "Amount",
                            2,
                            GridRect::new("book:default", "sheet:default", 2, 2, 4, 2, bounds())
                                .unwrap(),
                        ),
                        GridTableColumn::new(
                            "table1:tax",
                            "Tax",
                            3,
                            GridRect::new("book:default", "sheet:default", 2, 3, 4, 3, bounds())
                                .unwrap(),
                        ),
                    ],
                )
                .with_header_rect(
                    GridRect::new("book:default", "sheet:default", 1, 1, 1, 3, bounds()).unwrap(),
                )
                .with_totals_rect(
                    GridRect::new("book:default", "sheet:default", 5, 1, 5, 3, bounds()).unwrap(),
                ),
            )
            .unwrap();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 6, 1, 9, 3, bounds()).unwrap(),
                vec![
                    CalcValue::empty(),
                    CalcValue::empty(),
                    CalcValue::empty(),
                    CalcValue::empty(),
                    CalcValue::number(2.0),
                    CalcValue::number(1.0),
                    CalcValue::empty(),
                    CalcValue::number(4.0),
                    CalcValue::number(2.0),
                    CalcValue::empty(),
                    CalcValue::number(6.0),
                    CalcValue::number(3.0),
                ],
            )
            .unwrap();
        sheet
            .set_table_overlay(
                GridTableOverlay::new(
                    "table:default:escaped",
                    "TableEsc",
                    GridRect::new("book:default", "sheet:default", 6, 1, 9, 3, bounds()).unwrap(),
                    vec![
                        GridTableColumn::new(
                            "table-esc:label",
                            "Label",
                            1,
                            GridRect::new("book:default", "sheet:default", 7, 1, 9, 1, bounds())
                                .unwrap(),
                        ),
                        GridTableColumn::new(
                            "table-esc:hash-data",
                            "#Data",
                            2,
                            GridRect::new("book:default", "sheet:default", 7, 2, 9, 2, bounds())
                                .unwrap(),
                        ),
                        GridTableColumn::new(
                            "table-esc:gross-margin",
                            "Gross]Margin",
                            3,
                            GridRect::new("book:default", "sheet:default", 7, 3, 9, 3, bounds())
                                .unwrap(),
                        ),
                    ],
                )
                .with_header_rect(
                    GridRect::new("book:default", "sheet:default", 6, 1, 6, 3, bounds()).unwrap(),
                ),
            )
            .unwrap();
        sheet
            .set_formula(
                address(1, 4),
                GridFormulaCell::new(
                    "=SUM(Table1[[#Data],[Amount]:[Tax]])",
                    "excel.grid.v1:sum-table-data:Table1[Amount:Tax]",
                ),
            )
            .unwrap();
        sheet
            .set_formula(
                address(2, 4),
                GridFormulaCell::new(
                    "=SUM(INDIRECT(\"Table1[[#Data],[Amount]:[Tax]]\"))",
                    "excel.grid.v1:sum-indirect-table-data:Table1[Amount:Tax]",
                ),
            )
            .unwrap();
        sheet
            .set_formula(
                address(3, 4),
                GridFormulaCell::new(
                    "=SUM(Table1[[#Totals],[Amount]:[Tax]])",
                    "excel.grid.v1:sum-table-totals:Table1[Amount:Tax]",
                ),
            )
            .unwrap();
        sheet
            .set_formula(
                address(4, 4),
                GridFormulaCell::new(
                    "=SUM(Table1[[#All],[Amount]:[Tax]])",
                    "excel.grid.v1:sum-table-all:Table1[Amount:Tax]",
                ),
            )
            .unwrap();
        sheet
            .set_formula(
                address(5, 4),
                GridFormulaCell::new(
                    "=SUM(TableEsc['#Data])",
                    "excel.grid.v1:sum-table-escaped:TableEsc[#Data]",
                ),
            )
            .unwrap();
        sheet
            .set_formula(
                address(6, 4),
                GridFormulaCell::new(
                    "=SUM(TableEsc[[#Data],['#Data]:[Gross']Margin]])",
                    "excel.grid.v1:sum-table-escaped-range:TableEsc[#Data:GrossMargin]",
                ),
            )
            .unwrap();
        sheet
            .set_formula(
                address(7, 4),
                GridFormulaCell::new(
                    "=SUM(Table1[[#Headers],[#Totals],[Amount]:[Tax]])",
                    "excel.grid.v1:sum-table-union:Table1[Headers+Totals,Amount:Tax]",
                ),
            )
            .unwrap();
        sheet
            .set_formula(
                address(8, 4),
                GridFormulaCell::new(
                    "=SUM(INDIRECT(\"Table1[[#Headers],[#Totals],[Amount]:[Tax]]\"))",
                    "excel.grid.v1:sum-indirect-table-union:Table1[Headers+Totals,Amount:Tax]",
                ),
            )
            .unwrap();

        let report = sheet
            .run_engine_mode_with_oxfml(
                GridEngineMode::Both,
                [
                    address(1, 4),
                    address(2, 4),
                    address(3, 4),
                    address(4, 4),
                    address(5, 4),
                    address(6, 4),
                    address(7, 4),
                    address(8, 4),
                ],
                100,
            )
            .expect("section-qualified and escaped structured refs should resolve");

        assert!(report.mismatches.is_empty());
        let optimized = &report.optimized.as_ref().unwrap().readout;
        assert_eq!(optimized[0].computed, CalcValue::number(18.0));
        assert_eq!(optimized[1].computed, CalcValue::number(18.0));
        assert_eq!(optimized[2].computed, CalcValue::number(18.0));
        assert_eq!(optimized[3].computed, CalcValue::number(36.0));
        assert_eq!(optimized[4].computed, CalcValue::number(12.0));
        assert_eq!(optimized[5].computed, CalcValue::number(18.0));
        assert_eq!(optimized[6].computed, CalcValue::number(18.0));
        assert_eq!(optimized[7].computed, CalcValue::number(18.0));
    }

    #[test]
    fn optimized_grid_evaluates_spill_anchor_through_committed_ledger() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap(),
                vec![
                    CalcValue::number(4.0),
                    CalcValue::number(8.0),
                    CalcValue::number(16.0),
                ],
            )
            .unwrap();
        sheet
            .set_spill_fact(GridSpillFact {
                anchor: address(1, 1),
                extent: GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds())
                    .unwrap(),
                blocked: false,
            })
            .unwrap();
        sheet
            .set_formula(
                address(1, 2),
                GridFormulaCell::new("=SUM(A1#)", "excel.grid.v1:sum-spill:R[0]C[-1]#"),
            )
            .unwrap();

        let reference = sheet.project_authored_to_reference(100).unwrap();
        assert_eq!(reference.spill_facts().len(), 1);

        let report = sheet
            .run_engine_mode_with_oxfml(GridEngineMode::Both, [address(1, 2)], 100)
            .expect(
                "optimized and reference engines should both resolve A1# from the spill ledger",
            );

        assert!(report.mismatches.is_empty());
        assert_eq!(
            report.optimized.as_ref().unwrap().readout[0].computed,
            CalcValue::number(28.0)
        );
    }

    #[test]
    fn grid_engine_mode_both_matches_repeated_r1c1_region() {
        let mut sheet = optimized_sheet();
        sheet
            .put_dense_literal_region(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap(),
                vec![
                    CalcValue::number(10.0),
                    CalcValue::number(20.0),
                    CalcValue::number(30.0),
                ],
            )
            .unwrap();
        sheet
            .put_repeated_formula_region(
                GridRect::new("book:default", "sheet:default", 1, 2, 3, 2, bounds()).unwrap(),
                GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let report = sheet
            .run_engine_mode_with_oxfml(
                GridEngineMode::Both,
                [address(1, 2), address(2, 2), address(3, 2)],
                100,
            )
            .expect("both-mode harness should compare materialized and compact R1C1 recalc");

        assert!(report.mismatches.is_empty());
        assert_eq!(
            report.optimized.as_ref().unwrap().readout,
            vec![
                GridEngineCellReadout {
                    address: address(1, 2),
                    computed: CalcValue::number(20.0),
                },
                GridEngineCellReadout {
                    address: address(2, 2),
                    computed: CalcValue::number(40.0),
                },
                GridEngineCellReadout {
                    address: address(3, 2),
                    computed: CalcValue::number(60.0),
                },
            ]
        );
        assert!(matches!(
            &report.optimized.as_ref().unwrap().recalc,
            GridEngineRecalcReport::Optimized(recalc)
                if recalc.p11_template_prepare_once_holds()
                    && recalc.formula_templates_prepared == 1
        ));
    }

    #[test]
    fn grid_engine_mode_optimized_runs_without_reference_claim() {
        let mut sheet = optimized_sheet();
        sheet
            .set_literal(address(1, 1), CalcValue::number(7.0))
            .unwrap();

        let report = sheet
            .run_engine_mode_with_oxfml(GridEngineMode::Optimized, [address(1, 1)], 100)
            .expect("optimized-only mode should not require reference projection");

        assert_eq!(report.mode, GridEngineMode::Optimized);
        assert!(report.reference.is_none());
        assert!(report.optimized.is_some());
        assert!(report.mismatches.is_empty());
        assert_eq!(
            report.optimized.unwrap().readout,
            vec![GridEngineCellReadout {
                address: address(1, 1),
                computed: CalcValue::number(7.0),
            }]
        );
    }

    #[test]
    fn grid_calc_ref_enforces_bounds_and_produces_sampled_readout() {
        let mut sheet = sheet();
        let a1 = sheet.address(1, 1).unwrap();
        let b2 = sheet.address(2, 2).unwrap();
        let c3 = sheet.address(3, 3).unwrap();

        sheet
            .set_literal(a1.clone(), CalcValue::number(7.0))
            .expect("literal should be in bounds");
        sheet
            .set_formula(
                b2.clone(),
                GridFormulaCell::new("=A1*6", "excel.grid.v1:cell:R[-1]C[-1]*6"),
            )
            .expect("formula should be in bounds");

        assert_eq!(
            sheet.address(11, 1),
            Err(GridRefError::AddressOutOfBounds {
                row: 11,
                col: 1,
                max_rows: 10,
                max_cols: 5,
            })
        );

        let report = sheet.recalculate_mark_all_dirty(|request| {
            assert_eq!(request.address, &b2);
            assert_eq!(request.formula.source_text, "=A1*6");
            CalcValue::number(42.0)
        });

        assert!(report.p00_non_spill_exact_once_holds());
        assert_eq!(report.occupied_cells, 2);
        assert_eq!(report.literal_cells, 1);
        assert_eq!(report.formula_cells, 1);
        assert_eq!(report.formula_evaluations, 1);
        assert_eq!(report.visited_cells, vec![a1.clone(), b2.clone()]);

        let readout = sheet.sampled_readout([a1.clone(), b2.clone(), c3.clone()]);
        assert_eq!(readout[0].computed, CalcValue::number(7.0));
        assert_eq!(readout[1].computed, CalcValue::number(42.0));
        assert_eq!(readout[2].computed, CalcValue::empty());
        assert!(readout[2].authored.is_none());
    }

    #[test]
    fn grid_calc_ref_materializes_dense_literal_region_without_formula_evaluation() {
        let mut sheet = sheet();
        let rect = GridRect::new("book:default", "sheet:default", 1, 1, 4, 4, bounds()).unwrap();

        let materialization = sheet
            .materialize_literal_region(rect.clone(), |address| {
                CalcValue::number(f64::from((address.row * 100) + address.col))
            })
            .expect("dense literal region should materialize");

        assert_eq!(
            materialization,
            GridRegionMaterializationReport {
                cells_written: 16,
                rect
            }
        );

        let report = sheet.recalculate_mark_all_dirty(|_| {
            panic!("dense literal-only region should not evaluate formulas")
        });

        assert!(report.p00_non_spill_exact_once_holds());
        assert_eq!(report.occupied_cells, 16);
        assert_eq!(report.literal_cells, 16);
        assert_eq!(report.formula_cells, 0);
        assert_eq!(report.formula_evaluations, 0);
        assert_eq!(sheet.read_cell(&address(4, 4)), CalcValue::number(404.0));
    }

    #[test]
    fn grid_calc_ref_materializes_repeated_formula_region_and_punch_through_override() {
        let mut sheet = sheet();
        let rect = GridRect::new("book:default", "sheet:default", 2, 2, 3, 3, bounds()).unwrap();
        let formula = GridFormulaCell::new("=R[-1]C", "excel.grid.v1:r1c1-template:R[-1]C");

        let materialization = sheet
            .materialize_formula_region(rect, formula.clone())
            .expect("repeated formula region should materialize");
        sheet
            .set_literal(address(3, 3), CalcValue::number(99.0))
            .expect("literal punch-through should replace one materialized formula");

        assert_eq!(materialization.cells_written, 4);
        let formula_normal_forms = sheet
            .authored()
            .values()
            .filter_map(|cell| match cell {
                GridAuthoredCell::Formula(formula) => Some(formula.normal_form_key.clone()),
                GridAuthoredCell::Literal(_) => None,
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            formula_normal_forms,
            ["excel.grid.v1:r1c1-template:R[-1]C".to_string()]
                .into_iter()
                .collect()
        );

        let report = sheet.recalculate_mark_all_dirty(|request| {
            assert_eq!(request.formula.normal_form_key, formula.normal_form_key);
            CalcValue::number(f64::from((request.address.row * 10) + request.address.col))
        });

        assert!(report.p00_non_spill_exact_once_holds());
        assert_eq!(report.occupied_cells, 4);
        assert_eq!(report.formula_cells, 3);
        assert_eq!(report.literal_cells, 1);
        assert_eq!(report.formula_evaluations, 3);
        assert_eq!(sheet.read_cell(&address(2, 2)), CalcValue::number(22.0));
        assert_eq!(sheet.read_cell(&address(3, 3)), CalcValue::number(99.0));
    }

    #[test]
    fn grid_calc_ref_exports_computed_values_to_grid_provider() {
        let mut sheet = sheet();
        let b3 = sheet.address(3, 2).unwrap();
        sheet
            .set_literal(b3.clone(), CalcValue::number(15.0))
            .expect("literal should be in bounds");
        sheet.recalculate_mark_all_dirty(|_| CalcValue::empty());

        let provider = sheet.reference_system_provider(5, 4);
        let reference = bind_a1_reference_for_provider_test("B3", 5, 4);
        let value = oxfunc_core::resolver::ReferenceSystemProvider::dereference(
            &provider,
            &oxfunc_core::resolver::ReferenceDereferenceRequest { reference },
        )
        .expect("grid provider should dereference exported computed value");

        assert_eq!(value, CalcValue::number(15.0));
    }

    #[test]
    fn grid_calc_ref_publishes_array_formula_as_spill_extent() {
        let mut sheet = sheet();
        sheet
            .set_formula(
                address(1, 1),
                GridFormulaCell::new("={1;2;3}", "excel.grid.v1:test-array:R1C1#"),
            )
            .unwrap();

        let report = sheet.recalculate_mark_all_dirty(|_| {
            CalcValue::array(
                oxfunc_core::value::CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0)],
                    vec![CalcValue::number(2.0)],
                    vec![CalcValue::number(3.0)],
                ])
                .unwrap(),
            )
        });

        assert!(report.p00_non_spill_exact_once_holds());
        assert_eq!(report.spill_facts_published, 1);
        assert_eq!(report.spill_facts_blocked, 0);
        assert_eq!(report.spill_ghost_cells_published, 2);
        assert_eq!(sheet.read_cell(&address(1, 1)), CalcValue::number(1.0));
        assert_eq!(sheet.read_cell(&address(2, 1)), CalcValue::number(2.0));
        assert_eq!(sheet.read_cell(&address(3, 1)), CalcValue::number(3.0));
        assert_eq!(
            sheet.spill_facts().get(&address(1, 1)).unwrap().extent,
            GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap()
        );
        assert_eq!(
            sheet.sampled_readout([address(2, 1)])[0].spill_anchor,
            Some(address(1, 1))
        );
    }

    #[test]
    fn grid_calc_ref_spill_epoch_ledger_preserves_and_advances_anchor_epochs() {
        let mut sheet = sheet();
        sheet
            .set_formula(
                address(1, 1),
                GridFormulaCell::new("={1;2;3}", "excel.grid.v1:test-array:R1C1#"),
            )
            .unwrap();

        let mut values = vec![1.0, 2.0, 3.0];
        sheet.recalculate_mark_all_dirty(|request| {
            if request.address == &address(1, 1) {
                array_col(values.clone())
            } else {
                CalcValue::empty()
            }
        });
        let first = sheet
            .spill_epoch_ledger()
            .snapshot_for(&address(1, 1))
            .cloned()
            .unwrap();
        assert_eq!(first.value_epoch, 1);
        assert_eq!(
            first.extent,
            GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap()
        );

        sheet.recalculate_mark_all_dirty(|request| {
            if request.address == &address(1, 1) {
                array_col(values.clone())
            } else {
                CalcValue::empty()
            }
        });
        assert_eq!(
            sheet
                .spill_epoch_ledger()
                .snapshot_for(&address(1, 1))
                .unwrap()
                .value_epoch,
            1
        );

        values = vec![1.0, 2.0, 4.0];
        sheet.recalculate_mark_all_dirty(|request| {
            if request.address == &address(1, 1) {
                array_col(values.clone())
            } else {
                CalcValue::empty()
            }
        });
        assert_eq!(
            sheet
                .spill_epoch_ledger()
                .snapshot_for(&address(1, 1))
                .unwrap()
                .value_epoch,
            2
        );

        values = vec![1.0, 2.0];
        sheet.recalculate_mark_all_dirty(|request| {
            if request.address == &address(1, 1) {
                array_col(values.clone())
            } else {
                CalcValue::empty()
            }
        });
        let shape_changed = sheet
            .spill_epoch_ledger()
            .snapshot_for(&address(1, 1))
            .cloned()
            .unwrap();
        assert_eq!(shape_changed.value_epoch, 3);
        assert_eq!(
            shape_changed.extent,
            GridRect::new("book:default", "sheet:default", 1, 1, 2, 1, bounds()).unwrap()
        );

        values = vec![1.0, 2.0, 3.0];
        sheet
            .set_literal(address(2, 1), CalcValue::number(99.0))
            .unwrap();
        sheet.recalculate_mark_all_dirty(|request| {
            if request.address == &address(1, 1) {
                array_col(values.clone())
            } else {
                CalcValue::empty()
            }
        });
        let blocked = sheet
            .spill_epoch_ledger()
            .snapshot_for(&address(1, 1))
            .cloned()
            .unwrap();
        assert_eq!(blocked.value_epoch, 4);
        assert!(blocked.blocked);
        assert_eq!(
            sheet.read_cell(&address(1, 1)),
            CalcValue::error(WorksheetErrorCode::Spill)
        );
    }

    #[test]
    fn optimized_grid_spill_epoch_ledger_preserves_and_advances_anchor_epochs() {
        let mut sheet = optimized_sheet();
        sheet
            .set_formula(
                address(1, 1),
                GridFormulaCell::new("={1;2;3}", "excel.grid.v1:test-array:R1C1#"),
            )
            .unwrap();

        let mut values = vec![1.0, 2.0, 3.0];
        let (valuation, _) = sheet
            .recalculate_mark_all_dirty_compact(100, |request| {
                if request.address == &address(1, 1) {
                    array_col(values.clone())
                } else {
                    CalcValue::empty()
                }
            })
            .unwrap();
        assert_eq!(
            valuation
                .spill_epoch_ledger()
                .snapshot_for(&address(1, 1))
                .unwrap()
                .value_epoch,
            1
        );
        let commit = sheet
            .commit_spill_publication_from_valuation(&valuation)
            .unwrap();
        assert_eq!(commit.ledger_update.anchors_added, 1);
        assert_eq!(commit.committed_epoch_anchors, 1);

        let (valuation, _) = sheet
            .recalculate_mark_all_dirty_compact(100, |request| {
                if request.address == &address(1, 1) {
                    array_col(values.clone())
                } else {
                    CalcValue::empty()
                }
            })
            .unwrap();
        assert_eq!(
            valuation
                .spill_epoch_ledger()
                .snapshot_for(&address(1, 1))
                .unwrap()
                .value_epoch,
            1
        );
        let commit = sheet
            .commit_spill_publication_from_valuation(&valuation)
            .unwrap();
        assert_eq!(commit.ledger_update.epochs_preserved, 1);
        assert_eq!(sheet.spill_epoch_ledger().entries().len(), 1);

        values = vec![1.0, 2.0, 4.0];
        let (valuation, _) = sheet
            .recalculate_mark_all_dirty_compact(100, |request| {
                if request.address == &address(1, 1) {
                    array_col(values.clone())
                } else {
                    CalcValue::empty()
                }
            })
            .unwrap();
        assert_eq!(
            valuation
                .spill_epoch_ledger()
                .snapshot_for(&address(1, 1))
                .unwrap()
                .value_epoch,
            2
        );
        let commit = sheet
            .commit_spill_publication_from_valuation(&valuation)
            .unwrap();
        assert_eq!(commit.ledger_update.value_changed_anchors, 1);
        assert_eq!(
            sheet
                .spill_epoch_ledger()
                .snapshot_for(&address(1, 1))
                .unwrap()
                .value_epoch,
            2
        );

        values = vec![1.0, 2.0];
        let (valuation, _) = sheet
            .recalculate_mark_all_dirty_compact(100, |request| {
                if request.address == &address(1, 1) {
                    array_col(values.clone())
                } else {
                    CalcValue::empty()
                }
            })
            .unwrap();
        let shape_changed = valuation
            .spill_epoch_ledger()
            .snapshot_for(&address(1, 1))
            .cloned()
            .unwrap();
        assert_eq!(shape_changed.value_epoch, 3);
        assert_eq!(
            shape_changed.extent,
            GridRect::new("book:default", "sheet:default", 1, 1, 2, 1, bounds()).unwrap()
        );
        let commit = sheet
            .commit_spill_publication_from_valuation(&valuation)
            .unwrap();
        assert_eq!(commit.ledger_update.extent_changed_anchors, 1);
        assert_eq!(
            sheet
                .spill_epoch_ledger()
                .snapshot_for(&address(1, 1))
                .unwrap()
                .extent,
            GridRect::new("book:default", "sheet:default", 1, 1, 2, 1, bounds()).unwrap()
        );

        values = vec![1.0, 2.0, 3.0];
        sheet
            .set_literal(address(2, 1), CalcValue::number(99.0))
            .unwrap();
        let (valuation, _) = sheet
            .recalculate_mark_all_dirty_compact(100, |request| {
                if request.address == &address(1, 1) {
                    array_col(values.clone())
                } else {
                    CalcValue::empty()
                }
            })
            .unwrap();
        let blocked = valuation
            .spill_epoch_ledger()
            .snapshot_for(&address(1, 1))
            .cloned()
            .unwrap();
        assert_eq!(blocked.value_epoch, 4);
        assert!(blocked.blocked);
        assert_eq!(
            valuation.read_cell(&address(1, 1)).computed,
            CalcValue::error(WorksheetErrorCode::Spill)
        );
        let commit = sheet
            .commit_spill_publication_from_valuation(&valuation)
            .unwrap();
        assert_eq!(commit.ledger_update.extent_changed_anchors, 1);
        assert_eq!(commit.ledger_update.value_changed_anchors, 1);
        assert_eq!(commit.ledger_update.blocked_changed_anchors, 1);
        let committed_blocked = sheet
            .spill_epoch_ledger()
            .snapshot_for(&address(1, 1))
            .unwrap();
        assert_eq!(committed_blocked.value_epoch, 4);
        assert!(committed_blocked.blocked);
    }

    #[test]
    fn optimized_grid_rejects_spill_publication_commit_from_different_grid() {
        let mut sheet = optimized_sheet();
        let valuation = GridOptimizedValuation::new(
            "book:other",
            "sheet:default",
            ExcelGridBounds {
                max_rows: 10,
                max_cols: 5,
            },
        );

        assert!(matches!(
            sheet.commit_spill_publication_from_valuation(&valuation),
            Err(GridRefError::ValuationGridIdentityMismatch {
                actual_workbook_id,
                ..
            }) if actual_workbook_id == "book:other"
        ));
    }

    #[test]
    fn grid_calc_ref_blocks_array_formula_when_authored_cell_occupies_extent() {
        let mut sheet = sheet();
        sheet
            .set_formula(
                address(1, 1),
                GridFormulaCell::new("={1;2}", "excel.grid.v1:test-array:R1C1#"),
            )
            .unwrap();
        sheet
            .set_literal(address(2, 1), CalcValue::number(99.0))
            .unwrap();

        let report = sheet.recalculate_mark_all_dirty(|request| {
            if request.address == &address(1, 1) {
                CalcValue::array(
                    oxfunc_core::value::CalcArray::from_rows(vec![
                        vec![CalcValue::number(1.0)],
                        vec![CalcValue::number(2.0)],
                    ])
                    .unwrap(),
                )
            } else {
                CalcValue::empty()
            }
        });

        assert!(report.p00_non_spill_exact_once_holds());
        assert_eq!(report.spill_facts_published, 0);
        assert_eq!(report.spill_facts_blocked, 1);
        assert_eq!(
            sheet.read_cell(&address(1, 1)),
            CalcValue::error(WorksheetErrorCode::Spill)
        );
        assert_eq!(sheet.read_cell(&address(2, 1)), CalcValue::number(99.0));
        let fact = sheet.spill_facts().get(&address(1, 1)).unwrap();
        assert!(fact.blocked);
        assert_eq!(
            fact.extent,
            GridRect::new("book:default", "sheet:default", 1, 1, 2, 1, bounds()).unwrap()
        );
    }

    #[test]
    fn grid_calc_ref_blocks_array_formula_when_table_overlay_occupies_extent() {
        let mut sheet = sheet();
        sheet
            .set_formula(
                address(1, 1),
                GridFormulaCell::new("={1;2}", "excel.grid.v1:test-array:R1C1#"),
            )
            .unwrap();
        sheet.add_feature_rendered_region(
            GridRect::new("book:default", "sheet:default", 2, 1, 2, 1, bounds()).unwrap(),
            "table-overlay",
            false,
        );

        let report = sheet.recalculate_mark_all_dirty(|_| {
            CalcValue::array(
                oxfunc_core::value::CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0)],
                    vec![CalcValue::number(2.0)],
                ])
                .unwrap(),
            )
        });

        assert!(report.p00_non_spill_exact_once_holds());
        assert_eq!(report.spill_facts_published, 0);
        assert_eq!(report.spill_facts_blocked, 1);
        assert_eq!(
            sheet.read_cell(&address(1, 1)),
            CalcValue::error(WorksheetErrorCode::Spill)
        );
        assert_eq!(sheet.read_cell(&address(2, 1)), CalcValue::empty());
        let fact = sheet.spill_facts().get(&address(1, 1)).unwrap();
        assert!(fact.blocked);
        assert_eq!(
            fact.extent,
            GridRect::new("book:default", "sheet:default", 1, 1, 2, 1, bounds()).unwrap()
        );
    }

    #[test]
    fn grid_calc_ref_evaluates_a1_range_formula_through_oxfml_provider() {
        let mut sheet = sheet();
        sheet
            .set_literal(address(1, 1), CalcValue::number(2.0))
            .unwrap();
        sheet
            .set_literal(address(1, 2), CalcValue::number(3.0))
            .unwrap();
        sheet
            .set_formula(
                address(1, 3),
                GridFormulaCell::new("=SUM(A1:B1)", "excel.grid.v1:sum:R[0]C[-2]:R[0]C[-1]"),
            )
            .unwrap();

        let report = sheet
            .recalculate_mark_all_dirty_with_oxfml()
            .expect("A1 range formula should evaluate through OxFml/OxFunc with the grid provider");

        assert!(report.p00_non_spill_exact_once_holds());
        assert_eq!(report.literal_cells, 2);
        assert_eq!(report.formula_cells, 1);
        assert_eq!(report.formula_evaluations, 1);
        assert_eq!(sheet.read_cell(&address(1, 3)), CalcValue::number(5.0));
    }

    #[test]
    fn grid_calc_ref_evaluates_repeated_r1c1_formula_region_through_oxfml_provider() {
        let mut sheet = sheet();
        sheet
            .set_literal(address(1, 1), CalcValue::number(10.0))
            .unwrap();
        sheet
            .set_literal(address(2, 1), CalcValue::number(11.0))
            .unwrap();
        let rect = GridRect::new("book:default", "sheet:default", 1, 2, 2, 2, bounds()).unwrap();
        sheet
            .materialize_formula_region(
                rect,
                GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let report = sheet
            .recalculate_mark_all_dirty_with_oxfml()
            .expect("R1C1 formulas should evaluate per caller through the grid provider");

        assert!(report.p00_non_spill_exact_once_holds());
        assert_eq!(report.literal_cells, 2);
        assert_eq!(report.formula_cells, 2);
        assert_eq!(report.formula_evaluations, 2);
        assert_eq!(sheet.read_cell(&address(1, 2)), CalcValue::number(20.0));
        assert_eq!(sheet.read_cell(&address(2, 2)), CalcValue::number(22.0));
    }

    #[test]
    fn grid_host_info_provider_reports_row_hidden_context_for_area() {
        let mut sheet = sheet();
        sheet.axis_state_mut().set_row(
            2,
            GridAxisProps {
                hidden_manual: true,
                ..GridAxisProps::visible()
            },
        );
        sheet.axis_state_mut().set_row(
            3,
            GridAxisProps {
                hidden_filter: true,
                ..GridAxisProps::visible()
            },
        );
        let provider = sheet.host_info_provider(1, 4);
        let reference = oxfunc_core::value::ReferenceLike::new(
            oxfunc_core::value::ReferenceKind::Area,
            "A1:A3",
        );

        let context = oxfunc_core::host_info::HostInfoProvider::query_aggregate_reference_context(
            &provider, &reference,
        )
        .expect("grid host info provider should resolve same-sheet A1 area context");

        assert_eq!(context.shape, ArrayShape { rows: 3, cols: 1 });
        assert_eq!(context.cells.len(), 3);
        assert!(!context.cells[0].row_hidden_manual);
        assert!(!context.cells[0].row_filtered_out);
        assert!(context.cells[1].row_hidden_manual);
        assert!(!context.cells[1].row_filtered_out);
        assert!(!context.cells[2].row_hidden_manual);
        assert!(context.cells[2].row_filtered_out);
    }

    #[test]
    fn grid_host_info_provider_reports_aggregate_context_row_run_counters() {
        let mut sheet = sheet();
        sheet.axis_state_mut().set_row(
            2,
            GridAxisProps {
                hidden_manual: true,
                ..GridAxisProps::visible()
            },
        );
        sheet.axis_state_mut().set_row(
            4,
            GridAxisProps {
                hidden_filter: true,
                ..GridAxisProps::visible()
            },
        );
        let provider = sheet.host_info_provider(1, 4);
        let reference = oxfunc_core::value::ReferenceLike::new(
            oxfunc_core::value::ReferenceKind::Area,
            "A1:A5",
        );

        let report = provider
            .aggregate_context_query_report(&reference)
            .expect("grid host info provider should report aggregate row-run counters");

        assert_eq!(report.rows, 5);
        assert_eq!(report.cols, 1);
        assert_eq!(report.declared_cell_count, 5);
        assert_eq!(report.explicit_axis_row_entries_visited, 2);
        assert_eq!(report.default_row_runs, 3);
        assert_eq!(report.row_context_runs, 5);
        assert_eq!(report.axis_run_probe_count, 5);
        assert_eq!(report.per_cell_context_expansion_count, 5);
        assert_eq!(report.manually_hidden_rows, 1);
        assert_eq!(report.filtered_hidden_rows, 1);
    }

    #[test]
    fn grid_calc_ref_evaluates_subtotal_with_axis_visibility_context() {
        let mut sheet = sheet();
        sheet
            .set_literal(address(1, 1), CalcValue::number(10.0))
            .unwrap();
        sheet
            .set_literal(address(2, 1), CalcValue::number(20.0))
            .unwrap();
        sheet
            .set_literal(address(3, 1), CalcValue::number(30.0))
            .unwrap();
        sheet.axis_state_mut().set_row(
            2,
            GridAxisProps {
                hidden_manual: true,
                ..GridAxisProps::visible()
            },
        );
        sheet.axis_state_mut().set_row(
            3,
            GridAxisProps {
                hidden_filter: true,
                ..GridAxisProps::visible()
            },
        );
        sheet
            .set_formula(
                address(1, 2),
                GridFormulaCell::new(
                    "=SUBTOTAL(9,A1:A3)",
                    "excel.grid.v1:subtotal9:R[0]C[-1]:R[2]C[-1]",
                ),
            )
            .unwrap();
        sheet
            .set_formula(
                address(1, 3),
                GridFormulaCell::new(
                    "=SUBTOTAL(109,A1:A3)",
                    "excel.grid.v1:subtotal109:R[0]C[-2]:R[2]C[-2]",
                ),
            )
            .unwrap();

        let report = sheet
            .recalculate_mark_all_dirty_with_oxfml()
            .expect("SUBTOTAL should evaluate with grid host visibility context");

        assert!(report.p00_non_spill_exact_once_holds());
        assert_eq!(report.literal_cells, 3);
        assert_eq!(report.formula_cells, 2);
        assert_eq!(sheet.read_cell(&address(1, 2)), CalcValue::number(30.0));
        assert_eq!(sheet.read_cell(&address(1, 3)), CalcValue::number(10.0));
    }

    #[test]
    fn grid_calc_ref_evaluates_spill_anchor_formula_through_provider_ledger() {
        let mut sheet = sheet();
        sheet
            .set_literal(address(1, 1), CalcValue::number(10.0))
            .unwrap();
        sheet
            .set_literal(address(2, 1), CalcValue::number(20.0))
            .unwrap();
        sheet
            .set_spill_fact(GridSpillFact {
                anchor: address(1, 1),
                extent: GridRect::new("book:default", "sheet:default", 1, 1, 2, 1, bounds())
                    .unwrap(),
                blocked: false,
            })
            .unwrap();
        sheet
            .set_formula(
                address(1, 2),
                GridFormulaCell::new("=SUM(A1#)", "excel.grid.v1:sum-spill:R[0]C[-1]#"),
            )
            .unwrap();

        let report = sheet
            .recalculate_mark_all_dirty_with_oxfml()
            .expect("A1# should evaluate through the grid spill ledger");

        assert!(report.p00_non_spill_exact_once_holds());
        assert_eq!(report.literal_cells, 2);
        assert_eq!(report.formula_cells, 1);
        assert_eq!(sheet.read_cell(&address(1, 2)), CalcValue::number(30.0));
    }

    #[test]
    fn grid_host_info_provider_reports_spill_anchor_visibility_context() {
        let mut sheet = sheet();
        sheet
            .set_spill_fact(GridSpillFact {
                anchor: address(1, 1),
                extent: GridRect::new("book:default", "sheet:default", 1, 1, 2, 1, bounds())
                    .unwrap(),
                blocked: false,
            })
            .unwrap();
        sheet.axis_state_mut().set_row(
            2,
            GridAxisProps {
                hidden_manual: true,
                ..GridAxisProps::visible()
            },
        );
        let provider = sheet.host_info_provider(1, 4);
        let reference = oxfunc_core::value::ReferenceLike::new(
            oxfunc_core::value::ReferenceKind::SpillAnchor,
            "A1#",
        );

        let context = oxfunc_core::host_info::HostInfoProvider::query_aggregate_reference_context(
            &provider, &reference,
        )
        .expect("grid host info provider should resolve spill anchor context");

        assert_eq!(context.shape, ArrayShape { rows: 2, cols: 1 });
        assert!(!context.cells[0].row_hidden_manual);
        assert!(context.cells[1].row_hidden_manual);
    }

    #[test]
    fn grid_structural_edit_insert_rows_shifts_storage_and_expands_spill_extent() {
        let mut sheet = sheet();
        sheet
            .set_literal(address(1, 1), CalcValue::number(1.0))
            .unwrap();
        sheet
            .set_literal(address(3, 1), CalcValue::number(3.0))
            .unwrap();
        sheet
            .set_literal(address(10, 1), CalcValue::number(10.0))
            .unwrap();
        sheet.axis_state_mut().set_row(
            3,
            GridAxisProps {
                hidden_filter: true,
                ..GridAxisProps::visible()
            },
        );
        sheet
            .set_spill_fact(GridSpillFact {
                anchor: address(1, 1),
                extent: GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds())
                    .unwrap(),
                blocked: false,
            })
            .unwrap();
        sheet
            .set_defined_name(
                "InputRange",
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap(),
            )
            .unwrap();
        sheet.add_merged_region(
            GridRect::new("book:default", "sheet:default", 3, 1, 4, 1, bounds()).unwrap(),
        );
        sheet.add_feature_rendered_region(
            GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap(),
            "table-overlay",
            false,
        );
        sheet.recalculate_mark_all_dirty(|_| CalcValue::empty());

        let report = sheet
            .apply_axis_edit(GridAxisEdit::insert_rows(2, 1))
            .expect("row insertion should transform bounded grid state");

        assert_eq!(report.authored_cells_kept, 2);
        assert_eq!(report.authored_cells_dropped, 1);
        assert_eq!(report.computed_cells_kept, 2);
        assert_eq!(report.computed_cells_dropped, 1);
        assert_eq!(report.spill_facts_kept, 1);
        assert_eq!(report.merged_regions_kept, 1);
        assert_eq!(report.feature_regions_kept, 1);
        assert_eq!(report.feature_regions_marked_needs_refresh, 0);
        assert_eq!(report.axis_entries_kept, 1);
        assert_eq!(
            sheet.read_cell(&address(4, 1)),
            CalcValue::number(3.0),
            "row 3 literal should shift to row 4"
        );
        assert_eq!(sheet.read_cell(&address(10, 1)), CalcValue::empty());
        assert!(sheet.axis_state().row(4).hidden_filter);
        assert!(!sheet.axis_state().row(3).hidden_filter);
        assert_eq!(
            sheet.spill_facts().get(&address(1, 1)).unwrap().extent,
            GridRect::new("book:default", "sheet:default", 1, 1, 4, 1, bounds()).unwrap()
        );
        assert_eq!(
            sheet.defined_names().get("INPUTRANGE").unwrap(),
            &GridRect::new("book:default", "sheet:default", 1, 1, 4, 1, bounds()).unwrap()
        );
        assert_eq!(
            sheet.merged_regions()[0].rect,
            GridRect::new("book:default", "sheet:default", 4, 1, 5, 1, bounds()).unwrap()
        );
        assert_eq!(
            sheet.feature_rendered_regions()[0].rect,
            GridRect::new("book:default", "sheet:default", 1, 1, 4, 1, bounds()).unwrap()
        );
    }

    #[test]
    fn grid_structural_edit_delete_rows_drops_inside_and_shifts_after() {
        let mut sheet = sheet();
        sheet
            .set_literal(address(1, 1), CalcValue::number(1.0))
            .unwrap();
        sheet
            .set_literal(address(2, 1), CalcValue::number(2.0))
            .unwrap();
        sheet
            .set_literal(address(4, 1), CalcValue::number(4.0))
            .unwrap();
        sheet.axis_state_mut().set_row(
            4,
            GridAxisProps {
                hidden_manual: true,
                ..GridAxisProps::visible()
            },
        );
        sheet
            .set_spill_fact(GridSpillFact {
                anchor: address(1, 1),
                extent: GridRect::new("book:default", "sheet:default", 1, 1, 4, 1, bounds())
                    .unwrap(),
                blocked: false,
            })
            .unwrap();
        sheet.add_feature_rendered_region(
            GridRect::new("book:default", "sheet:default", 2, 1, 2, 1, bounds()).unwrap(),
            "deleted-overlay",
            true,
        );
        sheet.recalculate_mark_all_dirty(|_| CalcValue::empty());

        let report = sheet
            .apply_axis_edit(GridAxisEdit::delete_rows(2, 1))
            .expect("row deletion should transform bounded grid state");

        assert_eq!(report.authored_cells_kept, 2);
        assert_eq!(report.authored_cells_dropped, 1);
        assert_eq!(report.computed_cells_kept, 2);
        assert_eq!(report.computed_cells_dropped, 1);
        assert_eq!(report.spill_facts_kept, 1);
        assert_eq!(report.feature_regions_kept, 0);
        assert_eq!(report.feature_regions_dropped, 1);
        assert_eq!(report.feature_regions_marked_needs_refresh, 0);
        assert_eq!(sheet.read_cell(&address(2, 1)), CalcValue::empty());
        assert_eq!(sheet.read_cell(&address(3, 1)), CalcValue::number(4.0));
        assert!(sheet.axis_state().row(3).hidden_manual);
        assert!(!sheet.axis_state().row(4).hidden_manual);
        assert_eq!(
            sheet.spill_facts().get(&address(1, 1)).unwrap().extent,
            GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap()
        );
    }

    #[test]
    fn grid_structural_edit_insert_before_pivot_feature_marks_refresh() {
        let mut sheet = sheet();
        sheet.add_feature_rendered_region(
            GridRect::new("book:default", "sheet:default", 2, 1, 4, 2, bounds()).unwrap(),
            "pivot-report",
            false,
        );

        let report = sheet
            .apply_axis_edit(GridAxisEdit::insert_rows(2, 1))
            .expect("pivot feature region should shift when edit is before it");

        assert_eq!(report.feature_regions_kept, 1);
        assert_eq!(report.feature_regions_dropped, 0);
        assert_eq!(report.feature_regions_marked_needs_refresh, 1);
        assert_eq!(
            sheet.feature_rendered_regions()[0],
            FeatureRenderedRegion {
                rect: GridRect::new("book:default", "sheet:default", 3, 1, 5, 2, bounds()).unwrap(),
                feature_kind: "pivot-report".to_string(),
                needs_refresh: true,
            }
        );
    }

    #[test]
    fn grid_structural_edit_insert_inside_pivot_feature_is_refused_without_mutation() {
        let mut sheet = sheet();
        sheet
            .set_literal(address(5, 1), CalcValue::number(55.0))
            .unwrap();
        sheet.add_feature_rendered_region(
            GridRect::new("book:default", "sheet:default", 2, 1, 4, 2, bounds()).unwrap(),
            "pivot-report",
            false,
        );

        let error = sheet
            .apply_axis_edit(GridAxisEdit::insert_rows(3, 1))
            .expect_err("pivot feature region should refuse an inside row insertion");

        assert!(matches!(
            error,
            GridRefError::FeatureRenderedRegionEditRefused { feature_kind, .. }
                if feature_kind == "pivot-report"
        ));
        assert_eq!(
            sheet.authored().get(&address(5, 1)),
            Some(&GridAuthoredCell::Literal(CalcValue::number(55.0)))
        );
        assert_eq!(
            sheet.feature_rendered_regions()[0],
            FeatureRenderedRegion {
                rect: GridRect::new("book:default", "sheet:default", 2, 1, 4, 2, bounds()).unwrap(),
                feature_kind: "pivot-report".to_string(),
                needs_refresh: false,
            }
        );
    }

    #[test]
    fn grid_structural_edit_delete_rows_drops_spill_when_anchor_is_deleted() {
        let mut sheet = sheet();
        sheet
            .set_spill_fact(GridSpillFact {
                anchor: address(2, 1),
                extent: GridRect::new("book:default", "sheet:default", 2, 1, 3, 1, bounds())
                    .unwrap(),
                blocked: false,
            })
            .unwrap();

        let report = sheet
            .apply_axis_edit(GridAxisEdit::delete_rows(2, 1))
            .expect("row deletion should drop spill facts whose anchor is deleted");

        assert_eq!(report.spill_facts_kept, 0);
        assert_eq!(report.spill_facts_dropped, 1);
        assert!(sheet.spill_facts().is_empty());
    }

    #[test]
    fn grid_structural_edit_delete_columns_shifts_cells_and_axis_state() {
        let mut sheet = sheet();
        sheet
            .set_literal(address(1, 1), CalcValue::number(1.0))
            .unwrap();
        sheet
            .set_literal(address(1, 2), CalcValue::number(2.0))
            .unwrap();
        sheet
            .set_literal(address(1, 4), CalcValue::number(4.0))
            .unwrap();
        sheet.axis_state_mut().set_col(
            4,
            GridAxisProps {
                hidden_manual: true,
                ..GridAxisProps::visible()
            },
        );
        sheet.add_merged_region(
            GridRect::new("book:default", "sheet:default", 1, 2, 1, 4, bounds()).unwrap(),
        );
        sheet.recalculate_mark_all_dirty(|_| CalcValue::empty());

        let report = sheet
            .apply_axis_edit(GridAxisEdit::delete_columns(2, 1))
            .expect("column deletion should transform bounded grid state");

        assert_eq!(report.authored_cells_kept, 2);
        assert_eq!(report.authored_cells_dropped, 1);
        assert_eq!(report.axis_entries_kept, 1);
        assert_eq!(sheet.read_cell(&address(1, 2)), CalcValue::empty());
        assert_eq!(sheet.read_cell(&address(1, 3)), CalcValue::number(4.0));
        assert!(sheet.axis_state().col(3).hidden_manual);
        assert!(!sheet.axis_state().col(4).hidden_manual);
        assert_eq!(
            sheet.merged_regions()[0].rect,
            GridRect::new("book:default", "sheet:default", 1, 2, 1, 3, bounds()).unwrap()
        );
    }

    #[test]
    fn grid_structural_edit_rewrites_formula_point_reference() {
        let mut sheet = sheet();
        sheet
            .set_literal(address(1, 1), CalcValue::number(2.0))
            .unwrap();
        sheet
            .set_formula(
                address(1, 2),
                GridFormulaCell::new("=A1+1", "stale-before-transform"),
            )
            .unwrap();

        let report = sheet
            .apply_axis_edit(GridAxisEdit::insert_columns(1, 1))
            .expect("column insertion should rewrite grid formula references");

        assert_eq!(report.formula_cells_transformed, 1);
        assert_eq!(report.formula_reference_transforms, 1);
        let formula = match sheet.authored().get(&address(1, 3)).unwrap() {
            GridAuthoredCell::Formula(formula) => formula,
            other => panic!("expected transformed formula at C1, got {other:?}"),
        };
        assert_eq!(formula.source_text, "=B1+1");

        sheet
            .recalculate_mark_all_dirty_with_oxfml()
            .expect("transformed formula should evaluate through grid provider");
        assert_eq!(sheet.read_cell(&address(1, 3)), CalcValue::number(3.0));
    }

    #[test]
    fn grid_structural_edit_rewrites_deleted_formula_reference_to_ref_error() {
        let mut sheet = sheet();
        sheet
            .set_literal(address(1, 1), CalcValue::number(2.0))
            .unwrap();
        sheet
            .set_formula(
                address(1, 2),
                GridFormulaCell::new("=A1+1", "stale-before-transform"),
            )
            .unwrap();

        let report = sheet
            .apply_axis_edit(GridAxisEdit::delete_columns(1, 1))
            .expect("column deletion should preserve formula with #REF! reference error");

        assert_eq!(report.authored_cells_kept, 1);
        assert_eq!(report.authored_cells_dropped, 1);
        assert_eq!(report.formula_cells_transformed, 1);
        assert_eq!(report.formula_reference_transforms, 1);
        let formula = match sheet.authored().get(&address(1, 1)).unwrap() {
            GridAuthoredCell::Formula(formula) => formula,
            other => panic!("expected transformed formula at A1, got {other:?}"),
        };
        assert_eq!(formula.source_text, "=#REF!+1");
    }

    #[test]
    fn grid_structural_edit_preserves_r1c1_formula_text_when_shape_survives() {
        let mut sheet = sheet();
        sheet
            .set_literal(address(2, 1), CalcValue::number(4.0))
            .unwrap();
        sheet
            .set_formula(
                address(2, 2),
                GridFormulaCell::new("=RC[-1]*2", "stale-before-transform")
                    .with_source_channel(FormulaChannelKind::WorksheetR1C1),
            )
            .unwrap();

        let report = sheet
            .apply_axis_edit(GridAxisEdit::insert_columns(1, 1))
            .expect("column insertion should keep relative R1C1 shape");

        assert_eq!(report.formula_cells_transformed, 1);
        assert_eq!(report.formula_reference_transforms, 1);
        let formula = match sheet.authored().get(&address(2, 3)).unwrap() {
            GridAuthoredCell::Formula(formula) => formula,
            other => panic!("expected transformed formula at C2, got {other:?}"),
        };
        assert_eq!(formula.source_text, "=RC[-1]*2");
        assert_eq!(formula.source_channel, FormulaChannelKind::WorksheetR1C1);

        sheet
            .recalculate_mark_all_dirty_with_oxfml()
            .expect("R1C1 transformed formula should evaluate through grid provider");
        assert_eq!(sheet.read_cell(&address(2, 3)), CalcValue::number(8.0));
    }

    #[test]
    fn grid_invalidation_ref_scalarizes_ranges_and_closes_dependents() {
        let mut invalidation = GridInvalidationRef::new(bounds());
        let a1 = address(1, 1);
        let b1 = address(1, 2);
        let c1 = address(1, 3);
        let a2 = address(2, 1);
        let a3 = address(3, 1);
        let a2_a3 = GridRect::new("book:default", "sheet:default", 2, 1, 3, 1, bounds()).unwrap();

        invalidation
            .set_cell_dependencies(b1.clone(), [GridDependency::Cell(a1.clone())])
            .expect("cell dependency should install");
        let edge_count = invalidation
            .set_cell_dependencies(
                c1.clone(),
                [
                    GridDependency::Cell(b1.clone()),
                    GridDependency::Range(a2_a3),
                ],
            )
            .expect("range dependency should scalarize");

        assert_eq!(edge_count, 3);
        assert_eq!(
            invalidation.dirty_closure([a1.clone()]),
            set([a1.clone(), b1.clone(), c1.clone()])
        );
        assert_eq!(
            invalidation.dirty_closure([a2.clone()]),
            set([a2.clone(), c1.clone()])
        );
        assert_eq!(
            invalidation.dirty_closure([a3.clone()]),
            set([a3.clone(), c1.clone()])
        );
    }

    #[test]
    fn grid_invalidation_ref_replaces_dependencies_without_stale_edges() {
        let mut invalidation = GridInvalidationRef::new(bounds());
        let a1 = address(1, 1);
        let a2 = address(2, 1);
        let b1 = address(1, 2);

        invalidation
            .set_cell_dependencies(b1.clone(), [GridDependency::Cell(a1.clone())])
            .unwrap();
        invalidation
            .set_cell_dependencies(b1.clone(), [GridDependency::Cell(a2.clone())])
            .unwrap();

        assert_eq!(invalidation.dirty_closure([a1.clone()]), set([a1]));
        assert_eq!(
            invalidation.dirty_closure([a2.clone()]),
            set([a2, b1.clone()])
        );
        assert_eq!(invalidation.dependencies_for(&b1), set([address(2, 1)]));
    }

    #[test]
    fn grid_invalidation_ref_classifies_dynamic_dependencies_separately() {
        let mut invalidation = GridInvalidationRef::new(bounds());
        let b1 = address(1, 2);
        let c1 = address(1, 3);

        invalidation
            .set_cell_dependencies(
                b1.clone(),
                [GridDependency::DynamicRequest(
                    "indirect:Sheet1!A1".to_string(),
                )],
            )
            .unwrap();
        invalidation
            .set_cell_dependencies(c1.clone(), [GridDependency::Cell(b1.clone())])
            .unwrap();

        assert_eq!(
            invalidation.dynamic_dependencies_for(&b1),
            ["indirect:Sheet1!A1".to_string()].into_iter().collect()
        );
        assert_eq!(
            invalidation.dirty_closure_for_dynamic_request("indirect:Sheet1!A1"),
            set([b1, c1])
        );
    }

    #[test]
    fn grid_invalidation_ref_closes_over_name_dependencies() {
        let mut invalidation = GridInvalidationRef::new(bounds());
        let a1 = address(1, 1);
        let a2 = address(2, 1);
        let b1 = address(1, 2);
        let c1 = address(1, 3);
        let input_range =
            GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap();

        invalidation
            .set_cell_dependencies(
                b1.clone(),
                [GridDependency::Name(
                    GridNameDependency::new("InputRange", input_range.clone(), bounds()).unwrap(),
                )],
            )
            .expect("name dependency should install");
        invalidation
            .set_cell_dependencies(c1.clone(), [GridDependency::Cell(b1.clone())])
            .expect("downstream consumer should install");

        assert_eq!(invalidation.name_edge_count(), 1);
        assert_eq!(
            invalidation.name_dependencies_for(&b1),
            [GridNameDependency::new("InputRange", input_range, bounds()).unwrap()]
                .into_iter()
                .collect()
        );
        assert_eq!(
            invalidation.dirty_closure([a2.clone()]),
            set([a2, b1.clone(), c1.clone()])
        );
        assert_eq!(
            invalidation
                .dirty_closure_for_name("inputrange")
                .expect("name namespace closure should be case-insensitive"),
            set([b1.clone(), c1.clone()])
        );
        assert_eq!(invalidation.dirty_closure([a1.clone()]), set([a1, b1, c1]));
    }

    #[test]
    fn grid_invalidation_ref_transforms_name_dependencies_under_axis_edit() {
        let mut invalidation = GridInvalidationRef::new(bounds());
        let b1 = address(1, 2);
        let input_range =
            GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap();

        invalidation
            .set_cell_dependencies(
                b1.clone(),
                [GridDependency::Name(
                    GridNameDependency::new("InputRange", input_range, bounds()).unwrap(),
                )],
            )
            .expect("name dependency should install");

        let report = invalidation
            .apply_axis_edit(GridAxisEdit::insert_rows(2, 1))
            .expect("name dependencies should transform with row insertion");

        assert_eq!(report.name_edges_before, 1);
        assert_eq!(report.name_edges_after, 1);
        assert_eq!(
            invalidation.semantic_dependencies_for(&b1),
            vec![GridDependency::Name(
                GridNameDependency::new(
                    "InputRange",
                    GridRect::new("book:default", "sheet:default", 1, 1, 4, 1, bounds()).unwrap(),
                    bounds()
                )
                .unwrap()
            )]
        );
        assert_eq!(
            invalidation.dirty_closure([address(4, 1)]),
            set([address(4, 1), b1])
        );
    }

    #[test]
    fn grid_invalidation_ref_renames_and_deletes_name_namespace_dependencies() {
        let mut invalidation = GridInvalidationRef::new(bounds());
        let b1 = address(1, 2);
        let c1 = address(1, 3);
        let input_range =
            GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap();

        invalidation
            .set_cell_dependencies(
                b1.clone(),
                [GridDependency::Name(
                    GridNameDependency::new("InputRange", input_range.clone(), bounds()).unwrap(),
                )],
            )
            .expect("name dependency should install");
        invalidation
            .set_cell_dependencies(c1.clone(), [GridDependency::Cell(b1.clone())])
            .expect("downstream consumer should install");

        let rename = invalidation
            .rename_defined_name("InputRange", "DataRange")
            .expect("name namespace rename should retarget semantic dependencies");
        assert_eq!(
            rename.operation,
            GridInvalidationNamespaceLifecycleOperation::RenameName {
                old_name_key: "INPUTRANGE".to_string(),
                new_name_key: "DATARANGE".to_string()
            }
        );
        assert_eq!(rename.dirty_closure, set([b1.clone(), c1.clone()]));
        assert_eq!(rename.name_edges_before, 1);
        assert_eq!(rename.name_edges_after, 1);
        assert_eq!(
            invalidation.name_dependencies_for(&b1),
            [GridNameDependency::new("DataRange", input_range.clone(), bounds()).unwrap()]
                .into_iter()
                .collect()
        );
        assert!(
            invalidation
                .dirty_closure_for_name("InputRange")
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            invalidation.dirty_closure_for_name("DataRange").unwrap(),
            set([b1.clone(), c1.clone()])
        );
        assert_eq!(
            invalidation.dirty_closure([address(2, 1)]),
            set([address(2, 1), b1.clone(), c1.clone()])
        );

        let delete = invalidation
            .delete_defined_name("DataRange")
            .expect("name namespace delete should remove semantic dependencies");
        assert_eq!(
            delete.operation,
            GridInvalidationNamespaceLifecycleOperation::DeleteName {
                name_key: "DATARANGE".to_string()
            }
        );
        assert_eq!(delete.dirty_closure, set([b1.clone(), c1]));
        assert_eq!(delete.name_edges_before, 1);
        assert_eq!(delete.name_edges_after, 0);
        assert_eq!(delete.semantic_dependencies_dropped, 1);
        assert!(invalidation.name_dependencies_for(&b1).is_empty());
        assert!(invalidation.semantic_dependencies_for(&b1).is_empty());
        assert_eq!(
            invalidation.dirty_closure([address(2, 1)]),
            set([address(2, 1)])
        );
    }

    #[test]
    fn grid_invalidation_ref_closes_over_table_dependencies() {
        let mut invalidation = GridInvalidationRef::new(bounds());
        let b3 = address(3, 2);
        let c1 = address(1, 3);
        let d1 = address(1, 4);
        let amount_data =
            GridRect::new("book:default", "sheet:default", 2, 2, 4, 2, bounds()).unwrap();

        invalidation
            .set_cell_dependencies(
                c1.clone(),
                [GridDependency::Table(
                    GridTableDependency::new("Table1", amount_data.clone(), bounds()).unwrap(),
                )],
            )
            .expect("table dependency should install");
        invalidation
            .set_cell_dependencies(d1.clone(), [GridDependency::Cell(c1.clone())])
            .expect("downstream consumer should install");

        assert_eq!(invalidation.table_edge_count(), 1);
        assert_eq!(
            invalidation.table_dependencies_for(&c1),
            [GridTableDependency::new("Table1", amount_data, bounds()).unwrap()]
                .into_iter()
                .collect()
        );
        assert_eq!(
            invalidation.dirty_closure([b3.clone()]),
            set([b3, c1.clone(), d1.clone()])
        );
        assert_eq!(
            invalidation
                .dirty_closure_for_table("table1")
                .expect("table namespace closure should be case-insensitive"),
            set([c1, d1])
        );
    }

    #[test]
    fn grid_invalidation_ref_transforms_table_dependencies_under_axis_edit() {
        let mut invalidation = GridInvalidationRef::new(bounds());
        let c1 = address(1, 3);
        let amount_data =
            GridRect::new("book:default", "sheet:default", 2, 2, 4, 2, bounds()).unwrap();

        invalidation
            .set_cell_dependencies(
                c1.clone(),
                [GridDependency::Table(
                    GridTableDependency::new("Table1", amount_data, bounds()).unwrap(),
                )],
            )
            .expect("table dependency should install");

        let report = invalidation
            .apply_axis_edit(GridAxisEdit::insert_rows(3, 1))
            .expect("table dependencies should transform with row insertion");

        assert_eq!(report.table_edges_before, 1);
        assert_eq!(report.table_edges_after, 1);
        assert_eq!(
            invalidation.semantic_dependencies_for(&c1),
            vec![GridDependency::Table(
                GridTableDependency::new(
                    "Table1",
                    GridRect::new("book:default", "sheet:default", 2, 2, 5, 2, bounds()).unwrap(),
                    bounds()
                )
                .unwrap()
            )]
        );
        assert_eq!(
            invalidation.dirty_closure([address(5, 2)]),
            set([address(5, 2), c1])
        );
    }

    #[test]
    fn grid_invalidation_ref_table_namespace_lifecycle_retains_and_rebuilds_edges() {
        let mut invalidation = GridInvalidationRef::new(bounds());
        let c1 = address(1, 3);
        let d1 = address(1, 4);
        let amount_data =
            GridRect::new("book:default", "sheet:default", 2, 2, 4, 2, bounds()).unwrap();
        let resized_amount_data =
            GridRect::new("book:default", "sheet:default", 2, 2, 5, 2, bounds()).unwrap();

        invalidation
            .set_cell_dependencies(
                c1.clone(),
                [GridDependency::Table(
                    GridTableDependency::new("Table1", amount_data, bounds()).unwrap(),
                )],
            )
            .expect("table dependency should install");
        invalidation
            .set_cell_dependencies(d1.clone(), [GridDependency::Cell(c1.clone())])
            .expect("downstream consumer should install");

        let rename = invalidation
            .rename_table("Table1", "Sales")
            .expect("table namespace rename should retarget semantic dependencies");
        assert_eq!(
            rename.operation,
            GridInvalidationNamespaceLifecycleOperation::RenameTable {
                old_table_key: "TABLE1".to_string(),
                new_table_key: "SALES".to_string()
            }
        );
        assert_eq!(rename.dirty_closure, set([c1.clone(), d1.clone()]));
        assert_eq!(rename.table_edges_before, 1);
        assert_eq!(rename.table_edges_after, 1);
        assert!(
            invalidation
                .dirty_closure_for_table("Table1")
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            invalidation.dirty_closure_for_table("Sales").unwrap(),
            set([c1.clone(), d1.clone()])
        );

        let resize = invalidation
            .resize_table("Sales", resized_amount_data.clone())
            .expect("table namespace resize should rebuild scalar edges for new extent");
        assert_eq!(
            resize.operation,
            GridInvalidationNamespaceLifecycleOperation::ResizeTable {
                table_key: "SALES".to_string()
            }
        );
        assert_eq!(resize.dirty_closure, set([c1.clone(), d1.clone()]));
        assert_eq!(resize.table_edges_before, 1);
        assert_eq!(resize.table_edges_after, 1);
        assert_eq!(
            invalidation.table_dependencies_for(&c1),
            [GridTableDependency::new("Sales", resized_amount_data, bounds()).unwrap()]
                .into_iter()
                .collect()
        );
        assert_eq!(
            invalidation.dirty_closure([address(5, 2)]),
            set([address(5, 2), c1.clone(), d1.clone()])
        );

        let delete = invalidation
            .delete_table("Sales")
            .expect("table namespace delete should remove semantic dependencies");
        assert_eq!(
            delete.operation,
            GridInvalidationNamespaceLifecycleOperation::DeleteTable {
                table_key: "SALES".to_string()
            }
        );
        assert_eq!(delete.dirty_closure, set([c1.clone(), d1.clone()]));
        assert_eq!(delete.table_edges_before, 1);
        assert_eq!(delete.table_edges_after, 0);
        assert_eq!(delete.semantic_dependencies_dropped, 1);
        assert!(invalidation.table_dependencies_for(&c1).is_empty());
        assert!(invalidation.semantic_dependencies_for(&c1).is_empty());
        assert_eq!(
            invalidation.dirty_closure([address(5, 2)]),
            set([address(5, 2)])
        );
    }

    #[test]
    fn grid_invalidation_ref_closes_over_spill_fact_dependencies() {
        let mut invalidation = GridInvalidationRef::new(bounds());
        let a1 = address(1, 1);
        let a2 = address(2, 1);
        let b1 = address(1, 2);
        let c1 = address(1, 3);
        let a1_a3 = GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap();

        invalidation
            .set_cell_dependencies(
                b1.clone(),
                [
                    GridDependency::Range(a1_a3),
                    GridDependency::SpillFact(GridSpillDependency::anchor(a1.clone())),
                ],
            )
            .expect("A1# consumer should install both value and spill-shape dependencies");
        invalidation
            .set_cell_dependencies(c1.clone(), [GridDependency::Cell(b1.clone())])
            .expect("downstream consumer should install");

        assert_eq!(
            invalidation.spill_dependencies_for(&b1),
            [GridSpillDependency::anchor(a1.clone())]
                .into_iter()
                .collect()
        );
        assert_eq!(invalidation.spill_edge_count(), 1);
        assert_eq!(
            invalidation.dirty_closure([a2.clone()]),
            set([a2.clone(), b1.clone(), c1.clone()])
        );
        assert_eq!(
            invalidation
                .dirty_closure_for_spill_fact(GridSpillDependency::anchor(a1.clone()))
                .unwrap(),
            set([b1.clone(), c1.clone()])
        );

        let report = invalidation
            .apply_axis_edit(GridAxisEdit::insert_rows(2, 1))
            .expect("spill dependencies should transform with row insertion");
        assert_eq!(report.semantic_dependencies_kept, 3);
        assert_eq!(report.spill_edges_before, 1);
        assert_eq!(report.spill_edges_after, 1);
        assert_eq!(
            invalidation.semantic_dependencies_for(&b1),
            vec![
                GridDependency::Range(
                    GridRect::new("book:default", "sheet:default", 1, 1, 4, 1, bounds()).unwrap()
                ),
                GridDependency::SpillFact(GridSpillDependency::anchor(a1.clone())),
            ]
        );
        assert_eq!(
            invalidation
                .dirty_closure_for_spill_fact(GridSpillDependency::anchor(a1))
                .unwrap(),
            set([b1, c1])
        );
    }

    #[test]
    fn grid_invalidation_ref_reports_spill_epoch_precision_for_a1_hash_consumers() {
        let mut invalidation = GridInvalidationRef::new(bounds());
        let a1 = address(1, 1);
        let b2 = address(2, 2);
        let c1 = address(1, 3);
        let d1 = address(1, 4);
        invalidation
            .set_cell_dependencies(
                c1.clone(),
                [GridDependency::SpillFact(GridSpillDependency::anchor(
                    a1.clone(),
                ))],
            )
            .expect("A1# consumer should install a spill fact dependency");
        invalidation
            .set_cell_dependencies(d1.clone(), [GridDependency::Cell(c1.clone())])
            .expect("downstream A1# consumer should install a scalar dependency");
        let a1_full = GridSpillEpochSnapshot::new(
            GridSpillFact {
                anchor: a1.clone(),
                extent: GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds())
                    .unwrap(),
                blocked: false,
            },
            1,
        );
        let b2_full = GridSpillEpochSnapshot::new(
            GridSpillFact {
                anchor: b2.clone(),
                extent: GridRect::new("book:default", "sheet:default", 2, 2, 3, 2, bounds())
                    .unwrap(),
                blocked: false,
            },
            1,
        );

        let unchanged = invalidation
            .dirty_closure_for_spill_epoch_changes(
                [a1_full.clone(), b2_full.clone()],
                [a1_full.clone(), b2_full.clone()],
            )
            .expect("unchanged spill snapshots should be valid");
        assert_eq!(unchanged.anchors_compared, 2);
        assert_eq!(unchanged.unchanged_anchors, 2);
        assert!(unchanged.dirty_closure.is_empty());

        let unrelated = invalidation
            .dirty_closure_for_spill_epoch_changes(
                [a1_full.clone(), b2_full.clone()],
                [
                    a1_full.clone(),
                    GridSpillEpochSnapshot {
                        value_epoch: 2,
                        ..b2_full.clone()
                    },
                ],
            )
            .expect("unrelated spill value churn should be valid");
        assert_eq!(unrelated.changed_anchors.len(), 1);
        assert_eq!(unrelated.value_epoch_changed_anchors, 1);
        assert!(unrelated.dirty_closure.is_empty());

        let value_change = invalidation
            .dirty_closure_for_spill_epoch_changes(
                [a1_full.clone()],
                [GridSpillEpochSnapshot {
                    value_epoch: 2,
                    ..a1_full.clone()
                }],
            )
            .expect("A1 value epoch change should be valid");
        assert_eq!(value_change.value_epoch_changed_anchors, 1);
        assert_eq!(value_change.dirty_closure, set([c1.clone(), d1.clone()]));

        let extent_change = invalidation
            .dirty_closure_for_spill_epoch_changes(
                [a1_full.clone()],
                [GridSpillEpochSnapshot {
                    extent: GridRect::new("book:default", "sheet:default", 1, 1, 4, 1, bounds())
                        .unwrap(),
                    ..a1_full
                }],
            )
            .expect("A1 extent epoch change should be valid");
        assert_eq!(extent_change.extent_epoch_changed_anchors, 1);
        assert_eq!(extent_change.dirty_closure, set([c1, d1]));
    }

    #[test]
    fn grid_invalidation_ref_closes_over_spill_blocker_dependencies() {
        let mut invalidation = GridInvalidationRef::new(bounds());
        let a1 = address(1, 1);
        let a2 = address(2, 1);
        let b1 = address(1, 2);
        let a1_a3 = GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap();

        invalidation
            .set_cell_dependencies(
                a1.clone(),
                [GridDependency::SpillBlocker(
                    GridSpillBlockerDependency::extent(a1_a3),
                )],
            )
            .expect("blocked spill anchor should watch its potential extent");
        invalidation
            .set_cell_dependencies(b1.clone(), [GridDependency::Cell(a1.clone())])
            .expect("downstream cell dependency should install");

        assert_eq!(
            invalidation.spill_blocker_dependencies_for(&a1),
            [GridSpillBlockerDependency::extent(
                GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap()
            )]
            .into_iter()
            .collect()
        );
        assert_eq!(invalidation.spill_blocker_edge_count(), 1);
        assert_eq!(
            invalidation
                .dirty_closure_for_spill_blocker(GridSpillBlockerDependency::extent(
                    GridRect::new("book:default", "sheet:default", 2, 1, 2, 1, bounds()).unwrap()
                ))
                .unwrap(),
            set([a1.clone(), b1.clone()])
        );
        assert_eq!(
            invalidation
                .dirty_closure_for_spill_blocker(GridSpillBlockerDependency::extent(
                    GridRect::new("book:default", "sheet:default", 4, 1, 4, 1, bounds()).unwrap()
                ))
                .unwrap(),
            BTreeSet::new()
        );

        let report = invalidation
            .apply_axis_edit(GridAxisEdit::insert_rows(2, 1))
            .expect("spill blocker dependencies should transform with row insertion");
        assert_eq!(report.semantic_dependencies_kept, 2);
        assert_eq!(report.spill_blocker_edges_before, 1);
        assert_eq!(report.spill_blocker_edges_after, 1);
        assert_eq!(
            invalidation.semantic_dependencies_for(&a1),
            vec![GridDependency::SpillBlocker(
                GridSpillBlockerDependency::extent(
                    GridRect::new("book:default", "sheet:default", 1, 1, 4, 1, bounds()).unwrap()
                )
            )]
        );
        assert_eq!(
            invalidation
                .dirty_closure_for_spill_blocker(GridSpillBlockerDependency::extent(
                    GridRect::new("book:default", "sheet:default", 4, 1, 4, 1, bounds()).unwrap()
                ))
                .unwrap(),
            set([a1, b1])
        );
        assert_eq!(invalidation.dirty_closure([a2.clone()]), set([a2]));
    }

    #[test]
    fn grid_invalidation_ref_closes_over_axis_visibility_dependencies() {
        let mut invalidation = GridInvalidationRef::new(bounds());
        let b1 = address(1, 2);
        let c1 = address(1, 3);
        let a1_a3 = GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap();
        invalidation
            .set_cell_dependencies(
                b1.clone(),
                [
                    GridDependency::Range(a1_a3),
                    GridDependency::AxisVisibility(GridAxisVisibilityDependency::rows(1, 3)),
                ],
            )
            .expect("hidden-sensitive aggregate dependencies should install");
        invalidation
            .set_cell_dependencies(c1.clone(), [GridDependency::Cell(b1.clone())])
            .expect("downstream cell dependency should install");

        assert_eq!(
            invalidation.axis_visibility_dependencies_for(&b1),
            [GridAxisVisibilityDependency::rows(1, 3)]
                .into_iter()
                .collect()
        );
        assert_eq!(
            invalidation
                .dirty_closure_for_axis_visibility(GridAxisVisibilityDependency::rows(2, 2))
                .unwrap(),
            set([b1.clone(), c1.clone()])
        );
        assert_eq!(
            invalidation
                .dirty_closure_for_axis_visibility(GridAxisVisibilityDependency::rows(4, 4))
                .unwrap(),
            BTreeSet::new()
        );

        let report = invalidation
            .apply_axis_edit(GridAxisEdit::insert_rows(2, 1))
            .expect("visibility dependency should transform with row insertion");
        assert_eq!(report.semantic_dependencies_kept, 3);
        assert_eq!(
            invalidation.semantic_dependencies_for(&b1),
            vec![
                GridDependency::Range(
                    GridRect::new("book:default", "sheet:default", 1, 1, 4, 1, bounds()).unwrap()
                ),
                GridDependency::AxisVisibility(GridAxisVisibilityDependency::rows(1, 4)),
            ]
        );
        assert_eq!(
            invalidation
                .dirty_closure_for_axis_visibility(GridAxisVisibilityDependency::rows(2, 2))
                .unwrap(),
            set([b1, c1])
        );
    }

    #[test]
    fn grid_invalidation_ref_closes_over_whole_row_value_dependencies() {
        let mut invalidation = GridInvalidationRef::new(bounds());
        let b1 = address(1, 2);
        let c1 = address(1, 3);
        let a2 = address(2, 1);
        let e4 = address(4, 5);

        invalidation
            .set_cell_dependencies(
                b1.clone(),
                [GridDependency::AxisValue(GridAxisValueDependency::rows(
                    2, 3,
                ))],
            )
            .expect("whole-row dependency should install without scalarizing all columns");
        invalidation
            .set_cell_dependencies(c1.clone(), [GridDependency::Cell(b1.clone())])
            .expect("downstream cell dependency should install");

        assert_eq!(
            invalidation.axis_value_dependencies_for(&b1),
            [GridAxisValueDependency::rows(2, 3)].into_iter().collect()
        );
        assert_eq!(invalidation.axis_value_edge_count(), 1);
        assert_eq!(
            invalidation.dirty_closure([a2.clone()]),
            set([a2.clone(), b1.clone(), c1.clone()])
        );
        assert_eq!(
            invalidation
                .dirty_closure_for_axis_value(GridAxisValueDependency::rows(3, 3))
                .unwrap(),
            set([b1.clone(), c1.clone()])
        );

        let report = invalidation
            .apply_axis_edit(GridAxisEdit::insert_rows(3, 1))
            .expect("whole-row dependency should transform with row insertion");
        assert_eq!(report.axis_value_edges_before, 1);
        assert_eq!(report.axis_value_edges_after, 1);
        assert_eq!(
            invalidation.semantic_dependencies_for(&b1),
            vec![GridDependency::AxisValue(GridAxisValueDependency::rows(
                2, 4
            ))]
        );
        assert_eq!(invalidation.dirty_closure([e4.clone()]), set([e4, b1, c1]));
    }

    #[test]
    fn grid_invalidation_ref_closes_over_whole_column_value_dependencies() {
        let mut invalidation = GridInvalidationRef::new(bounds());
        let d1 = address(1, 4);
        let e1 = address(1, 5);
        let c5 = address(5, 3);

        invalidation
            .set_cell_dependencies(
                d1.clone(),
                [GridDependency::AxisValue(GridAxisValueDependency::columns(
                    2, 3,
                ))],
            )
            .expect("whole-column dependency should install without scalarizing all rows");
        invalidation
            .set_cell_dependencies(e1.clone(), [GridDependency::Cell(d1.clone())])
            .expect("downstream cell dependency should install");

        assert_eq!(
            invalidation.dirty_closure([c5.clone()]),
            set([c5.clone(), d1.clone(), e1.clone()])
        );

        let report = invalidation
            .apply_axis_edit(GridAxisEdit::delete_columns(2, 1))
            .expect("whole-column dependency should shrink with column deletion");
        assert_eq!(report.axis_value_edges_before, 1);
        assert_eq!(report.axis_value_edges_after, 1);
        assert_eq!(
            invalidation.semantic_dependencies_for(&address(1, 3)),
            vec![GridDependency::AxisValue(GridAxisValueDependency::columns(
                2, 2
            ))]
        );
        assert_eq!(
            invalidation.dirty_closure([address(5, 2)]),
            set([address(5, 2), address(1, 3), address(1, 4)])
        );
    }

    #[test]
    fn grid_invalidation_ref_axis_insert_shifts_dependents_and_dependencies() {
        let mut invalidation = GridInvalidationRef::new(bounds());
        invalidation
            .set_cell_dependencies(address(1, 2), [GridDependency::Cell(address(1, 1))])
            .expect("dependency should install");
        invalidation
            .set_cell_dependencies(address(1, 3), [GridDependency::Cell(address(1, 2))])
            .expect("dependency should install");

        let report = invalidation
            .apply_axis_edit(GridAxisEdit::insert_columns(1, 1))
            .expect("column insertion should transform invalidation graph");

        assert_eq!(report.dependent_cells_kept, 2);
        assert_eq!(report.dependent_cells_dropped, 0);
        assert_eq!(report.semantic_dependencies_kept, 2);
        assert_eq!(report.semantic_dependencies_dropped, 0);
        assert_eq!(report.scalar_edges_before, 2);
        assert_eq!(report.scalar_edges_after, 2);
        assert_eq!(
            invalidation.dependencies_for(&address(1, 3)),
            set([address(1, 2)])
        );
        assert_eq!(
            invalidation.dependencies_for(&address(1, 4)),
            set([address(1, 3)])
        );
        assert_eq!(
            invalidation.dirty_closure([address(1, 2)]),
            set([address(1, 2), address(1, 3), address(1, 4)])
        );
    }

    #[test]
    fn grid_invalidation_ref_axis_delete_drops_deleted_dependency_edges() {
        let mut invalidation = GridInvalidationRef::new(bounds());
        invalidation
            .set_cell_dependencies(address(1, 2), [GridDependency::Cell(address(1, 1))])
            .expect("dependency should install");

        let report = invalidation
            .apply_axis_edit(GridAxisEdit::delete_columns(1, 1))
            .expect("column deletion should transform invalidation graph");

        assert_eq!(report.dependent_cells_kept, 1);
        assert_eq!(report.dependent_cells_dropped, 0);
        assert_eq!(report.semantic_dependencies_kept, 0);
        assert_eq!(report.semantic_dependencies_dropped, 1);
        assert_eq!(report.scalar_edges_before, 1);
        assert_eq!(report.scalar_edges_after, 0);
        assert!(invalidation.dependencies_for(&address(1, 1)).is_empty());
        assert_eq!(
            invalidation.dirty_closure([address(1, 1)]),
            set([address(1, 1)])
        );
    }

    #[test]
    fn grid_invalidation_ref_axis_insert_expands_range_dependency() {
        let mut invalidation = GridInvalidationRef::new(bounds());
        let a1_a3 = GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap();
        invalidation
            .set_cell_dependencies(address(1, 2), [GridDependency::Range(a1_a3)])
            .expect("range dependency should install");

        let report = invalidation
            .apply_axis_edit(GridAxisEdit::insert_rows(2, 1))
            .expect("row insertion should expand semantic range dependencies");

        let expanded =
            GridRect::new("book:default", "sheet:default", 1, 1, 4, 1, bounds()).unwrap();
        assert_eq!(report.dependent_cells_kept, 1);
        assert_eq!(report.semantic_dependencies_kept, 1);
        assert_eq!(report.semantic_dependencies_dropped, 0);
        assert_eq!(report.scalar_edges_before, 3);
        assert_eq!(report.scalar_edges_after, 4);
        assert_eq!(
            invalidation.semantic_dependencies_for(&address(1, 2)),
            vec![GridDependency::Range(expanded)]
        );
        assert_eq!(
            invalidation.dirty_closure([address(2, 1)]),
            set([address(2, 1), address(1, 2)])
        );
    }

    #[test]
    fn grid_invalidation_ref_keeps_large_ranges_compressed() {
        let mut invalidation = GridInvalidationRef::with_scalarization_limit(bounds(), 2);
        let dependent = address(1, 5);
        let downstream = address(1, 4);
        let range = GridRect::new("book:default", "sheet:default", 1, 1, 3, 1, bounds()).unwrap();

        let installed = invalidation
            .set_cell_dependencies(dependent.clone(), [GridDependency::Range(range.clone())])
            .expect("large finite ranges should install as compressed reverse edges");
        invalidation
            .set_cell_dependencies(
                downstream.clone(),
                [GridDependency::Cell(dependent.clone())],
            )
            .expect("downstream cell dependency should install");

        assert_eq!(installed, 0);
        assert_eq!(invalidation.scalar_edge_count(), 1);
        assert_eq!(invalidation.compressed_range_edge_count(), 1);
        assert_eq!(
            invalidation.compressed_range_dependencies_for(&dependent),
            [range].into_iter().collect()
        );
        assert_eq!(
            invalidation.dirty_closure([address(2, 1)]),
            set([address(2, 1), dependent, downstream])
        );
    }

    #[test]
    fn grid_invalidation_ref_queries_compressed_ranges_through_block_index() {
        let large_bounds = ExcelGridBounds {
            max_rows: 3_000,
            max_cols: 5,
        };
        let mut invalidation = GridInvalidationRef::with_scalarization_limit(large_bounds, 2);
        let ranges = [
            GridRect::new(
                "book:default",
                "sheet:default",
                1,
                1,
                1_000,
                1,
                large_bounds,
            )
            .unwrap(),
            GridRect::new(
                "book:default",
                "sheet:default",
                1_001,
                1,
                2_000,
                1,
                large_bounds,
            )
            .unwrap(),
            GridRect::new(
                "book:default",
                "sheet:default",
                2_001,
                1,
                3_000,
                1,
                large_bounds,
            )
            .unwrap(),
        ];
        for (index, range) in ranges.into_iter().enumerate() {
            invalidation
                .set_cell_dependencies(
                    ExcelGridCellAddress::new(
                        "book:default",
                        "sheet:default",
                        u32::try_from(index + 1).unwrap(),
                        2,
                    ),
                    [GridDependency::Range(range)],
                )
                .expect("large ranges should install as compressed edges");
        }

        let report = invalidation
            .compressed_range_query_report(ExcelGridCellAddress::new(
                "book:default",
                "sheet:default",
                1_500,
                1,
            ))
            .expect("seed should be in bounds");

        assert_eq!(report.total_compressed_range_edges, 3);
        assert!(report.indexed_candidate_count < report.total_compressed_range_edges);
        assert_eq!(report.matched_dependent_count, 1);
        assert_eq!(
            report.dependents,
            [ExcelGridCellAddress::new(
                "book:default",
                "sheet:default",
                2,
                2
            )]
            .into_iter()
            .collect()
        );
    }

    #[test]
    fn grid_invalidation_ref_queries_dirty_rect_through_block_indexes() {
        let large_bounds = ExcelGridBounds {
            max_rows: 3_000,
            max_cols: 5,
        };
        let mut invalidation = GridInvalidationRef::with_scalarization_limit(large_bounds, 2);
        for index in 0..3 {
            let start_row = index * 1_000 + 1;
            let end_row = start_row + 999;
            invalidation
                .set_cell_dependencies(
                    ExcelGridCellAddress::new("book:default", "sheet:default", index + 1, 2),
                    [GridDependency::Range(
                        GridRect::new(
                            "book:default",
                            "sheet:default",
                            start_row,
                            1,
                            end_row,
                            1,
                            large_bounds,
                        )
                        .unwrap(),
                    )],
                )
                .expect("large ranges should install as compressed edges");
            invalidation
                .set_cell_dependencies(
                    ExcelGridCellAddress::new("book:default", "sheet:default", index + 1, 3),
                    [GridDependency::Cell(ExcelGridCellAddress::new(
                        "book:default",
                        "sheet:default",
                        start_row + 500,
                        1,
                    ))],
                )
                .expect("scalar dependency should install into the scalar block index");
        }
        invalidation
            .set_cell_dependencies(
                ExcelGridCellAddress::new("book:default", "sheet:default", 1, 4),
                [
                    GridDependency::Cell(ExcelGridCellAddress::new(
                        "book:default",
                        "sheet:default",
                        2,
                        2,
                    )),
                    GridDependency::Cell(ExcelGridCellAddress::new(
                        "book:default",
                        "sheet:default",
                        2,
                        3,
                    )),
                ],
            )
            .expect("downstream dependency should install");

        let report = invalidation
            .dirty_rect_query_report(
                GridRect::new(
                    "book:default",
                    "sheet:default",
                    1_495,
                    1,
                    1_505,
                    1,
                    large_bounds,
                )
                .unwrap(),
            )
            .expect("dirty rectangle should query");

        assert_eq!(report.seed_rect_cell_count, 11);
        assert_eq!(report.total_compressed_range_edges, 3);
        assert_eq!(report.total_scalar_edges, 5);
        assert!(
            report.indexed_compressed_range_candidate_count < report.total_compressed_range_edges
        );
        assert!(report.indexed_scalar_candidate_count < report.total_scalar_edges);
        assert_eq!(report.matched_compressed_range_dependent_count, 1);
        assert_eq!(report.matched_scalar_dependent_count, 1);
        assert_eq!(
            report.direct_dependents,
            [
                ExcelGridCellAddress::new("book:default", "sheet:default", 2, 2),
                ExcelGridCellAddress::new("book:default", "sheet:default", 2, 3)
            ]
            .into_iter()
            .collect()
        );
        assert_eq!(
            report.dirty_closure,
            [
                ExcelGridCellAddress::new("book:default", "sheet:default", 2, 2),
                ExcelGridCellAddress::new("book:default", "sheet:default", 2, 3),
                ExcelGridCellAddress::new("book:default", "sheet:default", 1, 4)
            ]
            .into_iter()
            .collect()
        );
    }

    #[test]
    fn grid_invalidation_ref_queries_axis_visibility_through_block_index() {
        let large_bounds = ExcelGridBounds {
            max_rows: 3_000,
            max_cols: 5,
        };
        let mut invalidation = GridInvalidationRef::new(large_bounds);
        let dependencies = [
            GridAxisVisibilityDependency::rows(1, 1_000),
            GridAxisVisibilityDependency::rows(1_001, 2_000),
            GridAxisVisibilityDependency::rows(2_001, 3_000),
        ];
        for (index, dependency) in dependencies.into_iter().enumerate() {
            invalidation
                .set_cell_dependencies(
                    ExcelGridCellAddress::new(
                        "book:default",
                        "sheet:default",
                        u32::try_from(index + 1).unwrap(),
                        2,
                    ),
                    [GridDependency::AxisVisibility(dependency)],
                )
                .expect("visibility dependencies should install as indexed edges");
        }
        let selected_dependent = ExcelGridCellAddress::new("book:default", "sheet:default", 2, 2);
        let downstream = ExcelGridCellAddress::new("book:default", "sheet:default", 1, 3);
        invalidation
            .set_cell_dependencies(
                downstream.clone(),
                [GridDependency::Cell(selected_dependent.clone())],
            )
            .expect("downstream dependency should install");

        let report = invalidation
            .axis_visibility_query_report(GridAxisVisibilityDependency::rows(1_500, 1_500))
            .expect("visibility seed should be in bounds");

        assert_eq!(report.total_axis_visibility_edges, 3);
        assert!(report.indexed_candidate_count < report.total_axis_visibility_edges);
        assert_eq!(report.matched_dependent_count, 1);
        assert_eq!(
            report.dependents,
            [selected_dependent.clone()].into_iter().collect()
        );
        assert_eq!(
            invalidation
                .dirty_closure_for_axis_visibility(GridAxisVisibilityDependency::rows(1_500, 1_500))
                .unwrap(),
            [selected_dependent, downstream].into_iter().collect()
        );
    }

    fn bind_a1_reference_for_provider_test(
        source_text: &str,
        caller_row: u32,
        caller_col: u32,
    ) -> oxfunc_core::value::ReferenceLike {
        use crate::grid::reference_engine::{
            StrictExcelGridReferenceProfile, excel_grid_reference_like_from_profile_record,
        };
        use oxfml_core::binding::{
            ReferenceAtomBindRequest, ReferenceAtomBindResult, ReferenceBindProfile,
        };
        use oxfml_core::source::FormulaChannelKind;
        use oxfml_core::syntax::token::TextSpan;

        let profile = StrictExcelGridReferenceProfile::with_bounds(bounds());
        let record = match profile.bind_atom(&ReferenceAtomBindRequest {
            source_channel: FormulaChannelKind::WorksheetA1,
            source_span: TextSpan::new(1, source_text.len()),
            source_text: source_text.to_string(),
            parsed_qualifier: None,
            workbook_id: "book:default".to_string(),
            sheet_id: "sheet:default".to_string(),
            caller_row,
            caller_col,
        }) {
            ReferenceAtomBindResult::Bound(record) => record,
            other => panic!("expected grid A1 bind to produce a profile record, got {other:?}"),
        };

        excel_grid_reference_like_from_profile_record(&record)
            .expect("profile record should lower to grid reference")
    }

    #[test]
    fn grid_axis_state_reports_hidden_sensitive_row_context() {
        let mut sheet = sheet();
        sheet.axis_state_mut().set_row(
            2,
            GridAxisProps {
                hidden_manual: true,
                ..GridAxisProps::visible()
            },
        );
        sheet.axis_state_mut().set_row(
            3,
            GridAxisProps {
                hidden_filter: true,
                ..GridAxisProps::visible()
            },
        );

        assert_eq!(
            sheet.axis_state().hidden_sensitive_row_context([1, 2, 3]),
            GridVisibilityRange {
                total_rows: 3,
                manually_hidden_rows: 1,
                filtered_hidden_rows: 1,
            }
        );
    }
}
