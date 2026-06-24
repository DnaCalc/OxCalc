//! Exact dirty-closure invalidation reference (GridInvalidation-Ref) for the
//! strict-excel-grid engines: the semantic dependency model (cell, axis-value,
//! axis-visibility, spill, name, table dependencies), the compressed
//! reverse-edge and interval block index, and the dirty-closure computation
//! the optimized engine is differentially checked against. Internal to the
//! machine; shares the machine's types via `use super::*`.

use super::*;

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
pub(super) struct GridAxisVisibilityIndexedDependency {
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
pub(super) struct GridCompressedRangeDependency {
    dependent: ExcelGridCellAddress,
    extent: GridRect,
}

impl GridCompressedRangeDependency {
    fn new(dependent: ExcelGridCellAddress, extent: GridRect) -> Self {
        Self { dependent, extent }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct GridScalarCellDependency {
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
