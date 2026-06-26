//! Host-side grid context for the strict-excel-grid engines: the
//! GridHostInfoProvider that supplies hidden-row aggregate context, table
//! overlay and structured-reference registration, merged and
//! feature-rendered regions, and table/name lifecycle operations. Internal
//! to the machine; shares the machine's types via `use super::*`.

use super::*;

use crate::table_backing::{TableBacking, TableSpec};

#[derive(Debug, Clone)]
pub struct GridHostInfoProvider<'a> {
    /// Profile-pure shape resolver (spill extents only, no cell values) used to
    /// resolve the aggregate target's rect through the shared strict-grid
    /// coordinate implementation; hidden-row context then comes from
    /// `axis_state`. Same intentional sharing rationale as
    /// [`GridOptimizedValuation::shape_resolver`].
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

pub(super) fn register_table_overlay_references<'a>(
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

pub(super) fn table_data_rect(table: &GridTableOverlay) -> Option<GridRect> {
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

impl TableBacking for GridTableOverlay {
    fn table_spec(&self) -> TableSpec {
        TableSpec::new(self.to_table_descriptor())
    }
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

    pub(super) fn check_sheet(
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

    pub(super) fn transform_for_axis_edit(
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

pub(super) fn excel_grid_table_name_key(name: &str, _bounds: ExcelGridBounds) -> Option<String> {
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

pub(super) fn table_key_for_name(
    name: &str,
    bounds: ExcelGridBounds,
) -> Result<String, GridRefError> {
    excel_grid_table_name_key(name, bounds).ok_or_else(|| GridRefError::InvalidTableName {
        name: name.to_string(),
    })
}

pub(super) fn defined_name_key_for_name(
    name: &str,
    bounds: ExcelGridBounds,
) -> Result<String, GridRefError> {
    excel_grid_defined_name_key(name, bounds).ok_or_else(|| GridRefError::InvalidDefinedName {
        name: name.to_string(),
    })
}

pub(super) fn remove_table_overlay_feature_regions(
    regions: &mut Vec<FeatureRenderedRegion>,
    rect: &GridRect,
) -> usize {
    let before = regions.len();
    regions.retain(|region| !(region.feature_kind == "table-overlay" && region.rect == *rect));
    before - regions.len()
}

pub(super) fn grid_table_descriptor_catalog<'a>(
    tables: impl IntoIterator<Item = &'a GridTableOverlay>,
) -> Vec<TableDescriptor> {
    tables
        .into_iter()
        .map(|overlay| overlay.table_spec().descriptor)
        .collect()
}

pub(super) fn grid_table_caller_context<'a>(
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

pub(super) fn grid_rect_a1_ref(rect: &GridRect) -> String {
    format!(
        "{}{}:{}{}",
        grid_column_letters(rect.left_col),
        rect.top_row,
        grid_column_letters(rect.right_col),
        rect.bottom_row
    )
}

pub(super) fn grid_column_letters(mut col: u32) -> String {
    let mut letters = Vec::new();
    while col > 0 {
        let zero_based = (col - 1) % 26;
        letters.push(char::from(b'A' + u8::try_from(zero_based).unwrap_or(0)));
        col = (col - 1) / 26;
    }
    letters.iter().rev().collect()
}

pub(super) fn check_grid_rect_on_sheet(
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

pub(super) fn feature_rendered_region_blocks_spill(feature_kind: &str) -> bool {
    feature_kind.eq_ignore_ascii_case("table-overlay") || feature_kind.eq_ignore_ascii_case("table")
}

pub(super) fn feature_rendered_region_refuses_inside_axis_edit(feature_kind: &str) -> bool {
    feature_kind.eq_ignore_ascii_case("pivot")
        || feature_kind.eq_ignore_ascii_case("pivot-report")
        || feature_kind.eq_ignore_ascii_case("pivot-table")
}

pub(super) fn feature_rendered_region_marks_refresh_on_transform(feature_kind: &str) -> bool {
    feature_rendered_region_refuses_inside_axis_edit(feature_kind)
}
