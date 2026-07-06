//! The optimized engine's eval-time reference provider: implements
//! oxfunc_core's ReferenceSystemProvider over a GridOptimizedValuation,
//! resolving reference shape through the shared strict-grid resolver and
//! serving cell values from the compact sparse/dense storage with
//! occupancy-proportional enumeration and the materialization guard.
//! Internal to the machine; shares the machine's types via `use super::*`.

use super::*;

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

    pub(super) fn add_rect_report(&mut self, other: &Self) {
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

    #[must_use]
    pub const fn bounds(&self) -> ExcelGridBounds {
        self.valuation.bounds
    }

    #[must_use]
    pub fn workbook_id(&self) -> &str {
        &self.valuation.workbook_id
    }

    #[must_use]
    pub fn sheet_id(&self) -> &str {
        &self.valuation.sheet_id
    }

    pub(super) fn resolved_values_for_rect(
        &self,
        rect: &GridRect,
    ) -> Result<ResolvedReferenceValues, ReferenceResolutionError> {
        self.resolved_values_for_rect_with_report(rect)
            .map(|measured| measured.values)
    }

    pub(super) fn resolved_values_for_rect_with_report(
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

        // W062 R4.6 cross-sheet value resolution: a rect naming a *different*
        // sheet than this valuation reads from the coordinator-injected
        // cross-sheet input view, never from local dense/sparse storage (which
        // holds only this sheet's cells). This is the optimized-lane analogue of
        // `ExcelGridReferenceSystemProvider::cell_value`'s cross-sheet fallback:
        // own-sheet reads take the local path below; foreign reads terminate
        // here. A foreign cell absent from the view (unknown/dormant sheet, or a
        // not-yet-published peer) reads empty, matching the oracle.
        let foreign_sheet = rect.workbook_id != self.valuation.workbook_id
            || rect.sheet_id != self.valuation.sheet_id;
        if foreign_sheet {
            // Iterate the SPARSE cross-sheet view filtered to the rect (like the
            // own-sheet dense/sparse paths below serve only occupied cells), not the
            // full rect extent — a whole-column cross-sheet range must not do a dense
            // million-cell scan. Positions are ONE-based relative to the rect's
            // top-left (`row - top_row + 1`); a zero-based index here would fail
            // OxFunc's 1-based `materialize` bounds check (W062 R6.67 / calc-5kqg.67 —
            // a latent bug the cross-sheet single-cell scalar path never exercised).
            for (address, value) in self.valuation.cross_sheet_cells.iter() {
                if !rect.contains(address) {
                    continue;
                }
                let row = usize::try_from(address.row - rect.top_row + 1).unwrap_or(usize::MAX);
                let col = usize::try_from(address.col - rect.left_col + 1).unwrap_or(usize::MAX);
                cells.insert((row, col), (0, value.clone()));
            }
            report.defined_cell_count = cells.len();
            let values = ResolvedReferenceValues::new(
                ResolvedReferenceExtent::new(rows, cols),
                cells
                    .into_iter()
                    .map(|((row, col), (_revision, value))| {
                        ResolvedReferenceCell::new(row, col, value)
                    })
                    .collect(),
                Some(format!(
                    "optimized-grid:v1:cross:{}:{}:R{}C{}:R{}C{}",
                    rect.workbook_id,
                    rect.sheet_id,
                    rect.top_row,
                    rect.left_col,
                    rect.bottom_row,
                    rect.right_col
                )),
            );
            return Ok(GridOptimizedMeasuredReferenceValues { values, report });
        }

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

    pub(super) fn dense_cover_for_rect(
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

    pub(super) fn materialize_large_dense_rect(
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

    pub(super) fn resolved_values_for_rects(
        &self,
        rects: &[GridRect],
    ) -> Result<ResolvedReferenceValues, ReferenceResolutionError> {
        self.resolved_values_for_rects_with_report(rects)
            .map(|measured| measured.values)
    }

    /// Flatten a 3D sheet-span's per-member-sheet rects into one aggregation
    /// array (W062 R4.12), reading each member cell through the valuation's own/
    /// cross-sheet path. Unlike [`Self::resolved_values_for_rects_with_report`]
    /// (multi-area, which assumes a shared coordinate origin and flattens with
    /// `row-1` math), the member rects live on *different sheets*, so this walks
    /// each member rect's cells with a running 1-based column and keeps only
    /// populated cells — matching the oracle provider's span flatten exactly.
    pub(super) fn resolved_values_for_span_rects_with_report(
        &self,
        member_rects: &[GridRect],
    ) -> Result<GridOptimizedMeasuredReferenceValues, ReferenceResolutionError> {
        let mut cells = Vec::new();
        let mut column = 1usize;
        let mut declared_total = 0usize;
        let mut report = GridOptimizedReferenceEnumerationReport::default();
        let mut identities = Vec::with_capacity(member_rects.len());
        for rect in member_rects {
            let measured = self.resolved_values_for_rect_with_report(rect)?;
            report.add_rect_report(&measured.report);
            let area_cells = measured.values.declared_extent.declared_cell_count();
            // Re-index the member area's defined cells into the running column,
            // row-major, preserving only populated cells (sparse aggregation).
            let area_cols = measured.values.declared_extent.cols.max(1);
            for cell in measured.values.defined_cells {
                let within = (cell.row.saturating_sub(1))
                    .saturating_mul(area_cols)
                    .saturating_add(cell.col.saturating_sub(1));
                cells.push(ResolvedReferenceCell::new(1, column + within, cell.value));
            }
            column = column.checked_add(area_cells).ok_or_else(|| {
                ReferenceResolutionError::ProviderFailure {
                    detail: "optimized_grid_sheetspan_extent_overflow".to_string(),
                }
            })?;
            declared_total = declared_total.checked_add(area_cells).ok_or_else(|| {
                ReferenceResolutionError::ProviderFailure {
                    detail: "optimized_grid_sheetspan_extent_overflow".to_string(),
                }
            })?;
            if let Some(identity) = measured.values.reader_identity {
                identities.push(identity);
            }
        }
        cells.sort_by_key(|cell| (cell.row, cell.col));
        report.defined_cell_count = cells.len();
        let values = ResolvedReferenceValues::new(
            ResolvedReferenceExtent::new(1, declared_total),
            cells,
            Some(format!("optimized-grid:v1:sheetspan:{}", identities.join("|"))),
        );
        Ok(GridOptimizedMeasuredReferenceValues { values, report })
    }

    pub(super) fn resolved_values_for_rects_with_report(
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
        // W062 R4.12: enumerate a 3D span's aggregated member values (SUM/COUNT/…
        // enumerate rather than dereference). Same member-rect flatten as
        // `dereference`, through the valuation's own/cross read path.
        if let Some(member_rects) = self.shape_provider.span_member_rects(&request.reference) {
            return self
                .resolved_values_for_span_rects_with_report(&member_rects)
                .map(Some);
        }
        let rects = self
            .shape_provider
            .resolved_rects_for_reference(&request.reference)?;
        self.resolved_values_for_rects_with_report(&rects).map(Some)
    }

    pub(super) fn resolved_rects_for_reference(
        &self,
        reference: &ReferenceLike,
    ) -> Result<Vec<GridRect>, ReferenceResolutionError> {
        self.shape_provider.resolved_rects_for_reference(reference)
    }

    pub(super) fn defined_name_dependency_key_for_scope(
        &self,
        workbook_id: &str,
        sheet_id: &str,
        name: &str,
    ) -> Option<String> {
        self.shape_provider
            .defined_name_dependency_key_for_scope(workbook_id, sheet_id, name)
    }

    pub(super) fn defined_name_candidate_dependency_keys_for_scope(
        &self,
        workbook_id: &str,
        sheet_id: &str,
        name: &str,
    ) -> Vec<String> {
        self.shape_provider
            .defined_name_candidate_dependency_keys_for_scope(workbook_id, sheet_id, name)
    }

    pub(super) fn defined_name_dependency_resolution_for_scope(
        &self,
        workbook_id: &str,
        sheet_id: &str,
        name: &str,
    ) -> GridNameDependencyScopeResolution {
        self.shape_provider
            .defined_name_dependency_resolution_for_scope(workbook_id, sheet_id, name)
    }
}

impl ReferenceSystemProvider for GridOptimizedReferenceSystemProvider<'_> {
    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        // W062 R4.12: a 3D sheet-span reference resolves to the aggregated
        // values of its member sheets' target cells. Flatten the seeded member
        // rects through the valuation's own/cross-sheet read path (each member
        // rect names its own sheet, foreign or own), matching the oracle.
        if let Some(member_rects) = self.shape_provider.span_member_rects(&request.reference) {
            let values = self
                .resolved_values_for_span_rects_with_report(&member_rects)?
                .values;
            if values.declared_extent.declared_cell_count()
                > GRID_OPTIMIZED_PROVIDER_MATERIALIZATION_LIMIT
            {
                return Err(ReferenceResolutionError::ProviderFailure {
                    detail: "optimized_grid_sheetspan_requires_sparse_enumeration".to_string(),
                });
            }
            return materialize_resolved_reference_values(&values).map(CalcValue::array);
        }
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
            let address = ExcelGridCellAddress::new(
                rect.workbook_id,
                rect.sheet_id,
                rect.top_row,
                rect.left_col,
            );
            // W062 R4.6: a scalar reference to a *foreign* sheet reads the
            // coordinator-injected cross-sheet view, never local storage (which
            // holds only this sheet's cells). Own-sheet scalars take the fast
            // `read_cell` path below. A foreign cell absent from the view reads
            // empty, matching the oracle's cross-sheet miss semantics.
            if address.workbook_id != self.valuation.workbook_id
                || address.sheet_id != self.valuation.sheet_id
            {
                return Ok(self
                    .valuation
                    .cross_sheet_cells
                    .get(&address)
                    .cloned()
                    .unwrap_or_else(CalcValue::empty));
            }
            return Ok(self.valuation.read_cell(&address).computed);
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

pub(super) fn insert_resolved_cell(
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

pub(super) fn dense_region_covers_resolved_rect(
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

pub(super) fn intersect_rects(lhs: &GridRect, rhs: &GridRect) -> Option<(u32, u32, u32, u32)> {
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

pub(super) fn grid_rect_intersection(
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

pub(super) fn pairwise_rect_partition_report<'a>(
    rects: impl Iterator<Item = &'a GridRect>,
) -> (u64, usize) {
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

pub(super) fn grid_rects_overlap(lhs: &GridRect, rhs: &GridRect) -> bool {
    lhs.workbook_id == rhs.workbook_id
        && lhs.sheet_id == rhs.sheet_id
        && lhs.top_row <= rhs.bottom_row
        && rhs.top_row <= lhs.bottom_row
        && lhs.left_col <= rhs.right_col
        && rhs.left_col <= lhs.right_col
}

pub(super) fn rects_overlap_outside_anchor(
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
    pub structural_dependency_edges: usize,
    pub overlay_dependency_edges: usize,
    pub dynamic_defined_name_evaluations: usize,
    pub external_subscription_updates: Vec<GridExternalAvailabilitySubscriptionUpdate>,
}

impl GridOptimizedRecalcReport {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
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
            structural_dependency_edges: 0,
            overlay_dependency_edges: 0,
            dynamic_defined_name_evaluations: 0,
            external_subscription_updates: Vec::new(),
        }
    }

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
    pub(super) fn r1c1_double_left() -> Self {
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

    pub(super) fn evaluate_repeated_region_cell(
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

    pub(super) fn evaluate_single_cell(
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
