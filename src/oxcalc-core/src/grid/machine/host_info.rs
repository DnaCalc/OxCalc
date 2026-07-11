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
    /// Builds a host-info provider from the full set of context an
    /// aggregate function's reference (`SUBTOTAL`/`AGGREGATE` over a defined
    /// name, dynamic-name extent, or structured/table reference) may need to
    /// resolve, mirroring `reference_system_provider`'s registration:
    /// unblocked spill extents, static defined names, dynamic defined-name
    /// extents, and table overlays (structured references). Callers
    /// mid-recalc must pass the LIVE in-progress facts (e.g. a
    /// `GridOptimizedValuation`'s `spill_facts`/`defined_names`/
    /// `dynamic_defined_name_extents`/`table_overlays`), not a committed
    /// pre-recalc snapshot, or hidden-row-aware aggregates over
    /// not-yet-committed spills/names will resolve against stale/empty
    /// state.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        workbook_id: impl Into<String>,
        sheet_id: impl Into<String>,
        caller_row: u32,
        caller_col: u32,
        bounds: ExcelGridBounds,
        spill_facts: impl IntoIterator<Item = &'a GridSpillFact>,
        defined_names: impl IntoIterator<Item = (&'a String, &'a GridRect)>,
        dynamic_defined_name_extents: impl IntoIterator<Item = (&'a String, &'a GridRect)>,
        table_overlays: impl IntoIterator<Item = &'a GridTableOverlay>,
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
        for (name, rect) in defined_names {
            reference_provider =
                reference_provider.with_defined_name_key(name.clone(), rect.clone());
        }
        for (name, rect) in dynamic_defined_name_extents {
            reference_provider =
                reference_provider.with_defined_name_key(name.clone(), rect.clone());
        }
        let caller_address = ExcelGridCellAddress::new(
            reference_provider.workbook_id().to_string(),
            reference_provider.sheet_id().to_string(),
            caller_row,
            caller_col,
        );
        for table in table_overlays {
            reference_provider =
                register_table_overlay_references(reference_provider, table, Some(&caller_address));
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
    pub dirty_seeds: BTreeSet<GridDirtySeed>,
    pub feature_regions_removed: usize,
    pub feature_regions_added: usize,
    pub formula_cells_transformed: usize,
    pub formula_reference_transforms: usize,
}

pub(super) fn grid_table_lifecycle_dirty_seeds(
    table_names: impl IntoIterator<Item = String>,
    extents: impl IntoIterator<Item = GridRect>,
) -> BTreeSet<GridDirtySeed> {
    table_names
        .into_iter()
        .map(GridDirtySeed::Table)
        .chain(
            extents
                .into_iter()
                .map(GridSpillBlockerDependency::extent)
                .map(GridDirtySeed::SpillBlocker),
        )
        .collect()
}

pub(super) fn grid_name_lifecycle_dirty_seeds(
    names: impl IntoIterator<Item = String>,
) -> BTreeSet<GridDirtySeed> {
    names.into_iter().map(GridDirtySeed::Name).collect()
}

/// Sheet-scoped name lifecycle key set: the canonical scoped key for the
/// name being created/redefined/deleted, plus the bare global key for the
/// same name text when a same-text global (or other-scope-but-visible)
/// namespace entry already exists. A consumer already bound to that global
/// key (e.g. `SUM(InputRange)` bound before any scoped `InputRange` existed)
/// is otherwise never dirtied when the scoped shadow is created/changed/
/// removed, so it keeps evaluating against the shadowed global value. See
/// D2 in the scoped-names/local-structured-refs batch.
pub(super) fn grid_scoped_name_lifecycle_keys(
    scoped_key: String,
    global_key_candidate: Option<&str>,
    global_entry_exists: bool,
) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    keys.insert(scoped_key);
    if global_entry_exists && let Some(global_key) = global_key_candidate {
        keys.insert(global_key.to_string());
    }
    keys
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridNameLifecycleOperation {
    Create,
    Redefine,
    Rename,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridNameLifecycleReport {
    pub operation: GridNameLifecycleOperation,
    pub old_name_key: Option<String>,
    pub new_name_key: Option<String>,
    pub dirty_seeds: BTreeSet<GridDirtySeed>,
    pub formula_cells_transformed: usize,
    pub formula_reference_transforms: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridDynamicDefinedName {
    pub formula: GridFormulaCell,
    pub anchor: ExcelGridCellAddress,
}

impl GridDynamicDefinedName {
    #[must_use]
    pub fn new(formula: GridFormulaCell, anchor: ExcelGridCellAddress) -> Self {
        Self { formula, anchor }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GridDynamicDefinedNameEvaluationOutcome {
    pub extent: Option<GridRect>,
    pub formula_dependencies: BTreeSet<GridDependency>,
    pub volatile: bool,
    pub external_pending: bool,
    pub external_subscriptions: BTreeSet<GridRuntimeExternalSubscription>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct GridDynamicDefinedNameRefreshReport {
    pub dirty_seeds: BTreeSet<GridDirtySeed>,
    pub evaluations: usize,
    pub external_subscription_updates: Vec<GridExternalAvailabilitySubscriptionUpdate>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GridDynamicDefinedNameDependencyState {
    dependencies_by_name: BTreeMap<String, BTreeSet<GridDependency>>,
}

impl GridDynamicDefinedNameDependencyState {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.dependencies_by_name.is_empty()
    }

    #[must_use]
    pub fn contains_name(&self, name_key: impl AsRef<str>) -> bool {
        self.dependencies_by_name.contains_key(name_key.as_ref())
    }

    #[must_use]
    pub fn dependencies_for(&self, name_key: impl AsRef<str>) -> BTreeSet<GridDependency> {
        self.dependencies_by_name
            .get(name_key.as_ref())
            .cloned()
            .unwrap_or_default()
    }

    pub fn set_dependencies(
        &mut self,
        name_key: impl Into<String>,
        dependencies: BTreeSet<GridDependency>,
    ) {
        let name_key = name_key.into();
        if dependencies.is_empty() {
            self.dependencies_by_name.remove(&name_key);
        } else {
            self.dependencies_by_name.insert(name_key, dependencies);
        }
    }

    pub fn remove(&mut self, name_key: impl AsRef<str>) -> bool {
        self.dependencies_by_name
            .remove(name_key.as_ref())
            .is_some()
    }

    pub fn clear(&mut self) {
        self.dependencies_by_name.clear();
    }

    pub fn retain_names(&mut self, names: &BTreeSet<String>) {
        self.dependencies_by_name
            .retain(|name_key, _| names.contains(name_key));
    }

    pub fn rename(&mut self, old_key: &str, new_key: String) {
        if let Some(dependencies) = self.dependencies_by_name.remove(old_key) {
            self.dependencies_by_name.insert(new_key, dependencies);
        }
    }

    pub fn affected_names_for_seeds(
        &self,
        seeds: &BTreeSet<GridDirtySeed>,
        bounds: ExcelGridBounds,
    ) -> Result<BTreeSet<String>, GridRefError> {
        let mut affected = BTreeSet::new();
        for (name_key, dependencies) in &self.dependencies_by_name {
            if dependencies.iter().any(|dependency| {
                seeds.iter().any(|seed| {
                    dynamic_defined_name_dependency_matches_seed(dependency, seed, bounds)
                })
            }) {
                affected.insert(name_key.clone());
            }
        }
        Ok(affected)
    }

    #[must_use]
    pub fn dependent_names_for_name(
        &self,
        name_key: impl AsRef<str>,
        dynamic_name_keys: &BTreeSet<String>,
    ) -> BTreeSet<String> {
        let name_key = name_key.as_ref();
        self.dependencies_by_name
            .iter()
            .filter(|(dependent_name, _)| dynamic_name_keys.contains(*dependent_name))
            .filter(|(_, dependencies)| {
                dependencies.iter().any(|dependency| {
                    dynamic_defined_name_dependency_mentions_name(dependency, name_key)
                })
            })
            .map(|(dependent_name, _)| dependent_name.clone())
            .collect()
    }

    #[must_use]
    pub fn dynamic_name_cycle(&self, dynamic_name_keys: &BTreeSet<String>) -> Option<Vec<String>> {
        let mut visited = BTreeSet::new();
        let mut stack = Vec::new();
        let mut in_stack = BTreeSet::new();
        for name_key in dynamic_name_keys {
            if let Some(cycle) = self.dynamic_name_cycle_from(
                name_key,
                dynamic_name_keys,
                &mut visited,
                &mut stack,
                &mut in_stack,
            ) {
                return Some(cycle);
            }
        }
        None
    }

    fn dynamic_name_cycle_from(
        &self,
        name_key: &str,
        dynamic_name_keys: &BTreeSet<String>,
        visited: &mut BTreeSet<String>,
        stack: &mut Vec<String>,
        in_stack: &mut BTreeSet<String>,
    ) -> Option<Vec<String>> {
        if in_stack.contains(name_key) {
            let start = stack
                .iter()
                .position(|entry| entry == name_key)
                .unwrap_or(0);
            let mut cycle = stack[start..].to_vec();
            cycle.push(name_key.to_string());
            return Some(cycle);
        }
        if !visited.insert(name_key.to_string()) {
            return None;
        }
        stack.push(name_key.to_string());
        in_stack.insert(name_key.to_string());
        for dependency in self.dynamic_name_dependencies_for_name(name_key, dynamic_name_keys) {
            if let Some(cycle) = self.dynamic_name_cycle_from(
                &dependency,
                dynamic_name_keys,
                visited,
                stack,
                in_stack,
            ) {
                return Some(cycle);
            }
        }
        stack.pop();
        in_stack.remove(name_key);
        None
    }

    fn dynamic_name_dependencies_for_name(
        &self,
        name_key: &str,
        dynamic_name_keys: &BTreeSet<String>,
    ) -> BTreeSet<String> {
        self.dependencies_by_name
            .get(name_key)
            .into_iter()
            .flat_map(|dependencies| dependencies.iter())
            .filter_map(dynamic_defined_name_dependency_name_key)
            .filter(|dependency_name| dynamic_name_keys.contains(dependency_name))
            .collect()
    }
}

pub(super) fn dynamic_defined_name_keys_to_refresh(
    all_dynamic_name_keys: &BTreeSet<String>,
    dependency_state: &GridDynamicDefinedNameDependencyState,
    volatile_names: &BTreeSet<String>,
    external_pending_names: &BTreeSet<String>,
    seeds: &BTreeSet<GridDirtySeed>,
    bounds: ExcelGridBounds,
    force_volatile: bool,
    force_external: bool,
) -> Result<BTreeSet<String>, GridRefError> {
    let mut keys = dependency_state.affected_names_for_seeds(seeds, bounds)?;
    for seed in seeds {
        if let GridDirtySeed::Name(name) = seed
            && let Some(candidate_keys) = excel_grid_defined_name_seed_keys(name, bounds)
        {
            for name_key in candidate_keys {
                if all_dynamic_name_keys.contains(&name_key) {
                    keys.insert(name_key);
                }
            }
        }
    }
    if dependency_state.is_empty() {
        keys.extend(all_dynamic_name_keys.iter().cloned());
    } else {
        keys.extend(
            all_dynamic_name_keys
                .iter()
                .filter(|name_key| !dependency_state.contains_name(*name_key))
                .cloned(),
        );
    }
    if force_volatile {
        keys.extend(
            volatile_names
                .iter()
                .filter(|name_key| all_dynamic_name_keys.contains(*name_key))
                .cloned(),
        );
    }
    if force_external {
        keys.extend(
            external_pending_names
                .iter()
                .filter(|name_key| all_dynamic_name_keys.contains(*name_key))
                .cloned(),
        );
    }
    Ok(keys)
}

pub(super) fn default_dynamic_defined_name_anchor(
    workbook_id: impl Into<String>,
    sheet_id: impl Into<String>,
) -> ExcelGridCellAddress {
    ExcelGridCellAddress::new(workbook_id, sheet_id, 1, 1)
}

#[must_use]
pub(super) fn dynamic_defined_name_extent_from_dependencies(
    dependencies: &BTreeSet<GridDependency>,
    bounds: ExcelGridBounds,
) -> Option<GridRect> {
    dependencies
        .iter()
        .find_map(|dependency| grid_dependency_extent(dependency, bounds))
}

/// Picks the calc-time realized extent for a dynamic defined name from the
/// formula's runtime trace, instead of taking the first extent-bearing
/// dependency in `BTreeSet` order (which put a branch selector such as `C1`
/// in `IF(C1>0,A1:A10,B1:B10)` ahead of the actual branch target, since
/// `Cell(C1)` sorts before `Range(A1:A10)`).
///
/// `resolution_effects` preserve evaluation order, so the LITERAL LAST effect
/// (not the last effect of a matching kind) is examined: if it is a
/// `Dereferenced`/`Enumerated`/`Transformed` effect with non-empty
/// dependencies, that is the reference the formula actually resolved to and
/// returned — the realized target. A single such effect's dependencies are
/// unioned via `grid_extent_for_dependencies` (which itself uses
/// `grid_rect_union`), so a genuinely multi-area target (e.g. a name whose
/// formula composes a multi-area reference) yields the union rect, or `None`
/// (unresolved) when the union cannot be formed (e.g. cross-sheet parts).
///
/// It is important that this only inspects the LITERAL last effect, not the
/// last effect of a matching kind: a formula whose final resolution step
/// FAILED (e.g. `INDIRECT(C1)` where `C1`'s text does not resolve) still
/// records an earlier `Dereferenced` effect for reading the selector cell's
/// own value (`C1`), followed by a last `TextResolved` effect. That terminal
/// `TextResolved` effect is ALWAYS present for a `resolve_text` call — even
/// on failure with no identity dependencies to carry (see
/// `GridTracingReferenceSystemProvider::record_terminal` in
/// `runtime_trace.rs`) — precisely so its discriminant marks "the terminal
/// step was a text resolution" and this function can tell a failed/unresolved
/// text resolution apart from an earlier selector read. A terminal
/// `TextResolved` is therefore its OWN case, distinct from
/// `Dereferenced`/`Enumerated`/`Transformed`: a successful one with extent-
/// bearing dependencies realizes those dependencies; a failed/empty one
/// realizes to `None` (unresolved) directly, and must NOT fall through to
/// `fallback_dependencies`, since that bag is the formula's structural/
/// realized dependencies at large and can still contain the selector cell's
/// own `Cell` dependency — which would wrongly realize the selector itself as
/// the target instead of reporting unresolved.
///
/// Falls back to the first extent-bearing dependency in the formula's
/// dependency bag when the trace has no resolution effect at all (a name
/// formula with no runtime reference resolution, e.g. a name whose formula
/// returns a directly-bound literal range with no dynamic step), or when the
/// literal last effect is a non-text `Dereferenced`/`Enumerated`/
/// `Transformed` effect that is not itself extent-bearing.
#[must_use]
pub(super) fn dynamic_defined_name_extent_from_trace(
    trace: &GridRuntimeDependencyTrace,
    fallback_dependencies: &BTreeSet<GridDependency>,
    bounds: ExcelGridBounds,
) -> Option<GridRect> {
    match trace.resolution_effects.last() {
        Some(GridRuntimeResolutionEffect::TextResolved { dependencies, .. }) => {
            if dependencies.is_empty() {
                // Terminal text resolution failed (or resolved to identity-
                // only dependencies with no extent, e.g. an unresolved name
                // reference): the name is unresolved. Do not fall through to
                // `fallback_dependencies`, which can still carry the
                // selector cell's own dependency.
                return None;
            }
            grid_extent_for_dependencies(dependencies, bounds)
        }
        Some(
            GridRuntimeResolutionEffect::Dereferenced { dependencies }
            | GridRuntimeResolutionEffect::Enumerated { dependencies }
            | GridRuntimeResolutionEffect::Transformed { dependencies },
        ) if !dependencies.is_empty() => grid_extent_for_dependencies(dependencies, bounds),
        _ => dynamic_defined_name_extent_from_dependencies(fallback_dependencies, bounds),
    }
}

fn dynamic_defined_name_dependency_mentions_name(
    dependency: &GridDependency,
    name_key: &str,
) -> bool {
    dynamic_defined_name_dependency_name_key(dependency).is_some_and(|key| key == name_key)
}

fn dynamic_defined_name_dependency_name_key(dependency: &GridDependency) -> Option<String> {
    match dependency {
        GridDependency::Name(dependency) => Some(dependency.name_key.clone()),
        GridDependency::NameIdentity(dependency) => Some(dependency.name_key.clone()),
        _ => None,
    }
}

fn dynamic_defined_name_dependency_matches_seed(
    dependency: &GridDependency,
    seed: &GridDirtySeed,
    bounds: ExcelGridBounds,
) -> bool {
    match seed {
        GridDirtySeed::Cell(address) => grid_dependency_contains_cell(dependency, address),
        GridDirtySeed::Range(rect) => grid_dependency_overlaps_rect(dependency, rect),
        GridDirtySeed::Name(name) => {
            excel_grid_defined_name_seed_keys(name, bounds).is_some_and(|keys| {
                keys.iter().any(|key| {
                    matches!(
                        dependency,
                        GridDependency::Name(dependency) if dependency.name_key == *key
                    ) || matches!(
                        dependency,
                        GridDependency::NameIdentity(dependency) if dependency.name_key == *key
                    )
                })
            })
        }
        GridDirtySeed::Table(table_name) => excel_grid_table_name_key(table_name, bounds)
            .is_some_and(|key| {
                matches!(
                    dependency,
                    GridDependency::Table(dependency) if dependency.table_key == key
                ) || matches!(
                    dependency,
                    GridDependency::TableIdentity(dependency) if dependency.table_key == key
                )
            }),
        GridDirtySeed::SpillFact(seed_dependency) => {
            matches!(
                dependency,
                GridDependency::SpillFact(dependency) if dependency.anchor == seed_dependency.anchor
            )
        }
        GridDirtySeed::SpillBlocker(seed_dependency) => {
            matches!(
                dependency,
                GridDependency::SpillBlocker(dependency)
                    if grid_rects_overlap(&dependency.extent, &seed_dependency.extent)
            )
        }
        GridDirtySeed::AxisVisibility(seed_dependency) => {
            matches!(
                dependency,
                GridDependency::AxisVisibility(dependency)
                    if dependency.axis == seed_dependency.axis
                        && ranges_overlap(
                            dependency.first,
                            dependency.last,
                            seed_dependency.first,
                            seed_dependency.last,
                        )
            )
        }
        GridDirtySeed::AxisValue(seed_dependency) => {
            matches!(
                dependency,
                GridDependency::AxisValue(dependency)
                    if dependency.axis == seed_dependency.axis
                        && ranges_overlap(
                            dependency.first,
                            dependency.last,
                            seed_dependency.first,
                            seed_dependency.last,
                        )
            )
        }
        GridDirtySeed::DynamicRequest(request_key) => {
            matches!(
                dependency,
                GridDependency::DynamicRequest(dependency_key) if dependency_key == request_key
            )
        }
        GridDirtySeed::Volatile | GridDirtySeed::External => false,
    }
}

fn grid_dependency_contains_cell(
    dependency: &GridDependency,
    address: &ExcelGridCellAddress,
) -> bool {
    match dependency {
        GridDependency::Cell(cell) => cell == address,
        GridDependency::Range(rect) => rect.contains(address),
        GridDependency::Name(dependency) => dependency.extent.contains(address),
        GridDependency::Table(dependency) => dependency.extent.contains(address),
        GridDependency::SpillFact(dependency) => dependency.anchor == *address,
        GridDependency::SpillBlocker(dependency) => dependency.extent.contains(address),
        // G6: without this arm, a dynamic name whose formula consumes a
        // whole-axis reference (`AxisValue` dependency) never matches a
        // plain `GridDirtySeed::Cell`/`Range` edit through the ledger
        // (`affected_names_for_seeds`), even though
        // `GridInvalidationRef::direct_dependents_for_cell` (the structural
        // graph's own matcher for the same dependency kind) does match it.
        GridDependency::AxisValue(dependency) => {
            axis_value_dependency_contains_address(dependency, address)
        }
        _ => false,
    }
}

fn grid_dependency_overlaps_rect(dependency: &GridDependency, rect: &GridRect) -> bool {
    match dependency {
        GridDependency::Cell(address) => rect.contains(address),
        GridDependency::Range(dependency_rect) => grid_rects_overlap(dependency_rect, rect),
        GridDependency::Name(dependency) => grid_rects_overlap(&dependency.extent, rect),
        GridDependency::Table(dependency) => grid_rects_overlap(&dependency.extent, rect),
        GridDependency::SpillFact(dependency) => rect.contains(&dependency.anchor),
        GridDependency::SpillBlocker(dependency) => grid_rects_overlap(&dependency.extent, rect),
        // G6: see the matching comment on `grid_dependency_contains_cell`.
        // Overlap (not containment) matches the `Range`/`Name`/`Table` arms
        // above: a `Range` dirty seed only needs to intersect the
        // whole-axis span, not be fully inside it.
        GridDependency::AxisValue(dependency) => match dependency.axis {
            GridAxis::Row => ranges_overlap(
                dependency.first,
                dependency.last,
                rect.top_row,
                rect.bottom_row,
            ),
            GridAxis::Column => ranges_overlap(
                dependency.first,
                dependency.last,
                rect.left_col,
                rect.right_col,
            ),
        },
        _ => false,
    }
}

fn ranges_overlap(lhs_first: u32, lhs_last: u32, rhs_first: u32, rhs_last: u32) -> bool {
    lhs_first <= rhs_last && rhs_first <= lhs_last
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

pub(super) fn defined_name_key_for_name_or_key(
    name_or_key: &str,
    bounds: ExcelGridBounds,
) -> Result<String, GridRefError> {
    excel_grid_defined_name_seed_keys(name_or_key, bounds)
        .and_then(|keys| keys.into_iter().next())
        .ok_or_else(|| GridRefError::InvalidDefinedName {
            name: name_or_key.to_string(),
        })
}

pub(super) fn sheet_defined_name_key_for_name(
    workbook_id: &str,
    sheet_id: &str,
    name: &str,
    bounds: ExcelGridBounds,
) -> Result<String, GridRefError> {
    excel_grid_sheet_defined_name_key(workbook_id, sheet_id, name, bounds).ok_or_else(|| {
        GridRefError::InvalidDefinedName {
            name: name.to_string(),
        }
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
