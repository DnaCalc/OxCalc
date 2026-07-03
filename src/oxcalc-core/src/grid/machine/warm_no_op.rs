//! The optimized engine's warm no-op cache: a seed-scale snapshot token
//! over compact authored state, axis state, spill facts, names, tables, and
//! the materialization limit, plus the report it produces. An unchanged
//! sheet reuses its cached valuation with zero cells visited (P-19). The
//! recalc constructs these snapshot tokens, so members are pub(super).
//! Internal to the machine; shares the machine's types via `use super::*`.

use super::*;

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

pub(super) const TILE_SNAPSHOT_FRAME_HEADER_BYTES: u64 = 128;
pub(super) const TILE_SNAPSHOT_CELL_ENTRY_BYTES: u64 = 16;

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
    pub(super) token: GridOptimizedWarmNoOpToken,
    pub(super) valuation: GridOptimizedValuation,
    pub(super) baseline_report: GridOptimizedRecalcReport,
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

    /// Mark (or clear) a formula root as externally pending on the cached valuation. Hosts
    /// that learn about pending external availability after a warm-no-op cache was captured
    /// (for example, an RTD provider marking a root pending mid-session) use this to make the
    /// pending state reachable through the public API — `recalculate_warm_noop_compact_with_oxfml`
    /// then correctly refuses to reuse the cache instead of silently returning a stale value.
    pub fn mark_external_pending_root(
        &mut self,
        address: ExcelGridCellAddress,
        external_pending: bool,
    ) -> Result<bool, GridRefError> {
        self.valuation
            .set_external_pending_root(address, external_pending)
    }

    /// Mark (or clear) a dynamic defined name as externally pending on the cached valuation.
    /// See [`Self::mark_external_pending_root`] for why hosts need this public surface.
    pub fn mark_dynamic_defined_name_external_pending(
        &mut self,
        name: impl AsRef<str>,
        external_pending: bool,
    ) -> Result<bool, GridRefError> {
        self.valuation
            .set_dynamic_defined_name_external_pending(name, external_pending)
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
pub(super) struct GridOptimizedWarmNoOpToken {
    pub(super) materialization_limit: u64,
    pub(super) next_revision: u64,
    pub(super) axis_state: GridAxisState,
    pub(super) sparse_points: Vec<GridOptimizedSparsePointToken>,
    pub(super) dense_value_regions: Vec<GridOptimizedDenseValueRegionToken>,
    pub(super) repeated_formula_regions: Vec<GridOptimizedRepeatedFormulaRegionToken>,
    pub(super) merged_regions: Vec<GridMergedRegion>,
    pub(super) feature_rendered_regions: Vec<FeatureRenderedRegion>,
    pub(super) spill_facts: Vec<(ExcelGridCellAddress, GridSpillFact)>,
    pub(super) defined_names: Vec<(String, GridRect)>,
    pub(super) dynamic_defined_names: Vec<(String, GridDynamicDefinedName)>,
    pub(super) dynamic_defined_name_extents: Vec<(String, GridRect)>,
    pub(super) table_overlays: Vec<(String, GridTableOverlay)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GridOptimizedSparsePointToken {
    pub(super) coord: GridCellCoord,
    pub(super) revision: u64,
    pub(super) authored: GridOptimizedAuthoredCellToken,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum GridOptimizedAuthoredCellToken {
    Literal,
    Formula {
        source_text: String,
        normal_form_key: String,
        source_channel: FormulaChannelKind,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GridOptimizedDenseValueRegionToken {
    pub(super) rect: GridRect,
    pub(super) revision: u64,
    pub(super) value_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GridOptimizedRepeatedFormulaRegionToken {
    pub(super) rect: GridRect,
    pub(super) revision: u64,
    pub(super) source_text: String,
    pub(super) normal_form_key: String,
    pub(super) source_channel: FormulaChannelKind,
}
