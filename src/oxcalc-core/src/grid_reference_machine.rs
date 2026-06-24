#![forbid(unsafe_code)]

//! Strict Excel-grid reference machines for W061.
//!
//! This module is deliberately boring storage: finite BTreeMap support for the
//! authored/computed grid surface plus a separate scalar dirty-closure oracle.
//! Optimized grid storage, template coalescing, spill repair, and structural
//! edit transforms prove themselves against this floor; they do not live here.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

use crate::excel_grid_reference::{
    EXCEL_GRID_PROFILE_ID, ExcelGridBounds, ExcelGridCellAddress, ExcelGridFormulaAnchor,
    ExcelGridReference, ExcelGridReferenceSystemProvider, ExcelGridReferenceTransformPayload,
    ExcelGridStructuralEdit, ExcelGridStructuredTable, ExcelGridStructuredTableColumn,
    StrictExcelGridReferenceProfile, decode_excel_grid_reference_payload,
    excel_grid_defined_name_key, excel_grid_reference_like_from_profile_record,
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

// GridRefError now lives in `crate::grid::error`; this re-export keeps the
// historical `grid_reference_machine::GridRefError` path during the
// decomposition.
pub use crate::grid::error::GridRefError;
// GridRect now lives in `crate::grid::geometry`; this re-export keeps the
// historical `grid_reference_machine::GridRect` path during the decomposition.
pub use crate::grid::geometry::GridRect;
// The authored grid cell types now live in `crate::grid::authored`; this
// re-export keeps the historical paths during the decomposition.
pub use crate::grid::authored::{GridAuthoredCell, GridFormulaCell};

pub const GRID_CALC_REF_DEFAULT_MATERIALIZATION_LIMIT: u64 = 100_000;
pub const GRID_INVALIDATION_REF_DEFAULT_SCALARIZATION_LIMIT: u64 = 100_000;
pub const GRID_INVALIDATION_COMPRESSED_RANGE_INDEX_BLOCK_SIZE: u32 = 1_024;
pub const GRID_OPTIMIZED_PROVIDER_MATERIALIZATION_LIMIT: usize = 100_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridAxisProps {
    pub size_twips: Option<u32>,
    pub hidden_manual: bool,
    pub hidden_filter: bool,
    pub outline_level: u8,
    pub collapsed: bool,
}

impl GridAxisProps {
    #[must_use]
    pub const fn visible() -> Self {
        Self {
            size_twips: None,
            hidden_manual: false,
            hidden_filter: false,
            outline_level: 0,
            collapsed: false,
        }
    }
}

impl Default for GridAxisProps {
    fn default() -> Self {
        Self::visible()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GridAxisState {
    rows: BTreeMap<u32, GridAxisProps>,
    cols: BTreeMap<u32, GridAxisProps>,
}

impl GridAxisState {
    #[must_use]
    pub fn row(&self, row: u32) -> GridAxisProps {
        self.rows.get(&row).cloned().unwrap_or_default()
    }

    #[must_use]
    pub fn col(&self, col: u32) -> GridAxisProps {
        self.cols.get(&col).cloned().unwrap_or_default()
    }

    pub fn set_row(&mut self, row: u32, props: GridAxisProps) {
        self.rows.insert(row, props);
    }

    pub fn set_col(&mut self, col: u32, props: GridAxisProps) {
        self.cols.insert(col, props);
    }

    #[must_use]
    pub fn hidden_sensitive_row_context(
        &self,
        rows: impl IntoIterator<Item = u32>,
    ) -> GridVisibilityRange {
        let mut total_rows = 0;
        let mut manually_hidden_rows = 0;
        let mut filtered_hidden_rows = 0;
        for row in rows {
            total_rows += 1;
            let props = self.row(row);
            if props.hidden_manual {
                manually_hidden_rows += 1;
            }
            if props.hidden_filter {
                filtered_hidden_rows += 1;
            }
        }
        GridVisibilityRange {
            total_rows,
            manually_hidden_rows,
            filtered_hidden_rows,
        }
    }

    fn aggregate_row_context_runs(
        &self,
        first_row: u32,
        last_row: u32,
    ) -> (Vec<GridAggregateRowContextRun>, usize, usize) {
        let mut runs = Vec::new();
        let mut default_row_runs = 0_usize;
        let mut explicit_axis_row_entries_visited = 0_usize;
        let mut cursor = first_row;
        for (&row, props) in self.rows.range(first_row..=last_row) {
            explicit_axis_row_entries_visited += 1;
            if cursor < row {
                push_aggregate_row_context_run(
                    &mut runs,
                    cursor,
                    row - 1,
                    GridAxisProps::visible(),
                );
                default_row_runs += 1;
            }
            push_aggregate_row_context_run(&mut runs, row, row, props.clone());
            cursor = row.saturating_add(1);
        }
        if cursor <= last_row {
            push_aggregate_row_context_run(&mut runs, cursor, last_row, GridAxisProps::visible());
            default_row_runs += 1;
        }
        (runs, explicit_axis_row_entries_visited, default_row_runs)
    }

    fn apply_axis_edit(
        &mut self,
        edit: GridAxisEdit,
        bounds: ExcelGridBounds,
    ) -> Result<(usize, usize), GridRefError> {
        validate_axis_edit(edit, bounds)?;
        let max = axis_max(edit.axis, bounds);
        let map = match edit.axis {
            GridAxis::Row => &mut self.rows,
            GridAxis::Column => &mut self.cols,
        };
        let old = std::mem::take(map);
        let mut kept = 0;
        let mut dropped = 0;
        for (index, props) in old {
            match transform_axis_index(index, edit.kind, max)? {
                Some(new_index) => {
                    map.insert(new_index, props);
                    kept += 1;
                }
                None => dropped += 1,
            }
        }
        Ok((kept, dropped))
    }
}

fn push_aggregate_row_context_run(
    runs: &mut Vec<GridAggregateRowContextRun>,
    first_row: u32,
    last_row: u32,
    props: GridAxisProps,
) {
    if first_row > last_row {
        return;
    }
    if let Some(last) = runs.last_mut() {
        if last.last_row.saturating_add(1) == first_row && last.props == props {
            last.last_row = last_row;
            return;
        }
    }
    runs.push(GridAggregateRowContextRun {
        first_row,
        last_row,
        props,
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridVisibilityRange {
    pub total_rows: u32,
    pub manually_hidden_rows: u32,
    pub filtered_hidden_rows: u32,
}

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

#[derive(Debug, Clone)]
pub struct GridHostInfoProvider<'a> {
    reference_provider: ExcelGridReferenceSystemProvider<'static>,
    axis_state: &'a GridAxisState,
}

impl<'a> GridHostInfoProvider<'a> {
    #[must_use]
    pub fn new(
        workbook_id: impl Into<String>,
        sheet_id: impl Into<String>,
        caller_row: u32,
        caller_col: u32,
        bounds: ExcelGridBounds,
        spill_facts: impl IntoIterator<Item = &'a GridSpillFact>,
        axis_state: &'a GridAxisState,
    ) -> Self {
        let mut reference_provider =
            ExcelGridReferenceSystemProvider::new(workbook_id, sheet_id, caller_row, caller_col)
                .with_bounds(bounds);
        for fact in spill_facts {
            if fact.blocked {
                continue;
            }
            reference_provider = reference_provider.with_spill_extent(
                fact.anchor.workbook_id.clone(),
                fact.anchor.sheet_id.clone(),
                fact.anchor.row,
                fact.anchor.col,
                fact.extent.clone(),
            );
        }
        Self {
            reference_provider,
            axis_state,
        }
    }

    pub fn aggregate_context_query_report(
        &self,
        reference: &oxfunc_core::value::ReferenceLike,
    ) -> Result<GridAggregateContextQueryReport, HostInfoError> {
        let plan = self.aggregate_context_query_plan(reference)?;
        let mut manually_hidden_rows = 0_u32;
        let mut filtered_hidden_rows = 0_u32;
        for run in &plan.row_runs {
            if run.props.hidden_manual {
                manually_hidden_rows = manually_hidden_rows.saturating_add(run.row_count());
            }
            if run.props.hidden_filter {
                filtered_hidden_rows = filtered_hidden_rows.saturating_add(run.row_count());
            }
        }
        Ok(GridAggregateContextQueryReport {
            resolved_rect: plan.resolved_rect,
            rows: plan.rows,
            cols: plan.cols,
            declared_cell_count: plan.declared_cell_count,
            row_context_runs: plan.row_runs.len(),
            explicit_axis_row_entries_visited: plan.explicit_axis_row_entries_visited,
            default_row_runs: plan.default_row_runs,
            axis_run_probe_count: plan.row_runs.len(),
            per_cell_context_expansion_count: plan.declared_cell_count,
            manually_hidden_rows,
            filtered_hidden_rows,
        })
    }

    fn aggregate_context_query_plan(
        &self,
        reference: &oxfunc_core::value::ReferenceLike,
    ) -> Result<GridAggregateContextQueryPlan, HostInfoError> {
        let rect = self
            .reference_provider
            .resolved_rect_for_reference(reference)
            .map_err(|error| HostInfoError::ProviderFailure {
                detail: format!("grid_aggregate_reference_context:{error:?}"),
            })?;
        let rows =
            usize::try_from(rect.row_count()).map_err(|_| HostInfoError::ProviderFailure {
                detail: "grid_aggregate_reference_rows_overflow".to_string(),
            })?;
        let cols =
            usize::try_from(rect.col_count()).map_err(|_| HostInfoError::ProviderFailure {
                detail: "grid_aggregate_reference_cols_overflow".to_string(),
            })?;
        let declared_cell_count =
            rows.checked_mul(cols)
                .ok_or_else(|| HostInfoError::ProviderFailure {
                    detail: "grid_aggregate_reference_cell_count_overflow".to_string(),
                })?;
        let (row_runs, explicit_axis_row_entries_visited, default_row_runs) = self
            .axis_state
            .aggregate_row_context_runs(rect.top_row, rect.bottom_row);
        Ok(GridAggregateContextQueryPlan {
            resolved_rect: rect,
            rows,
            cols,
            declared_cell_count,
            row_runs,
            explicit_axis_row_entries_visited,
            default_row_runs,
        })
    }
}

impl HostInfoProvider for GridHostInfoProvider<'_> {
    fn query_aggregate_reference_context(
        &self,
        reference: &oxfunc_core::value::ReferenceLike,
    ) -> Result<AggregateReferenceContext, HostInfoError> {
        let plan = self.aggregate_context_query_plan(reference)?;
        let mut cells = Vec::with_capacity(plan.declared_cell_count);
        for run in &plan.row_runs {
            let context = AggregateCellContext {
                row_hidden_manual: run.props.hidden_manual,
                row_filtered_out: run.props.hidden_filter,
                nested_subtotal_or_aggregate: false,
            };
            for _row in run.first_row..=run.last_row {
                for _col in 0..plan.cols {
                    cells.push(context);
                }
            }
        }

        AggregateReferenceContext::new(
            ArrayShape {
                rows: plan.rows,
                cols: plan.cols,
            },
            cells,
        )
        .ok_or_else(|| HostInfoError::ProviderFailure {
            detail: "grid_aggregate_reference_context_shape_invalid".to_string(),
        })
    }
}

fn register_table_overlay_references<'a>(
    mut provider: ExcelGridReferenceSystemProvider<'a>,
    table: &GridTableOverlay,
    caller_address: Option<&ExcelGridCellAddress>,
) -> ExcelGridReferenceSystemProvider<'a> {
    let mut structured_table = ExcelGridStructuredTable::new(
        table.table_name.clone(),
        table.table_range.clone(),
        table
            .columns
            .iter()
            .map(|column| {
                ExcelGridStructuredTableColumn::new(
                    column.column_name.clone(),
                    column.ordinal,
                    column.data_rect.clone(),
                )
            })
            .collect(),
    );
    if let Some(header_rect) = &table.header_rect {
        structured_table = structured_table.with_header_rect(header_rect.clone());
    }
    if let Some(totals_rect) = &table.totals_rect {
        structured_table = structured_table.with_totals_rect(totals_rect.clone());
    }
    provider = provider.with_structured_table(structured_table);
    provider = provider
        .with_structured_reference_text(&table.table_name, table.table_range.clone())
        .with_structured_reference_text(
            format!("{}[#All]", table.table_name),
            table.table_range.clone(),
        );
    if let Some(data_rect) = table_data_rect(table) {
        provider = provider.with_structured_reference_text(
            format!("{}[#Data]", table.table_name),
            data_rect.clone(),
        );
    }
    for column in &table.columns {
        provider = provider.with_structured_reference_text(
            format!("{}[{}]", table.table_name, column.column_name),
            column.data_rect.clone(),
        );
    }
    if caller_address.is_some_and(|address| table.table_range.contains(address)) {
        provider = provider.with_structured_reference_text("[#All]", table.table_range.clone());
        if let Some(header_rect) = &table.header_rect {
            provider = provider.with_structured_reference_text("[#Headers]", header_rect.clone());
        }
        if let Some(data_rect) = table_data_rect(table) {
            provider = provider.with_structured_reference_text("[#Data]", data_rect.clone());
        }
        if let Some(totals_rect) = &table.totals_rect {
            provider = provider.with_structured_reference_text("[#Totals]", totals_rect.clone());
        }
        for column in &table.columns {
            provider = provider.with_structured_reference_text(
                format!("[{}]", column.column_name),
                column.data_rect.clone(),
            );
        }
        if let Some(address) = caller_address.filter(|address| {
            table_data_rect(table).is_some_and(|data_rect| data_rect.contains(address))
        }) {
            for column in &table.columns {
                if column.data_rect.top_row <= address.row
                    && address.row <= column.data_rect.bottom_row
                {
                    provider = provider.with_structured_reference_text(
                        format!("[@{}]", column.column_name),
                        GridRect {
                            workbook_id: column.data_rect.workbook_id.clone(),
                            sheet_id: column.data_rect.sheet_id.clone(),
                            top_row: address.row,
                            left_col: column.data_rect.left_col,
                            bottom_row: address.row,
                            right_col: column.data_rect.right_col,
                        },
                    );
                }
            }
        }
    }
    provider
}

fn table_data_rect(table: &GridTableOverlay) -> Option<GridRect> {
    let first = &table.columns.first()?.data_rect;
    let mut top_row = first.top_row;
    let mut left_col = first.left_col;
    let mut bottom_row = first.bottom_row;
    let mut right_col = first.right_col;
    for column in &table.columns {
        top_row = top_row.min(column.data_rect.top_row);
        left_col = left_col.min(column.data_rect.left_col);
        bottom_row = bottom_row.max(column.data_rect.bottom_row);
        right_col = right_col.max(column.data_rect.right_col);
    }
    Some(GridRect {
        workbook_id: table.table_range.workbook_id.clone(),
        sheet_id: table.table_range.sheet_id.clone(),
        top_row,
        left_col,
        bottom_row,
        right_col,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridMergedRegion {
    pub rect: GridRect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureRenderedRegion {
    pub rect: GridRect,
    pub feature_kind: String,
    pub needs_refresh: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridTableLifecycleOperation {
    Set,
    Resize,
    Rename,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridTableLifecycleReport {
    pub operation: GridTableLifecycleOperation,
    pub old_table_key: Option<String>,
    pub new_table_key: Option<String>,
    pub feature_regions_removed: usize,
    pub feature_regions_added: usize,
    pub formula_cells_transformed: usize,
    pub formula_reference_transforms: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridNameLifecycleOperation {
    Rename,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridNameLifecycleReport {
    pub operation: GridNameLifecycleOperation,
    pub old_name_key: Option<String>,
    pub new_name_key: Option<String>,
    pub formula_cells_transformed: usize,
    pub formula_reference_transforms: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridTableColumn {
    pub column_id: String,
    pub column_name: String,
    pub ordinal: u32,
    pub data_rect: GridRect,
}

impl GridTableColumn {
    #[must_use]
    pub fn new(
        column_id: impl Into<String>,
        column_name: impl Into<String>,
        ordinal: u32,
        data_rect: GridRect,
    ) -> Self {
        Self {
            column_id: column_id.into(),
            column_name: column_name.into(),
            ordinal,
            data_rect,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridTableOverlay {
    pub table_id: String,
    pub table_name: String,
    pub table_range: GridRect,
    pub header_rect: Option<GridRect>,
    pub totals_rect: Option<GridRect>,
    pub columns: Vec<GridTableColumn>,
}

impl GridTableOverlay {
    #[must_use]
    pub fn new(
        table_id: impl Into<String>,
        table_name: impl Into<String>,
        table_range: GridRect,
        columns: Vec<GridTableColumn>,
    ) -> Self {
        Self {
            table_id: table_id.into(),
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

    fn check_sheet(
        &self,
        workbook_id: &str,
        sheet_id: &str,
        bounds: ExcelGridBounds,
    ) -> Result<(), GridRefError> {
        check_grid_rect_on_sheet(&self.table_range, workbook_id, sheet_id, bounds)?;
        if let Some(rect) = &self.header_rect {
            check_grid_rect_on_sheet(rect, workbook_id, sheet_id, bounds)?;
        }
        if let Some(rect) = &self.totals_rect {
            check_grid_rect_on_sheet(rect, workbook_id, sheet_id, bounds)?;
        }
        for column in &self.columns {
            check_grid_rect_on_sheet(&column.data_rect, workbook_id, sheet_id, bounds)?;
        }
        Ok(())
    }

    fn to_table_descriptor(&self) -> TableDescriptor {
        TableDescriptor {
            table_id: self.table_id.clone(),
            table_name: self.table_name.clone(),
            workbook_scope_ref: self.table_range.workbook_id.clone(),
            sheet_scope_ref: self.table_range.sheet_id.clone(),
            table_range_ref: grid_rect_a1_ref(&self.table_range),
            row_membership_identity: Some(format!("{}:rows:v1", self.table_id)),
            row_order_identity: Some(format!("{}:row-order:v1", self.table_id)),
            header_region_ref: self.header_rect.as_ref().map(grid_rect_a1_ref),
            totals_region_ref: self.totals_rect.as_ref().map(grid_rect_a1_ref),
            header_row_present: self.header_rect.is_some(),
            totals_row_present: self.totals_rect.is_some(),
            columns: self
                .columns
                .iter()
                .map(|column| TableColumnDescriptor {
                    column_id: column.column_id.clone(),
                    column_name: column.column_name.clone(),
                    ordinal: column.ordinal,
                    column_range_ref: grid_rect_a1_ref(&column.data_rect),
                })
                .collect(),
        }
    }

    fn transform_for_axis_edit(
        &self,
        edit: GridAxisEdit,
        bounds: ExcelGridBounds,
    ) -> Result<Option<Self>, GridRefError> {
        let (Some(table_range), _) = transform_rect_for_edit(&self.table_range, edit, bounds)?
        else {
            return Ok(None);
        };
        let header_rect = self
            .header_rect
            .as_ref()
            .map(|rect| transform_rect_for_edit(rect, edit, bounds).map(|(rect, _)| rect))
            .transpose()?
            .flatten();
        let totals_rect = self
            .totals_rect
            .as_ref()
            .map(|rect| transform_rect_for_edit(rect, edit, bounds).map(|(rect, _)| rect))
            .transpose()?
            .flatten();
        let mut columns = Vec::new();
        for column in &self.columns {
            if let Some(data_rect) = transform_rect_for_edit(&column.data_rect, edit, bounds)?.0 {
                columns.push(GridTableColumn {
                    column_id: column.column_id.clone(),
                    column_name: column.column_name.clone(),
                    ordinal: column.ordinal,
                    data_rect,
                });
            }
        }
        if columns.is_empty() {
            return Ok(None);
        }
        Ok(Some(Self {
            table_id: self.table_id.clone(),
            table_name: self.table_name.clone(),
            table_range,
            header_rect,
            totals_rect,
            columns,
        }))
    }
}

fn excel_grid_table_name_key(name: &str, _bounds: ExcelGridBounds) -> Option<String> {
    let name = name.trim();
    if name.is_empty()
        || name.contains('!')
        || name.contains(':')
        || name.contains(' ')
        || name.contains('[')
        || name.contains(']')
        || name.contains('#')
        || name.chars().any(char::is_control)
    {
        return None;
    }
    let mut chars = name.chars();
    let first = chars.next()?;
    if !(first.is_ascii_alphabetic() || first == '_' || first == '\\') {
        return None;
    }
    if !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '.') {
        return None;
    }
    Some(name.to_ascii_uppercase())
}

fn table_key_for_name(name: &str, bounds: ExcelGridBounds) -> Result<String, GridRefError> {
    excel_grid_table_name_key(name, bounds).ok_or_else(|| GridRefError::InvalidTableName {
        name: name.to_string(),
    })
}

fn defined_name_key_for_name(name: &str, bounds: ExcelGridBounds) -> Result<String, GridRefError> {
    excel_grid_defined_name_key(name, bounds).ok_or_else(|| GridRefError::InvalidDefinedName {
        name: name.to_string(),
    })
}

fn remove_table_overlay_feature_regions(
    regions: &mut Vec<FeatureRenderedRegion>,
    rect: &GridRect,
) -> usize {
    let before = regions.len();
    regions.retain(|region| !(region.feature_kind == "table-overlay" && region.rect == *rect));
    before - regions.len()
}

fn grid_table_descriptor_catalog<'a>(
    tables: impl IntoIterator<Item = &'a GridTableOverlay>,
) -> Vec<TableDescriptor> {
    tables
        .into_iter()
        .map(GridTableOverlay::to_table_descriptor)
        .collect()
}

fn grid_table_caller_context<'a>(
    tables: impl IntoIterator<Item = &'a GridTableOverlay>,
    address: &ExcelGridCellAddress,
) -> (Option<TableRef>, Option<TableCallerRegion>) {
    for table in tables {
        if !table.table_range.contains(address) {
            continue;
        }
        let enclosing_table_ref = Some(TableRef {
            table_id: table.table_id.clone(),
        });
        let caller_table_region = if table
            .header_rect
            .as_ref()
            .is_some_and(|rect| rect.contains(address))
        {
            Some(TableCallerRegion {
                table_id: table.table_id.clone(),
                region_kind: TableRegionKind::Headers,
                data_row_offset: None,
            })
        } else if let Some(data_rect) = table_data_rect(table) {
            if data_rect.contains(address) {
                Some(TableCallerRegion {
                    table_id: table.table_id.clone(),
                    region_kind: TableRegionKind::Data,
                    data_row_offset: Some(address.row - data_rect.top_row),
                })
            } else if table
                .totals_rect
                .as_ref()
                .is_some_and(|rect| rect.contains(address))
            {
                Some(TableCallerRegion {
                    table_id: table.table_id.clone(),
                    region_kind: TableRegionKind::Totals,
                    data_row_offset: None,
                })
            } else {
                None
            }
        } else if table
            .totals_rect
            .as_ref()
            .is_some_and(|rect| rect.contains(address))
        {
            Some(TableCallerRegion {
                table_id: table.table_id.clone(),
                region_kind: TableRegionKind::Totals,
                data_row_offset: None,
            })
        } else {
            None
        };
        return (enclosing_table_ref, caller_table_region);
    }
    (None, None)
}

fn grid_rect_a1_ref(rect: &GridRect) -> String {
    format!(
        "{}{}:{}{}",
        grid_column_letters(rect.left_col),
        rect.top_row,
        grid_column_letters(rect.right_col),
        rect.bottom_row
    )
}

fn grid_column_letters(mut col: u32) -> String {
    let mut letters = Vec::new();
    while col > 0 {
        let zero_based = (col - 1) % 26;
        letters.push(char::from(b'A' + u8::try_from(zero_based).unwrap_or(0)));
        col = (col - 1) / 26;
    }
    letters.iter().rev().collect()
}

fn check_grid_rect_on_sheet(
    rect: &GridRect,
    workbook_id: &str,
    sheet_id: &str,
    bounds: ExcelGridBounds,
) -> Result<(), GridRefError> {
    if rect.workbook_id != workbook_id || rect.sheet_id != sheet_id {
        return Err(GridRefError::RangeOnDifferentSheet {
            expected_workbook_id: workbook_id.to_string(),
            expected_sheet_id: sheet_id.to_string(),
            actual_workbook_id: rect.workbook_id.clone(),
            actual_sheet_id: rect.sheet_id.clone(),
        });
    }
    if !bounds.contains_row(rect.top_row)
        || !bounds.contains_row(rect.bottom_row)
        || !bounds.contains_col(rect.left_col)
        || !bounds.contains_col(rect.right_col)
    {
        return Err(GridRefError::RangeOutOfBounds {
            top_row: rect.top_row,
            left_col: rect.left_col,
            bottom_row: rect.bottom_row,
            right_col: rect.right_col,
            max_rows: bounds.max_rows,
            max_cols: bounds.max_cols,
        });
    }
    Ok(())
}

fn feature_rendered_region_blocks_spill(feature_kind: &str) -> bool {
    feature_kind.eq_ignore_ascii_case("table-overlay") || feature_kind.eq_ignore_ascii_case("table")
}

fn feature_rendered_region_refuses_inside_axis_edit(feature_kind: &str) -> bool {
    feature_kind.eq_ignore_ascii_case("pivot")
        || feature_kind.eq_ignore_ascii_case("pivot-report")
        || feature_kind.eq_ignore_ascii_case("pivot-table")
}

fn feature_rendered_region_marks_refresh_on_transform(feature_kind: &str) -> bool {
    feature_rendered_region_refuses_inside_axis_edit(feature_kind)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSpillFact {
    pub anchor: ExcelGridCellAddress,
    pub extent: GridRect,
    pub blocked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSpillEpochSnapshot {
    pub anchor: ExcelGridCellAddress,
    pub extent: GridRect,
    pub blocked: bool,
    pub value_epoch: u64,
}

impl GridSpillEpochSnapshot {
    #[must_use]
    pub fn new(fact: GridSpillFact, value_epoch: u64) -> Self {
        Self {
            anchor: fact.anchor,
            extent: fact.extent,
            blocked: fact.blocked,
            value_epoch,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSpillEpochLedgerEntry {
    pub snapshot: GridSpillEpochSnapshot,
    pub value_fingerprint: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GridSpillEpochLedger {
    entries: BTreeMap<ExcelGridCellAddress, GridSpillEpochLedgerEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSpillEpochLedgerUpdateReport {
    pub anchors_before: usize,
    pub anchors_after: usize,
    pub anchors_added: usize,
    pub anchors_removed: usize,
    pub anchors_changed: usize,
    pub extent_changed_anchors: usize,
    pub value_changed_anchors: usize,
    pub blocked_changed_anchors: usize,
    pub epochs_preserved: usize,
}

impl GridSpillEpochLedger {
    #[must_use]
    pub fn entries(&self) -> &BTreeMap<ExcelGridCellAddress, GridSpillEpochLedgerEntry> {
        &self.entries
    }

    #[must_use]
    pub fn snapshots(&self) -> BTreeMap<ExcelGridCellAddress, GridSpillEpochSnapshot> {
        self.entries
            .iter()
            .map(|(anchor, entry)| (anchor.clone(), entry.snapshot.clone()))
            .collect()
    }

    #[must_use]
    pub fn snapshot_for(&self, anchor: &ExcelGridCellAddress) -> Option<&GridSpillEpochSnapshot> {
        self.entries.get(anchor).map(|entry| &entry.snapshot)
    }

    pub fn update_from_spill_facts<F>(
        &mut self,
        spill_facts: &BTreeMap<ExcelGridCellAddress, GridSpillFact>,
        mut value_fingerprint_for: F,
    ) -> GridSpillEpochLedgerUpdateReport
    where
        F: FnMut(&GridSpillFact) -> String,
    {
        let previous = std::mem::take(&mut self.entries);
        let mut next = BTreeMap::new();
        let mut report = GridSpillEpochLedgerUpdateReport {
            anchors_before: previous.len(),
            anchors_after: spill_facts.len(),
            anchors_added: 0,
            anchors_removed: previous
                .keys()
                .filter(|anchor| !spill_facts.contains_key(*anchor))
                .count(),
            anchors_changed: 0,
            extent_changed_anchors: 0,
            value_changed_anchors: 0,
            blocked_changed_anchors: 0,
            epochs_preserved: 0,
        };

        for (anchor, fact) in spill_facts {
            let value_fingerprint = value_fingerprint_for(fact);
            let (value_epoch, changed) = match previous.get(anchor) {
                Some(entry) => {
                    let extent_changed = entry.snapshot.extent != fact.extent;
                    let blocked_changed = entry.snapshot.blocked != fact.blocked;
                    let value_changed = entry.value_fingerprint != value_fingerprint;
                    if extent_changed {
                        report.extent_changed_anchors += 1;
                    }
                    if blocked_changed {
                        report.blocked_changed_anchors += 1;
                    }
                    if value_changed {
                        report.value_changed_anchors += 1;
                    }
                    if extent_changed || blocked_changed || value_changed {
                        (entry.snapshot.value_epoch.saturating_add(1), true)
                    } else {
                        report.epochs_preserved += 1;
                        (entry.snapshot.value_epoch, false)
                    }
                }
                None => {
                    report.anchors_added += 1;
                    (1, true)
                }
            };
            if changed {
                report.anchors_changed += 1;
            }
            next.insert(
                anchor.clone(),
                GridSpillEpochLedgerEntry {
                    snapshot: GridSpillEpochSnapshot::new(fact.clone(), value_epoch),
                    value_fingerprint,
                },
            );
        }

        self.entries = next;
        report
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridSpillEpochChangeKind {
    Added,
    Removed,
    ExtentChanged,
    ValueChanged,
    BlockedChanged,
    ExtentAndValueChanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSpillEpochAnchorChange {
    pub anchor: ExcelGridCellAddress,
    pub kind: GridSpillEpochChangeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSpillEpochInvalidationReport {
    pub anchors_compared: usize,
    pub changed_anchors: Vec<GridSpillEpochAnchorChange>,
    pub unchanged_anchors: usize,
    pub extent_epoch_changed_anchors: usize,
    pub value_epoch_changed_anchors: usize,
    pub blocked_epoch_changed_anchors: usize,
    pub dirty_closure: BTreeSet<ExcelGridCellAddress>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct GridSpillPublicationCounters {
    facts_published: usize,
    facts_blocked: usize,
    ghost_cells_published: usize,
}

fn anchor_cell_rect(address: &ExcelGridCellAddress, bounds: ExcelGridBounds) -> GridRect {
    GridRect::new(
        address.workbook_id.clone(),
        address.sheet_id.clone(),
        address.row,
        address.col,
        address.row,
        address.col,
        bounds,
    )
    .expect("anchor address was already checked against grid bounds")
}

fn spill_extent_for_array(
    anchor: &ExcelGridCellAddress,
    shape: ArrayShape,
    bounds: ExcelGridBounds,
) -> Option<GridRect> {
    let rows = u32::try_from(shape.rows).ok()?;
    let cols = u32::try_from(shape.cols).ok()?;
    let bottom_row = anchor.row.checked_add(rows.checked_sub(1)?)?;
    let right_col = anchor.col.checked_add(cols.checked_sub(1)?)?;
    if !bounds.contains_row(bottom_row) || !bounds.contains_col(right_col) {
        return None;
    }
    GridRect::new(
        anchor.workbook_id.clone(),
        anchor.sheet_id.clone(),
        anchor.row,
        anchor.col,
        bottom_row,
        right_col,
        bounds,
    )
    .ok()
}

fn array_cell_address(
    anchor: &ExcelGridCellAddress,
    row_offset: usize,
    col_offset: usize,
) -> Option<ExcelGridCellAddress> {
    let row_offset = u32::try_from(row_offset).ok()?;
    let col_offset = u32::try_from(col_offset).ok()?;
    Some(ExcelGridCellAddress::new(
        anchor.workbook_id.clone(),
        anchor.sheet_id.clone(),
        anchor.row.checked_add(row_offset)?,
        anchor.col.checked_add(col_offset)?,
    ))
}

fn formula_count(authored: &BTreeMap<ExcelGridCellAddress, GridAuthoredCell>) -> usize {
    authored
        .values()
        .filter(|cell| matches!(cell, GridAuthoredCell::Formula(_)))
        .count()
}

fn formula_contains_grid_spill_reference(
    formula: &GridFormulaCell,
    address: &ExcelGridCellAddress,
    profile: &StrictExcelGridReferenceProfile,
    bounds: ExcelGridBounds,
) -> bool {
    let bound = bind_grid_formula_for_transform(formula, address, profile, bounds);
    if bound.normalized_references.iter().any(|normalized| {
        let NormalizedReference::ProfileSymbolic(record) = normalized else {
            return false;
        };
        if record.profile_id != EXCEL_GRID_PROFILE_ID {
            return false;
        }
        matches!(
            decode_excel_grid_reference_payload(&record.profile_payload),
            Some(ExcelGridReference::SpillAnchor { .. })
        )
    }) {
        return true;
    }

    formula.source_text.contains('#')
}

fn authored_contains_grid_spill_reference(
    authored: &BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
    profile: &StrictExcelGridReferenceProfile,
    bounds: ExcelGridBounds,
) -> bool {
    authored.iter().any(|(address, cell)| {
        let GridAuthoredCell::Formula(formula) = cell else {
            return false;
        };
        formula_contains_grid_spill_reference(formula, address, profile, bounds)
    })
}

fn count_formula_spill_publications<F>(
    spill_facts: &BTreeMap<ExcelGridCellAddress, GridSpillFact>,
    mut is_formula_anchor: F,
) -> GridSpillPublicationCounters
where
    F: FnMut(&ExcelGridCellAddress) -> bool,
{
    let mut counters = GridSpillPublicationCounters::default();
    for fact in spill_facts.values() {
        if !is_formula_anchor(&fact.anchor) {
            continue;
        }
        if fact.blocked {
            counters.facts_blocked += 1;
            continue;
        }
        counters.facts_published += 1;
        counters.ghost_cells_published +=
            usize::try_from(fact.extent.cell_count().saturating_sub(1)).unwrap_or(usize::MAX);
    }
    counters
}

fn calc_value_fingerprint(value: &CalcValue) -> String {
    format!("{value:?}")
}

fn calc_array_value_fingerprint(array: &CalcArray) -> String {
    let shape = array.shape();
    let mut fingerprint = format!("array:{}x{}:", shape.rows, shape.cols);
    for row in 0..shape.rows {
        for col in 0..shape.cols {
            match array.get(row, col) {
                Some(value) => fingerprint.push_str(&calc_value_fingerprint(value)),
                None => fingerprint.push_str("<missing>"),
            }
            fingerprint.push('|');
        }
    }
    fingerprint
}

fn blocked_spill_value_fingerprint(array: &CalcArray) -> String {
    format!("blocked:{}", calc_array_value_fingerprint(array))
}

fn manual_spill_fact_value_fingerprint(fact: &GridSpillFact) -> String {
    format!(
        "manual:{}:{}:{}:{}:{}:{}:{}",
        fact.anchor.workbook_id,
        fact.anchor.sheet_id,
        fact.anchor.row,
        fact.anchor.col,
        fact.extent.cell_count(),
        fact.blocked,
        if fact.blocked { "blocked" } else { "published" }
    )
}

fn blocked_formula_spill_extent_contains_anchor<F>(
    anchor: &ExcelGridCellAddress,
    spill_facts: &BTreeMap<ExcelGridCellAddress, GridSpillFact>,
    mut is_formula_anchor: F,
) -> bool
where
    F: FnMut(&ExcelGridCellAddress) -> bool,
{
    spill_facts.values().any(|fact| {
        fact.blocked
            && fact.anchor != *anchor
            && is_formula_anchor(&fact.anchor)
            && fact.extent.contains(anchor)
    })
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

#[derive(Debug, Clone, PartialEq)]
enum GridDenseValuePayload {
    CalcValues(Vec<CalcValue>),
    Numbers(Vec<f64>),
    Logicals(Vec<bool>),
    RepeatedCalcValue { value: CalcValue, len: usize },
}

impl GridDenseValuePayload {
    fn from_calc_array(array: &CalcArray) -> Self {
        let mut numbers = Vec::with_capacity(array.cell_count());
        let mut logicals = Vec::with_capacity(array.cell_count());
        for value in array.iter_row_major() {
            if value.rich.is_some() {
                return Self::CalcValues(array.iter_row_major().cloned().collect());
            }
            match &value.core {
                CoreValue::Number(number) if logicals.is_empty() => numbers.push(*number),
                CoreValue::Logical(logical) if numbers.is_empty() => logicals.push(*logical),
                _ => return Self::CalcValues(array.iter_row_major().cloned().collect()),
            }
        }
        if !logicals.is_empty() {
            return Self::Logicals(logicals);
        }
        Self::Numbers(numbers)
    }

    fn from_calc_values(values: Vec<CalcValue>) -> Self {
        let mut numbers = Vec::with_capacity(values.len());
        let mut logicals = Vec::with_capacity(values.len());
        for value in &values {
            if value.rich.is_some() {
                return Self::from_non_packed_calc_values(values);
            }
            match &value.core {
                CoreValue::Number(number) if logicals.is_empty() => numbers.push(*number),
                CoreValue::Logical(logical) if numbers.is_empty() => logicals.push(*logical),
                _ => return Self::from_non_packed_calc_values(values),
            }
        }
        if !logicals.is_empty() {
            return Self::Logicals(logicals);
        }
        Self::Numbers(numbers)
    }

    fn from_non_packed_calc_values(values: Vec<CalcValue>) -> Self {
        if let Some(first) = values.first() {
            if values.iter().all(|value| value == first) {
                return Self::RepeatedCalcValue {
                    value: first.clone(),
                    len: values.len(),
                };
            }
        }
        Self::CalcValues(values)
    }

    fn from_numbers(values: Vec<f64>) -> Self {
        Self::Numbers(values)
    }

    fn value_at_index(&self, index: usize) -> Option<CalcValue> {
        match self {
            Self::CalcValues(values) => values.get(index).cloned(),
            Self::Numbers(values) => values.get(index).copied().map(CalcValue::number),
            Self::Logicals(values) => values.get(index).copied().map(CalcValue::logical),
            Self::RepeatedCalcValue { value, len } => (index < *len).then(|| value.clone()),
        }
    }

    fn estimated_payload_bytes(&self) -> u64 {
        match self {
            Self::CalcValues(values) => values
                .iter()
                .map(estimated_calc_value_bytes)
                .fold(0_u64, u64::saturating_add),
            Self::Numbers(values) => u64::try_from(values.len())
                .unwrap_or(u64::MAX)
                .saturating_mul(u64::try_from(std::mem::size_of::<f64>()).unwrap_or(u64::MAX)),
            Self::Logicals(values) => {
                u64::try_from(values.len())
                    .unwrap_or(u64::MAX)
                    .saturating_add(7)
                    / 8
            }
            Self::RepeatedCalcValue { value, .. } => estimated_calc_value_bytes(value)
                .saturating_add(u64::try_from(std::mem::size_of::<usize>()).unwrap_or(u64::MAX)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct GridDenseValueStorage {
    payload: Arc<GridDenseValuePayload>,
    row_offset: u32,
    col_offset: u32,
    payload_col_count: u32,
}

impl GridDenseValueStorage {
    fn new_for_rect(rect: &GridRect, payload: GridDenseValuePayload) -> Self {
        Self {
            payload: Arc::new(payload),
            row_offset: 0,
            col_offset: 0,
            payload_col_count: rect.col_count(),
        }
    }

    fn slice_for_subrect(&self, parent_rect: &GridRect, subrect: &GridRect) -> Self {
        Self {
            payload: Arc::clone(&self.payload),
            row_offset: self
                .row_offset
                .saturating_add(subrect.top_row.saturating_sub(parent_rect.top_row)),
            col_offset: self
                .col_offset
                .saturating_add(subrect.left_col.saturating_sub(parent_rect.left_col)),
            payload_col_count: self.payload_col_count,
        }
    }

    fn value_at_rect(&self, rect: &GridRect, row: u32, col: u32) -> Option<CalcValue> {
        if row < rect.top_row
            || row > rect.bottom_row
            || col < rect.left_col
            || col > rect.right_col
        {
            return None;
        }
        let row_offset = u64::from(
            self.row_offset
                .saturating_add(row.saturating_sub(rect.top_row)),
        );
        let col_offset = u64::from(
            self.col_offset
                .saturating_add(col.saturating_sub(rect.left_col)),
        );
        let index = row_offset
            .saturating_mul(u64::from(self.payload_col_count))
            .saturating_add(col_offset);
        let index = usize::try_from(index).ok()?;
        self.payload.value_at_index(index)
    }

    fn row_major_values(&self, rect: &GridRect) -> Vec<CalcValue> {
        let mut values =
            Vec::with_capacity(usize::try_from(rect.cell_count()).unwrap_or(usize::MAX));
        for row in rect.top_row..=rect.bottom_row {
            for col in rect.left_col..=rect.right_col {
                if let Some(value) = self.value_at_rect(rect, row, col) {
                    values.push(value);
                }
            }
        }
        values
    }

    #[must_use]
    fn packed_numeric_cells(&self, rect: &GridRect) -> u64 {
        match self.payload.as_ref() {
            GridDenseValuePayload::Numbers(_) => rect.cell_count(),
            GridDenseValuePayload::CalcValues(_)
            | GridDenseValuePayload::Logicals(_)
            | GridDenseValuePayload::RepeatedCalcValue { .. } => 0,
        }
    }

    #[must_use]
    fn packed_logical_cells(&self, rect: &GridRect) -> u64 {
        match self.payload.as_ref() {
            GridDenseValuePayload::Logicals(_) => rect.cell_count(),
            GridDenseValuePayload::CalcValues(_)
            | GridDenseValuePayload::Numbers(_)
            | GridDenseValuePayload::RepeatedCalcValue { .. } => 0,
        }
    }

    #[must_use]
    fn shared_payload_id(&self) -> usize {
        Arc::as_ptr(&self.payload) as usize
    }

    #[must_use]
    fn shared_payload_bytes(&self) -> u64 {
        self.payload.estimated_payload_bytes()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridDenseValueRegion {
    pub rect: GridRect,
    storage: GridDenseValueStorage,
    revision: u64,
}

impl GridDenseValueRegion {
    #[must_use]
    pub fn row_major_values(&self) -> Vec<CalcValue> {
        self.storage.row_major_values(&self.rect)
    }

    fn value_at(&self, address: &ExcelGridCellAddress) -> Option<CalcValue> {
        if !self.rect.contains(address) {
            return None;
        }
        self.value_at_row_col(address.row, address.col)
    }

    fn value_at_row_col(&self, row: u32, col: u32) -> Option<CalcValue> {
        if row < self.rect.top_row
            || row > self.rect.bottom_row
            || col < self.rect.left_col
            || col > self.rect.right_col
        {
            return None;
        }
        self.storage.value_at_rect(&self.rect, row, col)
    }

    fn estimated_authored_bytes(&self) -> u64 {
        u64::try_from(std::mem::size_of::<Self>())
            .unwrap_or(u64::MAX)
            .saturating_add(estimated_grid_rect_heap_bytes(&self.rect))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridComputedDenseValueRegion {
    pub rect: GridRect,
    storage: GridDenseValueStorage,
    revision: u64,
    source: GridOptimizedCellSource,
}

impl GridComputedDenseValueRegion {
    #[must_use]
    pub fn cell_count(&self) -> u64 {
        self.rect.cell_count()
    }

    #[must_use]
    pub fn packed_numeric_cells(&self) -> u64 {
        self.storage.packed_numeric_cells(&self.rect)
    }

    #[must_use]
    pub fn packed_logical_cells(&self) -> u64 {
        self.storage.packed_logical_cells(&self.rect)
    }

    #[must_use]
    pub fn row_major_values(&self) -> Vec<CalcValue> {
        self.storage.row_major_values(&self.rect)
    }

    fn value_at(&self, address: &ExcelGridCellAddress) -> Option<CalcValue> {
        if !self.rect.contains(address) {
            return None;
        }
        self.value_at_row_col(address.row, address.col)
    }

    fn value_at_row_col(&self, row: u32, col: u32) -> Option<CalcValue> {
        if row < self.rect.top_row
            || row > self.rect.bottom_row
            || col < self.rect.left_col
            || col > self.rect.right_col
        {
            return None;
        }
        self.storage.value_at_rect(&self.rect, row, col)
    }
}

fn dense_region_publication_key_matches(
    lhs: &GridComputedDenseValueRegion,
    rhs: &GridComputedDenseValueRegion,
) -> bool {
    lhs.rect == rhs.rect && lhs.source == rhs.source
}

fn dense_region_publication_payload_matches(
    lhs: &GridComputedDenseValueRegion,
    rhs: &GridComputedDenseValueRegion,
) -> bool {
    lhs.row_major_values() == rhs.row_major_values()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridRepeatedFormulaRegion {
    pub rect: GridRect,
    pub formula: GridFormulaCell,
    revision: u64,
}

#[derive(Debug, Clone, PartialEq)]
enum GridOptimizedAuthoredCell {
    Number(f64),
    Literal(Box<CalcValue>),
    Formula(Box<GridFormulaCell>),
}

impl GridOptimizedAuthoredCell {
    fn from_authored(cell: GridAuthoredCell) -> Self {
        match cell {
            GridAuthoredCell::Literal(value) => Self::literal(value),
            GridAuthoredCell::Formula(formula) => Self::Formula(Box::new(formula)),
        }
    }

    fn literal(value: CalcValue) -> Self {
        if let CoreValue::Number(number) = value.core {
            if value.rich.is_none() {
                return Self::Number(number);
            }
            return Self::Literal(Box::new(CalcValue {
                core: CoreValue::Number(number),
                rich: value.rich,
            }));
        }
        Self::Literal(Box::new(value))
    }

    fn formula(formula: GridFormulaCell) -> Self {
        Self::Formula(Box::new(formula))
    }

    fn to_authored(&self) -> GridAuthoredCell {
        match self {
            Self::Number(number) => GridAuthoredCell::Literal(CalcValue::number(*number)),
            Self::Literal(value) => GridAuthoredCell::Literal((**value).clone()),
            Self::Formula(formula) => GridAuthoredCell::Formula((**formula).clone()),
        }
    }

    fn literal_value(&self) -> Option<CalcValue> {
        match self {
            Self::Number(number) => Some(CalcValue::number(*number)),
            Self::Literal(value) => Some((**value).clone()),
            Self::Formula(_) => None,
        }
    }

    fn formula_ref(&self) -> Option<&GridFormulaCell> {
        match self {
            Self::Formula(formula) => Some(formula.as_ref()),
            _ => None,
        }
    }

    fn formula_mut(&mut self) -> Option<&mut GridFormulaCell> {
        match self {
            Self::Formula(formula) => Some(formula.as_mut()),
            _ => None,
        }
    }

    fn estimated_authored_bytes(&self) -> u64 {
        match self {
            Self::Number(_) => u64::try_from(std::mem::size_of::<f64>()).unwrap_or(u64::MAX),
            Self::Literal(value) => estimated_calc_value_bytes(value),
            Self::Formula(formula) => estimated_formula_cell_bytes(formula),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct GridVersionedAuthoredCell {
    revision: u64,
    cell: GridOptimizedAuthoredCell,
}

#[derive(Debug, Clone, PartialEq)]
struct GridVersionedComputedCell {
    revision: u64,
    value: CalcValue,
    source: GridOptimizedCellSource,
}

/// Sparse computed point-cell storage with row- and column-occupancy indexes
/// kept consistent behind a single mutating API, so the point map and its two
/// indexes cannot silently drift out of sync.
#[derive(Debug, Clone, Default, PartialEq)]
struct SparsePointMap {
    points: BTreeMap<ExcelGridCellAddress, GridVersionedComputedCell>,
    by_row: BTreeMap<u32, BTreeSet<ExcelGridCellAddress>>,
    by_col: BTreeMap<u32, BTreeSet<ExcelGridCellAddress>>,
}

impl SparsePointMap {
    fn len(&self) -> usize {
        self.points.len()
    }

    fn get(&self, address: &ExcelGridCellAddress) -> Option<&GridVersionedComputedCell> {
        self.points.get(address)
    }

    fn contains_key(&self, address: &ExcelGridCellAddress) -> bool {
        self.points.contains_key(address)
    }

    fn keys(&self) -> impl Iterator<Item = &ExcelGridCellAddress> {
        self.points.keys()
    }

    fn iter(&self) -> impl Iterator<Item = (&ExcelGridCellAddress, &GridVersionedComputedCell)> {
        self.points.iter()
    }

    /// Insert or replace the cell at `address`, keeping both occupancy indexes
    /// in step in the same call.
    fn upsert(&mut self, address: ExcelGridCellAddress, cell: GridVersionedComputedCell) {
        self.by_row
            .entry(address.row)
            .or_default()
            .insert(address.clone());
        self.by_col
            .entry(address.col)
            .or_default()
            .insert(address.clone());
        self.points.insert(address, cell);
    }

    /// Remove the cell at `address`, unindexing it from both occupancy indexes
    /// in the same call.
    fn remove(&mut self, address: &ExcelGridCellAddress) -> Option<GridVersionedComputedCell> {
        let removed = self.points.remove(address);
        if removed.is_some() {
            if let Some(indexed) = self.by_row.get_mut(&address.row) {
                indexed.remove(address);
                if indexed.is_empty() {
                    self.by_row.remove(&address.row);
                }
            }
            if let Some(indexed) = self.by_col.get_mut(&address.col) {
                indexed.remove(address);
                if indexed.is_empty() {
                    self.by_col.remove(&address.col);
                }
            }
        }
        removed
    }

    /// Occupancy-proportional enumeration of occupied addresses inside `rect`:
    /// scan whichever axis index is smaller, never the full rect area (P-20).
    fn addresses_in_rect(&self, rect: &GridRect) -> Vec<ExcelGridCellAddress> {
        let mut addresses = BTreeSet::new();
        if rect.col_count() <= rect.row_count() {
            for col in rect.left_col..=rect.right_col {
                let Some(indexed) = self.by_col.get(&col) else {
                    continue;
                };
                addresses.extend(
                    indexed
                        .iter()
                        .filter(|address| {
                            rect.top_row <= address.row && address.row <= rect.bottom_row
                        })
                        .cloned(),
                );
            }
        } else {
            for row in rect.top_row..=rect.bottom_row {
                let Some(indexed) = self.by_row.get(&row) else {
                    continue;
                };
                addresses.extend(
                    indexed
                        .iter()
                        .filter(|address| {
                            rect.left_col <= address.col && address.col <= rect.right_col
                        })
                        .cloned(),
                );
            }
        }
        addresses.into_iter().collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridOptimizedValuation {
    workbook_id: String,
    sheet_id: String,
    bounds: ExcelGridBounds,
    sparse: SparsePointMap,
    dense_value_regions: Vec<GridComputedDenseValueRegion>,
    spill_facts: BTreeMap<ExcelGridCellAddress, GridSpillFact>,
    spill_value_fingerprints: BTreeMap<ExcelGridCellAddress, String>,
    spill_epoch_ledger: GridSpillEpochLedger,
    defined_names: BTreeMap<String, GridRect>,
    table_overlays: BTreeMap<String, GridTableOverlay>,
}

impl GridOptimizedValuation {
    #[must_use]
    pub fn new(
        workbook_id: impl Into<String>,
        sheet_id: impl Into<String>,
        bounds: ExcelGridBounds,
    ) -> Self {
        Self {
            workbook_id: workbook_id.into(),
            sheet_id: sheet_id.into(),
            bounds,
            sparse: SparsePointMap::default(),
            dense_value_regions: Vec::new(),
            spill_facts: BTreeMap::new(),
            spill_value_fingerprints: BTreeMap::new(),
            spill_epoch_ledger: GridSpillEpochLedger::default(),
            defined_names: BTreeMap::new(),
            table_overlays: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn with_spill_facts(
        workbook_id: impl Into<String>,
        sheet_id: impl Into<String>,
        bounds: ExcelGridBounds,
        spill_facts: BTreeMap<ExcelGridCellAddress, GridSpillFact>,
    ) -> Self {
        let spill_value_fingerprints = spill_facts
            .iter()
            .map(|(anchor, fact)| (anchor.clone(), manual_spill_fact_value_fingerprint(fact)))
            .collect::<BTreeMap<_, _>>();
        let mut spill_epoch_ledger = GridSpillEpochLedger::default();
        spill_epoch_ledger.update_from_spill_facts(&spill_facts, |fact| {
            manual_spill_fact_value_fingerprint(fact)
        });
        Self::with_spill_state(
            workbook_id,
            sheet_id,
            bounds,
            spill_facts,
            spill_value_fingerprints,
            spill_epoch_ledger,
        )
    }

    #[must_use]
    pub fn with_spill_state(
        workbook_id: impl Into<String>,
        sheet_id: impl Into<String>,
        bounds: ExcelGridBounds,
        spill_facts: BTreeMap<ExcelGridCellAddress, GridSpillFact>,
        spill_value_fingerprints: BTreeMap<ExcelGridCellAddress, String>,
        spill_epoch_ledger: GridSpillEpochLedger,
    ) -> Self {
        Self {
            workbook_id: workbook_id.into(),
            sheet_id: sheet_id.into(),
            bounds,
            sparse: SparsePointMap::default(),
            dense_value_regions: Vec::new(),
            spill_facts,
            spill_value_fingerprints,
            spill_epoch_ledger,
            defined_names: BTreeMap::new(),
            table_overlays: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn with_defined_names(mut self, defined_names: BTreeMap<String, GridRect>) -> Self {
        self.defined_names = defined_names;
        self
    }

    #[must_use]
    pub fn with_table_overlays(
        mut self,
        table_overlays: BTreeMap<String, GridTableOverlay>,
    ) -> Self {
        self.table_overlays = table_overlays;
        self
    }

    #[must_use]
    pub fn dense_value_regions(&self) -> &[GridComputedDenseValueRegion] {
        &self.dense_value_regions
    }

    #[must_use]
    pub fn spill_facts(&self) -> &BTreeMap<ExcelGridCellAddress, GridSpillFact> {
        &self.spill_facts
    }

    #[must_use]
    pub fn spill_epoch_ledger(&self) -> &GridSpillEpochLedger {
        &self.spill_epoch_ledger
    }

    #[must_use]
    pub fn spill_epoch_snapshots(&self) -> BTreeMap<ExcelGridCellAddress, GridSpillEpochSnapshot> {
        self.spill_epoch_ledger.snapshots()
    }

    pub fn set_spill_fact(&mut self, fact: GridSpillFact) -> Result<(), GridRefError> {
        self.check_address(&fact.anchor)?;
        self.check_rect(&fact.extent)?;
        self.spill_value_fingerprints.insert(
            fact.anchor.clone(),
            manual_spill_fact_value_fingerprint(&fact),
        );
        self.spill_facts.insert(fact.anchor.clone(), fact);
        self.refresh_spill_epoch_ledger();
        Ok(())
    }

    pub fn refresh_spill_epoch_ledger(&mut self) -> GridSpillEpochLedgerUpdateReport {
        let fingerprints = self.spill_value_fingerprints.clone();
        self.spill_epoch_ledger
            .update_from_spill_facts(&self.spill_facts, |fact| {
                fingerprints
                    .get(&fact.anchor)
                    .cloned()
                    .unwrap_or_else(|| manual_spill_fact_value_fingerprint(fact))
            })
    }

    #[must_use]
    pub fn sparse_computed_cells(&self) -> usize {
        self.sparse.len()
    }

    #[must_use]
    pub fn dense_computed_cells(&self) -> u64 {
        self.dense_value_regions
            .iter()
            .map(GridComputedDenseValueRegion::cell_count)
            .fold(0_u64, u64::saturating_add)
    }

    #[must_use]
    pub fn dense_computed_numeric_packed_cells(&self) -> u64 {
        self.dense_value_regions
            .iter()
            .map(GridComputedDenseValueRegion::packed_numeric_cells)
            .fold(0_u64, u64::saturating_add)
    }

    #[must_use]
    pub fn dense_computed_logical_packed_cells(&self) -> u64 {
        self.dense_value_regions
            .iter()
            .map(GridComputedDenseValueRegion::packed_logical_cells)
            .fold(0_u64, u64::saturating_add)
    }

    #[must_use]
    pub fn publication_delta_report_since(
        &self,
        previous: &Self,
    ) -> GridOptimizedPublicationDeltaReport {
        let mut report = GridOptimizedPublicationDeltaReport {
            same_grid_identity: self.workbook_id == previous.workbook_id
                && self.sheet_id == previous.sheet_id
                && self.bounds == previous.bounds,
            previous_sparse_cells: previous.sparse.len(),
            current_sparse_cells: self.sparse.len(),
            previous_dense_region_entries: previous.dense_value_regions.len(),
            current_dense_region_entries: self.dense_value_regions.len(),
            previous_dense_cells: previous.dense_computed_cells(),
            current_dense_cells: self.dense_computed_cells(),
            previous_spill_fact_entries: previous.spill_facts.len(),
            current_spill_fact_entries: self.spill_facts.len(),
            naive_current_computed_cell_publication_floor: self
                .dense_computed_cells()
                .saturating_add(u64::try_from(self.sparse.len()).unwrap_or(u64::MAX)),
            naive_full_grid_publication_floor: u64::from(self.bounds.max_rows)
                .saturating_mul(u64::from(self.bounds.max_cols)),
            ..GridOptimizedPublicationDeltaReport::default()
        };

        for (address, current) in self.sparse.iter() {
            match previous.sparse.get(address) {
                None => report.sparse_entries_added += 1,
                Some(previous_cell)
                    if previous_cell.value == current.value
                        && previous_cell.source == current.source =>
                {
                    report.sparse_entries_unchanged += 1;
                }
                Some(_) => report.sparse_entries_changed += 1,
            }
        }
        for address in previous.sparse.keys() {
            if !self.sparse.contains_key(address) {
                report.sparse_entries_removed += 1;
            }
        }

        let mut previous_dense_matched = vec![false; previous.dense_value_regions.len()];
        for current in &self.dense_value_regions {
            let mut matched_index = None;
            for (index, previous_region) in previous.dense_value_regions.iter().enumerate() {
                if previous_dense_matched[index]
                    || !dense_region_publication_key_matches(previous_region, current)
                {
                    continue;
                }
                matched_index = Some(index);
                break;
            }

            if let Some(index) = matched_index {
                previous_dense_matched[index] = true;
                let previous_region = &previous.dense_value_regions[index];
                if dense_region_publication_payload_matches(previous_region, current) {
                    report.dense_region_entries_unchanged += 1;
                    report.dense_region_cells_unchanged = report
                        .dense_region_cells_unchanged
                        .saturating_add(current.cell_count());
                } else {
                    report.dense_region_entries_changed += 1;
                    report.dense_region_cells_changed = report
                        .dense_region_cells_changed
                        .saturating_add(current.cell_count());
                }
            } else {
                report.dense_region_entries_added += 1;
                report.dense_region_cells_added = report
                    .dense_region_cells_added
                    .saturating_add(current.cell_count());
            }
        }
        for (index, previous_region) in previous.dense_value_regions.iter().enumerate() {
            if previous_dense_matched[index] {
                continue;
            }
            report.dense_region_entries_removed += 1;
            report.dense_region_cells_removed = report
                .dense_region_cells_removed
                .saturating_add(previous_region.cell_count());
        }

        for (anchor, current) in &self.spill_facts {
            match previous.spill_facts.get(anchor) {
                None => report.spill_fact_entries_added += 1,
                Some(previous_fact) if previous_fact == current => {
                    report.spill_fact_entries_unchanged += 1;
                }
                Some(_) => report.spill_fact_entries_changed += 1,
            }
        }
        for anchor in previous.spill_facts.keys() {
            if !self.spill_facts.contains_key(anchor) {
                report.spill_fact_entries_removed += 1;
            }
        }

        report
    }

    #[must_use]
    pub fn read_cell(&self, address: &ExcelGridCellAddress) -> GridOptimizedComputedReadout {
        if !self.contains_address(address) {
            return GridOptimizedComputedReadout {
                address: address.clone(),
                computed: CalcValue::empty(),
                source: None,
            };
        }

        let mut best_revision = 0;
        let mut best_value = None;
        let mut best_source = None;

        if let Some(point) = self.sparse.get(address) {
            best_revision = point.revision;
            best_value = Some(point.value.clone());
            best_source = Some(point.source);
        }

        for region in &self.dense_value_regions {
            if region.revision <= best_revision {
                continue;
            }
            let Some(value) = region.value_at(address) else {
                continue;
            };
            best_revision = region.revision;
            best_value = Some(value);
            best_source = Some(region.source);
        }

        GridOptimizedComputedReadout {
            address: address.clone(),
            computed: best_value.unwrap_or_else(CalcValue::empty),
            source: best_source,
        }
    }

    #[must_use]
    pub fn sampled_readout(
        &self,
        addresses: impl IntoIterator<Item = ExcelGridCellAddress>,
    ) -> Vec<GridOptimizedComputedReadout> {
        addresses
            .into_iter()
            .map(|address| self.read_cell(&address))
            .collect()
    }

    pub fn tile_snapshot_report(
        &self,
        rect: GridRect,
    ) -> Result<GridOptimizedTileSnapshotReport, GridRefError> {
        self.check_rect(&rect)?;
        let resolved_rect = rect.clone();
        let provider = self.reference_system_provider(rect.top_row, rect.left_col);
        let measured = provider
            .resolved_values_for_rect_with_report(&resolved_rect)
            .map_err(|error| GridRefError::ReferenceProvider {
                detail: format!("{error:?}"),
            })?;
        let value_payload_bytes = measured
            .values
            .defined_cells
            .iter()
            .map(|cell| estimated_calc_value_frame_payload_bytes(&cell.value))
            .fold(0_u64, u64::saturating_add);
        let defined_cell_count = measured.report.defined_cell_count;
        let subscribed_cell_count = rect.cell_count();
        let defined_entry_bytes = u64::try_from(defined_cell_count)
            .unwrap_or(u64::MAX)
            .saturating_mul(TILE_SNAPSHOT_CELL_ENTRY_BYTES)
            .saturating_add(value_payload_bytes);
        let estimated_frame_bytes = TILE_SNAPSHOT_FRAME_HEADER_BYTES
            .saturating_add(estimated_grid_rect_heap_bytes(&rect))
            .saturating_add(defined_entry_bytes);

        Ok(GridOptimizedTileSnapshotReport {
            rect,
            subscribed_cell_count,
            defined_cell_count,
            blank_cell_count: subscribed_cell_count
                .saturating_sub(u64::try_from(defined_cell_count).unwrap_or(u64::MAX)),
            dense_value_cells_visited: measured.report.dense_value_cells_visited,
            sparse_value_cells_visited: measured.report.sparse_value_cells_visited,
            compact_regions_intersected: measured.report.compact_regions_intersected,
            estimated_value_payload_bytes: value_payload_bytes,
            estimated_frame_bytes,
            full_grid_cell_floor: u64::from(self.bounds.max_rows)
                .saturating_mul(u64::from(self.bounds.max_cols)),
            full_grid_dense_numeric_bytes_floor: u64::from(self.bounds.max_rows)
                .saturating_mul(u64::from(self.bounds.max_cols))
                .saturating_mul(u64::try_from(std::mem::size_of::<f64>()).unwrap_or(u64::MAX)),
        })
    }

    #[must_use]
    pub fn reference_system_provider(
        &self,
        caller_row: u32,
        caller_col: u32,
    ) -> GridOptimizedReferenceSystemProvider<'_> {
        GridOptimizedReferenceSystemProvider::new(self, caller_row, caller_col)
    }

    #[must_use]
    pub fn reference_system_provider_with_dense_materialization_limit(
        &self,
        caller_row: u32,
        caller_col: u32,
        dense_materialization_limit: u64,
    ) -> GridOptimizedReferenceSystemProvider<'_> {
        let dense_materialization_limit =
            usize::try_from(dense_materialization_limit).unwrap_or(usize::MAX);
        self.reference_system_provider(caller_row, caller_col)
            .with_dense_materialization_limit(dense_materialization_limit)
    }

    pub fn insert_sparse_computed_value(
        &mut self,
        address: ExcelGridCellAddress,
        revision: u64,
        value: CalcValue,
        source: GridOptimizedCellSource,
    ) -> Result<(), GridRefError> {
        if !self.contains_address(&address) {
            return Err(GridRefError::AddressOutOfBounds {
                row: address.row,
                col: address.col,
                max_rows: self.bounds.max_rows,
                max_cols: self.bounds.max_cols,
            });
        }
        self.insert_sparse_value(address, revision, value, source);
        Ok(())
    }

    pub fn clear_formula_output_for_anchor_report(
        &mut self,
        anchor: &ExcelGridCellAddress,
    ) -> Result<GridOptimizedSpillClearReport, GridRefError> {
        if !self.contains_address(anchor) {
            return Err(GridRefError::AddressOutOfBounds {
                row: anchor.row,
                col: anchor.col,
                max_rows: self.bounds.max_rows,
                max_cols: self.bounds.max_cols,
            });
        }
        let report = self.clear_formula_output_for_anchor(anchor);
        self.refresh_spill_epoch_ledger();
        Ok(report)
    }

    fn insert_sparse_value(
        &mut self,
        address: ExcelGridCellAddress,
        revision: u64,
        value: CalcValue,
        source: GridOptimizedCellSource,
    ) {
        self.sparse.upsert(
            address,
            GridVersionedComputedCell {
                revision,
                value,
                source,
            },
        );
    }

    fn clear_formula_output_for_anchor(
        &mut self,
        anchor: &ExcelGridCellAddress,
    ) -> GridOptimizedSpillClearReport {
        let sparse_values_before = self.sparse.len();
        if let Some(fact) = self.spill_facts.remove(anchor) {
            self.spill_value_fingerprints.remove(anchor);
            let keys = self.sparse_addresses_in_grid_rect(&fact.extent);
            let sparse_values_removed = keys.len();
            let old_extent_cell_count = fact.extent.cell_count();
            for key in keys {
                self.remove_sparse_value(&key);
            }
            let (dense_value_regions_removed, dense_value_cells_removed) =
                self.remove_dense_value_regions_in_grid_rect(&fact.extent);
            GridOptimizedSpillClearReport {
                anchor: anchor.clone(),
                had_spill_fact: true,
                old_extent: fact.extent,
                old_extent_cell_count,
                naive_sparse_value_scan_floor: sparse_values_before,
                indexed_candidate_count: sparse_values_removed,
                sparse_values_removed,
                dense_value_regions_removed,
                dense_value_cells_removed,
            }
        } else {
            self.spill_value_fingerprints.remove(anchor);
            let sparse_values_removed = usize::from(self.remove_sparse_value(anchor).is_some());
            let old_extent = anchor_cell_rect(anchor, self.bounds);
            GridOptimizedSpillClearReport {
                anchor: anchor.clone(),
                had_spill_fact: false,
                old_extent,
                old_extent_cell_count: 1,
                naive_sparse_value_scan_floor: sparse_values_before,
                indexed_candidate_count: sparse_values_removed,
                sparse_values_removed,
                dense_value_regions_removed: 0,
                dense_value_cells_removed: 0,
            }
        }
    }

    fn sparse_addresses_in_grid_rect(&self, rect: &GridRect) -> Vec<ExcelGridCellAddress> {
        self.sparse_addresses_in_rect(rect)
    }

    fn remove_dense_value_regions_in_grid_rect(&mut self, rect: &GridRect) -> (usize, u64) {
        let mut regions_removed = 0_usize;
        let mut cells_removed = 0_u64;
        self.dense_value_regions.retain(|region| {
            if grid_rects_overlap(&region.rect, rect) {
                regions_removed += 1;
                cells_removed = cells_removed.saturating_add(region.rect.cell_count());
                false
            } else {
                true
            }
        });
        (regions_removed, cells_removed)
    }

    fn sparse_addresses_in_rect(&self, rect: &GridRect) -> Vec<ExcelGridCellAddress> {
        if rect.workbook_id != self.workbook_id || rect.sheet_id != self.sheet_id {
            return Vec::new();
        }
        self.sparse.addresses_in_rect(rect)
    }

    fn remove_sparse_value(
        &mut self,
        address: &ExcelGridCellAddress,
    ) -> Option<GridVersionedComputedCell> {
        self.sparse.remove(address)
    }

    fn push_dense_value_region(
        &mut self,
        rect: GridRect,
        values: Vec<CalcValue>,
        revision: u64,
        source: GridOptimizedCellSource,
    ) {
        self.push_dense_value_payload(
            rect,
            GridDenseValuePayload::from_calc_values(values),
            revision,
            source,
        );
    }

    fn push_dense_value_payload(
        &mut self,
        rect: GridRect,
        values: GridDenseValuePayload,
        revision: u64,
        source: GridOptimizedCellSource,
    ) {
        let storage = GridDenseValueStorage::new_for_rect(&rect, values);
        self.dense_value_regions.push(GridComputedDenseValueRegion {
            rect,
            storage,
            revision,
            source,
        });
    }

    fn contains_address(&self, address: &ExcelGridCellAddress) -> bool {
        address.workbook_id == self.workbook_id
            && address.sheet_id == self.sheet_id
            && self.bounds.contains_row(address.row)
            && self.bounds.contains_col(address.col)
    }

    fn check_address(&self, address: &ExcelGridCellAddress) -> Result<(), GridRefError> {
        if address.workbook_id != self.workbook_id || address.sheet_id != self.sheet_id {
            return Err(GridRefError::AddressOnDifferentSheet {
                expected_workbook_id: self.workbook_id.clone(),
                expected_sheet_id: self.sheet_id.clone(),
                actual_workbook_id: address.workbook_id.clone(),
                actual_sheet_id: address.sheet_id.clone(),
            });
        }
        if !self.bounds.contains_row(address.row) || !self.bounds.contains_col(address.col) {
            return Err(GridRefError::AddressOutOfBounds {
                row: address.row,
                col: address.col,
                max_rows: self.bounds.max_rows,
                max_cols: self.bounds.max_cols,
            });
        }
        Ok(())
    }

    fn check_rect(&self, rect: &GridRect) -> Result<(), GridRefError> {
        rect.check_workbook_sheet(&self.workbook_id, &self.sheet_id)?;
        if !self.bounds.contains_row(rect.top_row)
            || !self.bounds.contains_row(rect.bottom_row)
            || !self.bounds.contains_col(rect.left_col)
            || !self.bounds.contains_col(rect.right_col)
        {
            return Err(GridRefError::RangeOutOfBounds {
                top_row: rect.top_row,
                left_col: rect.left_col,
                bottom_row: rect.bottom_row,
                right_col: rect.right_col,
                max_rows: self.bounds.max_rows,
                max_cols: self.bounds.max_cols,
            });
        }
        Ok(())
    }
}

impl GridOptimizedValuation {
    /// Build the profile-pure shape resolver for this valuation: a strict-grid
    /// reference provider carrying only the valuation's spill extents, defined
    /// names, and table overlays, and deliberately NO cell values. The
    /// optimized engine resolves reference *shape* (rects, offsets, names,
    /// structured refs) through this one strict-grid coordinate implementation,
    /// while serving every cell *value* from its own compact storage. Sharing
    /// the coordinate logic here is intentional — shape is profile-pure spec,
    /// so the differential harness keeps its teeth on values, invalidation, and
    /// committed effects, which remain fully independent between the engines.
    fn shape_resolver(
        &self,
        caller_row: u32,
        caller_col: u32,
    ) -> ExcelGridReferenceSystemProvider<'static> {
        let mut shape_provider = ExcelGridReferenceSystemProvider::new(
            self.workbook_id.clone(),
            self.sheet_id.clone(),
            caller_row,
            caller_col,
        )
        .with_bounds(self.bounds);
        for fact in self.spill_facts.values() {
            if fact.blocked {
                continue;
            }
            shape_provider = shape_provider.with_spill_extent(
                fact.anchor.workbook_id.clone(),
                fact.anchor.sheet_id.clone(),
                fact.anchor.row,
                fact.anchor.col,
                fact.extent.clone(),
            );
        }
        for (name, rect) in &self.defined_names {
            shape_provider = shape_provider.with_defined_name(name, rect.clone());
        }
        let caller_address = ExcelGridCellAddress::new(
            self.workbook_id.clone(),
            self.sheet_id.clone(),
            caller_row,
            caller_col,
        );
        for table in self.table_overlays.values() {
            shape_provider =
                register_table_overlay_references(shape_provider, table, Some(&caller_address));
        }
        shape_provider
    }
}

#[derive(Debug, Clone)]
pub struct GridOptimizedReferenceSystemProvider<'a> {
    valuation: &'a GridOptimizedValuation,
    /// Profile-pure shape resolver built from the valuation (no cell values);
    /// see [`GridOptimizedValuation::shape_resolver`].
    shape_provider: ExcelGridReferenceSystemProvider<'static>,
    dense_materialization_limit: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GridOptimizedReferenceEnumerationReport {
    pub declared_cell_count: usize,
    pub defined_cell_count: usize,
    pub dense_value_cells_visited: u64,
    pub sparse_value_cells_visited: u64,
    pub compact_regions_intersected: usize,
}

impl GridOptimizedReferenceEnumerationReport {
    #[must_use]
    pub const fn slots_visited(&self) -> u64 {
        self.dense_value_cells_visited
            .saturating_add(self.sparse_value_cells_visited)
    }

    #[must_use]
    pub fn p20_occupied_slots_holds(&self) -> bool {
        self.slots_visited() == u64::try_from(self.defined_cell_count).unwrap_or(u64::MAX)
    }

    fn add_rect_report(&mut self, other: &Self) {
        self.declared_cell_count = self
            .declared_cell_count
            .saturating_add(other.declared_cell_count);
        self.defined_cell_count = self
            .defined_cell_count
            .saturating_add(other.defined_cell_count);
        self.dense_value_cells_visited = self
            .dense_value_cells_visited
            .saturating_add(other.dense_value_cells_visited);
        self.sparse_value_cells_visited = self
            .sparse_value_cells_visited
            .saturating_add(other.sparse_value_cells_visited);
        self.compact_regions_intersected = self
            .compact_regions_intersected
            .saturating_add(other.compact_regions_intersected);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridOptimizedMeasuredReferenceValues {
    pub values: ResolvedReferenceValues,
    pub report: GridOptimizedReferenceEnumerationReport,
}

impl<'a> GridOptimizedReferenceSystemProvider<'a> {
    #[must_use]
    pub fn new(valuation: &'a GridOptimizedValuation, caller_row: u32, caller_col: u32) -> Self {
        Self {
            shape_provider: valuation.shape_resolver(caller_row, caller_col),
            valuation,
            dense_materialization_limit: GRID_OPTIMIZED_PROVIDER_MATERIALIZATION_LIMIT,
        }
    }

    #[must_use]
    pub const fn with_dense_materialization_limit(mut self, limit: usize) -> Self {
        self.dense_materialization_limit = limit;
        self
    }

    fn resolved_values_for_rect(
        &self,
        rect: &GridRect,
    ) -> Result<ResolvedReferenceValues, ReferenceResolutionError> {
        self.resolved_values_for_rect_with_report(rect)
            .map(|measured| measured.values)
    }

    fn resolved_values_for_rect_with_report(
        &self,
        rect: &GridRect,
    ) -> Result<GridOptimizedMeasuredReferenceValues, ReferenceResolutionError> {
        let rows = usize::try_from(rect.row_count()).map_err(|_| {
            ReferenceResolutionError::ProviderFailure {
                detail: "optimized_grid_reference_extent_overflow".to_string(),
            }
        })?;
        let cols = usize::try_from(rect.col_count()).map_err(|_| {
            ReferenceResolutionError::ProviderFailure {
                detail: "optimized_grid_reference_extent_overflow".to_string(),
            }
        })?;

        let mut cells = BTreeMap::<(usize, usize), (u64, CalcValue)>::new();
        let mut report = GridOptimizedReferenceEnumerationReport {
            declared_cell_count: rows.saturating_mul(cols),
            ..GridOptimizedReferenceEnumerationReport::default()
        };
        for region in &self.valuation.dense_value_regions {
            let Some((top_row, left_col, bottom_row, right_col)) =
                intersect_rects(rect, &region.rect)
            else {
                continue;
            };
            report.compact_regions_intersected += 1;
            for row in top_row..=bottom_row {
                for col in left_col..=right_col {
                    let address = ExcelGridCellAddress::new(
                        rect.workbook_id.clone(),
                        rect.sheet_id.clone(),
                        row,
                        col,
                    );
                    let Some(value) = region.value_at(&address) else {
                        continue;
                    };
                    report.dense_value_cells_visited += 1;
                    insert_resolved_cell(&mut cells, rect, row, col, region.revision, value);
                }
            }
        }
        for address in self.valuation.sparse_addresses_in_rect(rect) {
            let Some(cell) = self.valuation.sparse.get(&address) else {
                continue;
            };
            report.sparse_value_cells_visited += 1;
            insert_resolved_cell(
                &mut cells,
                rect,
                address.row,
                address.col,
                cell.revision,
                cell.value.clone(),
            );
        }
        report.defined_cell_count = cells.len();

        let values = ResolvedReferenceValues::new(
            ResolvedReferenceExtent::new(rows, cols),
            cells
                .into_iter()
                .map(|((row, col), (_revision, value))| ResolvedReferenceCell::new(row, col, value))
                .collect(),
            Some(format!(
                "optimized-grid:v1:{}:{}:R{}C{}:R{}C{}",
                rect.workbook_id,
                rect.sheet_id,
                rect.top_row,
                rect.left_col,
                rect.bottom_row,
                rect.right_col
            )),
        );
        Ok(GridOptimizedMeasuredReferenceValues { values, report })
    }

    fn dense_cover_for_rect(
        &self,
        rect: &GridRect,
    ) -> Option<(usize, &GridComputedDenseValueRegion)> {
        self.valuation
            .dense_value_regions
            .iter()
            .enumerate()
            .filter(|(_, region)| dense_region_covers_resolved_rect(region, rect))
            .max_by_key(|(_, region)| region.revision)
    }

    fn materialize_large_dense_rect(
        &self,
        rect: &GridRect,
    ) -> Result<Option<CalcArray>, ReferenceResolutionError> {
        let rows = usize::try_from(rect.row_count()).map_err(|_| {
            ReferenceResolutionError::ProviderFailure {
                detail: "optimized_grid_reference_extent_overflow".to_string(),
            }
        })?;
        let cols = usize::try_from(rect.col_count()).map_err(|_| {
            ReferenceResolutionError::ProviderFailure {
                detail: "optimized_grid_reference_extent_overflow".to_string(),
            }
        })?;
        let cell_count =
            rows.checked_mul(cols)
                .ok_or_else(|| ReferenceResolutionError::ProviderFailure {
                    detail: "optimized_grid_reference_extent_overflow".to_string(),
                })?;
        if cell_count <= GRID_OPTIMIZED_PROVIDER_MATERIALIZATION_LIMIT {
            return Ok(None);
        }
        if cell_count > self.dense_materialization_limit {
            return Ok(None);
        }

        let Some((cover_index, cover)) = self.dense_cover_for_rect(rect) else {
            return Ok(None);
        };

        let mut cells = Vec::with_capacity(cell_count);
        let mut cell_revisions = Vec::with_capacity(cell_count);
        for row in rect.top_row..=rect.bottom_row {
            for col in rect.left_col..=rect.right_col {
                let Some(value) = cover.value_at_row_col(row, col) else {
                    return Ok(None);
                };
                cells.push(value);
                cell_revisions.push(cover.revision);
            }
        }

        for (region_index, region) in self.valuation.dense_value_regions.iter().enumerate() {
            if region_index == cover_index || region.revision < cover.revision {
                continue;
            }
            let Some((top_row, left_col, bottom_row, right_col)) =
                intersect_rects(rect, &region.rect)
            else {
                continue;
            };
            for row in top_row..=bottom_row {
                for col in left_col..=right_col {
                    let Some(value) = region.value_at_row_col(row, col) else {
                        continue;
                    };
                    let row_offset = usize::try_from(row - rect.top_row).unwrap_or(usize::MAX);
                    let col_offset = usize::try_from(col - rect.left_col).unwrap_or(usize::MAX);
                    let Some(index) = row_offset
                        .checked_mul(cols)
                        .and_then(|base| base.checked_add(col_offset))
                    else {
                        return Err(ReferenceResolutionError::ProviderFailure {
                            detail: "optimized_grid_reference_extent_overflow".to_string(),
                        });
                    };
                    if index >= cells.len() {
                        return Err(ReferenceResolutionError::ProviderFailure {
                            detail: "optimized_grid_reference_extent_overflow".to_string(),
                        });
                    }
                    if region.revision >= cell_revisions[index] {
                        cells[index] = value;
                        cell_revisions[index] = region.revision;
                    }
                }
            }
        }

        for address in self.valuation.sparse_addresses_in_rect(rect) {
            let Some(cell) = self.valuation.sparse.get(&address) else {
                continue;
            };
            if cell.revision < cover.revision {
                continue;
            }
            let row_offset = usize::try_from(address.row - rect.top_row).unwrap_or(usize::MAX);
            let col_offset = usize::try_from(address.col - rect.left_col).unwrap_or(usize::MAX);
            let Some(index) = row_offset
                .checked_mul(cols)
                .and_then(|base| base.checked_add(col_offset))
            else {
                return Err(ReferenceResolutionError::ProviderFailure {
                    detail: "optimized_grid_reference_extent_overflow".to_string(),
                });
            };
            if index >= cells.len() {
                return Err(ReferenceResolutionError::ProviderFailure {
                    detail: "optimized_grid_reference_extent_overflow".to_string(),
                });
            }
            if cell.revision >= cell_revisions[index] {
                cells[index] = cell.value.clone();
                cell_revisions[index] = cell.revision;
            }
        }

        CalcArray::new(ArrayShape { rows, cols }, cells)
            .map(Some)
            .ok_or_else(|| ReferenceResolutionError::ProviderFailure {
                detail: "optimized_grid_reference_dense_shape_invalid".to_string(),
            })
    }

    fn resolved_values_for_rects(
        &self,
        rects: &[GridRect],
    ) -> Result<ResolvedReferenceValues, ReferenceResolutionError> {
        self.resolved_values_for_rects_with_report(rects)
            .map(|measured| measured.values)
    }

    fn resolved_values_for_rects_with_report(
        &self,
        rects: &[GridRect],
    ) -> Result<GridOptimizedMeasuredReferenceValues, ReferenceResolutionError> {
        match rects {
            [] => Err(ReferenceResolutionError::UnresolvedReference {
                target: "empty optimized grid reference area set".to_string(),
            }),
            [rect] => self.resolved_values_for_rect_with_report(rect),
            _ => {
                let mut cells = Vec::new();
                let mut col_offset = 0usize;
                let mut identities = Vec::with_capacity(rects.len());
                let mut report = GridOptimizedReferenceEnumerationReport::default();
                for rect in rects {
                    let measured = self.resolved_values_for_rect_with_report(rect)?;
                    report.add_rect_report(&measured.report);
                    let values = measured.values;
                    let area_cols = values.declared_extent.cols;
                    for cell in values.defined_cells {
                        let flattened_col = (cell.row - 1)
                            .checked_mul(area_cols)
                            .and_then(|base| base.checked_add(cell.col))
                            .and_then(|col| col_offset.checked_add(col))
                            .ok_or_else(|| ReferenceResolutionError::ProviderFailure {
                                detail: "optimized_grid_multi_area_extent_overflow".to_string(),
                            })?;
                        cells.push(ResolvedReferenceCell::new(1, flattened_col, cell.value));
                    }
                    col_offset = col_offset
                        .checked_add(values.declared_extent.declared_cell_count())
                        .ok_or_else(|| ReferenceResolutionError::ProviderFailure {
                            detail: "optimized_grid_multi_area_extent_overflow".to_string(),
                        })?;
                    if let Some(identity) = values.reader_identity {
                        identities.push(identity);
                    }
                }
                cells.sort_by_key(|cell| (cell.row, cell.col));
                report.defined_cell_count = cells.len();
                let values = ResolvedReferenceValues::new(
                    ResolvedReferenceExtent::new(1, col_offset),
                    cells,
                    Some(format!(
                        "optimized-grid:v1:multi-area:{}",
                        identities.join("|")
                    )),
                );
                Ok(GridOptimizedMeasuredReferenceValues { values, report })
            }
        }
    }

    pub fn enumerate_values_with_report(
        &self,
        request: &ReferenceEnumerationRequest,
    ) -> Result<Option<GridOptimizedMeasuredReferenceValues>, ReferenceResolutionError> {
        if request.reference.system.0 != EXCEL_GRID_PROFILE_ID {
            return Ok(None);
        }
        let rects = self
            .shape_provider
            .resolved_rects_for_reference(&request.reference)?;
        self.resolved_values_for_rects_with_report(&rects).map(Some)
    }
}

impl ReferenceSystemProvider for GridOptimizedReferenceSystemProvider<'_> {
    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        let rects = self
            .shape_provider
            .resolved_rects_for_reference(&request.reference)?;
        if rects.len() > 1 {
            let values = self.resolved_values_for_rects(&rects)?;
            if values.declared_extent.declared_cell_count()
                > GRID_OPTIMIZED_PROVIDER_MATERIALIZATION_LIMIT
            {
                return Err(ReferenceResolutionError::ProviderFailure {
                    detail: "optimized_grid_reference_requires_sparse_enumeration".to_string(),
                });
            }
            return materialize_resolved_reference_values(&values).map(CalcValue::array);
        }
        let rect = rects[0].clone();
        if rect.row_count() == 1 && rect.col_count() == 1 {
            return Ok(self
                .valuation
                .read_cell(&ExcelGridCellAddress::new(
                    rect.workbook_id,
                    rect.sheet_id,
                    rect.top_row,
                    rect.left_col,
                ))
                .computed);
        }

        if let Some(array) = self.materialize_large_dense_rect(&rect)? {
            return Ok(CalcValue::array(array));
        }
        let declared_cell_count = usize::try_from(rect.row_count())
            .ok()
            .and_then(|rows| {
                usize::try_from(rect.col_count())
                    .ok()
                    .and_then(|cols| rows.checked_mul(cols))
            })
            .unwrap_or(usize::MAX);
        if declared_cell_count > GRID_OPTIMIZED_PROVIDER_MATERIALIZATION_LIMIT {
            return Err(ReferenceResolutionError::ProviderFailure {
                detail: "optimized_grid_reference_requires_sparse_enumeration".to_string(),
            });
        }
        let values = self.resolved_values_for_rect(&rect)?;
        materialize_resolved_reference_values(&values).map(CalcValue::array)
    }

    fn transform_reference(
        &self,
        request: &EvalTransformRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        match &request.transform {
            EvalTransformKind::Offset {
                row_offset,
                col_offset,
                height,
                width,
            } => self.shape_provider.offset_reference(
                &request.reference,
                *row_offset,
                *col_offset,
                *height,
                *width,
            ),
            _ => Err(ReferenceSystemError::Unsupported {
                operation: ReferenceSystemOperation::Transform,
            }),
        }
    }

    fn enumerate_values(
        &self,
        request: &ReferenceEnumerationRequest,
    ) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
        self.enumerate_values_with_report(request)
            .map(|measured| measured.map(|measured| measured.values))
    }

    fn resolve_text(
        &self,
        request: &ReferenceTextResolveRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        self.shape_provider.resolve_text(request)
    }

    fn facts(
        &self,
        request: &ReferenceFactsRequest,
    ) -> Result<ReferenceFacts, ReferenceSystemError> {
        Ok(reference_facts(&request.reference))
    }

    fn compose_references(
        &self,
        request: &ReferenceComposeRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        self.shape_provider.compose_references(request)
    }

    fn caller_context(&self) -> Option<CallerContext> {
        self.shape_provider.caller_context()
    }
}

fn insert_resolved_cell(
    cells: &mut BTreeMap<(usize, usize), (u64, CalcValue)>,
    rect: &GridRect,
    row: u32,
    col: u32,
    revision: u64,
    value: CalcValue,
) {
    let relative_row = usize::try_from(row - rect.top_row + 1).unwrap_or(usize::MAX);
    let relative_col = usize::try_from(col - rect.left_col + 1).unwrap_or(usize::MAX);
    let key = (relative_row, relative_col);
    let should_insert = cells.get(&key).map_or(true, |(existing_revision, _)| {
        revision >= *existing_revision
    });
    if should_insert {
        cells.insert(key, (revision, value));
    }
}

fn dense_region_covers_resolved_rect(
    region: &GridComputedDenseValueRegion,
    rect: &GridRect,
) -> bool {
    region.rect.workbook_id == rect.workbook_id
        && region.rect.sheet_id == rect.sheet_id
        && region.rect.top_row <= rect.top_row
        && region.rect.left_col <= rect.left_col
        && region.rect.bottom_row >= rect.bottom_row
        && region.rect.right_col >= rect.right_col
}

fn intersect_rects(lhs: &GridRect, rhs: &GridRect) -> Option<(u32, u32, u32, u32)> {
    if lhs.workbook_id != rhs.workbook_id || lhs.sheet_id != rhs.sheet_id {
        return None;
    }
    let top_row = lhs.top_row.max(rhs.top_row);
    let left_col = lhs.left_col.max(rhs.left_col);
    let bottom_row = lhs.bottom_row.min(rhs.bottom_row);
    let right_col = lhs.right_col.min(rhs.right_col);
    (top_row <= bottom_row && left_col <= right_col)
        .then_some((top_row, left_col, bottom_row, right_col))
}

fn grid_rect_intersection(
    lhs: &GridRect,
    rhs: &GridRect,
    bounds: ExcelGridBounds,
) -> Result<Option<GridRect>, GridRefError> {
    if lhs.workbook_id != rhs.workbook_id || lhs.sheet_id != rhs.sheet_id {
        return Ok(None);
    }
    let top_row = lhs.top_row.max(rhs.top_row);
    let left_col = lhs.left_col.max(rhs.left_col);
    let bottom_row = lhs.bottom_row.min(rhs.bottom_row);
    let right_col = lhs.right_col.min(rhs.right_col);
    if top_row > bottom_row || left_col > right_col {
        return Ok(None);
    }
    GridRect::new(
        lhs.workbook_id.clone(),
        lhs.sheet_id.clone(),
        top_row,
        left_col,
        bottom_row,
        right_col,
        bounds,
    )
    .map(Some)
}

fn pairwise_rect_partition_report<'a>(rects: impl Iterator<Item = &'a GridRect>) -> (u64, usize) {
    let rects = rects.collect::<Vec<_>>();
    let mut pair_checks = 0_u64;
    let mut overlap_count = 0_usize;
    for left_index in 0..rects.len() {
        for right_index in (left_index + 1)..rects.len() {
            pair_checks = pair_checks.saturating_add(1);
            if grid_rects_overlap(rects[left_index], rects[right_index]) {
                overlap_count += 1;
            }
        }
    }
    (pair_checks, overlap_count)
}

fn grid_rects_overlap(lhs: &GridRect, rhs: &GridRect) -> bool {
    lhs.workbook_id == rhs.workbook_id
        && lhs.sheet_id == rhs.sheet_id
        && lhs.top_row <= rhs.bottom_row
        && rhs.top_row <= lhs.bottom_row
        && lhs.left_col <= rhs.right_col
        && rhs.left_col <= lhs.right_col
}

fn rects_overlap_outside_anchor(
    lhs: &GridRect,
    rhs: &GridRect,
    anchor: &ExcelGridCellAddress,
) -> bool {
    if lhs.workbook_id != rhs.workbook_id || lhs.sheet_id != rhs.sheet_id {
        return false;
    }
    let top_row = lhs.top_row.max(rhs.top_row);
    let left_col = lhs.left_col.max(rhs.left_col);
    let bottom_row = lhs.bottom_row.min(rhs.bottom_row);
    let right_col = lhs.right_col.min(rhs.right_col);
    if top_row > bottom_row || left_col > right_col {
        return false;
    }
    let overlap_cells = u64::from(bottom_row - top_row + 1) * u64::from(right_col - left_col + 1);
    overlap_cells > 1
        || anchor.workbook_id != lhs.workbook_id
        || anchor.sheet_id != lhs.sheet_id
        || anchor.row < top_row
        || anchor.row > bottom_row
        || anchor.col < left_col
        || anchor.col > right_col
}

#[derive(Debug, Clone, Copy)]
pub struct GridOptimizedFormulaEvaluationRequest<'a> {
    pub address: &'a ExcelGridCellAddress,
    pub formula: &'a GridFormulaCell,
    pub source: GridOptimizedCellSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedRecalcReport {
    pub occupied_cells: u64,
    pub literal_cells: u64,
    pub formula_cells: u64,
    pub cells_evaluated: u64,
    pub formula_evaluations: u64,
    pub spill_repair_passes: usize,
    pub spill_repair_formula_evaluations: u64,
    pub spill_repair_converged: bool,
    pub sparse_literal_cells: u64,
    pub sparse_formula_cells: u64,
    pub dense_value_region_cells: u64,
    pub repeated_formula_region_cells: u64,
    pub formula_templates_prepared: usize,
    pub distinct_formula_templates: usize,
    pub formula_plan_cache_hits: u64,
    pub formula_plan_cache_misses: u64,
    pub compiled_formula_plan_cache_hits: u64,
    pub compiled_formula_plan_cache_misses: u64,
    pub compiled_formula_plans_cached: usize,
    pub computed_dense_value_regions: usize,
    pub computed_sparse_cells: usize,
    pub spill_facts_published: usize,
    pub spill_facts_blocked: usize,
    pub spill_ghost_cells_published: usize,
}

impl GridOptimizedRecalcReport {
    #[must_use]
    pub const fn p00_primary_exact_once_holds(&self) -> bool {
        self.cells_evaluated == self.occupied_cells
    }

    #[must_use]
    pub const fn p11_template_prepare_once_holds(&self) -> bool {
        self.formula_templates_prepared == self.distinct_formula_templates
    }

    #[must_use]
    pub const fn formula_plan_cache_lookups(&self) -> u64 {
        self.formula_plan_cache_hits
            .saturating_add(self.formula_plan_cache_misses)
    }

    #[must_use]
    pub fn formula_plan_cache_hit_rate_micros(&self) -> u64 {
        bytes_per_cell_micros(
            self.formula_plan_cache_hits,
            self.formula_plan_cache_lookups(),
        )
    }

    #[must_use]
    pub fn p14_plan_cache_hit_floor_holds(&self) -> bool {
        if self.formula_cells == 0 {
            return self.formula_plan_cache_lookups() == 0;
        }
        let distinct_templates = u64::try_from(self.distinct_formula_templates).unwrap_or(u64::MAX);
        self.formula_plan_cache_lookups() == self.formula_cells
            && self.formula_plan_cache_misses == distinct_templates
            && self.formula_plan_cache_hits >= self.formula_cells.saturating_sub(distinct_templates)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedFormulaPlanCacheRoundReport {
    pub round_index: usize,
    pub formula_cells: u64,
    pub distinct_formula_templates: usize,
    pub formula_plan_cache_hits: u64,
    pub formula_plan_cache_misses: u64,
    pub compiled_formula_plan_cache_hits: u64,
    pub compiled_formula_plan_cache_misses: u64,
    pub cached_template_count_after_round: usize,
    pub cached_compiled_plan_count_after_round: usize,
}

impl GridOptimizedFormulaPlanCacheRoundReport {
    #[must_use]
    pub const fn formula_plan_cache_lookups(&self) -> u64 {
        self.formula_plan_cache_hits
            .saturating_add(self.formula_plan_cache_misses)
    }

    #[must_use]
    pub fn formula_plan_cache_hit_rate_micros(&self) -> u64 {
        bytes_per_cell_micros(
            self.formula_plan_cache_hits,
            self.formula_plan_cache_lookups(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedFormulaPlanCacheReport {
    pub rounds: usize,
    pub formula_cells_per_round: u64,
    pub distinct_formula_templates: usize,
    pub first_round_misses: u64,
    pub later_round_misses: u64,
    pub total_hits: u64,
    pub total_misses: u64,
    pub total_compiled_plan_hits: u64,
    pub total_compiled_plan_misses: u64,
    pub cached_template_count: usize,
    pub cached_compiled_plan_count: usize,
    pub round_reports: Vec<GridOptimizedFormulaPlanCacheRoundReport>,
}

impl GridOptimizedFormulaPlanCacheReport {
    #[must_use]
    pub const fn total_lookups(&self) -> u64 {
        self.total_hits.saturating_add(self.total_misses)
    }

    #[must_use]
    pub fn hit_rate_micros(&self) -> u64 {
        bytes_per_cell_micros(self.total_hits, self.total_lookups())
    }

    #[must_use]
    pub fn p14_persistent_plan_cache_holds(&self) -> bool {
        if self.rounds == 0 || self.round_reports.len() != self.rounds {
            return false;
        }
        if self.formula_cells_per_round == 0 {
            return self.total_lookups() == 0;
        }
        let expected_lookups = self
            .formula_cells_per_round
            .saturating_mul(u64::try_from(self.rounds).unwrap_or(u64::MAX));
        self.total_lookups() == expected_lookups
            && self.first_round_misses
                == u64::try_from(self.distinct_formula_templates).unwrap_or(u64::MAX)
            && self.later_round_misses == 0
            && self.total_misses == self.first_round_misses
            && self.cached_template_count == self.distinct_formula_templates
            && self.cached_compiled_plan_count == self.distinct_formula_templates
            && self
                .round_reports
                .iter()
                .all(|round| round.formula_cells == self.formula_cells_per_round)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridOptimizedCompiledFormulaPlan {
    R1C1Scalar(GridOptimizedR1C1ScalarExpression),
    R1C1Binary(GridOptimizedR1C1BinaryPlan),
    R1C1RangeAggregate(GridOptimizedR1C1RangeAggregatePlan),
    R1C1If(GridOptimizedR1C1IfPlan),
    R1C1IfError(GridOptimizedR1C1IfErrorPlan),
    R1C1LogicalFunction(GridOptimizedR1C1LogicalFunctionPlan),
    R1C1Comparison(GridOptimizedR1C1ComparisonPlan),
    R1C1TextFunction(GridOptimizedR1C1TextFunctionPlan),
    R1C1Index(GridOptimizedR1C1IndexPlan),
}

impl GridOptimizedCompiledFormulaPlan {
    #[must_use]
    pub fn compile(formula: &GridFormulaCell) -> Option<Self> {
        if formula.source_channel != FormulaChannelKind::WorksheetR1C1 {
            return None;
        }
        let expression = normalized_r1c1_expression(&formula.source_text)?;
        compile_r1c1_range_aggregate_expression(&expression)
            .map(Self::R1C1RangeAggregate)
            .or_else(|| compile_r1c1_iferror_expression(&expression).map(Self::R1C1IfError))
            .or_else(|| compile_r1c1_if_expression(&expression).map(Self::R1C1If))
            .or_else(|| {
                compile_r1c1_logical_function_expression(&expression).map(Self::R1C1LogicalFunction)
            })
            .or_else(|| compile_r1c1_index_expression(&expression).map(Self::R1C1Index))
            .or_else(|| {
                compile_r1c1_text_function_expression(&expression).map(Self::R1C1TextFunction)
            })
            .or_else(|| compile_r1c1_comparison_expression(&expression).map(Self::R1C1Comparison))
            .or_else(|| compile_r1c1_binary_expression(&expression).map(Self::R1C1Binary))
            .or_else(|| parse_r1c1_scalar_expression(&expression).map(Self::R1C1Scalar))
    }

    #[must_use]
    fn r1c1_double_left() -> Self {
        Self::R1C1Binary(GridOptimizedR1C1BinaryPlan {
            left: Box::new(GridOptimizedR1C1ScalarExpression::Operand(
                GridOptimizedR1C1Operand::Ref(GridOptimizedR1C1Ref {
                    row: GridOptimizedR1C1AxisRef::Relative(0),
                    col: GridOptimizedR1C1AxisRef::Relative(-1),
                }),
            )),
            op: GridOptimizedR1C1BinaryOp::Multiply,
            right: Box::new(GridOptimizedR1C1ScalarExpression::Operand(
                GridOptimizedR1C1Operand::Number(
                    GridOptimizedR1C1NumberLiteral::new(2.0)
                        .expect("finite literal should compile"),
                ),
            )),
        })
    }

    fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        match self {
            Self::R1C1Scalar(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::R1C1Binary(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::R1C1RangeAggregate(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::R1C1If(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::R1C1IfError(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::R1C1LogicalFunction(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::R1C1Comparison(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::R1C1TextFunction(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::R1C1Index(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
        }
    }

    fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        match self {
            Self::R1C1Scalar(plan) => plan.evaluate_single_cell(address, valuation),
            Self::R1C1Binary(plan) => plan.evaluate_single_cell(address, valuation),
            Self::R1C1RangeAggregate(plan) => plan.evaluate_single_cell(address, valuation),
            Self::R1C1If(plan) => plan.evaluate_single_cell(address, valuation),
            Self::R1C1IfError(plan) => plan.evaluate_single_cell(address, valuation),
            Self::R1C1LogicalFunction(plan) => plan.evaluate_single_cell(address, valuation),
            Self::R1C1Comparison(plan) => plan.evaluate_single_cell(address, valuation),
            Self::R1C1TextFunction(plan) => plan.evaluate_single_cell(address, valuation),
            Self::R1C1Index(plan) => plan.evaluate_single_cell(address, valuation),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1BinaryPlan {
    left: Box<GridOptimizedR1C1ScalarExpression>,
    op: GridOptimizedR1C1BinaryOp,
    right: Box<GridOptimizedR1C1ScalarExpression>,
}

impl GridOptimizedR1C1BinaryPlan {
    fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let left = self.left.value_for_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?;
        let right = self.right.value_for_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?;
        self.op.apply(left, right)
    }

    fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let left = self.left.value_for_single_cell(address, valuation)?;
        let right = self.right.value_for_single_cell(address, valuation)?;
        self.op.apply(left, right)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridOptimizedR1C1RangeAggregatePlan {
    function: GridOptimizedR1C1RangeAggregateFunction,
    start: GridOptimizedR1C1Ref,
    end: GridOptimizedR1C1Ref,
}

impl GridOptimizedR1C1RangeAggregatePlan {
    fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let (start_row, start_col) = self.start.resolve(row, col)?;
        let (end_row, end_col) = self.end.resolve(row, col)?;
        aggregate_optimized_r1c1_rect(
            self.function,
            start_row.min(end_row),
            start_col.min(end_col),
            start_row.max(end_row),
            start_col.max(end_col),
            Some(region),
            row_major_formula_values,
            valuation,
        )
    }

    fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let (start_row, start_col) = self.start.resolve(address.row, address.col)?;
        let (end_row, end_col) = self.end.resolve(address.row, address.col)?;
        aggregate_optimized_r1c1_rect(
            self.function,
            start_row.min(end_row),
            start_col.min(end_col),
            start_row.max(end_row),
            start_col.max(end_col),
            None,
            &[],
            valuation,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1IfPlan {
    condition: GridOptimizedR1C1LogicalExpression,
    when_true: GridOptimizedR1C1ScalarExpression,
    when_false: GridOptimizedR1C1ScalarExpression,
}

impl GridOptimizedR1C1IfPlan {
    fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let condition = self.condition.evaluate_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?;
        match condition {
            GridOptimizedR1C1ConditionValue::Logical(true) => {
                self.when_true.evaluate_repeated_region_cell(
                    row,
                    col,
                    region,
                    row_major_formula_values,
                    valuation,
                )
            }
            GridOptimizedR1C1ConditionValue::Logical(false) => {
                self.when_false.evaluate_repeated_region_cell(
                    row,
                    col,
                    region,
                    row_major_formula_values,
                    valuation,
                )
            }
            GridOptimizedR1C1ConditionValue::Error(error) => Some(CalcValue::error(error)),
        }
    }

    fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let condition = self.condition.evaluate_single_cell(address, valuation)?;
        match condition {
            GridOptimizedR1C1ConditionValue::Logical(true) => {
                self.when_true.evaluate_single_cell(address, valuation)
            }
            GridOptimizedR1C1ConditionValue::Logical(false) => {
                self.when_false.evaluate_single_cell(address, valuation)
            }
            GridOptimizedR1C1ConditionValue::Error(error) => Some(CalcValue::error(error)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1IfErrorPlan {
    value: GridOptimizedR1C1ScalarExpression,
    fallback: GridOptimizedR1C1ScalarExpression,
}

impl GridOptimizedR1C1IfErrorPlan {
    fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let value = self.value.evaluate_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?;
        if matches!(value.core, CoreValue::Error(_)) {
            self.fallback.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            )
        } else {
            Some(value)
        }
    }

    fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let value = self.value.evaluate_single_cell(address, valuation)?;
        if matches!(value.core, CoreValue::Error(_)) {
            self.fallback.evaluate_single_cell(address, valuation)
        } else {
            Some(value)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1ComparisonPlan {
    comparison: GridOptimizedR1C1Comparison,
}

impl GridOptimizedR1C1ComparisonPlan {
    fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        self.comparison
            .evaluate_repeated_region_cell(row, col, region, row_major_formula_values, valuation)?
            .into_calc_value()
    }

    fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        self.comparison
            .evaluate_single_cell(address, valuation)?
            .into_calc_value()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1LogicalFunctionPlan {
    function: GridOptimizedR1C1LogicalFunction,
    arguments: Vec<GridOptimizedR1C1LogicalExpression>,
}

impl GridOptimizedR1C1LogicalFunctionPlan {
    fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        self.condition_for_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?
        .into_calc_value()
    }

    fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        self.condition_for_single_cell(address, valuation)?
            .into_calc_value()
    }

    fn condition_for_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<GridOptimizedR1C1ConditionValue> {
        let arguments = self
            .arguments
            .iter()
            .map(|argument| {
                argument.evaluate_repeated_region_cell(
                    row,
                    col,
                    region,
                    row_major_formula_values,
                    valuation,
                )
            })
            .collect::<Option<Vec<_>>>()?;
        Some(self.function.apply(&arguments))
    }

    fn condition_for_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<GridOptimizedR1C1ConditionValue> {
        let arguments = self
            .arguments
            .iter()
            .map(|argument| argument.evaluate_single_cell(address, valuation))
            .collect::<Option<Vec<_>>>()?;
        Some(self.function.apply(&arguments))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridOptimizedR1C1LogicalExpression {
    Comparison(GridOptimizedR1C1Comparison),
    Function(Box<GridOptimizedR1C1LogicalFunctionPlan>),
}

impl GridOptimizedR1C1LogicalExpression {
    fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<GridOptimizedR1C1ConditionValue> {
        match self {
            Self::Comparison(comparison) => comparison.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::Function(plan) => plan.condition_for_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
        }
    }

    fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<GridOptimizedR1C1ConditionValue> {
        match self {
            Self::Comparison(comparison) => comparison.evaluate_single_cell(address, valuation),
            Self::Function(plan) => plan.condition_for_single_cell(address, valuation),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridOptimizedR1C1LogicalFunction {
    And,
    Or,
    Not,
}

impl GridOptimizedR1C1LogicalFunction {
    const ALL: [Self; 3] = [Self::And, Self::Or, Self::Not];

    const fn name(self) -> &'static str {
        match self {
            Self::And => "AND",
            Self::Or => "OR",
            Self::Not => "NOT",
        }
    }

    fn arity_holds(self, len: usize) -> bool {
        match self {
            Self::And | Self::Or => len >= 1,
            Self::Not => len == 1,
        }
    }

    fn apply(
        self,
        arguments: &[GridOptimizedR1C1ConditionValue],
    ) -> GridOptimizedR1C1ConditionValue {
        let mut values = Vec::with_capacity(arguments.len());
        for argument in arguments {
            match *argument {
                GridOptimizedR1C1ConditionValue::Logical(value) => values.push(value),
                GridOptimizedR1C1ConditionValue::Error(error) => {
                    return GridOptimizedR1C1ConditionValue::Error(error);
                }
            }
        }
        match self {
            Self::And => {
                GridOptimizedR1C1ConditionValue::Logical(values.into_iter().all(|value| value))
            }
            Self::Or => {
                GridOptimizedR1C1ConditionValue::Logical(values.into_iter().any(|value| value))
            }
            Self::Not => GridOptimizedR1C1ConditionValue::Logical(!values[0]),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridOptimizedR1C1ScalarExpression {
    Operand(GridOptimizedR1C1Operand),
    UnaryMinus(GridOptimizedR1C1UnaryMinusPlan),
    Binary(GridOptimizedR1C1BinaryPlan),
    RangeAggregate(GridOptimizedR1C1RangeAggregatePlan),
    ArgumentAggregate(GridOptimizedR1C1ArgumentAggregatePlan),
    ScalarFunction(GridOptimizedR1C1ScalarFunctionPlan),
    ReferenceFunction(GridOptimizedR1C1ReferenceFunctionPlan),
    TextFunction(GridOptimizedR1C1TextFunctionPlan),
    Index(GridOptimizedR1C1IndexPlan),
    Match(GridOptimizedR1C1MatchPlan),
    VLookup(GridOptimizedR1C1VLookupPlan),
    If(Box<GridOptimizedR1C1IfPlan>),
    IfError(Box<GridOptimizedR1C1IfErrorPlan>),
}

impl GridOptimizedR1C1ScalarExpression {
    fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        match self {
            Self::Operand(operand) => operand.calc_value_for_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::UnaryMinus(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::Binary(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::RangeAggregate(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::ArgumentAggregate(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::ScalarFunction(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::ReferenceFunction(plan) => plan.evaluate_repeated_region_cell(row, col),
            Self::TextFunction(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::Index(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::Match(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::VLookup(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::If(plan) => (*plan).evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::IfError(plan) => (*plan).evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
        }
    }

    fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        match self {
            Self::Operand(operand) => operand.calc_value_for_single_cell(address, valuation),
            Self::UnaryMinus(plan) => plan.evaluate_single_cell(address, valuation),
            Self::Binary(plan) => plan.evaluate_single_cell(address, valuation),
            Self::RangeAggregate(plan) => plan.evaluate_single_cell(address, valuation),
            Self::ArgumentAggregate(plan) => plan.evaluate_single_cell(address, valuation),
            Self::ScalarFunction(plan) => plan.evaluate_single_cell(address, valuation),
            Self::ReferenceFunction(plan) => plan.evaluate_single_cell(address),
            Self::TextFunction(plan) => plan.evaluate_single_cell(address, valuation),
            Self::Index(plan) => plan.evaluate_single_cell(address, valuation),
            Self::Match(plan) => plan.evaluate_single_cell(address, valuation),
            Self::VLookup(plan) => plan.evaluate_single_cell(address, valuation),
            Self::If(plan) => (*plan).evaluate_single_cell(address, valuation),
            Self::IfError(plan) => (*plan).evaluate_single_cell(address, valuation),
        }
    }

    fn value_for_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<GridOptimizedR1C1Value> {
        let value = self.evaluate_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?;
        optimized_r1c1_value_from_calc_value(&value)
    }

    fn value_for_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<GridOptimizedR1C1Value> {
        let value = self.evaluate_single_cell(address, valuation)?;
        optimized_r1c1_value_from_calc_value(&value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1UnaryMinusPlan {
    value: Box<GridOptimizedR1C1ScalarExpression>,
}

impl GridOptimizedR1C1UnaryMinusPlan {
    fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let value = self.value.value_for_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?;
        Some(negate_optimized_r1c1_value(value))
    }

    fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let value = self.value.value_for_single_cell(address, valuation)?;
        Some(negate_optimized_r1c1_value(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1ArgumentAggregatePlan {
    function: GridOptimizedR1C1RangeAggregateFunction,
    arguments: Vec<GridOptimizedR1C1AggregateArgument>,
}

impl GridOptimizedR1C1ArgumentAggregatePlan {
    fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let mut state = GridOptimizedR1C1AggregateState::new();
        for argument in &self.arguments {
            match argument.accumulate_repeated_region_cell(
                self.function,
                &mut state,
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            )? {
                Ok(()) => {}
                Err(error) => return Some(error),
            }
        }
        Some(state.finish(self.function))
    }

    fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let mut state = GridOptimizedR1C1AggregateState::new();
        for argument in &self.arguments {
            match argument.accumulate_single_cell(self.function, &mut state, address, valuation)? {
                Ok(()) => {}
                Err(error) => return Some(error),
            }
        }
        Some(state.finish(self.function))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1ScalarFunctionPlan {
    function: GridOptimizedR1C1ScalarFunction,
    arguments: Vec<GridOptimizedR1C1ScalarExpression>,
}

impl GridOptimizedR1C1ScalarFunctionPlan {
    fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let arguments = self
            .arguments
            .iter()
            .map(|argument| {
                argument.value_for_repeated_region_cell(
                    row,
                    col,
                    region,
                    row_major_formula_values,
                    valuation,
                )
            })
            .collect::<Option<Vec<_>>>()?;
        Some(self.function.apply(&arguments))
    }

    fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let arguments = self
            .arguments
            .iter()
            .map(|argument| argument.value_for_single_cell(address, valuation))
            .collect::<Option<Vec<_>>>()?;
        Some(self.function.apply(&arguments))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridOptimizedR1C1ReferenceFunctionPlan {
    function: GridOptimizedR1C1ReferenceFunction,
    argument: GridOptimizedR1C1ReferenceFunctionArgument,
}

impl GridOptimizedR1C1ReferenceFunctionPlan {
    fn evaluate_repeated_region_cell(&self, row: u32, col: u32) -> Option<CalcValue> {
        self.function.apply(self.argument.resolve(row, col)?)
    }

    fn evaluate_single_cell(&self, address: &ExcelGridCellAddress) -> Option<CalcValue> {
        self.function
            .apply(self.argument.resolve(address.row, address.col)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1IndexPlan {
    start: GridOptimizedR1C1Ref,
    end: GridOptimizedR1C1Ref,
    row_index: Box<GridOptimizedR1C1ScalarExpression>,
    col_index: Box<GridOptimizedR1C1ScalarExpression>,
}

impl GridOptimizedR1C1IndexPlan {
    fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let range = self.resolve_range(row, col)?;
        let row_index =
            match optimized_r1c1_index_from_value(self.row_index.value_for_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            )?) {
                Ok(index) => index,
                Err(error) => return Some(error),
            };
        let col_index =
            match optimized_r1c1_index_from_value(self.col_index.value_for_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            )?) {
                Ok(index) => index,
                Err(error) => return Some(error),
            };
        self.value_at_indexed_range(
            range,
            row_index,
            col_index,
            Some(region),
            row_major_formula_values,
            valuation,
        )
    }

    fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let range = self.resolve_range(address.row, address.col)?;
        let row_index = match optimized_r1c1_index_from_value(
            self.row_index.value_for_single_cell(address, valuation)?,
        ) {
            Ok(index) => index,
            Err(error) => return Some(error),
        };
        let col_index = match optimized_r1c1_index_from_value(
            self.col_index.value_for_single_cell(address, valuation)?,
        ) {
            Ok(index) => index,
            Err(error) => return Some(error),
        };
        self.value_at_indexed_range(range, row_index, col_index, None, &[], valuation)
    }

    fn resolve_range(
        &self,
        anchor_row: u32,
        anchor_col: u32,
    ) -> Option<GridOptimizedR1C1ResolvedRef> {
        let (start_row, start_col) = self.start.resolve(anchor_row, anchor_col)?;
        let (end_row, end_col) = self.end.resolve(anchor_row, anchor_col)?;
        Some(GridOptimizedR1C1ResolvedRef {
            top_row: start_row.min(end_row),
            left_col: start_col.min(end_col),
            bottom_row: start_row.max(end_row),
            right_col: start_col.max(end_col),
        })
    }

    fn value_at_indexed_range(
        &self,
        range: GridOptimizedR1C1ResolvedRef,
        row_index: usize,
        col_index: usize,
        region: Option<&GridRepeatedFormulaRegion>,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let row_offset = u32::try_from(row_index.checked_sub(1)?).ok()?;
        let col_offset = u32::try_from(col_index.checked_sub(1)?).ok()?;
        let target_row = range.top_row.checked_add(row_offset)?;
        let target_col = range.left_col.checked_add(col_offset)?;
        if target_row > range.bottom_row || target_col > range.right_col {
            return Some(CalcValue::error(WorksheetErrorCode::Ref));
        }
        optimized_r1c1_calc_value_for_cell(
            target_row,
            target_col,
            region,
            row_major_formula_values,
            valuation,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1MatchPlan {
    lookup: Box<GridOptimizedR1C1ScalarExpression>,
    start: GridOptimizedR1C1Ref,
    end: GridOptimizedR1C1Ref,
}

impl GridOptimizedR1C1MatchPlan {
    fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let lookup = self.lookup.value_for_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?;
        self.match_in_range(
            row,
            col,
            lookup,
            Some(region),
            row_major_formula_values,
            valuation,
        )
    }

    fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let lookup = self.lookup.value_for_single_cell(address, valuation)?;
        self.match_in_range(address.row, address.col, lookup, None, &[], valuation)
    }

    fn resolve_range(
        &self,
        anchor_row: u32,
        anchor_col: u32,
    ) -> Option<GridOptimizedR1C1ResolvedRef> {
        let (start_row, start_col) = self.start.resolve(anchor_row, anchor_col)?;
        let (end_row, end_col) = self.end.resolve(anchor_row, anchor_col)?;
        Some(GridOptimizedR1C1ResolvedRef {
            top_row: start_row.min(end_row),
            left_col: start_col.min(end_col),
            bottom_row: start_row.max(end_row),
            right_col: start_col.max(end_col),
        })
    }

    fn match_in_range(
        &self,
        anchor_row: u32,
        anchor_col: u32,
        lookup: GridOptimizedR1C1Value,
        region: Option<&GridRepeatedFormulaRegion>,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let lookup_number = match lookup {
            GridOptimizedR1C1Value::Number(number) => number,
            GridOptimizedR1C1Value::Error(error) => return Some(CalcValue::error(error)),
        };
        let range = self.resolve_range(anchor_row, anchor_col)?;
        let row_count = range.bottom_row - range.top_row + 1;
        let col_count = range.right_col - range.left_col + 1;
        if row_count > 1 && col_count > 1 {
            return Some(CalcValue::error(WorksheetErrorCode::Value));
        }

        let mut position = 1_u64;
        for row in range.top_row..=range.bottom_row {
            for col in range.left_col..=range.right_col {
                let candidate = optimized_r1c1_value_for_cell(
                    row,
                    col,
                    region,
                    row_major_formula_values,
                    valuation,
                )?;
                match candidate {
                    GridOptimizedR1C1Value::Number(candidate) if candidate == lookup_number => {
                        return Some(CalcValue::number(position as f64));
                    }
                    GridOptimizedR1C1Value::Number(_) => {}
                    GridOptimizedR1C1Value::Error(error) => {
                        return Some(CalcValue::error(error));
                    }
                }
                position = position.saturating_add(1);
            }
        }
        Some(CalcValue::error(WorksheetErrorCode::NA))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1VLookupPlan {
    lookup: Box<GridOptimizedR1C1ScalarExpression>,
    start: GridOptimizedR1C1Ref,
    end: GridOptimizedR1C1Ref,
    col_index: Box<GridOptimizedR1C1ScalarExpression>,
}

impl GridOptimizedR1C1VLookupPlan {
    fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let lookup = self.lookup.value_for_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?;
        let col_index =
            match optimized_r1c1_index_from_value(self.col_index.value_for_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            )?) {
                Ok(index) => index,
                Err(error) => return Some(error),
            };
        self.lookup_in_range(
            row,
            col,
            lookup,
            col_index,
            Some(region),
            row_major_formula_values,
            valuation,
        )
    }

    fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let lookup = self.lookup.value_for_single_cell(address, valuation)?;
        let col_index = match optimized_r1c1_index_from_value(
            self.col_index.value_for_single_cell(address, valuation)?,
        ) {
            Ok(index) => index,
            Err(error) => return Some(error),
        };
        self.lookup_in_range(
            address.row,
            address.col,
            lookup,
            col_index,
            None,
            &[],
            valuation,
        )
    }

    fn resolve_range(
        &self,
        anchor_row: u32,
        anchor_col: u32,
    ) -> Option<GridOptimizedR1C1ResolvedRef> {
        let (start_row, start_col) = self.start.resolve(anchor_row, anchor_col)?;
        let (end_row, end_col) = self.end.resolve(anchor_row, anchor_col)?;
        Some(GridOptimizedR1C1ResolvedRef {
            top_row: start_row.min(end_row),
            left_col: start_col.min(end_col),
            bottom_row: start_row.max(end_row),
            right_col: start_col.max(end_col),
        })
    }

    fn lookup_in_range(
        &self,
        anchor_row: u32,
        anchor_col: u32,
        lookup: GridOptimizedR1C1Value,
        col_index: usize,
        region: Option<&GridRepeatedFormulaRegion>,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let lookup_number = match lookup {
            GridOptimizedR1C1Value::Number(number) => number,
            GridOptimizedR1C1Value::Error(error) => return Some(CalcValue::error(error)),
        };
        let range = self.resolve_range(anchor_row, anchor_col)?;
        let col_offset = u32::try_from(col_index.checked_sub(1)?).ok()?;
        let value_col = range.left_col.checked_add(col_offset)?;
        if value_col > range.right_col {
            return Some(CalcValue::error(WorksheetErrorCode::Ref));
        }

        for row in range.top_row..=range.bottom_row {
            let candidate = optimized_r1c1_value_for_cell(
                row,
                range.left_col,
                region,
                row_major_formula_values,
                valuation,
            )?;
            match candidate {
                GridOptimizedR1C1Value::Number(candidate) if candidate == lookup_number => {
                    return optimized_r1c1_calc_value_for_cell(
                        row,
                        value_col,
                        region,
                        row_major_formula_values,
                        valuation,
                    );
                }
                GridOptimizedR1C1Value::Number(_) => {}
                GridOptimizedR1C1Value::Error(error) => {
                    return Some(CalcValue::error(error));
                }
            }
        }
        Some(CalcValue::error(WorksheetErrorCode::NA))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridOptimizedR1C1TextFunctionPlan {
    Len {
        text: GridOptimizedR1C1Ref,
    },
    Left {
        text: GridOptimizedR1C1Ref,
        count: Box<GridOptimizedR1C1ScalarExpression>,
    },
    Right {
        text: GridOptimizedR1C1Ref,
        count: Box<GridOptimizedR1C1ScalarExpression>,
    },
    Concat {
        texts: Vec<GridOptimizedR1C1Ref>,
    },
}

impl GridOptimizedR1C1TextFunctionPlan {
    fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        match self {
            Self::Len { text } => {
                let text = self.text_for_repeated_region_cell(
                    *text,
                    row,
                    col,
                    region,
                    row_major_formula_values,
                    valuation,
                )?;
                Some(match text {
                    Ok(text) => CalcValue::number(text.len_utf16_code_units() as f64),
                    Err(error) => error,
                })
            }
            Self::Left { text, count } => self.text_slice_for_repeated_region_cell(
                *text,
                count,
                GridOptimizedR1C1TextSliceSide::Left,
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::Right { text, count } => self.text_slice_for_repeated_region_cell(
                *text,
                count,
                GridOptimizedR1C1TextSliceSide::Right,
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::Concat { texts } => {
                let mut units = Vec::new();
                for text_ref in texts {
                    match self.text_for_repeated_region_cell(
                        *text_ref,
                        row,
                        col,
                        region,
                        row_major_formula_values,
                        valuation,
                    )? {
                        Ok(text) => units.extend_from_slice(text.utf16_code_units()),
                        Err(error) => return Some(error),
                    }
                }
                Some(CalcValue::text(ExcelText::from_utf16_code_units(units)))
            }
        }
    }

    fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        match self {
            Self::Len { text } => {
                let text = self.text_for_single_cell(*text, address, valuation)?;
                Some(match text {
                    Ok(text) => CalcValue::number(text.len_utf16_code_units() as f64),
                    Err(error) => error,
                })
            }
            Self::Left { text, count } => self.text_slice_for_single_cell(
                *text,
                count,
                GridOptimizedR1C1TextSliceSide::Left,
                address,
                valuation,
            ),
            Self::Right { text, count } => self.text_slice_for_single_cell(
                *text,
                count,
                GridOptimizedR1C1TextSliceSide::Right,
                address,
                valuation,
            ),
            Self::Concat { texts } => {
                let mut units = Vec::new();
                for text_ref in texts {
                    match self.text_for_single_cell(*text_ref, address, valuation)? {
                        Ok(text) => units.extend_from_slice(text.utf16_code_units()),
                        Err(error) => return Some(error),
                    }
                }
                Some(CalcValue::text(ExcelText::from_utf16_code_units(units)))
            }
        }
    }

    fn text_slice_for_repeated_region_cell(
        &self,
        text_ref: GridOptimizedR1C1Ref,
        count: &GridOptimizedR1C1ScalarExpression,
        side: GridOptimizedR1C1TextSliceSide,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let text = match self.text_for_repeated_region_cell(
            text_ref,
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )? {
            Ok(text) => text,
            Err(error) => return Some(error),
        };
        let count =
            match optimized_r1c1_text_count_from_value(count.value_for_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            )?) {
                Ok(count) => count,
                Err(error) => return Some(error),
            };
        Some(optimized_r1c1_text_slice(text, count, side))
    }

    fn text_slice_for_single_cell(
        &self,
        text_ref: GridOptimizedR1C1Ref,
        count: &GridOptimizedR1C1ScalarExpression,
        side: GridOptimizedR1C1TextSliceSide,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let text = match self.text_for_single_cell(text_ref, address, valuation)? {
            Ok(text) => text,
            Err(error) => return Some(error),
        };
        let count = match optimized_r1c1_text_count_from_value(
            count.value_for_single_cell(address, valuation)?,
        ) {
            Ok(count) => count,
            Err(error) => return Some(error),
        };
        Some(optimized_r1c1_text_slice(text, count, side))
    }

    fn text_for_repeated_region_cell(
        &self,
        text_ref: GridOptimizedR1C1Ref,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<Result<ExcelText, CalcValue>> {
        let (target_row, target_col) = text_ref.resolve(row, col)?;
        optimized_r1c1_calc_value_for_cell(
            target_row,
            target_col,
            Some(region),
            row_major_formula_values,
            valuation,
        )
        .map(optimized_r1c1_text_from_calc_value)
    }

    fn text_for_single_cell(
        &self,
        text_ref: GridOptimizedR1C1Ref,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<Result<ExcelText, CalcValue>> {
        let (target_row, target_col) = text_ref.resolve(address.row, address.col)?;
        optimized_r1c1_calc_value_for_cell(target_row, target_col, None, &[], valuation)
            .map(optimized_r1c1_text_from_calc_value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GridOptimizedR1C1TextSliceSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GridOptimizedR1C1ReferenceFunctionArgument {
    CurrentCell,
    Ref(GridOptimizedR1C1Ref),
    Range {
        start: GridOptimizedR1C1Ref,
        end: GridOptimizedR1C1Ref,
    },
}

impl GridOptimizedR1C1ReferenceFunctionArgument {
    fn resolve(self, anchor_row: u32, anchor_col: u32) -> Option<GridOptimizedR1C1ResolvedRef> {
        match self {
            Self::CurrentCell => Some(GridOptimizedR1C1ResolvedRef {
                top_row: anchor_row,
                left_col: anchor_col,
                bottom_row: anchor_row,
                right_col: anchor_col,
            }),
            Self::Ref(reference) => {
                let (row, col) = reference.resolve(anchor_row, anchor_col)?;
                Some(GridOptimizedR1C1ResolvedRef {
                    top_row: row,
                    left_col: col,
                    bottom_row: row,
                    right_col: col,
                })
            }
            Self::Range { start, end } => {
                let (start_row, start_col) = start.resolve(anchor_row, anchor_col)?;
                let (end_row, end_col) = end.resolve(anchor_row, anchor_col)?;
                Some(GridOptimizedR1C1ResolvedRef {
                    top_row: start_row.min(end_row),
                    left_col: start_col.min(end_col),
                    bottom_row: start_row.max(end_row),
                    right_col: start_col.max(end_col),
                })
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GridOptimizedR1C1ResolvedRef {
    top_row: u32,
    left_col: u32,
    bottom_row: u32,
    right_col: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GridOptimizedR1C1ReferenceFunction {
    Row,
    Column,
    Rows,
    Columns,
}

impl GridOptimizedR1C1ReferenceFunction {
    const ALL: [Self; 4] = [Self::Row, Self::Column, Self::Rows, Self::Columns];

    const fn name(self) -> &'static str {
        match self {
            Self::Row => "ROW",
            Self::Column => "COLUMN",
            Self::Rows => "ROWS",
            Self::Columns => "COLUMNS",
        }
    }

    const fn allows_current_cell_argument(self) -> bool {
        matches!(self, Self::Row | Self::Column)
    }

    fn apply(self, reference: GridOptimizedR1C1ResolvedRef) -> Option<CalcValue> {
        let value = match self {
            Self::Row => f64::from(reference.top_row),
            Self::Column => f64::from(reference.left_col),
            Self::Rows => f64::from(
                reference
                    .bottom_row
                    .checked_sub(reference.top_row)?
                    .saturating_add(1),
            ),
            Self::Columns => f64::from(
                reference
                    .right_col
                    .checked_sub(reference.left_col)?
                    .saturating_add(1),
            ),
        };
        Some(CalcValue::number(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GridOptimizedR1C1ScalarFunction {
    Abs,
    Sqrt,
    Power,
    Mod,
    Round,
    RoundUp,
    RoundDown,
    Int,
    Trunc,
    Exp,
    Ln,
    Log10,
    Log,
    Sin,
    Cos,
    Tan,
    Radians,
    Degrees,
    Pi,
}

impl GridOptimizedR1C1ScalarFunction {
    const ALL: [Self; 19] = [
        Self::Abs,
        Self::Sqrt,
        Self::Power,
        Self::Mod,
        Self::Round,
        Self::RoundUp,
        Self::RoundDown,
        Self::Int,
        Self::Trunc,
        Self::Exp,
        Self::Ln,
        Self::Log10,
        Self::Log,
        Self::Sin,
        Self::Cos,
        Self::Tan,
        Self::Radians,
        Self::Degrees,
        Self::Pi,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::Abs => "ABS",
            Self::Sqrt => "SQRT",
            Self::Power => "POWER",
            Self::Mod => "MOD",
            Self::Round => "ROUND",
            Self::RoundUp => "ROUNDUP",
            Self::RoundDown => "ROUNDDOWN",
            Self::Int => "INT",
            Self::Trunc => "TRUNC",
            Self::Exp => "EXP",
            Self::Ln => "LN",
            Self::Log10 => "LOG10",
            Self::Log => "LOG",
            Self::Sin => "SIN",
            Self::Cos => "COS",
            Self::Tan => "TAN",
            Self::Radians => "RADIANS",
            Self::Degrees => "DEGREES",
            Self::Pi => "PI",
        }
    }

    const fn arity_holds(self, arity: usize) -> bool {
        match self {
            Self::Abs
            | Self::Sqrt
            | Self::Int
            | Self::Exp
            | Self::Ln
            | Self::Log10
            | Self::Sin
            | Self::Cos
            | Self::Tan
            | Self::Radians
            | Self::Degrees => arity == 1,
            Self::Power | Self::Mod | Self::Round | Self::RoundUp | Self::RoundDown => arity == 2,
            Self::Trunc => arity == 1 || arity == 2,
            Self::Log => arity == 1 || arity == 2,
            Self::Pi => arity == 0,
        }
    }

    fn apply(self, arguments: &[GridOptimizedR1C1Value]) -> CalcValue {
        let number_at = |index: usize| match arguments.get(index).copied() {
            Some(GridOptimizedR1C1Value::Number(number)) => Ok(number),
            Some(GridOptimizedR1C1Value::Error(error)) => Err(CalcValue::error(error)),
            None => Err(CalcValue::error(WorksheetErrorCode::Value)),
        };
        let result = match self {
            Self::Abs => match number_at(0) {
                Ok(number) => number.abs(),
                Err(error) => return error,
            },
            Self::Sqrt => match number_at(0) {
                Ok(number) if number >= 0.0 => number.sqrt(),
                Ok(_) => return CalcValue::error(WorksheetErrorCode::Num),
                Err(error) => return error,
            },
            Self::Power => {
                let base = match number_at(0) {
                    Ok(number) => number,
                    Err(error) => return error,
                };
                let exponent = match number_at(1) {
                    Ok(number) => number,
                    Err(error) => return error,
                };
                base.powf(exponent)
            }
            Self::Mod => {
                let number = match number_at(0) {
                    Ok(number) => number,
                    Err(error) => return error,
                };
                let divisor = match number_at(1) {
                    Ok(number) => number,
                    Err(error) => return error,
                };
                if divisor == 0.0 {
                    return CalcValue::error(WorksheetErrorCode::Div0);
                }
                let result = number - divisor * (number / divisor).floor();
                if result == 0.0 { 0.0 } else { result }
            }
            Self::Round | Self::RoundUp | Self::RoundDown => {
                let number = match number_at(0) {
                    Ok(number) => number,
                    Err(error) => return error,
                };
                let digits = match number_at(1) {
                    Ok(number) => number,
                    Err(error) => return error,
                };
                return self.apply_rounding(number, digits);
            }
            Self::Int => match number_at(0) {
                Ok(number) => {
                    if number.is_finite() {
                        let result = number.floor();
                        if result == 0.0 { 0.0 } else { result }
                    } else {
                        return CalcValue::error(WorksheetErrorCode::Num);
                    }
                }
                Err(error) => return error,
            },
            Self::Trunc => {
                let number = match number_at(0) {
                    Ok(number) => number,
                    Err(error) => return error,
                };
                let digits = match arguments.get(1).copied() {
                    Some(GridOptimizedR1C1Value::Number(number)) => number,
                    Some(GridOptimizedR1C1Value::Error(error)) => {
                        return CalcValue::error(error);
                    }
                    None => 0.0,
                };
                return Self::RoundDown.apply_rounding(number, digits);
            }
            Self::Exp => match number_at(0) {
                Ok(number) if number.is_finite() => number.exp(),
                Ok(_) => return CalcValue::error(WorksheetErrorCode::Num),
                Err(error) => return error,
            },
            Self::Ln => {
                let number = match number_at(0) {
                    Ok(number) => number,
                    Err(error) => return error,
                };
                return Self::apply_logarithm(number, std::f64::consts::E);
            }
            Self::Log10 => {
                let number = match number_at(0) {
                    Ok(number) => number,
                    Err(error) => return error,
                };
                return Self::apply_logarithm(number, 10.0);
            }
            Self::Log => {
                let number = match number_at(0) {
                    Ok(number) => number,
                    Err(error) => return error,
                };
                let base = match arguments.get(1).copied() {
                    Some(GridOptimizedR1C1Value::Number(number)) => number,
                    Some(GridOptimizedR1C1Value::Error(error)) => {
                        return CalcValue::error(error);
                    }
                    None => 10.0,
                };
                return Self::apply_logarithm(number, base);
            }
            Self::Sin => match number_at(0) {
                Ok(number) if number.is_finite() => number.sin(),
                Ok(_) => return CalcValue::error(WorksheetErrorCode::Num),
                Err(error) => return error,
            },
            Self::Cos => match number_at(0) {
                Ok(number) if number.is_finite() => number.cos(),
                Ok(_) => return CalcValue::error(WorksheetErrorCode::Num),
                Err(error) => return error,
            },
            Self::Tan => match number_at(0) {
                Ok(number) if number.is_finite() => number.tan(),
                Ok(_) => return CalcValue::error(WorksheetErrorCode::Num),
                Err(error) => return error,
            },
            Self::Radians => match number_at(0) {
                Ok(number) if number.is_finite() => number * std::f64::consts::PI / 180.0,
                Ok(_) => return CalcValue::error(WorksheetErrorCode::Num),
                Err(error) => return error,
            },
            Self::Degrees => match number_at(0) {
                Ok(number) if number.is_finite() => number * 180.0 / std::f64::consts::PI,
                Ok(_) => return CalcValue::error(WorksheetErrorCode::Num),
                Err(error) => return error,
            },
            Self::Pi => std::f64::consts::PI,
        };
        if result.is_finite() {
            CalcValue::number(result)
        } else {
            CalcValue::error(WorksheetErrorCode::Num)
        }
    }

    fn apply_rounding(self, number: f64, digits: f64) -> CalcValue {
        if !number.is_finite() || !digits.is_finite() {
            return CalcValue::error(WorksheetErrorCode::Num);
        }
        let digits = digits.trunc();
        if !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&digits) {
            return CalcValue::error(WorksheetErrorCode::Num);
        }
        let digits = digits as i32;
        let exponent = digits.unsigned_abs();
        if exponent > 308 {
            return CalcValue::error(WorksheetErrorCode::Num);
        }
        let scale = 10_f64.powi(i32::try_from(exponent).unwrap_or(308));
        if !scale.is_finite() {
            return CalcValue::error(WorksheetErrorCode::Num);
        }
        let scaled = if digits >= 0 {
            number * scale
        } else {
            number / scale
        };
        if !scaled.is_finite() {
            return CalcValue::error(WorksheetErrorCode::Num);
        }
        let rounded = match self {
            Self::Round => scaled.round(),
            Self::RoundUp if scaled.is_sign_negative() => scaled.floor(),
            Self::RoundUp => scaled.ceil(),
            Self::RoundDown => scaled.trunc(),
            Self::Abs
            | Self::Sqrt
            | Self::Power
            | Self::Mod
            | Self::Int
            | Self::Trunc
            | Self::Exp
            | Self::Ln
            | Self::Log10
            | Self::Log
            | Self::Sin
            | Self::Cos
            | Self::Tan
            | Self::Radians
            | Self::Degrees
            | Self::Pi => return CalcValue::error(WorksheetErrorCode::Value),
        };
        let result = if digits >= 0 {
            rounded / scale
        } else {
            rounded * scale
        };
        if !result.is_finite() {
            return CalcValue::error(WorksheetErrorCode::Num);
        }
        if result == 0.0 {
            CalcValue::number(0.0)
        } else {
            CalcValue::number(result)
        }
    }

    fn apply_logarithm(number: f64, base: f64) -> CalcValue {
        if !number.is_finite() || !base.is_finite() || number <= 0.0 || base <= 0.0 {
            return CalcValue::error(WorksheetErrorCode::Num);
        }
        if base == 1.0 {
            return CalcValue::error(WorksheetErrorCode::Div0);
        }
        let result = if base == std::f64::consts::E {
            number.ln()
        } else if base == 10.0 {
            number.log10()
        } else if base == 2.0 {
            number.log2()
        } else {
            number.ln() / base.ln()
        };
        if result.is_finite() {
            CalcValue::number(result)
        } else {
            CalcValue::error(WorksheetErrorCode::Num)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GridOptimizedR1C1AggregateArgument {
    Scalar(GridOptimizedR1C1ScalarExpression),
    Range {
        start: GridOptimizedR1C1Ref,
        end: GridOptimizedR1C1Ref,
    },
}

impl GridOptimizedR1C1AggregateArgument {
    fn accumulate_repeated_region_cell(
        &self,
        function: GridOptimizedR1C1RangeAggregateFunction,
        state: &mut GridOptimizedR1C1AggregateState,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<Result<(), CalcValue>> {
        match self {
            Self::Scalar(expression) => Some(state.accumulate(
                function,
                expression.value_for_repeated_region_cell(
                    row,
                    col,
                    region,
                    row_major_formula_values,
                    valuation,
                )?,
            )),
            Self::Range { start, end } => {
                let (start_row, start_col) = start.resolve(row, col)?;
                let (end_row, end_col) = end.resolve(row, col)?;
                accumulate_optimized_r1c1_rect(
                    function,
                    start_row.min(end_row),
                    start_col.min(end_col),
                    start_row.max(end_row),
                    start_col.max(end_col),
                    Some(region),
                    row_major_formula_values,
                    valuation,
                    state,
                )
            }
        }
    }

    fn accumulate_single_cell(
        &self,
        function: GridOptimizedR1C1RangeAggregateFunction,
        state: &mut GridOptimizedR1C1AggregateState,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<Result<(), CalcValue>> {
        match self {
            Self::Scalar(expression) => Some(state.accumulate(
                function,
                expression.value_for_single_cell(address, valuation)?,
            )),
            Self::Range { start, end } => {
                let (start_row, start_col) = start.resolve(address.row, address.col)?;
                let (end_row, end_col) = end.resolve(address.row, address.col)?;
                accumulate_optimized_r1c1_rect(
                    function,
                    start_row.min(end_row),
                    start_col.min(end_col),
                    start_row.max(end_row),
                    start_col.max(end_col),
                    None,
                    &[],
                    valuation,
                    state,
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1Comparison {
    left: GridOptimizedR1C1ScalarExpression,
    op: GridOptimizedR1C1ComparisonOp,
    right: GridOptimizedR1C1ScalarExpression,
}

impl GridOptimizedR1C1Comparison {
    fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<GridOptimizedR1C1ConditionValue> {
        let left = self.left.value_for_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?;
        let right = self.right.value_for_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?;
        Some(self.op.apply(left, right))
    }

    fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<GridOptimizedR1C1ConditionValue> {
        let left = self.left.value_for_single_cell(address, valuation)?;
        let right = self.right.value_for_single_cell(address, valuation)?;
        Some(self.op.apply(left, right))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridOptimizedR1C1ComparisonOp {
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
    GreaterThanOrEqual,
    GreaterThan,
}

impl GridOptimizedR1C1ComparisonOp {
    fn apply(
        self,
        left: GridOptimizedR1C1Value,
        right: GridOptimizedR1C1Value,
    ) -> GridOptimizedR1C1ConditionValue {
        let left = match left {
            GridOptimizedR1C1Value::Number(number) => number,
            GridOptimizedR1C1Value::Error(error) => {
                return GridOptimizedR1C1ConditionValue::Error(error);
            }
        };
        let right = match right {
            GridOptimizedR1C1Value::Number(number) => number,
            GridOptimizedR1C1Value::Error(error) => {
                return GridOptimizedR1C1ConditionValue::Error(error);
            }
        };
        let result = match self {
            Self::LessThan => left < right,
            Self::LessThanOrEqual => left <= right,
            Self::Equal => left == right,
            Self::NotEqual => left != right,
            Self::GreaterThanOrEqual => left >= right,
            Self::GreaterThan => left > right,
        };
        GridOptimizedR1C1ConditionValue::Logical(result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GridOptimizedR1C1ConditionValue {
    Logical(bool),
    Error(WorksheetErrorCode),
}

impl GridOptimizedR1C1ConditionValue {
    fn into_calc_value(self) -> Option<CalcValue> {
        match self {
            Self::Logical(value) => Some(CalcValue::logical(value)),
            Self::Error(error) => Some(CalcValue::error(error)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GridOptimizedR1C1RangeAggregateFunction {
    Sum,
    Count,
    Product,
    Average,
    Min,
    Max,
    SumSq,
}

impl GridOptimizedR1C1RangeAggregateFunction {
    const ALL: [Self; 7] = [
        Self::Sum,
        Self::Count,
        Self::Product,
        Self::Average,
        Self::Min,
        Self::Max,
        Self::SumSq,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::Sum => "SUM",
            Self::Count => "COUNT",
            Self::Product => "PRODUCT",
            Self::Average => "AVERAGE",
            Self::Min => "MIN",
            Self::Max => "MAX",
            Self::SumSq => "SUMSQ",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridOptimizedR1C1BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl GridOptimizedR1C1BinaryOp {
    #[must_use]
    fn apply(
        self,
        left: GridOptimizedR1C1Value,
        right: GridOptimizedR1C1Value,
    ) -> Option<CalcValue> {
        let left = match left {
            GridOptimizedR1C1Value::Number(number) => number,
            GridOptimizedR1C1Value::Error(error) => return Some(CalcValue::error(error)),
        };
        let right = match right {
            GridOptimizedR1C1Value::Number(number) => number,
            GridOptimizedR1C1Value::Error(error) => return Some(CalcValue::error(error)),
        };
        match self {
            Self::Add => Some(CalcValue::number(left + right)),
            Self::Subtract => Some(CalcValue::number(left - right)),
            Self::Multiply => Some(CalcValue::number(left * right)),
            Self::Divide if right == 0.0 => Some(CalcValue::error(WorksheetErrorCode::Div0)),
            Self::Divide => Some(CalcValue::number(left / right)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum GridOptimizedR1C1Value {
    Number(f64),
    Error(WorksheetErrorCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridOptimizedR1C1Ref {
    row: GridOptimizedR1C1AxisRef,
    col: GridOptimizedR1C1AxisRef,
}

impl GridOptimizedR1C1Ref {
    fn resolve(self, anchor_row: u32, anchor_col: u32) -> Option<(u32, u32)> {
        Some((self.row.resolve(anchor_row)?, self.col.resolve(anchor_col)?))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GridOptimizedR1C1AxisRef {
    Relative(i32),
    Absolute(u32),
}

impl GridOptimizedR1C1AxisRef {
    fn resolve(self, anchor: u32) -> Option<u32> {
        match self {
            Self::Relative(delta) => add_i32_to_u32(anchor, delta),
            Self::Absolute(value) => Some(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridOptimizedR1C1Operand {
    Ref(GridOptimizedR1C1Ref),
    Number(GridOptimizedR1C1NumberLiteral),
}

impl GridOptimizedR1C1Operand {
    fn value_for_repeated_region_cell(
        self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<GridOptimizedR1C1Value> {
        match self {
            Self::Number(value) => Some(GridOptimizedR1C1Value::Number(value.as_f64())),
            Self::Ref(reference) => {
                let (target_row, target_col) = reference.resolve(row, col)?;
                optimized_r1c1_value_for_cell(
                    target_row,
                    target_col,
                    Some(region),
                    row_major_formula_values,
                    valuation,
                )
            }
        }
    }

    fn value_for_single_cell(
        self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<GridOptimizedR1C1Value> {
        match self {
            Self::Number(value) => Some(GridOptimizedR1C1Value::Number(value.as_f64())),
            Self::Ref(reference) => {
                let (target_row, target_col) = reference.resolve(address.row, address.col)?;
                optimized_r1c1_value_for_cell(target_row, target_col, None, &[], valuation)
            }
        }
    }

    fn calc_value_for_repeated_region_cell(
        self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        optimized_r1c1_calc_value_from_value(self.value_for_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?)
    }

    fn calc_value_for_single_cell(
        self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        optimized_r1c1_calc_value_from_value(self.value_for_single_cell(address, valuation)?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridOptimizedR1C1NumberLiteral {
    bits: u64,
}

impl GridOptimizedR1C1NumberLiteral {
    #[must_use]
    pub fn new(value: f64) -> Option<Self> {
        if !value.is_finite() {
            return None;
        }
        let normalized = if value == 0.0 { 0.0 } else { value };
        Some(Self {
            bits: normalized.to_bits(),
        })
    }

    #[must_use]
    pub const fn as_f64(self) -> f64 {
        f64::from_bits(self.bits)
    }
}

fn r1c1_number_literal_expression(value: f64) -> Option<GridOptimizedR1C1ScalarExpression> {
    Some(GridOptimizedR1C1ScalarExpression::Operand(
        GridOptimizedR1C1Operand::Number(GridOptimizedR1C1NumberLiteral::new(value)?),
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GridOptimizedFormulaPlanFingerprint {
    source_text: String,
    source_channel: FormulaChannelKind,
}

impl GridOptimizedFormulaPlanFingerprint {
    fn from_formula(formula: &GridFormulaCell) -> Self {
        Self {
            source_text: formula.source_text.clone(),
            source_channel: formula.source_channel,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GridOptimizedCompiledFormulaPlanEntry {
    fingerprint: GridOptimizedFormulaPlanFingerprint,
    plan: GridOptimizedCompiledFormulaPlan,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GridOptimizedFormulaPlanCache {
    templates: BTreeSet<String>,
    compiled_plans: BTreeMap<String, GridOptimizedCompiledFormulaPlanEntry>,
}

impl GridOptimizedFormulaPlanCache {
    #[must_use]
    pub fn cached_template_count(&self) -> usize {
        self.templates.len()
    }

    #[must_use]
    pub fn cached_compiled_plan_count(&self) -> usize {
        self.compiled_plans.len()
    }

    #[must_use]
    pub fn contains_template(&self, normal_form_key: &str) -> bool {
        self.templates.contains(normal_form_key)
    }

    #[must_use]
    pub fn contains_compiled_plan(&self, normal_form_key: &str) -> bool {
        self.compiled_plans.contains_key(normal_form_key)
    }

    #[must_use]
    fn compiled_plan_for_formula(
        &self,
        formula: &GridFormulaCell,
    ) -> Option<GridOptimizedCompiledFormulaPlan> {
        let fingerprint = GridOptimizedFormulaPlanFingerprint::from_formula(formula);
        self.compiled_plans
            .get(&formula.normal_form_key)
            .filter(|entry| entry.fingerprint == fingerprint)
            .map(|entry| entry.plan.clone())
            .or_else(|| GridOptimizedCompiledFormulaPlan::compile(formula))
    }

    fn prune_to_templates(&mut self, active_templates: &BTreeSet<String>) {
        self.templates
            .retain(|template| active_templates.contains(template));
        self.compiled_plans
            .retain(|template, _| active_templates.contains(template));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedWarmNoOpReport {
    pub cache_hit: bool,
    pub cached_occupied_cells: u64,
    pub cached_formula_cells: u64,
    pub cells_visited: u64,
    pub formula_evaluations: u64,
}

impl GridOptimizedWarmNoOpReport {
    #[must_use]
    pub const fn p19_warm_noop_holds(&self) -> bool {
        self.cache_hit && self.cells_visited == 0 && self.formula_evaluations == 0
    }
}

const TILE_SNAPSHOT_FRAME_HEADER_BYTES: u64 = 128;
const TILE_SNAPSHOT_CELL_ENTRY_BYTES: u64 = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedTileSnapshotReport {
    pub rect: GridRect,
    pub subscribed_cell_count: u64,
    pub defined_cell_count: usize,
    pub blank_cell_count: u64,
    pub dense_value_cells_visited: u64,
    pub sparse_value_cells_visited: u64,
    pub compact_regions_intersected: usize,
    pub estimated_value_payload_bytes: u64,
    pub estimated_frame_bytes: u64,
    pub full_grid_cell_floor: u64,
    pub full_grid_dense_numeric_bytes_floor: u64,
}

impl GridOptimizedTileSnapshotReport {
    #[must_use]
    pub fn frame_bytes_per_subscribed_cell_micros(&self) -> u64 {
        bytes_per_cell_micros(self.estimated_frame_bytes, self.subscribed_cell_count)
    }

    #[must_use]
    pub fn p15_tile_streaming_holds(&self, max_bytes_per_subscribed_cell: u64) -> bool {
        self.estimated_frame_bytes
            <= self
                .subscribed_cell_count
                .saturating_mul(max_bytes_per_subscribed_cell)
            && self.estimated_frame_bytes < self.full_grid_dense_numeric_bytes_floor
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedVisibleFirstReport {
    pub visible_rect: GridRect,
    pub upstream_rect: GridRect,
    pub visible_cell_count: u64,
    pub visible_upstream_cell_count: u64,
    pub cells_evaluated_before_visible_complete: u64,
    pub formula_evaluations_before_visible_complete: u64,
    pub dense_value_cells_projected: u64,
    pub repeated_formula_cells_projected: u64,
    pub sparse_point_cells_projected: u64,
    pub computed_dense_value_regions: usize,
    pub computed_sparse_cells: usize,
    pub full_recalc_occupied_cell_floor: u64,
    pub full_grid_cell_floor: u64,
}

impl GridOptimizedVisibleFirstReport {
    #[must_use]
    pub fn evaluated_to_full_occupied_ratio_micros(&self) -> u64 {
        bytes_per_cell_micros(
            self.cells_evaluated_before_visible_complete,
            self.full_recalc_occupied_cell_floor,
        )
    }

    #[must_use]
    pub const fn p16_visible_first_holds(&self) -> bool {
        self.cells_evaluated_before_visible_complete <= self.visible_upstream_cell_count
            && self.cells_evaluated_before_visible_complete < self.full_recalc_occupied_cell_floor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridOptimizedWarmNoOpCache {
    token: GridOptimizedWarmNoOpToken,
    valuation: GridOptimizedValuation,
    baseline_report: GridOptimizedRecalcReport,
}

impl GridOptimizedWarmNoOpCache {
    #[must_use]
    pub const fn valuation(&self) -> &GridOptimizedValuation {
        &self.valuation
    }

    #[must_use]
    pub const fn baseline_report(&self) -> &GridOptimizedRecalcReport {
        &self.baseline_report
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedFormulaReferenceEnumerationReport {
    pub formula_address: ExcelGridCellAddress,
    pub reference_source_text: String,
    pub declared_cell_count: usize,
    pub defined_cell_count: usize,
    pub dense_value_cells_visited: u64,
    pub sparse_value_cells_visited: u64,
    pub compact_regions_intersected: usize,
}

impl GridOptimizedFormulaReferenceEnumerationReport {
    #[must_use]
    pub const fn slots_visited(&self) -> u64 {
        self.dense_value_cells_visited
            .saturating_add(self.sparse_value_cells_visited)
    }

    #[must_use]
    pub fn p20_occupied_slots_holds(&self) -> bool {
        self.slots_visited() == u64::try_from(self.defined_cell_count).unwrap_or(u64::MAX)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GridOptimizedPublicationDeltaReport {
    pub same_grid_identity: bool,
    pub previous_sparse_cells: usize,
    pub current_sparse_cells: usize,
    pub previous_dense_region_entries: usize,
    pub current_dense_region_entries: usize,
    pub previous_dense_cells: u64,
    pub current_dense_cells: u64,
    pub previous_spill_fact_entries: usize,
    pub current_spill_fact_entries: usize,
    pub sparse_entries_added: usize,
    pub sparse_entries_changed: usize,
    pub sparse_entries_removed: usize,
    pub sparse_entries_unchanged: usize,
    pub dense_region_entries_added: usize,
    pub dense_region_entries_changed: usize,
    pub dense_region_entries_removed: usize,
    pub dense_region_entries_unchanged: usize,
    pub dense_region_cells_added: u64,
    pub dense_region_cells_changed: u64,
    pub dense_region_cells_removed: u64,
    pub dense_region_cells_unchanged: u64,
    pub spill_fact_entries_added: usize,
    pub spill_fact_entries_changed: usize,
    pub spill_fact_entries_removed: usize,
    pub spill_fact_entries_unchanged: usize,
    pub naive_current_computed_cell_publication_floor: u64,
    pub naive_full_grid_publication_floor: u64,
}

impl GridOptimizedPublicationDeltaReport {
    #[must_use]
    pub fn publication_entries_total(&self) -> usize {
        self.sparse_entries_added
            .saturating_add(self.sparse_entries_changed)
            .saturating_add(self.sparse_entries_removed)
            .saturating_add(self.dense_region_entries_added)
            .saturating_add(self.dense_region_entries_changed)
            .saturating_add(self.dense_region_entries_removed)
            .saturating_add(self.spill_fact_entries_added)
            .saturating_add(self.spill_fact_entries_changed)
            .saturating_add(self.spill_fact_entries_removed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedSpillPublicationCommitReport {
    pub previous_spill_fact_entries: usize,
    pub committed_spill_fact_entries: usize,
    pub previous_spill_fingerprint_entries: usize,
    pub committed_spill_fingerprint_entries: usize,
    pub previous_epoch_anchors: usize,
    pub committed_epoch_anchors: usize,
    pub ledger_update: GridSpillEpochLedgerUpdateReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedRecalcAndCommitReport {
    pub recalc: GridOptimizedRecalcReport,
    pub spill_commit: GridOptimizedSpillPublicationCommitReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedSpillClearReport {
    pub anchor: ExcelGridCellAddress,
    pub had_spill_fact: bool,
    pub old_extent: GridRect,
    pub old_extent_cell_count: u64,
    pub naive_sparse_value_scan_floor: usize,
    pub indexed_candidate_count: usize,
    pub sparse_values_removed: usize,
    pub dense_value_regions_removed: usize,
    pub dense_value_cells_removed: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedSpillBlockageProbeReport {
    pub anchor: ExcelGridCellAddress,
    pub extent: GridRect,
    pub extent_cell_count: u64,
    pub naive_extent_cell_probe_floor: u64,
    pub sparse_point_candidates: usize,
    pub dense_value_region_candidates: usize,
    pub repeated_formula_region_candidates: usize,
    pub merged_region_candidates: usize,
    pub feature_rendered_region_candidates: usize,
    pub blocked_formula_spill_fact_candidates: usize,
    pub unblocked_spill_fact_candidates: usize,
    pub blocked: bool,
}

impl GridOptimizedSpillBlockageProbeReport {
    #[must_use]
    pub fn compact_blocker_probe_count(&self) -> usize {
        self.sparse_point_candidates
            .saturating_add(self.dense_value_region_candidates)
            .saturating_add(self.repeated_formula_region_candidates)
            .saturating_add(self.merged_region_candidates)
            .saturating_add(self.feature_rendered_region_candidates)
            .saturating_add(self.blocked_formula_spill_fact_candidates)
            .saturating_add(self.unblocked_spill_fact_candidates)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GridOptimizedWarmNoOpToken {
    materialization_limit: u64,
    next_revision: u64,
    axis_state: GridAxisState,
    sparse_points: Vec<GridOptimizedSparsePointToken>,
    dense_value_regions: Vec<GridOptimizedDenseValueRegionToken>,
    repeated_formula_regions: Vec<GridOptimizedRepeatedFormulaRegionToken>,
    merged_regions: Vec<GridMergedRegion>,
    feature_rendered_regions: Vec<FeatureRenderedRegion>,
    spill_facts: Vec<(ExcelGridCellAddress, GridSpillFact)>,
    defined_names: Vec<(String, GridRect)>,
    table_overlays: Vec<(String, GridTableOverlay)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GridOptimizedSparsePointToken {
    coord: GridCellCoord,
    revision: u64,
    authored: GridOptimizedAuthoredCellToken,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GridOptimizedAuthoredCellToken {
    Literal,
    Formula {
        source_text: String,
        normal_form_key: String,
        source_channel: FormulaChannelKind,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GridOptimizedDenseValueRegionToken {
    rect: GridRect,
    revision: u64,
    value_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GridOptimizedRepeatedFormulaRegionToken {
    rect: GridRect,
    revision: u64,
    source_text: String,
    normal_form_key: String,
    source_channel: FormulaChannelKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridEngineMode {
    Reference,
    Optimized,
    Both,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridEngineCellReadout {
    pub address: ExcelGridCellAddress,
    pub computed: CalcValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GridEngineRecalcReport {
    Reference(GridCalcRefRecalcReport),
    Optimized(GridOptimizedRecalcReport),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridEngineRunReport {
    pub mode: GridEngineMode,
    pub recalc: GridEngineRecalcReport,
    pub readout: Vec<GridEngineCellReadout>,
    pub warm_noop: Option<GridEngineWarmNoOpReport>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridEngineWarmNoOpReport {
    pub recalc: GridOptimizedWarmNoOpReport,
    pub readout: Vec<GridEngineCellReadout>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridDifferentialMismatch {
    pub address: ExcelGridCellAddress,
    pub reference: CalcValue,
    pub optimized: CalcValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridDifferentialRunReport {
    pub mode: GridEngineMode,
    pub reference: Option<GridEngineRunReport>,
    pub optimized: Option<GridEngineRunReport>,
    pub mismatches: Vec<GridDifferentialMismatch>,
}

fn compare_grid_engine_readouts(
    reference: &[GridEngineCellReadout],
    optimized: &[GridEngineCellReadout],
) -> Vec<GridDifferentialMismatch> {
    reference
        .iter()
        .zip(optimized.iter())
        .filter_map(|(reference, optimized)| {
            (reference.computed != optimized.computed).then(|| GridDifferentialMismatch {
                address: reference.address.clone(),
                reference: reference.computed.clone(),
                optimized: optimized.computed.clone(),
            })
        })
        .collect()
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

#[derive(Debug, Clone, PartialEq)]
pub struct GridOptimizedSheet {
    workbook_id: String,
    sheet_id: String,
    bounds: ExcelGridBounds,
    next_revision: u64,
    axis_state: GridAxisState,
    sparse_points: BTreeMap<GridCellCoord, GridVersionedAuthoredCell>,
    dense_value_regions: Vec<GridDenseValueRegion>,
    repeated_formula_regions: Vec<GridRepeatedFormulaRegion>,
    merged_regions: Vec<GridMergedRegion>,
    feature_rendered_regions: Vec<FeatureRenderedRegion>,
    spill_facts: BTreeMap<ExcelGridCellAddress, GridSpillFact>,
    spill_value_fingerprints: BTreeMap<ExcelGridCellAddress, String>,
    spill_epoch_ledger: GridSpillEpochLedger,
    defined_names: BTreeMap<String, GridRect>,
    table_overlays: BTreeMap<String, GridTableOverlay>,
}

impl GridOptimizedSheet {
    #[must_use]
    pub fn new(
        workbook_id: impl Into<String>,
        sheet_id: impl Into<String>,
        bounds: ExcelGridBounds,
    ) -> Self {
        Self {
            workbook_id: workbook_id.into(),
            sheet_id: sheet_id.into(),
            bounds,
            next_revision: 1,
            axis_state: GridAxisState::default(),
            sparse_points: BTreeMap::new(),
            dense_value_regions: Vec::new(),
            repeated_formula_regions: Vec::new(),
            merged_regions: Vec::new(),
            feature_rendered_regions: Vec::new(),
            spill_facts: BTreeMap::new(),
            spill_value_fingerprints: BTreeMap::new(),
            spill_epoch_ledger: GridSpillEpochLedger::default(),
            defined_names: BTreeMap::new(),
            table_overlays: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn strict_excel(workbook_id: impl Into<String>, sheet_id: impl Into<String>) -> Self {
        Self::new(workbook_id, sheet_id, ExcelGridBounds::strict_excel())
    }

    #[must_use]
    pub fn workbook_id(&self) -> &str {
        &self.workbook_id
    }

    #[must_use]
    pub fn sheet_id(&self) -> &str {
        &self.sheet_id
    }

    #[must_use]
    pub const fn bounds(&self) -> ExcelGridBounds {
        self.bounds
    }

    #[must_use]
    pub fn axis_state(&self) -> &GridAxisState {
        &self.axis_state
    }

    #[must_use]
    pub fn axis_state_mut(&mut self) -> &mut GridAxisState {
        &mut self.axis_state
    }

    #[must_use]
    pub fn host_info_provider(&self, caller_row: u32, caller_col: u32) -> GridHostInfoProvider<'_> {
        GridHostInfoProvider::new(
            self.workbook_id.clone(),
            self.sheet_id.clone(),
            caller_row,
            caller_col,
            self.bounds,
            self.spill_facts.values(),
            &self.axis_state,
        )
    }

    fn empty_valuation_with_committed_spill_state(&self) -> GridOptimizedValuation {
        GridOptimizedValuation::with_spill_state(
            self.workbook_id.clone(),
            self.sheet_id.clone(),
            self.bounds,
            self.spill_facts.clone(),
            self.spill_value_fingerprints.clone(),
            self.spill_epoch_ledger.clone(),
        )
        .with_defined_names(self.defined_names.clone())
        .with_table_overlays(self.table_overlays.clone())
    }

    pub fn set_literal(
        &mut self,
        address: ExcelGridCellAddress,
        value: CalcValue,
    ) -> Result<(), GridRefError> {
        self.check_address(&address)?;
        let revision = self.allocate_revision();
        self.sparse_points.insert(
            GridCellCoord::from_address(&address),
            GridVersionedAuthoredCell {
                revision,
                cell: GridOptimizedAuthoredCell::literal(value),
            },
        );
        Ok(())
    }

    pub fn set_formula(
        &mut self,
        address: ExcelGridCellAddress,
        formula: GridFormulaCell,
    ) -> Result<(), GridRefError> {
        self.check_address(&address)?;
        let revision = self.allocate_revision();
        self.sparse_points.insert(
            GridCellCoord::from_address(&address),
            GridVersionedAuthoredCell {
                revision,
                cell: GridOptimizedAuthoredCell::formula(formula),
            },
        );
        Ok(())
    }

    pub fn put_dense_literal_region(
        &mut self,
        rect: GridRect,
        row_major_values: Vec<CalcValue>,
    ) -> Result<GridRegionMaterializationReport, GridRefError> {
        self.check_rect(&rect)?;
        let cell_count = rect.cell_count();
        if u64::try_from(row_major_values.len()).unwrap_or(u64::MAX) != cell_count {
            return Err(GridRefError::DenseRegionValueCountMismatch {
                cells: cell_count,
                values: row_major_values.len(),
            });
        }
        let revision = self.allocate_revision();
        self.dense_value_regions.push(GridDenseValueRegion {
            rect: rect.clone(),
            storage: GridDenseValueStorage::new_for_rect(
                &rect,
                GridDenseValuePayload::from_calc_values(row_major_values),
            ),
            revision,
        });
        Ok(GridRegionMaterializationReport {
            cells_written: usize::try_from(cell_count).unwrap_or(usize::MAX),
            rect,
        })
    }

    pub fn put_dense_literal_region_with<F>(
        &mut self,
        rect: GridRect,
        mut make_value: F,
    ) -> Result<GridRegionMaterializationReport, GridRefError>
    where
        F: FnMut(&ExcelGridCellAddress) -> CalcValue,
    {
        self.check_rect(&rect)?;
        let mut values =
            Vec::with_capacity(usize::try_from(rect.cell_count()).unwrap_or(usize::MAX));
        for row in rect.top_row..=rect.bottom_row {
            for col in rect.left_col..=rect.right_col {
                values.push(make_value(&ExcelGridCellAddress::new(
                    self.workbook_id.clone(),
                    self.sheet_id.clone(),
                    row,
                    col,
                )));
            }
        }
        self.put_dense_literal_region(rect, values)
    }

    pub fn put_dense_number_region_with<F>(
        &mut self,
        rect: GridRect,
        mut make_number: F,
    ) -> Result<GridRegionMaterializationReport, GridRefError>
    where
        F: FnMut(&ExcelGridCellAddress) -> f64,
    {
        self.check_rect(&rect)?;
        let cell_count = rect.cell_count();
        let mut values = Vec::with_capacity(usize::try_from(cell_count).unwrap_or(usize::MAX));
        for row in rect.top_row..=rect.bottom_row {
            for col in rect.left_col..=rect.right_col {
                values.push(make_number(&ExcelGridCellAddress::new(
                    self.workbook_id.clone(),
                    self.sheet_id.clone(),
                    row,
                    col,
                )));
            }
        }
        let revision = self.allocate_revision();
        self.dense_value_regions.push(GridDenseValueRegion {
            rect: rect.clone(),
            storage: GridDenseValueStorage::new_for_rect(
                &rect,
                GridDenseValuePayload::from_numbers(values),
            ),
            revision,
        });
        Ok(GridRegionMaterializationReport {
            cells_written: usize::try_from(cell_count).unwrap_or(usize::MAX),
            rect,
        })
    }

    pub fn put_repeated_formula_region(
        &mut self,
        rect: GridRect,
        formula: GridFormulaCell,
    ) -> Result<GridRegionMaterializationReport, GridRefError> {
        self.check_rect(&rect)?;
        let revision = self.allocate_revision();
        let cell_count = rect.cell_count();
        self.repeated_formula_regions
            .push(GridRepeatedFormulaRegion {
                rect: rect.clone(),
                formula,
                revision,
            });
        Ok(GridRegionMaterializationReport {
            cells_written: usize::try_from(cell_count).unwrap_or(usize::MAX),
            rect,
        })
    }

    #[must_use]
    pub fn spill_facts(&self) -> &BTreeMap<ExcelGridCellAddress, GridSpillFact> {
        &self.spill_facts
    }

    #[must_use]
    pub fn spill_epoch_ledger(&self) -> &GridSpillEpochLedger {
        &self.spill_epoch_ledger
    }

    #[must_use]
    pub fn spill_epoch_snapshots(&self) -> BTreeMap<ExcelGridCellAddress, GridSpillEpochSnapshot> {
        self.spill_epoch_ledger.snapshots()
    }

    pub fn set_spill_fact(&mut self, fact: GridSpillFact) -> Result<(), GridRefError> {
        self.check_address(&fact.anchor)?;
        self.check_rect(&fact.extent)?;
        self.spill_value_fingerprints.insert(
            fact.anchor.clone(),
            manual_spill_fact_value_fingerprint(&fact),
        );
        self.spill_facts.insert(fact.anchor.clone(), fact);
        self.refresh_spill_epoch_ledger();
        Ok(())
    }

    pub fn refresh_spill_epoch_ledger(&mut self) -> GridSpillEpochLedgerUpdateReport {
        let fingerprints = self.spill_value_fingerprints.clone();
        self.spill_epoch_ledger
            .update_from_spill_facts(&self.spill_facts, |fact| {
                fingerprints
                    .get(&fact.anchor)
                    .cloned()
                    .unwrap_or_else(|| manual_spill_fact_value_fingerprint(fact))
            })
    }

    pub fn commit_spill_publication_from_valuation(
        &mut self,
        valuation: &GridOptimizedValuation,
    ) -> Result<GridOptimizedSpillPublicationCommitReport, GridRefError> {
        if valuation.workbook_id != self.workbook_id
            || valuation.sheet_id != self.sheet_id
            || valuation.bounds != self.bounds
        {
            return Err(GridRefError::ValuationGridIdentityMismatch {
                expected_workbook_id: self.workbook_id.clone(),
                expected_sheet_id: self.sheet_id.clone(),
                expected_bounds: self.bounds,
                actual_workbook_id: valuation.workbook_id.clone(),
                actual_sheet_id: valuation.sheet_id.clone(),
                actual_bounds: valuation.bounds,
            });
        }

        let previous_spill_fact_entries = self.spill_facts.len();
        let previous_spill_fingerprint_entries = self.spill_value_fingerprints.len();
        let previous_epoch_anchors = self.spill_epoch_ledger.entries().len();

        self.spill_facts = valuation.spill_facts.clone();
        self.spill_value_fingerprints = valuation.spill_value_fingerprints.clone();
        let ledger_update = self.refresh_spill_epoch_ledger();

        Ok(GridOptimizedSpillPublicationCommitReport {
            previous_spill_fact_entries,
            committed_spill_fact_entries: self.spill_facts.len(),
            previous_spill_fingerprint_entries,
            committed_spill_fingerprint_entries: self.spill_value_fingerprints.len(),
            previous_epoch_anchors,
            committed_epoch_anchors: self.spill_epoch_ledger.entries().len(),
            ledger_update,
        })
    }

    #[must_use]
    pub fn defined_names(&self) -> &BTreeMap<String, GridRect> {
        &self.defined_names
    }

    pub fn set_defined_name(
        &mut self,
        name: impl AsRef<str>,
        rect: GridRect,
    ) -> Result<(), GridRefError> {
        self.check_rect(&rect)?;
        let name_key = defined_name_key_for_name(name.as_ref(), self.bounds)?;
        self.defined_names.insert(name_key, rect);
        Ok(())
    }

    pub fn rename_defined_name(
        &mut self,
        old_name: impl AsRef<str>,
        new_name: impl AsRef<str>,
    ) -> Result<GridNameLifecycleReport, GridRefError> {
        let old_name = old_name.as_ref();
        let new_name = new_name.as_ref();
        let old_key = defined_name_key_for_name(old_name, self.bounds)?;
        let new_key = defined_name_key_for_name(new_name, self.bounds)?;
        if old_key != new_key && self.defined_names.contains_key(&new_key) {
            return Err(GridRefError::DefinedNameAlreadyExists {
                name: new_name.to_string(),
            });
        }
        let Some(rect) = self.defined_names.remove(&old_key) else {
            return Err(GridRefError::DefinedNameNotFound {
                name: old_name.to_string(),
            });
        };
        self.defined_names.insert(new_key.clone(), rect);
        let stats = transform_sparse_point_formulas_for_defined_name_rename(
            &mut self.sparse_points,
            &self.workbook_id,
            &self.sheet_id,
            &old_key,
            new_name,
            self.bounds,
        )?;
        let repeated_stats = transform_repeated_formula_regions_for_defined_name_rename(
            &mut self.repeated_formula_regions,
            &old_key,
            new_name,
            self.bounds,
        )?;
        Ok(GridNameLifecycleReport {
            operation: GridNameLifecycleOperation::Rename,
            old_name_key: Some(old_key),
            new_name_key: Some(new_key),
            formula_cells_transformed: stats.formula_cells_transformed
                + repeated_stats.formula_cells_transformed,
            formula_reference_transforms: stats.formula_reference_transforms
                + repeated_stats.formula_reference_transforms,
        })
    }

    pub fn delete_defined_name(
        &mut self,
        name: impl AsRef<str>,
    ) -> Result<GridNameLifecycleReport, GridRefError> {
        let name = name.as_ref();
        let name_key = defined_name_key_for_name(name, self.bounds)?;
        if self.defined_names.remove(&name_key).is_none() {
            return Err(GridRefError::DefinedNameNotFound {
                name: name.to_string(),
            });
        }
        let stats = transform_sparse_point_formulas_for_defined_name_delete(
            &mut self.sparse_points,
            &self.workbook_id,
            &self.sheet_id,
            &name_key,
            self.bounds,
        )?;
        let repeated_stats = transform_repeated_formula_regions_for_defined_name_delete(
            &mut self.repeated_formula_regions,
            &name_key,
            self.bounds,
        )?;
        Ok(GridNameLifecycleReport {
            operation: GridNameLifecycleOperation::Delete,
            old_name_key: Some(name_key),
            new_name_key: None,
            formula_cells_transformed: stats.formula_cells_transformed
                + repeated_stats.formula_cells_transformed,
            formula_reference_transforms: stats.formula_reference_transforms
                + repeated_stats.formula_reference_transforms,
        })
    }

    #[must_use]
    pub fn table_overlays(&self) -> &BTreeMap<String, GridTableOverlay> {
        &self.table_overlays
    }

    pub fn set_table_overlay(&mut self, table: GridTableOverlay) -> Result<(), GridRefError> {
        table.check_sheet(&self.workbook_id, &self.sheet_id, self.bounds)?;
        let table_key = table_key_for_name(&table.table_name, self.bounds)?;
        let table_range = table.table_range.clone();
        if let Some(old_table) = self.table_overlays.get(&table_key) {
            remove_table_overlay_feature_regions(
                &mut self.feature_rendered_regions,
                &old_table.table_range,
            );
        }
        self.table_overlays.insert(table_key, table);
        self.add_feature_rendered_region(table_range, "table-overlay", false)?;
        Ok(())
    }

    pub fn resize_table_overlay(
        &mut self,
        table: GridTableOverlay,
    ) -> Result<GridTableLifecycleReport, GridRefError> {
        table.check_sheet(&self.workbook_id, &self.sheet_id, self.bounds)?;
        let table_key = table_key_for_name(&table.table_name, self.bounds)?;
        let Some(old_table) = self.table_overlays.get(&table_key) else {
            return Err(GridRefError::TableOverlayNotFound {
                name: table.table_name,
            });
        };
        let feature_regions_removed = remove_table_overlay_feature_regions(
            &mut self.feature_rendered_regions,
            &old_table.table_range,
        );
        let table_range = table.table_range.clone();
        self.table_overlays.insert(table_key.clone(), table);
        self.add_feature_rendered_region(table_range, "table-overlay", false)?;
        Ok(GridTableLifecycleReport {
            operation: GridTableLifecycleOperation::Resize,
            old_table_key: Some(table_key.clone()),
            new_table_key: Some(table_key),
            feature_regions_removed,
            feature_regions_added: 1,
            formula_cells_transformed: 0,
            formula_reference_transforms: 0,
        })
    }

    pub fn rename_table_overlay(
        &mut self,
        old_name: impl AsRef<str>,
        new_name: impl AsRef<str>,
    ) -> Result<GridTableLifecycleReport, GridRefError> {
        let old_name = old_name.as_ref();
        let new_name = new_name.as_ref();
        let old_key = table_key_for_name(old_name, self.bounds)?;
        let new_key = table_key_for_name(new_name, self.bounds)?;
        if old_key != new_key && self.table_overlays.contains_key(&new_key) {
            return Err(GridRefError::TableOverlayAlreadyExists {
                name: new_name.to_string(),
            });
        }
        let Some(mut table) = self.table_overlays.remove(&old_key) else {
            return Err(GridRefError::TableOverlayNotFound {
                name: old_name.to_string(),
            });
        };
        table.table_name = new_name.to_string();
        self.table_overlays.insert(new_key.clone(), table);
        let stats = transform_sparse_point_formulas_for_table_rename(
            &mut self.sparse_points,
            &self.workbook_id,
            &self.sheet_id,
            &old_key,
            new_name,
            self.bounds,
        )?;
        let repeated_stats = transform_repeated_formula_regions_for_table_rename(
            &mut self.repeated_formula_regions,
            &old_key,
            new_name,
            self.bounds,
        )?;
        Ok(GridTableLifecycleReport {
            operation: GridTableLifecycleOperation::Rename,
            old_table_key: Some(old_key),
            new_table_key: Some(new_key),
            feature_regions_removed: 0,
            feature_regions_added: 0,
            formula_cells_transformed: stats.formula_cells_transformed
                + repeated_stats.formula_cells_transformed,
            formula_reference_transforms: stats.formula_reference_transforms
                + repeated_stats.formula_reference_transforms,
        })
    }

    pub fn delete_table_overlay(
        &mut self,
        table_name: impl AsRef<str>,
    ) -> Result<GridTableLifecycleReport, GridRefError> {
        let table_name = table_name.as_ref();
        let table_key = table_key_for_name(table_name, self.bounds)?;
        let Some(table) = self.table_overlays.remove(&table_key) else {
            return Err(GridRefError::TableOverlayNotFound {
                name: table_name.to_string(),
            });
        };
        let feature_regions_removed = remove_table_overlay_feature_regions(
            &mut self.feature_rendered_regions,
            &table.table_range,
        );
        let stats = transform_sparse_point_formulas_for_table_delete(
            &mut self.sparse_points,
            &self.workbook_id,
            &self.sheet_id,
            &table_key,
            self.bounds,
        )?;
        let repeated_stats = transform_repeated_formula_regions_for_table_delete(
            &mut self.repeated_formula_regions,
            &table_key,
            self.bounds,
        )?;
        Ok(GridTableLifecycleReport {
            operation: GridTableLifecycleOperation::Delete,
            old_table_key: Some(table_key),
            new_table_key: None,
            feature_regions_removed,
            feature_regions_added: 0,
            formula_cells_transformed: stats.formula_cells_transformed
                + repeated_stats.formula_cells_transformed,
            formula_reference_transforms: stats.formula_reference_transforms
                + repeated_stats.formula_reference_transforms,
        })
    }

    #[must_use]
    pub fn merged_regions(&self) -> &[GridMergedRegion] {
        &self.merged_regions
    }

    pub fn add_merged_region(&mut self, rect: GridRect) -> Result<(), GridRefError> {
        self.check_rect(&rect)?;
        self.merged_regions.push(GridMergedRegion { rect });
        Ok(())
    }

    #[must_use]
    pub fn feature_rendered_regions(&self) -> &[FeatureRenderedRegion] {
        &self.feature_rendered_regions
    }

    pub fn add_feature_rendered_region(
        &mut self,
        rect: GridRect,
        feature_kind: impl Into<String>,
        needs_refresh: bool,
    ) -> Result<(), GridRefError> {
        self.check_rect(&rect)?;
        self.feature_rendered_regions.push(FeatureRenderedRegion {
            rect,
            feature_kind: feature_kind.into(),
            needs_refresh,
        });
        Ok(())
    }

    #[must_use]
    pub fn dense_value_regions(&self) -> &[GridDenseValueRegion] {
        &self.dense_value_regions
    }

    #[must_use]
    pub fn repeated_formula_regions(&self) -> &[GridRepeatedFormulaRegion] {
        &self.repeated_formula_regions
    }

    pub fn apply_axis_edit(
        &mut self,
        edit: GridAxisEdit,
    ) -> Result<GridOptimizedStructuralEditReport, GridRefError> {
        validate_axis_edit(edit, self.bounds)?;
        let feature_region_transform = transform_feature_rendered_regions_for_axis_edit(
            &self.feature_rendered_regions,
            edit,
            self.bounds,
        )?;

        let dense_value_regions_before = self.dense_value_regions.len();
        let dense_value_cells_before = self
            .dense_value_regions
            .iter()
            .map(|region| region.rect.cell_count())
            .sum::<u64>();
        let repeated_formula_regions_before = self.repeated_formula_regions.len();
        let repeated_formula_cells_before = self
            .repeated_formula_regions
            .iter()
            .map(|region| region.rect.cell_count())
            .sum::<u64>();

        let (
            sparse_points,
            sparse_points_kept,
            sparse_points_dropped,
            sparse_formula_cells_transformed,
            sparse_formula_reference_transforms,
        ) = transform_optimized_sparse_points_for_edit(
            std::mem::take(&mut self.sparse_points),
            &self.workbook_id,
            &self.sheet_id,
            edit,
            self.bounds,
        )?;
        self.sparse_points = sparse_points;
        self.axis_state.apply_axis_edit(edit, self.bounds)?;

        let mut dense_value_regions_after = Vec::new();
        let mut dense_value_regions_dropped = 0;
        for region in std::mem::take(&mut self.dense_value_regions) {
            let transformed = transform_dense_value_region_for_edit(&region, edit, self.bounds)?;
            if transformed.is_empty() {
                dense_value_regions_dropped += 1;
            }
            dense_value_regions_after.extend(transformed);
        }
        self.dense_value_regions = dense_value_regions_after;

        let mut repeated_formula_regions_after = Vec::new();
        let mut repeated_formula_regions_dropped = 0;
        let mut repeated_formula_segments_transformed = 0;
        let mut repeated_formula_reference_transforms = 0;
        for region in std::mem::take(&mut self.repeated_formula_regions) {
            let output = transform_repeated_formula_region_for_edit(&region, edit, self.bounds)?;
            if output.regions.is_empty() {
                repeated_formula_regions_dropped += 1;
            }
            repeated_formula_segments_transformed += output.formula_segments_transformed;
            repeated_formula_reference_transforms += output.formula_reference_transforms;
            repeated_formula_regions_after.extend(output.regions);
        }
        self.repeated_formula_regions = repeated_formula_regions_after;

        let old_spill_facts = std::mem::take(&mut self.spill_facts);
        let mut spill_facts_kept = 0;
        let mut spill_facts_dropped = 0;
        for fact in old_spill_facts.into_values() {
            let Some(anchor) = transform_address_for_edit(&fact.anchor, edit, self.bounds)? else {
                spill_facts_dropped += 1;
                continue;
            };
            let (Some(extent), _) = transform_rect_for_edit(&fact.extent, edit, self.bounds)?
            else {
                spill_facts_dropped += 1;
                continue;
            };
            let transformed = GridSpillFact {
                anchor: anchor.clone(),
                extent,
                blocked: fact.blocked,
            };
            self.spill_facts.insert(anchor, transformed);
            spill_facts_kept += 1;
        }
        self.spill_value_fingerprints = transform_spill_value_fingerprints_for_edit(
            std::mem::take(&mut self.spill_value_fingerprints),
            edit,
            self.bounds,
        )?;
        self.refresh_spill_epoch_ledger();

        let old_defined_names = std::mem::take(&mut self.defined_names);
        for (name_key, rect) in old_defined_names {
            let (Some(rect), _) = transform_rect_for_edit(&rect, edit, self.bounds)? else {
                continue;
            };
            self.defined_names.insert(name_key, rect);
        }

        let old_table_overlays = std::mem::take(&mut self.table_overlays);
        for (table_key, table) in old_table_overlays {
            let Some(table) = table.transform_for_axis_edit(edit, self.bounds)? else {
                continue;
            };
            self.table_overlays.insert(table_key, table);
        }

        let old_merged_regions = std::mem::take(&mut self.merged_regions);
        let mut merged_regions_kept = 0;
        let mut merged_regions_dropped = 0;
        for region in old_merged_regions {
            let (Some(rect), _) = transform_rect_for_edit(&region.rect, edit, self.bounds)? else {
                merged_regions_dropped += 1;
                continue;
            };
            self.merged_regions.push(GridMergedRegion { rect });
            merged_regions_kept += 1;
        }

        self.feature_rendered_regions = feature_region_transform.regions;

        let dense_value_cells_after = self
            .dense_value_regions
            .iter()
            .map(|region| region.rect.cell_count())
            .sum::<u64>();
        let repeated_formula_cells_after = self
            .repeated_formula_regions
            .iter()
            .map(|region| region.rect.cell_count())
            .sum::<u64>();

        Ok(GridOptimizedStructuralEditReport {
            edit,
            sparse_points_kept,
            sparse_points_dropped,
            sparse_formula_cells_transformed,
            sparse_formula_reference_transforms,
            dense_value_regions_before,
            dense_value_regions_after: self.dense_value_regions.len(),
            dense_value_regions_dropped,
            dense_value_cells_before,
            dense_value_cells_after,
            repeated_formula_regions_before,
            repeated_formula_regions_after: self.repeated_formula_regions.len(),
            repeated_formula_regions_dropped,
            repeated_formula_cells_before,
            repeated_formula_cells_after,
            repeated_formula_segments_transformed,
            repeated_formula_reference_transforms,
            spill_facts_kept,
            spill_facts_dropped,
            merged_regions_kept,
            merged_regions_dropped,
            feature_regions_kept: feature_region_transform.kept,
            feature_regions_dropped: feature_region_transform.dropped,
            feature_regions_marked_needs_refresh: feature_region_transform.marked_needs_refresh,
        })
    }

    #[must_use]
    pub fn sparse_point_cells(&self) -> usize {
        self.sparse_points.len()
    }

    #[must_use]
    pub fn storage_stats(&self) -> GridOptimizedStorageStats {
        let dense_value_cells = self
            .dense_value_regions
            .iter()
            .map(|region| region.rect.cell_count())
            .sum::<u64>();
        let repeated_formula_cells = self
            .repeated_formula_regions
            .iter()
            .map(|region| region.rect.cell_count())
            .sum::<u64>();
        let distinct_repeated_formula_templates = self
            .repeated_formula_regions
            .iter()
            .map(|region| region.formula.normal_form_key.clone())
            .collect::<BTreeSet<_>>()
            .len();
        GridOptimizedStorageStats {
            sparse_point_cells: self.sparse_points.len(),
            dense_value_regions: self.dense_value_regions.len(),
            dense_value_cells,
            repeated_formula_regions: self.repeated_formula_regions.len(),
            repeated_formula_cells,
            distinct_repeated_formula_templates,
            spill_facts: self.spill_facts.len(),
            authored_cells_upper_bound: u64::try_from(self.sparse_points.len())
                .unwrap_or(u64::MAX)
                .saturating_add(dense_value_cells)
                .saturating_add(repeated_formula_cells),
        }
    }

    #[must_use]
    pub fn partition_witness_report(&self) -> GridOptimizedPartitionWitnessReport {
        let (dense_value_pair_checks, dense_value_overlap_count) = pairwise_rect_partition_report(
            self.dense_value_regions.iter().map(|region| &region.rect),
        );
        let (repeated_formula_pair_checks, repeated_formula_overlap_count) =
            pairwise_rect_partition_report(
                self.repeated_formula_regions
                    .iter()
                    .map(|region| &region.rect),
            );
        GridOptimizedPartitionWitnessReport {
            sparse_point_cells: self.sparse_points.len(),
            dense_value_regions: self.dense_value_regions.len(),
            repeated_formula_regions: self.repeated_formula_regions.len(),
            dense_value_pair_checks,
            repeated_formula_pair_checks,
            dense_value_overlap_count,
            repeated_formula_overlap_count,
            max_parallelism_bound: u64::try_from(self.sparse_points.len())
                .unwrap_or(u64::MAX)
                .saturating_add(u64::try_from(self.dense_value_regions.len()).unwrap_or(u64::MAX))
                .saturating_add(
                    u64::try_from(self.repeated_formula_regions.len()).unwrap_or(u64::MAX),
                ),
        }
    }

    #[must_use]
    pub fn storage_byte_report(&self) -> GridOptimizedStorageByteReport {
        let stats = self.storage_stats();
        let sparse_point_bytes = self
            .sparse_points
            .iter()
            .map(|(coord, point)| {
                estimated_grid_cell_coord_bytes(*coord)
                    .saturating_add(estimated_versioned_authored_cell_bytes(point))
            })
            .fold(0_u64, u64::saturating_add);
        let dense_region_metadata_bytes = self
            .dense_value_regions
            .iter()
            .map(GridDenseValueRegion::estimated_authored_bytes)
            .fold(0_u64, u64::saturating_add);
        let mut dense_payload_ids = BTreeSet::new();
        let dense_payload_bytes = self
            .dense_value_regions
            .iter()
            .filter_map(|region| {
                dense_payload_ids
                    .insert(region.storage.shared_payload_id())
                    .then(|| region.storage.shared_payload_bytes())
            })
            .fold(0_u64, u64::saturating_add);
        let dense_value_region_bytes =
            dense_region_metadata_bytes.saturating_add(dense_payload_bytes);
        let dense_numeric_packed_cells = self
            .dense_value_regions
            .iter()
            .map(|region| region.storage.packed_numeric_cells(&region.rect))
            .fold(0_u64, u64::saturating_add);
        let repeated_formula_region_bytes = self
            .repeated_formula_regions
            .iter()
            .map(estimated_repeated_formula_region_bytes)
            .fold(0_u64, u64::saturating_add);
        let metadata_bytes = u64::try_from(std::mem::size_of::<Self>())
            .unwrap_or(u64::MAX)
            .saturating_add(u64::try_from(self.workbook_id.len()).unwrap_or(u64::MAX))
            .saturating_add(u64::try_from(self.sheet_id.len()).unwrap_or(u64::MAX))
            .saturating_add(
                u64::try_from(std::mem::size_of_val(&self.sparse_points)).unwrap_or(u64::MAX),
            )
            .saturating_add(
                u64::try_from(std::mem::size_of_val(&self.dense_value_regions)).unwrap_or(u64::MAX),
            )
            .saturating_add(
                u64::try_from(std::mem::size_of_val(&self.repeated_formula_regions))
                    .unwrap_or(u64::MAX),
            );
        let grid_cell_capacity =
            u64::from(self.bounds.max_rows).saturating_mul(u64::from(self.bounds.max_cols));
        let blank_cells = grid_cell_capacity.saturating_sub(stats.authored_cells_upper_bound);
        let authored_storage_bytes = metadata_bytes
            .saturating_add(sparse_point_bytes)
            .saturating_add(dense_value_region_bytes)
            .saturating_add(repeated_formula_region_bytes);

        GridOptimizedStorageByteReport {
            accounting_model: "oxcalc.grid.optimized.authored_storage_bytes.v1",
            authored_storage_bytes,
            sparse_point_bytes,
            dense_value_region_bytes,
            repeated_formula_region_bytes,
            metadata_bytes,
            authored_cells_upper_bound: stats.authored_cells_upper_bound,
            dense_value_cells: stats.dense_value_cells,
            dense_numeric_packed_cells,
            repeated_formula_cells: stats.repeated_formula_cells,
            sparse_point_cells: u64::try_from(stats.sparse_point_cells).unwrap_or(u64::MAX),
            grid_cell_capacity,
            blank_cells,
            blank_cell_bytes: 0,
        }
    }

    #[must_use]
    pub fn cow_retention_report(retained_roots: &[Self]) -> GridOptimizedCowRetentionReport {
        let mut unique_dense_payloads = BTreeMap::<usize, u64>::new();
        let mut dense_region_metadata_bytes = 0_u64;
        let mut repeated_formula_region_bytes = 0_u64;
        let mut sparse_point_bytes = 0_u64;
        let mut sheet_root_metadata_bytes = 0_u64;
        let mut retained_compact_regions = 0_usize;
        let mut naive_full_snapshot_retention_bytes_floor = 0_u64;

        for root in retained_roots {
            let byte_report = root.storage_byte_report();
            naive_full_snapshot_retention_bytes_floor = naive_full_snapshot_retention_bytes_floor
                .saturating_add(byte_report.authored_storage_bytes);
            sheet_root_metadata_bytes =
                sheet_root_metadata_bytes.saturating_add(byte_report.metadata_bytes);
            sparse_point_bytes = sparse_point_bytes.saturating_add(byte_report.sparse_point_bytes);
            retained_compact_regions = retained_compact_regions
                .saturating_add(root.dense_value_regions.len())
                .saturating_add(root.repeated_formula_regions.len())
                .saturating_add(root.sparse_points.len());

            for region in &root.dense_value_regions {
                dense_region_metadata_bytes =
                    dense_region_metadata_bytes.saturating_add(region.estimated_authored_bytes());
                unique_dense_payloads
                    .entry(region.storage.shared_payload_id())
                    .or_insert_with(|| region.storage.shared_payload_bytes());
            }
            for region in &root.repeated_formula_regions {
                repeated_formula_region_bytes = repeated_formula_region_bytes
                    .saturating_add(estimated_repeated_formula_region_bytes(region));
            }
        }

        let unique_dense_payload_bytes = unique_dense_payloads
            .values()
            .copied()
            .fold(0_u64, u64::saturating_add);
        let cow_retained_bytes = sheet_root_metadata_bytes
            .saturating_add(sparse_point_bytes)
            .saturating_add(dense_region_metadata_bytes)
            .saturating_add(unique_dense_payload_bytes)
            .saturating_add(repeated_formula_region_bytes);
        GridOptimizedCowRetentionReport {
            retained_revision_count: retained_roots.len(),
            unique_dense_payloads: unique_dense_payloads.len(),
            unique_dense_payload_bytes,
            dense_region_metadata_bytes,
            repeated_formula_region_bytes,
            sparse_point_bytes,
            sheet_root_metadata_bytes,
            retained_compact_regions,
            cow_retained_bytes,
            naive_full_snapshot_retention_bytes_floor,
            retained_to_naive_ratio_micros: bytes_per_cell_micros(
                cow_retained_bytes,
                naive_full_snapshot_retention_bytes_floor,
            ),
        }
    }

    #[must_use]
    pub fn authored_cell_at(
        &self,
        address: &ExcelGridCellAddress,
    ) -> Option<GridOptimizedCellReadout> {
        if self.check_address(address).is_err() {
            return None;
        }
        let mut best_revision = 0;
        let mut best_cell = None;
        let mut best_source = None;

        let coord = GridCellCoord::from_address(address);
        if let Some(point) = self.sparse_points.get(&coord) {
            best_revision = point.revision;
            best_cell = Some(point.cell.to_authored());
            best_source = Some(GridOptimizedCellSource::SparsePoint);
        }

        for (region_index, region) in self.dense_value_regions.iter().enumerate() {
            if region.revision <= best_revision {
                continue;
            }
            let Some(value) = region.value_at(address) else {
                continue;
            };
            best_revision = region.revision;
            best_cell = Some(GridAuthoredCell::Literal(value));
            best_source = Some(GridOptimizedCellSource::DenseValueRegion { region_index });
        }

        for (region_index, region) in self.repeated_formula_regions.iter().enumerate() {
            if region.revision <= best_revision || !region.rect.contains(address) {
                continue;
            }
            best_revision = region.revision;
            best_cell = Some(GridAuthoredCell::Formula(region.formula.clone()));
            best_source = Some(GridOptimizedCellSource::RepeatedFormulaRegion { region_index });
        }

        Some(GridOptimizedCellReadout {
            address: address.clone(),
            authored: best_cell,
            source: best_source,
        })
    }

    #[must_use]
    pub fn sampled_authored_readout(
        &self,
        addresses: impl IntoIterator<Item = ExcelGridCellAddress>,
    ) -> Vec<GridOptimizedCellReadout> {
        addresses
            .into_iter()
            .filter_map(|address| self.authored_cell_at(&address))
            .collect()
    }

    pub fn optimized_formula_reference_enumeration_reports(
        &self,
        formula_address: &ExcelGridCellAddress,
        materialization_limit: u64,
    ) -> Result<Vec<GridOptimizedFormulaReferenceEnumerationReport>, GridRefError> {
        let Some(readout) = self.authored_cell_at(formula_address) else {
            return Err(GridRefError::FormulaReferenceEnumerationFailed {
                address: formula_address.clone(),
                detail: "formula address is outside the optimized grid".to_string(),
            });
        };
        let Some(GridAuthoredCell::Formula(formula)) = readout.authored else {
            return Err(GridRefError::FormulaReferenceEnumerationFailed {
                address: formula_address.clone(),
                detail: "formula address does not contain an authored formula".to_string(),
            });
        };

        let (valuation, _, _) =
            self.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
        let provider =
            valuation.reference_system_provider(formula_address.row, formula_address.col);
        let profile = StrictExcelGridReferenceProfile::with_bounds(self.bounds);
        let bound =
            bind_grid_formula_for_transform(&formula, formula_address, &profile, self.bounds);
        let mut reports = Vec::new();
        for normalized in &bound.normalized_references {
            let NormalizedReference::ProfileSymbolic(record) = normalized else {
                continue;
            };
            if record.profile_id != EXCEL_GRID_PROFILE_ID {
                continue;
            }
            let Some(reference) = excel_grid_reference_like_from_profile_record(record) else {
                continue;
            };
            let Some(measured) = provider
                .enumerate_values_with_report(&ReferenceEnumerationRequest { reference })
                .map_err(|error| GridRefError::FormulaReferenceEnumerationFailed {
                    address: formula_address.clone(),
                    detail: format!("{error:?}"),
                })?
            else {
                continue;
            };
            reports.push(GridOptimizedFormulaReferenceEnumerationReport {
                formula_address: formula_address.clone(),
                reference_source_text: record.source_info.source_text.clone(),
                declared_cell_count: measured.report.declared_cell_count,
                defined_cell_count: measured.report.defined_cell_count,
                dense_value_cells_visited: measured.report.dense_value_cells_visited,
                sparse_value_cells_visited: measured.report.sparse_value_cells_visited,
                compact_regions_intersected: measured.report.compact_regions_intersected,
            });
        }
        Ok(reports)
    }

    pub fn run_engine_mode_with_oxfml(
        &self,
        mode: GridEngineMode,
        probes: impl IntoIterator<Item = ExcelGridCellAddress>,
        materialization_limit: u64,
    ) -> Result<GridDifferentialRunReport, GridRefError> {
        let probes = probes.into_iter().collect::<Vec<_>>();
        match mode {
            GridEngineMode::Reference => {
                let reference =
                    self.run_reference_engine_with_oxfml(&probes, materialization_limit)?;
                Ok(GridDifferentialRunReport {
                    mode,
                    reference: Some(reference),
                    optimized: None,
                    mismatches: Vec::new(),
                })
            }
            GridEngineMode::Optimized => {
                let optimized =
                    self.run_optimized_engine_with_oxfml(&probes, materialization_limit)?;
                Ok(GridDifferentialRunReport {
                    mode,
                    reference: None,
                    optimized: Some(optimized),
                    mismatches: Vec::new(),
                })
            }
            GridEngineMode::Both => {
                let reference =
                    self.run_reference_engine_with_oxfml(&probes, materialization_limit)?;
                let optimized =
                    self.run_optimized_engine_with_oxfml(&probes, materialization_limit)?;
                let mismatches =
                    compare_grid_engine_readouts(&reference.readout, &optimized.readout);
                Ok(GridDifferentialRunReport {
                    mode,
                    reference: Some(reference),
                    optimized: Some(optimized),
                    mismatches,
                })
            }
        }
    }

    fn run_reference_engine_with_oxfml(
        &self,
        probes: &[ExcelGridCellAddress],
        materialization_limit: u64,
    ) -> Result<GridEngineRunReport, GridRefError> {
        let mut reference = self.project_authored_to_reference(materialization_limit)?;
        let report = reference.recalculate_mark_all_dirty_with_oxfml()?;
        let readout = probes
            .iter()
            .map(|address| GridEngineCellReadout {
                address: address.clone(),
                computed: reference.read_cell(address),
            })
            .collect();
        Ok(GridEngineRunReport {
            mode: GridEngineMode::Reference,
            recalc: GridEngineRecalcReport::Reference(report),
            readout,
            warm_noop: None,
        })
    }

    fn run_optimized_engine_with_oxfml(
        &self,
        probes: &[ExcelGridCellAddress],
        materialization_limit: u64,
    ) -> Result<GridEngineRunReport, GridRefError> {
        let (valuation, report, cache) =
            self.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
        let readout = probes
            .iter()
            .map(|address| GridEngineCellReadout {
                address: address.clone(),
                computed: valuation.read_cell(address).computed,
            })
            .collect();
        let (warm_valuation, warm_report) = self
            .recalculate_warm_noop_compact_with_oxfml(&cache)
            .ok_or(GridRefError::OptimizedWarmNoOpCacheStale)?;
        let warm_readout = probes
            .iter()
            .map(|address| GridEngineCellReadout {
                address: address.clone(),
                computed: warm_valuation.read_cell(address).computed,
            })
            .collect();
        Ok(GridEngineRunReport {
            mode: GridEngineMode::Optimized,
            recalc: GridEngineRecalcReport::Optimized(report),
            readout,
            warm_noop: Some(GridEngineWarmNoOpReport {
                recalc: warm_report,
                readout: warm_readout,
            }),
        })
    }

    pub fn recalculate_mark_all_dirty_compact<F>(
        &self,
        materialization_limit: u64,
        mut evaluate_formula: F,
    ) -> Result<(GridOptimizedValuation, GridOptimizedRecalcReport), GridRefError>
    where
        F: FnMut(GridOptimizedFormulaEvaluationRequest<'_>) -> CalcValue,
    {
        let mut valuation = self.empty_valuation_with_committed_spill_state();
        let mut report = GridOptimizedRecalcReport {
            occupied_cells: 0,
            literal_cells: 0,
            formula_cells: 0,
            cells_evaluated: 0,
            formula_evaluations: 0,
            spill_repair_passes: 0,
            spill_repair_formula_evaluations: 0,
            spill_repair_converged: true,
            sparse_literal_cells: 0,
            sparse_formula_cells: 0,
            dense_value_region_cells: 0,
            repeated_formula_region_cells: 0,
            formula_templates_prepared: 0,
            distinct_formula_templates: 0,
            formula_plan_cache_hits: 0,
            formula_plan_cache_misses: 0,
            compiled_formula_plan_cache_hits: 0,
            compiled_formula_plan_cache_misses: 0,
            compiled_formula_plans_cached: 0,
            computed_dense_value_regions: 0,
            computed_sparse_cells: 0,
            spill_facts_published: 0,
            spill_facts_blocked: 0,
            spill_ghost_cells_published: 0,
        };
        let mut prepared_templates = BTreeSet::new();
        let mut formula_plan_cache = GridOptimizedFormulaPlanCache::default();

        for (coord, point) in &self.sparse_points {
            let address = self.address_from_coord(*coord);
            if !self.final_source_matches(&address, GridOptimizedCellSource::SparsePoint) {
                continue;
            }
            report.occupied_cells += 1;
            report.cells_evaluated += 1;
            if let Some(value) = point.cell.literal_value() {
                report.literal_cells += 1;
                report.sparse_literal_cells += 1;
                valuation.insert_sparse_value(
                    address.clone(),
                    point.revision,
                    value,
                    GridOptimizedCellSource::SparsePoint,
                );
            } else if let Some(formula) = point.cell.formula_ref() {
                report.formula_cells += 1;
                report.sparse_formula_cells += 1;
                report.formula_evaluations += 1;
                register_formula_plan_cache_access(
                    &mut prepared_templates,
                    &mut formula_plan_cache,
                    formula,
                    None,
                    &mut report,
                    1,
                );
                let value = evaluate_formula(GridOptimizedFormulaEvaluationRequest {
                    address: &address,
                    formula,
                    source: GridOptimizedCellSource::SparsePoint,
                });
                let spill_counters = self.publish_formula_value_to_valuation(
                    &mut valuation,
                    address.clone(),
                    point.revision,
                    value,
                    GridOptimizedCellSource::SparsePoint,
                );
                report.spill_facts_published += spill_counters.facts_published;
                report.spill_facts_blocked += spill_counters.facts_blocked;
                report.spill_ghost_cells_published += spill_counters.ghost_cells_published;
            }
        }

        for (region_index, region) in self.dense_value_regions.iter().enumerate() {
            let source = GridOptimizedCellSource::DenseValueRegion { region_index };
            let mut final_cells = 0;
            for address in region.rect.scalar_cells(materialization_limit)? {
                if self.final_source_matches(&address, source) {
                    final_cells += 1;
                }
            }
            if final_cells == 0 {
                continue;
            }
            report.occupied_cells += final_cells;
            report.cells_evaluated += final_cells;
            report.literal_cells += final_cells;
            report.dense_value_region_cells += final_cells;
            valuation.push_dense_value_payload(
                region.rect.clone(),
                GridDenseValuePayload::from_calc_values(region.row_major_values()),
                region.revision,
                source,
            );
        }

        for (region_index, region) in self.repeated_formula_regions.iter().enumerate() {
            let source = GridOptimizedCellSource::RepeatedFormulaRegion { region_index };
            for address in region.rect.scalar_cells(materialization_limit)? {
                if !self.final_source_matches(&address, source) {
                    continue;
                }
                report.occupied_cells += 1;
                report.cells_evaluated += 1;
                report.formula_cells += 1;
                report.repeated_formula_region_cells += 1;
                report.formula_evaluations += 1;
                register_formula_plan_cache_access(
                    &mut prepared_templates,
                    &mut formula_plan_cache,
                    &region.formula,
                    None,
                    &mut report,
                    1,
                );
                let value = evaluate_formula(GridOptimizedFormulaEvaluationRequest {
                    address: &address,
                    formula: &region.formula,
                    source,
                });
                let spill_counters = self.publish_formula_value_to_valuation(
                    &mut valuation,
                    address,
                    region.revision,
                    value,
                    source,
                );
                report.spill_facts_published += spill_counters.facts_published;
                report.spill_facts_blocked += spill_counters.facts_blocked;
                report.spill_ghost_cells_published += spill_counters.ghost_cells_published;
            }
        }

        report.formula_templates_prepared = prepared_templates.len();
        report.distinct_formula_templates = prepared_templates.len();
        formula_plan_cache.prune_to_templates(&prepared_templates);
        report.compiled_formula_plans_cached = formula_plan_cache.cached_compiled_plan_count();
        report.computed_dense_value_regions = valuation.dense_value_regions().len();
        report.computed_sparse_cells = valuation.sparse_computed_cells();
        valuation.refresh_spill_epoch_ledger();
        self.refresh_optimized_report_spill_counters(&valuation, &mut report);
        Ok((valuation, report))
    }

    pub fn recalculate_mark_all_dirty_compact_with_oxfml(
        &self,
        materialization_limit: u64,
    ) -> Result<(GridOptimizedValuation, GridOptimizedRecalcReport), GridRefError> {
        let mut formula_plan_cache = GridOptimizedFormulaPlanCache::default();
        self.recalculate_mark_all_dirty_compact_with_oxfml_using_formula_plan_cache(
            materialization_limit,
            &mut formula_plan_cache,
        )
    }

    pub fn recalculate_mark_all_dirty_compact_with_oxfml_and_commit_spill_publication(
        &mut self,
        materialization_limit: u64,
    ) -> Result<(GridOptimizedValuation, GridOptimizedRecalcAndCommitReport), GridRefError> {
        let (valuation, recalc) =
            self.recalculate_mark_all_dirty_compact_with_oxfml(materialization_limit)?;
        let spill_commit = self.commit_spill_publication_from_valuation(&valuation)?;
        Ok((
            valuation,
            GridOptimizedRecalcAndCommitReport {
                recalc,
                spill_commit,
            },
        ))
    }

    pub fn recalculate_mark_all_dirty_compact_with_oxfml_using_formula_plan_cache(
        &self,
        materialization_limit: u64,
        formula_plan_cache: &mut GridOptimizedFormulaPlanCache,
    ) -> Result<(GridOptimizedValuation, GridOptimizedRecalcReport), GridRefError> {
        let mut valuation = self.empty_valuation_with_committed_spill_state();
        let mut report = GridOptimizedRecalcReport {
            occupied_cells: 0,
            literal_cells: 0,
            formula_cells: 0,
            cells_evaluated: 0,
            formula_evaluations: 0,
            spill_repair_passes: 0,
            spill_repair_formula_evaluations: 0,
            spill_repair_converged: true,
            sparse_literal_cells: 0,
            sparse_formula_cells: 0,
            dense_value_region_cells: 0,
            repeated_formula_region_cells: 0,
            formula_templates_prepared: 0,
            distinct_formula_templates: 0,
            formula_plan_cache_hits: 0,
            formula_plan_cache_misses: 0,
            compiled_formula_plan_cache_hits: 0,
            compiled_formula_plan_cache_misses: 0,
            compiled_formula_plans_cached: 0,
            computed_dense_value_regions: 0,
            computed_sparse_cells: 0,
            spill_facts_published: 0,
            spill_facts_blocked: 0,
            spill_ghost_cells_published: 0,
        };
        let mut prepared_templates = BTreeSet::new();
        self.populate_compact_literal_valuation(
            &mut valuation,
            &mut report,
            materialization_limit,
        )?;

        for (coord, point) in &self.sparse_points {
            let address = self.address_from_coord(*coord);
            if !self.final_source_matches(&address, GridOptimizedCellSource::SparsePoint) {
                continue;
            }
            let Some(formula) = point.cell.formula_ref() else {
                continue;
            };
            report.occupied_cells += 1;
            report.cells_evaluated += 1;
            report.formula_cells += 1;
            report.sparse_formula_cells += 1;
            report.formula_evaluations += 1;
            register_formula_plan_cache_access(
                &mut prepared_templates,
                formula_plan_cache,
                formula,
                None,
                &mut report,
                1,
            );
            let value = self.evaluate_optimized_formula_with_spill_repair(
                &address,
                formula,
                &valuation,
                materialization_limit,
            )?;
            let spill_counters = self.publish_formula_value_to_valuation(
                &mut valuation,
                address.clone(),
                point.revision,
                value,
                GridOptimizedCellSource::SparsePoint,
            );
            report.spill_facts_published += spill_counters.facts_published;
            report.spill_facts_blocked += spill_counters.facts_blocked;
            report.spill_ghost_cells_published += spill_counters.ghost_cells_published;
        }

        for (region_index, region) in self.repeated_formula_regions.iter().enumerate() {
            let source = GridOptimizedCellSource::RepeatedFormulaRegion { region_index };
            if self.try_evaluate_repeated_formula_region_fast_path(
                region_index,
                region,
                source,
                &mut valuation,
                &mut report,
                &mut prepared_templates,
                formula_plan_cache,
                materialization_limit,
            )? {
                continue;
            }
            for address in region.rect.scalar_cells(materialization_limit)? {
                if !self.final_source_matches(&address, source) {
                    continue;
                }
                report.occupied_cells += 1;
                report.cells_evaluated += 1;
                report.formula_cells += 1;
                report.repeated_formula_region_cells += 1;
                report.formula_evaluations += 1;
                register_formula_plan_cache_access(
                    &mut prepared_templates,
                    formula_plan_cache,
                    &region.formula,
                    None,
                    &mut report,
                    1,
                );
                let value = self.evaluate_optimized_formula_with_spill_repair(
                    &address,
                    &region.formula,
                    &valuation,
                    materialization_limit,
                )?;
                let spill_counters = self.publish_formula_value_to_valuation(
                    &mut valuation,
                    address,
                    region.revision,
                    value,
                    source,
                );
                report.spill_facts_published += spill_counters.facts_published;
                report.spill_facts_blocked += spill_counters.facts_blocked;
                report.spill_ghost_cells_published += spill_counters.ghost_cells_published;
            }
        }

        report.formula_templates_prepared = prepared_templates.len();
        report.distinct_formula_templates = prepared_templates.len();
        formula_plan_cache.prune_to_templates(&prepared_templates);
        report.compiled_formula_plans_cached = formula_plan_cache.cached_compiled_plan_count();

        self.repair_optimized_spills_with_oxfml(
            &mut valuation,
            &mut report,
            materialization_limit,
        )?;

        report.computed_dense_value_regions = valuation.dense_value_regions().len();
        report.computed_sparse_cells = valuation.sparse_computed_cells();
        valuation.refresh_spill_epoch_ledger();
        self.refresh_optimized_report_spill_counters(&valuation, &mut report);
        Ok((valuation, report))
    }

    pub fn recalculate_visible_rect_compact_with_oxfml(
        &self,
        visible_rect: GridRect,
        materialization_limit: u64,
    ) -> Result<(GridOptimizedValuation, GridOptimizedVisibleFirstReport), GridRefError> {
        self.check_rect(&visible_rect)?;
        let upstream_rect = self.visible_same_row_left_upstream_rect(&visible_rect)?;
        if upstream_rect.cell_count() > materialization_limit {
            return Err(GridRefError::RangeTooLargeForScalarInvalidation {
                cells: upstream_rect.cell_count(),
                limit: materialization_limit,
            });
        }

        let mut valuation = self.empty_valuation_with_committed_spill_state();
        let mut recalc = GridOptimizedRecalcReport {
            occupied_cells: 0,
            literal_cells: 0,
            formula_cells: 0,
            cells_evaluated: 0,
            formula_evaluations: 0,
            spill_repair_passes: 0,
            spill_repair_formula_evaluations: 0,
            spill_repair_converged: true,
            sparse_literal_cells: 0,
            sparse_formula_cells: 0,
            dense_value_region_cells: 0,
            repeated_formula_region_cells: 0,
            formula_templates_prepared: 0,
            distinct_formula_templates: 0,
            formula_plan_cache_hits: 0,
            formula_plan_cache_misses: 0,
            compiled_formula_plan_cache_hits: 0,
            compiled_formula_plan_cache_misses: 0,
            compiled_formula_plans_cached: 0,
            computed_dense_value_regions: 0,
            computed_sparse_cells: 0,
            spill_facts_published: 0,
            spill_facts_blocked: 0,
            spill_ghost_cells_published: 0,
        };
        let mut prepared_templates = BTreeSet::new();
        let mut formula_plan_cache = GridOptimizedFormulaPlanCache::default();

        for (coord, point) in &self.sparse_points {
            let address = self.address_from_coord(*coord);
            if !upstream_rect.contains(&address)
                || !self.final_source_matches(&address, GridOptimizedCellSource::SparsePoint)
            {
                continue;
            }
            recalc.occupied_cells += 1;
            recalc.cells_evaluated += 1;
            if let Some(value) = point.cell.literal_value() {
                recalc.literal_cells += 1;
                recalc.sparse_literal_cells += 1;
                valuation.insert_sparse_value(
                    address.clone(),
                    point.revision,
                    value,
                    GridOptimizedCellSource::SparsePoint,
                );
            } else if let Some(formula) = point.cell.formula_ref() {
                recalc.formula_cells += 1;
                recalc.sparse_formula_cells += 1;
                recalc.formula_evaluations += 1;
                register_formula_plan_cache_access(
                    &mut prepared_templates,
                    &mut formula_plan_cache,
                    formula,
                    None,
                    &mut recalc,
                    1,
                );
                let value = self.evaluate_optimized_formula_with_spill_repair(
                    &address,
                    formula,
                    &valuation,
                    materialization_limit,
                )?;
                self.publish_formula_value_to_valuation(
                    &mut valuation,
                    address.clone(),
                    point.revision,
                    value,
                    GridOptimizedCellSource::SparsePoint,
                );
            }
        }

        for (region_index, region) in self.dense_value_regions.iter().enumerate() {
            let Some(subrect) = grid_rect_intersection(&region.rect, &upstream_rect, self.bounds)?
            else {
                continue;
            };
            let source = GridOptimizedCellSource::DenseValueRegion { region_index };
            let mut final_cells = 0_u64;
            let mut all_cells_match = true;
            for address in subrect.scalar_cells(materialization_limit)? {
                if self.final_source_matches(&address, source) {
                    final_cells += 1;
                } else {
                    all_cells_match = false;
                }
            }
            if final_cells == 0 {
                continue;
            }
            recalc.occupied_cells = recalc.occupied_cells.saturating_add(final_cells);
            recalc.cells_evaluated = recalc.cells_evaluated.saturating_add(final_cells);
            recalc.literal_cells = recalc.literal_cells.saturating_add(final_cells);
            recalc.dense_value_region_cells =
                recalc.dense_value_region_cells.saturating_add(final_cells);

            if all_cells_match && final_cells == subrect.cell_count() {
                valuation.push_dense_value_payload(
                    subrect.clone(),
                    GridDenseValuePayload::from_calc_values(dense_values_for_subrect(
                        region, &subrect,
                    )),
                    region.revision,
                    source,
                );
            } else {
                for address in subrect.scalar_cells(materialization_limit)? {
                    if !self.final_source_matches(&address, source) {
                        continue;
                    }
                    let Some(value) = region.value_at(&address) else {
                        continue;
                    };
                    valuation.insert_sparse_value(address, region.revision, value, source);
                }
            }
        }

        for (region_index, region) in self.repeated_formula_regions.iter().enumerate() {
            let Some(subrect) = grid_rect_intersection(&region.rect, &upstream_rect, self.bounds)?
            else {
                continue;
            };
            let source = GridOptimizedCellSource::RepeatedFormulaRegion { region_index };
            if self.try_evaluate_repeated_formula_visible_subrect(
                region,
                &subrect,
                source,
                &mut valuation,
                &mut recalc,
                &mut prepared_templates,
                &mut formula_plan_cache,
                materialization_limit,
            )? {
                continue;
            }
            for address in subrect.scalar_cells(materialization_limit)? {
                if !self.final_source_matches(&address, source) {
                    continue;
                }
                recalc.occupied_cells += 1;
                recalc.cells_evaluated += 1;
                recalc.formula_cells += 1;
                recalc.repeated_formula_region_cells += 1;
                recalc.formula_evaluations += 1;
                register_formula_plan_cache_access(
                    &mut prepared_templates,
                    &mut formula_plan_cache,
                    &region.formula,
                    None,
                    &mut recalc,
                    1,
                );
                let value = self.evaluate_optimized_formula_with_spill_repair(
                    &address,
                    &region.formula,
                    &valuation,
                    materialization_limit,
                )?;
                self.publish_formula_value_to_valuation(
                    &mut valuation,
                    address,
                    region.revision,
                    value,
                    source,
                );
            }
        }

        recalc.formula_templates_prepared = prepared_templates.len();
        recalc.distinct_formula_templates = prepared_templates.len();
        formula_plan_cache.prune_to_templates(&prepared_templates);
        recalc.compiled_formula_plans_cached = formula_plan_cache.cached_compiled_plan_count();
        recalc.computed_dense_value_regions = valuation.dense_value_regions().len();
        recalc.computed_sparse_cells = valuation.sparse_computed_cells();
        valuation.refresh_spill_epoch_ledger();
        self.refresh_optimized_report_spill_counters(&valuation, &mut recalc);

        let stats = self.storage_stats();
        let report = GridOptimizedVisibleFirstReport {
            visible_cell_count: visible_rect.cell_count(),
            visible_upstream_cell_count: upstream_rect.cell_count(),
            cells_evaluated_before_visible_complete: recalc.cells_evaluated,
            formula_evaluations_before_visible_complete: recalc.formula_evaluations,
            dense_value_cells_projected: recalc.dense_value_region_cells,
            repeated_formula_cells_projected: recalc.repeated_formula_region_cells,
            sparse_point_cells_projected: recalc.sparse_literal_cells + recalc.sparse_formula_cells,
            computed_dense_value_regions: recalc.computed_dense_value_regions,
            computed_sparse_cells: recalc.computed_sparse_cells,
            full_recalc_occupied_cell_floor: stats.authored_cells_upper_bound,
            full_grid_cell_floor: u64::from(self.bounds.max_rows)
                .saturating_mul(u64::from(self.bounds.max_cols)),
            visible_rect,
            upstream_rect,
        };
        Ok((valuation, report))
    }

    pub fn recalculate_mark_all_dirty_compact_with_oxfml_cached(
        &self,
        materialization_limit: u64,
    ) -> Result<
        (
            GridOptimizedValuation,
            GridOptimizedRecalcReport,
            GridOptimizedWarmNoOpCache,
        ),
        GridRefError,
    > {
        let (valuation, report) =
            self.recalculate_mark_all_dirty_compact_with_oxfml(materialization_limit)?;
        let cache = GridOptimizedWarmNoOpCache {
            token: self.warm_noop_token(materialization_limit),
            valuation: valuation.clone(),
            baseline_report: report.clone(),
        };
        Ok((valuation, report, cache))
    }

    pub fn persistent_formula_plan_cache_report(
        &self,
        rounds: usize,
        materialization_limit: u64,
    ) -> Result<GridOptimizedFormulaPlanCacheReport, GridRefError> {
        let mut formula_plan_cache = GridOptimizedFormulaPlanCache::default();
        let mut round_reports = Vec::with_capacity(rounds);

        for round_index in 0..rounds {
            let (_, recalc) = self
                .recalculate_mark_all_dirty_compact_with_oxfml_using_formula_plan_cache(
                    materialization_limit,
                    &mut formula_plan_cache,
                )?;
            round_reports.push(GridOptimizedFormulaPlanCacheRoundReport {
                round_index: round_index + 1,
                formula_cells: recalc.formula_cells,
                distinct_formula_templates: recalc.distinct_formula_templates,
                formula_plan_cache_hits: recalc.formula_plan_cache_hits,
                formula_plan_cache_misses: recalc.formula_plan_cache_misses,
                compiled_formula_plan_cache_hits: recalc.compiled_formula_plan_cache_hits,
                compiled_formula_plan_cache_misses: recalc.compiled_formula_plan_cache_misses,
                cached_template_count_after_round: formula_plan_cache.cached_template_count(),
                cached_compiled_plan_count_after_round: formula_plan_cache
                    .cached_compiled_plan_count(),
            });
        }

        let formula_cells_per_round = round_reports.first().map_or(0, |round| round.formula_cells);
        let distinct_formula_templates = round_reports
            .first()
            .map_or(0, |round| round.distinct_formula_templates);
        let first_round_misses = round_reports
            .first()
            .map_or(0, |round| round.formula_plan_cache_misses);
        let later_round_misses = round_reports
            .iter()
            .skip(1)
            .map(|round| round.formula_plan_cache_misses)
            .sum();
        let total_hits = round_reports
            .iter()
            .map(|round| round.formula_plan_cache_hits)
            .sum();
        let total_misses = round_reports
            .iter()
            .map(|round| round.formula_plan_cache_misses)
            .sum();
        let total_compiled_plan_hits = round_reports
            .iter()
            .map(|round| round.compiled_formula_plan_cache_hits)
            .sum();
        let total_compiled_plan_misses = round_reports
            .iter()
            .map(|round| round.compiled_formula_plan_cache_misses)
            .sum();

        Ok(GridOptimizedFormulaPlanCacheReport {
            rounds,
            formula_cells_per_round,
            distinct_formula_templates,
            first_round_misses,
            later_round_misses,
            total_hits,
            total_misses,
            total_compiled_plan_hits,
            total_compiled_plan_misses,
            cached_template_count: formula_plan_cache.cached_template_count(),
            cached_compiled_plan_count: formula_plan_cache.cached_compiled_plan_count(),
            round_reports,
        })
    }

    #[must_use]
    pub fn recalculate_warm_noop_compact_with_oxfml(
        &self,
        cache: &GridOptimizedWarmNoOpCache,
    ) -> Option<(GridOptimizedValuation, GridOptimizedWarmNoOpReport)> {
        if self.warm_noop_token(cache.token.materialization_limit) != cache.token {
            return None;
        }
        Some((
            cache.valuation.clone(),
            GridOptimizedWarmNoOpReport {
                cache_hit: true,
                cached_occupied_cells: cache.baseline_report.occupied_cells,
                cached_formula_cells: cache.baseline_report.formula_cells,
                cells_visited: 0,
                formula_evaluations: 0,
            },
        ))
    }

    fn repair_optimized_spills_with_oxfml(
        &self,
        valuation: &mut GridOptimizedValuation,
        report: &mut GridOptimizedRecalcReport,
        materialization_limit: u64,
    ) -> Result<(), GridRefError> {
        let formula_cells = usize::try_from(report.formula_cells).unwrap_or(usize::MAX);
        if formula_cells == 0
            || valuation.spill_facts == self.spill_facts
            || !self.contains_grid_spill_reference_formula(materialization_limit)?
        {
            return Ok(());
        }

        report.spill_repair_converged = false;
        for _ in 0..formula_cells {
            let spill_facts_before = valuation.spill_facts.clone();
            report.spill_repair_passes += 1;

            for (coord, point) in &self.sparse_points {
                let address = self.address_from_coord(*coord);
                if !self.final_source_matches(&address, GridOptimizedCellSource::SparsePoint) {
                    continue;
                }
                let Some(formula) = point.cell.formula_ref() else {
                    continue;
                };
                report.spill_repair_formula_evaluations += 1;
                let value = self.evaluate_optimized_formula_with_spill_repair(
                    &address,
                    formula,
                    valuation,
                    materialization_limit,
                )?;
                self.publish_formula_value_to_valuation(
                    valuation,
                    address.clone(),
                    point.revision,
                    value,
                    GridOptimizedCellSource::SparsePoint,
                );
            }

            for (region_index, region) in self.repeated_formula_regions.iter().enumerate() {
                let source = GridOptimizedCellSource::RepeatedFormulaRegion { region_index };
                for address in region.rect.scalar_cells(materialization_limit)? {
                    if !self.final_source_matches(&address, source) {
                        continue;
                    }
                    report.spill_repair_formula_evaluations += 1;
                    let value = self.evaluate_optimized_formula_with_spill_repair(
                        &address,
                        &region.formula,
                        valuation,
                        materialization_limit,
                    )?;
                    self.publish_formula_value_to_valuation(
                        valuation,
                        address,
                        region.revision,
                        value,
                        source,
                    );
                }
            }

            if valuation.spill_facts == spill_facts_before {
                report.spill_repair_converged = true;
                break;
            }
        }

        Ok(())
    }

    fn contains_grid_spill_reference_formula(
        &self,
        materialization_limit: u64,
    ) -> Result<bool, GridRefError> {
        let profile = StrictExcelGridReferenceProfile::with_bounds(self.bounds);
        for (coord, point) in &self.sparse_points {
            let address = self.address_from_coord(*coord);
            if !self.final_source_matches(&address, GridOptimizedCellSource::SparsePoint) {
                continue;
            }
            let Some(formula) = point.cell.formula_ref() else {
                continue;
            };
            if formula_contains_grid_spill_reference(formula, &address, &profile, self.bounds) {
                return Ok(true);
            }
        }
        for (region_index, region) in self.repeated_formula_regions.iter().enumerate() {
            let source = GridOptimizedCellSource::RepeatedFormulaRegion { region_index };
            for address in region.rect.scalar_cells(materialization_limit)? {
                if self.final_source_matches(&address, source)
                    && formula_contains_grid_spill_reference(
                        &region.formula,
                        &address,
                        &profile,
                        self.bounds,
                    )
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn visible_same_row_left_upstream_rect(
        &self,
        visible_rect: &GridRect,
    ) -> Result<GridRect, GridRefError> {
        let mut left_col = visible_rect.left_col;
        for region in &self.repeated_formula_regions {
            if !grid_rects_overlap(&region.rect, visible_rect) {
                continue;
            }
            if region.formula.source_channel != FormulaChannelKind::WorksheetR1C1 {
                continue;
            }
            let template = region.formula.source_text.trim().to_ascii_uppercase();
            if template == "=RC[-1]*2" {
                let input_col = region.rect.left_col.saturating_sub(1).max(1);
                left_col = left_col.min(input_col);
            }
        }
        GridRect::new(
            visible_rect.workbook_id.clone(),
            visible_rect.sheet_id.clone(),
            visible_rect.top_row,
            left_col,
            visible_rect.bottom_row,
            visible_rect.right_col,
            self.bounds,
        )
    }

    fn try_evaluate_repeated_formula_visible_subrect(
        &self,
        region: &GridRepeatedFormulaRegion,
        subrect: &GridRect,
        source: GridOptimizedCellSource,
        valuation: &mut GridOptimizedValuation,
        report: &mut GridOptimizedRecalcReport,
        prepared_templates: &mut BTreeSet<String>,
        formula_plan_cache: &mut GridOptimizedFormulaPlanCache,
        materialization_limit: u64,
    ) -> Result<bool, GridRefError> {
        if region.formula.source_channel != FormulaChannelKind::WorksheetR1C1 {
            return Ok(false);
        }
        let Some(plan) = formula_plan_cache.compiled_plan_for_formula(&region.formula) else {
            return Ok(false);
        };
        if plan != GridOptimizedCompiledFormulaPlan::r1c1_double_left() {
            return Ok(false);
        }
        let cell_count = subrect.cell_count();
        if cell_count > materialization_limit {
            return Err(GridRefError::RangeTooLargeForScalarInvalidation {
                cells: cell_count,
                limit: materialization_limit,
            });
        }
        let cell_count_usize = usize::try_from(cell_count).map_err(|_| {
            GridRefError::RangeTooLargeForScalarInvalidation {
                cells: cell_count,
                limit: materialization_limit,
            }
        })?;
        for address in subrect.scalar_cells(materialization_limit)? {
            if !self.final_source_matches(&address, source) {
                return Ok(false);
            }
        }

        let mut values = Vec::<f64>::with_capacity(cell_count_usize);
        for row in subrect.top_row..=subrect.bottom_row {
            for col in subrect.left_col..=subrect.right_col {
                let input_number = if col > subrect.left_col && col > region.rect.left_col {
                    values[values.len().saturating_sub(1)]
                } else {
                    let Some(input_col) = col.checked_sub(1).filter(|input_col| *input_col >= 1)
                    else {
                        return Ok(false);
                    };
                    let input = valuation
                        .read_cell(&ExcelGridCellAddress::new(
                            region.rect.workbook_id.clone(),
                            region.rect.sheet_id.clone(),
                            row,
                            input_col,
                        ))
                        .computed;
                    let Some(number) = number_from_calc_value(&input) else {
                        return Ok(false);
                    };
                    number
                };
                values.push(input_number * 2.0);
            }
        }

        report.occupied_cells = report.occupied_cells.saturating_add(cell_count);
        report.cells_evaluated = report.cells_evaluated.saturating_add(cell_count);
        report.formula_cells = report.formula_cells.saturating_add(cell_count);
        report.repeated_formula_region_cells = report
            .repeated_formula_region_cells
            .saturating_add(cell_count);
        report.formula_evaluations = report.formula_evaluations.saturating_add(cell_count);
        register_formula_plan_cache_access(
            prepared_templates,
            formula_plan_cache,
            &region.formula,
            Some(plan),
            report,
            cell_count,
        );
        valuation.push_dense_value_payload(
            subrect.clone(),
            GridDenseValuePayload::from_numbers(values),
            region.revision,
            source,
        );
        Ok(true)
    }

    fn try_evaluate_repeated_formula_region_fast_path(
        &self,
        _region_index: usize,
        region: &GridRepeatedFormulaRegion,
        source: GridOptimizedCellSource,
        valuation: &mut GridOptimizedValuation,
        report: &mut GridOptimizedRecalcReport,
        prepared_templates: &mut BTreeSet<String>,
        formula_plan_cache: &mut GridOptimizedFormulaPlanCache,
        materialization_limit: u64,
    ) -> Result<bool, GridRefError> {
        if region.formula.source_channel != FormulaChannelKind::WorksheetR1C1 {
            return Ok(false);
        }
        let Some(plan) = formula_plan_cache.compiled_plan_for_formula(&region.formula) else {
            return Ok(false);
        };
        let cell_count = region.rect.cell_count();
        if cell_count > materialization_limit {
            return Err(GridRefError::RangeTooLargeForScalarInvalidation {
                cells: cell_count,
                limit: materialization_limit,
            });
        }
        let cell_count_usize = usize::try_from(cell_count).map_err(|_| {
            GridRefError::RangeTooLargeForScalarInvalidation {
                cells: cell_count,
                limit: materialization_limit,
            }
        })?;
        for row in region.rect.top_row..=region.rect.bottom_row {
            for col in region.rect.left_col..=region.rect.right_col {
                let address = ExcelGridCellAddress::new(
                    region.rect.workbook_id.clone(),
                    region.rect.sheet_id.clone(),
                    row,
                    col,
                );
                if !self.final_source_matches(&address, source) {
                    return Ok(false);
                }
            }
        }

        let mut values = Vec::<CalcValue>::with_capacity(cell_count_usize);
        for row in region.rect.top_row..=region.rect.bottom_row {
            for col in region.rect.left_col..=region.rect.right_col {
                let Some(value) =
                    plan.evaluate_repeated_region_cell(row, col, region, &values, valuation)
                else {
                    return Ok(false);
                };
                values.push(value);
            }
        }

        report.occupied_cells = report.occupied_cells.saturating_add(cell_count);
        report.cells_evaluated = report.cells_evaluated.saturating_add(cell_count);
        report.formula_cells = report.formula_cells.saturating_add(cell_count);
        report.repeated_formula_region_cells = report
            .repeated_formula_region_cells
            .saturating_add(cell_count);
        report.formula_evaluations = report.formula_evaluations.saturating_add(cell_count);
        register_formula_plan_cache_access(
            prepared_templates,
            formula_plan_cache,
            &region.formula,
            Some(plan),
            report,
            cell_count,
        );
        valuation.push_dense_value_payload(
            region.rect.clone(),
            GridDenseValuePayload::from_calc_values(values),
            region.revision,
            source,
        );
        Ok(true)
    }

    fn evaluate_optimized_formula_with_spill_repair(
        &self,
        address: &ExcelGridCellAddress,
        formula: &GridFormulaCell,
        valuation: &GridOptimizedValuation,
        materialization_limit: u64,
    ) -> Result<CalcValue, GridRefError> {
        if let Some(value) = evaluate_optimized_formula_fast_path(address, formula, valuation) {
            return Ok(value);
        }
        match self.evaluate_optimized_formula_with_oxfml(
            address,
            formula,
            valuation,
            materialization_limit,
        ) {
            Ok(value) => Ok(value),
            Err(error) => {
                let profile = StrictExcelGridReferenceProfile::with_bounds(self.bounds);
                if formula_contains_grid_spill_reference(formula, address, &profile, self.bounds) {
                    Ok(CalcValue::error(WorksheetErrorCode::Ref))
                } else {
                    Err(error)
                }
            }
        }
    }

    fn refresh_optimized_report_spill_counters(
        &self,
        valuation: &GridOptimizedValuation,
        report: &mut GridOptimizedRecalcReport,
    ) {
        let counters = count_formula_spill_publications(&valuation.spill_facts, |anchor| {
            self.authored_cell_at(anchor).is_some_and(|readout| {
                matches!(readout.authored, Some(GridAuthoredCell::Formula(_)))
            })
        });
        report.spill_facts_published = counters.facts_published;
        report.spill_facts_blocked = counters.facts_blocked;
        report.spill_ghost_cells_published = counters.ghost_cells_published;
    }

    pub fn project_authored_to_reference(
        &self,
        materialization_limit: u64,
    ) -> Result<GridCalcRefSheet, GridRefError> {
        let mut authored = BTreeMap::<ExcelGridCellAddress, GridVersionedAuthoredCell>::new();
        for (coord, point) in &self.sparse_points {
            authored.insert(self.address_from_coord(*coord), point.clone());
        }
        self.overlay_dense_regions(&mut authored, materialization_limit)?;
        self.overlay_repeated_formula_regions(&mut authored, materialization_limit)?;

        let mut reference =
            GridCalcRefSheet::new(self.workbook_id.clone(), self.sheet_id.clone(), self.bounds);
        reference.axis_state = self.axis_state.clone();
        reference.spill_facts = self.spill_facts.clone();
        reference.spill_value_fingerprints = self.spill_value_fingerprints.clone();
        reference.spill_epoch_ledger = self.spill_epoch_ledger.clone();
        reference.defined_names = self.defined_names.clone();
        reference.table_overlays = self.table_overlays.clone();
        reference.merged_regions = self.merged_regions.clone();
        reference.feature_rendered_regions = self.feature_rendered_regions.clone();
        for (address, cell) in authored {
            match cell.cell.to_authored() {
                GridAuthoredCell::Literal(value) => reference.set_literal(address, value)?,
                GridAuthoredCell::Formula(formula) => reference.set_formula(address, formula)?,
            }
        }
        Ok(reference)
    }

    fn populate_compact_literal_valuation(
        &self,
        valuation: &mut GridOptimizedValuation,
        report: &mut GridOptimizedRecalcReport,
        materialization_limit: u64,
    ) -> Result<(), GridRefError> {
        for (coord, point) in &self.sparse_points {
            let address = self.address_from_coord(*coord);
            if !self.final_source_matches(&address, GridOptimizedCellSource::SparsePoint) {
                continue;
            }
            let Some(value) = point.cell.literal_value() else {
                continue;
            };
            report.occupied_cells += 1;
            report.cells_evaluated += 1;
            report.literal_cells += 1;
            report.sparse_literal_cells += 1;
            valuation.insert_sparse_value(
                address.clone(),
                point.revision,
                value,
                GridOptimizedCellSource::SparsePoint,
            );
        }

        for (region_index, region) in self.dense_value_regions.iter().enumerate() {
            let source = GridOptimizedCellSource::DenseValueRegion { region_index };
            let mut final_cells = 0;
            for address in region.rect.scalar_cells(materialization_limit)? {
                if self.final_source_matches(&address, source) {
                    final_cells += 1;
                }
            }
            if final_cells == 0 {
                continue;
            }
            report.occupied_cells += final_cells;
            report.cells_evaluated += final_cells;
            report.literal_cells += final_cells;
            report.dense_value_region_cells += final_cells;
            valuation.push_dense_value_region(
                region.rect.clone(),
                region.row_major_values(),
                region.revision,
                source,
            );
        }
        Ok(())
    }

    fn evaluate_optimized_formula_with_oxfml(
        &self,
        address: &ExcelGridCellAddress,
        formula: &GridFormulaCell,
        valuation: &GridOptimizedValuation,
        materialization_limit: u64,
    ) -> Result<CalcValue, GridRefError> {
        let provider = valuation.reference_system_provider_with_dense_materialization_limit(
            address.row,
            address.col,
            materialization_limit,
        );
        let host_info = GridHostInfoProvider::new(
            self.workbook_id.clone(),
            self.sheet_id.clone(),
            address.row,
            address.col,
            self.bounds,
            self.spill_facts.values(),
            &self.axis_state,
        );
        let query_bundle = TypedContextQueryBundle::new(
            Some(&host_info as &dyn HostInfoProvider),
            None,
            None,
            None,
            None,
        )
        .with_reference_system_provider(Some(&provider as &dyn ReferenceSystemProvider));
        let source = FormulaSourceRecord::new(
            format!(
                "grid-optimized:{}:{}:R{}C{}",
                self.workbook_id, self.sheet_id, address.row, address.col
            ),
            1,
            formula.source_text.clone(),
        )
        .with_formula_channel_kind(formula.source_channel);
        let profile = StrictExcelGridReferenceProfile::with_bounds(self.bounds);
        let (enclosing_table_ref, caller_table_region) =
            grid_table_caller_context(self.table_overlays.values(), address);
        let environment = RuntimeEnvironment::new()
            .with_formula_scope(self.workbook_id.clone(), self.sheet_id.clone())
            .with_caller_position(address.row, address.col)
            .with_primary_locus(Locus {
                sheet_id: self.sheet_id.clone(),
                row: address.row,
                col: address.col,
            })
            .with_structure_context_version(StructureContextVersion(format!(
                "grid-optimized:{}:{}:{}x{}",
                self.workbook_id, self.sheet_id, self.bounds.max_rows, self.bounds.max_cols
            )))
            .with_table_context(
                grid_table_descriptor_catalog(self.table_overlays.values()),
                enclosing_table_ref,
                caller_table_region,
            )
            .with_reference_bind_profile(&profile);
        let request = RuntimeFormulaRequest::new(source, query_bundle)
            .with_backend(EvaluationBackend::OxFuncBacked);
        environment
            .execute(request)
            .map(|result| result.published_calc_value())
            .map_err(|detail| GridRefError::OxfmlEvaluationFailed {
                address: address.clone(),
                detail,
            })
    }

    fn publish_formula_value_to_valuation(
        &self,
        valuation: &mut GridOptimizedValuation,
        address: ExcelGridCellAddress,
        revision: u64,
        value: CalcValue,
        source: GridOptimizedCellSource,
    ) -> GridSpillPublicationCounters {
        valuation.clear_formula_output_for_anchor(&address);

        let Some(array) = value.as_array() else {
            valuation.insert_sparse_value(address, revision, value, source);
            return GridSpillPublicationCounters::default();
        };

        let Some(extent) = spill_extent_for_array(&address, array.shape(), self.bounds) else {
            valuation.insert_sparse_value(
                address.clone(),
                revision,
                CalcValue::error(WorksheetErrorCode::Spill),
                source,
            );
            valuation
                .spill_value_fingerprints
                .insert(address.clone(), blocked_spill_value_fingerprint(array));
            valuation.spill_facts.insert(
                address.clone(),
                GridSpillFact {
                    anchor: address.clone(),
                    extent: anchor_cell_rect(&address, self.bounds),
                    blocked: true,
                },
            );
            return GridSpillPublicationCounters {
                facts_blocked: 1,
                ..GridSpillPublicationCounters::default()
            };
        };

        if self.optimized_spill_extent_is_blocked(&address, &extent, valuation) {
            valuation.insert_sparse_value(
                address.clone(),
                revision,
                CalcValue::error(WorksheetErrorCode::Spill),
                source,
            );
            valuation
                .spill_value_fingerprints
                .insert(address.clone(), blocked_spill_value_fingerprint(array));
            valuation.spill_facts.insert(
                address.clone(),
                GridSpillFact {
                    anchor: address.clone(),
                    extent,
                    blocked: true,
                },
            );
            return GridSpillPublicationCounters {
                facts_blocked: 1,
                ..GridSpillPublicationCounters::default()
            };
        }

        valuation.push_dense_value_payload(
            extent.clone(),
            GridDenseValuePayload::from_calc_array(array),
            revision,
            source,
        );
        valuation.spill_facts.insert(
            address.clone(),
            GridSpillFact {
                anchor: address.clone(),
                extent,
                blocked: false,
            },
        );
        valuation
            .spill_value_fingerprints
            .insert(address, calc_array_value_fingerprint(array));
        GridSpillPublicationCounters {
            facts_published: 1,
            ghost_cells_published: array.cell_count().saturating_sub(1),
            ..GridSpillPublicationCounters::default()
        }
    }

    fn optimized_spill_extent_is_blocked(
        &self,
        anchor: &ExcelGridCellAddress,
        extent: &GridRect,
        valuation: &GridOptimizedValuation,
    ) -> bool {
        self.optimized_spill_blockage_probe_report_with_facts(
            anchor,
            extent,
            &valuation.spill_facts,
        )
        .is_ok_and(|report| report.blocked)
    }

    pub fn optimized_spill_blockage_probe_report(
        &self,
        anchor: &ExcelGridCellAddress,
        extent: &GridRect,
    ) -> Result<GridOptimizedSpillBlockageProbeReport, GridRefError> {
        self.optimized_spill_blockage_probe_report_with_facts(anchor, extent, &self.spill_facts)
    }

    fn optimized_spill_blockage_probe_report_with_facts(
        &self,
        anchor: &ExcelGridCellAddress,
        extent: &GridRect,
        spill_facts: &BTreeMap<ExcelGridCellAddress, GridSpillFact>,
    ) -> Result<GridOptimizedSpillBlockageProbeReport, GridRefError> {
        self.check_address(anchor)?;
        self.check_rect(extent)?;
        let mut report = GridOptimizedSpillBlockageProbeReport {
            anchor: anchor.clone(),
            extent: extent.clone(),
            extent_cell_count: extent.cell_count(),
            naive_extent_cell_probe_floor: extent.cell_count(),
            sparse_point_candidates: 0,
            dense_value_region_candidates: 0,
            repeated_formula_region_candidates: 0,
            merged_region_candidates: 0,
            feature_rendered_region_candidates: 0,
            blocked_formula_spill_fact_candidates: 0,
            unblocked_spill_fact_candidates: 0,
            blocked: false,
        };

        for fact in spill_facts.values() {
            if fact.blocked
                && fact.anchor != *anchor
                && fact.extent.contains(anchor)
                && self.authored_cell_at(&fact.anchor).is_some_and(|readout| {
                    matches!(readout.authored, Some(GridAuthoredCell::Formula(_)))
                })
            {
                report.blocked_formula_spill_fact_candidates += 1;
                report.blocked = true;
            }
        }

        for (coord, _point) in self.sparse_points.range(
            GridCellCoord::new(extent.top_row, 0)..=GridCellCoord::new(extent.bottom_row, u32::MAX),
        ) {
            if coord.col < extent.left_col || coord.col > extent.right_col {
                continue;
            }
            let address = ExcelGridCellAddress::new(
                self.workbook_id.clone(),
                self.sheet_id.clone(),
                coord.row,
                coord.col,
            );
            if &address == anchor {
                continue;
            }
            report.sparse_point_candidates += 1;
            report.blocked = true;
        }

        for region in &self.dense_value_regions {
            if grid_rects_overlap(&region.rect, extent)
                && rects_overlap_outside_anchor(&region.rect, extent, anchor)
            {
                report.dense_value_region_candidates += 1;
                report.blocked = true;
            }
        }

        for region in &self.repeated_formula_regions {
            if grid_rects_overlap(&region.rect, extent)
                && rects_overlap_outside_anchor(&region.rect, extent, anchor)
            {
                report.repeated_formula_region_candidates += 1;
                report.blocked = true;
            }
        }

        for region in &self.merged_regions {
            if grid_rects_overlap(&region.rect, extent)
                && rects_overlap_outside_anchor(&region.rect, extent, anchor)
            {
                report.merged_region_candidates += 1;
                report.blocked = true;
            }
        }

        for region in &self.feature_rendered_regions {
            if feature_rendered_region_blocks_spill(&region.feature_kind)
                && grid_rects_overlap(&region.rect, extent)
                && rects_overlap_outside_anchor(&region.rect, extent, anchor)
            {
                report.feature_rendered_region_candidates += 1;
                report.blocked = true;
            }
        }

        for fact in spill_facts.values() {
            if fact.blocked || fact.anchor == *anchor {
                continue;
            }
            if grid_rects_overlap(&fact.extent, extent)
                && rects_overlap_outside_anchor(&fact.extent, extent, anchor)
            {
                report.unblocked_spill_fact_candidates += 1;
                report.blocked = true;
            }
        }

        Ok(report)
    }

    fn overlay_dense_regions(
        &self,
        authored: &mut BTreeMap<ExcelGridCellAddress, GridVersionedAuthoredCell>,
        materialization_limit: u64,
    ) -> Result<(), GridRefError> {
        for region in &self.dense_value_regions {
            let cells = region.rect.scalar_cells(materialization_limit)?;
            for (address, value) in cells.into_iter().zip(region.row_major_values()) {
                overlay_versioned_cell(
                    authored,
                    address,
                    region.revision,
                    GridAuthoredCell::Literal(value),
                );
            }
        }
        Ok(())
    }

    fn overlay_repeated_formula_regions(
        &self,
        authored: &mut BTreeMap<ExcelGridCellAddress, GridVersionedAuthoredCell>,
        materialization_limit: u64,
    ) -> Result<(), GridRefError> {
        for region in &self.repeated_formula_regions {
            for address in region.rect.scalar_cells(materialization_limit)? {
                overlay_versioned_cell(
                    authored,
                    address,
                    region.revision,
                    GridAuthoredCell::Formula(region.formula.clone()),
                );
            }
        }
        Ok(())
    }

    fn final_source_matches(
        &self,
        address: &ExcelGridCellAddress,
        source: GridOptimizedCellSource,
    ) -> bool {
        self.authored_cell_at(address)
            .map_or(false, |readout| readout.source == Some(source))
    }

    fn address_from_coord(&self, coord: GridCellCoord) -> ExcelGridCellAddress {
        ExcelGridCellAddress::new(
            self.workbook_id.clone(),
            self.sheet_id.clone(),
            coord.row,
            coord.col,
        )
    }

    fn warm_noop_token(&self, materialization_limit: u64) -> GridOptimizedWarmNoOpToken {
        GridOptimizedWarmNoOpToken {
            materialization_limit,
            next_revision: self.next_revision,
            axis_state: self.axis_state.clone(),
            sparse_points: self
                .sparse_points
                .iter()
                .map(|(coord, point)| GridOptimizedSparsePointToken {
                    coord: *coord,
                    revision: point.revision,
                    authored: match &point.cell {
                        GridOptimizedAuthoredCell::Number(_)
                        | GridOptimizedAuthoredCell::Literal(_) => {
                            GridOptimizedAuthoredCellToken::Literal
                        }
                        GridOptimizedAuthoredCell::Formula(formula) => {
                            GridOptimizedAuthoredCellToken::Formula {
                                source_text: formula.source_text.clone(),
                                normal_form_key: formula.normal_form_key.clone(),
                                source_channel: formula.source_channel,
                            }
                        }
                    },
                })
                .collect(),
            dense_value_regions: self
                .dense_value_regions
                .iter()
                .map(|region| GridOptimizedDenseValueRegionToken {
                    rect: region.rect.clone(),
                    revision: region.revision,
                    value_count: usize::try_from(region.rect.cell_count()).unwrap_or(usize::MAX),
                })
                .collect(),
            repeated_formula_regions: self
                .repeated_formula_regions
                .iter()
                .map(|region| GridOptimizedRepeatedFormulaRegionToken {
                    rect: region.rect.clone(),
                    revision: region.revision,
                    source_text: region.formula.source_text.clone(),
                    normal_form_key: region.formula.normal_form_key.clone(),
                    source_channel: region.formula.source_channel,
                })
                .collect(),
            merged_regions: self.merged_regions.clone(),
            feature_rendered_regions: self.feature_rendered_regions.clone(),
            spill_facts: self
                .spill_facts
                .iter()
                .map(|(address, fact)| (address.clone(), fact.clone()))
                .collect(),
            defined_names: self
                .defined_names
                .iter()
                .map(|(name, rect)| (name.clone(), rect.clone()))
                .collect(),
            table_overlays: self
                .table_overlays
                .iter()
                .map(|(table, overlay)| (table.clone(), overlay.clone()))
                .collect(),
        }
    }

    fn allocate_revision(&mut self) -> u64 {
        let revision = self.next_revision;
        self.next_revision = self.next_revision.saturating_add(1);
        revision
    }

    fn check_address(&self, address: &ExcelGridCellAddress) -> Result<(), GridRefError> {
        if address.workbook_id != self.workbook_id || address.sheet_id != self.sheet_id {
            return Err(GridRefError::AddressOnDifferentSheet {
                expected_workbook_id: self.workbook_id.clone(),
                expected_sheet_id: self.sheet_id.clone(),
                actual_workbook_id: address.workbook_id.clone(),
                actual_sheet_id: address.sheet_id.clone(),
            });
        }
        if !self.bounds.contains_row(address.row) || !self.bounds.contains_col(address.col) {
            return Err(GridRefError::AddressOutOfBounds {
                row: address.row,
                col: address.col,
                max_rows: self.bounds.max_rows,
                max_cols: self.bounds.max_cols,
            });
        }
        Ok(())
    }

    fn check_rect(&self, rect: &GridRect) -> Result<(), GridRefError> {
        rect.check_workbook_sheet(&self.workbook_id, &self.sheet_id)?;
        if !self.bounds.contains_row(rect.top_row)
            || !self.bounds.contains_row(rect.bottom_row)
            || !self.bounds.contains_col(rect.left_col)
            || !self.bounds.contains_col(rect.right_col)
        {
            return Err(GridRefError::RangeOutOfBounds {
                top_row: rect.top_row,
                left_col: rect.left_col,
                bottom_row: rect.bottom_row,
                right_col: rect.right_col,
                max_rows: self.bounds.max_rows,
                max_cols: self.bounds.max_cols,
            });
        }
        Ok(())
    }
}

fn overlay_versioned_cell(
    authored: &mut BTreeMap<ExcelGridCellAddress, GridVersionedAuthoredCell>,
    address: ExcelGridCellAddress,
    revision: u64,
    cell: GridAuthoredCell,
) {
    let should_insert = authored
        .get(&address)
        .map_or(true, |existing| revision >= existing.revision);
    if should_insert {
        authored.insert(
            address,
            GridVersionedAuthoredCell {
                revision,
                cell: GridOptimizedAuthoredCell::from_authored(cell),
            },
        );
    }
}

fn register_formula_plan_cache_access(
    prepared_templates: &mut BTreeSet<String>,
    formula_plan_cache: &mut GridOptimizedFormulaPlanCache,
    formula: &GridFormulaCell,
    compiled_plan: Option<GridOptimizedCompiledFormulaPlan>,
    report: &mut GridOptimizedRecalcReport,
    lookups: u64,
) {
    if lookups == 0 {
        return;
    }
    let key = formula.normal_form_key.clone();
    prepared_templates.insert(key.clone());
    if let Some(plan) = compiled_plan {
        let fingerprint = GridOptimizedFormulaPlanFingerprint::from_formula(formula);
        if formula_plan_cache
            .compiled_plans
            .get(&key)
            .is_some_and(|entry| entry.fingerprint == fingerprint)
        {
            report.compiled_formula_plan_cache_hits =
                report.compiled_formula_plan_cache_hits.saturating_add(1);
        } else {
            report.compiled_formula_plan_cache_misses =
                report.compiled_formula_plan_cache_misses.saturating_add(1);
            formula_plan_cache.compiled_plans.insert(
                key.clone(),
                GridOptimizedCompiledFormulaPlanEntry { fingerprint, plan },
            );
        }
    }
    if formula_plan_cache.templates.insert(key) {
        report.formula_plan_cache_misses = report.formula_plan_cache_misses.saturating_add(1);
        report.formula_plan_cache_hits = report
            .formula_plan_cache_hits
            .saturating_add(lookups.saturating_sub(1));
    } else {
        report.formula_plan_cache_hits = report.formula_plan_cache_hits.saturating_add(lookups);
    }
}

fn normalized_r1c1_expression(source_text: &str) -> Option<String> {
    let text = source_text.trim();
    let expression = text.strip_prefix('=')?;
    Some(
        expression
            .chars()
            .filter(|ch| !ch.is_ascii_whitespace())
            .collect::<String>()
            .to_ascii_uppercase(),
    )
}

fn compile_r1c1_range_aggregate_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1RangeAggregatePlan> {
    GridOptimizedR1C1RangeAggregateFunction::ALL
        .into_iter()
        .find_map(|function| compile_r1c1_range_function_expression(expression, function))
}

fn compile_r1c1_range_function_expression(
    expression: &str,
    function: GridOptimizedR1C1RangeAggregateFunction,
) -> Option<GridOptimizedR1C1RangeAggregatePlan> {
    let prefix = format!("{}(", function.name());
    let inner = expression.strip_prefix(&prefix)?.strip_suffix(')')?;
    let separator = find_r1c1_range_separator(inner)?;
    let start = inner.get(..separator)?;
    let end = inner.get(separator + 1..)?;
    Some(GridOptimizedR1C1RangeAggregatePlan {
        function,
        start: parse_r1c1_reference(start)?,
        end: parse_r1c1_reference(end)?,
    })
}

fn compile_r1c1_argument_aggregate_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1ArgumentAggregatePlan> {
    GridOptimizedR1C1RangeAggregateFunction::ALL
        .into_iter()
        .find_map(|function| {
            compile_r1c1_argument_aggregate_function_expression(expression, function)
        })
}

fn compile_r1c1_argument_aggregate_function_expression(
    expression: &str,
    function: GridOptimizedR1C1RangeAggregateFunction,
) -> Option<GridOptimizedR1C1ArgumentAggregatePlan> {
    let prefix = format!("{}(", function.name());
    let inner = expression.strip_prefix(&prefix)?.strip_suffix(')')?;
    let arguments = split_top_level_commas(inner)?
        .into_iter()
        .map(parse_r1c1_aggregate_argument)
        .collect::<Option<Vec<_>>>()?;
    Some(GridOptimizedR1C1ArgumentAggregatePlan {
        function,
        arguments,
    })
}

fn parse_r1c1_aggregate_argument(text: &str) -> Option<GridOptimizedR1C1AggregateArgument> {
    let mut text = text;
    while let Some(inner) = strip_outer_r1c1_parens(text) {
        text = inner;
    }
    if let Some(separator) = find_r1c1_range_separator(text) {
        let start = text.get(..separator)?;
        let end = text.get(separator + 1..)?;
        if let (Some(start), Some(end)) = (parse_r1c1_reference(start), parse_r1c1_reference(end)) {
            return Some(GridOptimizedR1C1AggregateArgument::Range { start, end });
        }
    }
    parse_r1c1_scalar_expression(text).map(GridOptimizedR1C1AggregateArgument::Scalar)
}

fn compile_r1c1_scalar_function_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1ScalarFunctionPlan> {
    GridOptimizedR1C1ScalarFunction::ALL
        .into_iter()
        .find_map(|function| compile_r1c1_scalar_function_call(expression, function))
}

fn compile_r1c1_scalar_function_call(
    expression: &str,
    function: GridOptimizedR1C1ScalarFunction,
) -> Option<GridOptimizedR1C1ScalarFunctionPlan> {
    let prefix = format!("{}(", function.name());
    let inner = expression.strip_prefix(&prefix)?.strip_suffix(')')?;
    let arguments = if inner.is_empty() {
        Vec::new()
    } else {
        split_top_level_commas(inner)?
            .into_iter()
            .map(parse_r1c1_scalar_expression)
            .collect::<Option<Vec<_>>>()?
    };
    if !function.arity_holds(arguments.len()) {
        return None;
    }
    Some(GridOptimizedR1C1ScalarFunctionPlan {
        function,
        arguments,
    })
}

fn compile_r1c1_reference_function_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1ReferenceFunctionPlan> {
    GridOptimizedR1C1ReferenceFunction::ALL
        .into_iter()
        .find_map(|function| compile_r1c1_reference_function_call(expression, function))
}

fn compile_r1c1_index_expression(expression: &str) -> Option<GridOptimizedR1C1IndexPlan> {
    let inner = expression.strip_prefix("INDEX(")?.strip_suffix(')')?;
    let args = split_top_level_commas(inner)?;
    let [range_text, row_text] = args.as_slice() else {
        let [range_text, row_text, col_text] = args.as_slice() else {
            return None;
        };
        let (start, end) = parse_r1c1_range_or_single_reference(range_text)?;
        return Some(GridOptimizedR1C1IndexPlan {
            start,
            end,
            row_index: Box::new(parse_r1c1_scalar_expression(row_text)?),
            col_index: Box::new(parse_r1c1_scalar_expression(col_text)?),
        });
    };
    let (start, end) = parse_r1c1_range_or_single_reference(range_text)?;
    Some(GridOptimizedR1C1IndexPlan {
        start,
        end,
        row_index: Box::new(parse_r1c1_scalar_expression(row_text)?),
        col_index: Box::new(r1c1_number_literal_expression(1.0)?),
    })
}

fn compile_r1c1_match_expression(expression: &str) -> Option<GridOptimizedR1C1MatchPlan> {
    let inner = expression.strip_prefix("MATCH(")?.strip_suffix(')')?;
    let args = split_top_level_commas(inner)?;
    let [lookup_text, range_text, match_type_text] = args.as_slice() else {
        return None;
    };
    parse_r1c1_exact_match_type(match_type_text)?;
    let (start, end) = parse_r1c1_range_or_single_reference(range_text)?;
    Some(GridOptimizedR1C1MatchPlan {
        lookup: Box::new(parse_r1c1_scalar_expression(lookup_text)?),
        start,
        end,
    })
}

fn parse_r1c1_exact_match_type(text: &str) -> Option<()> {
    let value = text.parse::<f64>().ok()?;
    (value == 0.0).then_some(())
}

fn compile_r1c1_vlookup_expression(expression: &str) -> Option<GridOptimizedR1C1VLookupPlan> {
    let inner = expression.strip_prefix("VLOOKUP(")?.strip_suffix(')')?;
    let args = split_top_level_commas(inner)?;
    let [lookup_text, range_text, col_index_text, exact_mode_text] = args.as_slice() else {
        return None;
    };
    parse_r1c1_exact_lookup_mode(exact_mode_text)?;
    let (start, end) = parse_r1c1_range_or_single_reference(range_text)?;
    Some(GridOptimizedR1C1VLookupPlan {
        lookup: Box::new(parse_r1c1_scalar_expression(lookup_text)?),
        start,
        end,
        col_index: Box::new(parse_r1c1_scalar_expression(col_index_text)?),
    })
}

fn parse_r1c1_exact_lookup_mode(text: &str) -> Option<()> {
    if text == "FALSE" {
        return Some(());
    }
    parse_r1c1_exact_match_type(text)
}

fn compile_r1c1_text_function_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1TextFunctionPlan> {
    compile_r1c1_len_function_expression(expression)
        .or_else(|| compile_r1c1_left_function_expression(expression))
        .or_else(|| compile_r1c1_right_function_expression(expression))
        .or_else(|| compile_r1c1_concat_function_expression(expression))
}

fn compile_r1c1_len_function_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1TextFunctionPlan> {
    let inner = expression.strip_prefix("LEN(")?.strip_suffix(')')?;
    let args = split_top_level_commas(inner)?;
    let [text] = args.as_slice() else {
        return None;
    };
    Some(GridOptimizedR1C1TextFunctionPlan::Len {
        text: parse_r1c1_text_reference_argument(text)?,
    })
}

fn compile_r1c1_left_function_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1TextFunctionPlan> {
    let inner = expression.strip_prefix("LEFT(")?.strip_suffix(')')?;
    let args = split_top_level_commas(inner)?;
    let [text, count] = args.as_slice() else {
        return None;
    };
    Some(GridOptimizedR1C1TextFunctionPlan::Left {
        text: parse_r1c1_text_reference_argument(text)?,
        count: Box::new(parse_r1c1_scalar_expression(count)?),
    })
}

fn compile_r1c1_right_function_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1TextFunctionPlan> {
    let inner = expression.strip_prefix("RIGHT(")?.strip_suffix(')')?;
    let args = split_top_level_commas(inner)?;
    let [text, count] = args.as_slice() else {
        return None;
    };
    Some(GridOptimizedR1C1TextFunctionPlan::Right {
        text: parse_r1c1_text_reference_argument(text)?,
        count: Box::new(parse_r1c1_scalar_expression(count)?),
    })
}

fn compile_r1c1_concat_function_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1TextFunctionPlan> {
    let inner = expression.strip_prefix("CONCAT(")?.strip_suffix(')')?;
    let texts = split_top_level_commas(inner)?
        .into_iter()
        .map(parse_r1c1_text_reference_argument)
        .collect::<Option<Vec<_>>>()?;
    if texts.is_empty() {
        return None;
    }
    Some(GridOptimizedR1C1TextFunctionPlan::Concat { texts })
}

fn compile_r1c1_reference_function_call(
    expression: &str,
    function: GridOptimizedR1C1ReferenceFunction,
) -> Option<GridOptimizedR1C1ReferenceFunctionPlan> {
    let prefix = format!("{}(", function.name());
    let inner = expression.strip_prefix(&prefix)?.strip_suffix(')')?;
    let argument = if inner.is_empty() {
        if !function.allows_current_cell_argument() {
            return None;
        }
        GridOptimizedR1C1ReferenceFunctionArgument::CurrentCell
    } else {
        let arguments = split_top_level_commas(inner)?;
        let [argument] = arguments.as_slice() else {
            return None;
        };
        parse_r1c1_reference_function_argument(argument)?
    };
    Some(GridOptimizedR1C1ReferenceFunctionPlan { function, argument })
}

fn parse_r1c1_reference_function_argument(
    text: &str,
) -> Option<GridOptimizedR1C1ReferenceFunctionArgument> {
    let mut text = text;
    while let Some(inner) = strip_outer_r1c1_parens(text) {
        text = inner;
    }
    if let Some(separator) = find_r1c1_range_separator(text) {
        let start = text.get(..separator)?;
        let end = text.get(separator + 1..)?;
        return Some(GridOptimizedR1C1ReferenceFunctionArgument::Range {
            start: parse_r1c1_reference(start)?,
            end: parse_r1c1_reference(end)?,
        });
    }
    parse_r1c1_reference(text).map(GridOptimizedR1C1ReferenceFunctionArgument::Ref)
}

fn parse_r1c1_text_reference_argument(text: &str) -> Option<GridOptimizedR1C1Ref> {
    let mut text = text;
    while let Some(inner) = strip_outer_r1c1_parens(text) {
        text = inner;
    }
    parse_r1c1_reference(text)
}

fn parse_r1c1_range_or_single_reference(
    text: &str,
) -> Option<(GridOptimizedR1C1Ref, GridOptimizedR1C1Ref)> {
    let mut text = text;
    while let Some(inner) = strip_outer_r1c1_parens(text) {
        text = inner;
    }
    if let Some(separator) = find_r1c1_range_separator(text) {
        let start = text.get(..separator)?;
        let end = text.get(separator + 1..)?;
        return Some((parse_r1c1_reference(start)?, parse_r1c1_reference(end)?));
    }
    let reference = parse_r1c1_reference(text)?;
    Some((reference, reference))
}

fn compile_r1c1_iferror_expression(expression: &str) -> Option<GridOptimizedR1C1IfErrorPlan> {
    let inner = expression.strip_prefix("IFERROR(")?.strip_suffix(')')?;
    let args = split_top_level_commas(inner)?;
    let [value_text, fallback_text] = args.as_slice() else {
        return None;
    };
    Some(GridOptimizedR1C1IfErrorPlan {
        value: parse_r1c1_scalar_expression(value_text)?,
        fallback: parse_r1c1_scalar_expression(fallback_text)?,
    })
}

fn compile_r1c1_if_expression(expression: &str) -> Option<GridOptimizedR1C1IfPlan> {
    let inner = expression.strip_prefix("IF(")?.strip_suffix(')')?;
    let args = split_top_level_commas(inner)?;
    let [condition_text, when_true_text, when_false_text] = args.as_slice() else {
        return None;
    };
    Some(GridOptimizedR1C1IfPlan {
        condition: parse_r1c1_logical_expression(condition_text)?,
        when_true: parse_r1c1_scalar_expression(when_true_text)?,
        when_false: parse_r1c1_scalar_expression(when_false_text)?,
    })
}

fn compile_r1c1_logical_function_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1LogicalFunctionPlan> {
    GridOptimizedR1C1LogicalFunction::ALL
        .into_iter()
        .find_map(|function| compile_r1c1_logical_function_call(expression, function))
}

fn compile_r1c1_logical_function_call(
    expression: &str,
    function: GridOptimizedR1C1LogicalFunction,
) -> Option<GridOptimizedR1C1LogicalFunctionPlan> {
    let prefix = format!("{}(", function.name());
    let inner = expression.strip_prefix(&prefix)?.strip_suffix(')')?;
    let arguments = split_top_level_commas(inner)?
        .into_iter()
        .map(parse_r1c1_logical_expression)
        .collect::<Option<Vec<_>>>()?;
    if !function.arity_holds(arguments.len()) {
        return None;
    }
    Some(GridOptimizedR1C1LogicalFunctionPlan {
        function,
        arguments,
    })
}

fn parse_r1c1_logical_expression(expression: &str) -> Option<GridOptimizedR1C1LogicalExpression> {
    let mut expression = expression;
    while let Some(inner) = strip_outer_r1c1_parens(expression) {
        expression = inner;
    }
    compile_r1c1_logical_function_expression(expression)
        .map(|plan| GridOptimizedR1C1LogicalExpression::Function(Box::new(plan)))
        .or_else(|| {
            parse_r1c1_comparison(expression).map(GridOptimizedR1C1LogicalExpression::Comparison)
        })
}

fn parse_r1c1_scalar_expression(expression: &str) -> Option<GridOptimizedR1C1ScalarExpression> {
    let mut expression = expression;
    while let Some(inner) = strip_outer_r1c1_parens(expression) {
        expression = inner;
    }
    compile_r1c1_range_aggregate_expression(expression)
        .map(GridOptimizedR1C1ScalarExpression::RangeAggregate)
        .or_else(|| {
            compile_r1c1_argument_aggregate_expression(expression)
                .map(GridOptimizedR1C1ScalarExpression::ArgumentAggregate)
        })
        .or_else(|| {
            compile_r1c1_iferror_expression(expression)
                .map(|plan| GridOptimizedR1C1ScalarExpression::IfError(Box::new(plan)))
        })
        .or_else(|| {
            compile_r1c1_if_expression(expression)
                .map(|plan| GridOptimizedR1C1ScalarExpression::If(Box::new(plan)))
        })
        .or_else(|| {
            compile_r1c1_scalar_function_expression(expression)
                .map(GridOptimizedR1C1ScalarExpression::ScalarFunction)
        })
        .or_else(|| {
            compile_r1c1_reference_function_expression(expression)
                .map(GridOptimizedR1C1ScalarExpression::ReferenceFunction)
        })
        .or_else(|| {
            compile_r1c1_match_expression(expression).map(GridOptimizedR1C1ScalarExpression::Match)
        })
        .or_else(|| {
            compile_r1c1_vlookup_expression(expression)
                .map(GridOptimizedR1C1ScalarExpression::VLookup)
        })
        .or_else(|| {
            compile_r1c1_index_expression(expression).map(GridOptimizedR1C1ScalarExpression::Index)
        })
        .or_else(|| {
            compile_r1c1_text_function_expression(expression)
                .map(GridOptimizedR1C1ScalarExpression::TextFunction)
        })
        .or_else(|| {
            compile_r1c1_binary_expression(expression)
                .map(GridOptimizedR1C1ScalarExpression::Binary)
        })
        .or_else(|| {
            compile_r1c1_unary_minus_expression(expression)
                .map(GridOptimizedR1C1ScalarExpression::UnaryMinus)
        })
        .or_else(|| parse_r1c1_operand(expression).map(GridOptimizedR1C1ScalarExpression::Operand))
}

fn strip_outer_r1c1_parens(expression: &str) -> Option<&str> {
    let inner = expression.strip_prefix('(')?.strip_suffix(')')?;
    if r1c1_outer_parens_enclose_expression(expression) {
        Some(inner)
    } else {
        None
    }
}

fn r1c1_outer_parens_enclose_expression(expression: &str) -> bool {
    let mut bracket_depth = 0_u32;
    let mut paren_depth = 0_u32;
    for (index, ch) in expression.char_indices() {
        match ch {
            '[' => bracket_depth = bracket_depth.saturating_add(1),
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '(' if bracket_depth == 0 => paren_depth = paren_depth.saturating_add(1),
            ')' if bracket_depth == 0 => {
                paren_depth = paren_depth.saturating_sub(1);
                if paren_depth == 0 && index + ch.len_utf8() < expression.len() {
                    return false;
                }
            }
            _ => {}
        }
    }
    paren_depth == 0
}

fn split_top_level_commas(expression: &str) -> Option<Vec<&str>> {
    let mut args = Vec::new();
    let mut start = 0_usize;
    let mut bracket_depth = 0_u32;
    let mut paren_depth = 0_u32;
    for (index, ch) in expression.char_indices() {
        match ch {
            '[' => bracket_depth = bracket_depth.saturating_add(1),
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '(' if bracket_depth == 0 => paren_depth = paren_depth.saturating_add(1),
            ')' if bracket_depth == 0 => paren_depth = paren_depth.saturating_sub(1),
            ',' if bracket_depth == 0 && paren_depth == 0 => {
                args.push(expression.get(start..index)?);
                start = index + 1;
            }
            _ => {}
        }
    }
    args.push(expression.get(start..)?);
    args.iter().all(|arg| !arg.is_empty()).then_some(args)
}

fn parse_r1c1_comparison(expression: &str) -> Option<GridOptimizedR1C1Comparison> {
    let mut expression = expression;
    while let Some(inner) = strip_outer_r1c1_parens(expression) {
        expression = inner;
    }
    let (operator_index, operator_len, op) = find_r1c1_comparison_operator(expression)?;
    let left = expression.get(..operator_index)?;
    let right = expression.get(operator_index + operator_len..)?;
    Some(GridOptimizedR1C1Comparison {
        left: parse_r1c1_scalar_expression(left)?,
        op,
        right: parse_r1c1_scalar_expression(right)?,
    })
}

fn compile_r1c1_comparison_expression(expression: &str) -> Option<GridOptimizedR1C1ComparisonPlan> {
    Some(GridOptimizedR1C1ComparisonPlan {
        comparison: parse_r1c1_comparison(expression)?,
    })
}

fn find_r1c1_comparison_operator(
    expression: &str,
) -> Option<(usize, usize, GridOptimizedR1C1ComparisonOp)> {
    let mut bracket_depth = 0_u32;
    let mut paren_depth = 0_u32;
    for (index, ch) in expression.char_indices() {
        match ch {
            '[' => bracket_depth = bracket_depth.saturating_add(1),
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '(' if bracket_depth == 0 => paren_depth = paren_depth.saturating_add(1),
            ')' if bracket_depth == 0 => paren_depth = paren_depth.saturating_sub(1),
            _ if bracket_depth == 0 && paren_depth == 0 => {
                let tail = expression.get(index..)?;
                if tail.starts_with("<=") {
                    return Some((index, 2, GridOptimizedR1C1ComparisonOp::LessThanOrEqual));
                }
                if tail.starts_with(">=") {
                    return Some((index, 2, GridOptimizedR1C1ComparisonOp::GreaterThanOrEqual));
                }
                if tail.starts_with("<>") {
                    return Some((index, 2, GridOptimizedR1C1ComparisonOp::NotEqual));
                }
                if tail.starts_with('<') {
                    return Some((index, 1, GridOptimizedR1C1ComparisonOp::LessThan));
                }
                if tail.starts_with('>') {
                    return Some((index, 1, GridOptimizedR1C1ComparisonOp::GreaterThan));
                }
                if tail.starts_with('=') {
                    return Some((index, 1, GridOptimizedR1C1ComparisonOp::Equal));
                }
            }
            _ => {}
        }
    }
    None
}

fn find_r1c1_range_separator(expression: &str) -> Option<usize> {
    let mut bracket_depth = 0_u32;
    for (index, ch) in expression.char_indices() {
        match ch {
            '[' => bracket_depth = bracket_depth.saturating_add(1),
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ':' if bracket_depth == 0 && index > 0 => return Some(index),
            _ => {}
        }
    }
    None
}

fn compile_r1c1_binary_expression(expression: &str) -> Option<GridOptimizedR1C1BinaryPlan> {
    let (operator_index, op) = find_r1c1_binary_operator(expression)?;
    let left = expression.get(..operator_index)?;
    let right = expression.get(operator_index + 1..)?;
    Some(GridOptimizedR1C1BinaryPlan {
        left: Box::new(parse_r1c1_scalar_expression(left)?),
        op,
        right: Box::new(parse_r1c1_scalar_expression(right)?),
    })
}

fn compile_r1c1_unary_minus_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1UnaryMinusPlan> {
    let value = expression.strip_prefix('-')?;
    if value.is_empty() {
        return None;
    }
    Some(GridOptimizedR1C1UnaryMinusPlan {
        value: Box::new(parse_r1c1_scalar_expression(value)?),
    })
}

fn find_r1c1_binary_operator(expression: &str) -> Option<(usize, GridOptimizedR1C1BinaryOp)> {
    find_r1c1_binary_operator_in_precedence_group(
        expression,
        &[
            ('+', GridOptimizedR1C1BinaryOp::Add),
            ('-', GridOptimizedR1C1BinaryOp::Subtract),
        ],
    )
    .or_else(|| {
        find_r1c1_binary_operator_in_precedence_group(
            expression,
            &[
                ('*', GridOptimizedR1C1BinaryOp::Multiply),
                ('/', GridOptimizedR1C1BinaryOp::Divide),
            ],
        )
    })
}

fn find_r1c1_binary_operator_in_precedence_group(
    expression: &str,
    candidates: &[(char, GridOptimizedR1C1BinaryOp)],
) -> Option<(usize, GridOptimizedR1C1BinaryOp)> {
    let mut bracket_depth = 0_u32;
    let mut paren_depth = 0_u32;
    for (index, ch) in expression.char_indices().rev() {
        match ch {
            ']' => bracket_depth = bracket_depth.saturating_add(1),
            '[' => bracket_depth = bracket_depth.saturating_sub(1),
            ')' if bracket_depth == 0 => paren_depth = paren_depth.saturating_add(1),
            '(' if bracket_depth == 0 => paren_depth = paren_depth.saturating_sub(1),
            _ if bracket_depth == 0 && paren_depth == 0 && index > 0 => {
                if let Some((_, op)) = candidates.iter().find(|(candidate, _)| *candidate == ch) {
                    if ch == '-' && is_unary_r1c1_minus(expression, index) {
                        continue;
                    }
                    return Some((index, *op));
                }
            }
            _ => {}
        }
    }
    None
}

fn is_unary_r1c1_minus(expression: &str, index: usize) -> bool {
    expression
        .get(..index)
        .and_then(|prefix| prefix.chars().next_back())
        .is_none_or(|previous| matches!(previous, '+' | '-' | '*' | '/' | '(' | ','))
}

fn parse_r1c1_operand(text: &str) -> Option<GridOptimizedR1C1Operand> {
    if let Ok(value) = text.parse::<f64>() {
        return GridOptimizedR1C1NumberLiteral::new(value).map(GridOptimizedR1C1Operand::Number);
    }
    parse_r1c1_reference_operand(text)
}

fn parse_r1c1_reference_operand(text: &str) -> Option<GridOptimizedR1C1Operand> {
    parse_r1c1_reference(text).map(GridOptimizedR1C1Operand::Ref)
}

fn parse_r1c1_reference(text: &str) -> Option<GridOptimizedR1C1Ref> {
    let chars = text.as_bytes();
    if chars.first().copied()? != b'R' {
        return None;
    }
    let mut index = 1_usize;
    let row = parse_r1c1_axis_ref(text, &mut index)?;
    if chars.get(index).copied()? != b'C' {
        return None;
    }
    index += 1;
    let col = parse_r1c1_axis_ref(text, &mut index)?;
    if index != text.len() {
        return None;
    }
    Some(GridOptimizedR1C1Ref { row, col })
}

fn parse_r1c1_axis_ref(text: &str, index: &mut usize) -> Option<GridOptimizedR1C1AxisRef> {
    let bytes = text.as_bytes();
    match bytes.get(*index).copied() {
        Some(b'[') => {
            let start = *index + 1;
            let end = text.get(start..)?.find(']')?.saturating_add(start);
            let value = text.get(start..end)?.parse::<i32>().ok()?;
            *index = end + 1;
            Some(GridOptimizedR1C1AxisRef::Relative(value))
        }
        Some(ch) if ch.is_ascii_digit() => {
            let start = *index;
            while bytes
                .get(*index)
                .copied()
                .is_some_and(|ch| ch.is_ascii_digit())
            {
                *index += 1;
            }
            let value = text.get(start..*index)?.parse::<u32>().ok()?;
            if value == 0 {
                return None;
            }
            Some(GridOptimizedR1C1AxisRef::Absolute(value))
        }
        _ => Some(GridOptimizedR1C1AxisRef::Relative(0)),
    }
}

fn add_i32_to_u32(value: u32, delta: i32) -> Option<u32> {
    let result = i64::from(value).checked_add(i64::from(delta))?;
    u32::try_from(result).ok().filter(|result| *result >= 1)
}

fn evaluate_optimized_formula_fast_path(
    address: &ExcelGridCellAddress,
    formula: &GridFormulaCell,
    valuation: &GridOptimizedValuation,
) -> Option<CalcValue> {
    GridOptimizedCompiledFormulaPlan::compile(formula)?.evaluate_single_cell(address, valuation)
}

fn number_from_calc_value(value: &CalcValue) -> Option<f64> {
    match value.core {
        CoreValue::Number(number) => Some(number),
        _ => None,
    }
}

fn aggregate_optimized_r1c1_rect(
    function: GridOptimizedR1C1RangeAggregateFunction,
    top_row: u32,
    left_col: u32,
    bottom_row: u32,
    right_col: u32,
    region: Option<&GridRepeatedFormulaRegion>,
    row_major_formula_values: &[CalcValue],
    valuation: &GridOptimizedValuation,
) -> Option<CalcValue> {
    let mut state = GridOptimizedR1C1AggregateState::new();
    match accumulate_optimized_r1c1_rect(
        function,
        top_row,
        left_col,
        bottom_row,
        right_col,
        region,
        row_major_formula_values,
        valuation,
        &mut state,
    )? {
        Ok(()) => Some(state.finish(function)),
        Err(error) => Some(error),
    }
}

fn accumulate_optimized_r1c1_rect(
    function: GridOptimizedR1C1RangeAggregateFunction,
    top_row: u32,
    left_col: u32,
    bottom_row: u32,
    right_col: u32,
    region: Option<&GridRepeatedFormulaRegion>,
    row_major_formula_values: &[CalcValue],
    valuation: &GridOptimizedValuation,
    state: &mut GridOptimizedR1C1AggregateState,
) -> Option<Result<(), CalcValue>> {
    for row in top_row..=bottom_row {
        for col in left_col..=right_col {
            let value = optimized_r1c1_value_for_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            )?;
            if let Err(error) = state.accumulate(function, value) {
                return Some(Err(error));
            }
        }
    }
    Some(Ok(()))
}

#[derive(Debug, Clone, Copy)]
struct GridOptimizedR1C1AggregateState {
    sum: f64,
    sum_sq: f64,
    product: f64,
    count: u64,
    extreme: Option<f64>,
}

impl GridOptimizedR1C1AggregateState {
    const fn new() -> Self {
        Self {
            sum: 0.0,
            sum_sq: 0.0,
            product: 1.0,
            count: 0,
            extreme: None,
        }
    }

    fn accumulate(
        &mut self,
        function: GridOptimizedR1C1RangeAggregateFunction,
        value: GridOptimizedR1C1Value,
    ) -> Result<(), CalcValue> {
        let number = match value {
            GridOptimizedR1C1Value::Number(number) => number,
            GridOptimizedR1C1Value::Error(error) => return Err(CalcValue::error(error)),
        };
        self.sum += number;
        self.sum_sq += number * number;
        self.product *= number;
        self.count = self.count.saturating_add(1);
        self.extreme = Some(match (function, self.extreme) {
            (GridOptimizedR1C1RangeAggregateFunction::Min, Some(current)) => current.min(number),
            (GridOptimizedR1C1RangeAggregateFunction::Max, Some(current)) => current.max(number),
            (GridOptimizedR1C1RangeAggregateFunction::Min, None)
            | (GridOptimizedR1C1RangeAggregateFunction::Max, None) => number,
            (_, current) => current.unwrap_or(number),
        });
        Ok(())
    }

    fn finish(self, function: GridOptimizedR1C1RangeAggregateFunction) -> CalcValue {
        match function {
            GridOptimizedR1C1RangeAggregateFunction::Sum => CalcValue::number(self.sum),
            GridOptimizedR1C1RangeAggregateFunction::SumSq => CalcValue::number(self.sum_sq),
            GridOptimizedR1C1RangeAggregateFunction::Count => CalcValue::number(self.count as f64),
            GridOptimizedR1C1RangeAggregateFunction::Product if self.count == 0 => {
                CalcValue::number(0.0)
            }
            GridOptimizedR1C1RangeAggregateFunction::Product => CalcValue::number(self.product),
            GridOptimizedR1C1RangeAggregateFunction::Average if self.count == 0 => {
                CalcValue::error(WorksheetErrorCode::Div0)
            }
            GridOptimizedR1C1RangeAggregateFunction::Average => {
                CalcValue::number(self.sum / self.count as f64)
            }
            GridOptimizedR1C1RangeAggregateFunction::Min
            | GridOptimizedR1C1RangeAggregateFunction::Max => {
                CalcValue::number(self.extreme.unwrap_or(0.0))
            }
        }
    }
}

fn optimized_r1c1_value_for_cell(
    row: u32,
    col: u32,
    region: Option<&GridRepeatedFormulaRegion>,
    row_major_formula_values: &[CalcValue],
    valuation: &GridOptimizedValuation,
) -> Option<GridOptimizedR1C1Value> {
    let input =
        optimized_r1c1_calc_value_for_cell(row, col, region, row_major_formula_values, valuation)?;
    optimized_r1c1_value_from_calc_value(&input)
}

fn optimized_r1c1_calc_value_for_cell(
    row: u32,
    col: u32,
    region: Option<&GridRepeatedFormulaRegion>,
    row_major_formula_values: &[CalcValue],
    valuation: &GridOptimizedValuation,
) -> Option<CalcValue> {
    if let Some(region) = region {
        if region.rect.contains(&ExcelGridCellAddress::new(
            region.rect.workbook_id.clone(),
            region.rect.sheet_id.clone(),
            row,
            col,
        )) {
            let col_count = usize::try_from(region.rect.col_count()).ok()?;
            let row_offset = usize::try_from(row - region.rect.top_row).ok()?;
            let col_offset = usize::try_from(col - region.rect.left_col).ok()?;
            let index = row_offset.checked_mul(col_count)?.checked_add(col_offset)?;
            return row_major_formula_values.get(index).cloned();
        }
    }
    Some(
        valuation
            .read_cell(&ExcelGridCellAddress::new(
                valuation.workbook_id.clone(),
                valuation.sheet_id.clone(),
                row,
                col,
            ))
            .computed,
    )
}

fn optimized_r1c1_value_from_calc_value(value: &CalcValue) -> Option<GridOptimizedR1C1Value> {
    match value.core {
        CoreValue::Number(number) => Some(GridOptimizedR1C1Value::Number(number)),
        CoreValue::Error(error) => Some(GridOptimizedR1C1Value::Error(error)),
        _ => None,
    }
}

fn optimized_r1c1_calc_value_from_value(value: GridOptimizedR1C1Value) -> Option<CalcValue> {
    match value {
        GridOptimizedR1C1Value::Number(number) => Some(CalcValue::number(number)),
        GridOptimizedR1C1Value::Error(error) => Some(CalcValue::error(error)),
    }
}

fn optimized_r1c1_text_from_calc_value(value: CalcValue) -> Result<ExcelText, CalcValue> {
    match value.core {
        CoreValue::Text(text) => Ok(text),
        CoreValue::Error(error) => Err(CalcValue::error(error)),
        _ => Err(CalcValue::error(WorksheetErrorCode::Value)),
    }
}

fn optimized_r1c1_text_count_from_value(value: GridOptimizedR1C1Value) -> Result<usize, CalcValue> {
    match value {
        GridOptimizedR1C1Value::Number(number) if number.is_finite() && number >= 0.0 => {
            if number > usize::MAX as f64 {
                return Err(CalcValue::error(WorksheetErrorCode::Value));
            }
            Ok(number.trunc() as usize)
        }
        GridOptimizedR1C1Value::Number(_) => Err(CalcValue::error(WorksheetErrorCode::Value)),
        GridOptimizedR1C1Value::Error(error) => Err(CalcValue::error(error)),
    }
}

fn optimized_r1c1_index_from_value(value: GridOptimizedR1C1Value) -> Result<usize, CalcValue> {
    match value {
        GridOptimizedR1C1Value::Number(number) if number.is_finite() && number >= 1.0 => {
            if number > usize::MAX as f64 {
                return Err(CalcValue::error(WorksheetErrorCode::Ref));
            }
            Ok(number.trunc() as usize)
        }
        GridOptimizedR1C1Value::Number(_) => Err(CalcValue::error(WorksheetErrorCode::Value)),
        GridOptimizedR1C1Value::Error(error) => Err(CalcValue::error(error)),
    }
}

fn optimized_r1c1_text_slice(
    text: ExcelText,
    count: usize,
    side: GridOptimizedR1C1TextSliceSide,
) -> CalcValue {
    let units = text.utf16_code_units();
    let count = count.min(units.len());
    let slice = match side {
        GridOptimizedR1C1TextSliceSide::Left => units[..count].to_vec(),
        GridOptimizedR1C1TextSliceSide::Right => {
            units[units.len().saturating_sub(count)..].to_vec()
        }
    };
    CalcValue::text(ExcelText::from_utf16_code_units(slice))
}

fn negate_optimized_r1c1_value(value: GridOptimizedR1C1Value) -> CalcValue {
    match value {
        GridOptimizedR1C1Value::Number(number) => CalcValue::number(-number),
        GridOptimizedR1C1Value::Error(error) => CalcValue::error(error),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GridAxis {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridAxisEditKind {
    Insert { before: u32, count: u32 },
    Delete { first: u32, count: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridAxisEdit {
    pub axis: GridAxis,
    pub kind: GridAxisEditKind,
}

impl GridAxisEdit {
    #[must_use]
    pub const fn insert_rows(before: u32, count: u32) -> Self {
        Self {
            axis: GridAxis::Row,
            kind: GridAxisEditKind::Insert { before, count },
        }
    }

    #[must_use]
    pub const fn delete_rows(first: u32, count: u32) -> Self {
        Self {
            axis: GridAxis::Row,
            kind: GridAxisEditKind::Delete { first, count },
        }
    }

    #[must_use]
    pub const fn insert_columns(before: u32, count: u32) -> Self {
        Self {
            axis: GridAxis::Column,
            kind: GridAxisEditKind::Insert { before, count },
        }
    }

    #[must_use]
    pub const fn delete_columns(first: u32, count: u32) -> Self {
        Self {
            axis: GridAxis::Column,
            kind: GridAxisEditKind::Delete { first, count },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridStructuralTransformOutcome {
    Unchanged,
    Shifted,
    Expanded,
    Shrunk,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridStructuralEditReport {
    pub edit: GridAxisEdit,
    pub authored_cells_kept: usize,
    pub authored_cells_dropped: usize,
    pub formula_cells_transformed: usize,
    pub formula_reference_transforms: usize,
    pub computed_cells_kept: usize,
    pub computed_cells_dropped: usize,
    pub spill_facts_kept: usize,
    pub spill_facts_dropped: usize,
    pub merged_regions_kept: usize,
    pub merged_regions_dropped: usize,
    pub feature_regions_kept: usize,
    pub feature_regions_dropped: usize,
    pub feature_regions_marked_needs_refresh: usize,
    pub axis_entries_kept: usize,
    pub axis_entries_dropped: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridCalcRefSheet {
    workbook_id: String,
    sheet_id: String,
    bounds: ExcelGridBounds,
    authored: BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
    computed: BTreeMap<ExcelGridCellAddress, CalcValue>,
    axis_state: GridAxisState,
    merged_regions: Vec<GridMergedRegion>,
    feature_rendered_regions: Vec<FeatureRenderedRegion>,
    spill_facts: BTreeMap<ExcelGridCellAddress, GridSpillFact>,
    spill_value_fingerprints: BTreeMap<ExcelGridCellAddress, String>,
    spill_epoch_ledger: GridSpillEpochLedger,
    defined_names: BTreeMap<String, GridRect>,
    table_overlays: BTreeMap<String, GridTableOverlay>,
}

impl GridCalcRefSheet {
    #[must_use]
    pub fn new(
        workbook_id: impl Into<String>,
        sheet_id: impl Into<String>,
        bounds: ExcelGridBounds,
    ) -> Self {
        Self {
            workbook_id: workbook_id.into(),
            sheet_id: sheet_id.into(),
            bounds,
            authored: BTreeMap::new(),
            computed: BTreeMap::new(),
            axis_state: GridAxisState::default(),
            merged_regions: Vec::new(),
            feature_rendered_regions: Vec::new(),
            spill_facts: BTreeMap::new(),
            spill_value_fingerprints: BTreeMap::new(),
            spill_epoch_ledger: GridSpillEpochLedger::default(),
            defined_names: BTreeMap::new(),
            table_overlays: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn strict_excel(workbook_id: impl Into<String>, sheet_id: impl Into<String>) -> Self {
        Self::new(workbook_id, sheet_id, ExcelGridBounds::strict_excel())
    }

    #[must_use]
    pub fn workbook_id(&self) -> &str {
        &self.workbook_id
    }

    #[must_use]
    pub fn sheet_id(&self) -> &str {
        &self.sheet_id
    }

    #[must_use]
    pub const fn bounds(&self) -> ExcelGridBounds {
        self.bounds
    }

    #[must_use]
    pub fn address(&self, row: u32, col: u32) -> Result<ExcelGridCellAddress, GridRefError> {
        if !self.bounds.contains_row(row) || !self.bounds.contains_col(col) {
            return Err(GridRefError::AddressOutOfBounds {
                row,
                col,
                max_rows: self.bounds.max_rows,
                max_cols: self.bounds.max_cols,
            });
        }
        Ok(ExcelGridCellAddress::new(
            self.workbook_id.clone(),
            self.sheet_id.clone(),
            row,
            col,
        ))
    }

    pub fn set_literal(
        &mut self,
        address: ExcelGridCellAddress,
        value: CalcValue,
    ) -> Result<(), GridRefError> {
        self.check_address(&address)?;
        self.authored
            .insert(address, GridAuthoredCell::Literal(value));
        Ok(())
    }

    pub fn set_formula(
        &mut self,
        address: ExcelGridCellAddress,
        formula: GridFormulaCell,
    ) -> Result<(), GridRefError> {
        self.check_address(&address)?;
        self.authored
            .insert(address, GridAuthoredCell::Formula(formula));
        Ok(())
    }

    pub fn materialize_formula_region(
        &mut self,
        rect: GridRect,
        formula: GridFormulaCell,
    ) -> Result<GridRegionMaterializationReport, GridRefError> {
        self.materialize_formula_region_with_limit(
            rect,
            formula,
            GRID_CALC_REF_DEFAULT_MATERIALIZATION_LIMIT,
        )
    }

    pub fn materialize_formula_region_with_limit(
        &mut self,
        rect: GridRect,
        formula: GridFormulaCell,
        limit: u64,
    ) -> Result<GridRegionMaterializationReport, GridRefError> {
        rect.check_sheet(self)?;
        let cells = rect.scalar_cells(limit)?;
        for address in &cells {
            self.authored
                .insert(address.clone(), GridAuthoredCell::Formula(formula.clone()));
        }
        Ok(GridRegionMaterializationReport {
            cells_written: cells.len(),
            rect,
        })
    }

    pub fn materialize_literal_region<F>(
        &mut self,
        rect: GridRect,
        make_value: F,
    ) -> Result<GridRegionMaterializationReport, GridRefError>
    where
        F: FnMut(&ExcelGridCellAddress) -> CalcValue,
    {
        self.materialize_literal_region_with_limit(
            rect,
            make_value,
            GRID_CALC_REF_DEFAULT_MATERIALIZATION_LIMIT,
        )
    }

    pub fn materialize_literal_region_with_limit<F>(
        &mut self,
        rect: GridRect,
        mut make_value: F,
        limit: u64,
    ) -> Result<GridRegionMaterializationReport, GridRefError>
    where
        F: FnMut(&ExcelGridCellAddress) -> CalcValue,
    {
        rect.check_sheet(self)?;
        let cells = rect.scalar_cells(limit)?;
        for address in &cells {
            self.authored.insert(
                address.clone(),
                GridAuthoredCell::Literal(make_value(address)),
            );
        }
        Ok(GridRegionMaterializationReport {
            cells_written: cells.len(),
            rect,
        })
    }

    pub fn clear_cell(&mut self, address: &ExcelGridCellAddress) -> Result<(), GridRefError> {
        self.check_address(address)?;
        self.authored.remove(address);
        self.computed.remove(address);
        self.spill_facts.remove(address);
        self.spill_value_fingerprints.remove(address);
        self.refresh_spill_epoch_ledger();
        Ok(())
    }

    #[must_use]
    pub fn authored(&self) -> &BTreeMap<ExcelGridCellAddress, GridAuthoredCell> {
        &self.authored
    }

    #[must_use]
    pub fn computed(&self) -> &BTreeMap<ExcelGridCellAddress, CalcValue> {
        &self.computed
    }

    #[must_use]
    pub fn axis_state(&self) -> &GridAxisState {
        &self.axis_state
    }

    #[must_use]
    pub fn axis_state_mut(&mut self) -> &mut GridAxisState {
        &mut self.axis_state
    }

    #[must_use]
    pub fn merged_regions(&self) -> &[GridMergedRegion] {
        &self.merged_regions
    }

    pub fn add_merged_region(&mut self, rect: GridRect) {
        self.merged_regions.push(GridMergedRegion { rect });
    }

    #[must_use]
    pub fn feature_rendered_regions(&self) -> &[FeatureRenderedRegion] {
        &self.feature_rendered_regions
    }

    pub fn add_feature_rendered_region(
        &mut self,
        rect: GridRect,
        feature_kind: impl Into<String>,
        needs_refresh: bool,
    ) {
        self.feature_rendered_regions.push(FeatureRenderedRegion {
            rect,
            feature_kind: feature_kind.into(),
            needs_refresh,
        });
    }

    #[must_use]
    pub fn spill_facts(&self) -> &BTreeMap<ExcelGridCellAddress, GridSpillFact> {
        &self.spill_facts
    }

    #[must_use]
    pub fn spill_epoch_ledger(&self) -> &GridSpillEpochLedger {
        &self.spill_epoch_ledger
    }

    #[must_use]
    pub fn spill_epoch_snapshots(&self) -> BTreeMap<ExcelGridCellAddress, GridSpillEpochSnapshot> {
        self.spill_epoch_ledger.snapshots()
    }

    pub fn set_spill_fact(&mut self, fact: GridSpillFact) -> Result<(), GridRefError> {
        self.check_address(&fact.anchor)?;
        fact.extent.check_sheet(self)?;
        self.spill_value_fingerprints.insert(
            fact.anchor.clone(),
            manual_spill_fact_value_fingerprint(&fact),
        );
        self.spill_facts.insert(fact.anchor.clone(), fact);
        self.refresh_spill_epoch_ledger();
        Ok(())
    }

    pub fn refresh_spill_epoch_ledger(&mut self) -> GridSpillEpochLedgerUpdateReport {
        let fingerprints = self.spill_value_fingerprints.clone();
        self.spill_epoch_ledger
            .update_from_spill_facts(&self.spill_facts, |fact| {
                fingerprints
                    .get(&fact.anchor)
                    .cloned()
                    .unwrap_or_else(|| manual_spill_fact_value_fingerprint(fact))
            })
    }

    #[must_use]
    pub fn defined_names(&self) -> &BTreeMap<String, GridRect> {
        &self.defined_names
    }

    pub fn set_defined_name(
        &mut self,
        name: impl AsRef<str>,
        rect: GridRect,
    ) -> Result<(), GridRefError> {
        rect.check_sheet(self)?;
        let name_key = defined_name_key_for_name(name.as_ref(), self.bounds)?;
        self.defined_names.insert(name_key, rect);
        Ok(())
    }

    pub fn rename_defined_name(
        &mut self,
        old_name: impl AsRef<str>,
        new_name: impl AsRef<str>,
    ) -> Result<GridNameLifecycleReport, GridRefError> {
        let old_name = old_name.as_ref();
        let new_name = new_name.as_ref();
        let old_key = defined_name_key_for_name(old_name, self.bounds)?;
        let new_key = defined_name_key_for_name(new_name, self.bounds)?;
        if old_key != new_key && self.defined_names.contains_key(&new_key) {
            return Err(GridRefError::DefinedNameAlreadyExists {
                name: new_name.to_string(),
            });
        }
        let Some(rect) = self.defined_names.remove(&old_key) else {
            return Err(GridRefError::DefinedNameNotFound {
                name: old_name.to_string(),
            });
        };
        self.defined_names.insert(new_key.clone(), rect);
        let stats = transform_authored_formulas_for_defined_name_rename(
            &mut self.authored,
            &old_key,
            new_name,
            self.bounds,
        )?;
        Ok(GridNameLifecycleReport {
            operation: GridNameLifecycleOperation::Rename,
            old_name_key: Some(old_key),
            new_name_key: Some(new_key),
            formula_cells_transformed: stats.formula_cells_transformed,
            formula_reference_transforms: stats.formula_reference_transforms,
        })
    }

    pub fn delete_defined_name(
        &mut self,
        name: impl AsRef<str>,
    ) -> Result<GridNameLifecycleReport, GridRefError> {
        let name = name.as_ref();
        let name_key = defined_name_key_for_name(name, self.bounds)?;
        if self.defined_names.remove(&name_key).is_none() {
            return Err(GridRefError::DefinedNameNotFound {
                name: name.to_string(),
            });
        }
        let stats = transform_authored_formulas_for_defined_name_delete(
            &mut self.authored,
            &name_key,
            self.bounds,
        )?;
        Ok(GridNameLifecycleReport {
            operation: GridNameLifecycleOperation::Delete,
            old_name_key: Some(name_key),
            new_name_key: None,
            formula_cells_transformed: stats.formula_cells_transformed,
            formula_reference_transforms: stats.formula_reference_transforms,
        })
    }

    #[must_use]
    pub fn table_overlays(&self) -> &BTreeMap<String, GridTableOverlay> {
        &self.table_overlays
    }

    pub fn set_table_overlay(&mut self, table: GridTableOverlay) -> Result<(), GridRefError> {
        table.check_sheet(&self.workbook_id, &self.sheet_id, self.bounds)?;
        let table_key = table_key_for_name(&table.table_name, self.bounds)?;
        let table_range = table.table_range.clone();
        if let Some(old_table) = self.table_overlays.get(&table_key) {
            remove_table_overlay_feature_regions(
                &mut self.feature_rendered_regions,
                &old_table.table_range,
            );
        }
        self.table_overlays.insert(table_key, table);
        self.add_feature_rendered_region(table_range, "table-overlay", false);
        Ok(())
    }

    pub fn resize_table_overlay(
        &mut self,
        table: GridTableOverlay,
    ) -> Result<GridTableLifecycleReport, GridRefError> {
        table.check_sheet(&self.workbook_id, &self.sheet_id, self.bounds)?;
        let table_key = table_key_for_name(&table.table_name, self.bounds)?;
        let Some(old_table) = self.table_overlays.get(&table_key) else {
            return Err(GridRefError::TableOverlayNotFound {
                name: table.table_name,
            });
        };
        let feature_regions_removed = remove_table_overlay_feature_regions(
            &mut self.feature_rendered_regions,
            &old_table.table_range,
        );
        let table_range = table.table_range.clone();
        self.table_overlays.insert(table_key.clone(), table);
        self.add_feature_rendered_region(table_range, "table-overlay", false);
        Ok(GridTableLifecycleReport {
            operation: GridTableLifecycleOperation::Resize,
            old_table_key: Some(table_key.clone()),
            new_table_key: Some(table_key),
            feature_regions_removed,
            feature_regions_added: 1,
            formula_cells_transformed: 0,
            formula_reference_transforms: 0,
        })
    }

    pub fn rename_table_overlay(
        &mut self,
        old_name: impl AsRef<str>,
        new_name: impl AsRef<str>,
    ) -> Result<GridTableLifecycleReport, GridRefError> {
        let old_name = old_name.as_ref();
        let new_name = new_name.as_ref();
        let old_key = table_key_for_name(old_name, self.bounds)?;
        let new_key = table_key_for_name(new_name, self.bounds)?;
        if old_key != new_key && self.table_overlays.contains_key(&new_key) {
            return Err(GridRefError::TableOverlayAlreadyExists {
                name: new_name.to_string(),
            });
        }
        let Some(mut table) = self.table_overlays.remove(&old_key) else {
            return Err(GridRefError::TableOverlayNotFound {
                name: old_name.to_string(),
            });
        };
        table.table_name = new_name.to_string();
        self.table_overlays.insert(new_key.clone(), table);
        let stats = transform_authored_formulas_for_table_rename(
            &mut self.authored,
            &old_key,
            new_name,
            self.bounds,
        )?;
        Ok(GridTableLifecycleReport {
            operation: GridTableLifecycleOperation::Rename,
            old_table_key: Some(old_key),
            new_table_key: Some(new_key),
            feature_regions_removed: 0,
            feature_regions_added: 0,
            formula_cells_transformed: stats.formula_cells_transformed,
            formula_reference_transforms: stats.formula_reference_transforms,
        })
    }

    pub fn delete_table_overlay(
        &mut self,
        table_name: impl AsRef<str>,
    ) -> Result<GridTableLifecycleReport, GridRefError> {
        let table_name = table_name.as_ref();
        let table_key = table_key_for_name(table_name, self.bounds)?;
        let Some(table) = self.table_overlays.remove(&table_key) else {
            return Err(GridRefError::TableOverlayNotFound {
                name: table_name.to_string(),
            });
        };
        let feature_regions_removed = remove_table_overlay_feature_regions(
            &mut self.feature_rendered_regions,
            &table.table_range,
        );
        let stats = transform_authored_formulas_for_table_delete(
            &mut self.authored,
            &table_key,
            self.bounds,
        )?;
        Ok(GridTableLifecycleReport {
            operation: GridTableLifecycleOperation::Delete,
            old_table_key: Some(table_key),
            new_table_key: None,
            feature_regions_removed,
            feature_regions_added: 0,
            formula_cells_transformed: stats.formula_cells_transformed,
            formula_reference_transforms: stats.formula_reference_transforms,
        })
    }

    pub fn apply_axis_edit(
        &mut self,
        edit: GridAxisEdit,
    ) -> Result<GridStructuralEditReport, GridRefError> {
        validate_axis_edit(edit, self.bounds)?;
        let feature_region_transform = transform_feature_rendered_regions_for_axis_edit(
            &self.feature_rendered_regions,
            edit,
            self.bounds,
        )?;

        let (
            authored,
            authored_cells_kept,
            authored_cells_dropped,
            formula_cells_transformed,
            formula_reference_transforms,
        ) = transform_authored_cell_map_for_edit(
            std::mem::take(&mut self.authored),
            edit,
            self.bounds,
        )?;
        self.authored = authored;

        let (computed, computed_cells_kept, computed_cells_dropped) =
            transform_cell_map(std::mem::take(&mut self.computed), edit, self.bounds)?;
        self.computed = computed;

        let (axis_entries_kept, axis_entries_dropped) =
            self.axis_state.apply_axis_edit(edit, self.bounds)?;

        let old_spills = std::mem::take(&mut self.spill_facts);
        let mut spill_facts_kept = 0;
        let mut spill_facts_dropped = 0;
        for fact in old_spills.into_values() {
            let Some(anchor) = transform_address_for_edit(&fact.anchor, edit, self.bounds)? else {
                spill_facts_dropped += 1;
                continue;
            };
            let (Some(extent), _) = transform_rect_for_edit(&fact.extent, edit, self.bounds)?
            else {
                spill_facts_dropped += 1;
                continue;
            };
            let transformed = GridSpillFact {
                anchor: anchor.clone(),
                extent,
                blocked: fact.blocked,
            };
            self.spill_facts.insert(anchor, transformed);
            spill_facts_kept += 1;
        }
        self.spill_value_fingerprints = transform_spill_value_fingerprints_for_edit(
            std::mem::take(&mut self.spill_value_fingerprints),
            edit,
            self.bounds,
        )?;
        self.refresh_spill_epoch_ledger();

        let old_defined_names = std::mem::take(&mut self.defined_names);
        for (name_key, rect) in old_defined_names {
            let (Some(rect), _) = transform_rect_for_edit(&rect, edit, self.bounds)? else {
                continue;
            };
            self.defined_names.insert(name_key, rect);
        }

        let old_table_overlays = std::mem::take(&mut self.table_overlays);
        for (table_key, table) in old_table_overlays {
            let Some(table) = table.transform_for_axis_edit(edit, self.bounds)? else {
                continue;
            };
            self.table_overlays.insert(table_key, table);
        }

        let old_merged_regions = std::mem::take(&mut self.merged_regions);
        let mut merged_regions_kept = 0;
        let mut merged_regions_dropped = 0;
        for region in old_merged_regions {
            let (Some(rect), _) = transform_rect_for_edit(&region.rect, edit, self.bounds)? else {
                merged_regions_dropped += 1;
                continue;
            };
            self.merged_regions.push(GridMergedRegion { rect });
            merged_regions_kept += 1;
        }

        self.feature_rendered_regions = feature_region_transform.regions;

        Ok(GridStructuralEditReport {
            edit,
            authored_cells_kept,
            authored_cells_dropped,
            formula_cells_transformed,
            formula_reference_transforms,
            computed_cells_kept,
            computed_cells_dropped,
            spill_facts_kept,
            spill_facts_dropped,
            merged_regions_kept,
            merged_regions_dropped,
            feature_regions_kept: feature_region_transform.kept,
            feature_regions_dropped: feature_region_transform.dropped,
            feature_regions_marked_needs_refresh: feature_region_transform.marked_needs_refresh,
            axis_entries_kept,
            axis_entries_dropped,
        })
    }

    #[must_use]
    pub fn read_cell(&self, address: &ExcelGridCellAddress) -> CalcValue {
        self.computed
            .get(address)
            .cloned()
            .unwrap_or_else(CalcValue::empty)
    }

    #[must_use]
    pub fn sampled_readout(
        &self,
        addresses: impl IntoIterator<Item = ExcelGridCellAddress>,
    ) -> Vec<GridCalcRefCellReadout> {
        addresses
            .into_iter()
            .map(|address| GridCalcRefCellReadout {
                computed: self.read_cell(&address),
                authored: self.authored.get(&address).cloned(),
                spill_anchor: self.spill_anchor_for(&address),
                address,
            })
            .collect()
    }

    pub fn recalculate_mark_all_dirty<F>(
        &mut self,
        mut evaluate_formula: F,
    ) -> GridCalcRefRecalcReport
    where
        F: FnMut(GridFormulaEvaluationRequest<'_>) -> CalcValue,
    {
        let previous_computed = self.computed.clone();
        let authored = self.authored.clone();
        self.computed.clear();
        self.clear_formula_spill_facts(&authored);

        let mut report = GridCalcRefRecalcReport {
            occupied_cells: authored.len(),
            literal_cells: 0,
            formula_cells: 0,
            cells_evaluated: 0,
            formula_evaluations: 0,
            spill_repair_passes: 0,
            spill_repair_formula_evaluations: 0,
            spill_repair_converged: true,
            spill_facts_published: 0,
            spill_facts_blocked: 0,
            spill_ghost_cells_published: 0,
            visited_cells: Vec::with_capacity(authored.len()),
        };

        for (address, cell) in &authored {
            report.cells_evaluated += 1;
            report.visited_cells.push(address.clone());
            match cell {
                GridAuthoredCell::Literal(value) => {
                    report.literal_cells += 1;
                    self.computed.insert(address.clone(), value.clone());
                }
                GridAuthoredCell::Formula(formula) => {
                    report.formula_cells += 1;
                    report.formula_evaluations += 1;
                    let value = evaluate_formula(GridFormulaEvaluationRequest {
                        address,
                        formula,
                        authored: &authored,
                        previous_computed: &previous_computed,
                    });
                    let spill_counters =
                        self.publish_formula_value(address.clone(), value, &authored);
                    report.spill_facts_published += spill_counters.facts_published;
                    report.spill_facts_blocked += spill_counters.facts_blocked;
                    report.spill_ghost_cells_published += spill_counters.ghost_cells_published;
                }
            }
        }

        self.refresh_reference_report_spill_counters(&mut report, &authored);
        self.refresh_spill_epoch_ledger();
        report
    }

    pub fn recalculate_mark_all_dirty_with_oxfml(
        &mut self,
    ) -> Result<GridCalcRefRecalcReport, GridRefError> {
        let authored = self.authored.clone();
        self.computed.clear();
        self.clear_formula_spill_facts(&authored);
        let base_spill_facts = self.spill_facts.clone();

        let mut report = GridCalcRefRecalcReport {
            occupied_cells: authored.len(),
            literal_cells: 0,
            formula_cells: 0,
            cells_evaluated: 0,
            formula_evaluations: 0,
            spill_repair_passes: 0,
            spill_repair_formula_evaluations: 0,
            spill_repair_converged: true,
            spill_facts_published: 0,
            spill_facts_blocked: 0,
            spill_ghost_cells_published: 0,
            visited_cells: Vec::with_capacity(authored.len()),
        };

        for (address, cell) in &authored {
            if let GridAuthoredCell::Literal(value) = cell {
                self.computed.insert(address.clone(), value.clone());
            }
        }

        let profile = StrictExcelGridReferenceProfile::with_bounds(self.bounds);
        for (address, cell) in &authored {
            report.cells_evaluated += 1;
            report.visited_cells.push(address.clone());
            match cell {
                GridAuthoredCell::Literal(_) => {
                    report.literal_cells += 1;
                }
                GridAuthoredCell::Formula(formula) => {
                    report.formula_cells += 1;
                    report.formula_evaluations += 1;
                    let value =
                        self.evaluate_formula_with_spill_repair(address, formula, &profile)?;
                    let spill_counters =
                        self.publish_formula_value(address.clone(), value, &authored);
                    report.spill_facts_published += spill_counters.facts_published;
                    report.spill_facts_blocked += spill_counters.facts_blocked;
                    report.spill_ghost_cells_published += spill_counters.ghost_cells_published;
                }
            }
        }

        self.repair_reference_spills_with_oxfml(
            &authored,
            &profile,
            &base_spill_facts,
            &mut report,
        )?;
        self.refresh_reference_report_spill_counters(&mut report, &authored);
        self.refresh_spill_epoch_ledger();
        Ok(report)
    }

    fn repair_reference_spills_with_oxfml(
        &mut self,
        authored: &BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
        profile: &StrictExcelGridReferenceProfile,
        base_spill_facts: &BTreeMap<ExcelGridCellAddress, GridSpillFact>,
        report: &mut GridCalcRefRecalcReport,
    ) -> Result<(), GridRefError> {
        let formula_cells = formula_count(authored);
        if formula_cells == 0
            || self.spill_facts == *base_spill_facts
            || !authored_contains_grid_spill_reference(authored, profile, self.bounds)
        {
            return Ok(());
        }

        report.spill_repair_converged = false;
        for _ in 0..formula_cells {
            let spill_facts_before = self.spill_facts.clone();
            report.spill_repair_passes += 1;

            for (address, cell) in authored {
                let GridAuthoredCell::Formula(formula) = cell else {
                    continue;
                };
                report.spill_repair_formula_evaluations += 1;
                let value = self.evaluate_formula_with_spill_repair(address, formula, profile)?;
                self.publish_formula_value(address.clone(), value, authored);
            }

            if self.spill_facts == spill_facts_before {
                report.spill_repair_converged = true;
                break;
            }
        }

        Ok(())
    }

    fn evaluate_formula_with_spill_repair(
        &self,
        address: &ExcelGridCellAddress,
        formula: &GridFormulaCell,
        profile: &StrictExcelGridReferenceProfile,
    ) -> Result<CalcValue, GridRefError> {
        match self.evaluate_formula_with_oxfml(address, formula, profile) {
            Ok(value) => Ok(value),
            Err(error) => {
                if formula_contains_grid_spill_reference(formula, address, profile, self.bounds) {
                    Ok(CalcValue::error(WorksheetErrorCode::Ref))
                } else {
                    Err(error)
                }
            }
        }
    }

    fn refresh_reference_report_spill_counters(
        &self,
        report: &mut GridCalcRefRecalcReport,
        authored: &BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
    ) {
        let counters = count_formula_spill_publications(&self.spill_facts, |anchor| {
            matches!(authored.get(anchor), Some(GridAuthoredCell::Formula(_)))
        });
        report.spill_facts_published = counters.facts_published;
        report.spill_facts_blocked = counters.facts_blocked;
        report.spill_ghost_cells_published = counters.ghost_cells_published;
    }

    fn clear_formula_spill_facts(
        &mut self,
        authored: &BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
    ) {
        self.spill_facts.retain(|anchor, _| {
            !matches!(authored.get(anchor), Some(GridAuthoredCell::Formula(_)))
        });
        self.spill_value_fingerprints.retain(|anchor, _| {
            !matches!(authored.get(anchor), Some(GridAuthoredCell::Formula(_)))
        });
    }

    fn clear_formula_output_for_anchor(&mut self, anchor: &ExcelGridCellAddress) {
        if let Some(fact) = self.spill_facts.remove(anchor) {
            self.spill_value_fingerprints.remove(anchor);
            let keys = self
                .computed
                .keys()
                .filter(|address| fact.extent.contains(address))
                .cloned()
                .collect::<Vec<_>>();
            for key in keys {
                self.computed.remove(&key);
            }
        } else {
            self.spill_value_fingerprints.remove(anchor);
            self.computed.remove(anchor);
        }
    }

    fn publish_formula_value(
        &mut self,
        address: ExcelGridCellAddress,
        value: CalcValue,
        authored: &BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
    ) -> GridSpillPublicationCounters {
        self.clear_formula_output_for_anchor(&address);

        let Some(array) = value.as_array() else {
            self.computed.insert(address, value);
            return GridSpillPublicationCounters::default();
        };

        let Some(extent) = spill_extent_for_array(&address, array.shape(), self.bounds) else {
            self.computed
                .insert(address.clone(), CalcValue::error(WorksheetErrorCode::Spill));
            self.spill_value_fingerprints
                .insert(address.clone(), blocked_spill_value_fingerprint(array));
            self.spill_facts.insert(
                address.clone(),
                GridSpillFact {
                    anchor: address.clone(),
                    extent: anchor_cell_rect(&address, self.bounds),
                    blocked: true,
                },
            );
            return GridSpillPublicationCounters {
                facts_blocked: 1,
                ..GridSpillPublicationCounters::default()
            };
        };

        if self.reference_spill_extent_is_blocked(&address, &extent, authored) {
            self.computed
                .insert(address.clone(), CalcValue::error(WorksheetErrorCode::Spill));
            self.spill_value_fingerprints
                .insert(address.clone(), blocked_spill_value_fingerprint(array));
            self.spill_facts.insert(
                address.clone(),
                GridSpillFact {
                    anchor: address,
                    extent,
                    blocked: true,
                },
            );
            return GridSpillPublicationCounters {
                facts_blocked: 1,
                ..GridSpillPublicationCounters::default()
            };
        }

        let shape = array.shape();
        for row_offset in 0..shape.rows {
            for col_offset in 0..shape.cols {
                let Some(cell_address) = array_cell_address(&address, row_offset, col_offset)
                else {
                    continue;
                };
                let Some(cell_value) = array.get(row_offset, col_offset) else {
                    continue;
                };
                self.computed.insert(cell_address, cell_value.clone());
            }
        }
        self.spill_facts.insert(
            address.clone(),
            GridSpillFact {
                anchor: address.clone(),
                extent,
                blocked: false,
            },
        );
        self.spill_value_fingerprints
            .insert(address, calc_array_value_fingerprint(array));
        GridSpillPublicationCounters {
            facts_published: 1,
            ghost_cells_published: array.cell_count().saturating_sub(1),
            ..GridSpillPublicationCounters::default()
        }
    }

    fn reference_spill_extent_is_blocked(
        &self,
        anchor: &ExcelGridCellAddress,
        extent: &GridRect,
        authored: &BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
    ) -> bool {
        if blocked_formula_spill_extent_contains_anchor(anchor, &self.spill_facts, |fact_anchor| {
            matches!(
                authored.get(fact_anchor),
                Some(GridAuthoredCell::Formula(_))
            )
        }) {
            return true;
        }

        for row in extent.top_row..=extent.bottom_row {
            for col in extent.left_col..=extent.right_col {
                let address = ExcelGridCellAddress::new(
                    extent.workbook_id.clone(),
                    extent.sheet_id.clone(),
                    row,
                    col,
                );
                if &address == anchor {
                    continue;
                }
                if authored.contains_key(&address) {
                    return true;
                }
                if self
                    .merged_regions
                    .iter()
                    .any(|region| region.rect.contains(&address))
                {
                    return true;
                }
                if self.feature_rendered_regions.iter().any(|region| {
                    feature_rendered_region_blocks_spill(&region.feature_kind)
                        && region.rect.contains(&address)
                }) {
                    return true;
                }
                if self.spill_facts.values().any(|fact| {
                    !fact.blocked && fact.anchor != *anchor && fact.extent.contains(&address)
                }) {
                    return true;
                }
            }
        }
        false
    }

    #[must_use]
    pub fn reference_system_provider(
        &self,
        caller_row: u32,
        caller_col: u32,
    ) -> ExcelGridReferenceSystemProvider<'_> {
        let mut provider = ExcelGridReferenceSystemProvider::new(
            self.workbook_id.clone(),
            self.sheet_id.clone(),
            caller_row,
            caller_col,
        )
        .with_bounds(self.bounds)
        .with_borrowed_cells(&self.computed);
        for fact in self.spill_facts.values() {
            if fact.blocked {
                continue;
            }
            provider = provider.with_spill_extent(
                fact.anchor.workbook_id.clone(),
                fact.anchor.sheet_id.clone(),
                fact.anchor.row,
                fact.anchor.col,
                fact.extent.clone(),
            );
        }
        for (name, rect) in &self.defined_names {
            provider = provider.with_defined_name(name, rect.clone());
        }
        let caller_address = ExcelGridCellAddress::new(
            self.workbook_id.clone(),
            self.sheet_id.clone(),
            caller_row,
            caller_col,
        );
        for table in self.table_overlays.values() {
            provider = register_table_overlay_references(provider, table, Some(&caller_address));
        }
        provider
    }

    #[must_use]
    pub fn host_info_provider(&self, caller_row: u32, caller_col: u32) -> GridHostInfoProvider<'_> {
        GridHostInfoProvider::new(
            self.workbook_id.clone(),
            self.sheet_id.clone(),
            caller_row,
            caller_col,
            self.bounds,
            self.spill_facts.values(),
            &self.axis_state,
        )
    }

    fn evaluate_formula_with_oxfml(
        &self,
        address: &ExcelGridCellAddress,
        formula: &GridFormulaCell,
        profile: &StrictExcelGridReferenceProfile,
    ) -> Result<CalcValue, GridRefError> {
        let provider = self.reference_system_provider(address.row, address.col);
        let host_info = self.host_info_provider(address.row, address.col);
        let query_bundle = TypedContextQueryBundle::new(
            Some(&host_info as &dyn HostInfoProvider),
            None,
            None,
            None,
            None,
        )
        .with_reference_system_provider(Some(
            &provider as &dyn oxfunc_core::resolver::ReferenceSystemProvider,
        ));
        let source = FormulaSourceRecord::new(
            format!(
                "grid-calc-ref:{}:{}:R{}C{}",
                self.workbook_id, self.sheet_id, address.row, address.col
            ),
            1,
            formula.source_text.clone(),
        )
        .with_formula_channel_kind(formula.source_channel);
        let (enclosing_table_ref, caller_table_region) =
            grid_table_caller_context(self.table_overlays.values(), address);
        let environment = RuntimeEnvironment::new()
            .with_formula_scope(self.workbook_id.clone(), self.sheet_id.clone())
            .with_caller_position(address.row, address.col)
            .with_primary_locus(Locus {
                sheet_id: self.sheet_id.clone(),
                row: address.row,
                col: address.col,
            })
            .with_structure_context_version(StructureContextVersion(format!(
                "grid-calc-ref:{}:{}:{}x{}",
                self.workbook_id, self.sheet_id, self.bounds.max_rows, self.bounds.max_cols
            )))
            .with_table_context(
                grid_table_descriptor_catalog(self.table_overlays.values()),
                enclosing_table_ref,
                caller_table_region,
            )
            .with_reference_bind_profile(profile);
        let request = RuntimeFormulaRequest::new(source, query_bundle)
            .with_backend(EvaluationBackend::OxFuncBacked);
        let result = environment.execute(request);
        result
            .map(|result| result.published_calc_value())
            .map_err(|detail| GridRefError::OxfmlEvaluationFailed {
                address: address.clone(),
                detail,
            })
    }

    fn check_address(&self, address: &ExcelGridCellAddress) -> Result<(), GridRefError> {
        if address.workbook_id != self.workbook_id || address.sheet_id != self.sheet_id {
            return Err(GridRefError::AddressOnDifferentSheet {
                expected_workbook_id: self.workbook_id.clone(),
                expected_sheet_id: self.sheet_id.clone(),
                actual_workbook_id: address.workbook_id.clone(),
                actual_sheet_id: address.sheet_id.clone(),
            });
        }
        if !self.bounds.contains_row(address.row) || !self.bounds.contains_col(address.col) {
            return Err(GridRefError::AddressOutOfBounds {
                row: address.row,
                col: address.col,
                max_rows: self.bounds.max_rows,
                max_cols: self.bounds.max_cols,
            });
        }
        Ok(())
    }

    fn spill_anchor_for(&self, address: &ExcelGridCellAddress) -> Option<ExcelGridCellAddress> {
        self.spill_facts
            .values()
            .find(|fact| fact.extent.contains(address))
            .map(|fact| fact.anchor.clone())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GridFormulaEvaluationRequest<'a> {
    pub address: &'a ExcelGridCellAddress,
    pub formula: &'a GridFormulaCell,
    pub authored: &'a BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
    pub previous_computed: &'a BTreeMap<ExcelGridCellAddress, CalcValue>,
}

impl GridRect {
    fn check_sheet(&self, sheet: &GridCalcRefSheet) -> Result<(), GridRefError> {
        self.check_workbook_sheet(&sheet.workbook_id, &sheet.sheet_id)
    }
}

fn scalar_cells_unchecked(rect: &GridRect) -> Vec<ExcelGridCellAddress> {
    let mut cells = Vec::new();
    for row in rect.top_row..=rect.bottom_row {
        for col in rect.left_col..=rect.right_col {
            cells.push(ExcelGridCellAddress::new(
                rect.workbook_id.clone(),
                rect.sheet_id.clone(),
                row,
                col,
            ));
        }
    }
    cells
}

fn compressed_range_block_for_cell(row: u32, col: u32) -> (u32, u32) {
    (
        (row.saturating_sub(1)) / GRID_INVALIDATION_COMPRESSED_RANGE_INDEX_BLOCK_SIZE,
        (col.saturating_sub(1)) / GRID_INVALIDATION_COMPRESSED_RANGE_INDEX_BLOCK_SIZE,
    )
}

fn compressed_range_blocks_for_rect(rect: &GridRect) -> Vec<(u32, u32)> {
    let (top_block, left_block) = compressed_range_block_for_cell(rect.top_row, rect.left_col);
    let (bottom_block, right_block) =
        compressed_range_block_for_cell(rect.bottom_row, rect.right_col);
    let mut blocks = Vec::new();
    for row_block in top_block..=bottom_block {
        for col_block in left_block..=right_block {
            blocks.push((row_block, col_block));
        }
    }
    blocks
}

fn axis_visibility_block_for_index(axis: GridAxis, index: u32) -> (GridAxis, u32) {
    (
        axis,
        (index.saturating_sub(1)) / GRID_INVALIDATION_COMPRESSED_RANGE_INDEX_BLOCK_SIZE,
    )
}

fn axis_visibility_blocks_for_dependency(
    dependency: &GridAxisVisibilityDependency,
) -> Vec<(GridAxis, u32)> {
    let (_, first_block) = axis_visibility_block_for_index(dependency.axis, dependency.first);
    let (_, last_block) = axis_visibility_block_for_index(dependency.axis, dependency.last);
    (first_block..=last_block)
        .map(|block| (dependency.axis, block))
        .collect()
}

fn axis_visibility_dependencies_intersect(
    lhs: &GridAxisVisibilityDependency,
    rhs: &GridAxisVisibilityDependency,
) -> bool {
    lhs.axis == rhs.axis && lhs.first <= rhs.last && rhs.first <= lhs.last
}

fn spill_epoch_snapshot_map(
    snapshots: impl IntoIterator<Item = GridSpillEpochSnapshot>,
    bounds: ExcelGridBounds,
) -> Result<BTreeMap<ExcelGridCellAddress, GridSpillEpochSnapshot>, GridRefError> {
    let mut by_anchor = BTreeMap::new();
    for snapshot in snapshots {
        validate_spill_epoch_snapshot(&snapshot, bounds)?;
        by_anchor.insert(snapshot.anchor.clone(), snapshot);
    }
    Ok(by_anchor)
}

fn validate_spill_epoch_snapshot(
    snapshot: &GridSpillEpochSnapshot,
    bounds: ExcelGridBounds,
) -> Result<(), GridRefError> {
    if !bounds.contains_row(snapshot.anchor.row) || !bounds.contains_col(snapshot.anchor.col) {
        return Err(GridRefError::AddressOutOfBounds {
            row: snapshot.anchor.row,
            col: snapshot.anchor.col,
            max_rows: bounds.max_rows,
            max_cols: bounds.max_cols,
        });
    }
    if !snapshot.extent.contains(&snapshot.anchor) {
        return Err(GridRefError::InvalidStructuralEdit {
            detail: format!(
                "spill epoch snapshot extent R{}C{}:R{}C{} does not contain anchor R{}C{}",
                snapshot.extent.top_row,
                snapshot.extent.left_col,
                snapshot.extent.bottom_row,
                snapshot.extent.right_col,
                snapshot.anchor.row,
                snapshot.anchor.col
            ),
        });
    }
    Ok(())
}

fn spill_epoch_change_kind(
    old: Option<&GridSpillEpochSnapshot>,
    new: Option<&GridSpillEpochSnapshot>,
) -> Option<GridSpillEpochChangeKind> {
    match (old, new) {
        (None, None) => None,
        (None, Some(_)) => Some(GridSpillEpochChangeKind::Added),
        (Some(_), None) => Some(GridSpillEpochChangeKind::Removed),
        (Some(old), Some(new)) => {
            let extent_changed = old.extent != new.extent;
            let value_changed = old.value_epoch != new.value_epoch;
            let blocked_changed = old.blocked != new.blocked;
            match (extent_changed, value_changed, blocked_changed) {
                (false, false, false) => None,
                (true, true, _) => Some(GridSpillEpochChangeKind::ExtentAndValueChanged),
                (true, false, _) => Some(GridSpillEpochChangeKind::ExtentChanged),
                (false, true, _) => Some(GridSpillEpochChangeKind::ValueChanged),
                (false, false, true) => Some(GridSpillEpochChangeKind::BlockedChanged),
            }
        }
    }
}

fn transform_cell_map<T>(
    old: BTreeMap<ExcelGridCellAddress, T>,
    edit: GridAxisEdit,
    bounds: ExcelGridBounds,
) -> Result<(BTreeMap<ExcelGridCellAddress, T>, usize, usize), GridRefError> {
    let mut transformed = BTreeMap::new();
    let mut kept = 0;
    let mut dropped = 0;

    for (address, value) in old {
        match transform_address_for_edit(&address, edit, bounds)? {
            Some(new_address) => {
                transformed.insert(new_address, value);
                kept += 1;
            }
            None => dropped += 1,
        }
    }

    Ok((transformed, kept, dropped))
}

fn transform_authored_cell_map_for_edit(
    old: BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
    edit: GridAxisEdit,
    bounds: ExcelGridBounds,
) -> Result<
    (
        BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
        usize,
        usize,
        usize,
        usize,
    ),
    GridRefError,
> {
    let mut transformed = BTreeMap::new();
    let mut kept = 0;
    let mut dropped = 0;
    let mut formula_cells_transformed = 0;
    let mut formula_reference_transforms = 0;

    for (old_address, value) in old {
        let Some(new_address) = transform_address_for_edit(&old_address, edit, bounds)? else {
            dropped += 1;
            continue;
        };

        let value = match value {
            GridAuthoredCell::Formula(formula) => {
                let (formula, stats) = transform_formula_cell_for_axis_edit(
                    formula,
                    &old_address,
                    &new_address,
                    edit,
                    bounds,
                )?;
                formula_cells_transformed += stats.formula_cells_transformed;
                formula_reference_transforms += stats.formula_reference_transforms;
                GridAuthoredCell::Formula(formula)
            }
            GridAuthoredCell::Literal(value) => GridAuthoredCell::Literal(value),
        };

        transformed.insert(new_address, value);
        kept += 1;
    }

    Ok((
        transformed,
        kept,
        dropped,
        formula_cells_transformed,
        formula_reference_transforms,
    ))
}

fn transform_optimized_sparse_points_for_edit(
    old: BTreeMap<GridCellCoord, GridVersionedAuthoredCell>,
    workbook_id: &str,
    sheet_id: &str,
    edit: GridAxisEdit,
    bounds: ExcelGridBounds,
) -> Result<
    (
        BTreeMap<GridCellCoord, GridVersionedAuthoredCell>,
        usize,
        usize,
        usize,
        usize,
    ),
    GridRefError,
> {
    let mut transformed = BTreeMap::new();
    let mut kept = 0;
    let mut dropped = 0;
    let mut formula_cells_transformed = 0;
    let mut formula_reference_transforms = 0;

    for (old_coord, point) in old {
        let old_address = ExcelGridCellAddress::new(
            workbook_id.to_string(),
            sheet_id.to_string(),
            old_coord.row,
            old_coord.col,
        );
        let Some(new_address) = transform_address_for_edit(&old_address, edit, bounds)? else {
            dropped += 1;
            continue;
        };
        let cell = if let Some(formula) = point.cell.formula_ref() {
            let (formula, stats) = transform_formula_cell_for_axis_edit(
                formula.clone(),
                &old_address,
                &new_address,
                edit,
                bounds,
            )?;
            formula_cells_transformed += stats.formula_cells_transformed;
            formula_reference_transforms += stats.formula_reference_transforms;
            GridOptimizedAuthoredCell::formula(formula)
        } else {
            point.cell
        };
        transformed.insert(
            GridCellCoord::from_address(&new_address),
            GridVersionedAuthoredCell {
                revision: point.revision,
                cell,
            },
        );
        kept += 1;
    }

    Ok((
        transformed,
        kept,
        dropped,
        formula_cells_transformed,
        formula_reference_transforms,
    ))
}

fn transform_dense_value_region_for_edit(
    region: &GridDenseValueRegion,
    edit: GridAxisEdit,
    bounds: ExcelGridBounds,
) -> Result<Vec<GridDenseValueRegion>, GridRefError> {
    let mut transformed = Vec::new();
    for (old_rect, new_rect) in rect_segments_for_axis_edit(&region.rect, edit, bounds)? {
        let storage = region.storage.slice_for_subrect(&region.rect, &old_rect);
        transformed.push(GridDenseValueRegion {
            rect: new_rect,
            storage,
            revision: region.revision,
        });
    }
    Ok(transformed)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GridRepeatedFormulaRegionTransformOutput {
    regions: Vec<GridRepeatedFormulaRegion>,
    formula_segments_transformed: usize,
    formula_reference_transforms: usize,
}

fn transform_repeated_formula_region_for_edit(
    region: &GridRepeatedFormulaRegion,
    edit: GridAxisEdit,
    bounds: ExcelGridBounds,
) -> Result<GridRepeatedFormulaRegionTransformOutput, GridRefError> {
    let mut regions = Vec::new();
    let mut formula_segments_transformed = 0;
    let mut formula_reference_transforms = 0;
    for (old_rect, new_rect) in rect_segments_for_axis_edit(&region.rect, edit, bounds)? {
        let old_anchor = ExcelGridCellAddress::new(
            old_rect.workbook_id.clone(),
            old_rect.sheet_id.clone(),
            old_rect.top_row,
            old_rect.left_col,
        );
        let new_anchor = ExcelGridCellAddress::new(
            new_rect.workbook_id.clone(),
            new_rect.sheet_id.clone(),
            new_rect.top_row,
            new_rect.left_col,
        );
        let (formula, stats) = transform_formula_cell_for_axis_edit(
            region.formula.clone(),
            &old_anchor,
            &new_anchor,
            edit,
            bounds,
        )?;
        formula_segments_transformed += stats.formula_cells_transformed;
        formula_reference_transforms += stats.formula_reference_transforms;
        regions.push(GridRepeatedFormulaRegion {
            rect: new_rect,
            formula,
            revision: region.revision,
        });
    }
    Ok(GridRepeatedFormulaRegionTransformOutput {
        regions,
        formula_segments_transformed,
        formula_reference_transforms,
    })
}

fn dense_values_for_subrect(region: &GridDenseValueRegion, subrect: &GridRect) -> Vec<CalcValue> {
    let mut values =
        Vec::with_capacity(usize::try_from(subrect.cell_count()).unwrap_or(usize::MAX));
    for row in subrect.top_row..=subrect.bottom_row {
        for col in subrect.left_col..=subrect.right_col {
            let address = ExcelGridCellAddress::new(
                subrect.workbook_id.clone(),
                subrect.sheet_id.clone(),
                row,
                col,
            );
            values.push(
                region
                    .value_at(&address)
                    .expect("subrect cells must be inside dense value region"),
            );
        }
    }
    values
}

fn bytes_per_cell_micros(bytes: u64, cells: u64) -> u64 {
    if cells == 0 {
        return 0;
    }
    bytes
        .saturating_mul(1_000_000)
        .saturating_add(cells.saturating_sub(1))
        / cells
}

fn estimated_grid_cell_coord_bytes(_coord: GridCellCoord) -> u64 {
    u64::try_from(std::mem::size_of::<GridCellCoord>()).unwrap_or(u64::MAX)
}

fn estimated_grid_rect_heap_bytes(rect: &GridRect) -> u64 {
    u64::try_from(rect.workbook_id.len())
        .unwrap_or(u64::MAX)
        .saturating_add(u64::try_from(rect.sheet_id.len()).unwrap_or(u64::MAX))
}

fn estimated_versioned_authored_cell_bytes(cell: &GridVersionedAuthoredCell) -> u64 {
    u64::try_from(std::mem::size_of::<u64>())
        .unwrap_or(u64::MAX)
        .saturating_add(cell.cell.estimated_authored_bytes())
}

fn estimated_calc_value_bytes(value: &CalcValue) -> u64 {
    let core_payload = match &value.core {
        CoreValue::Number(_) => u64::try_from(std::mem::size_of::<f64>()).unwrap_or(u64::MAX),
        CoreValue::Text(text) => u64::try_from(text.len_utf16_code_units())
            .unwrap_or(u64::MAX)
            .saturating_mul(2),
        CoreValue::Logical(_) | CoreValue::Error(_) => 1,
        CoreValue::Empty | CoreValue::Missing => 0,
        CoreValue::Array(array) => array
            .iter_row_major()
            .map(estimated_calc_value_bytes)
            .fold(0_u64, u64::saturating_add),
        CoreValue::Reference(reference) => {
            u64::try_from(std::mem::size_of_val(reference)).unwrap_or(u64::MAX)
        }
    };
    u64::try_from(std::mem::size_of::<CalcValue>())
        .unwrap_or(u64::MAX)
        .saturating_add(core_payload)
        .saturating_add(if value.rich.is_some() {
            u64::try_from(std::mem::size_of_val(&value.rich)).unwrap_or(u64::MAX)
        } else {
            0
        })
}

fn estimated_calc_value_frame_payload_bytes(value: &CalcValue) -> u64 {
    let core_payload = match &value.core {
        CoreValue::Number(_) => u64::try_from(std::mem::size_of::<f64>()).unwrap_or(u64::MAX),
        CoreValue::Text(text) => u64::try_from(text.len_utf16_code_units())
            .unwrap_or(u64::MAX)
            .saturating_mul(2),
        CoreValue::Logical(_) | CoreValue::Error(_) => 1,
        CoreValue::Empty | CoreValue::Missing => 0,
        CoreValue::Array(array) => array
            .iter_row_major()
            .map(estimated_calc_value_frame_payload_bytes)
            .fold(0_u64, u64::saturating_add),
        CoreValue::Reference(reference) => {
            u64::try_from(std::mem::size_of_val(reference)).unwrap_or(u64::MAX)
        }
    };
    core_payload.saturating_add(if value.rich.is_some() {
        u64::try_from(std::mem::size_of_val(&value.rich)).unwrap_or(u64::MAX)
    } else {
        0
    })
}

fn estimated_formula_cell_bytes(formula: &GridFormulaCell) -> u64 {
    u64::try_from(std::mem::size_of::<GridFormulaCell>())
        .unwrap_or(u64::MAX)
        .saturating_add(u64::try_from(formula.source_text.len()).unwrap_or(u64::MAX))
        .saturating_add(u64::try_from(formula.normal_form_key.len()).unwrap_or(u64::MAX))
}

fn estimated_repeated_formula_region_bytes(region: &GridRepeatedFormulaRegion) -> u64 {
    u64::try_from(std::mem::size_of::<GridRepeatedFormulaRegion>())
        .unwrap_or(u64::MAX)
        .saturating_add(estimated_grid_rect_heap_bytes(&region.rect))
        .saturating_add(estimated_formula_cell_bytes(&region.formula))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GridAxisEditSegment {
    old_start: u32,
    old_end: u32,
    new_start: u32,
    new_end: u32,
}

fn rect_segments_for_axis_edit(
    rect: &GridRect,
    edit: GridAxisEdit,
    bounds: ExcelGridBounds,
) -> Result<Vec<(GridRect, GridRect)>, GridRefError> {
    validate_axis_edit(edit, bounds)?;
    let max = axis_max(edit.axis, bounds);
    let (start, end) = rect_axis_range(rect, edit.axis);
    let segments = axis_segments_for_edit(start, end, edit.kind, max)?;
    let mut rects = Vec::with_capacity(segments.len());
    for segment in segments {
        let (old_rect, new_rect) = match edit.axis {
            GridAxis::Row => (
                GridRect::new(
                    rect.workbook_id.clone(),
                    rect.sheet_id.clone(),
                    segment.old_start,
                    rect.left_col,
                    segment.old_end,
                    rect.right_col,
                    bounds,
                )?,
                GridRect::new(
                    rect.workbook_id.clone(),
                    rect.sheet_id.clone(),
                    segment.new_start,
                    rect.left_col,
                    segment.new_end,
                    rect.right_col,
                    bounds,
                )?,
            ),
            GridAxis::Column => (
                GridRect::new(
                    rect.workbook_id.clone(),
                    rect.sheet_id.clone(),
                    rect.top_row,
                    segment.old_start,
                    rect.bottom_row,
                    segment.old_end,
                    bounds,
                )?,
                GridRect::new(
                    rect.workbook_id.clone(),
                    rect.sheet_id.clone(),
                    rect.top_row,
                    segment.new_start,
                    rect.bottom_row,
                    segment.new_end,
                    bounds,
                )?,
            ),
        };
        rects.push((old_rect, new_rect));
    }
    Ok(rects)
}

fn axis_segments_for_edit(
    start: u32,
    end: u32,
    kind: GridAxisEditKind,
    max: u32,
) -> Result<Vec<GridAxisEditSegment>, GridRefError> {
    let mut segments = Vec::new();
    match kind {
        GridAxisEditKind::Insert { before, count } => {
            if before > end {
                segments.push(GridAxisEditSegment {
                    old_start: start,
                    old_end: end,
                    new_start: start,
                    new_end: end,
                });
                return Ok(segments);
            }
            if before <= start {
                let Some(new_start) = start.checked_add(count) else {
                    return Ok(segments);
                };
                if new_start > max {
                    return Ok(segments);
                }
                let new_end = end.saturating_add(count).min(max);
                let old_end = new_end.saturating_sub(count);
                if start <= old_end {
                    segments.push(GridAxisEditSegment {
                        old_start: start,
                        old_end,
                        new_start,
                        new_end,
                    });
                }
                return Ok(segments);
            }

            segments.push(GridAxisEditSegment {
                old_start: start,
                old_end: before - 1,
                new_start: start,
                new_end: before - 1,
            });
            let Some(new_start) = before.checked_add(count) else {
                return Ok(segments);
            };
            if new_start <= max {
                let new_end = end.saturating_add(count).min(max);
                let old_end = new_end.saturating_sub(count);
                if before <= old_end {
                    segments.push(GridAxisEditSegment {
                        old_start: before,
                        old_end,
                        new_start,
                        new_end,
                    });
                }
            }
        }
        GridAxisEditKind::Delete { first, count } => {
            let last = delete_last(first, count)?;
            if last < start {
                segments.push(GridAxisEditSegment {
                    old_start: start,
                    old_end: end,
                    new_start: start - count,
                    new_end: end - count,
                });
                return Ok(segments);
            }
            if first > end {
                segments.push(GridAxisEditSegment {
                    old_start: start,
                    old_end: end,
                    new_start: start,
                    new_end: end,
                });
                return Ok(segments);
            }
            if start < first {
                segments.push(GridAxisEditSegment {
                    old_start: start,
                    old_end: first - 1,
                    new_start: start,
                    new_end: first - 1,
                });
            }
            if last < end {
                segments.push(GridAxisEditSegment {
                    old_start: last + 1,
                    old_end: end,
                    new_start: first,
                    new_end: end - count,
                });
            }
        }
    }
    Ok(segments)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct GridFormulaStructuralTransformStats {
    formula_cells_transformed: usize,
    formula_reference_transforms: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FormulaSourceReplacement {
    start: usize,
    end: usize,
    replacement: String,
    transformed_reference: bool,
}

fn transform_formula_cell_for_axis_edit(
    formula: GridFormulaCell,
    old_address: &ExcelGridCellAddress,
    new_address: &ExcelGridCellAddress,
    edit: GridAxisEdit,
    bounds: ExcelGridBounds,
) -> Result<(GridFormulaCell, GridFormulaStructuralTransformStats), GridRefError> {
    let profile = StrictExcelGridReferenceProfile::with_bounds(bounds);
    let bound_before = bind_grid_formula_for_transform(&formula, old_address, &profile, bounds);
    let payload = ExcelGridReferenceTransformPayload::new(
        excel_grid_structural_edit_from_axis_edit(
            edit,
            &old_address.workbook_id,
            &old_address.sheet_id,
        ),
        Some(ExcelGridFormulaAnchor::new(
            old_address.workbook_id.clone(),
            old_address.sheet_id.clone(),
            old_address.row,
            old_address.col,
        )),
    )
    .with_formula_anchor_after(ExcelGridFormulaAnchor::new(
        new_address.workbook_id.clone(),
        new_address.sheet_id.clone(),
        new_address.row,
        new_address.col,
    ))
    .into_profile_payload();

    let mut replacements = Vec::new();
    for normalized in &bound_before.normalized_references {
        let NormalizedReference::ProfileSymbolic(record) = normalized else {
            continue;
        };
        if record.profile_id != EXCEL_GRID_PROFILE_ID {
            continue;
        }
        let result = profile.transform_reference(&ReferenceTransformRequest {
            reference: record.clone(),
            transform_kind: ReferenceTransformKind::StructuralEdit,
            payload: Some(payload.clone()),
        });

        match result.outcome {
            ReferenceTransformOutcome::Unchanged
            | ReferenceTransformOutcome::Shifted
            | ReferenceTransformOutcome::Expanded
            | ReferenceTransformOutcome::Shrunk
            | ReferenceTransformOutcome::Split
            | ReferenceTransformOutcome::PartiallyInvalid
            | ReferenceTransformOutcome::FullyInvalid => {}
            ReferenceTransformOutcome::DynamicOrHostSensitive
            | ReferenceTransformOutcome::Unsupported
            | ReferenceTransformOutcome::GeometryCoupledOpaqueConflict => {
                return Err(GridRefError::FormulaStructuralTransformFailed {
                    address: old_address.clone(),
                    detail: format!(
                        "reference '{}' returned {:?}: {}",
                        record.source_info.source_text,
                        result.outcome,
                        transform_diagnostics(&result.diagnostics)
                    ),
                });
            }
        }

        let Some(transformed_record) = result.reference else {
            return Err(GridRefError::FormulaStructuralTransformFailed {
                address: old_address.clone(),
                detail: format!(
                    "reference '{}' returned no transformed record",
                    record.source_info.source_text
                ),
            });
        };
        let Some(render_hint) = transformed_record.render_hint.clone() else {
            continue;
        };
        let span = record.source_info.source_span;
        replacements.push(FormulaSourceReplacement {
            start: span.start,
            end: span.end(),
            replacement: render_hint,
            transformed_reference: result.outcome != ReferenceTransformOutcome::Unchanged
                || transformed_record.normal_form_key != record.normal_form_key,
        });
    }

    let selected_replacements = select_non_overlapping_replacements(replacements);
    let transformed_reference_count = selected_replacements
        .iter()
        .filter(|replacement| replacement.transformed_reference)
        .count();
    let mut source_text = formula.source_text.clone();
    apply_formula_source_replacements(&mut source_text, selected_replacements, old_address)?;

    let mut transformed = formula;
    transformed.source_text = source_text;
    let bound_after = bind_grid_formula_for_transform(&transformed, new_address, &profile, bounds);
    transformed.normal_form_key = bound_after.formula_template_identity.key;
    let formula_cell_changed = transformed_reference_count > 0
        || transformed.normal_form_key != bound_before.formula_template_identity.key;

    Ok((
        transformed,
        GridFormulaStructuralTransformStats {
            formula_cells_transformed: usize::from(formula_cell_changed),
            formula_reference_transforms: transformed_reference_count,
        },
    ))
}

fn transform_authored_formulas_for_table_rename(
    authored: &mut BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
    old_table_key: &str,
    new_table_name: &str,
    bounds: ExcelGridBounds,
) -> Result<GridFormulaStructuralTransformStats, GridRefError> {
    let mut total = GridFormulaStructuralTransformStats::default();
    for (address, cell) in authored {
        let GridAuthoredCell::Formula(formula) = cell else {
            continue;
        };
        let (transformed, stats) = transform_formula_cell_for_table_rename(
            formula.clone(),
            address,
            old_table_key,
            new_table_name,
            bounds,
        )?;
        *formula = transformed;
        total.formula_cells_transformed += stats.formula_cells_transformed;
        total.formula_reference_transforms += stats.formula_reference_transforms;
    }
    Ok(total)
}

fn transform_authored_formulas_for_defined_name_rename(
    authored: &mut BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
    old_name_key: &str,
    new_name: &str,
    bounds: ExcelGridBounds,
) -> Result<GridFormulaStructuralTransformStats, GridRefError> {
    let mut total = GridFormulaStructuralTransformStats::default();
    for (address, cell) in authored {
        let GridAuthoredCell::Formula(formula) = cell else {
            continue;
        };
        let (transformed, stats) = transform_formula_cell_for_defined_name_rename(
            formula.clone(),
            address,
            old_name_key,
            new_name,
            bounds,
        )?;
        *formula = transformed;
        total.formula_cells_transformed += stats.formula_cells_transformed;
        total.formula_reference_transforms += stats.formula_reference_transforms;
    }
    Ok(total)
}

fn transform_authored_formulas_for_defined_name_delete(
    authored: &mut BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
    deleted_name_key: &str,
    bounds: ExcelGridBounds,
) -> Result<GridFormulaStructuralTransformStats, GridRefError> {
    let mut total = GridFormulaStructuralTransformStats::default();
    for (address, cell) in authored {
        let GridAuthoredCell::Formula(formula) = cell else {
            continue;
        };
        let (transformed, stats) = transform_formula_cell_for_defined_name_delete(
            formula.clone(),
            address,
            deleted_name_key,
            bounds,
        )?;
        *formula = transformed;
        total.formula_cells_transformed += stats.formula_cells_transformed;
        total.formula_reference_transforms += stats.formula_reference_transforms;
    }
    Ok(total)
}

fn transform_authored_formulas_for_table_delete(
    authored: &mut BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
    deleted_table_key: &str,
    bounds: ExcelGridBounds,
) -> Result<GridFormulaStructuralTransformStats, GridRefError> {
    let mut total = GridFormulaStructuralTransformStats::default();
    for (address, cell) in authored {
        let GridAuthoredCell::Formula(formula) = cell else {
            continue;
        };
        let (transformed, stats) = transform_formula_cell_for_table_delete(
            formula.clone(),
            address,
            deleted_table_key,
            bounds,
        )?;
        *formula = transformed;
        total.formula_cells_transformed += stats.formula_cells_transformed;
        total.formula_reference_transforms += stats.formula_reference_transforms;
    }
    Ok(total)
}

fn transform_sparse_point_formulas_for_defined_name_rename(
    sparse_points: &mut BTreeMap<GridCellCoord, GridVersionedAuthoredCell>,
    workbook_id: &str,
    sheet_id: &str,
    old_name_key: &str,
    new_name: &str,
    bounds: ExcelGridBounds,
) -> Result<GridFormulaStructuralTransformStats, GridRefError> {
    let mut total = GridFormulaStructuralTransformStats::default();
    for (coord, point) in sparse_points {
        let address = ExcelGridCellAddress::new(
            workbook_id.to_string(),
            sheet_id.to_string(),
            coord.row,
            coord.col,
        );
        let Some(formula) = point.cell.formula_mut() else {
            continue;
        };
        let (transformed, stats) = transform_formula_cell_for_defined_name_rename(
            formula.clone(),
            &address,
            old_name_key,
            new_name,
            bounds,
        )?;
        *formula = transformed;
        total.formula_cells_transformed += stats.formula_cells_transformed;
        total.formula_reference_transforms += stats.formula_reference_transforms;
    }
    Ok(total)
}

fn transform_sparse_point_formulas_for_defined_name_delete(
    sparse_points: &mut BTreeMap<GridCellCoord, GridVersionedAuthoredCell>,
    workbook_id: &str,
    sheet_id: &str,
    deleted_name_key: &str,
    bounds: ExcelGridBounds,
) -> Result<GridFormulaStructuralTransformStats, GridRefError> {
    let mut total = GridFormulaStructuralTransformStats::default();
    for (coord, point) in sparse_points {
        let address = ExcelGridCellAddress::new(
            workbook_id.to_string(),
            sheet_id.to_string(),
            coord.row,
            coord.col,
        );
        let Some(formula) = point.cell.formula_mut() else {
            continue;
        };
        let (transformed, stats) = transform_formula_cell_for_defined_name_delete(
            formula.clone(),
            &address,
            deleted_name_key,
            bounds,
        )?;
        *formula = transformed;
        total.formula_cells_transformed += stats.formula_cells_transformed;
        total.formula_reference_transforms += stats.formula_reference_transforms;
    }
    Ok(total)
}

fn transform_sparse_point_formulas_for_table_rename(
    sparse_points: &mut BTreeMap<GridCellCoord, GridVersionedAuthoredCell>,
    workbook_id: &str,
    sheet_id: &str,
    old_table_key: &str,
    new_table_name: &str,
    bounds: ExcelGridBounds,
) -> Result<GridFormulaStructuralTransformStats, GridRefError> {
    let mut total = GridFormulaStructuralTransformStats::default();
    for (coord, point) in sparse_points {
        let address = ExcelGridCellAddress::new(
            workbook_id.to_string(),
            sheet_id.to_string(),
            coord.row,
            coord.col,
        );
        let Some(formula) = point.cell.formula_mut() else {
            continue;
        };
        let (transformed, stats) = transform_formula_cell_for_table_rename(
            formula.clone(),
            &address,
            old_table_key,
            new_table_name,
            bounds,
        )?;
        *formula = transformed;
        total.formula_cells_transformed += stats.formula_cells_transformed;
        total.formula_reference_transforms += stats.formula_reference_transforms;
    }
    Ok(total)
}

fn transform_sparse_point_formulas_for_table_delete(
    sparse_points: &mut BTreeMap<GridCellCoord, GridVersionedAuthoredCell>,
    workbook_id: &str,
    sheet_id: &str,
    deleted_table_key: &str,
    bounds: ExcelGridBounds,
) -> Result<GridFormulaStructuralTransformStats, GridRefError> {
    let mut total = GridFormulaStructuralTransformStats::default();
    for (coord, point) in sparse_points {
        let address = ExcelGridCellAddress::new(
            workbook_id.to_string(),
            sheet_id.to_string(),
            coord.row,
            coord.col,
        );
        let Some(formula) = point.cell.formula_mut() else {
            continue;
        };
        let (transformed, stats) = transform_formula_cell_for_table_delete(
            formula.clone(),
            &address,
            deleted_table_key,
            bounds,
        )?;
        *formula = transformed;
        total.formula_cells_transformed += stats.formula_cells_transformed;
        total.formula_reference_transforms += stats.formula_reference_transforms;
    }
    Ok(total)
}

fn transform_repeated_formula_regions_for_defined_name_rename(
    regions: &mut [GridRepeatedFormulaRegion],
    old_name_key: &str,
    new_name: &str,
    bounds: ExcelGridBounds,
) -> Result<GridFormulaStructuralTransformStats, GridRefError> {
    let mut total = GridFormulaStructuralTransformStats::default();
    for region in regions {
        let address = ExcelGridCellAddress::new(
            region.rect.workbook_id.clone(),
            region.rect.sheet_id.clone(),
            region.rect.top_row,
            region.rect.left_col,
        );
        let (transformed, stats) = transform_formula_cell_for_defined_name_rename(
            region.formula.clone(),
            &address,
            old_name_key,
            new_name,
            bounds,
        )?;
        region.formula = transformed;
        total.formula_cells_transformed += stats.formula_cells_transformed;
        total.formula_reference_transforms += stats.formula_reference_transforms;
    }
    Ok(total)
}

fn transform_repeated_formula_regions_for_defined_name_delete(
    regions: &mut [GridRepeatedFormulaRegion],
    deleted_name_key: &str,
    bounds: ExcelGridBounds,
) -> Result<GridFormulaStructuralTransformStats, GridRefError> {
    let mut total = GridFormulaStructuralTransformStats::default();
    for region in regions {
        let address = ExcelGridCellAddress::new(
            region.rect.workbook_id.clone(),
            region.rect.sheet_id.clone(),
            region.rect.top_row,
            region.rect.left_col,
        );
        let (transformed, stats) = transform_formula_cell_for_defined_name_delete(
            region.formula.clone(),
            &address,
            deleted_name_key,
            bounds,
        )?;
        region.formula = transformed;
        total.formula_cells_transformed += stats.formula_cells_transformed;
        total.formula_reference_transforms += stats.formula_reference_transforms;
    }
    Ok(total)
}

fn transform_repeated_formula_regions_for_table_rename(
    regions: &mut [GridRepeatedFormulaRegion],
    old_table_key: &str,
    new_table_name: &str,
    bounds: ExcelGridBounds,
) -> Result<GridFormulaStructuralTransformStats, GridRefError> {
    let mut total = GridFormulaStructuralTransformStats::default();
    for region in regions {
        let address = ExcelGridCellAddress::new(
            region.rect.workbook_id.clone(),
            region.rect.sheet_id.clone(),
            region.rect.top_row,
            region.rect.left_col,
        );
        let (transformed, stats) = transform_formula_cell_for_table_rename(
            region.formula.clone(),
            &address,
            old_table_key,
            new_table_name,
            bounds,
        )?;
        region.formula = transformed;
        total.formula_cells_transformed += stats.formula_cells_transformed;
        total.formula_reference_transforms += stats.formula_reference_transforms;
    }
    Ok(total)
}

fn transform_repeated_formula_regions_for_table_delete(
    regions: &mut [GridRepeatedFormulaRegion],
    deleted_table_key: &str,
    bounds: ExcelGridBounds,
) -> Result<GridFormulaStructuralTransformStats, GridRefError> {
    let mut total = GridFormulaStructuralTransformStats::default();
    for region in regions {
        let address = ExcelGridCellAddress::new(
            region.rect.workbook_id.clone(),
            region.rect.sheet_id.clone(),
            region.rect.top_row,
            region.rect.left_col,
        );
        let (transformed, stats) = transform_formula_cell_for_table_delete(
            region.formula.clone(),
            &address,
            deleted_table_key,
            bounds,
        )?;
        region.formula = transformed;
        total.formula_cells_transformed += stats.formula_cells_transformed;
        total.formula_reference_transforms += stats.formula_reference_transforms;
    }
    Ok(total)
}

fn transform_formula_cell_for_table_rename(
    formula: GridFormulaCell,
    address: &ExcelGridCellAddress,
    old_table_key: &str,
    new_table_name: &str,
    bounds: ExcelGridBounds,
) -> Result<(GridFormulaCell, GridFormulaStructuralTransformStats), GridRefError> {
    let profile = StrictExcelGridReferenceProfile::with_bounds(bounds);
    let bound_before = bind_grid_formula_for_transform(&formula, address, &profile, bounds);
    let mut replacements = Vec::new();

    for normalized in &bound_before.normalized_references {
        let NormalizedReference::ProfileSymbolic(record) = normalized else {
            continue;
        };
        if record.profile_id != EXCEL_GRID_PROFILE_ID {
            continue;
        }
        let Some(ExcelGridReference::StructuredReference { .. }) =
            decode_excel_grid_reference_payload(&record.profile_payload)
        else {
            continue;
        };
        let Some(replacement) = rewrite_structured_reference_table_name(
            &record.source_info.source_text,
            old_table_key,
            new_table_name,
        ) else {
            continue;
        };
        let span = record.source_info.source_span;
        replacements.push(FormulaSourceReplacement {
            start: span.start,
            end: span.end(),
            replacement,
            transformed_reference: true,
        });
    }

    let selected_replacements = select_non_overlapping_replacements(replacements);
    let transformed_reference_count = selected_replacements
        .iter()
        .filter(|replacement| replacement.transformed_reference)
        .count();
    if transformed_reference_count == 0 {
        return Ok((
            formula,
            GridFormulaStructuralTransformStats {
                formula_cells_transformed: 0,
                formula_reference_transforms: 0,
            },
        ));
    }

    let mut source_text = formula.source_text.clone();
    apply_formula_source_replacements(&mut source_text, selected_replacements, address)?;
    let mut transformed = formula;
    transformed.source_text = source_text;
    let bound_after = bind_grid_formula_for_transform(&transformed, address, &profile, bounds);
    transformed.normal_form_key = bound_after.formula_template_identity.key;

    Ok((
        transformed,
        GridFormulaStructuralTransformStats {
            formula_cells_transformed: 1,
            formula_reference_transforms: transformed_reference_count,
        },
    ))
}

fn transform_formula_cell_for_defined_name_rename(
    formula: GridFormulaCell,
    address: &ExcelGridCellAddress,
    old_name_key: &str,
    new_name: &str,
    bounds: ExcelGridBounds,
) -> Result<(GridFormulaCell, GridFormulaStructuralTransformStats), GridRefError> {
    let profile = StrictExcelGridReferenceProfile::with_bounds(bounds);
    let bound_before = bind_grid_formula_for_transform(&formula, address, &profile, bounds);
    let mut replacements = Vec::new();

    for normalized in &bound_before.normalized_references {
        let NormalizedReference::ProfileSymbolic(record) = normalized else {
            continue;
        };
        if record.profile_id != EXCEL_GRID_PROFILE_ID {
            continue;
        }
        let Some(ExcelGridReference::Name { .. }) =
            decode_excel_grid_reference_payload(&record.profile_payload)
        else {
            continue;
        };
        let Some(replacement) = rewrite_defined_name_reference(
            &record.source_info.source_text,
            old_name_key,
            new_name,
            bounds,
        ) else {
            continue;
        };
        let span = record.source_info.source_span;
        replacements.push(FormulaSourceReplacement {
            start: span.start,
            end: span.end(),
            replacement,
            transformed_reference: true,
        });
    }

    let selected_replacements = select_non_overlapping_replacements(replacements);
    let transformed_reference_count = selected_replacements
        .iter()
        .filter(|replacement| replacement.transformed_reference)
        .count();
    if transformed_reference_count == 0 {
        return Ok((
            formula,
            GridFormulaStructuralTransformStats {
                formula_cells_transformed: 0,
                formula_reference_transforms: 0,
            },
        ));
    }

    let mut source_text = formula.source_text.clone();
    apply_formula_source_replacements(&mut source_text, selected_replacements, address)?;
    let mut transformed = formula;
    transformed.source_text = source_text;
    let bound_after = bind_grid_formula_for_transform(&transformed, address, &profile, bounds);
    transformed.normal_form_key = bound_after.formula_template_identity.key;

    Ok((
        transformed,
        GridFormulaStructuralTransformStats {
            formula_cells_transformed: 1,
            formula_reference_transforms: transformed_reference_count,
        },
    ))
}

fn transform_formula_cell_for_defined_name_delete(
    formula: GridFormulaCell,
    address: &ExcelGridCellAddress,
    deleted_name_key: &str,
    bounds: ExcelGridBounds,
) -> Result<(GridFormulaCell, GridFormulaStructuralTransformStats), GridRefError> {
    let profile = StrictExcelGridReferenceProfile::with_bounds(bounds);
    let bound_before = bind_grid_formula_for_transform(&formula, address, &profile, bounds);
    let mut replacements = Vec::new();

    for normalized in &bound_before.normalized_references {
        let NormalizedReference::ProfileSymbolic(record) = normalized else {
            continue;
        };
        if record.profile_id != EXCEL_GRID_PROFILE_ID {
            continue;
        }
        let Some(ExcelGridReference::Name { .. }) =
            decode_excel_grid_reference_payload(&record.profile_payload)
        else {
            continue;
        };
        if !defined_name_reference_has_key(
            &record.source_info.source_text,
            deleted_name_key,
            bounds,
        ) {
            continue;
        }
        let span = record.source_info.source_span;
        replacements.push(FormulaSourceReplacement {
            start: span.start,
            end: span.end(),
            replacement: "#NAME?".to_string(),
            transformed_reference: true,
        });
    }

    let selected_replacements = select_non_overlapping_replacements(replacements);
    let transformed_reference_count = selected_replacements
        .iter()
        .filter(|replacement| replacement.transformed_reference)
        .count();
    if transformed_reference_count == 0 {
        return Ok((
            formula,
            GridFormulaStructuralTransformStats {
                formula_cells_transformed: 0,
                formula_reference_transforms: 0,
            },
        ));
    }

    let mut source_text = formula.source_text.clone();
    apply_formula_source_replacements(&mut source_text, selected_replacements, address)?;
    let mut transformed = formula;
    transformed.source_text = source_text;
    let bound_after = bind_grid_formula_for_transform(&transformed, address, &profile, bounds);
    transformed.normal_form_key = bound_after.formula_template_identity.key;

    Ok((
        transformed,
        GridFormulaStructuralTransformStats {
            formula_cells_transformed: 1,
            formula_reference_transforms: transformed_reference_count,
        },
    ))
}

fn transform_formula_cell_for_table_delete(
    formula: GridFormulaCell,
    address: &ExcelGridCellAddress,
    deleted_table_key: &str,
    bounds: ExcelGridBounds,
) -> Result<(GridFormulaCell, GridFormulaStructuralTransformStats), GridRefError> {
    let profile = StrictExcelGridReferenceProfile::with_bounds(bounds);
    let bound_before = bind_grid_formula_for_transform(&formula, address, &profile, bounds);
    let mut replacements = Vec::new();

    for normalized in &bound_before.normalized_references {
        let NormalizedReference::ProfileSymbolic(record) = normalized else {
            continue;
        };
        if record.profile_id != EXCEL_GRID_PROFILE_ID {
            continue;
        }
        let Some(ExcelGridReference::StructuredReference { .. }) =
            decode_excel_grid_reference_payload(&record.profile_payload)
        else {
            continue;
        };
        if !structured_reference_has_explicit_table_key(
            &record.source_info.source_text,
            deleted_table_key,
        ) {
            continue;
        }
        let span = record.source_info.source_span;
        replacements.push(FormulaSourceReplacement {
            start: span.start,
            end: span.end(),
            replacement: "#REF!".to_string(),
            transformed_reference: true,
        });
    }

    let selected_replacements = select_non_overlapping_replacements(replacements);
    let transformed_reference_count = selected_replacements
        .iter()
        .filter(|replacement| replacement.transformed_reference)
        .count();
    if transformed_reference_count == 0 {
        return Ok((
            formula,
            GridFormulaStructuralTransformStats {
                formula_cells_transformed: 0,
                formula_reference_transforms: 0,
            },
        ));
    }

    let mut source_text = formula.source_text.clone();
    apply_formula_source_replacements(&mut source_text, selected_replacements, address)?;
    let mut transformed = formula;
    transformed.source_text = source_text;
    let bound_after = bind_grid_formula_for_transform(&transformed, address, &profile, bounds);
    transformed.normal_form_key = bound_after.formula_template_identity.key;

    Ok((
        transformed,
        GridFormulaStructuralTransformStats {
            formula_cells_transformed: 1,
            formula_reference_transforms: transformed_reference_count,
        },
    ))
}

fn rewrite_defined_name_reference(
    source_text: &str,
    old_name_key: &str,
    new_name: &str,
    bounds: ExcelGridBounds,
) -> Option<String> {
    if !defined_name_reference_has_key(source_text, old_name_key, bounds) {
        return None;
    }
    let local_start = source_text.rfind('!').map_or(0, |index| index + 1);
    Some(format!("{}{}", &source_text[..local_start], new_name))
}

fn defined_name_reference_has_key(
    source_text: &str,
    name_key: &str,
    bounds: ExcelGridBounds,
) -> bool {
    let local_start = source_text.rfind('!').map_or(0, |index| index + 1);
    let name = source_text[local_start..].trim();
    defined_name_key_for_name(name, bounds).is_ok_and(|key| key == name_key)
}

fn rewrite_structured_reference_table_name(
    source_text: &str,
    old_table_key: &str,
    new_table_name: &str,
) -> Option<String> {
    if !structured_reference_has_explicit_table_key(source_text, old_table_key) {
        return None;
    }
    let bracket_index = source_text.find('[')?;
    let local_start = source_text[..bracket_index]
        .rfind('!')
        .map_or(0, |index| index + 1);
    Some(format!(
        "{}{}{}",
        &source_text[..local_start],
        new_table_name,
        &source_text[bracket_index..]
    ))
}

fn structured_reference_has_explicit_table_key(source_text: &str, table_key: &str) -> bool {
    let Some(bracket_index) = source_text.find('[') else {
        return false;
    };
    let local_start = source_text[..bracket_index]
        .rfind('!')
        .map_or(0, |index| index + 1);
    let table_name = source_text[local_start..bracket_index].trim();
    !table_name.is_empty() && table_name.to_ascii_uppercase() == table_key
}

fn bind_grid_formula_for_transform(
    formula: &GridFormulaCell,
    address: &ExcelGridCellAddress,
    profile: &dyn ReferenceBindProfile,
    bounds: ExcelGridBounds,
) -> BoundFormula {
    let source = FormulaSourceRecord::new(
        format!(
            "grid-structural-transform:{}:{}:R{}C{}",
            address.workbook_id, address.sheet_id, address.row, address.col
        ),
        1,
        formula.source_text.clone(),
    )
    .with_formula_channel_kind(formula.source_channel);
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let request = BindRequest {
        source,
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            workbook_id: address.workbook_id.clone(),
            sheet_id: address.sheet_id.clone(),
            caller_row: address.row,
            caller_col: address.col,
            formula_token: FormulaToken(format!(
                "grid-structural-transform:{}:{}:R{}C{}",
                address.workbook_id, address.sheet_id, address.row, address.col
            )),
            structure_context_version: StructureContextVersion(format!(
                "grid-structural-transform:{}:{}:{}x{}",
                address.workbook_id, address.sheet_id, bounds.max_rows, bounds.max_cols
            )),
            ..BindContext::default()
        },
        reference_bind_profile: Some(profile),
    };
    bind_formula(request).bound_formula
}

fn excel_grid_structural_edit_from_axis_edit(
    edit: GridAxisEdit,
    workbook_id: &str,
    sheet_id: &str,
) -> ExcelGridStructuralEdit {
    match (edit.axis, edit.kind) {
        (GridAxis::Row, GridAxisEditKind::Insert { before, count }) => {
            ExcelGridStructuralEdit::insert_rows(workbook_id, sheet_id, before, count)
        }
        (GridAxis::Row, GridAxisEditKind::Delete { first, count }) => {
            ExcelGridStructuralEdit::delete_rows(workbook_id, sheet_id, first, count)
        }
        (GridAxis::Column, GridAxisEditKind::Insert { before, count }) => {
            ExcelGridStructuralEdit::insert_columns(workbook_id, sheet_id, before, count)
        }
        (GridAxis::Column, GridAxisEditKind::Delete { first, count }) => {
            ExcelGridStructuralEdit::delete_columns(workbook_id, sheet_id, first, count)
        }
    }
}

fn select_non_overlapping_replacements(
    mut replacements: Vec<FormulaSourceReplacement>,
) -> Vec<FormulaSourceReplacement> {
    replacements.sort_by(|left, right| {
        left.start
            .cmp(&right.start)
            .then_with(|| right.end.cmp(&left.end))
    });

    let mut selected = Vec::new();
    let mut covered_until = 0;
    for replacement in replacements {
        if replacement.start >= covered_until {
            covered_until = replacement.end;
            selected.push(replacement);
        }
    }
    selected
}

fn apply_formula_source_replacements(
    source_text: &mut String,
    mut replacements: Vec<FormulaSourceReplacement>,
    address: &ExcelGridCellAddress,
) -> Result<(), GridRefError> {
    replacements.sort_by(|left, right| right.start.cmp(&left.start));
    for replacement in replacements {
        if replacement.start > replacement.end
            || replacement.end > source_text.len()
            || !source_text.is_char_boundary(replacement.start)
            || !source_text.is_char_boundary(replacement.end)
        {
            return Err(GridRefError::FormulaStructuralTransformFailed {
                address: address.clone(),
                detail: format!(
                    "invalid formula source span {}..{} for text length {}",
                    replacement.start,
                    replacement.end,
                    source_text.len()
                ),
            });
        }
        source_text.replace_range(replacement.start..replacement.end, &replacement.replacement);
    }
    Ok(())
}

fn transform_diagnostics(diagnostics: &[String]) -> String {
    if diagnostics.is_empty() {
        "no diagnostics".to_string()
    } else {
        diagnostics.join("; ")
    }
}

fn transform_address_for_edit(
    address: &ExcelGridCellAddress,
    edit: GridAxisEdit,
    bounds: ExcelGridBounds,
) -> Result<Option<ExcelGridCellAddress>, GridRefError> {
    validate_axis_edit(edit, bounds)?;
    let max = axis_max(edit.axis, bounds);
    let Some(new_index) =
        transform_axis_index(address_axis_index(address, edit.axis), edit.kind, max)?
    else {
        return Ok(None);
    };
    let mut transformed = address.clone();
    match edit.axis {
        GridAxis::Row => transformed.row = new_index,
        GridAxis::Column => transformed.col = new_index,
    }
    Ok(Some(transformed))
}

fn transform_spill_value_fingerprints_for_edit(
    fingerprints: BTreeMap<ExcelGridCellAddress, String>,
    edit: GridAxisEdit,
    bounds: ExcelGridBounds,
) -> Result<BTreeMap<ExcelGridCellAddress, String>, GridRefError> {
    let mut transformed = BTreeMap::new();
    for (anchor, fingerprint) in fingerprints {
        if let Some(new_anchor) = transform_address_for_edit(&anchor, edit, bounds)? {
            transformed.insert(new_anchor, fingerprint);
        }
    }
    Ok(transformed)
}

fn transform_rect_for_edit(
    rect: &GridRect,
    edit: GridAxisEdit,
    bounds: ExcelGridBounds,
) -> Result<(Option<GridRect>, GridStructuralTransformOutcome), GridRefError> {
    validate_axis_edit(edit, bounds)?;
    let max = axis_max(edit.axis, bounds);
    let (start, end) = rect_axis_range(rect, edit.axis);
    let Some((new_start, new_end, outcome)) = transform_axis_range(start, end, edit.kind, max)?
    else {
        return Ok((None, GridStructuralTransformOutcome::Deleted));
    };

    let mut transformed = rect.clone();
    match edit.axis {
        GridAxis::Row => {
            transformed.top_row = new_start;
            transformed.bottom_row = new_end;
        }
        GridAxis::Column => {
            transformed.left_col = new_start;
            transformed.right_col = new_end;
        }
    }
    Ok((Some(transformed), outcome))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FeatureRenderedRegionTransformBatch {
    regions: Vec<FeatureRenderedRegion>,
    kept: usize,
    dropped: usize,
    marked_needs_refresh: usize,
}

fn transform_feature_rendered_regions_for_axis_edit(
    regions: &[FeatureRenderedRegion],
    edit: GridAxisEdit,
    bounds: ExcelGridBounds,
) -> Result<FeatureRenderedRegionTransformBatch, GridRefError> {
    let mut transformed_regions = Vec::new();
    let mut kept = 0;
    let mut dropped = 0;
    let mut marked_needs_refresh = 0;

    for region in regions {
        if feature_rendered_region_axis_edit_refused(region, edit)? {
            return Err(GridRefError::FeatureRenderedRegionEditRefused {
                feature_kind: region.feature_kind.clone(),
                detail: format!(
                    "{:?} edit intersects claimed region R{}C{}:R{}C{}",
                    edit.axis,
                    region.rect.top_row,
                    region.rect.left_col,
                    region.rect.bottom_row,
                    region.rect.right_col
                ),
            });
        }

        let (Some(rect), outcome) = transform_rect_for_edit(&region.rect, edit, bounds)? else {
            dropped += 1;
            continue;
        };
        let mut needs_refresh = region.needs_refresh;
        if feature_rendered_region_marks_refresh_on_transform(&region.feature_kind)
            && outcome != GridStructuralTransformOutcome::Unchanged
            && !needs_refresh
        {
            needs_refresh = true;
            marked_needs_refresh += 1;
        }
        transformed_regions.push(FeatureRenderedRegion {
            rect,
            feature_kind: region.feature_kind.clone(),
            needs_refresh,
        });
        kept += 1;
    }

    Ok(FeatureRenderedRegionTransformBatch {
        regions: transformed_regions,
        kept,
        dropped,
        marked_needs_refresh,
    })
}

fn feature_rendered_region_axis_edit_refused(
    region: &FeatureRenderedRegion,
    edit: GridAxisEdit,
) -> Result<bool, GridRefError> {
    if !feature_rendered_region_refuses_inside_axis_edit(&region.feature_kind) {
        return Ok(false);
    }
    let (start, end) = rect_axis_range(&region.rect, edit.axis);
    match edit.kind {
        GridAxisEditKind::Insert { before, .. } => Ok(start < before && before <= end),
        GridAxisEditKind::Delete { first, count } => {
            let last = delete_last(first, count)?;
            Ok(first <= end && start <= last)
        }
    }
}

fn transform_dependency_for_axis_edit(
    dependency: &GridDependency,
    edit: GridAxisEdit,
    bounds: ExcelGridBounds,
) -> Result<Option<GridDependency>, GridRefError> {
    match dependency {
        GridDependency::Cell(address) => {
            Ok(transform_address_for_edit(address, edit, bounds)?.map(GridDependency::Cell))
        }
        GridDependency::Range(rect) => Ok(transform_rect_for_edit(rect, edit, bounds)?
            .0
            .map(GridDependency::Range)),
        GridDependency::Name(dependency) => {
            Ok(transform_rect_for_edit(&dependency.extent, edit, bounds)?
                .0
                .map(|extent| {
                    GridDependency::Name(GridNameDependency {
                        name_key: dependency.name_key.clone(),
                        extent,
                    })
                }))
        }
        GridDependency::Table(dependency) => {
            Ok(transform_rect_for_edit(&dependency.extent, edit, bounds)?
                .0
                .map(|extent| {
                    GridDependency::Table(GridTableDependency {
                        table_key: dependency.table_key.clone(),
                        extent,
                    })
                }))
        }
        GridDependency::SpillFact(dependency) => {
            Ok(
                transform_address_for_edit(&dependency.anchor, edit, bounds)?
                    .map(|anchor| GridDependency::SpillFact(GridSpillDependency { anchor })),
            )
        }
        GridDependency::SpillBlocker(dependency) => {
            Ok(transform_rect_for_edit(&dependency.extent, edit, bounds)?
                .0
                .map(|extent| GridDependency::SpillBlocker(GridSpillBlockerDependency { extent })))
        }
        GridDependency::AxisVisibility(dependency) => Ok(
            transform_axis_visibility_dependency_for_edit(dependency, edit, bounds)?
                .map(GridDependency::AxisVisibility),
        ),
        GridDependency::AxisValue(dependency) => Ok(transform_axis_value_dependency_for_edit(
            dependency, edit, bounds,
        )?
        .map(GridDependency::AxisValue)),
        GridDependency::DynamicRequest(request_key) => {
            Ok(Some(GridDependency::DynamicRequest(request_key.clone())))
        }
    }
}

fn transform_axis_visibility_dependency_for_edit(
    dependency: &GridAxisVisibilityDependency,
    edit: GridAxisEdit,
    bounds: ExcelGridBounds,
) -> Result<Option<GridAxisVisibilityDependency>, GridRefError> {
    validate_axis_visibility_dependency(dependency, bounds)?;
    if dependency.axis != edit.axis {
        return Ok(Some(dependency.clone()));
    }
    let max = axis_max(edit.axis, bounds);
    let Some((new_start, new_end, _)) =
        transform_axis_range(dependency.first, dependency.last, edit.kind, max)?
    else {
        return Ok(None);
    };
    Ok(Some(GridAxisVisibilityDependency {
        axis: dependency.axis,
        first: new_start,
        last: new_end,
    }))
}

fn transform_axis_value_dependency_for_edit(
    dependency: &GridAxisValueDependency,
    edit: GridAxisEdit,
    bounds: ExcelGridBounds,
) -> Result<Option<GridAxisValueDependency>, GridRefError> {
    validate_axis_value_dependency(dependency, bounds)?;
    if dependency.axis != edit.axis {
        return Ok(Some(dependency.clone()));
    }
    let max = axis_max(edit.axis, bounds);
    let Some((new_start, new_end, _)) =
        transform_axis_range(dependency.first, dependency.last, edit.kind, max)?
    else {
        return Ok(None);
    };
    Ok(Some(GridAxisValueDependency {
        axis: dependency.axis,
        first: new_start,
        last: new_end,
    }))
}

fn transform_axis_range(
    start: u32,
    end: u32,
    kind: GridAxisEditKind,
    max: u32,
) -> Result<Option<(u32, u32, GridStructuralTransformOutcome)>, GridRefError> {
    match kind {
        GridAxisEditKind::Insert { before, count } => {
            if before > end {
                return Ok(Some((
                    start,
                    end,
                    GridStructuralTransformOutcome::Unchanged,
                )));
            }
            if before <= start {
                let Some(new_start) = start.checked_add(count) else {
                    return Ok(None);
                };
                if new_start > max {
                    return Ok(None);
                }
                let unclipped_end = end.saturating_add(count);
                let new_end = unclipped_end.min(max);
                let outcome = if unclipped_end > max {
                    GridStructuralTransformOutcome::Shrunk
                } else {
                    GridStructuralTransformOutcome::Shifted
                };
                return Ok(Some((new_start, new_end, outcome)));
            }

            let unclipped_end = end.saturating_add(count);
            let new_end = unclipped_end.min(max);
            let outcome = if new_end > end {
                GridStructuralTransformOutcome::Expanded
            } else {
                GridStructuralTransformOutcome::Unchanged
            };
            Ok(Some((start, new_end, outcome)))
        }
        GridAxisEditKind::Delete { first, count } => {
            let last = delete_last(first, count)?;
            if last < start {
                return Ok(Some((
                    start - count,
                    end - count,
                    GridStructuralTransformOutcome::Shifted,
                )));
            }
            if first > end {
                return Ok(Some((
                    start,
                    end,
                    GridStructuralTransformOutcome::Unchanged,
                )));
            }

            let overlap_start = start.max(first);
            let overlap_end = end.min(last);
            let overlap_count = overlap_end - overlap_start + 1;
            let length = end - start + 1;
            if overlap_count == length {
                return Ok(None);
            }

            let new_length = length - overlap_count;
            let new_start = if first <= start { first } else { start };
            let new_end = new_start + new_length - 1;
            Ok(Some((
                new_start,
                new_end,
                GridStructuralTransformOutcome::Shrunk,
            )))
        }
    }
}

fn transform_axis_index(
    index: u32,
    kind: GridAxisEditKind,
    max: u32,
) -> Result<Option<u32>, GridRefError> {
    match kind {
        GridAxisEditKind::Insert { before, count } => {
            if index < before {
                return Ok(Some(index));
            }
            let Some(new_index) = index.checked_add(count) else {
                return Ok(None);
            };
            Ok((new_index <= max).then_some(new_index))
        }
        GridAxisEditKind::Delete { first, count } => {
            let last = delete_last(first, count)?;
            if index < first {
                Ok(Some(index))
            } else if index <= last {
                Ok(None)
            } else {
                Ok(Some(index - count))
            }
        }
    }
}

fn validate_axis_edit(edit: GridAxisEdit, bounds: ExcelGridBounds) -> Result<(), GridRefError> {
    let max = axis_max(edit.axis, bounds);
    match edit.kind {
        GridAxisEditKind::Insert { before, count } => {
            if count == 0 || before == 0 || before > max.saturating_add(1) {
                return Err(GridRefError::InvalidStructuralEdit {
                    detail: format!(
                        "insert {:?} before {before} count {count} outside 1..={}",
                        edit.axis,
                        max.saturating_add(1)
                    ),
                });
            }
        }
        GridAxisEditKind::Delete { first, count } => {
            if count == 0 || first == 0 {
                return Err(GridRefError::InvalidStructuralEdit {
                    detail: format!(
                        "delete {:?} first {first} count {count} is invalid",
                        edit.axis
                    ),
                });
            }
            let last = delete_last(first, count)?;
            if first > max || last > max {
                return Err(GridRefError::InvalidStructuralEdit {
                    detail: format!(
                        "delete {:?} first {first} count {count} outside 1..={max}",
                        edit.axis
                    ),
                });
            }
        }
    }
    Ok(())
}

fn validate_axis_visibility_dependency(
    dependency: &GridAxisVisibilityDependency,
    bounds: ExcelGridBounds,
) -> Result<(), GridRefError> {
    let max = axis_max(dependency.axis, bounds);
    if dependency.first == 0 || dependency.last == 0 || dependency.first > dependency.last {
        return Err(GridRefError::InvalidAxisVisibilityDependency {
            detail: format!(
                "axis visibility {:?} range {}..{} is invalid",
                dependency.axis, dependency.first, dependency.last
            ),
        });
    }
    if dependency.last > max {
        return Err(GridRefError::InvalidAxisVisibilityDependency {
            detail: format!(
                "axis visibility {:?} range {}..{} outside 1..={max}",
                dependency.axis, dependency.first, dependency.last
            ),
        });
    }
    Ok(())
}

fn validate_axis_value_dependency(
    dependency: &GridAxisValueDependency,
    bounds: ExcelGridBounds,
) -> Result<(), GridRefError> {
    let max = axis_max(dependency.axis, bounds);
    if dependency.first == 0 || dependency.last == 0 || dependency.first > dependency.last {
        return Err(GridRefError::InvalidAxisValueDependency {
            detail: format!(
                "axis value {:?} range {}..{} is invalid",
                dependency.axis, dependency.first, dependency.last
            ),
        });
    }
    if dependency.last > max {
        return Err(GridRefError::InvalidAxisValueDependency {
            detail: format!(
                "axis value {:?} range {}..{} outside 1..={max}",
                dependency.axis, dependency.first, dependency.last
            ),
        });
    }
    Ok(())
}

fn delete_last(first: u32, count: u32) -> Result<u32, GridRefError> {
    first
        .checked_add(count.saturating_sub(1))
        .ok_or_else(|| GridRefError::InvalidStructuralEdit {
            detail: format!("delete first {first} count {count} overflows axis"),
        })
}

const fn axis_max(axis: GridAxis, bounds: ExcelGridBounds) -> u32 {
    match axis {
        GridAxis::Row => bounds.max_rows,
        GridAxis::Column => bounds.max_cols,
    }
}

const fn address_axis_index(address: &ExcelGridCellAddress, axis: GridAxis) -> u32 {
    match axis {
        GridAxis::Row => address.row,
        GridAxis::Column => address.col,
    }
}

const fn rect_axis_range(rect: &GridRect, axis: GridAxis) -> (u32, u32) {
    match axis {
        GridAxis::Row => (rect.top_row, rect.bottom_row),
        GridAxis::Column => (rect.left_col, rect.right_col),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridAxisVisibilityDependency {
    pub axis: GridAxis,
    pub first: u32,
    pub last: u32,
}

impl GridAxisVisibilityDependency {
    #[must_use]
    pub fn rows(first: u32, last: u32) -> Self {
        Self {
            axis: GridAxis::Row,
            first: first.min(last),
            last: first.max(last),
        }
    }

    #[must_use]
    pub fn columns(first: u32, last: u32) -> Self {
        Self {
            axis: GridAxis::Column,
            first: first.min(last),
            last: first.max(last),
        }
    }

    #[must_use]
    pub fn index_count(&self) -> u64 {
        u64::from(self.last - self.first + 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GridAxisVisibilityIndexedDependency {
    dependent: ExcelGridCellAddress,
    dependency: GridAxisVisibilityDependency,
}

impl GridAxisVisibilityIndexedDependency {
    fn new(dependent: ExcelGridCellAddress, dependency: GridAxisVisibilityDependency) -> Self {
        Self {
            dependent,
            dependency,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridAxisValueDependency {
    pub axis: GridAxis,
    pub first: u32,
    pub last: u32,
}

impl GridAxisValueDependency {
    #[must_use]
    pub fn rows(first: u32, last: u32) -> Self {
        Self {
            axis: GridAxis::Row,
            first: first.min(last),
            last: first.max(last),
        }
    }

    #[must_use]
    pub fn columns(first: u32, last: u32) -> Self {
        Self {
            axis: GridAxis::Column,
            first: first.min(last),
            last: first.max(last),
        }
    }

    #[must_use]
    pub fn index_count(&self) -> u64 {
        u64::from(self.last - self.first + 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridSpillDependency {
    pub anchor: ExcelGridCellAddress,
}

impl GridSpillDependency {
    #[must_use]
    pub fn anchor(anchor: ExcelGridCellAddress) -> Self {
        Self { anchor }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridSpillBlockerDependency {
    pub extent: GridRect,
}

impl GridSpillBlockerDependency {
    #[must_use]
    pub fn extent(extent: GridRect) -> Self {
        Self { extent }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridNameDependency {
    pub name_key: String,
    pub extent: GridRect,
}

impl GridNameDependency {
    pub fn new(
        name: impl AsRef<str>,
        extent: GridRect,
        bounds: ExcelGridBounds,
    ) -> Result<Self, GridRefError> {
        let Some(name_key) = excel_grid_defined_name_key(name.as_ref(), bounds) else {
            return Err(GridRefError::InvalidDefinedName {
                name: name.as_ref().to_string(),
            });
        };
        Ok(Self { name_key, extent })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridTableDependency {
    pub table_key: String,
    pub extent: GridRect,
}

impl GridTableDependency {
    pub fn new(
        table_name: impl AsRef<str>,
        extent: GridRect,
        bounds: ExcelGridBounds,
    ) -> Result<Self, GridRefError> {
        let Some(table_key) = excel_grid_table_name_key(table_name.as_ref(), bounds) else {
            return Err(GridRefError::InvalidTableName {
                name: table_name.as_ref().to_string(),
            });
        };
        Ok(Self { table_key, extent })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridDependency {
    Cell(ExcelGridCellAddress),
    Range(GridRect),
    Name(GridNameDependency),
    Table(GridTableDependency),
    SpillFact(GridSpillDependency),
    SpillBlocker(GridSpillBlockerDependency),
    AxisVisibility(GridAxisVisibilityDependency),
    AxisValue(GridAxisValueDependency),
    DynamicRequest(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GridCompressedRangeDependency {
    dependent: ExcelGridCellAddress,
    extent: GridRect,
}

impl GridCompressedRangeDependency {
    fn new(dependent: ExcelGridCellAddress, extent: GridRect) -> Self {
        Self { dependent, extent }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GridScalarCellDependency {
    dependent: ExcelGridCellAddress,
    dependency: ExcelGridCellAddress,
}

impl GridScalarCellDependency {
    fn new(dependent: ExcelGridCellAddress, dependency: ExcelGridCellAddress) -> Self {
        Self {
            dependent,
            dependency,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridInvalidationRef {
    bounds: ExcelGridBounds,
    scalarization_limit: u64,
    semantic_dependencies_by_cell: BTreeMap<ExcelGridCellAddress, Vec<GridDependency>>,
    dependencies_by_cell: BTreeMap<ExcelGridCellAddress, BTreeSet<ExcelGridCellAddress>>,
    dependents_by_cell: BTreeMap<ExcelGridCellAddress, BTreeSet<ExcelGridCellAddress>>,
    scalar_dependents_by_block: BTreeMap<(u32, u32), BTreeSet<GridScalarCellDependency>>,
    compressed_range_dependencies_by_cell:
        BTreeMap<ExcelGridCellAddress, BTreeSet<GridCompressedRangeDependency>>,
    compressed_range_dependents: BTreeSet<GridCompressedRangeDependency>,
    compressed_range_dependents_by_block:
        BTreeMap<(u32, u32), BTreeSet<GridCompressedRangeDependency>>,
    spill_dependencies_by_cell: BTreeMap<ExcelGridCellAddress, BTreeSet<GridSpillDependency>>,
    spill_dependents_by_anchor: BTreeMap<ExcelGridCellAddress, BTreeSet<ExcelGridCellAddress>>,
    spill_blocker_dependencies_by_cell:
        BTreeMap<ExcelGridCellAddress, BTreeSet<GridSpillBlockerDependency>>,
    spill_blocker_dependents_by_cell:
        BTreeMap<ExcelGridCellAddress, BTreeSet<ExcelGridCellAddress>>,
    axis_visibility_dependencies_by_cell:
        BTreeMap<ExcelGridCellAddress, BTreeSet<GridAxisVisibilityDependency>>,
    axis_visibility_dependents_by_block:
        BTreeMap<(GridAxis, u32), BTreeSet<GridAxisVisibilityIndexedDependency>>,
    axis_value_dependencies_by_cell:
        BTreeMap<ExcelGridCellAddress, BTreeSet<GridAxisValueDependency>>,
    axis_value_dependents_by_index: BTreeMap<(GridAxis, u32), BTreeSet<ExcelGridCellAddress>>,
    name_dependencies_by_cell: BTreeMap<ExcelGridCellAddress, BTreeSet<GridNameDependency>>,
    name_dependents_by_key: BTreeMap<String, BTreeSet<ExcelGridCellAddress>>,
    table_dependencies_by_cell: BTreeMap<ExcelGridCellAddress, BTreeSet<GridTableDependency>>,
    table_dependents_by_key: BTreeMap<String, BTreeSet<ExcelGridCellAddress>>,
    dynamic_dependencies_by_cell: BTreeMap<ExcelGridCellAddress, BTreeSet<String>>,
    dynamic_dependents_by_request: BTreeMap<String, BTreeSet<ExcelGridCellAddress>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridInvalidationStructuralEditReport {
    pub edit: GridAxisEdit,
    pub dependent_cells_kept: usize,
    pub dependent_cells_dropped: usize,
    pub semantic_dependencies_kept: usize,
    pub semantic_dependencies_dropped: usize,
    pub scalar_edges_before: usize,
    pub scalar_edges_after: usize,
    pub compressed_range_edges_before: usize,
    pub compressed_range_edges_after: usize,
    pub spill_edges_before: usize,
    pub spill_edges_after: usize,
    pub spill_blocker_edges_before: usize,
    pub spill_blocker_edges_after: usize,
    pub axis_value_edges_before: usize,
    pub axis_value_edges_after: usize,
    pub name_edges_before: usize,
    pub name_edges_after: usize,
    pub table_edges_before: usize,
    pub table_edges_after: usize,
    pub dynamic_edges_before: usize,
    pub dynamic_edges_after: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridInvalidationNamespaceLifecycleOperation {
    RenameName {
        old_name_key: String,
        new_name_key: String,
    },
    DeleteName {
        name_key: String,
    },
    RenameTable {
        old_table_key: String,
        new_table_key: String,
    },
    DeleteTable {
        table_key: String,
    },
    ResizeTable {
        table_key: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridInvalidationNamespaceLifecycleReport {
    pub operation: GridInvalidationNamespaceLifecycleOperation,
    pub dirty_closure: BTreeSet<ExcelGridCellAddress>,
    pub semantic_dependencies_kept: usize,
    pub semantic_dependencies_dropped: usize,
    pub name_edges_before: usize,
    pub name_edges_after: usize,
    pub table_edges_before: usize,
    pub table_edges_after: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridCompressedRangeQueryReport {
    pub seed: ExcelGridCellAddress,
    pub indexed_candidate_count: usize,
    pub matched_dependent_count: usize,
    pub total_compressed_range_edges: usize,
    pub dependents: BTreeSet<ExcelGridCellAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridDirtyRectQueryReport {
    pub rect: GridRect,
    pub seed_rect_cell_count: u64,
    pub indexed_scalar_candidate_count: usize,
    pub matched_scalar_dependent_count: usize,
    pub indexed_compressed_range_candidate_count: usize,
    pub matched_compressed_range_dependent_count: usize,
    pub total_scalar_edges: usize,
    pub total_compressed_range_edges: usize,
    pub direct_dependents: BTreeSet<ExcelGridCellAddress>,
    pub dirty_closure: BTreeSet<ExcelGridCellAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridAxisVisibilityQueryReport {
    pub dependency: GridAxisVisibilityDependency,
    pub indexed_candidate_count: usize,
    pub matched_dependent_count: usize,
    pub total_axis_visibility_edges: usize,
    pub dependents: BTreeSet<ExcelGridCellAddress>,
}

impl GridInvalidationRef {
    #[must_use]
    pub fn new(bounds: ExcelGridBounds) -> Self {
        Self::with_scalarization_limit(bounds, GRID_INVALIDATION_REF_DEFAULT_SCALARIZATION_LIMIT)
    }

    #[must_use]
    pub fn with_scalarization_limit(bounds: ExcelGridBounds, scalarization_limit: u64) -> Self {
        Self {
            bounds,
            scalarization_limit,
            semantic_dependencies_by_cell: BTreeMap::new(),
            dependencies_by_cell: BTreeMap::new(),
            dependents_by_cell: BTreeMap::new(),
            scalar_dependents_by_block: BTreeMap::new(),
            compressed_range_dependencies_by_cell: BTreeMap::new(),
            compressed_range_dependents: BTreeSet::new(),
            compressed_range_dependents_by_block: BTreeMap::new(),
            spill_dependencies_by_cell: BTreeMap::new(),
            spill_dependents_by_anchor: BTreeMap::new(),
            spill_blocker_dependencies_by_cell: BTreeMap::new(),
            spill_blocker_dependents_by_cell: BTreeMap::new(),
            axis_visibility_dependencies_by_cell: BTreeMap::new(),
            axis_visibility_dependents_by_block: BTreeMap::new(),
            axis_value_dependencies_by_cell: BTreeMap::new(),
            axis_value_dependents_by_index: BTreeMap::new(),
            name_dependencies_by_cell: BTreeMap::new(),
            name_dependents_by_key: BTreeMap::new(),
            table_dependencies_by_cell: BTreeMap::new(),
            table_dependents_by_key: BTreeMap::new(),
            dynamic_dependencies_by_cell: BTreeMap::new(),
            dynamic_dependents_by_request: BTreeMap::new(),
        }
    }

    #[must_use]
    pub const fn bounds(&self) -> ExcelGridBounds {
        self.bounds
    }

    pub fn set_cell_dependencies(
        &mut self,
        dependent: ExcelGridCellAddress,
        dependencies: impl IntoIterator<Item = GridDependency>,
    ) -> Result<usize, GridRefError> {
        self.check_address(&dependent)?;
        self.remove_existing_dependencies(&dependent);

        let mut semantic_dependencies = Vec::new();
        let mut scalar_dependencies = BTreeSet::new();
        let mut compressed_range_dependencies = BTreeSet::new();
        let mut spill_dependencies = BTreeSet::new();
        let mut spill_blocker_dependencies = BTreeSet::new();
        let mut axis_visibility_dependencies = BTreeSet::new();
        let mut axis_value_dependencies = BTreeSet::new();
        let mut name_dependencies = BTreeSet::new();
        let mut table_dependencies = BTreeSet::new();
        let mut dynamic_dependencies = BTreeSet::new();

        for dependency in dependencies {
            match dependency {
                GridDependency::Cell(address) => {
                    self.check_address(&address)?;
                    semantic_dependencies.push(GridDependency::Cell(address.clone()));
                    scalar_dependencies.insert(address);
                }
                GridDependency::Range(rect) => {
                    self.check_rect(&rect)?;
                    self.maybe_scalarize_rect(&rect, &mut scalar_dependencies)?;
                    compressed_range_dependencies.insert(GridCompressedRangeDependency::new(
                        dependent.clone(),
                        rect.clone(),
                    ));
                    semantic_dependencies.push(GridDependency::Range(rect));
                }
                GridDependency::Name(dependency) => {
                    self.check_rect(&dependency.extent)?;
                    self.maybe_scalarize_rect(&dependency.extent, &mut scalar_dependencies)?;
                    compressed_range_dependencies.insert(GridCompressedRangeDependency::new(
                        dependent.clone(),
                        dependency.extent.clone(),
                    ));
                    semantic_dependencies.push(GridDependency::Name(dependency.clone()));
                    name_dependencies.insert(dependency);
                }
                GridDependency::Table(dependency) => {
                    self.check_rect(&dependency.extent)?;
                    self.maybe_scalarize_rect(&dependency.extent, &mut scalar_dependencies)?;
                    compressed_range_dependencies.insert(GridCompressedRangeDependency::new(
                        dependent.clone(),
                        dependency.extent.clone(),
                    ));
                    semantic_dependencies.push(GridDependency::Table(dependency.clone()));
                    table_dependencies.insert(dependency);
                }
                GridDependency::SpillFact(dependency) => {
                    self.check_address(&dependency.anchor)?;
                    semantic_dependencies.push(GridDependency::SpillFact(dependency.clone()));
                    spill_dependencies.insert(dependency);
                }
                GridDependency::SpillBlocker(dependency) => {
                    self.check_rect(&dependency.extent)?;
                    dependency.extent.scalar_cells(self.scalarization_limit)?;
                    semantic_dependencies.push(GridDependency::SpillBlocker(dependency.clone()));
                    spill_blocker_dependencies.insert(dependency);
                }
                GridDependency::AxisVisibility(dependency) => {
                    self.check_axis_visibility_dependency(&dependency)?;
                    semantic_dependencies.push(GridDependency::AxisVisibility(dependency.clone()));
                    axis_visibility_dependencies.insert(dependency);
                }
                GridDependency::AxisValue(dependency) => {
                    self.check_axis_value_dependency(&dependency)?;
                    semantic_dependencies.push(GridDependency::AxisValue(dependency.clone()));
                    axis_value_dependencies.insert(dependency);
                }
                GridDependency::DynamicRequest(request_key) => {
                    semantic_dependencies.push(GridDependency::DynamicRequest(request_key.clone()));
                    dynamic_dependencies.insert(request_key);
                }
            }
        }

        for dependency in &scalar_dependencies {
            self.dependents_by_cell
                .entry(dependency.clone())
                .or_default()
                .insert(dependent.clone());
        }
        for dependency in &compressed_range_dependencies {
            self.compressed_range_dependents.insert(dependency.clone());
            self.insert_compressed_range_dependency_into_blocks(dependency);
        }
        for dependency in &spill_dependencies {
            self.spill_dependents_by_anchor
                .entry(dependency.anchor.clone())
                .or_default()
                .insert(dependent.clone());
        }
        for dependency in &spill_blocker_dependencies {
            for address in dependency.extent.scalar_cells(self.scalarization_limit)? {
                self.spill_blocker_dependents_by_cell
                    .entry(address)
                    .or_default()
                    .insert(dependent.clone());
            }
        }
        for dependency in &axis_value_dependencies {
            for index in dependency.first..=dependency.last {
                self.axis_value_dependents_by_index
                    .entry((dependency.axis, index))
                    .or_default()
                    .insert(dependent.clone());
            }
        }
        for dependency in &name_dependencies {
            self.name_dependents_by_key
                .entry(dependency.name_key.clone())
                .or_default()
                .insert(dependent.clone());
        }
        for dependency in &table_dependencies {
            self.table_dependents_by_key
                .entry(dependency.table_key.clone())
                .or_default()
                .insert(dependent.clone());
        }
        for request_key in &dynamic_dependencies {
            self.dynamic_dependents_by_request
                .entry(request_key.clone())
                .or_default()
                .insert(dependent.clone());
        }
        for dependency in &axis_visibility_dependencies {
            self.insert_axis_visibility_dependency_into_blocks(&dependent, dependency);
        }
        for dependency in &scalar_dependencies {
            self.insert_scalar_dependency_into_blocks(&dependent, dependency);
        }

        let edge_count = scalar_dependencies.len();
        if scalar_dependencies.is_empty() {
            self.dependencies_by_cell.remove(&dependent);
        } else {
            self.dependencies_by_cell
                .insert(dependent.clone(), scalar_dependencies);
        }
        if compressed_range_dependencies.is_empty() {
            self.compressed_range_dependencies_by_cell
                .remove(&dependent);
        } else {
            self.compressed_range_dependencies_by_cell
                .insert(dependent.clone(), compressed_range_dependencies);
        }
        if spill_dependencies.is_empty() {
            self.spill_dependencies_by_cell.remove(&dependent);
        } else {
            self.spill_dependencies_by_cell
                .insert(dependent.clone(), spill_dependencies);
        }
        if spill_blocker_dependencies.is_empty() {
            self.spill_blocker_dependencies_by_cell.remove(&dependent);
        } else {
            self.spill_blocker_dependencies_by_cell
                .insert(dependent.clone(), spill_blocker_dependencies);
        }
        if dynamic_dependencies.is_empty() {
            self.dynamic_dependencies_by_cell.remove(&dependent);
        } else {
            self.dynamic_dependencies_by_cell
                .insert(dependent.clone(), dynamic_dependencies);
        }
        if axis_visibility_dependencies.is_empty() {
            self.axis_visibility_dependencies_by_cell.remove(&dependent);
        } else {
            self.axis_visibility_dependencies_by_cell
                .insert(dependent.clone(), axis_visibility_dependencies);
        }
        if axis_value_dependencies.is_empty() {
            self.axis_value_dependencies_by_cell.remove(&dependent);
        } else {
            self.axis_value_dependencies_by_cell
                .insert(dependent.clone(), axis_value_dependencies);
        }
        if name_dependencies.is_empty() {
            self.name_dependencies_by_cell.remove(&dependent);
        } else {
            self.name_dependencies_by_cell
                .insert(dependent.clone(), name_dependencies);
        }
        if table_dependencies.is_empty() {
            self.table_dependencies_by_cell.remove(&dependent);
        } else {
            self.table_dependencies_by_cell
                .insert(dependent.clone(), table_dependencies);
        }
        if semantic_dependencies.is_empty() {
            self.semantic_dependencies_by_cell.remove(&dependent);
        } else {
            self.semantic_dependencies_by_cell
                .insert(dependent, semantic_dependencies);
        }
        Ok(edge_count)
    }

    pub fn apply_axis_edit(
        &mut self,
        edit: GridAxisEdit,
    ) -> Result<GridInvalidationStructuralEditReport, GridRefError> {
        validate_axis_edit(edit, self.bounds)?;

        let scalar_edges_before = self.scalar_edge_count();
        let compressed_range_edges_before = self.compressed_range_edge_count();
        let spill_edges_before = self.spill_edge_count();
        let spill_blocker_edges_before = self.spill_blocker_edge_count();
        let axis_value_edges_before = self.axis_value_edge_count();
        let name_edges_before = self.name_edge_count();
        let table_edges_before = self.table_edge_count();
        let dynamic_edges_before = self.dynamic_edge_count();
        let old_semantic = self.semantic_dependencies_by_cell.clone();
        let mut transformed_semantic = Vec::new();
        let mut dependent_cells_kept = 0;
        let mut dependent_cells_dropped = 0;
        let mut semantic_dependencies_kept = 0;
        let mut semantic_dependencies_dropped = 0;

        for (dependent, dependencies) in old_semantic {
            let Some(transformed_dependent) =
                transform_address_for_edit(&dependent, edit, self.bounds)?
            else {
                dependent_cells_dropped += 1;
                semantic_dependencies_dropped += dependencies.len();
                continue;
            };

            let mut transformed_dependencies = Vec::new();
            for dependency in dependencies {
                match transform_dependency_for_axis_edit(&dependency, edit, self.bounds)? {
                    Some(transformed_dependency) => {
                        transformed_dependencies.push(transformed_dependency);
                        semantic_dependencies_kept += 1;
                    }
                    None => {
                        semantic_dependencies_dropped += 1;
                    }
                }
            }

            dependent_cells_kept += 1;
            transformed_semantic.push((transformed_dependent, transformed_dependencies));
        }

        let mut rebuilt = Self::with_scalarization_limit(self.bounds, self.scalarization_limit);
        for (dependent, dependencies) in transformed_semantic {
            rebuilt.set_cell_dependencies(dependent, dependencies)?;
        }
        let scalar_edges_after = rebuilt.scalar_edge_count();
        let compressed_range_edges_after = rebuilt.compressed_range_edge_count();
        let spill_edges_after = rebuilt.spill_edge_count();
        let spill_blocker_edges_after = rebuilt.spill_blocker_edge_count();
        let axis_value_edges_after = rebuilt.axis_value_edge_count();
        let name_edges_after = rebuilt.name_edge_count();
        let table_edges_after = rebuilt.table_edge_count();
        let dynamic_edges_after = rebuilt.dynamic_edge_count();
        *self = rebuilt;

        Ok(GridInvalidationStructuralEditReport {
            edit,
            dependent_cells_kept,
            dependent_cells_dropped,
            semantic_dependencies_kept,
            semantic_dependencies_dropped,
            scalar_edges_before,
            scalar_edges_after,
            compressed_range_edges_before,
            compressed_range_edges_after,
            spill_edges_before,
            spill_edges_after,
            spill_blocker_edges_before,
            spill_blocker_edges_after,
            axis_value_edges_before,
            axis_value_edges_after,
            name_edges_before,
            name_edges_after,
            table_edges_before,
            table_edges_after,
            dynamic_edges_before,
            dynamic_edges_after,
        })
    }

    pub fn rename_defined_name(
        &mut self,
        old_name: impl AsRef<str>,
        new_name: impl AsRef<str>,
    ) -> Result<GridInvalidationNamespaceLifecycleReport, GridRefError> {
        let old_name_key = defined_name_key_for_name(old_name.as_ref(), self.bounds)?;
        let new_name_key = defined_name_key_for_name(new_name.as_ref(), self.bounds)?;
        let dirty_closure = self.dirty_closure_for_name_keys([&old_name_key, &new_name_key]);
        let name_edges_before = self.name_edge_count();
        let table_edges_before = self.table_edge_count();
        let (transformed_semantic, kept, dropped) =
            self.transform_namespace_dependencies(|dependency| match dependency {
                GridDependency::Name(mut dependency) if dependency.name_key == old_name_key => {
                    dependency.name_key = new_name_key.clone();
                    Some(GridDependency::Name(dependency))
                }
                other => Some(other),
            });
        self.replace_semantic_dependencies(transformed_semantic)?;
        Ok(GridInvalidationNamespaceLifecycleReport {
            operation: GridInvalidationNamespaceLifecycleOperation::RenameName {
                old_name_key,
                new_name_key,
            },
            dirty_closure,
            semantic_dependencies_kept: kept,
            semantic_dependencies_dropped: dropped,
            name_edges_before,
            name_edges_after: self.name_edge_count(),
            table_edges_before,
            table_edges_after: self.table_edge_count(),
        })
    }

    pub fn delete_defined_name(
        &mut self,
        name: impl AsRef<str>,
    ) -> Result<GridInvalidationNamespaceLifecycleReport, GridRefError> {
        let name_key = defined_name_key_for_name(name.as_ref(), self.bounds)?;
        let dirty_closure = self.dirty_closure_for_name_keys([&name_key]);
        let name_edges_before = self.name_edge_count();
        let table_edges_before = self.table_edge_count();
        let (transformed_semantic, kept, dropped) =
            self.transform_namespace_dependencies(|dependency| match dependency {
                GridDependency::Name(dependency) if dependency.name_key == name_key => None,
                other => Some(other),
            });
        self.replace_semantic_dependencies(transformed_semantic)?;
        Ok(GridInvalidationNamespaceLifecycleReport {
            operation: GridInvalidationNamespaceLifecycleOperation::DeleteName { name_key },
            dirty_closure,
            semantic_dependencies_kept: kept,
            semantic_dependencies_dropped: dropped,
            name_edges_before,
            name_edges_after: self.name_edge_count(),
            table_edges_before,
            table_edges_after: self.table_edge_count(),
        })
    }

    pub fn rename_table(
        &mut self,
        old_table_name: impl AsRef<str>,
        new_table_name: impl AsRef<str>,
    ) -> Result<GridInvalidationNamespaceLifecycleReport, GridRefError> {
        let old_table_key = table_key_for_name(old_table_name.as_ref(), self.bounds)?;
        let new_table_key = table_key_for_name(new_table_name.as_ref(), self.bounds)?;
        let dirty_closure = self.dirty_closure_for_table_keys([&old_table_key, &new_table_key]);
        let name_edges_before = self.name_edge_count();
        let table_edges_before = self.table_edge_count();
        let (transformed_semantic, kept, dropped) =
            self.transform_namespace_dependencies(|dependency| match dependency {
                GridDependency::Table(mut dependency) if dependency.table_key == old_table_key => {
                    dependency.table_key = new_table_key.clone();
                    Some(GridDependency::Table(dependency))
                }
                other => Some(other),
            });
        self.replace_semantic_dependencies(transformed_semantic)?;
        Ok(GridInvalidationNamespaceLifecycleReport {
            operation: GridInvalidationNamespaceLifecycleOperation::RenameTable {
                old_table_key,
                new_table_key,
            },
            dirty_closure,
            semantic_dependencies_kept: kept,
            semantic_dependencies_dropped: dropped,
            name_edges_before,
            name_edges_after: self.name_edge_count(),
            table_edges_before,
            table_edges_after: self.table_edge_count(),
        })
    }

    pub fn delete_table(
        &mut self,
        table_name: impl AsRef<str>,
    ) -> Result<GridInvalidationNamespaceLifecycleReport, GridRefError> {
        let table_key = table_key_for_name(table_name.as_ref(), self.bounds)?;
        let dirty_closure = self.dirty_closure_for_table_keys([&table_key]);
        let name_edges_before = self.name_edge_count();
        let table_edges_before = self.table_edge_count();
        let (transformed_semantic, kept, dropped) =
            self.transform_namespace_dependencies(|dependency| match dependency {
                GridDependency::Table(dependency) if dependency.table_key == table_key => None,
                other => Some(other),
            });
        self.replace_semantic_dependencies(transformed_semantic)?;
        Ok(GridInvalidationNamespaceLifecycleReport {
            operation: GridInvalidationNamespaceLifecycleOperation::DeleteTable { table_key },
            dirty_closure,
            semantic_dependencies_kept: kept,
            semantic_dependencies_dropped: dropped,
            name_edges_before,
            name_edges_after: self.name_edge_count(),
            table_edges_before,
            table_edges_after: self.table_edge_count(),
        })
    }

    pub fn resize_table(
        &mut self,
        table_name: impl AsRef<str>,
        new_extent: GridRect,
    ) -> Result<GridInvalidationNamespaceLifecycleReport, GridRefError> {
        self.check_rect(&new_extent)?;
        let table_key = table_key_for_name(table_name.as_ref(), self.bounds)?;
        let dirty_closure = self.dirty_closure_for_table_keys([&table_key]);
        let name_edges_before = self.name_edge_count();
        let table_edges_before = self.table_edge_count();
        let (transformed_semantic, kept, dropped) =
            self.transform_namespace_dependencies(|dependency| match dependency {
                GridDependency::Table(mut dependency) if dependency.table_key == table_key => {
                    dependency.extent = new_extent.clone();
                    Some(GridDependency::Table(dependency))
                }
                other => Some(other),
            });
        self.replace_semantic_dependencies(transformed_semantic)?;
        Ok(GridInvalidationNamespaceLifecycleReport {
            operation: GridInvalidationNamespaceLifecycleOperation::ResizeTable { table_key },
            dirty_closure,
            semantic_dependencies_kept: kept,
            semantic_dependencies_dropped: dropped,
            name_edges_before,
            name_edges_after: self.name_edge_count(),
            table_edges_before,
            table_edges_after: self.table_edge_count(),
        })
    }

    #[must_use]
    pub fn dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<ExcelGridCellAddress> {
        self.dependencies_by_cell
            .get(dependent)
            .cloned()
            .unwrap_or_default()
    }

    #[must_use]
    pub fn semantic_dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> Vec<GridDependency> {
        self.semantic_dependencies_by_cell
            .get(dependent)
            .cloned()
            .unwrap_or_default()
    }

    #[must_use]
    pub fn scalar_edge_count(&self) -> usize {
        self.dependencies_by_cell.values().map(BTreeSet::len).sum()
    }

    #[must_use]
    pub fn compressed_range_edge_count(&self) -> usize {
        self.compressed_range_dependencies_by_cell
            .values()
            .map(BTreeSet::len)
            .sum()
    }

    #[must_use]
    pub fn compressed_range_dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<GridRect> {
        self.compressed_range_dependencies_by_cell
            .get(dependent)
            .into_iter()
            .flat_map(|dependencies| {
                dependencies
                    .iter()
                    .map(|dependency| dependency.extent.clone())
            })
            .collect()
    }

    pub fn compressed_range_query_report(
        &self,
        seed: ExcelGridCellAddress,
    ) -> Result<GridCompressedRangeQueryReport, GridRefError> {
        self.check_address(&seed)?;
        let candidates = self.compressed_range_candidates_for(&seed);
        let dependents = candidates
            .iter()
            .filter(|dependency| dependency.extent.contains(&seed))
            .map(|dependency| dependency.dependent.clone())
            .collect::<BTreeSet<_>>();
        Ok(GridCompressedRangeQueryReport {
            seed,
            indexed_candidate_count: candidates.len(),
            matched_dependent_count: dependents.len(),
            total_compressed_range_edges: self.compressed_range_edge_count(),
            dependents,
        })
    }

    pub fn dirty_rect_query_report(
        &self,
        rect: GridRect,
    ) -> Result<GridDirtyRectQueryReport, GridRefError> {
        self.check_rect(&rect)?;
        let scalar_candidates = self.scalar_candidates_for_rect(&rect);
        let scalar_dependents = scalar_candidates
            .iter()
            .filter(|dependency| rect.contains(&dependency.dependency))
            .map(|dependency| dependency.dependent.clone())
            .collect::<BTreeSet<_>>();
        let compressed_range_candidates = self.compressed_range_candidates_for_rect(&rect);
        let compressed_range_dependents = compressed_range_candidates
            .iter()
            .filter(|dependency| grid_rects_overlap(&dependency.extent, &rect))
            .map(|dependency| dependency.dependent.clone())
            .collect::<BTreeSet<_>>();
        let mut direct_dependents = scalar_dependents.clone();
        direct_dependents.extend(compressed_range_dependents.iter().cloned());
        let dirty_closure = self.close_over_dependents(direct_dependents.iter().cloned());

        Ok(GridDirtyRectQueryReport {
            seed_rect_cell_count: rect.cell_count(),
            indexed_scalar_candidate_count: scalar_candidates.len(),
            matched_scalar_dependent_count: scalar_dependents.len(),
            indexed_compressed_range_candidate_count: compressed_range_candidates.len(),
            matched_compressed_range_dependent_count: compressed_range_dependents.len(),
            total_scalar_edges: self.scalar_edge_count(),
            total_compressed_range_edges: self.compressed_range_edge_count(),
            rect,
            direct_dependents,
            dirty_closure,
        })
    }

    #[must_use]
    pub fn spill_edge_count(&self) -> usize {
        self.spill_dependencies_by_cell
            .values()
            .map(BTreeSet::len)
            .sum()
    }

    #[must_use]
    pub fn spill_blocker_edge_count(&self) -> usize {
        self.spill_blocker_dependencies_by_cell
            .values()
            .map(BTreeSet::len)
            .sum()
    }

    #[must_use]
    pub fn axis_value_edge_count(&self) -> usize {
        self.axis_value_dependencies_by_cell
            .values()
            .map(BTreeSet::len)
            .sum()
    }

    #[must_use]
    pub fn name_edge_count(&self) -> usize {
        self.name_dependencies_by_cell
            .values()
            .map(BTreeSet::len)
            .sum()
    }

    #[must_use]
    pub fn table_edge_count(&self) -> usize {
        self.table_dependencies_by_cell
            .values()
            .map(BTreeSet::len)
            .sum()
    }

    #[must_use]
    pub fn dynamic_edge_count(&self) -> usize {
        self.dynamic_dependencies_by_cell
            .values()
            .map(BTreeSet::len)
            .sum()
    }

    #[must_use]
    pub fn dynamic_dependencies_for(&self, dependent: &ExcelGridCellAddress) -> BTreeSet<String> {
        self.dynamic_dependencies_by_cell
            .get(dependent)
            .cloned()
            .unwrap_or_default()
    }

    #[must_use]
    pub fn spill_dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<GridSpillDependency> {
        self.spill_dependencies_by_cell
            .get(dependent)
            .cloned()
            .unwrap_or_default()
    }

    #[must_use]
    pub fn spill_blocker_dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<GridSpillBlockerDependency> {
        self.spill_blocker_dependencies_by_cell
            .get(dependent)
            .cloned()
            .unwrap_or_default()
    }

    #[must_use]
    pub fn axis_visibility_dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<GridAxisVisibilityDependency> {
        self.axis_visibility_dependencies_by_cell
            .get(dependent)
            .cloned()
            .unwrap_or_default()
    }

    #[must_use]
    pub fn axis_visibility_edge_count(&self) -> usize {
        self.axis_visibility_dependencies_by_cell
            .values()
            .map(BTreeSet::len)
            .sum()
    }

    pub fn axis_visibility_query_report(
        &self,
        dependency: GridAxisVisibilityDependency,
    ) -> Result<GridAxisVisibilityQueryReport, GridRefError> {
        self.check_axis_visibility_dependency(&dependency)?;
        let candidates = self.axis_visibility_candidates_for(&dependency);
        let dependents = candidates
            .iter()
            .filter(|candidate| {
                axis_visibility_dependencies_intersect(&candidate.dependency, &dependency)
            })
            .map(|candidate| candidate.dependent.clone())
            .collect::<BTreeSet<_>>();
        Ok(GridAxisVisibilityQueryReport {
            dependency,
            indexed_candidate_count: candidates.len(),
            matched_dependent_count: dependents.len(),
            total_axis_visibility_edges: self.axis_visibility_edge_count(),
            dependents,
        })
    }

    #[must_use]
    pub fn axis_value_dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<GridAxisValueDependency> {
        self.axis_value_dependencies_by_cell
            .get(dependent)
            .cloned()
            .unwrap_or_default()
    }

    #[must_use]
    pub fn name_dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<GridNameDependency> {
        self.name_dependencies_by_cell
            .get(dependent)
            .cloned()
            .unwrap_or_default()
    }

    #[must_use]
    pub fn table_dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<GridTableDependency> {
        self.table_dependencies_by_cell
            .get(dependent)
            .cloned()
            .unwrap_or_default()
    }

    #[must_use]
    pub fn dirty_closure(
        &self,
        seeds: impl IntoIterator<Item = ExcelGridCellAddress>,
    ) -> BTreeSet<ExcelGridCellAddress> {
        self.close_over_dependents(seeds)
    }

    #[must_use]
    pub fn dirty_closure_for_dynamic_request(
        &self,
        request_key: &str,
    ) -> BTreeSet<ExcelGridCellAddress> {
        let seeds = self
            .dynamic_dependents_by_request
            .get(request_key)
            .into_iter()
            .flat_map(|dependents| dependents.iter().cloned());
        self.close_over_dependents(seeds)
    }

    pub fn dirty_closure_for_spill_fact(
        &self,
        dependency: GridSpillDependency,
    ) -> Result<BTreeSet<ExcelGridCellAddress>, GridRefError> {
        self.check_address(&dependency.anchor)?;
        let seeds = self
            .spill_dependents_by_anchor
            .get(&dependency.anchor)
            .into_iter()
            .flat_map(|dependents| dependents.iter().cloned());
        Ok(self.close_over_dependents(seeds))
    }

    pub fn dirty_closure_for_spill_epoch_changes(
        &self,
        old_snapshots: impl IntoIterator<Item = GridSpillEpochSnapshot>,
        new_snapshots: impl IntoIterator<Item = GridSpillEpochSnapshot>,
    ) -> Result<GridSpillEpochInvalidationReport, GridRefError> {
        let old_by_anchor = spill_epoch_snapshot_map(old_snapshots, self.bounds)?;
        let new_by_anchor = spill_epoch_snapshot_map(new_snapshots, self.bounds)?;
        let anchors = old_by_anchor
            .keys()
            .chain(new_by_anchor.keys())
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut changed_anchors = Vec::new();
        let mut unchanged_anchors = 0;
        let mut extent_epoch_changed_anchors = 0;
        let mut value_epoch_changed_anchors = 0;
        let mut blocked_epoch_changed_anchors = 0;
        let mut dirty_closure = BTreeSet::new();

        for anchor in anchors {
            let old = old_by_anchor.get(&anchor);
            let new = new_by_anchor.get(&anchor);
            let Some(kind) = spill_epoch_change_kind(old, new) else {
                unchanged_anchors += 1;
                continue;
            };
            match kind {
                GridSpillEpochChangeKind::Added | GridSpillEpochChangeKind::Removed => {
                    extent_epoch_changed_anchors += 1;
                    value_epoch_changed_anchors += 1;
                    blocked_epoch_changed_anchors += 1;
                }
                GridSpillEpochChangeKind::ExtentChanged => {
                    extent_epoch_changed_anchors += 1;
                }
                GridSpillEpochChangeKind::ValueChanged => {
                    value_epoch_changed_anchors += 1;
                }
                GridSpillEpochChangeKind::BlockedChanged => {
                    blocked_epoch_changed_anchors += 1;
                }
                GridSpillEpochChangeKind::ExtentAndValueChanged => {
                    extent_epoch_changed_anchors += 1;
                    value_epoch_changed_anchors += 1;
                }
            }
            dirty_closure.extend(
                self.dirty_closure_for_spill_fact(GridSpillDependency::anchor(anchor.clone()))?,
            );
            changed_anchors.push(GridSpillEpochAnchorChange { anchor, kind });
        }

        Ok(GridSpillEpochInvalidationReport {
            anchors_compared: changed_anchors.len() + unchanged_anchors,
            changed_anchors,
            unchanged_anchors,
            extent_epoch_changed_anchors,
            value_epoch_changed_anchors,
            blocked_epoch_changed_anchors,
            dirty_closure,
        })
    }

    pub fn dirty_closure_for_spill_blocker(
        &self,
        dependency: GridSpillBlockerDependency,
    ) -> Result<BTreeSet<ExcelGridCellAddress>, GridRefError> {
        self.check_rect(&dependency.extent)?;
        let cells = dependency.extent.scalar_cells(self.scalarization_limit)?;
        let seeds = cells.into_iter().flat_map(|address| {
            self.spill_blocker_dependents_by_cell
                .get(&address)
                .into_iter()
                .flat_map(|dependents| dependents.iter().cloned())
        });
        Ok(self.close_over_dependents(seeds))
    }

    pub fn dirty_closure_for_axis_visibility(
        &self,
        dependency: GridAxisVisibilityDependency,
    ) -> Result<BTreeSet<ExcelGridCellAddress>, GridRefError> {
        let report = self.axis_visibility_query_report(dependency)?;
        Ok(self.close_over_dependents(report.dependents))
    }

    pub fn dirty_closure_for_axis_value(
        &self,
        dependency: GridAxisValueDependency,
    ) -> Result<BTreeSet<ExcelGridCellAddress>, GridRefError> {
        self.check_axis_value_dependency(&dependency)?;
        let seeds = (dependency.first..=dependency.last).flat_map(|index| {
            self.axis_value_dependents_by_index
                .get(&(dependency.axis, index))
                .into_iter()
                .flat_map(|dependents| dependents.iter().cloned())
        });
        Ok(self.close_over_dependents(seeds))
    }

    pub fn dirty_closure_for_name(
        &self,
        name: impl AsRef<str>,
    ) -> Result<BTreeSet<ExcelGridCellAddress>, GridRefError> {
        let Some(name_key) = excel_grid_defined_name_key(name.as_ref(), self.bounds) else {
            return Err(GridRefError::InvalidDefinedName {
                name: name.as_ref().to_string(),
            });
        };
        let seeds = self
            .name_dependents_by_key
            .get(&name_key)
            .into_iter()
            .flat_map(|dependents| dependents.iter().cloned());
        Ok(self.close_over_dependents(seeds))
    }

    pub fn dirty_closure_for_table(
        &self,
        table_name: impl AsRef<str>,
    ) -> Result<BTreeSet<ExcelGridCellAddress>, GridRefError> {
        let Some(table_key) = excel_grid_table_name_key(table_name.as_ref(), self.bounds) else {
            return Err(GridRefError::InvalidTableName {
                name: table_name.as_ref().to_string(),
            });
        };
        let seeds = self
            .table_dependents_by_key
            .get(&table_key)
            .into_iter()
            .flat_map(|dependents| dependents.iter().cloned());
        Ok(self.close_over_dependents(seeds))
    }

    fn remove_existing_dependencies(&mut self, dependent: &ExcelGridCellAddress) {
        self.semantic_dependencies_by_cell.remove(dependent);
        if let Some(existing) = self.dependencies_by_cell.remove(dependent) {
            for dependency in existing {
                if let Some(dependents) = self.dependents_by_cell.get_mut(&dependency) {
                    dependents.remove(dependent);
                    if dependents.is_empty() {
                        self.dependents_by_cell.remove(&dependency);
                    }
                }
                self.remove_scalar_dependency_from_blocks(dependent, &dependency);
            }
        }
        if let Some(existing) = self.compressed_range_dependencies_by_cell.remove(dependent) {
            for dependency in existing {
                self.compressed_range_dependents.remove(&dependency);
                self.remove_compressed_range_dependency_from_blocks(&dependency);
            }
        }
        if let Some(existing) = self.spill_dependencies_by_cell.remove(dependent) {
            for dependency in existing {
                if let Some(dependents) =
                    self.spill_dependents_by_anchor.get_mut(&dependency.anchor)
                {
                    dependents.remove(dependent);
                    if dependents.is_empty() {
                        self.spill_dependents_by_anchor.remove(&dependency.anchor);
                    }
                }
            }
        }
        if let Some(existing) = self.spill_blocker_dependencies_by_cell.remove(dependent) {
            for dependency in existing {
                for address in scalar_cells_unchecked(&dependency.extent) {
                    let should_remove = if let Some(dependents) =
                        self.spill_blocker_dependents_by_cell.get_mut(&address)
                    {
                        dependents.remove(dependent);
                        dependents.is_empty()
                    } else {
                        false
                    };
                    if should_remove {
                        self.spill_blocker_dependents_by_cell.remove(&address);
                    }
                }
            }
        }
        if let Some(existing) = self.axis_visibility_dependencies_by_cell.remove(dependent) {
            for dependency in existing {
                self.remove_axis_visibility_dependency_from_blocks(dependent, &dependency);
            }
        }
        if let Some(existing) = self.axis_value_dependencies_by_cell.remove(dependent) {
            for dependency in existing {
                for index in dependency.first..=dependency.last {
                    let key = (dependency.axis, index);
                    let should_remove = if let Some(dependents) =
                        self.axis_value_dependents_by_index.get_mut(&key)
                    {
                        dependents.remove(dependent);
                        dependents.is_empty()
                    } else {
                        false
                    };
                    if should_remove {
                        self.axis_value_dependents_by_index.remove(&key);
                    }
                }
            }
        }
        if let Some(existing) = self.name_dependencies_by_cell.remove(dependent) {
            for dependency in existing {
                if let Some(dependents) = self.name_dependents_by_key.get_mut(&dependency.name_key)
                {
                    dependents.remove(dependent);
                    if dependents.is_empty() {
                        self.name_dependents_by_key.remove(&dependency.name_key);
                    }
                }
            }
        }
        if let Some(existing) = self.table_dependencies_by_cell.remove(dependent) {
            for dependency in existing {
                if let Some(dependents) =
                    self.table_dependents_by_key.get_mut(&dependency.table_key)
                {
                    dependents.remove(dependent);
                    if dependents.is_empty() {
                        self.table_dependents_by_key.remove(&dependency.table_key);
                    }
                }
            }
        }
        if let Some(existing) = self.dynamic_dependencies_by_cell.remove(dependent) {
            for request_key in existing {
                if let Some(dependents) = self.dynamic_dependents_by_request.get_mut(&request_key) {
                    dependents.remove(dependent);
                    if dependents.is_empty() {
                        self.dynamic_dependents_by_request.remove(&request_key);
                    }
                }
            }
        }
    }

    fn dirty_closure_for_name_keys<'a>(
        &self,
        keys: impl IntoIterator<Item = &'a String>,
    ) -> BTreeSet<ExcelGridCellAddress> {
        let seeds = keys.into_iter().flat_map(|key| {
            self.name_dependents_by_key
                .get(key)
                .into_iter()
                .flat_map(|dependents| dependents.iter().cloned())
        });
        self.close_over_dependents(seeds)
    }

    fn dirty_closure_for_table_keys<'a>(
        &self,
        keys: impl IntoIterator<Item = &'a String>,
    ) -> BTreeSet<ExcelGridCellAddress> {
        let seeds = keys.into_iter().flat_map(|key| {
            self.table_dependents_by_key
                .get(key)
                .into_iter()
                .flat_map(|dependents| dependents.iter().cloned())
        });
        self.close_over_dependents(seeds)
    }

    fn transform_namespace_dependencies(
        &self,
        mut transform: impl FnMut(GridDependency) -> Option<GridDependency>,
    ) -> (
        Vec<(ExcelGridCellAddress, Vec<GridDependency>)>,
        usize,
        usize,
    ) {
        let mut transformed_semantic = Vec::new();
        let mut kept = 0;
        let mut dropped = 0;

        for (dependent, dependencies) in self.semantic_dependencies_by_cell.clone() {
            let mut transformed_dependencies = Vec::new();
            for dependency in dependencies {
                match transform(dependency) {
                    Some(transformed) => {
                        transformed_dependencies.push(transformed);
                        kept += 1;
                    }
                    None => {
                        dropped += 1;
                    }
                }
            }
            transformed_semantic.push((dependent, transformed_dependencies));
        }

        (transformed_semantic, kept, dropped)
    }

    fn replace_semantic_dependencies(
        &mut self,
        transformed_semantic: Vec<(ExcelGridCellAddress, Vec<GridDependency>)>,
    ) -> Result<(), GridRefError> {
        let mut rebuilt = Self::with_scalarization_limit(self.bounds, self.scalarization_limit);
        for (dependent, dependencies) in transformed_semantic {
            rebuilt.set_cell_dependencies(dependent, dependencies)?;
        }
        *self = rebuilt;
        Ok(())
    }

    fn close_over_dependents(
        &self,
        seeds: impl IntoIterator<Item = ExcelGridCellAddress>,
    ) -> BTreeSet<ExcelGridCellAddress> {
        let mut dirty = BTreeSet::new();
        let mut queue = VecDeque::new();

        for seed in seeds {
            if dirty.insert(seed.clone()) {
                queue.push_back(seed);
            }
        }

        while let Some(address) = queue.pop_front() {
            let compressed_range_dependents = self.compressed_range_dependents_containing(&address);
            let scalar_dependents = self
                .dependents_by_cell
                .get(&address)
                .into_iter()
                .flat_map(|dependents| dependents.iter());
            let row_dependents = self
                .axis_value_dependents_by_index
                .get(&(GridAxis::Row, address.row))
                .into_iter()
                .flat_map(|dependents| dependents.iter());
            let column_dependents = self
                .axis_value_dependents_by_index
                .get(&(GridAxis::Column, address.col))
                .into_iter()
                .flat_map(|dependents| dependents.iter());

            for dependent in scalar_dependents
                .chain(row_dependents)
                .chain(column_dependents)
            {
                if dirty.insert(dependent.clone()) {
                    queue.push_back(dependent.clone());
                }
            }
            for dependent in compressed_range_dependents {
                if dirty.insert(dependent.clone()) {
                    queue.push_back(dependent);
                }
            }
        }

        dirty
    }

    fn maybe_scalarize_rect(
        &self,
        rect: &GridRect,
        scalar_dependencies: &mut BTreeSet<ExcelGridCellAddress>,
    ) -> Result<(), GridRefError> {
        if rect.cell_count() <= self.scalarization_limit {
            for address in rect.scalar_cells(self.scalarization_limit)? {
                scalar_dependencies.insert(address);
            }
        }
        Ok(())
    }

    fn compressed_range_dependents_containing(
        &self,
        address: &ExcelGridCellAddress,
    ) -> BTreeSet<ExcelGridCellAddress> {
        self.compressed_range_candidates_for(address)
            .iter()
            .filter(|dependency| dependency.extent.contains(address))
            .map(|dependency| dependency.dependent.clone())
            .collect()
    }

    fn compressed_range_candidates_for(
        &self,
        address: &ExcelGridCellAddress,
    ) -> BTreeSet<GridCompressedRangeDependency> {
        self.compressed_range_dependents_by_block
            .get(&compressed_range_block_for_cell(address.row, address.col))
            .cloned()
            .unwrap_or_default()
    }

    fn scalar_candidates_for_rect(&self, rect: &GridRect) -> BTreeSet<GridScalarCellDependency> {
        compressed_range_blocks_for_rect(rect)
            .into_iter()
            .flat_map(|block| {
                self.scalar_dependents_by_block
                    .get(&block)
                    .into_iter()
                    .flat_map(|dependencies| dependencies.iter().cloned())
            })
            .collect()
    }

    fn compressed_range_candidates_for_rect(
        &self,
        rect: &GridRect,
    ) -> BTreeSet<GridCompressedRangeDependency> {
        compressed_range_blocks_for_rect(rect)
            .into_iter()
            .flat_map(|block| {
                self.compressed_range_dependents_by_block
                    .get(&block)
                    .into_iter()
                    .flat_map(|dependencies| dependencies.iter().cloned())
            })
            .collect()
    }

    fn insert_scalar_dependency_into_blocks(
        &mut self,
        dependent: &ExcelGridCellAddress,
        dependency: &ExcelGridCellAddress,
    ) {
        let indexed = GridScalarCellDependency::new(dependent.clone(), dependency.clone());
        self.scalar_dependents_by_block
            .entry(compressed_range_block_for_cell(
                dependency.row,
                dependency.col,
            ))
            .or_default()
            .insert(indexed);
    }

    fn remove_scalar_dependency_from_blocks(
        &mut self,
        dependent: &ExcelGridCellAddress,
        dependency: &ExcelGridCellAddress,
    ) {
        let block = compressed_range_block_for_cell(dependency.row, dependency.col);
        let indexed = GridScalarCellDependency::new(dependent.clone(), dependency.clone());
        let should_remove =
            if let Some(dependencies) = self.scalar_dependents_by_block.get_mut(&block) {
                dependencies.remove(&indexed);
                dependencies.is_empty()
            } else {
                false
            };
        if should_remove {
            self.scalar_dependents_by_block.remove(&block);
        }
    }

    fn insert_compressed_range_dependency_into_blocks(
        &mut self,
        dependency: &GridCompressedRangeDependency,
    ) {
        for block in compressed_range_blocks_for_rect(&dependency.extent) {
            self.compressed_range_dependents_by_block
                .entry(block)
                .or_default()
                .insert(dependency.clone());
        }
    }

    fn remove_compressed_range_dependency_from_blocks(
        &mut self,
        dependency: &GridCompressedRangeDependency,
    ) {
        for block in compressed_range_blocks_for_rect(&dependency.extent) {
            let should_remove = if let Some(dependencies) =
                self.compressed_range_dependents_by_block.get_mut(&block)
            {
                dependencies.remove(dependency);
                dependencies.is_empty()
            } else {
                false
            };
            if should_remove {
                self.compressed_range_dependents_by_block.remove(&block);
            }
        }
    }

    fn axis_visibility_candidates_for(
        &self,
        dependency: &GridAxisVisibilityDependency,
    ) -> BTreeSet<GridAxisVisibilityIndexedDependency> {
        axis_visibility_blocks_for_dependency(dependency)
            .into_iter()
            .flat_map(|block| {
                self.axis_visibility_dependents_by_block
                    .get(&block)
                    .into_iter()
                    .flat_map(|dependencies| dependencies.iter().cloned())
            })
            .collect()
    }

    fn insert_axis_visibility_dependency_into_blocks(
        &mut self,
        dependent: &ExcelGridCellAddress,
        dependency: &GridAxisVisibilityDependency,
    ) {
        let indexed =
            GridAxisVisibilityIndexedDependency::new(dependent.clone(), dependency.clone());
        for block in axis_visibility_blocks_for_dependency(dependency) {
            self.axis_visibility_dependents_by_block
                .entry(block)
                .or_default()
                .insert(indexed.clone());
        }
    }

    fn remove_axis_visibility_dependency_from_blocks(
        &mut self,
        dependent: &ExcelGridCellAddress,
        dependency: &GridAxisVisibilityDependency,
    ) {
        let indexed =
            GridAxisVisibilityIndexedDependency::new(dependent.clone(), dependency.clone());
        for block in axis_visibility_blocks_for_dependency(dependency) {
            let should_remove = if let Some(dependencies) =
                self.axis_visibility_dependents_by_block.get_mut(&block)
            {
                dependencies.remove(&indexed);
                dependencies.is_empty()
            } else {
                false
            };
            if should_remove {
                self.axis_visibility_dependents_by_block.remove(&block);
            }
        }
    }

    fn check_address(&self, address: &ExcelGridCellAddress) -> Result<(), GridRefError> {
        if !self.bounds.contains_row(address.row) || !self.bounds.contains_col(address.col) {
            return Err(GridRefError::AddressOutOfBounds {
                row: address.row,
                col: address.col,
                max_rows: self.bounds.max_rows,
                max_cols: self.bounds.max_cols,
            });
        }
        Ok(())
    }

    fn check_rect(&self, rect: &GridRect) -> Result<(), GridRefError> {
        if !self.bounds.contains_row(rect.top_row)
            || !self.bounds.contains_row(rect.bottom_row)
            || !self.bounds.contains_col(rect.left_col)
            || !self.bounds.contains_col(rect.right_col)
        {
            return Err(GridRefError::RangeOutOfBounds {
                top_row: rect.top_row,
                left_col: rect.left_col,
                bottom_row: rect.bottom_row,
                right_col: rect.right_col,
                max_rows: self.bounds.max_rows,
                max_cols: self.bounds.max_cols,
            });
        }
        Ok(())
    }

    fn check_axis_visibility_dependency(
        &self,
        dependency: &GridAxisVisibilityDependency,
    ) -> Result<(), GridRefError> {
        validate_axis_visibility_dependency(dependency, self.bounds)
    }

    fn check_axis_value_dependency(
        &self,
        dependency: &GridAxisValueDependency,
    ) -> Result<(), GridRefError> {
        validate_axis_value_dependency(dependency, self.bounds)
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
        use crate::excel_grid_reference::{
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
