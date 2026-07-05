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

    #[must_use]
    pub fn from_key(name_key: String, extent: GridRect) -> Self {
        Self { name_key, extent }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridNameIdentityDependency {
    pub name_key: String,
}

impl GridNameIdentityDependency {
    pub fn new(name: impl AsRef<str>, bounds: ExcelGridBounds) -> Result<Self, GridRefError> {
        let Some(name_key) = excel_grid_defined_name_key(name.as_ref(), bounds) else {
            return Err(GridRefError::InvalidDefinedName {
                name: name.as_ref().to_string(),
            });
        };
        Ok(Self { name_key })
    }

    #[must_use]
    pub fn from_key(name_key: String) -> Self {
        Self { name_key }
    }
}

/// The single stored dependency edge for a 3D sheet-span reference
/// (`Sheet1:Sheet3!A1`, W062 D2 §4.2 / R3.9).
///
/// **One stored edge, expanded at closure time — never a materialized per-sheet
/// fan** (§4.2 decision). The endpoints are the rename-immune
/// [`crate::reference_vocabulary::SheetIdentityToken`] strings (§10); the
/// `target` is the sheet-agnostic authored target text (the §4.2 rect
/// ignore-rule — no [`GridRect`] embedding a single sheet identity). Span
/// membership is a function of the *current* sheet order, so the workbook
/// coordination layer (D3) expands this edge against the C3 sheet-registry order
/// whenever it computes dirty closure or evaluation order; a stored fan would be
/// wrong between a sheet lifecycle edit and its rewrite, this re-expands
/// correctly for free.
///
/// R3.9 lands this variant + the emission seam only. **Closure expansion, the
/// span-interval index, endpoint delete/shrink transforms, and value seeding
/// are R4.12** — [`grid_dirty_seed_for_dependency`] returns `None` for this
/// variant today (no closure consumption), exactly as [`GridDependency::ReferenceMetadata`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridSheetSpanDependency {
    pub workbook_id: String,
    pub start_sheet: String,
    pub end_sheet: String,
    pub target: String,
}

impl GridSheetSpanDependency {
    #[must_use]
    pub fn new(
        workbook_id: impl Into<String>,
        start_sheet: impl Into<String>,
        end_sheet: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self {
            workbook_id: workbook_id.into(),
            start_sheet: start_sheet.into(),
            end_sheet: end_sheet.into(),
            target: target.into(),
        }
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridTableIdentityDependency {
    pub table_key: String,
}

impl GridTableIdentityDependency {
    pub fn new(table_name: impl AsRef<str>, bounds: ExcelGridBounds) -> Result<Self, GridRefError> {
        let Some(table_key) = excel_grid_table_name_key(table_name.as_ref(), bounds) else {
            return Err(GridRefError::InvalidTableName {
                name: table_name.as_ref().to_string(),
            });
        };
        Ok(Self { table_key })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GridDependency {
    Cell(ExcelGridCellAddress),
    Range(GridRect),
    Name(GridNameDependency),
    NameIdentity(GridNameIdentityDependency),
    Table(GridTableDependency),
    TableIdentity(GridTableIdentityDependency),
    /// The stored edge for a 3D sheet-span reference (`Sheet1:Sheet3!A1`, W062
    /// D2 §4.2 / R3.9). One edge; the per-sheet fan is a closure-time expansion
    /// against the current sheet order (**R4.12** — not consumed yet). See
    /// [`GridSheetSpanDependency`].
    SheetSpan(GridSheetSpanDependency),
    SpillFact(GridSpillDependency),
    SpillBlocker(GridSpillBlockerDependency),
    AxisVisibility(GridAxisVisibilityDependency),
    AxisValue(GridAxisValueDependency),
    ReferenceMetadata(Box<GridDependency>),
    DynamicRequest(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GridDirtySeed {
    Cell(ExcelGridCellAddress),
    Range(GridRect),
    SpillFact(GridSpillDependency),
    SpillBlocker(GridSpillBlockerDependency),
    AxisVisibility(GridAxisVisibilityDependency),
    AxisValue(GridAxisValueDependency),
    Name(String),
    Table(String),
    DynamicRequest(String),
    Volatile,
    External,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GridDirtyClosure {
    pub seeds: BTreeSet<GridDirtySeed>,
    pub dirty_cells: BTreeSet<ExcelGridCellAddress>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GridOverlayDependencyUpdate {
    pub dependent: Option<ExcelGridCellAddress>,
    pub old_dependencies: BTreeSet<GridDependency>,
    pub new_dependencies: BTreeSet<GridDependency>,
    pub added_dependencies: BTreeSet<GridDependency>,
    pub removed_dependencies: BTreeSet<GridDependency>,
    pub dirty_seeds: BTreeSet<GridDirtySeed>,
}

impl GridOverlayDependencyUpdate {
    #[must_use]
    pub fn unchanged(
        dependent: ExcelGridCellAddress,
        dependencies: BTreeSet<GridDependency>,
    ) -> Self {
        Self {
            dependent: Some(dependent),
            old_dependencies: dependencies.clone(),
            new_dependencies: dependencies,
            added_dependencies: BTreeSet::new(),
            removed_dependencies: BTreeSet::new(),
            dirty_seeds: BTreeSet::new(),
        }
    }

    #[must_use]
    pub fn changed(
        dependent: ExcelGridCellAddress,
        old_dependencies: BTreeSet<GridDependency>,
        new_dependencies: BTreeSet<GridDependency>,
    ) -> Self {
        let added_dependencies = new_dependencies
            .difference(&old_dependencies)
            .cloned()
            .collect::<BTreeSet<_>>();
        let removed_dependencies = old_dependencies
            .difference(&new_dependencies)
            .cloned()
            .collect::<BTreeSet<_>>();
        let dirty_seeds = added_dependencies
            .iter()
            .chain(removed_dependencies.iter())
            .filter_map(grid_dirty_seed_for_dependency)
            .collect();

        Self {
            dependent: Some(dependent),
            old_dependencies,
            new_dependencies,
            added_dependencies,
            removed_dependencies,
            dirty_seeds,
        }
    }

    /// Like [`Self::changed`], but for an overlay *identity* retarget (a
    /// formula releasing one runtime-resolved target and acquiring another,
    /// e.g. an `INDIRECT`/`OFFSET` selector flip): `added_dependencies` and
    /// `removed_dependencies` are still reported for diagnostics, but no
    /// value `dirty_seeds` are promoted from them. Seeding the released or
    /// acquired target's own value would re-dirty every other consumer of
    /// that target's downstream cone even though that target's *value*
    /// never changed — only this formula's relationship to it did. This
    /// formula already reads the acquired target's current value directly
    /// in the same evaluation pass, and going forward a real value change on
    /// either target reaches this formula again through ordinary
    /// publication-delta feedback over the (now current) overlay edge.
    #[must_use]
    pub fn identity_changed(
        dependent: ExcelGridCellAddress,
        old_dependencies: BTreeSet<GridDependency>,
        new_dependencies: BTreeSet<GridDependency>,
    ) -> Self {
        let added_dependencies = new_dependencies
            .difference(&old_dependencies)
            .cloned()
            .collect::<BTreeSet<_>>();
        let removed_dependencies = old_dependencies
            .difference(&new_dependencies)
            .cloned()
            .collect::<BTreeSet<_>>();

        Self {
            dependent: Some(dependent),
            old_dependencies,
            new_dependencies,
            added_dependencies,
            removed_dependencies,
            dirty_seeds: BTreeSet::new(),
        }
    }

    #[must_use]
    pub fn changed_dependency_count(&self) -> usize {
        self.added_dependencies.len() + self.removed_dependencies.len()
    }

    #[must_use]
    pub fn is_changed(&self) -> bool {
        self.changed_dependency_count() > 0
    }
}

fn grid_dirty_seed_for_dependency(dependency: &GridDependency) -> Option<GridDirtySeed> {
    match dependency {
        GridDependency::Cell(address) => Some(GridDirtySeed::Cell(address.clone())),
        GridDependency::Range(rect) => Some(GridDirtySeed::Range(rect.clone())),
        GridDependency::Name(dependency) => Some(GridDirtySeed::Name(dependency.name_key.clone())),
        GridDependency::NameIdentity(dependency) => {
            Some(GridDirtySeed::Name(dependency.name_key.clone()))
        }
        GridDependency::Table(dependency) => {
            Some(GridDirtySeed::Table(dependency.table_key.clone()))
        }
        GridDependency::TableIdentity(dependency) => {
            Some(GridDirtySeed::Table(dependency.table_key.clone()))
        }
        // A 3D sheet-span edge seeds no dirty target on its own: the per-sheet
        // fan and its membership-change dirtying are a closure-time expansion
        // against sheet order (W062 D2 §4.2, R4.12). Until R4.12 wires that
        // expansion, the span contributes no seed here — exactly like
        // `ReferenceMetadata` — rather than a silently-wrong one.
        GridDependency::SheetSpan(_) => None,
        GridDependency::SpillFact(dependency) => Some(GridDirtySeed::SpillFact(dependency.clone())),
        GridDependency::SpillBlocker(dependency) => {
            Some(GridDirtySeed::SpillBlocker(dependency.clone()))
        }
        GridDependency::AxisVisibility(dependency) => {
            Some(GridDirtySeed::AxisVisibility(dependency.clone()))
        }
        GridDependency::AxisValue(dependency) => Some(GridDirtySeed::AxisValue(dependency.clone())),
        GridDependency::ReferenceMetadata(_) => None,
        GridDependency::DynamicRequest(request_key) => {
            Some(GridDirtySeed::DynamicRequest(request_key.clone()))
        }
    }
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
struct GridDependencyIndex {
    bounds: ExcelGridBounds,
    scalarization_limit: u64,
    /// The sheet this per-sheet index belongs to, if stamped. When `Some`,
    /// the sheet-identity routing invariant (W062 D3 §1/§2) is enforced at
    /// registration: any dependent or cell/range dependency address that
    /// resolves to a different sheet is rejected with
    /// [`GridRefError::ForeignSheetDependency`]. `None` preserves the
    /// historical unchecked behavior for indexes constructed without an
    /// owning-sheet stamp.
    owning_sheet: Option<OwningSheetIdentity>,
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
    name_identity_dependencies_by_cell:
        BTreeMap<ExcelGridCellAddress, BTreeSet<GridNameIdentityDependency>>,
    name_dependents_by_key: BTreeMap<String, BTreeSet<ExcelGridCellAddress>>,
    table_dependencies_by_cell: BTreeMap<ExcelGridCellAddress, BTreeSet<GridTableDependency>>,
    table_identity_dependencies_by_cell:
        BTreeMap<ExcelGridCellAddress, BTreeSet<GridTableIdentityDependency>>,
    table_dependents_by_key: BTreeMap<String, BTreeSet<ExcelGridCellAddress>>,
    dynamic_dependencies_by_cell: BTreeMap<ExcelGridCellAddress, BTreeSet<String>>,
    dynamic_dependents_by_request: BTreeMap<String, BTreeSet<ExcelGridCellAddress>>,
    metadata_spill_dependencies_by_cell:
        BTreeMap<ExcelGridCellAddress, BTreeSet<GridSpillDependency>>,
    metadata_name_dependencies_by_cell:
        BTreeMap<ExcelGridCellAddress, BTreeSet<GridNameDependency>>,
    metadata_name_identity_dependencies_by_cell:
        BTreeMap<ExcelGridCellAddress, BTreeSet<GridNameIdentityDependency>>,
    metadata_table_dependencies_by_cell:
        BTreeMap<ExcelGridCellAddress, BTreeSet<GridTableDependency>>,
    metadata_table_identity_dependencies_by_cell:
        BTreeMap<ExcelGridCellAddress, BTreeSet<GridTableIdentityDependency>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridDependencyLayer {
    Structural,
    CalcOverlay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridInvalidationRef {
    structural: GridDependencyIndex,
    calc_overlay: GridDependencyIndex,
    volatile_roots: BTreeSet<ExcelGridCellAddress>,
    external_pending_roots: BTreeSet<ExcelGridCellAddress>,
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

fn rename_name_dependency(
    dependency: GridDependency,
    old_name_key: &str,
    new_name_key: &str,
) -> Option<GridDependency> {
    match dependency {
        GridDependency::Name(mut dependency) if dependency.name_key == old_name_key => {
            dependency.name_key = new_name_key.to_string();
            Some(GridDependency::Name(dependency))
        }
        GridDependency::NameIdentity(mut dependency) if dependency.name_key == old_name_key => {
            dependency.name_key = new_name_key.to_string();
            Some(GridDependency::NameIdentity(dependency))
        }
        GridDependency::ReferenceMetadata(dependency) => {
            rename_name_dependency(*dependency, old_name_key, new_name_key)
                .map(|dependency| GridDependency::ReferenceMetadata(Box::new(dependency)))
        }
        other => Some(other),
    }
}

fn delete_name_dependency(dependency: GridDependency, name_key: &str) -> Option<GridDependency> {
    match dependency {
        GridDependency::Name(dependency) if dependency.name_key == name_key => None,
        GridDependency::NameIdentity(dependency) if dependency.name_key == name_key => None,
        GridDependency::ReferenceMetadata(dependency) => {
            delete_name_dependency(*dependency, name_key)
                .map(|dependency| GridDependency::ReferenceMetadata(Box::new(dependency)))
        }
        other => Some(other),
    }
}

fn rename_table_dependency(
    dependency: GridDependency,
    old_table_key: &str,
    new_table_key: &str,
) -> Option<GridDependency> {
    match dependency {
        GridDependency::Table(mut dependency) if dependency.table_key == old_table_key => {
            dependency.table_key = new_table_key.to_string();
            Some(GridDependency::Table(dependency))
        }
        GridDependency::TableIdentity(mut dependency) if dependency.table_key == old_table_key => {
            dependency.table_key = new_table_key.to_string();
            Some(GridDependency::TableIdentity(dependency))
        }
        GridDependency::ReferenceMetadata(dependency) => {
            rename_table_dependency(*dependency, old_table_key, new_table_key)
                .map(|dependency| GridDependency::ReferenceMetadata(Box::new(dependency)))
        }
        other => Some(other),
    }
}

fn delete_table_dependency(dependency: GridDependency, table_key: &str) -> Option<GridDependency> {
    match dependency {
        GridDependency::Table(dependency) if dependency.table_key == table_key => None,
        GridDependency::TableIdentity(dependency) if dependency.table_key == table_key => None,
        GridDependency::ReferenceMetadata(dependency) => {
            delete_table_dependency(*dependency, table_key)
                .map(|dependency| GridDependency::ReferenceMetadata(Box::new(dependency)))
        }
        other => Some(other),
    }
}

fn resize_table_dependency(
    dependency: GridDependency,
    table_key: &str,
    new_extent: &GridRect,
) -> Option<GridDependency> {
    match dependency {
        GridDependency::Table(mut dependency) if dependency.table_key == table_key => {
            dependency.extent = new_extent.clone();
            Some(GridDependency::Table(dependency))
        }
        GridDependency::ReferenceMetadata(dependency) => {
            resize_table_dependency(*dependency, table_key, new_extent)
                .map(|dependency| GridDependency::ReferenceMetadata(Box::new(dependency)))
        }
        other => Some(other),
    }
}

impl GridDependencyIndex {
    #[must_use]
    pub fn with_scalarization_limit(bounds: ExcelGridBounds, scalarization_limit: u64) -> Self {
        Self {
            bounds,
            scalarization_limit,
            owning_sheet: None,
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
            name_identity_dependencies_by_cell: BTreeMap::new(),
            name_dependents_by_key: BTreeMap::new(),
            table_dependencies_by_cell: BTreeMap::new(),
            table_identity_dependencies_by_cell: BTreeMap::new(),
            table_dependents_by_key: BTreeMap::new(),
            dynamic_dependencies_by_cell: BTreeMap::new(),
            dynamic_dependents_by_request: BTreeMap::new(),
            metadata_spill_dependencies_by_cell: BTreeMap::new(),
            metadata_name_dependencies_by_cell: BTreeMap::new(),
            metadata_name_identity_dependencies_by_cell: BTreeMap::new(),
            metadata_table_dependencies_by_cell: BTreeMap::new(),
            metadata_table_identity_dependencies_by_cell: BTreeMap::new(),
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
        let mut name_identity_dependencies = BTreeSet::new();
        let mut table_dependencies = BTreeSet::new();
        let mut table_identity_dependencies = BTreeSet::new();
        let mut dynamic_dependencies = BTreeSet::new();
        let mut metadata_spill_dependencies = BTreeSet::new();
        let mut metadata_name_dependencies = BTreeSet::new();
        let mut metadata_name_identity_dependencies = BTreeSet::new();
        let mut metadata_table_dependencies = BTreeSet::new();
        let mut metadata_table_identity_dependencies = BTreeSet::new();

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
                GridDependency::NameIdentity(dependency) => {
                    semantic_dependencies.push(GridDependency::NameIdentity(dependency.clone()));
                    name_identity_dependencies.insert(dependency);
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
                GridDependency::TableIdentity(dependency) => {
                    semantic_dependencies.push(GridDependency::TableIdentity(dependency.clone()));
                    table_identity_dependencies.insert(dependency);
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
                GridDependency::ReferenceMetadata(dependency) => {
                    let dependency = *dependency;
                    match &dependency {
                        GridDependency::Cell(address) => {
                            self.check_address(address)?;
                        }
                        GridDependency::Range(rect) => {
                            self.check_rect(rect)?;
                        }
                        GridDependency::Name(dependency) => {
                            self.check_rect(&dependency.extent)?;
                            metadata_name_dependencies.insert(dependency.clone());
                        }
                        GridDependency::NameIdentity(dependency) => {
                            metadata_name_identity_dependencies.insert(dependency.clone());
                        }
                        GridDependency::Table(dependency) => {
                            self.check_rect(&dependency.extent)?;
                            metadata_table_dependencies.insert(dependency.clone());
                        }
                        GridDependency::TableIdentity(dependency) => {
                            metadata_table_identity_dependencies.insert(dependency.clone());
                        }
                        GridDependency::SpillFact(dependency) => {
                            self.check_address(&dependency.anchor)?;
                            metadata_spill_dependencies.insert(dependency.clone());
                        }
                        GridDependency::SpillBlocker(dependency) => {
                            self.check_rect(&dependency.extent)?;
                        }
                        GridDependency::AxisVisibility(dependency) => {
                            self.check_axis_visibility_dependency(dependency)?;
                        }
                        GridDependency::AxisValue(dependency) => {
                            self.check_axis_value_dependency(dependency)?;
                        }
                        GridDependency::SheetSpan(_)
                        | GridDependency::ReferenceMetadata(_)
                        | GridDependency::DynamicRequest(_) => {}
                    }
                    semantic_dependencies
                        .push(GridDependency::ReferenceMetadata(Box::new(dependency)));
                }
                // R3.9: the 3D sheet-span edge is stored as a semantic
                // dependency but registers no scalar/range/index consumer.
                // Its per-sheet fan is a closure-time expansion against the
                // current sheet order (W062 D2 §4.2); building that index and
                // its per-index dependents is **R4.12**. Storing the edge now
                // keeps authored truth and reserves the seat without faking a
                // (wrong, order-frozen) materialized fan.
                // R4.12: validate span endpoints against the sheet registry
                // here when the span-interval index lands.
                GridDependency::SheetSpan(dependency) => {
                    semantic_dependencies.push(GridDependency::SheetSpan(dependency));
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
        for dependency in &name_identity_dependencies {
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
        for dependency in &table_identity_dependencies {
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
        for dependency in &metadata_spill_dependencies {
            self.spill_dependents_by_anchor
                .entry(dependency.anchor.clone())
                .or_default()
                .insert(dependent.clone());
        }
        for dependency in &metadata_name_dependencies {
            self.name_dependents_by_key
                .entry(dependency.name_key.clone())
                .or_default()
                .insert(dependent.clone());
        }
        for dependency in &metadata_name_identity_dependencies {
            self.name_dependents_by_key
                .entry(dependency.name_key.clone())
                .or_default()
                .insert(dependent.clone());
        }
        for dependency in &metadata_table_dependencies {
            self.table_dependents_by_key
                .entry(dependency.table_key.clone())
                .or_default()
                .insert(dependent.clone());
        }
        for dependency in &metadata_table_identity_dependencies {
            self.table_dependents_by_key
                .entry(dependency.table_key.clone())
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
        if metadata_spill_dependencies.is_empty() {
            self.metadata_spill_dependencies_by_cell.remove(&dependent);
        } else {
            self.metadata_spill_dependencies_by_cell
                .insert(dependent.clone(), metadata_spill_dependencies);
        }
        if metadata_name_dependencies.is_empty() {
            self.metadata_name_dependencies_by_cell.remove(&dependent);
        } else {
            self.metadata_name_dependencies_by_cell
                .insert(dependent.clone(), metadata_name_dependencies);
        }
        if metadata_name_identity_dependencies.is_empty() {
            self.metadata_name_identity_dependencies_by_cell
                .remove(&dependent);
        } else {
            self.metadata_name_identity_dependencies_by_cell
                .insert(dependent.clone(), metadata_name_identity_dependencies);
        }
        if metadata_table_dependencies.is_empty() {
            self.metadata_table_dependencies_by_cell.remove(&dependent);
        } else {
            self.metadata_table_dependencies_by_cell
                .insert(dependent.clone(), metadata_table_dependencies);
        }
        if metadata_table_identity_dependencies.is_empty() {
            self.metadata_table_identity_dependencies_by_cell
                .remove(&dependent);
        } else {
            self.metadata_table_identity_dependencies_by_cell
                .insert(dependent.clone(), metadata_table_identity_dependencies);
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
        if name_identity_dependencies.is_empty() {
            self.name_identity_dependencies_by_cell.remove(&dependent);
        } else {
            self.name_identity_dependencies_by_cell
                .insert(dependent.clone(), name_identity_dependencies);
        }
        if table_dependencies.is_empty() {
            self.table_dependencies_by_cell.remove(&dependent);
        } else {
            self.table_dependencies_by_cell
                .insert(dependent.clone(), table_dependencies);
        }
        if table_identity_dependencies.is_empty() {
            self.table_identity_dependencies_by_cell.remove(&dependent);
        } else {
            self.table_identity_dependencies_by_cell
                .insert(dependent.clone(), table_identity_dependencies);
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

    pub fn clear_for_axis_edit(
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
        let dependent_cells_dropped = self.semantic_dependencies_by_cell.len();
        let semantic_dependencies_dropped = self
            .semantic_dependencies_by_cell
            .values()
            .map(Vec::len)
            .sum();

        *self = Self::with_scalarization_limit(self.bounds, self.scalarization_limit);

        Ok(GridInvalidationStructuralEditReport {
            edit,
            dependent_cells_kept: 0,
            dependent_cells_dropped,
            semantic_dependencies_kept: 0,
            semantic_dependencies_dropped,
            scalar_edges_before,
            scalar_edges_after: 0,
            compressed_range_edges_before,
            compressed_range_edges_after: 0,
            spill_edges_before,
            spill_edges_after: 0,
            spill_blocker_edges_before,
            spill_blocker_edges_after: 0,
            axis_value_edges_before,
            axis_value_edges_after: 0,
            name_edges_before,
            name_edges_after: 0,
            table_edges_before,
            table_edges_after: 0,
            dynamic_edges_before,
            dynamic_edges_after: 0,
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
            self.transform_namespace_dependencies(|dependency| {
                rename_name_dependency(dependency, &old_name_key, &new_name_key)
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
            self.transform_namespace_dependencies(|dependency| {
                delete_name_dependency(dependency, &name_key)
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
            self.transform_namespace_dependencies(|dependency| {
                rename_table_dependency(dependency, &old_table_key, &new_table_key)
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
            self.transform_namespace_dependencies(|dependency| {
                delete_table_dependency(dependency, &table_key)
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
            self.transform_namespace_dependencies(|dependency| {
                resize_table_dependency(dependency, &table_key, &new_extent)
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
    fn semantic_dependency_count(&self) -> usize {
        self.semantic_dependencies_by_cell
            .values()
            .map(Vec::len)
            .sum()
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
            .sum::<usize>()
            + self
                .metadata_spill_dependencies_by_cell
                .values()
                .map(BTreeSet::len)
                .sum::<usize>()
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
            .sum::<usize>()
            + self
                .name_identity_dependencies_by_cell
                .values()
                .map(BTreeSet::len)
                .sum::<usize>()
            + self
                .metadata_name_dependencies_by_cell
                .values()
                .map(BTreeSet::len)
                .sum::<usize>()
            + self
                .metadata_name_identity_dependencies_by_cell
                .values()
                .map(BTreeSet::len)
                .sum::<usize>()
    }

    #[must_use]
    pub fn table_edge_count(&self) -> usize {
        self.table_dependencies_by_cell
            .values()
            .map(BTreeSet::len)
            .sum::<usize>()
            + self
                .table_identity_dependencies_by_cell
                .values()
                .map(BTreeSet::len)
                .sum::<usize>()
            + self
                .metadata_table_dependencies_by_cell
                .values()
                .map(BTreeSet::len)
                .sum::<usize>()
            + self
                .metadata_table_identity_dependencies_by_cell
                .values()
                .map(BTreeSet::len)
                .sum::<usize>()
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
    pub fn name_identity_dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<GridNameIdentityDependency> {
        self.name_identity_dependencies_by_cell
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

    /// Whether `dependent` has at least one dependency in this layer whose
    /// target lies in `candidate_pool`, without materializing the full
    /// precedent set `effective_precedents_for_layer` would build. Mirrors
    /// that method's dependency-kind coverage (scalar edges, compressed
    /// ranges, spill anchors, axis-value spans, name/table extents)
    /// exactly, but returns on the first match via `Iterator::any`.
    #[must_use]
    fn has_pending_dependency_for(
        &self,
        dependent: &ExcelGridCellAddress,
        candidate_pool: &BTreeSet<ExcelGridCellAddress>,
    ) -> bool {
        if let Some(dependencies) = self.dependencies_by_cell.get(dependent)
            && dependencies
                .iter()
                .any(|dependency| candidate_pool.contains(dependency))
        {
            return true;
        }

        if let Some(dependencies) = self.compressed_range_dependencies_by_cell.get(dependent)
            && dependencies.iter().any(|dependency| {
                candidate_pool
                    .iter()
                    .any(|candidate| dependency.extent.contains(candidate))
            })
        {
            return true;
        }

        if let Some(dependencies) = self.spill_dependencies_by_cell.get(dependent)
            && dependencies
                .iter()
                .any(|dependency| candidate_pool.contains(&dependency.anchor))
        {
            return true;
        }

        if let Some(dependencies) = self.axis_value_dependencies_by_cell.get(dependent)
            && dependencies.iter().any(|dependency| {
                candidate_pool
                    .iter()
                    .any(|candidate| axis_value_dependency_contains_address(dependency, candidate))
            })
        {
            return true;
        }

        if let Some(dependencies) = self.name_dependencies_by_cell.get(dependent)
            && dependencies.iter().any(|dependency| {
                candidate_pool
                    .iter()
                    .any(|candidate| dependency.extent.contains(candidate))
            })
        {
            return true;
        }

        if let Some(dependencies) = self.table_dependencies_by_cell.get(dependent)
            && dependencies.iter().any(|dependency| {
                candidate_pool
                    .iter()
                    .any(|candidate| dependency.extent.contains(candidate))
            })
        {
            return true;
        }

        false
    }

    #[must_use]
    pub fn table_identity_dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<GridTableIdentityDependency> {
        self.table_identity_dependencies_by_cell
            .get(dependent)
            .cloned()
            .unwrap_or_default()
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
        if let Some(existing) = self.name_identity_dependencies_by_cell.remove(dependent) {
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
        if let Some(existing) = self.table_identity_dependencies_by_cell.remove(dependent) {
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
        if let Some(existing) = self.metadata_spill_dependencies_by_cell.remove(dependent) {
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
        if let Some(existing) = self.metadata_name_dependencies_by_cell.remove(dependent) {
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
        if let Some(existing) = self
            .metadata_name_identity_dependencies_by_cell
            .remove(dependent)
        {
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
        if let Some(existing) = self.metadata_table_dependencies_by_cell.remove(dependent) {
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
        if let Some(existing) = self
            .metadata_table_identity_dependencies_by_cell
            .remove(dependent)
        {
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

    fn clear_cell_dependencies(
        &mut self,
        dependent: &ExcelGridCellAddress,
    ) -> Result<usize, GridRefError> {
        self.check_address(dependent)?;
        let removed = self
            .semantic_dependencies_by_cell
            .get(dependent)
            .map_or(0, Vec::len);
        self.remove_existing_dependencies(dependent);
        Ok(removed)
    }

    fn direct_dependents_for_cell(
        &self,
        address: &ExcelGridCellAddress,
    ) -> BTreeSet<ExcelGridCellAddress> {
        let mut dependents = self.compressed_range_dependents_containing(address);
        dependents.extend(
            self.dependents_by_cell
                .get(address)
                .into_iter()
                .flat_map(|dependents| dependents.iter().cloned()),
        );
        dependents.extend(
            self.axis_value_dependents_by_index
                .get(&(GridAxis::Row, address.row))
                .into_iter()
                .flat_map(|dependents| dependents.iter().cloned()),
        );
        dependents.extend(
            self.axis_value_dependents_by_index
                .get(&(GridAxis::Column, address.col))
                .into_iter()
                .flat_map(|dependents| dependents.iter().cloned()),
        );
        dependents
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
        if let Some(owning_sheet) = &self.owning_sheet
            && !owning_sheet.owns_address(address)
        {
            return Err(GridRefError::ForeignSheetDependency {
                owning_workbook_id: owning_sheet.workbook_id.clone(),
                owning_sheet_id: owning_sheet.sheet_id.clone(),
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
        if let Some(owning_sheet) = &self.owning_sheet
            && !owning_sheet.owns_rect(rect)
        {
            return Err(GridRefError::ForeignSheetDependency {
                owning_workbook_id: owning_sheet.workbook_id.clone(),
                owning_sheet_id: owning_sheet.sheet_id.clone(),
                actual_workbook_id: rect.workbook_id.clone(),
                actual_sheet_id: rect.sheet_id.clone(),
            });
        }
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

    fn set_owning_sheet(&mut self, owning_sheet: OwningSheetIdentity) {
        self.owning_sheet = Some(owning_sheet);
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

impl GridInvalidationRef {
    #[must_use]
    pub fn new(bounds: ExcelGridBounds) -> Self {
        Self::with_scalarization_limit(bounds, GRID_INVALIDATION_REF_DEFAULT_SCALARIZATION_LIMIT)
    }

    #[must_use]
    pub fn with_scalarization_limit(bounds: ExcelGridBounds, scalarization_limit: u64) -> Self {
        Self {
            structural: GridDependencyIndex::with_scalarization_limit(bounds, scalarization_limit),
            calc_overlay: GridDependencyIndex::with_scalarization_limit(
                bounds,
                scalarization_limit,
            ),
            volatile_roots: BTreeSet::new(),
            external_pending_roots: BTreeSet::new(),
        }
    }

    /// Stamp this per-sheet invalidation ref with the identity of the sheet
    /// it belongs to, enabling the sheet-identity routing invariant (W062 D3
    /// §1/§2): once stamped, both the structural and calc-overlay indexes
    /// reject any dependency edge whose address resolves to a different sheet
    /// with [`GridRefError::ForeignSheetDependency`]. Constructing without
    /// this stamp preserves the historical unchecked behavior.
    #[must_use]
    pub fn with_owning_sheet(mut self, owning_sheet: OwningSheetIdentity) -> Self {
        self.set_owning_sheet(owning_sheet);
        self
    }

    /// Stamp (or restamp) the owning-sheet identity on both layers.
    pub fn set_owning_sheet(&mut self, owning_sheet: OwningSheetIdentity) {
        self.structural.set_owning_sheet(owning_sheet.clone());
        self.calc_overlay.set_owning_sheet(owning_sheet);
    }

    #[must_use]
    pub const fn bounds(&self) -> ExcelGridBounds {
        self.structural.bounds()
    }

    fn layer(&self, layer: GridDependencyLayer) -> &GridDependencyIndex {
        match layer {
            GridDependencyLayer::Structural => &self.structural,
            GridDependencyLayer::CalcOverlay => &self.calc_overlay,
        }
    }

    fn layer_mut(&mut self, layer: GridDependencyLayer) -> &mut GridDependencyIndex {
        match layer {
            GridDependencyLayer::Structural => &mut self.structural,
            GridDependencyLayer::CalcOverlay => &mut self.calc_overlay,
        }
    }

    pub fn set_cell_dependencies(
        &mut self,
        dependent: ExcelGridCellAddress,
        dependencies: impl IntoIterator<Item = GridDependency>,
    ) -> Result<usize, GridRefError> {
        self.set_structural_dependencies(dependent, dependencies)
    }

    pub fn set_structural_dependencies(
        &mut self,
        dependent: ExcelGridCellAddress,
        dependencies: impl IntoIterator<Item = GridDependency>,
    ) -> Result<usize, GridRefError> {
        self.structural
            .set_cell_dependencies(dependent, dependencies)
    }

    pub fn set_overlay_dependencies(
        &mut self,
        dependent: ExcelGridCellAddress,
        dependencies: impl IntoIterator<Item = GridDependency>,
    ) -> Result<usize, GridRefError> {
        self.calc_overlay
            .set_cell_dependencies(dependent, dependencies)
    }

    pub fn set_dependencies_in_layer(
        &mut self,
        layer: GridDependencyLayer,
        dependent: ExcelGridCellAddress,
        dependencies: impl IntoIterator<Item = GridDependency>,
    ) -> Result<usize, GridRefError> {
        self.layer_mut(layer)
            .set_cell_dependencies(dependent, dependencies)
    }

    pub fn clear_overlay_dependencies(
        &mut self,
        dependent: &ExcelGridCellAddress,
    ) -> Result<usize, GridRefError> {
        self.volatile_roots.remove(dependent);
        self.external_pending_roots.remove(dependent);
        self.calc_overlay.clear_cell_dependencies(dependent)
    }

    pub fn set_volatile_root(
        &mut self,
        address: ExcelGridCellAddress,
        volatile: bool,
    ) -> Result<bool, GridRefError> {
        self.structural.check_address(&address)?;
        Ok(if volatile {
            self.volatile_roots.insert(address)
        } else {
            self.volatile_roots.remove(&address)
        })
    }

    #[must_use]
    pub fn volatile_roots(&self) -> &BTreeSet<ExcelGridCellAddress> {
        &self.volatile_roots
    }

    #[must_use]
    pub fn has_volatile_roots(&self) -> bool {
        !self.volatile_roots.is_empty()
    }

    pub fn set_external_pending_root(
        &mut self,
        address: ExcelGridCellAddress,
        external_pending: bool,
    ) -> Result<bool, GridRefError> {
        self.structural.check_address(&address)?;
        Ok(if external_pending {
            self.external_pending_roots.insert(address)
        } else {
            self.external_pending_roots.remove(&address)
        })
    }

    #[must_use]
    pub fn external_pending_roots(&self) -> &BTreeSet<ExcelGridCellAddress> {
        &self.external_pending_roots
    }

    #[must_use]
    pub fn has_external_pending_roots(&self) -> bool {
        !self.external_pending_roots.is_empty()
    }

    pub fn apply_axis_edit(
        &mut self,
        edit: GridAxisEdit,
    ) -> Result<GridInvalidationStructuralEditReport, GridRefError> {
        let mut structural = self.structural.clone();
        let mut calc_overlay = self.calc_overlay.clone();
        let structural_report = structural.apply_axis_edit(edit)?;
        let overlay_report = calc_overlay.clear_for_axis_edit(edit)?;
        let mut volatile_roots = BTreeSet::new();
        for root in &self.volatile_roots {
            if let Some(transformed) = transform_address_for_edit(root, edit, self.bounds())? {
                volatile_roots.insert(transformed);
            }
        }
        let mut external_pending_roots = BTreeSet::new();
        for root in &self.external_pending_roots {
            if let Some(transformed) = transform_address_for_edit(root, edit, self.bounds())? {
                external_pending_roots.insert(transformed);
            }
        }
        self.structural = structural;
        self.calc_overlay = calc_overlay;
        self.volatile_roots = volatile_roots;
        self.external_pending_roots = external_pending_roots;

        Ok(GridInvalidationStructuralEditReport {
            edit,
            dependent_cells_kept: structural_report.dependent_cells_kept
                + overlay_report.dependent_cells_kept,
            dependent_cells_dropped: structural_report.dependent_cells_dropped
                + overlay_report.dependent_cells_dropped,
            semantic_dependencies_kept: structural_report.semantic_dependencies_kept
                + overlay_report.semantic_dependencies_kept,
            semantic_dependencies_dropped: structural_report.semantic_dependencies_dropped
                + overlay_report.semantic_dependencies_dropped,
            scalar_edges_before: structural_report.scalar_edges_before
                + overlay_report.scalar_edges_before,
            scalar_edges_after: structural_report.scalar_edges_after
                + overlay_report.scalar_edges_after,
            compressed_range_edges_before: structural_report.compressed_range_edges_before
                + overlay_report.compressed_range_edges_before,
            compressed_range_edges_after: structural_report.compressed_range_edges_after
                + overlay_report.compressed_range_edges_after,
            spill_edges_before: structural_report.spill_edges_before
                + overlay_report.spill_edges_before,
            spill_edges_after: structural_report.spill_edges_after
                + overlay_report.spill_edges_after,
            spill_blocker_edges_before: structural_report.spill_blocker_edges_before
                + overlay_report.spill_blocker_edges_before,
            spill_blocker_edges_after: structural_report.spill_blocker_edges_after
                + overlay_report.spill_blocker_edges_after,
            axis_value_edges_before: structural_report.axis_value_edges_before
                + overlay_report.axis_value_edges_before,
            axis_value_edges_after: structural_report.axis_value_edges_after
                + overlay_report.axis_value_edges_after,
            name_edges_before: structural_report.name_edges_before
                + overlay_report.name_edges_before,
            name_edges_after: structural_report.name_edges_after + overlay_report.name_edges_after,
            table_edges_before: structural_report.table_edges_before
                + overlay_report.table_edges_before,
            table_edges_after: structural_report.table_edges_after
                + overlay_report.table_edges_after,
            dynamic_edges_before: structural_report.dynamic_edges_before
                + overlay_report.dynamic_edges_before,
            dynamic_edges_after: structural_report.dynamic_edges_after
                + overlay_report.dynamic_edges_after,
        })
    }

    pub fn rename_defined_name(
        &mut self,
        old_name: impl AsRef<str>,
        new_name: impl AsRef<str>,
    ) -> Result<GridInvalidationNamespaceLifecycleReport, GridRefError> {
        let old_name_key = defined_name_key_for_name(old_name.as_ref(), self.bounds())?;
        let new_name_key = defined_name_key_for_name(new_name.as_ref(), self.bounds())?;
        let dirty_closure = self.dirty_closure_for_name_keys([&old_name_key, &new_name_key]);
        let name_edges_before = self.name_edge_count();
        let table_edges_before = self.table_edge_count();

        let mut structural = self.structural.clone();
        let mut calc_overlay = self.calc_overlay.clone();
        let structural_report =
            structural.rename_defined_name(old_name.as_ref(), new_name.as_ref())?;
        let overlay_report =
            calc_overlay.rename_defined_name(old_name.as_ref(), new_name.as_ref())?;
        self.structural = structural;
        self.calc_overlay = calc_overlay;

        Ok(GridInvalidationNamespaceLifecycleReport {
            operation: GridInvalidationNamespaceLifecycleOperation::RenameName {
                old_name_key,
                new_name_key,
            },
            dirty_closure,
            semantic_dependencies_kept: structural_report.semantic_dependencies_kept
                + overlay_report.semantic_dependencies_kept,
            semantic_dependencies_dropped: structural_report.semantic_dependencies_dropped
                + overlay_report.semantic_dependencies_dropped,
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
        let name_key = defined_name_key_for_name(name.as_ref(), self.bounds())?;
        let dirty_closure = self.dirty_closure_for_name_keys([&name_key]);
        let name_edges_before = self.name_edge_count();
        let table_edges_before = self.table_edge_count();

        let mut structural = self.structural.clone();
        let mut calc_overlay = self.calc_overlay.clone();
        let structural_report = structural.delete_defined_name(name.as_ref())?;
        let overlay_report = calc_overlay.delete_defined_name(name.as_ref())?;
        self.structural = structural;
        self.calc_overlay = calc_overlay;

        Ok(GridInvalidationNamespaceLifecycleReport {
            operation: GridInvalidationNamespaceLifecycleOperation::DeleteName { name_key },
            dirty_closure,
            semantic_dependencies_kept: structural_report.semantic_dependencies_kept
                + overlay_report.semantic_dependencies_kept,
            semantic_dependencies_dropped: structural_report.semantic_dependencies_dropped
                + overlay_report.semantic_dependencies_dropped,
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
        let old_table_key = table_key_for_name(old_table_name.as_ref(), self.bounds())?;
        let new_table_key = table_key_for_name(new_table_name.as_ref(), self.bounds())?;
        let dirty_closure = self.dirty_closure_for_table_keys([&old_table_key, &new_table_key]);
        let name_edges_before = self.name_edge_count();
        let table_edges_before = self.table_edge_count();

        let mut structural = self.structural.clone();
        let mut calc_overlay = self.calc_overlay.clone();
        let structural_report =
            structural.rename_table(old_table_name.as_ref(), new_table_name.as_ref())?;
        let overlay_report =
            calc_overlay.rename_table(old_table_name.as_ref(), new_table_name.as_ref())?;
        self.structural = structural;
        self.calc_overlay = calc_overlay;

        Ok(GridInvalidationNamespaceLifecycleReport {
            operation: GridInvalidationNamespaceLifecycleOperation::RenameTable {
                old_table_key,
                new_table_key,
            },
            dirty_closure,
            semantic_dependencies_kept: structural_report.semantic_dependencies_kept
                + overlay_report.semantic_dependencies_kept,
            semantic_dependencies_dropped: structural_report.semantic_dependencies_dropped
                + overlay_report.semantic_dependencies_dropped,
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
        let table_key = table_key_for_name(table_name.as_ref(), self.bounds())?;
        let dirty_closure = self.dirty_closure_for_table_keys([&table_key]);
        let name_edges_before = self.name_edge_count();
        let table_edges_before = self.table_edge_count();

        let mut structural = self.structural.clone();
        let mut calc_overlay = self.calc_overlay.clone();
        let structural_report = structural.delete_table(table_name.as_ref())?;
        let overlay_report = calc_overlay.delete_table(table_name.as_ref())?;
        self.structural = structural;
        self.calc_overlay = calc_overlay;

        Ok(GridInvalidationNamespaceLifecycleReport {
            operation: GridInvalidationNamespaceLifecycleOperation::DeleteTable { table_key },
            dirty_closure,
            semantic_dependencies_kept: structural_report.semantic_dependencies_kept
                + overlay_report.semantic_dependencies_kept,
            semantic_dependencies_dropped: structural_report.semantic_dependencies_dropped
                + overlay_report.semantic_dependencies_dropped,
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
        self.structural.check_rect(&new_extent)?;
        let table_key = table_key_for_name(table_name.as_ref(), self.bounds())?;
        let dirty_closure = self.dirty_closure_for_table_keys([&table_key]);
        let name_edges_before = self.name_edge_count();
        let table_edges_before = self.table_edge_count();

        let mut structural = self.structural.clone();
        let mut calc_overlay = self.calc_overlay.clone();
        let structural_report = structural.resize_table(table_name.as_ref(), new_extent.clone())?;
        let overlay_report = calc_overlay.resize_table(table_name.as_ref(), new_extent)?;
        self.structural = structural;
        self.calc_overlay = calc_overlay;

        Ok(GridInvalidationNamespaceLifecycleReport {
            operation: GridInvalidationNamespaceLifecycleOperation::ResizeTable { table_key },
            dirty_closure,
            semantic_dependencies_kept: structural_report.semantic_dependencies_kept
                + overlay_report.semantic_dependencies_kept,
            semantic_dependencies_dropped: structural_report.semantic_dependencies_dropped
                + overlay_report.semantic_dependencies_dropped,
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
        let mut dependencies = self.structural.dependencies_for(dependent);
        dependencies.extend(self.calc_overlay.dependencies_for(dependent));
        dependencies
    }

    #[must_use]
    pub fn next_ready_dirty_formula(
        &self,
        pending: &BTreeSet<ExcelGridCellAddress>,
    ) -> Option<ExcelGridCellAddress> {
        pending
            .iter()
            .find(|candidate| !self.has_pending_precedent(candidate, pending))
            .cloned()
    }

    /// Whether `candidate` (assumed a member of `pending`) has no
    /// outstanding pending precedent within `pending`, i.e. it is safe to
    /// evaluate `candidate` next regardless of where it sorts by address.
    /// Unlike `next_ready_dirty_formula`, which always returns the
    /// address-order-first ready member of `pending`, this checks the
    /// readiness of one specific candidate — the primitive a caller needs to
    /// validate a non-address-order execution sequence (e.g. a fast path
    /// that evaluates sparse formulas before repeated-region formulas).
    #[must_use]
    pub fn is_formula_ready(
        &self,
        candidate: &ExcelGridCellAddress,
        pending: &BTreeSet<ExcelGridCellAddress>,
    ) -> bool {
        !self.has_pending_precedent(candidate, pending)
    }

    /// Whether `dependent` has at least one outstanding pending precedent
    /// within `pending`, across both dependency layers. Readiness selection
    /// (`next_ready_dirty_formula`, `is_formula_ready`) only needs this
    /// boolean, not the full precedent set `pending_formula_precedents`
    /// builds, so this returns as soon as either layer reports a match
    /// instead of materializing and unioning two `BTreeSet`s per candidate
    /// per pick (previously O(P^2..P^3) across a worklist of size P).
    #[must_use]
    fn has_pending_precedent(
        &self,
        dependent: &ExcelGridCellAddress,
        pending: &BTreeSet<ExcelGridCellAddress>,
    ) -> bool {
        self.structural
            .has_pending_dependency_for(dependent, pending)
            || self
                .calc_overlay
                .has_pending_dependency_for(dependent, pending)
    }

    /// The full pending-precedent set for `dependent` across both layers.
    /// Readiness selection (`next_ready_dirty_formula`, `is_formula_ready`)
    /// only needs `has_pending_precedent`'s boolean and no longer calls
    /// this; kept as the full-set builder for any caller that needs the
    /// actual precedent addresses rather than just an emptiness check.
    #[must_use]
    #[allow(dead_code)]
    fn pending_formula_precedents(
        &self,
        dependent: &ExcelGridCellAddress,
        pending: &BTreeSet<ExcelGridCellAddress>,
    ) -> BTreeSet<ExcelGridCellAddress> {
        let mut precedents = self.effective_precedents_for_layer(
            GridDependencyLayer::Structural,
            dependent,
            pending,
        );
        precedents.extend(self.effective_precedents_for_layer(
            GridDependencyLayer::CalcOverlay,
            dependent,
            pending,
        ));
        precedents
    }

    /// The full effective precedent relation for `dependent` within a single
    /// dependency layer, restricted to members of `candidate_pool`: every
    /// kind of dependency readiness gating considers (scalar cell/range
    /// edges, compressed-range spans, spill anchors, axis-value spans, and
    /// name/table extents). This is the same relation
    /// `next_ready_dirty_formula` uses for scheduling; the cycle DFS reuses
    /// it (restricted to a bounded candidate pool, layer by layer) so a real
    /// cycle composed of non-scalar edges is not missed, and so a
    /// still-pending `CalcOverlay` edge can be told apart from a settled
    /// one.
    #[must_use]
    fn effective_precedents_for_layer(
        &self,
        layer: GridDependencyLayer,
        dependent: &ExcelGridCellAddress,
        candidate_pool: &BTreeSet<ExcelGridCellAddress>,
    ) -> BTreeSet<ExcelGridCellAddress> {
        let index = self.layer(layer);
        let mut precedents = index
            .dependencies_for(dependent)
            .into_iter()
            .filter(|dependency| candidate_pool.contains(dependency))
            .collect::<BTreeSet<_>>();

        for extent in index.compressed_range_dependencies_for(dependent) {
            precedents.extend(
                candidate_pool
                    .iter()
                    .filter(|candidate| extent.contains(candidate))
                    .cloned(),
            );
        }

        for dependency in index.spill_dependencies_for(dependent) {
            if candidate_pool.contains(&dependency.anchor) {
                precedents.insert(dependency.anchor);
            }
        }

        for dependency in index.axis_value_dependencies_for(dependent) {
            precedents.extend(
                candidate_pool
                    .iter()
                    .filter(|candidate| {
                        axis_value_dependency_contains_address(&dependency, candidate)
                    })
                    .cloned(),
            );
        }

        for dependency in index.name_dependencies_for(dependent) {
            precedents.extend(
                candidate_pool
                    .iter()
                    .filter(|candidate| dependency.extent.contains(candidate))
                    .cloned(),
            );
        }

        for dependency in index.table_dependencies_for(dependent) {
            precedents.extend(
                candidate_pool
                    .iter()
                    .filter(|candidate| dependency.extent.contains(candidate))
                    .cloned(),
            );
        }

        precedents
    }

    #[must_use]
    pub fn first_pending_with_overlay_dependencies(
        &self,
        pending: &BTreeSet<ExcelGridCellAddress>,
    ) -> Option<ExcelGridCellAddress> {
        pending
            .iter()
            .find(|candidate| {
                !self
                    .semantic_dependencies_for_layer(GridDependencyLayer::CalcOverlay, candidate)
                    .is_empty()
            })
            .cloned()
    }

    /// Searches for a genuine effective-dependency cycle reachable from
    /// `start`, restricted to `pending` (the deadlocked/still-unevaluated
    /// candidate set at the point the scheduler gave up finding a ready
    /// formula). Restricting to `pending` both guarantees DFS termination
    /// (`pending` is finite) and avoids false positives: a `CalcOverlay`
    /// edge belonging to a cell that is itself still `pending` is its
    /// *previous* pass's overlay, about to be replaced when that cell is
    /// evaluated, so it is treated as unresolved rather than a settled edge
    /// that can complete a cycle. `Structural` edges are never stale in this
    /// way (they are reinstalled from the authored formula before
    /// evaluation, not discovered at runtime), so they are always walked.
    #[must_use]
    pub fn effective_dependency_cycle_from(
        &self,
        start: &ExcelGridCellAddress,
        pending: &BTreeSet<ExcelGridCellAddress>,
    ) -> Option<Vec<ExcelGridCellAddress>> {
        let mut path = Vec::new();
        let mut visited = BTreeSet::new();
        self.effective_dependency_cycle_dfs(start, start, pending, &mut path, &mut visited)
    }

    fn effective_dependency_cycle_dfs(
        &self,
        start: &ExcelGridCellAddress,
        current: &ExcelGridCellAddress,
        pending: &BTreeSet<ExcelGridCellAddress>,
        path: &mut Vec<ExcelGridCellAddress>,
        visited: &mut BTreeSet<ExcelGridCellAddress>,
    ) -> Option<Vec<ExcelGridCellAddress>> {
        if !visited.insert(current.clone()) {
            return None;
        }
        path.push(current.clone());

        let mut candidate_pool = pending.clone();
        candidate_pool.insert(start.clone());
        let mut edges = self.effective_precedents_for_layer(
            GridDependencyLayer::Structural,
            current,
            &candidate_pool,
        );
        // A CalcOverlay edge owned by a cell still pending this pass has not
        // been settled yet: that cell is about to replace its overlay
        // dependencies when it evaluates, so this edge cannot be used to
        // conclude a real cycle. Only settled (non-pending) cells contribute
        // their CalcOverlay edges to the search.
        if !pending.contains(current) {
            edges.extend(self.effective_precedents_for_layer(
                GridDependencyLayer::CalcOverlay,
                current,
                &candidate_pool,
            ));
        }

        for dependency in edges {
            if dependency == *start {
                let mut cycle = path.clone();
                cycle.push(start.clone());
                return Some(cycle);
            }
            if path.contains(&dependency) {
                continue;
            }
            if let Some(cycle) =
                self.effective_dependency_cycle_dfs(start, &dependency, pending, path, visited)
            {
                return Some(cycle);
            }
        }
        path.pop();
        None
    }

    #[must_use]
    pub fn dependencies_for_layer(
        &self,
        layer: GridDependencyLayer,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<ExcelGridCellAddress> {
        self.layer(layer).dependencies_for(dependent)
    }

    #[must_use]
    pub fn semantic_dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> Vec<GridDependency> {
        let mut dependencies = self.structural.semantic_dependencies_for(dependent);
        dependencies.extend(self.calc_overlay.semantic_dependencies_for(dependent));
        dependencies
    }

    #[must_use]
    pub fn semantic_dependencies_for_layer(
        &self,
        layer: GridDependencyLayer,
        dependent: &ExcelGridCellAddress,
    ) -> Vec<GridDependency> {
        self.layer(layer).semantic_dependencies_for(dependent)
    }

    #[must_use]
    pub fn semantic_dependency_count_for_layer(&self, layer: GridDependencyLayer) -> usize {
        self.layer(layer).semantic_dependency_count()
    }

    #[must_use]
    pub fn scalar_edge_count(&self) -> usize {
        self.structural.scalar_edge_count() + self.calc_overlay.scalar_edge_count()
    }

    #[must_use]
    pub fn compressed_range_edge_count(&self) -> usize {
        self.structural.compressed_range_edge_count()
            + self.calc_overlay.compressed_range_edge_count()
    }

    #[must_use]
    pub fn compressed_range_dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<GridRect> {
        let mut dependencies = self.structural.compressed_range_dependencies_for(dependent);
        dependencies.extend(
            self.calc_overlay
                .compressed_range_dependencies_for(dependent),
        );
        dependencies
    }

    pub fn compressed_range_query_report(
        &self,
        seed: ExcelGridCellAddress,
    ) -> Result<GridCompressedRangeQueryReport, GridRefError> {
        let structural = self
            .structural
            .compressed_range_query_report(seed.clone())?;
        let overlay = self
            .calc_overlay
            .compressed_range_query_report(seed.clone())?;
        let mut dependents = structural.dependents;
        dependents.extend(overlay.dependents);
        Ok(GridCompressedRangeQueryReport {
            seed,
            indexed_candidate_count: structural.indexed_candidate_count
                + overlay.indexed_candidate_count,
            matched_dependent_count: dependents.len(),
            total_compressed_range_edges: structural.total_compressed_range_edges
                + overlay.total_compressed_range_edges,
            dependents,
        })
    }

    pub fn dirty_rect_query_report(
        &self,
        rect: GridRect,
    ) -> Result<GridDirtyRectQueryReport, GridRefError> {
        let structural = self.structural.dirty_rect_query_report(rect.clone())?;
        let overlay = self.calc_overlay.dirty_rect_query_report(rect.clone())?;
        let mut direct_dependents = structural.direct_dependents;
        direct_dependents.extend(overlay.direct_dependents);
        let dirty_closure = self.dirty_closure(direct_dependents.iter().cloned());
        Ok(GridDirtyRectQueryReport {
            rect,
            seed_rect_cell_count: structural.seed_rect_cell_count,
            indexed_scalar_candidate_count: structural.indexed_scalar_candidate_count
                + overlay.indexed_scalar_candidate_count,
            matched_scalar_dependent_count: structural.matched_scalar_dependent_count
                + overlay.matched_scalar_dependent_count,
            indexed_compressed_range_candidate_count: structural
                .indexed_compressed_range_candidate_count
                + overlay.indexed_compressed_range_candidate_count,
            matched_compressed_range_dependent_count: structural
                .matched_compressed_range_dependent_count
                + overlay.matched_compressed_range_dependent_count,
            total_scalar_edges: structural.total_scalar_edges + overlay.total_scalar_edges,
            total_compressed_range_edges: structural.total_compressed_range_edges
                + overlay.total_compressed_range_edges,
            direct_dependents,
            dirty_closure,
        })
    }

    #[must_use]
    pub fn spill_edge_count(&self) -> usize {
        self.structural.spill_edge_count() + self.calc_overlay.spill_edge_count()
    }

    #[must_use]
    pub fn spill_blocker_edge_count(&self) -> usize {
        self.structural.spill_blocker_edge_count() + self.calc_overlay.spill_blocker_edge_count()
    }

    #[must_use]
    pub fn axis_value_edge_count(&self) -> usize {
        self.structural.axis_value_edge_count() + self.calc_overlay.axis_value_edge_count()
    }

    #[must_use]
    pub fn name_edge_count(&self) -> usize {
        self.structural.name_edge_count() + self.calc_overlay.name_edge_count()
    }

    #[must_use]
    pub fn table_edge_count(&self) -> usize {
        self.structural.table_edge_count() + self.calc_overlay.table_edge_count()
    }

    #[must_use]
    pub fn dynamic_edge_count(&self) -> usize {
        self.structural.dynamic_edge_count() + self.calc_overlay.dynamic_edge_count()
    }

    #[must_use]
    pub fn dynamic_dependencies_for(&self, dependent: &ExcelGridCellAddress) -> BTreeSet<String> {
        let mut dependencies = self.structural.dynamic_dependencies_for(dependent);
        dependencies.extend(self.calc_overlay.dynamic_dependencies_for(dependent));
        dependencies
    }

    #[must_use]
    pub fn dynamic_dependencies_for_layer(
        &self,
        layer: GridDependencyLayer,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<String> {
        self.layer(layer).dynamic_dependencies_for(dependent)
    }

    #[must_use]
    pub fn spill_dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<GridSpillDependency> {
        let mut dependencies = self.structural.spill_dependencies_for(dependent);
        dependencies.extend(self.calc_overlay.spill_dependencies_for(dependent));
        dependencies
    }

    #[must_use]
    pub fn spill_blocker_dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<GridSpillBlockerDependency> {
        let mut dependencies = self.structural.spill_blocker_dependencies_for(dependent);
        dependencies.extend(self.calc_overlay.spill_blocker_dependencies_for(dependent));
        dependencies
    }

    #[must_use]
    pub fn axis_visibility_dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<GridAxisVisibilityDependency> {
        let mut dependencies = self.structural.axis_visibility_dependencies_for(dependent);
        dependencies.extend(
            self.calc_overlay
                .axis_visibility_dependencies_for(dependent),
        );
        dependencies
    }

    #[must_use]
    pub fn axis_visibility_edge_count(&self) -> usize {
        self.structural.axis_visibility_edge_count()
            + self.calc_overlay.axis_visibility_edge_count()
    }

    pub fn axis_visibility_query_report(
        &self,
        dependency: GridAxisVisibilityDependency,
    ) -> Result<GridAxisVisibilityQueryReport, GridRefError> {
        let structural = self
            .structural
            .axis_visibility_query_report(dependency.clone())?;
        let overlay = self
            .calc_overlay
            .axis_visibility_query_report(dependency.clone())?;
        let mut dependents = structural.dependents;
        dependents.extend(overlay.dependents);
        Ok(GridAxisVisibilityQueryReport {
            dependency,
            indexed_candidate_count: structural.indexed_candidate_count
                + overlay.indexed_candidate_count,
            matched_dependent_count: dependents.len(),
            total_axis_visibility_edges: structural.total_axis_visibility_edges
                + overlay.total_axis_visibility_edges,
            dependents,
        })
    }

    #[must_use]
    pub fn axis_value_dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<GridAxisValueDependency> {
        let mut dependencies = self.structural.axis_value_dependencies_for(dependent);
        dependencies.extend(self.calc_overlay.axis_value_dependencies_for(dependent));
        dependencies
    }

    #[must_use]
    pub fn name_dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<GridNameDependency> {
        let mut dependencies = self.structural.name_dependencies_for(dependent);
        dependencies.extend(self.calc_overlay.name_dependencies_for(dependent));
        dependencies
    }

    #[must_use]
    pub fn name_identity_dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<GridNameIdentityDependency> {
        let mut dependencies = self.structural.name_identity_dependencies_for(dependent);
        dependencies.extend(self.calc_overlay.name_identity_dependencies_for(dependent));
        dependencies
    }

    #[must_use]
    pub fn table_dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<GridTableDependency> {
        let mut dependencies = self.structural.table_dependencies_for(dependent);
        dependencies.extend(self.calc_overlay.table_dependencies_for(dependent));
        dependencies
    }

    #[must_use]
    pub fn table_identity_dependencies_for(
        &self,
        dependent: &ExcelGridCellAddress,
    ) -> BTreeSet<GridTableIdentityDependency> {
        let mut dependencies = self.structural.table_identity_dependencies_for(dependent);
        dependencies.extend(self.calc_overlay.table_identity_dependencies_for(dependent));
        dependencies
    }

    #[must_use]
    pub fn dirty_closure(
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
            let mut dependents = self.structural.direct_dependents_for_cell(&address);
            dependents.extend(self.calc_overlay.direct_dependents_for_cell(&address));
            for dependent in dependents {
                if dirty.insert(dependent.clone()) {
                    queue.push_back(dependent);
                }
            }
        }

        dirty
    }

    #[must_use]
    pub fn dirty_closure_for_dynamic_request(
        &self,
        request_key: &str,
    ) -> BTreeSet<ExcelGridCellAddress> {
        let seeds = self
            .structural
            .dynamic_dependents_by_request
            .get(request_key)
            .into_iter()
            .chain(
                self.calc_overlay
                    .dynamic_dependents_by_request
                    .get(request_key)
                    .into_iter(),
            )
            .flat_map(|dependents| dependents.iter().cloned());
        self.dirty_closure(seeds)
    }

    pub fn dirty_closure_for_seed(
        &self,
        seed: GridDirtySeed,
    ) -> Result<BTreeSet<ExcelGridCellAddress>, GridRefError> {
        match seed {
            GridDirtySeed::Cell(address) => {
                self.structural.check_address(&address)?;
                let mut dirty = self.dirty_closure([address.clone()]);
                dirty.extend(self.dirty_closure_for_spill_blocker(
                    GridSpillBlockerDependency::extent(anchor_cell_rect(&address, self.bounds())),
                )?);
                Ok(dirty)
            }
            GridDirtySeed::Range(rect) => {
                let mut dirty = self.dirty_rect_query_report(rect.clone())?.dirty_closure;
                dirty.extend(
                    self.dirty_closure_for_spill_blocker(GridSpillBlockerDependency::extent(rect))?,
                );
                Ok(dirty)
            }
            GridDirtySeed::SpillFact(dependency) => self.dirty_closure_for_spill_fact(dependency),
            GridDirtySeed::SpillBlocker(dependency) => {
                self.dirty_closure_for_spill_blocker(dependency)
            }
            GridDirtySeed::AxisVisibility(dependency) => {
                self.dirty_closure_for_axis_visibility(dependency)
            }
            GridDirtySeed::AxisValue(dependency) => self.dirty_closure_for_axis_value(dependency),
            GridDirtySeed::Name(name) => self.dirty_closure_for_name(name),
            GridDirtySeed::Table(table_name) => self.dirty_closure_for_table(table_name),
            GridDirtySeed::DynamicRequest(request_key) => {
                Ok(self.dirty_closure_for_dynamic_request(&request_key))
            }
            GridDirtySeed::Volatile => Ok(self.dirty_closure(self.volatile_roots.iter().cloned())),
            GridDirtySeed::External => {
                Ok(self.dirty_closure(self.external_pending_roots.iter().cloned()))
            }
        }
    }

    pub fn dirty_closure_for_seeds(
        &self,
        seeds: impl IntoIterator<Item = GridDirtySeed>,
    ) -> Result<GridDirtyClosure, GridRefError> {
        let seeds = seeds.into_iter().collect::<BTreeSet<_>>();
        let mut dirty_cells = BTreeSet::new();
        for seed in seeds.iter().cloned() {
            dirty_cells.extend(self.dirty_closure_for_seed(seed)?);
        }
        Ok(GridDirtyClosure { seeds, dirty_cells })
    }

    pub fn dirty_closure_for_spill_fact(
        &self,
        dependency: GridSpillDependency,
    ) -> Result<BTreeSet<ExcelGridCellAddress>, GridRefError> {
        self.structural.check_address(&dependency.anchor)?;
        let seeds = self
            .structural
            .spill_dependents_by_anchor
            .get(&dependency.anchor)
            .into_iter()
            .chain(
                self.calc_overlay
                    .spill_dependents_by_anchor
                    .get(&dependency.anchor)
                    .into_iter(),
            )
            .flat_map(|dependents| dependents.iter().cloned());
        Ok(self.dirty_closure(seeds))
    }

    pub fn dirty_closure_for_spill_epoch_changes(
        &self,
        old_snapshots: impl IntoIterator<Item = GridSpillEpochSnapshot>,
        new_snapshots: impl IntoIterator<Item = GridSpillEpochSnapshot>,
    ) -> Result<GridSpillEpochInvalidationReport, GridRefError> {
        let mut report = self
            .structural
            .dirty_closure_for_spill_epoch_changes(old_snapshots, new_snapshots)?;
        let mut dirty_closure = BTreeSet::new();
        for change in &report.changed_anchors {
            dirty_closure.extend(self.dirty_closure_for_spill_fact(
                GridSpillDependency::anchor(change.anchor.clone()),
            )?);
        }
        report.dirty_closure = dirty_closure;
        Ok(report)
    }

    pub fn dirty_closure_for_spill_blocker(
        &self,
        dependency: GridSpillBlockerDependency,
    ) -> Result<BTreeSet<ExcelGridCellAddress>, GridRefError> {
        self.structural.check_rect(&dependency.extent)?;
        let cells = dependency
            .extent
            .scalar_cells(self.structural.scalarization_limit)?;
        let seeds = cells.into_iter().flat_map(|address| {
            self.structural
                .spill_blocker_dependents_by_cell
                .get(&address)
                .into_iter()
                .chain(
                    self.calc_overlay
                        .spill_blocker_dependents_by_cell
                        .get(&address)
                        .into_iter(),
                )
                .flat_map(|dependents| dependents.iter().cloned())
        });
        Ok(self.dirty_closure(seeds))
    }

    pub fn dirty_closure_for_axis_visibility(
        &self,
        dependency: GridAxisVisibilityDependency,
    ) -> Result<BTreeSet<ExcelGridCellAddress>, GridRefError> {
        let report = self.axis_visibility_query_report(dependency)?;
        Ok(self.dirty_closure(report.dependents))
    }

    pub fn dirty_closure_for_axis_value(
        &self,
        dependency: GridAxisValueDependency,
    ) -> Result<BTreeSet<ExcelGridCellAddress>, GridRefError> {
        self.structural.check_axis_value_dependency(&dependency)?;
        let seeds = (dependency.first..=dependency.last).flat_map(|index| {
            self.structural
                .axis_value_dependents_by_index
                .get(&(dependency.axis, index))
                .into_iter()
                .chain(
                    self.calc_overlay
                        .axis_value_dependents_by_index
                        .get(&(dependency.axis, index))
                        .into_iter(),
                )
                .flat_map(|dependents| dependents.iter().cloned())
        });
        Ok(self.dirty_closure(seeds))
    }

    pub fn dirty_closure_for_name(
        &self,
        name: impl AsRef<str>,
    ) -> Result<BTreeSet<ExcelGridCellAddress>, GridRefError> {
        let Some(name_keys) = excel_grid_defined_name_seed_keys(name.as_ref(), self.bounds())
        else {
            return Err(GridRefError::InvalidDefinedName {
                name: name.as_ref().to_string(),
            });
        };
        Ok(self.dirty_closure_for_name_keys(name_keys.iter()))
    }

    pub fn dirty_closure_for_table(
        &self,
        table_name: impl AsRef<str>,
    ) -> Result<BTreeSet<ExcelGridCellAddress>, GridRefError> {
        let Some(table_key) = excel_grid_table_name_key(table_name.as_ref(), self.bounds()) else {
            return Err(GridRefError::InvalidTableName {
                name: table_name.as_ref().to_string(),
            });
        };
        Ok(self.dirty_closure_for_table_keys([&table_key]))
    }

    fn dirty_closure_for_name_keys<'a>(
        &self,
        keys: impl IntoIterator<Item = &'a String>,
    ) -> BTreeSet<ExcelGridCellAddress> {
        let keys = keys.into_iter().collect::<Vec<_>>();
        let seeds = keys.into_iter().flat_map(|key| {
            self.structural
                .name_dependents_by_key
                .get(key)
                .into_iter()
                .chain(
                    self.calc_overlay
                        .name_dependents_by_key
                        .get(key)
                        .into_iter(),
                )
                .flat_map(|dependents| dependents.iter().cloned())
        });
        self.dirty_closure(seeds)
    }

    fn dirty_closure_for_table_keys<'a>(
        &self,
        keys: impl IntoIterator<Item = &'a String>,
    ) -> BTreeSet<ExcelGridCellAddress> {
        let keys = keys.into_iter().collect::<Vec<_>>();
        let seeds = keys.into_iter().flat_map(|key| {
            self.structural
                .table_dependents_by_key
                .get(key)
                .into_iter()
                .chain(
                    self.calc_overlay
                        .table_dependents_by_key
                        .get(key)
                        .into_iter(),
                )
                .flat_map(|dependents| dependents.iter().cloned())
        });
        self.dirty_closure(seeds)
    }
}

/// `pub(super)` (not module-private) so `runtime_trace.rs`'s
/// `grid_dependency_covers` and `host_info.rs`'s
/// `grid_dependency_contains_cell`/`grid_dependency_overlaps_rect` can match
/// `GridDependency::AxisValue` against a cell/rect using the exact same
/// span-containment rule this module's own AxisValue candidate filtering
/// uses, instead of re-deriving (and risking drifting from) the rule. G6.
pub(super) fn axis_value_dependency_contains_address(
    dependency: &GridAxisValueDependency,
    address: &ExcelGridCellAddress,
) -> bool {
    match dependency.axis {
        GridAxis::Row => (dependency.first..=dependency.last).contains(&address.row),
        GridAxis::Column => (dependency.first..=dependency.last).contains(&address.col),
    }
}

/// Whether `dependency`'s axis span fully contains `rect` (both row-bound
/// edges within the span for a Row dependency, or both column-bound edges
/// within the span for a Column dependency). Used by
/// `grid_dependency_covers`'s `(AxisValue, Range)` arm: a whole-axis
/// dependency only "covers" a range candidate when the range's entire span
/// along that axis is inside the dependency's span, mirroring how
/// `grid_rect_contains_rect` requires full containment rather than mere
/// overlap.
#[must_use]
pub(super) fn axis_value_dependency_contains_rect(
    dependency: &GridAxisValueDependency,
    rect: &GridRect,
) -> bool {
    match dependency.axis {
        GridAxis::Row => dependency.first <= rect.top_row && rect.bottom_row <= dependency.last,
        GridAxis::Column => dependency.first <= rect.left_col && rect.right_col <= dependency.last,
    }
}

#[cfg(test)]
mod sheet_span_dependency_tests {
    use super::*;

    #[test]
    fn sheet_span_dependency_seeds_no_dirty_target_until_r4_12() {
        // The 3D span edge (W062 D2 §4.2 / R3.9) contributes no dirty seed on
        // its own: closure-time expansion against sheet order — and thus its
        // per-sheet dirtying — is R4.12. Until then it behaves like
        // ReferenceMetadata (None), never a silently-wrong seed.
        let dependency = GridDependency::SheetSpan(GridSheetSpanDependency::new(
            "book:default",
            "sheet-node:1",
            "sheet-node:3",
            "A1",
        ));
        assert_eq!(grid_dirty_seed_for_dependency(&dependency), None);
    }

    #[test]
    fn sheet_span_dependency_preserves_authored_endpoints_and_target() {
        let dependency =
            GridSheetSpanDependency::new("book:default", "sheet-node:1", "sheet-node:3", "A1");
        assert_eq!(dependency.workbook_id, "book:default");
        assert_eq!(dependency.start_sheet, "sheet-node:1");
        assert_eq!(dependency.end_sheet, "sheet-node:3");
        assert_eq!(dependency.target, "A1");
        // The edge is comparable (BTreeSet-storable) — ordering is total.
        let mut set = BTreeSet::new();
        set.insert(GridDependency::SheetSpan(dependency.clone()));
        assert!(set.contains(&GridDependency::SheetSpan(dependency)));
    }
}
