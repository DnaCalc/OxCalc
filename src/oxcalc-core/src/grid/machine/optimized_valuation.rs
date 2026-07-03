//! The optimized engine's computed valuation: the sparse/dense
//! computed-value layer over a sheet, defined-name and table-overlay shape
//! state, the spill ledger, and the readout plus shape-resolver construction
//! the optimized provider and recalc build on. Members are pub(super) - the
//! provider and recalc paths read its fields and call its helpers directly.
//! Internal to the machine; shares the machine's types via `use super::*`.

use super::*;

/// How much of the sheet a `GridOptimizedValuation` actually reflects.
/// `Full` valuations (mark-all, dirty recalc) cover every final authored
/// cell and are safe to feed back into dirty recalc or warm-no-op reuse.
/// `VisibleProjection` valuations (the visible-first fast path) only
/// evaluated an upstream cone and must never be mistaken for a complete
/// recalc: dirty recalc must reject or escalate them, and warm-no-op must
/// never treat them as a reusable cache hit.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum GridOptimizedValuationCoverage {
    #[default]
    Full,
    VisibleProjection {
        upstream_rect: GridRect,
    },
}

impl GridOptimizedValuationCoverage {
    #[must_use]
    pub const fn is_full(&self) -> bool {
        matches!(self, Self::Full)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridOptimizedValuation {
    pub(super) workbook_id: String,
    pub(super) sheet_id: String,
    pub(super) bounds: ExcelGridBounds,
    pub(super) sparse: SparsePointMap,
    pub(super) dense_value_regions: Vec<GridComputedDenseValueRegion>,
    pub(super) spill_facts: BTreeMap<ExcelGridCellAddress, GridSpillFact>,
    pub(super) spill_value_fingerprints: BTreeMap<ExcelGridCellAddress, String>,
    pub(super) spill_epoch_ledger: GridSpillEpochLedger,
    pub(super) defined_names: BTreeMap<String, GridRect>,
    pub(super) dynamic_defined_name_keys: BTreeSet<String>,
    pub(super) dynamic_defined_name_extents: BTreeMap<String, GridRect>,
    pub(super) dynamic_defined_name_dependencies: GridDynamicDefinedNameDependencyState,
    pub(super) volatile_dynamic_defined_names: BTreeSet<String>,
    pub(super) external_pending_dynamic_defined_names: BTreeSet<String>,
    pub(super) table_overlays: BTreeMap<String, GridTableOverlay>,
    pub(super) runtime_dependencies: GridInvalidationRef,
    pub(super) coverage: GridOptimizedValuationCoverage,
    /// True once a mark-all pass has committed a runtime dependency graph
    /// consistent with this valuation's authored state. Mirrors
    /// `GridCalcRefSheet::graph_installed` (see calc_ref_sheet.rs): a sheet
    /// whose formulas legitimately have no dependencies (`=1+1`, `=NOW()`)
    /// has a correct, zero-edge graph after mark-all, so
    /// `recalculate_dirty_compact_with_oxfml`'s escalation check must not
    /// infer "graph never built" purely from a zero edge count. Defaults to
    /// `false` for freshly constructed valuations; only
    /// `recalculate_mark_all_dirty_compact_with_oxfml` sets it `true`.
    pub(super) graph_installed: bool,
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
            dynamic_defined_name_keys: BTreeSet::new(),
            dynamic_defined_name_extents: BTreeMap::new(),
            dynamic_defined_name_dependencies: GridDynamicDefinedNameDependencyState::default(),
            volatile_dynamic_defined_names: BTreeSet::new(),
            external_pending_dynamic_defined_names: BTreeSet::new(),
            table_overlays: BTreeMap::new(),
            runtime_dependencies: GridInvalidationRef::new(bounds),
            coverage: GridOptimizedValuationCoverage::default(),
            graph_installed: false,
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
            dynamic_defined_name_keys: BTreeSet::new(),
            dynamic_defined_name_extents: BTreeMap::new(),
            dynamic_defined_name_dependencies: GridDynamicDefinedNameDependencyState::default(),
            volatile_dynamic_defined_names: BTreeSet::new(),
            external_pending_dynamic_defined_names: BTreeSet::new(),
            table_overlays: BTreeMap::new(),
            runtime_dependencies: GridInvalidationRef::new(bounds),
            coverage: GridOptimizedValuationCoverage::default(),
            graph_installed: false,
        }
    }

    #[must_use]
    pub fn with_defined_names(mut self, defined_names: BTreeMap<String, GridRect>) -> Self {
        self.defined_names = defined_names;
        self
    }

    #[must_use]
    pub fn with_dynamic_defined_name_extents(
        mut self,
        dynamic_defined_name_extents: BTreeMap<String, GridRect>,
    ) -> Self {
        self.dynamic_defined_name_keys
            .extend(dynamic_defined_name_extents.keys().cloned());
        self.dynamic_defined_name_extents = dynamic_defined_name_extents;
        self
    }

    #[must_use]
    pub fn with_dynamic_defined_name_keys(
        mut self,
        dynamic_defined_name_keys: BTreeSet<String>,
    ) -> Self {
        self.dynamic_defined_name_keys = dynamic_defined_name_keys;
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
    pub fn runtime_dependency_graph(&self) -> &GridInvalidationRef {
        &self.runtime_dependencies
    }

    #[cfg(test)]
    pub(super) fn runtime_dependency_graph_mut_for_test(&mut self) -> &mut GridInvalidationRef {
        &mut self.runtime_dependencies
    }

    #[must_use]
    pub const fn coverage(&self) -> &GridOptimizedValuationCoverage {
        &self.coverage
    }

    #[must_use]
    pub fn is_full_coverage(&self) -> bool {
        self.coverage.is_full()
    }

    pub(super) fn set_visible_projection_coverage(&mut self, upstream_rect: GridRect) {
        self.coverage = GridOptimizedValuationCoverage::VisibleProjection { upstream_rect };
    }

    #[must_use]
    pub fn dynamic_defined_name_extents(&self) -> &BTreeMap<String, GridRect> {
        &self.dynamic_defined_name_extents
    }

    #[must_use]
    pub fn dynamic_defined_name_dependencies(&self) -> &GridDynamicDefinedNameDependencyState {
        &self.dynamic_defined_name_dependencies
    }

    #[must_use]
    pub fn dynamic_defined_name_keys(&self) -> &BTreeSet<String> {
        &self.dynamic_defined_name_keys
    }

    #[must_use]
    pub fn volatile_dynamic_defined_names(&self) -> &BTreeSet<String> {
        &self.volatile_dynamic_defined_names
    }

    #[must_use]
    pub fn external_pending_dynamic_defined_names(&self) -> &BTreeSet<String> {
        &self.external_pending_dynamic_defined_names
    }

    pub fn set_dynamic_defined_name_external_pending(
        &mut self,
        name: impl AsRef<str>,
        external_pending: bool,
    ) -> Result<bool, GridRefError> {
        let name = name.as_ref();
        let name_key = defined_name_key_for_name_or_key(name, self.bounds)?;
        if !self.dynamic_defined_name_keys.contains(&name_key)
            && !self.dynamic_defined_name_extents.contains_key(&name_key)
            && !self
                .external_pending_dynamic_defined_names
                .contains(&name_key)
        {
            return Err(GridRefError::DefinedNameNotFound {
                name: name.to_string(),
            });
        }
        Ok(if external_pending {
            self.external_pending_dynamic_defined_names.insert(name_key)
        } else {
            self.external_pending_dynamic_defined_names
                .remove(&name_key)
        })
    }

    pub fn set_external_pending_root(
        &mut self,
        address: ExcelGridCellAddress,
        external_pending: bool,
    ) -> Result<bool, GridRefError> {
        self.check_address(&address)?;
        self.runtime_dependencies
            .set_external_pending_root(address, external_pending)
    }

    #[must_use]
    pub fn external_availability_dirty_seeds(&self) -> BTreeSet<GridDirtySeed> {
        let mut seeds = BTreeSet::new();
        if self.runtime_dependencies.has_external_pending_roots() {
            seeds.insert(GridDirtySeed::External);
        }
        seeds.extend(
            self.external_pending_dynamic_defined_names
                .iter()
                .cloned()
                .map(GridDirtySeed::Name),
        );
        seeds
    }

    pub fn external_availability_event_report(
        &self,
    ) -> Result<GridExternalAvailabilityEventReport, GridRefError> {
        let dirty_closure = self
            .runtime_dependencies
            .dirty_closure_for_seeds(self.external_availability_dirty_seeds())?;
        Ok(GridExternalAvailabilityEventReport {
            pending_formula_roots: self.runtime_dependencies.external_pending_roots().clone(),
            pending_dynamic_defined_names: self.external_pending_dynamic_defined_names.clone(),
            dirty_closure,
        })
    }

    pub fn external_availability_topic_event_report(
        &self,
        registry: &GridExternalAvailabilityTopicRegistry,
        topic_id: impl AsRef<str>,
        topic_sequence: u64,
    ) -> Result<GridExternalAvailabilityTopicEventReport, GridRefError> {
        let pending = self.external_availability_event_report()?;
        registry.external_availability_topic_event_report(
            topic_id,
            topic_sequence,
            &pending,
            &self.runtime_dependencies,
        )
    }

    pub fn dispatch_external_availability_topic_updates(
        &self,
        registry: &mut GridExternalAvailabilityTopicRegistry,
        updates: impl IntoIterator<Item = GridExternalAvailabilityTopicEnvelopeUpdate>,
    ) -> Result<GridExternalAvailabilityTopicDispatchReport, GridRefError> {
        let pending = self.external_availability_event_report()?;
        registry.dispatch_external_availability_topic_updates(
            updates,
            &pending,
            &self.runtime_dependencies,
        )
    }

    pub fn apply_axis_edit(
        &mut self,
        edit: GridAxisEdit,
    ) -> Result<GridOptimizedValuationAxisEditReport, GridRefError> {
        validate_axis_edit(edit, self.bounds)?;

        let old_sparse = std::mem::take(&mut self.sparse);
        let mut sparse = SparsePointMap::default();
        let mut sparse_computed_cells_kept = 0;
        let mut sparse_computed_cells_dropped = 0;
        for (address, cell) in old_sparse.iter() {
            if let Some(transformed) = transform_address_for_edit(address, edit, self.bounds)? {
                sparse.upsert(transformed, cell.clone());
                sparse_computed_cells_kept += 1;
            } else {
                sparse_computed_cells_dropped += 1;
            }
        }
        self.sparse = sparse;

        let dense_value_regions_before = self.dense_value_regions.len();
        let dense_value_cells_before = self.dense_computed_cells();
        let mut dense_value_regions = Vec::new();
        let mut dense_value_regions_dropped = 0;
        for region in std::mem::take(&mut self.dense_value_regions) {
            let transformed =
                transform_computed_dense_value_region_for_edit(&region, edit, self.bounds)?;
            if transformed.is_empty() {
                dense_value_regions_dropped += 1;
            }
            dense_value_regions.extend(transformed);
        }
        self.dense_value_regions = dense_value_regions;

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
            self.spill_facts.insert(
                anchor.clone(),
                GridSpillFact {
                    anchor,
                    extent,
                    blocked: fact.blocked,
                },
            );
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

        // Dynamic-name realized extents and the namespace-side dependency
        // ledger are calc-time realization state, not authored references:
        // per the axis-edit rule (structural transforms, calc-overlay
        // clears), they are cleared here and rebuilt by the next dynamic-name
        // refresh, instead of being shifted to follow the edit the way an
        // authored reference would. `dynamic_defined_name_keys` (namespace
        // identity) is untouched: the name still exists, it just has no
        // realized extent until refresh runs.
        self.dynamic_defined_name_extents.clear();
        self.dynamic_defined_name_dependencies.clear();

        let old_table_overlays = std::mem::take(&mut self.table_overlays);
        for (table_key, table) in old_table_overlays {
            let Some(table) = table.transform_for_axis_edit(edit, self.bounds)? else {
                continue;
            };
            self.table_overlays.insert(table_key, table);
        }

        let runtime_dependency_report = self.runtime_dependencies.apply_axis_edit(edit)?;
        for fact in self.spill_facts.values() {
            self.runtime_dependencies
                .refresh_overlay_spill_blocker_dependency(
                    fact.anchor.clone(),
                    Some(fact.extent.clone()),
                )?;
        }

        Ok(GridOptimizedValuationAxisEditReport {
            edit,
            sparse_computed_cells_kept,
            sparse_computed_cells_dropped,
            dense_value_regions_before,
            dense_value_regions_after: self.dense_value_regions.len(),
            dense_value_regions_dropped,
            dense_value_cells_before,
            dense_value_cells_after: self.dense_computed_cells(),
            spill_facts_kept,
            spill_facts_dropped,
            runtime_dependency_report,
        })
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

    pub(super) fn insert_sparse_value(
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

    pub(super) fn clear_formula_output_for_anchor(
        &mut self,
        anchor: &ExcelGridCellAddress,
    ) -> GridOptimizedSpillClearReport {
        let sparse_values_before = self.sparse.len();
        if let Some(fact) = self.spill_facts.remove(anchor) {
            self.spill_value_fingerprints.remove(anchor);
            let keys = grid_formula_output_cells_for_fact(&fact)
                .into_iter()
                .filter(|address| self.sparse.contains_key(address))
                .collect::<BTreeSet<_>>();
            let sparse_values_removed = keys.len();
            let old_extent_cell_count = fact.extent.cell_count();
            for key in keys {
                self.remove_sparse_value(&key);
            }
            let clear_rect = if fact.blocked {
                anchor_cell_rect(anchor, self.bounds)
            } else {
                fact.extent.clone()
            };
            let (dense_value_regions_removed, dense_value_cells_removed) =
                self.remove_dense_value_regions_in_grid_rect(&clear_rect);
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

    pub(super) fn remove_dense_value_regions_in_grid_rect(
        &mut self,
        rect: &GridRect,
    ) -> (usize, u64) {
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

    pub(super) fn sparse_addresses_in_rect(&self, rect: &GridRect) -> Vec<ExcelGridCellAddress> {
        if rect.workbook_id != self.workbook_id || rect.sheet_id != self.sheet_id {
            return Vec::new();
        }
        self.sparse.addresses_in_rect(rect)
    }

    pub(super) fn remove_sparse_value(
        &mut self,
        address: &ExcelGridCellAddress,
    ) -> Option<GridVersionedComputedCell> {
        self.sparse.remove(address)
    }

    pub(super) fn push_dense_value_region(
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

    pub(super) fn push_dense_value_payload(
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

    pub(super) fn contains_address(&self, address: &ExcelGridCellAddress) -> bool {
        address.workbook_id == self.workbook_id
            && address.sheet_id == self.sheet_id
            && self.bounds.contains_row(address.row)
            && self.bounds.contains_col(address.col)
    }

    pub(super) fn check_address(&self, address: &ExcelGridCellAddress) -> Result<(), GridRefError> {
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

    pub(super) fn check_rect(&self, rect: &GridRect) -> Result<(), GridRefError> {
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

fn transform_computed_dense_value_region_for_edit(
    region: &GridComputedDenseValueRegion,
    edit: GridAxisEdit,
    bounds: ExcelGridBounds,
) -> Result<Vec<GridComputedDenseValueRegion>, GridRefError> {
    let mut transformed = Vec::new();
    for (old_rect, new_rect) in rect_segments_for_axis_edit(&region.rect, edit, bounds)? {
        let storage = region.storage.slice_for_subrect(&region.rect, &old_rect);
        transformed.push(GridComputedDenseValueRegion {
            rect: new_rect,
            storage,
            revision: region.revision,
            source: region.source,
        });
    }
    Ok(transformed)
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
    pub(super) fn shape_resolver(
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
            shape_provider = shape_provider.with_defined_name_key(name.clone(), rect.clone());
        }
        for (name, rect) in &self.dynamic_defined_name_extents {
            shape_provider = shape_provider.with_defined_name_key(name.clone(), rect.clone());
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
