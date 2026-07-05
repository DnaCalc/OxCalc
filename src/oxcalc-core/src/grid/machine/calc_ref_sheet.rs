//! GridCalc-Ref: the simple-correct reference sheet oracle for the
//! strict-excel-grid profile. Plain BTreeMap authored/computed state with
//! mark-all-dirty recalc, serving as the value and committed-effects oracle
//! the optimized engine is differentially checked against. Internal to the
//! machine; shares the machine's types via `use super::*`.

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct GridCalcRefSheet {
    pub(super) workbook_id: String,
    pub(super) sheet_id: String,
    pub(super) bounds: ExcelGridBounds,
    pub(super) authored: BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
    pub(super) computed: BTreeMap<ExcelGridCellAddress, CalcValue>,
    pub(super) axis_state: GridAxisState,
    pub(super) spill_value_fingerprints: BTreeMap<ExcelGridCellAddress, String>,
    pub(super) spill_epoch_ledger: GridSpillEpochLedger,
    pub(super) defined_names: BTreeMap<String, GridRect>,
    pub(super) dynamic_defined_names: BTreeMap<String, GridDynamicDefinedName>,
    pub(super) dynamic_defined_name_extents: BTreeMap<String, GridRect>,
    pub(super) dynamic_defined_name_dependencies: GridDynamicDefinedNameDependencyState,
    pub(super) volatile_dynamic_defined_names: BTreeSet<String>,
    pub(super) external_pending_dynamic_defined_names: BTreeSet<String>,
    pub(super) overlays: GridOverlaySet,
    /// Cross-sheet computed values the workbook catalog router resolved for
    /// this sheet's formulas (W062 D2 §4.1, R3.3). Keyed by full
    /// [`ExcelGridCellAddress`] on *other* sheets; a formula here that
    /// references `Sheet2!A1` reads Sheet2's committed value through this map
    /// when the reference-system provider is built. This is a **read-only
    /// injected view**, never authored or computed on this sheet
    /// (`check_address` still rejects foreign addresses for authored/computed
    /// state) and never persisted into a valuation. It carries resolved values
    /// only; cross-sheet *dirty propagation* — re-running this sheet when a
    /// referenced sheet's cell changes — is explicitly pending on the D3
    /// workbook coordination layer (R4.6), which owns the reverse edge. The
    /// consumer refreshes this before each recalc from the peer sheets'
    /// committed readouts.
    pub(super) cross_sheet_cells: BTreeMap<ExcelGridCellAddress, CalcValue>,
    pub(super) runtime_dependencies: GridInvalidationRef,
    /// Set when a dirty recalc pass publishes at least one value into
    /// `computed`/`overlays.spill_facts` in place and then fails partway
    /// through the worklist (effective cycle, convergence-limit,
    /// evaluation, or graph error). `runtime_dependencies` is staged into a
    /// local clone and only committed to `self` on success, so a mid-pass
    /// error leaves already-published values inconsistent with the still-old
    /// committed dependency graph: the next dirty recalc's incremental
    /// closure would miss cells that the failed pass touched. When this flag
    /// is set, the next `recalculate_dirty_with_oxfml` call forces a full
    /// `recalculate_mark_all_dirty_with_oxfml` instead of an incremental
    /// pass, exactly like `!self.graph_installed` below.
    pub(super) graph_needs_full_rebuild: bool,
    /// True once a `recalculate_mark_all_dirty_with_oxfml` pass has
    /// committed a runtime dependency graph consistent with `authored`.
    /// A freshly constructed sheet (or one whose graph was never built)
    /// has this `false` and must escalate to mark-all regardless of
    /// `formula_cells`/edge counts: a sheet whose formulas legitimately
    /// have no dependencies (`=1+1`, `=NOW()`) has `graph_edge_count == 0`
    /// even after a correct mark-all, so edge-count emptiness alone cannot
    /// distinguish "never built" from "built with no edges". Only
    /// `recalculate_mark_all_dirty_with_oxfml` sets this `true`; nothing
    /// resets it back to `false` except `graph_needs_full_rebuild` forcing
    /// the next call back through mark-all (which then re-sets it `true`).
    pub(super) graph_installed: bool,
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
            spill_value_fingerprints: BTreeMap::new(),
            spill_epoch_ledger: GridSpillEpochLedger::default(),
            defined_names: BTreeMap::new(),
            dynamic_defined_names: BTreeMap::new(),
            dynamic_defined_name_extents: BTreeMap::new(),
            dynamic_defined_name_dependencies: GridDynamicDefinedNameDependencyState::default(),
            volatile_dynamic_defined_names: BTreeSet::new(),
            external_pending_dynamic_defined_names: BTreeSet::new(),
            overlays: GridOverlaySet::default(),
            cross_sheet_cells: BTreeMap::new(),
            runtime_dependencies: GridInvalidationRef::new(bounds),
            graph_needs_full_rebuild: false,
            graph_installed: false,
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

    /// Replace this sheet's cross-sheet resolved-value view (W062 D2 §4.1,
    /// R3.3). The consumer's workbook coordinator calls this before a recalc,
    /// feeding the committed computed values of every *other* sheet a formula
    /// on this sheet references, keyed by full cross-sheet address. Entries for
    /// this sheet's own `(workbook_id, sheet_id)` are ignored — own-sheet
    /// resolution always reads live `computed` state — so the caller cannot
    /// accidentally shadow authored truth. Passing an empty iterator clears the
    /// view (the single-sheet default).
    pub fn set_cross_sheet_cells(
        &mut self,
        cells: impl IntoIterator<Item = (ExcelGridCellAddress, CalcValue)>,
    ) {
        self.cross_sheet_cells = cells
            .into_iter()
            .filter(|(address, _)| {
                address.workbook_id != self.workbook_id || address.sheet_id != self.sheet_id
            })
            .collect();
    }

    /// The cross-sheet resolved-value view currently injected (R3.3), for the
    /// consumer coordinator and tests to inspect.
    #[must_use]
    pub fn cross_sheet_cells(&self) -> &BTreeMap<ExcelGridCellAddress, CalcValue> {
        &self.cross_sheet_cells
    }

    /// The union of the static structural dependencies of every authored
    /// formula on this sheet (W062 D3 §5, additive seam for R4.5).
    ///
    /// This is the exact same per-formula extraction the recalc worklist and
    /// the R3.3 two-sheet tests already use
    /// ([`grid_structural_dependencies_for_formula`]), gathered over the whole
    /// authored map. The workbook oracle ([`GridCalcRefWorkbook`]) routes this
    /// set through the [`WorkbookReferenceCatalog`] to discover which *other*
    /// sheets' cells it must gather into this sheet's cross-sheet view before a
    /// recalc round — the same route the single-sheet R3.3 test performs by
    /// hand, lifted to "every formula on the sheet".
    ///
    /// It is a **read** over authored state and installs nothing; the returned
    /// dependencies still carry full sheet identity on their addresses, so the
    /// catalog partitions same-sheet from cross-sheet targets. Cross-sheet
    /// resolved values seeded from a prior round feed the reference provider,
    /// so a `Sheet2!A1` reference extracts as a cross-sheet `Cell` dependency
    /// regardless of whether that value is currently known.
    #[must_use]
    pub fn authored_formula_structural_dependencies(&self) -> BTreeSet<GridDependency> {
        let profile = StrictExcelGridReferenceProfile::with_bounds(self.bounds);
        let mut dependencies = BTreeSet::new();
        for (address, cell) in &self.authored {
            let GridAuthoredCell::Formula(formula) = cell else {
                continue;
            };
            let provider = self.reference_system_provider(address.row, address.col);
            dependencies.extend(grid_structural_dependencies_for_formula(
                formula,
                address,
                &profile,
                self.bounds,
                &provider,
            ));
        }
        dependencies
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

    pub fn clear_cell(
        &mut self,
        address: &ExcelGridCellAddress,
    ) -> Result<GridCellClearReport, GridRefError> {
        self.check_address(address)?;
        self.authored.remove(address);
        let old_fact = self.clear_formula_output_for_anchor(address);
        self.computed.remove(address);
        self.refresh_spill_epoch_ledger();
        self.runtime_dependencies
            .set_structural_dependencies(address.clone(), Vec::new())?;
        self.runtime_dependencies
            .clear_overlay_dependencies(address)?;

        let mut dirty_seeds = BTreeSet::new();
        dirty_seeds.insert(GridDirtySeed::Cell(address.clone()));
        let mut vacated_extent_cells = BTreeSet::new();
        if let Some(fact) = &old_fact {
            vacated_extent_cells = grid_formula_output_cells_for_fact(fact)
                .into_iter()
                .filter(|cell| cell != address)
                .collect();
            dirty_seeds.extend(grid_vacated_spill_extent_dirty_seeds(address, &fact.extent));
        }

        Ok(GridCellClearReport {
            address: address.clone(),
            had_spill_fact: old_fact.is_some(),
            vacated_extent_cells,
            dirty_seeds,
        })
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
        &self.overlays.merged_regions
    }

    pub fn add_merged_region(&mut self, rect: GridRect) {
        self.overlays.merged_regions.push(GridMergedRegion { rect });
    }

    #[must_use]
    pub fn feature_rendered_regions(&self) -> &[FeatureRenderedRegion] {
        &self.overlays.feature_rendered_regions
    }

    pub fn add_feature_rendered_region(
        &mut self,
        rect: GridRect,
        feature_kind: impl Into<String>,
        needs_refresh: bool,
    ) {
        self.overlays
            .feature_rendered_regions
            .push(FeatureRenderedRegion {
                rect,
                feature_kind: feature_kind.into(),
                needs_refresh,
            });
    }

    #[must_use]
    pub fn spill_facts(&self) -> &BTreeMap<ExcelGridCellAddress, GridSpillFact> {
        &self.overlays.spill_facts
    }

    #[must_use]
    pub fn spill_epoch_ledger(&self) -> &GridSpillEpochLedger {
        &self.spill_epoch_ledger
    }

    #[must_use]
    pub fn runtime_dependency_graph(&self) -> &GridInvalidationRef {
        &self.runtime_dependencies
    }

    #[cfg(test)]
    pub(super) fn runtime_dependency_graph_mut_for_test(&mut self) -> &mut GridInvalidationRef {
        &mut self.runtime_dependencies
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
        self.overlays.spill_facts.insert(fact.anchor.clone(), fact);
        self.refresh_spill_epoch_ledger();
        Ok(())
    }

    pub fn refresh_spill_epoch_ledger(&mut self) -> GridSpillEpochLedgerUpdateReport {
        let fingerprints = self.spill_value_fingerprints.clone();
        self.spill_epoch_ledger
            .update_from_spill_facts(&self.overlays.spill_facts, |fact| {
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

    #[must_use]
    pub fn dynamic_defined_names(&self) -> &BTreeMap<String, GridDynamicDefinedName> {
        &self.dynamic_defined_names
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
        if !self.dynamic_defined_names.contains_key(&name_key)
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

    pub fn set_defined_name(
        &mut self,
        name: impl AsRef<str>,
        rect: GridRect,
    ) -> Result<GridNameLifecycleReport, GridRefError> {
        rect.check_sheet(self)?;
        let name_key = defined_name_key_for_name(name.as_ref(), self.bounds)?;
        let removed_dynamic_name = self.dynamic_defined_names.remove(&name_key).is_some();
        let removed_dynamic_extent = self
            .dynamic_defined_name_extents
            .remove(&name_key)
            .is_some();
        let removed_dynamic_dependencies = self.dynamic_defined_name_dependencies.remove(&name_key);
        let removed_dynamic_volatile = self.volatile_dynamic_defined_names.remove(&name_key);
        let removed_dynamic_external = self
            .external_pending_dynamic_defined_names
            .remove(&name_key);
        let replaced_static = self.defined_names.insert(name_key.clone(), rect).is_some();
        let operation = if replaced_static
            || removed_dynamic_name
            || removed_dynamic_extent
            || removed_dynamic_dependencies
            || removed_dynamic_volatile
            || removed_dynamic_external
        {
            GridNameLifecycleOperation::Redefine
        } else {
            GridNameLifecycleOperation::Create
        };
        Ok(GridNameLifecycleReport {
            operation,
            old_name_key: (operation == GridNameLifecycleOperation::Redefine)
                .then(|| name_key.clone()),
            new_name_key: Some(name_key.clone()),
            dirty_seeds: grid_name_lifecycle_dirty_seeds([name_key]),
            formula_cells_transformed: 0,
            formula_reference_transforms: 0,
        })
    }

    pub fn set_sheet_defined_name(
        &mut self,
        sheet_id: impl AsRef<str>,
        name: impl AsRef<str>,
        rect: GridRect,
    ) -> Result<GridNameLifecycleReport, GridRefError> {
        rect.check_sheet(self)?;
        let sheet_id = sheet_id.as_ref();
        let name = name.as_ref();
        let name_key =
            sheet_defined_name_key_for_name(&self.workbook_id, sheet_id, name, self.bounds)?;
        let global_key = excel_grid_defined_name_key(name, self.bounds);
        let global_entry_exists = global_key.as_deref().is_some_and(|global_key| {
            self.defined_names.contains_key(global_key)
                || self.dynamic_defined_names.contains_key(global_key)
        });
        let removed_dynamic_name = self.dynamic_defined_names.remove(&name_key).is_some();
        let removed_dynamic_extent = self
            .dynamic_defined_name_extents
            .remove(&name_key)
            .is_some();
        let removed_dynamic_dependencies = self.dynamic_defined_name_dependencies.remove(&name_key);
        let removed_dynamic_volatile = self.volatile_dynamic_defined_names.remove(&name_key);
        let removed_dynamic_external = self
            .external_pending_dynamic_defined_names
            .remove(&name_key);
        let replaced_static = self.defined_names.insert(name_key.clone(), rect).is_some();
        let operation = if replaced_static
            || removed_dynamic_name
            || removed_dynamic_extent
            || removed_dynamic_dependencies
            || removed_dynamic_volatile
            || removed_dynamic_external
        {
            GridNameLifecycleOperation::Redefine
        } else {
            GridNameLifecycleOperation::Create
        };
        Ok(GridNameLifecycleReport {
            operation,
            old_name_key: (operation == GridNameLifecycleOperation::Redefine)
                .then(|| name_key.clone()),
            new_name_key: Some(name_key.clone()),
            dirty_seeds: grid_name_lifecycle_dirty_seeds(grid_scoped_name_lifecycle_keys(
                name_key,
                global_key.as_deref(),
                global_entry_exists,
            )),
            formula_cells_transformed: 0,
            formula_reference_transforms: 0,
        })
    }

    pub fn set_sheet_dynamic_defined_name(
        &mut self,
        sheet_id: impl AsRef<str>,
        name: impl AsRef<str>,
        formula: GridFormulaCell,
    ) -> Result<GridNameLifecycleReport, GridRefError> {
        let sheet_id = sheet_id.as_ref();
        let name = name.as_ref();
        let name_key =
            sheet_defined_name_key_for_name(&self.workbook_id, sheet_id, name, self.bounds)?;
        let global_key = excel_grid_defined_name_key(name, self.bounds);
        let global_entry_exists = global_key.as_deref().is_some_and(|global_key| {
            self.defined_names.contains_key(global_key)
                || self.dynamic_defined_names.contains_key(global_key)
        });
        let anchor = default_dynamic_defined_name_anchor(self.workbook_id.clone(), sheet_id);
        let removed_static = self.defined_names.remove(&name_key).is_some();
        let replaced_dynamic = self
            .dynamic_defined_names
            .insert(
                name_key.clone(),
                GridDynamicDefinedName::new(formula, anchor),
            )
            .is_some();
        let removed_dynamic_extent = self
            .dynamic_defined_name_extents
            .remove(&name_key)
            .is_some();
        self.dynamic_defined_name_dependencies.remove(&name_key);
        let removed_dynamic_volatile = self.volatile_dynamic_defined_names.remove(&name_key);
        let removed_dynamic_external = self
            .external_pending_dynamic_defined_names
            .remove(&name_key);
        let operation = if removed_static
            || replaced_dynamic
            || removed_dynamic_extent
            || removed_dynamic_volatile
            || removed_dynamic_external
        {
            GridNameLifecycleOperation::Redefine
        } else {
            GridNameLifecycleOperation::Create
        };
        Ok(GridNameLifecycleReport {
            operation,
            old_name_key: (operation == GridNameLifecycleOperation::Redefine)
                .then(|| name_key.clone()),
            new_name_key: Some(name_key.clone()),
            dirty_seeds: grid_name_lifecycle_dirty_seeds(grid_scoped_name_lifecycle_keys(
                name_key,
                global_key.as_deref(),
                global_entry_exists,
            )),
            formula_cells_transformed: 0,
            formula_reference_transforms: 0,
        })
    }

    pub fn set_dynamic_defined_name(
        &mut self,
        name: impl AsRef<str>,
        formula: GridFormulaCell,
    ) -> Result<GridNameLifecycleReport, GridRefError> {
        let name_key = defined_name_key_for_name(name.as_ref(), self.bounds)?;
        let anchor =
            default_dynamic_defined_name_anchor(self.workbook_id.clone(), self.sheet_id.clone());
        let removed_static = self.defined_names.remove(&name_key).is_some();
        let replaced_dynamic = self
            .dynamic_defined_names
            .insert(
                name_key.clone(),
                GridDynamicDefinedName::new(formula, anchor),
            )
            .is_some();
        let removed_dynamic_extent = self
            .dynamic_defined_name_extents
            .remove(&name_key)
            .is_some();
        self.dynamic_defined_name_dependencies.remove(&name_key);
        let removed_dynamic_volatile = self.volatile_dynamic_defined_names.remove(&name_key);
        let removed_dynamic_external = self
            .external_pending_dynamic_defined_names
            .remove(&name_key);
        let operation = if removed_static
            || replaced_dynamic
            || removed_dynamic_extent
            || removed_dynamic_volatile
            || removed_dynamic_external
        {
            GridNameLifecycleOperation::Redefine
        } else {
            GridNameLifecycleOperation::Create
        };
        Ok(GridNameLifecycleReport {
            operation,
            old_name_key: (operation == GridNameLifecycleOperation::Redefine)
                .then(|| name_key.clone()),
            new_name_key: Some(name_key.clone()),
            dirty_seeds: grid_name_lifecycle_dirty_seeds([name_key]),
            formula_cells_transformed: 0,
            formula_reference_transforms: 0,
        })
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
        if old_key != new_key
            && (self.defined_names.contains_key(&new_key)
                || self.dynamic_defined_names.contains_key(&new_key))
        {
            return Err(GridRefError::DefinedNameAlreadyExists {
                name: new_name.to_string(),
            });
        }
        // A sheet-scoped entry of the same text as the global name being
        // renamed shadows it: a formula that resolved `old_name` bound to
        // the scoped entry, not the global one, and must not be rewritten
        // to the new global text. See D3.
        let scoped_shadow_key = sheet_defined_name_key_for_name(
            &self.workbook_id,
            &self.sheet_id,
            old_name,
            self.bounds,
        )
        .ok();
        let shadowed_by_scope = scoped_shadow_key.as_deref().is_some_and(|scoped_key| {
            self.defined_names.contains_key(scoped_key)
                || self.dynamic_defined_names.contains_key(scoped_key)
        });
        let rect = self.defined_names.remove(&old_key);
        let dynamic_name = self.dynamic_defined_names.remove(&old_key);
        let dynamic_extent = self.dynamic_defined_name_extents.remove(&old_key);
        self.dynamic_defined_name_dependencies
            .rename(&old_key, new_key.clone());
        let was_volatile_dynamic = self.volatile_dynamic_defined_names.remove(&old_key);
        let was_external_pending_dynamic =
            self.external_pending_dynamic_defined_names.remove(&old_key);
        if rect.is_none()
            && dynamic_name.is_none()
            && dynamic_extent.is_none()
            && !was_volatile_dynamic
            && !was_external_pending_dynamic
        {
            return Err(GridRefError::DefinedNameNotFound {
                name: old_name.to_string(),
            });
        }
        if let Some(rect) = rect {
            self.defined_names.insert(new_key.clone(), rect);
        }
        if let Some(dynamic_name) = dynamic_name {
            self.dynamic_defined_names
                .insert(new_key.clone(), dynamic_name);
        }
        if let Some(dynamic_extent) = dynamic_extent {
            self.dynamic_defined_name_extents
                .insert(new_key.clone(), dynamic_extent);
        }
        if was_volatile_dynamic {
            self.volatile_dynamic_defined_names.insert(new_key.clone());
        }
        if was_external_pending_dynamic {
            self.external_pending_dynamic_defined_names
                .insert(new_key.clone());
        }
        let stats = transform_authored_formulas_for_defined_name_rename(
            &mut self.authored,
            &old_key,
            new_name,
            self.bounds,
            shadowed_by_scope,
        )?;
        let mut dirty_seed_keys = vec![old_key.clone(), new_key.clone()];
        if shadowed_by_scope && let Some(scoped_key) = scoped_shadow_key {
            dirty_seed_keys.push(scoped_key);
        }
        Ok(GridNameLifecycleReport {
            operation: GridNameLifecycleOperation::Rename,
            old_name_key: Some(old_key.clone()),
            new_name_key: Some(new_key.clone()),
            dirty_seeds: grid_name_lifecycle_dirty_seeds(dirty_seed_keys),
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
        // A sheet-scoped entry of the same text as the global name being
        // deleted shadows it: a formula that resolved `name` bound to the
        // scoped entry, not the global one, and must not be rewritten to
        // #NAME?. See D3.
        let scoped_shadow_key =
            sheet_defined_name_key_for_name(&self.workbook_id, &self.sheet_id, name, self.bounds)
                .ok();
        let shadowed_by_scope = scoped_shadow_key.as_deref().is_some_and(|scoped_key| {
            self.defined_names.contains_key(scoped_key)
                || self.dynamic_defined_names.contains_key(scoped_key)
        });
        let removed_static = self.defined_names.remove(&name_key).is_some();
        let removed_dynamic = self.dynamic_defined_names.remove(&name_key).is_some();
        let removed_dynamic_extent = self
            .dynamic_defined_name_extents
            .remove(&name_key)
            .is_some();
        let removed_dynamic_dependencies = self.dynamic_defined_name_dependencies.remove(&name_key);
        let removed_dynamic_volatile = self.volatile_dynamic_defined_names.remove(&name_key);
        let removed_dynamic_external = self
            .external_pending_dynamic_defined_names
            .remove(&name_key);
        if !removed_static
            && !removed_dynamic
            && !removed_dynamic_extent
            && !removed_dynamic_dependencies
            && !removed_dynamic_volatile
            && !removed_dynamic_external
        {
            return Err(GridRefError::DefinedNameNotFound {
                name: name.to_string(),
            });
        }
        // Deleting a name no longer rewrites authored formula text to a
        // literal "#NAME?" (see delete_sheet_defined_name below, the
        // existing scoped-delete precedent this mirrors). The formula's
        // GridDependency::NameIdentity edge stays intact and the name-delete
        // dirty seed below drives the consumer to re-resolve; resolution now
        // reports the correct #NAME? via ReferenceResolutionError::
        // UnresolvedName instead of textually mutating the author's source.
        let mut dirty_seed_keys = vec![name_key.clone()];
        if shadowed_by_scope && let Some(scoped_key) = scoped_shadow_key {
            dirty_seed_keys.push(scoped_key);
        }
        Ok(GridNameLifecycleReport {
            operation: GridNameLifecycleOperation::Delete,
            old_name_key: Some(name_key.clone()),
            new_name_key: None,
            dirty_seeds: grid_name_lifecycle_dirty_seeds(dirty_seed_keys),
            formula_cells_transformed: 0,
            formula_reference_transforms: 0,
        })
    }

    pub fn delete_sheet_defined_name(
        &mut self,
        sheet_id: impl AsRef<str>,
        name: impl AsRef<str>,
    ) -> Result<GridNameLifecycleReport, GridRefError> {
        let sheet_id = sheet_id.as_ref();
        let name = name.as_ref();
        let name_key =
            sheet_defined_name_key_for_name(&self.workbook_id, sheet_id, name, self.bounds)?;
        let global_key = excel_grid_defined_name_key(name, self.bounds);
        let removed_static = self.defined_names.remove(&name_key).is_some();
        let removed_dynamic = self.dynamic_defined_names.remove(&name_key).is_some();
        let removed_dynamic_extent = self
            .dynamic_defined_name_extents
            .remove(&name_key)
            .is_some();
        let removed_dynamic_dependencies = self.dynamic_defined_name_dependencies.remove(&name_key);
        let removed_dynamic_volatile = self.volatile_dynamic_defined_names.remove(&name_key);
        let removed_dynamic_external = self
            .external_pending_dynamic_defined_names
            .remove(&name_key);
        if !removed_static
            && !removed_dynamic
            && !removed_dynamic_extent
            && !removed_dynamic_dependencies
            && !removed_dynamic_volatile
            && !removed_dynamic_external
        {
            return Err(GridRefError::DefinedNameNotFound {
                name: name.to_string(),
            });
        }
        // The deleted scoped name may have been shadowing a same-text
        // global (or other-scope) entry; a consumer that was bound to the
        // global key before the scope was ever created must also be
        // dirtied so a stale global binding does not linger. See D2.
        let global_entry_exists = global_key.as_deref().is_some_and(|global_key| {
            self.defined_names.contains_key(global_key)
                || self.dynamic_defined_names.contains_key(global_key)
        });
        Ok(GridNameLifecycleReport {
            operation: GridNameLifecycleOperation::Delete,
            old_name_key: Some(name_key.clone()),
            new_name_key: None,
            dirty_seeds: grid_name_lifecycle_dirty_seeds(grid_scoped_name_lifecycle_keys(
                name_key,
                global_key.as_deref(),
                global_entry_exists,
            )),
            formula_cells_transformed: 0,
            formula_reference_transforms: 0,
        })
    }

    #[must_use]
    pub fn table_overlays(&self) -> &BTreeMap<String, GridTableOverlay> {
        &self.overlays.table_overlays
    }

    pub fn set_table_overlay(
        &mut self,
        table: GridTableOverlay,
    ) -> Result<GridTableLifecycleReport, GridRefError> {
        table.check_sheet(&self.workbook_id, &self.sheet_id, self.bounds)?;
        let table_key = table_key_for_name(&table.table_name, self.bounds)?;
        let table_name = table.table_name.clone();
        let table_range = table.table_range.clone();
        let mut dirty_extents = vec![table_range.clone()];
        let mut feature_regions_removed = 0;
        let mut replaced_existing = false;
        if let Some(old_table) = self.overlays.table_overlays.get(&table_key) {
            replaced_existing = true;
            dirty_extents.push(old_table.table_range.clone());
            feature_regions_removed = remove_table_overlay_feature_regions(
                &mut self.overlays.feature_rendered_regions,
                &old_table.table_range,
            );
        }
        self.overlays
            .table_overlays
            .insert(table_key.clone(), table);
        self.add_feature_rendered_region(table_range.clone(), "table-overlay", false);
        Ok(GridTableLifecycleReport {
            operation: GridTableLifecycleOperation::Set,
            old_table_key: replaced_existing.then_some(table_key.clone()),
            new_table_key: Some(table_key),
            dirty_seeds: grid_table_lifecycle_dirty_seeds([table_name], dirty_extents),
            feature_regions_removed,
            feature_regions_added: 1,
            formula_cells_transformed: 0,
            formula_reference_transforms: 0,
        })
    }

    pub fn resize_table_overlay(
        &mut self,
        table: GridTableOverlay,
    ) -> Result<GridTableLifecycleReport, GridRefError> {
        table.check_sheet(&self.workbook_id, &self.sheet_id, self.bounds)?;
        let table_key = table_key_for_name(&table.table_name, self.bounds)?;
        let Some(old_table) = self.overlays.table_overlays.get(&table_key) else {
            return Err(GridRefError::TableOverlayNotFound {
                name: table.table_name,
            });
        };
        let old_table_range = old_table.table_range.clone();
        let table_name = table.table_name.clone();
        let feature_regions_removed = remove_table_overlay_feature_regions(
            &mut self.overlays.feature_rendered_regions,
            &old_table.table_range,
        );
        let table_range = table.table_range.clone();
        self.overlays
            .table_overlays
            .insert(table_key.clone(), table);
        self.add_feature_rendered_region(table_range.clone(), "table-overlay", false);
        Ok(GridTableLifecycleReport {
            operation: GridTableLifecycleOperation::Resize,
            old_table_key: Some(table_key.clone()),
            new_table_key: Some(table_key),
            dirty_seeds: grid_table_lifecycle_dirty_seeds(
                [table_name],
                [old_table_range, table_range.clone()],
            ),
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
        if old_key != new_key && self.overlays.table_overlays.contains_key(&new_key) {
            return Err(GridRefError::TableOverlayAlreadyExists {
                name: new_name.to_string(),
            });
        }
        let Some(mut table) = self.overlays.table_overlays.remove(&old_key) else {
            return Err(GridRefError::TableOverlayNotFound {
                name: old_name.to_string(),
            });
        };
        let table_range = table.table_range.clone();
        table.table_name = new_name.to_string();
        self.overlays.table_overlays.insert(new_key.clone(), table);
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
            dirty_seeds: grid_table_lifecycle_dirty_seeds(
                [old_name.to_string(), new_name.to_string()],
                [table_range],
            ),
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
        let Some(table) = self.overlays.table_overlays.remove(&table_key) else {
            return Err(GridRefError::TableOverlayNotFound {
                name: table_name.to_string(),
            });
        };
        let table_range = table.table_range.clone();
        let feature_regions_removed = remove_table_overlay_feature_regions(
            &mut self.overlays.feature_rendered_regions,
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
            dirty_seeds: grid_table_lifecycle_dirty_seeds([table_name.to_string()], [table_range]),
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
            &self.overlays.feature_rendered_regions,
            edit,
            self.bounds,
        )?;

        let (
            authored,
            authored_cells_kept,
            authored_cells_dropped,
            mut formula_cells_transformed,
            mut formula_reference_transforms,
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

        let old_spills = std::mem::take(&mut self.overlays.spill_facts);
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
            self.overlays.spill_facts.insert(anchor, transformed);
            spill_facts_kept += 1;
        }
        self.spill_value_fingerprints = transform_spill_value_fingerprints_for_edit(
            std::mem::take(&mut self.spill_value_fingerprints),
            edit,
            self.bounds,
        )?;
        self.refresh_spill_epoch_ledger();
        self.runtime_dependencies.apply_axis_edit(edit)?;
        for fact in self.overlays.spill_facts.values() {
            self.runtime_dependencies
                .refresh_overlay_spill_blocker_dependency(
                    fact.anchor.clone(),
                    Some(fact.extent.clone()),
                )?;
        }

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
        // refresh (the empty-ledger / first-discovery branch in
        // `refresh_dynamic_defined_names_with_oxfml`), instead of being
        // shifted to follow the edit the way an authored reference would.
        self.dynamic_defined_name_extents.clear();
        self.dynamic_defined_name_dependencies.clear();

        // The authored dynamic-name formula (and its anchor) IS an authored
        // reference, so it transforms like any other formula cell: Excel
        // rewrites a name's refers-to references on row/column insert.
        let old_dynamic_defined_names = std::mem::take(&mut self.dynamic_defined_names);
        for (name_key, definition) in old_dynamic_defined_names {
            let GridDynamicDefinedName { formula, anchor } = definition;
            let Some(new_anchor) = transform_address_for_edit(&anchor, edit, self.bounds)? else {
                continue;
            };
            let (formula, stats) = transform_formula_cell_for_axis_edit(
                formula,
                &anchor,
                &new_anchor,
                edit,
                self.bounds,
            )?;
            formula_cells_transformed += stats.formula_cells_transformed;
            formula_reference_transforms += stats.formula_reference_transforms;
            self.dynamic_defined_names
                .insert(name_key, GridDynamicDefinedName::new(formula, new_anchor));
        }

        let old_table_overlays = std::mem::take(&mut self.overlays.table_overlays);
        for (table_key, table) in old_table_overlays {
            let Some(table) = table.transform_for_axis_edit(edit, self.bounds)? else {
                continue;
            };
            self.overlays.table_overlays.insert(table_key, table);
        }

        let old_merged_regions = std::mem::take(&mut self.overlays.merged_regions);
        let mut merged_regions_kept = 0;
        let mut merged_regions_dropped = 0;
        for region in old_merged_regions {
            let (Some(rect), _) = transform_rect_for_edit(&region.rect, edit, self.bounds)? else {
                merged_regions_dropped += 1;
                continue;
            };
            self.overlays.merged_regions.push(GridMergedRegion { rect });
            merged_regions_kept += 1;
        }

        self.overlays.feature_rendered_regions = feature_region_transform.regions;

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
            structural_dependency_edges: 0,
            overlay_dependency_edges: 0,
            dynamic_defined_name_evaluations: 0,
            external_subscription_updates: Vec::new(),
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
                    let publication_delta =
                        self.publish_formula_value(address.clone(), value, &authored);
                    let spill_counters = publication_delta.counters;
                    report.spill_facts_published += spill_counters.facts_published;
                    report.spill_facts_blocked += spill_counters.facts_blocked;
                    report.spill_ghost_cells_published += spill_counters.ghost_cells_published;
                }
            }
        }

        self.refresh_reference_report_spill_counters(&mut report, &authored);
        self.refresh_spill_epoch_ledger();
        self.runtime_dependencies = GridInvalidationRef::new(self.bounds);
        report
    }

    pub fn recalculate_mark_all_dirty_with_oxfml(
        &mut self,
    ) -> Result<GridCalcRefRecalcReport, GridRefError> {
        // B4 fix: mark this pass "not installed" BEFORE any publishing work
        // starts, not just on success at the end. This function publishes
        // `computed`/spill-fact/dynamic-name-extent state into `self` in
        // place as it walks the worklist (only `runtime_dependencies` is
        // staged locally and committed at the end), so a mid-pass failure
        // (effective cycle, convergence limit, evaluation error, dynamic-name
        // cycle, scalarization failure) leaves `self` with partially
        // published values but the OLD `runtime_dependencies` graph — a torn
        // state. Without this reset, a direct caller of this function (not
        // going through `recalculate_dirty_with_oxfml`'s own failure
        // handling) that already had `graph_installed == true` from an
        // earlier successful mark-all would leave both flags saying "trust
        // the graph" after this failed pass, so the next
        // `recalculate_dirty_with_oxfml` would wrongly take the incremental
        // path over torn state. Only a SUCCESSFUL completion of this
        // function (the existing set-true at the end) restores trust.
        self.graph_installed = false;
        self.graph_needs_full_rebuild = true;

        let authored = self.authored.clone();
        self.computed.clear();
        self.clear_formula_spill_facts(&authored);
        let base_spill_facts = self.overlays.spill_facts.clone();

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
            structural_dependency_edges: 0,
            overlay_dependency_edges: 0,
            dynamic_defined_name_evaluations: 0,
            external_subscription_updates: Vec::new(),
            visited_cells: Vec::with_capacity(authored.len()),
        };
        let mut runtime_dependencies = GridInvalidationRef::new(self.bounds);

        let mut applied_literals = BTreeSet::new();
        let mut pending = BTreeSet::new();
        for (address, cell) in &authored {
            if let GridAuthoredCell::Literal(value) = cell {
                report.cells_evaluated += 1;
                report.visited_cells.push(address.clone());
                report.literal_cells += 1;
                self.computed.insert(address.clone(), value.clone());
                runtime_dependencies.set_structural_dependencies(address.clone(), Vec::new())?;
                runtime_dependencies.clear_overlay_dependencies(address)?;
                applied_literals.insert(address.clone());
            }
        }

        let profile = StrictExcelGridReferenceProfile::with_bounds(self.bounds);
        let dynamic_name_report =
            self.refresh_dynamic_defined_names_with_oxfml(&profile, None, false, false)?;
        report.dynamic_defined_name_evaluations += dynamic_name_report.evaluations;
        report
            .external_subscription_updates
            .extend(dynamic_name_report.external_subscription_updates);
        for (address, cell) in &authored {
            let GridAuthoredCell::Formula(formula) = cell else {
                continue;
            };
            self.install_structural_dependencies_for_formula(
                &mut runtime_dependencies,
                address,
                formula,
                &profile,
            )?;
            pending.insert(address.clone());
        }

        let formula_cells = formula_count(&authored);
        let iteration_limit = formula_cells
            .max(1)
            .saturating_mul(formula_cells.max(1))
            .saturating_mul(4);
        let mut formula_iterations = 0usize;

        while !pending.is_empty() {
            let address =
                if let Some(address) = runtime_dependencies.next_ready_dirty_formula(&pending) {
                    address
                } else if let Some(address) =
                    runtime_dependencies.first_pending_with_overlay_dependencies(&pending)
                {
                    address
                } else {
                    let cycle = pending
                        .iter()
                        .find_map(|address| {
                            runtime_dependencies.effective_dependency_cycle_from(address, &pending)
                        })
                        .unwrap_or_else(|| pending.iter().cloned().collect());
                    return Err(GridRefError::EffectiveDependencyCycleDetected { cycle });
                };
            pending.remove(&address);
            let Some(GridAuthoredCell::Formula(formula)) = authored.get(&address) else {
                continue;
            };

            formula_iterations += 1;
            if formula_iterations > iteration_limit {
                return Err(GridRefError::IncrementalRecalcDidNotConverge { iteration_limit });
            }

            report.cells_evaluated += 1;
            report.formula_cells += 1;
            report.formula_evaluations += 1;
            report.visited_cells.push(address.clone());
            self.install_structural_dependencies_for_formula(
                &mut runtime_dependencies,
                &address,
                formula,
                &profile,
            )?;
            let outcome = self.evaluate_formula_with_spill_repair(&address, formula, &profile)?;
            report.external_subscription_updates.push(
                GridExternalAvailabilitySubscriptionUpdate::formula_root(
                    address.clone(),
                    outcome.trace.external_subscriptions.clone(),
                ),
            );
            runtime_dependencies
                .replace_overlay_dependencies_from_trace(address.clone(), &outcome.trace)?;
            if let Some(cycle) =
                runtime_dependencies.effective_dependency_cycle_from(&address, &pending)
            {
                return Err(GridRefError::EffectiveDependencyCycleDetected { cycle });
            }
            let publication_delta =
                self.publish_formula_value(address.clone(), outcome.value, &authored);
            runtime_dependencies.refresh_overlay_spill_blocker_dependency(
                address.clone(),
                publication_delta.current_spill_blocker_extent.clone(),
            )?;
            let spill_counters = publication_delta.counters;
            report.spill_facts_published += spill_counters.facts_published;
            report.spill_facts_blocked += spill_counters.facts_blocked;
            report.spill_ghost_cells_published += spill_counters.ghost_cells_published;

            let mut dirty_cells = BTreeSet::new();
            let publication_dirty_seeds = publication_delta.dirty_seeds();
            let dynamic_refresh_seeds = publication_dirty_seeds.clone();
            dirty_cells.extend(
                runtime_dependencies
                    .dirty_closure_for_seeds(publication_dirty_seeds)?
                    .dirty_cells,
            );
            let dynamic_names_to_refresh = dynamic_defined_name_keys_to_refresh(
                &self.dynamic_defined_names.keys().cloned().collect(),
                &self.dynamic_defined_name_dependencies,
                &self.volatile_dynamic_defined_names,
                &self.external_pending_dynamic_defined_names,
                &dynamic_refresh_seeds,
                self.bounds,
                false,
                false,
            )?;
            let dynamic_name_report = self.refresh_dynamic_defined_names_with_oxfml(
                &profile,
                Some(&dynamic_names_to_refresh),
                false,
                false,
            )?;
            report.dynamic_defined_name_evaluations += dynamic_name_report.evaluations;
            report
                .external_subscription_updates
                .extend(dynamic_name_report.external_subscription_updates.clone());
            if !dynamic_name_report.dirty_seeds.is_empty() {
                dirty_cells.extend(
                    runtime_dependencies
                        .dirty_closure_for_seeds(dynamic_name_report.dirty_seeds)?
                        .dirty_cells,
                );
            }
            dirty_cells.remove(&address);
            self.apply_dirty_cells_to_reference_worklist(
                &authored,
                &mut runtime_dependencies,
                &dirty_cells,
                &mut applied_literals,
                &mut pending,
                &mut report,
            )?;
        }

        self.repair_reference_spills_with_oxfml(
            &authored,
            &profile,
            &base_spill_facts,
            &mut runtime_dependencies,
            &mut report,
        )?;
        report.structural_dependency_edges = runtime_dependencies
            .semantic_dependency_count_for_layer(GridDependencyLayer::Structural);
        report.overlay_dependency_edges = runtime_dependencies
            .semantic_dependency_count_for_layer(GridDependencyLayer::CalcOverlay);
        self.runtime_dependencies = runtime_dependencies;
        self.refresh_reference_report_spill_counters(&mut report, &authored);
        self.refresh_spill_epoch_ledger();
        // A full rebuild just committed a graph consistent with everything
        // published during this pass, regardless of how this call was
        // reached (bootstrap, direct host call, or the dirty-recalc
        // fallback), so any previously pending rebuild requirement is
        // satisfied and the graph is now installed and trustworthy for
        // incremental recalc, even if it has zero edges (a sheet of only
        // dependency-free formulas like `=1+1` or `=NOW()`).
        self.graph_needs_full_rebuild = false;
        self.graph_installed = true;
        Ok(report)
    }

    /// Incremental dirty recalc entry point. Returns `Ok` with a graph
    /// staged into a local clone and committed to `self.runtime_dependencies`
    /// only once the whole worklist has drained successfully.
    ///
    /// Error contract: `computed` cell values and spill facts are published
    /// into `self` in place as each formula in the worklist evaluates, *not*
    /// staged like the dependency graph. If this call returns `Err`, any
    /// values it already published stay in `self` even though the
    /// dependency-graph clone that would have made incremental recalc find
    /// them again was discarded. To keep the sheet self-consistent, any `Err`
    /// escaping the worklist sets `self.graph_needs_full_rebuild`, which
    /// forces the *next* `recalculate_dirty_with_oxfml` call to fall back to
    /// a full `recalculate_mark_all_dirty_with_oxfml` rebuild (the same
    /// fallback the `!self.graph_installed` bootstrap case already uses)
    /// instead of trusting the stale incremental graph.
    pub fn recalculate_dirty_with_oxfml(
        &mut self,
        seeds: impl IntoIterator<Item = GridDirtySeed>,
    ) -> Result<GridCalcRefRecalcReport, GridRefError> {
        let authored = self.authored.clone();
        let formula_cells = formula_count(&authored);
        // `!self.graph_installed` covers the true bootstrap case (no mark-all
        // has ever run) and the mid-pass-failure case (`graph_needs_full_rebuild`).
        // This intentionally does NOT infer "needs rebuild" from a zero edge
        // count: a sheet whose formulas legitimately have no dependencies
        // (`=1+1`, `=NOW()`) has a correctly-installed, zero-edge graph after
        // its first mark-all and must take the incremental path afterward.
        if self.graph_needs_full_rebuild || !self.graph_installed {
            return match self.recalculate_mark_all_dirty_with_oxfml() {
                Ok(report) => {
                    self.graph_needs_full_rebuild = false;
                    Ok(report)
                }
                // Mark-all itself also publishes in place before committing
                // its own graph at the end; if it fails mid-pass, the sheet
                // is in the same "needs a full rebuild" state it started in,
                // so keep the flag set rather than clearing it.
                Err(error) => {
                    self.graph_needs_full_rebuild = true;
                    Err(error)
                }
            };
        }

        match self.recalculate_dirty_with_oxfml_worklist(seeds, &authored, formula_cells) {
            Ok(report) => {
                self.graph_needs_full_rebuild = false;
                Ok(report)
            }
            Err(error) => {
                // The worklist may have published cell/spill values into
                // `self` before hitting this error; the dependency-graph
                // clone it was building was discarded along with the error,
                // so those publications are no longer reachable from
                // `self.runtime_dependencies`. Force a full rebuild next
                // time rather than risk an incremental pass that silently
                // misses them.
                self.graph_needs_full_rebuild = true;
                Err(error)
            }
        }
    }

    fn recalculate_dirty_with_oxfml_worklist(
        &mut self,
        seeds: impl IntoIterator<Item = GridDirtySeed>,
        authored: &BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
        formula_cells: usize,
    ) -> Result<GridCalcRefRecalcReport, GridRefError> {
        let seeds = seeds.into_iter().collect::<BTreeSet<_>>();
        let force_volatile_dynamic_names = seeds.contains(&GridDirtySeed::Volatile);
        let force_external_dynamic_names = seeds.contains(&GridDirtySeed::External);
        let all_dynamic_name_keys = self.dynamic_defined_names.keys().cloned().collect();
        let dynamic_names_to_refresh = dynamic_defined_name_keys_to_refresh(
            &all_dynamic_name_keys,
            &self.dynamic_defined_name_dependencies,
            &self.volatile_dynamic_defined_names,
            &self.external_pending_dynamic_defined_names,
            &seeds,
            self.bounds,
            force_volatile_dynamic_names,
            force_external_dynamic_names,
        )?;

        let profile = StrictExcelGridReferenceProfile::with_bounds(self.bounds);
        let mut runtime_dependencies = self.runtime_dependencies.clone();
        let initial_closure = runtime_dependencies.dirty_closure_for_seeds(seeds)?;
        let mut pending = BTreeSet::new();
        let mut applied_literals = BTreeSet::new();
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
            structural_dependency_edges: 0,
            overlay_dependency_edges: 0,
            dynamic_defined_name_evaluations: 0,
            external_subscription_updates: Vec::new(),
            visited_cells: Vec::with_capacity(initial_closure.dirty_cells.len()),
        };

        self.apply_dirty_cells_to_reference_worklist(
            &authored,
            &mut runtime_dependencies,
            &initial_closure.dirty_cells,
            &mut applied_literals,
            &mut pending,
            &mut report,
        )?;

        let dynamic_name_report = self.refresh_dynamic_defined_names_with_oxfml(
            &profile,
            Some(&dynamic_names_to_refresh),
            force_volatile_dynamic_names,
            force_external_dynamic_names,
        )?;
        report.dynamic_defined_name_evaluations += dynamic_name_report.evaluations;
        report
            .external_subscription_updates
            .extend(dynamic_name_report.external_subscription_updates.clone());
        if !dynamic_name_report.dirty_seeds.is_empty() {
            let dirty_cells = runtime_dependencies
                .dirty_closure_for_seeds(dynamic_name_report.dirty_seeds)?
                .dirty_cells;
            self.apply_dirty_cells_to_reference_worklist(
                &authored,
                &mut runtime_dependencies,
                &dirty_cells,
                &mut applied_literals,
                &mut pending,
                &mut report,
            )?;
        }

        let iteration_limit = formula_cells
            .max(1)
            .saturating_mul(formula_cells.max(1))
            .saturating_mul(4);
        let mut formula_iterations = 0usize;

        while !pending.is_empty() {
            let address =
                if let Some(address) = runtime_dependencies.next_ready_dirty_formula(&pending) {
                    address
                } else if let Some(address) =
                    runtime_dependencies.first_pending_with_overlay_dependencies(&pending)
                {
                    address
                } else {
                    let cycle = pending
                        .iter()
                        .find_map(|address| {
                            runtime_dependencies.effective_dependency_cycle_from(address, &pending)
                        })
                        .unwrap_or_else(|| pending.iter().cloned().collect());
                    return Err(GridRefError::EffectiveDependencyCycleDetected { cycle });
                };
            pending.remove(&address);
            let Some(GridAuthoredCell::Formula(formula)) = authored.get(&address) else {
                continue;
            };

            formula_iterations += 1;
            if formula_iterations > iteration_limit {
                return Err(GridRefError::IncrementalRecalcDidNotConverge { iteration_limit });
            }

            report.cells_evaluated += 1;
            report.formula_cells += 1;
            report.formula_evaluations += 1;
            report.visited_cells.push(address.clone());
            self.install_structural_dependencies_for_formula(
                &mut runtime_dependencies,
                &address,
                formula,
                &profile,
            )?;
            let outcome = self.evaluate_formula_with_spill_repair(&address, formula, &profile)?;
            report.external_subscription_updates.push(
                GridExternalAvailabilitySubscriptionUpdate::formula_root(
                    address.clone(),
                    outcome.trace.external_subscriptions.clone(),
                ),
            );
            let overlay_update = runtime_dependencies
                .replace_overlay_dependencies_from_trace(address.clone(), &outcome.trace)?;
            if let Some(cycle) =
                runtime_dependencies.effective_dependency_cycle_from(&address, &pending)
            {
                return Err(GridRefError::EffectiveDependencyCycleDetected { cycle });
            }
            let publication_delta =
                self.publish_formula_value(address.clone(), outcome.value, &authored);
            let spill_blocker_update = runtime_dependencies
                .refresh_overlay_spill_blocker_dependency(
                    address.clone(),
                    publication_delta.current_spill_blocker_extent.clone(),
                )?;
            let spill_counters = publication_delta.counters;
            report.spill_facts_published += spill_counters.facts_published;
            report.spill_facts_blocked += spill_counters.facts_blocked;
            report.spill_ghost_cells_published += spill_counters.ghost_cells_published;

            let mut dirty_cells = BTreeSet::new();
            let overlay_dirty_seeds = overlay_update.dirty_seeds.clone();
            let spill_blocker_dirty_seeds = spill_blocker_update.dirty_seeds.clone();
            let publication_dirty_seeds = publication_delta.dirty_seeds();
            let mut dynamic_refresh_seeds = BTreeSet::new();
            dynamic_refresh_seeds.extend(overlay_dirty_seeds.iter().cloned());
            dynamic_refresh_seeds.extend(spill_blocker_dirty_seeds.iter().cloned());
            dynamic_refresh_seeds.extend(publication_dirty_seeds.iter().cloned());
            if !overlay_update.dirty_seeds.is_empty() {
                dirty_cells.extend(
                    runtime_dependencies
                        .dirty_closure_for_seeds(overlay_update.dirty_seeds)?
                        .dirty_cells,
                );
            }
            if !spill_blocker_update.dirty_seeds.is_empty() {
                dirty_cells.extend(
                    runtime_dependencies
                        .dirty_closure_for_seeds(spill_blocker_update.dirty_seeds)?
                        .dirty_cells,
                );
            }
            dirty_cells.extend(
                runtime_dependencies
                    .dirty_closure_for_seeds(publication_dirty_seeds)?
                    .dirty_cells,
            );
            let dynamic_names_to_refresh = dynamic_defined_name_keys_to_refresh(
                &self.dynamic_defined_names.keys().cloned().collect(),
                &self.dynamic_defined_name_dependencies,
                &self.volatile_dynamic_defined_names,
                &self.external_pending_dynamic_defined_names,
                &dynamic_refresh_seeds,
                self.bounds,
                false,
                false,
            )?;
            let dynamic_name_report = self.refresh_dynamic_defined_names_with_oxfml(
                &profile,
                Some(&dynamic_names_to_refresh),
                false,
                false,
            )?;
            report.dynamic_defined_name_evaluations += dynamic_name_report.evaluations;
            report
                .external_subscription_updates
                .extend(dynamic_name_report.external_subscription_updates.clone());
            if !dynamic_name_report.dirty_seeds.is_empty() {
                dirty_cells.extend(
                    runtime_dependencies
                        .dirty_closure_for_seeds(dynamic_name_report.dirty_seeds)?
                        .dirty_cells,
                );
            }
            dirty_cells.remove(&address);
            self.apply_dirty_cells_to_reference_worklist(
                &authored,
                &mut runtime_dependencies,
                &dirty_cells,
                &mut applied_literals,
                &mut pending,
                &mut report,
            )?;
        }

        report.structural_dependency_edges = runtime_dependencies
            .semantic_dependency_count_for_layer(GridDependencyLayer::Structural);
        report.overlay_dependency_edges = runtime_dependencies
            .semantic_dependency_count_for_layer(GridDependencyLayer::CalcOverlay);
        self.runtime_dependencies = runtime_dependencies;
        self.refresh_reference_report_spill_counters(&mut report, &authored);
        self.refresh_spill_epoch_ledger();
        Ok(report)
    }

    fn refresh_dynamic_defined_names_with_oxfml(
        &mut self,
        profile: &StrictExcelGridReferenceProfile,
        names_to_refresh: Option<&BTreeSet<String>>,
        force_volatile: bool,
        force_external: bool,
    ) -> Result<GridDynamicDefinedNameRefreshReport, GridRefError> {
        if self.dynamic_defined_names.is_empty() {
            self.volatile_dynamic_defined_names.clear();
            self.external_pending_dynamic_defined_names.clear();
            self.dynamic_defined_name_dependencies.clear();
            return Ok(GridDynamicDefinedNameRefreshReport::default());
        }
        // Snapshot of the calc-time realization state this call is allowed to
        // mutate, taken before any evaluation runs. `self.dynamic_defined_name_extents`
        // and `self.dynamic_defined_name_dependencies` are mutated in place
        // DURING the loop below (intra-pass evaluation of one name needs to
        // see the just-committed extents of names evaluated earlier in the
        // same pass, via `reference_system_provider`), so they cannot be
        // staged in ordinary local variables the way `dirty_names`/
        // `volatile_names`/`external_pending_names` already are. Instead,
        // roll `self` back to this snapshot on any error path, so a
        // mid-pass failure (e.g. a cycle discovered on the SECOND name after
        // the FIRST name already committed a genuine extent change) leaves
        // `self` exactly as it was before this call, instead of stranding a
        // committed extent change whose dirty seed was discarded with the
        // `Err`. A later successful refresh of the same name then correctly
        // sees its old (pre-this-call) extent again and re-emits the dirty
        // seed, instead of comparing against the previously-stranded value
        // and concluding nothing changed.
        let extents_snapshot = self.dynamic_defined_name_extents.clone();
        let dependencies_snapshot = self.dynamic_defined_name_dependencies.clone();

        let dynamic_defined_names = self.dynamic_defined_names.clone();
        let active_names = dynamic_defined_names
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut dirty_names = BTreeSet::new();
        let mut evaluations = 0;
        let mut external_subscription_updates = Vec::new();
        let mut volatile_names = self.volatile_dynamic_defined_names.clone();
        let mut external_pending_names = self.external_pending_dynamic_defined_names.clone();
        volatile_names.retain(|name_key| self.dynamic_defined_names.contains_key(name_key));
        external_pending_names.retain(|name_key| self.dynamic_defined_names.contains_key(name_key));
        self.dynamic_defined_name_dependencies
            .retain_names(&active_names);
        let mut pending = names_to_refresh
            .map(|names| names.intersection(&active_names).cloned().collect())
            .unwrap_or_else(|| active_names.clone());
        // Names already re-evaluated THIS refresh pass. The cycle check below
        // must be scoped to this set (plus the name just evaluated), not the
        // full `active_names` universe: `dynamic_defined_name_dependencies`
        // is a single shared ledger, so a name not yet reached this pass
        // still holds its PREVIOUS pass's reverse edges. Checking a cycle
        // against those stale entries can report a cycle that an ordinary
        // cell edit (e.g. retargeting an INDIRECT selector) already broke,
        // simply because the name on the other side of the (now-stale) edge
        // has not been re-evaluated yet this pass.
        let mut evaluated_this_pass: BTreeSet<String> = BTreeSet::new();
        let iteration_limit = active_names
            .len()
            .max(1)
            .saturating_mul(active_names.len().max(1))
            .saturating_mul(4);
        while let Some(name_key) = pending.iter().next().cloned() {
            pending.remove(&name_key);
            let Some(definition) = dynamic_defined_names.get(&name_key).cloned() else {
                continue;
            };
            evaluations += 1;
            if evaluations > iteration_limit {
                let cycle = self
                    .dynamic_defined_name_dependencies
                    .dynamic_name_cycle(&active_names)
                    .unwrap_or_else(|| active_names.iter().cloned().collect());
                self.dynamic_defined_name_extents = extents_snapshot;
                self.dynamic_defined_name_dependencies = dependencies_snapshot;
                return Err(GridRefError::DynamicDefinedNameCycleDetected { cycle });
            }
            let old_extent = self.dynamic_defined_name_extents.get(&name_key).cloned();
            let was_volatile = volatile_names.contains(&name_key);
            let was_external_pending = external_pending_names.contains(&name_key);
            let outcome = match self.evaluate_dynamic_defined_name_extent_with_oxfml(
                &definition,
                profile,
                was_volatile,
                was_external_pending,
            ) {
                Ok(outcome) => outcome,
                Err(error) => {
                    self.dynamic_defined_name_extents = extents_snapshot;
                    self.dynamic_defined_name_dependencies = dependencies_snapshot;
                    return Err(error);
                }
            };
            external_subscription_updates.push(
                GridExternalAvailabilitySubscriptionUpdate::dynamic_defined_name(
                    name_key.clone(),
                    outcome.external_subscriptions.clone(),
                ),
            );
            self.dynamic_defined_name_dependencies
                .set_dependencies(name_key.clone(), outcome.formula_dependencies);
            evaluated_this_pass.insert(name_key.clone());
            if let Some(cycle) = self
                .dynamic_defined_name_dependencies
                .dynamic_name_cycle(&evaluated_this_pass)
            {
                self.dynamic_defined_name_extents = extents_snapshot;
                self.dynamic_defined_name_dependencies = dependencies_snapshot;
                return Err(GridRefError::DynamicDefinedNameCycleDetected { cycle });
            }
            volatile_names.remove(&name_key);
            external_pending_names.remove(&name_key);
            if outcome.volatile {
                volatile_names.insert(name_key.clone());
                if force_volatile {
                    dirty_names.insert(name_key.clone());
                }
            }
            if outcome.external_pending {
                external_pending_names.insert(name_key.clone());
                if force_external {
                    dirty_names.insert(name_key.clone());
                }
            }
            if old_extent == outcome.extent {
                continue;
            }
            if let Some(extent) = outcome.extent {
                self.dynamic_defined_name_extents
                    .insert(name_key.clone(), extent);
            } else {
                self.dynamic_defined_name_extents.remove(&name_key);
            }
            dirty_names.insert(name_key.clone());
            pending.extend(
                self.dynamic_defined_name_dependencies
                    .dependent_names_for_name(&name_key, &active_names),
            );
        }
        self.volatile_dynamic_defined_names = volatile_names;
        self.external_pending_dynamic_defined_names = external_pending_names;
        Ok(GridDynamicDefinedNameRefreshReport {
            dirty_seeds: grid_name_lifecycle_dirty_seeds(dirty_names),
            evaluations,
            external_subscription_updates,
        })
    }

    fn evaluate_dynamic_defined_name_extent_with_oxfml(
        &self,
        definition: &GridDynamicDefinedName,
        profile: &StrictExcelGridReferenceProfile,
        was_volatile: bool,
        was_external_pending: bool,
    ) -> Result<GridDynamicDefinedNameEvaluationOutcome, GridRefError> {
        let provider = self.reference_system_provider(definition.anchor.row, definition.anchor.col);
        let structural_dependencies = grid_structural_dependencies_for_formula(
            &definition.formula,
            &definition.anchor,
            profile,
            self.bounds,
            &provider,
        );
        let structural_dependencies_vec =
            structural_dependencies.iter().cloned().collect::<Vec<_>>();
        let outcome = match self.evaluate_formula_with_oxfml(
            &definition.anchor,
            &definition.formula,
            profile,
        ) {
            Ok(outcome) => outcome,
            Err(_) => {
                // A transient evaluation error must not strip the name's
                // re-poll protection: hard-coding `false` here would drop it
                // from the volatile/external root sets in the caller,
                // letting warm-no-op stop refusing reuse for a name that is
                // still volatile/external by construction (its formula
                // didn't change) and merely failed to evaluate this pass.
                // Preserve the name's previous root status instead.
                return Ok(GridDynamicDefinedNameEvaluationOutcome {
                    extent: None,
                    formula_dependencies: structural_dependencies,
                    volatile: was_volatile,
                    external_pending: was_external_pending,
                    external_subscriptions: BTreeSet::new(),
                });
            }
        };
        let volatile = outcome.trace.volatile;
        let external_pending = outcome.trace.is_external_pending();
        let external_subscriptions = outcome.trace.external_subscriptions.clone();
        let realized_dependencies = outcome.trace.realized_dependencies.clone();
        let mut formula_dependencies = structural_dependencies.clone();
        formula_dependencies.extend(realized_dependencies.iter().cloned());
        let overlay_dependencies = outcome
            .trace
            .overlay_dependencies_excluding_structural(&structural_dependencies_vec);
        let target_dependencies = if overlay_dependencies.is_empty() {
            if realized_dependencies.is_empty() {
                structural_dependencies.clone()
            } else {
                realized_dependencies
            }
        } else {
            overlay_dependencies
        };
        Ok(GridDynamicDefinedNameEvaluationOutcome {
            extent: dynamic_defined_name_extent_from_trace(
                &outcome.trace,
                &target_dependencies,
                self.bounds,
            ),
            formula_dependencies,
            volatile,
            external_pending,
            external_subscriptions,
        })
    }

    pub fn run_dirty_recalc_differential_with_oxfml(
        &self,
        seeds: impl IntoIterator<Item = GridDirtySeed>,
        probes: impl IntoIterator<Item = ExcelGridCellAddress>,
    ) -> Result<GridDirtyRecalcDifferentialRunReport, GridRefError> {
        let seeds = seeds.into_iter().collect::<Vec<_>>();
        let probes = probes.into_iter().collect::<Vec<_>>();

        let mut dirty = self.clone();
        let dirty_report = dirty.recalculate_dirty_with_oxfml(seeds)?;
        let dirty_readout = probes
            .iter()
            .map(|address| GridEngineCellReadout {
                address: address.clone(),
                computed: dirty.read_cell(address),
            })
            .collect::<Vec<_>>();
        let dirty_spill_facts = dirty.spill_facts().values().cloned().collect::<Vec<_>>();
        let dirty_dependencies = dirty.runtime_dependency_graph().clone();
        let dirty_dynamic_defined_names = GridDynamicDefinedNameRuntimeSnapshot::new(
            dirty.dynamic_defined_names().keys().cloned().collect(),
            dirty.dynamic_defined_name_extents().clone(),
            dirty.dynamic_defined_name_dependencies().clone(),
            dirty.volatile_dynamic_defined_names().clone(),
            dirty.external_pending_dynamic_defined_names().clone(),
        );

        let mut mark_all = self.clone();
        let mark_all_report = mark_all.recalculate_mark_all_dirty_with_oxfml()?;
        let mark_all_readout = probes
            .iter()
            .map(|address| GridEngineCellReadout {
                address: address.clone(),
                computed: mark_all.read_cell(address),
            })
            .collect::<Vec<_>>();
        let mark_all_spill_facts = mark_all.spill_facts().values().cloned().collect::<Vec<_>>();
        let mark_all_dependencies = mark_all.runtime_dependency_graph().clone();
        let mark_all_dynamic_defined_names = GridDynamicDefinedNameRuntimeSnapshot::new(
            mark_all.dynamic_defined_names().keys().cloned().collect(),
            mark_all.dynamic_defined_name_extents().clone(),
            mark_all.dynamic_defined_name_dependencies().clone(),
            mark_all.volatile_dynamic_defined_names().clone(),
            mark_all.external_pending_dynamic_defined_names().clone(),
        );

        let dirty_spill_epoch_ledger = dirty.spill_epoch_ledger().clone();
        let mark_all_spill_epoch_ledger = mark_all.spill_epoch_ledger().clone();

        // FIX 4: see the matching comment in the optimized-engine twin
        // (`run_dirty_recalc_differential_with_oxfml` in optimized_sheet.rs)
        // for why an empty-but-real seed is used here instead of `None`.
        let registry_effect_seed = GridExternalAvailabilityTopicRegistry::default();

        Ok(build_grid_dirty_recalc_differential_report(
            GridEngineMode::Reference,
            GridEngineRecalcReport::Reference(dirty_report.clone()),
            GridEngineRecalcReport::Reference(mark_all_report.clone()),
            dirty_readout,
            mark_all_readout,
            dirty_spill_facts,
            mark_all_spill_facts,
            &dirty_dependencies,
            &mark_all_dependencies,
            dirty_dynamic_defined_names,
            mark_all_dynamic_defined_names,
            &dirty_spill_epoch_ledger,
            &mark_all_spill_epoch_ledger,
            Some(&registry_effect_seed),
            &dirty_report.external_subscription_updates,
            &mark_all_report.external_subscription_updates,
        ))
    }

    fn repair_reference_spills_with_oxfml(
        &mut self,
        authored: &BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
        profile: &StrictExcelGridReferenceProfile,
        base_spill_facts: &BTreeMap<ExcelGridCellAddress, GridSpillFact>,
        runtime_dependencies: &mut GridInvalidationRef,
        report: &mut GridCalcRefRecalcReport,
    ) -> Result<(), GridRefError> {
        let formula_cells = formula_count(authored);
        if formula_cells == 0
            || self.overlays.spill_facts == *base_spill_facts
            || !authored_contains_grid_spill_reference(authored, profile, self.bounds)
        {
            return Ok(());
        }

        report.spill_repair_converged = false;
        for _ in 0..formula_cells {
            let spill_facts_before = self.overlays.spill_facts.clone();
            // B5: a pass can leave spill_facts unchanged while still
            // publishing a changed *plain* value (most commonly a volatile
            // precedent re-randomizing after a consumer already read it
            // earlier in this same pass). Convergence must require both
            // spill_facts stability AND published-value stability across the
            // whole pass; otherwise a later precedent's fresh value can go
            // unread by an already-finalized plain consumer. Every formula
            // is unconditionally re-evaluated address-order each pass
            // already, so requiring one more full pass after any such
            // change is enough to let every consumer observe the settled
            // values, still bounded by the existing `formula_cells` pass
            // limit. (`publication_delta.dirty_seeds()` is not used for this
            // check: it reports a formula's *current* effective cells
            // unconditionally, not a before/after value diff, so it cannot
            // distinguish "value changed" from "value republished
            // unchanged".)
            let computed_before = self.computed.clone();
            report.spill_repair_passes += 1;

            for (address, cell) in authored {
                let GridAuthoredCell::Formula(formula) = cell else {
                    continue;
                };
                report.spill_repair_formula_evaluations += 1;
                let outcome = self.evaluate_formula_with_spill_repair(address, formula, profile)?;
                report.external_subscription_updates.push(
                    GridExternalAvailabilitySubscriptionUpdate::formula_root(
                        address.clone(),
                        outcome.trace.external_subscriptions.clone(),
                    ),
                );
                runtime_dependencies
                    .replace_overlay_dependencies_from_trace(address.clone(), &outcome.trace)?;
                let publication_delta =
                    self.publish_formula_value(address.clone(), outcome.value, authored);
                runtime_dependencies.refresh_overlay_spill_blocker_dependency(
                    address.clone(),
                    publication_delta.current_spill_blocker_extent,
                )?;
            }

            if self.overlays.spill_facts == spill_facts_before && self.computed == computed_before {
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
    ) -> Result<GridFormulaEvaluationOutcome, GridRefError> {
        match self.evaluate_formula_with_oxfml(address, formula, profile) {
            Ok(outcome) => Ok(outcome),
            Err(error) => {
                if formula_contains_grid_spill_reference(formula, address, profile, self.bounds) {
                    Ok(GridFormulaEvaluationOutcome {
                        value: CalcValue::error(WorksheetErrorCode::Ref),
                        trace: GridRuntimeDependencyTrace::default(),
                    })
                } else {
                    Err(error)
                }
            }
        }
    }

    fn install_structural_dependencies_for_formula(
        &self,
        runtime_dependencies: &mut GridInvalidationRef,
        address: &ExcelGridCellAddress,
        formula: &GridFormulaCell,
        profile: &StrictExcelGridReferenceProfile,
    ) -> Result<(), GridRefError> {
        let provider = self.reference_system_provider(address.row, address.col);
        let structural_dependencies = grid_structural_dependencies_for_formula(
            formula,
            address,
            profile,
            self.bounds,
            &provider,
        );
        runtime_dependencies
            .set_structural_dependencies(address.clone(), structural_dependencies)
            .map(|_| ())
    }

    fn apply_dirty_cells_to_reference_worklist(
        &mut self,
        authored: &BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
        runtime_dependencies: &mut GridInvalidationRef,
        dirty_cells: &BTreeSet<ExcelGridCellAddress>,
        applied_literals: &mut BTreeSet<ExcelGridCellAddress>,
        pending: &mut BTreeSet<ExcelGridCellAddress>,
        report: &mut GridCalcRefRecalcReport,
    ) -> Result<(), GridRefError> {
        let mut vacated_dirty_seeds = BTreeSet::new();
        for address in dirty_cells {
            match authored.get(address) {
                Some(GridAuthoredCell::Literal(value)) => {
                    if applied_literals.insert(address.clone()) {
                        if let Some(old_fact) = self.clear_formula_output_for_anchor(address) {
                            vacated_dirty_seeds.extend(grid_vacated_spill_extent_dirty_seeds(
                                address,
                                &old_fact.extent,
                            ));
                        }
                        self.computed.insert(address.clone(), value.clone());
                        runtime_dependencies
                            .set_structural_dependencies(address.clone(), Vec::new())?;
                        runtime_dependencies.clear_overlay_dependencies(address)?;
                        report.cells_evaluated += 1;
                        report.literal_cells += 1;
                        report.visited_cells.push(address.clone());
                    }
                }
                Some(GridAuthoredCell::Formula(_)) => {
                    pending.insert(address.clone());
                }
                None => {}
            }
        }
        if !vacated_dirty_seeds.is_empty() {
            let vacated_dirty_cells = runtime_dependencies
                .dirty_closure_for_seeds(vacated_dirty_seeds)?
                .dirty_cells;
            self.apply_dirty_cells_to_reference_worklist(
                authored,
                runtime_dependencies,
                &vacated_dirty_cells,
                applied_literals,
                pending,
                report,
            )?;
        }
        Ok(())
    }

    fn refresh_reference_report_spill_counters(
        &self,
        report: &mut GridCalcRefRecalcReport,
        authored: &BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
    ) {
        let counters = count_formula_spill_publications(&self.overlays.spill_facts, |anchor| {
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
        self.overlays.spill_facts.retain(|anchor, _| {
            !matches!(authored.get(anchor), Some(GridAuthoredCell::Formula(_)))
        });
        self.spill_value_fingerprints.retain(|anchor, _| {
            !matches!(authored.get(anchor), Some(GridAuthoredCell::Formula(_)))
        });
    }

    fn clear_formula_output_for_anchor(
        &mut self,
        anchor: &ExcelGridCellAddress,
    ) -> Option<GridSpillFact> {
        if let Some(fact) = self.overlays.spill_facts.remove(anchor) {
            self.spill_value_fingerprints.remove(anchor);
            for key in grid_formula_output_cells_for_fact(&fact) {
                self.computed.remove(&key);
            }
            Some(fact)
        } else {
            self.spill_value_fingerprints.remove(anchor);
            self.computed.remove(anchor);
            None
        }
    }

    fn publish_formula_value(
        &mut self,
        address: ExcelGridCellAddress,
        value: CalcValue,
        authored: &BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
    ) -> GridValuePublicationDelta {
        let old_fact = self.overlays.spill_facts.get(&address).cloned();
        let old_effective_cells = grid_formula_output_cells_before_publication(
            &address,
            old_fact.as_ref(),
            self.computed.contains_key(&address),
        );
        self.clear_formula_output_for_anchor(&address);

        let Some(array) = value.as_array() else {
            self.computed.insert(address.clone(), value);
            let new_effective_cells = [address.clone()].into_iter().collect();
            return GridValuePublicationDelta::new(
                address,
                old_fact.as_ref(),
                old_effective_cells,
                None,
                new_effective_cells,
                GridSpillPublicationCounters::default(),
            );
        };

        let Some(extent) = spill_extent_for_array(&address, array.shape(), self.bounds) else {
            self.computed
                .insert(address.clone(), CalcValue::error(WorksheetErrorCode::Spill));
            self.spill_value_fingerprints
                .insert(address.clone(), blocked_spill_value_fingerprint(array));
            let new_fact = GridSpillFact {
                anchor: address.clone(),
                extent: anchor_cell_rect(&address, self.bounds),
                blocked: true,
            };
            self.overlays
                .spill_facts
                .insert(address.clone(), new_fact.clone());
            return GridValuePublicationDelta::new(
                address.clone(),
                old_fact.as_ref(),
                old_effective_cells,
                Some(&new_fact),
                [address].into_iter().collect(),
                GridSpillPublicationCounters {
                    facts_blocked: 1,
                    ..GridSpillPublicationCounters::default()
                },
            );
        };

        if self.reference_spill_extent_is_blocked(&address, &extent, authored) {
            self.computed
                .insert(address.clone(), CalcValue::error(WorksheetErrorCode::Spill));
            self.spill_value_fingerprints
                .insert(address.clone(), blocked_spill_value_fingerprint(array));
            let new_fact = GridSpillFact {
                anchor: address.clone(),
                extent,
                blocked: true,
            };
            self.overlays
                .spill_facts
                .insert(address.clone(), new_fact.clone());
            return GridValuePublicationDelta::new(
                address.clone(),
                old_fact.as_ref(),
                old_effective_cells,
                Some(&new_fact),
                [address].into_iter().collect(),
                GridSpillPublicationCounters {
                    facts_blocked: 1,
                    ..GridSpillPublicationCounters::default()
                },
            );
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
        let new_effective_cells = grid_formula_output_cells_for_extent(&extent);
        let new_fact = GridSpillFact {
            anchor: address.clone(),
            extent,
            blocked: false,
        };
        self.overlays
            .spill_facts
            .insert(address.clone(), new_fact.clone());
        self.spill_value_fingerprints
            .insert(address, calc_array_value_fingerprint(array));
        GridValuePublicationDelta::new(
            new_fact.anchor.clone(),
            old_fact.as_ref(),
            old_effective_cells,
            Some(&new_fact),
            new_effective_cells,
            GridSpillPublicationCounters {
                facts_published: 1,
                ghost_cells_published: array.cell_count().saturating_sub(1),
                ..GridSpillPublicationCounters::default()
            },
        )
    }

    fn reference_spill_extent_is_blocked(
        &self,
        anchor: &ExcelGridCellAddress,
        extent: &GridRect,
        authored: &BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
    ) -> bool {
        if blocked_formula_spill_extent_contains_anchor(
            anchor,
            &self.overlays.spill_facts,
            |fact_anchor| {
                matches!(
                    authored.get(fact_anchor),
                    Some(GridAuthoredCell::Formula(_))
                )
            },
        ) {
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
                    .overlays
                    .merged_regions
                    .iter()
                    .any(|region| region.rect.contains(&address))
                {
                    return true;
                }
                if self.overlays.feature_rendered_regions.iter().any(|region| {
                    feature_rendered_region_blocks_spill(&region.feature_kind)
                        && region.rect.contains(&address)
                }) {
                    return true;
                }
                if self.overlays.spill_facts.values().any(|fact| {
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
        .with_borrowed_cells(&self.computed)
        .with_cross_sheet_cells(
            self.cross_sheet_cells
                .iter()
                .map(|(address, value)| (address.clone(), value.clone())),
        );
        for fact in self.overlays.spill_facts.values() {
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
            provider = provider.with_defined_name_key(name.clone(), rect.clone());
        }
        for (name, rect) in &self.dynamic_defined_name_extents {
            provider = provider.with_defined_name_key(name.clone(), rect.clone());
        }
        // A dynamic name registered in `dynamic_defined_names` with no
        // matching entry in `dynamic_defined_name_extents` is defined but
        // currently unresolved (its own defining formula errored, e.g.
        // `InputRange = INDIRECT(C1)` off-grid): a consumer lookup miss on
        // it must classify as `#VALUE!`, not `#NAME?`. See
        // `is_name_class_reference`.
        for name in self.dynamic_defined_names.keys() {
            if !self.dynamic_defined_name_extents.contains_key(name) {
                provider = provider.with_unresolved_registered_name_key(name.clone());
            }
        }
        let caller_address = ExcelGridCellAddress::new(
            self.workbook_id.clone(),
            self.sheet_id.clone(),
            caller_row,
            caller_col,
        );
        for table in self.overlays.table_overlays.values() {
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
            self.overlays.spill_facts.values(),
            &self.defined_names,
            &self.dynamic_defined_name_extents,
            self.overlays.table_overlays.values(),
            &self.axis_state,
        )
    }

    fn evaluate_formula_with_oxfml(
        &self,
        address: &ExcelGridCellAddress,
        formula: &GridFormulaCell,
        profile: &StrictExcelGridReferenceProfile,
    ) -> Result<GridFormulaEvaluationOutcome, GridRefError> {
        let provider = self.reference_system_provider(address.row, address.col);
        let tracing_provider = GridTracingReferenceSystemProvider::new(&provider);
        let host_info = self.host_info_provider(address.row, address.col);
        let query_bundle = TypedContextQueryBundle::new(
            Some(&host_info as &dyn HostInfoProvider),
            None,
            None,
            None,
            None,
        )
        .with_reference_system_provider(Some(
            &tracing_provider as &dyn oxfunc_core::resolver::ReferenceSystemProvider,
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
            grid_table_caller_context(self.overlays.table_overlays.values(), address);
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
                grid_table_descriptor_catalog(self.overlays.table_overlays.values()),
                enclosing_table_ref,
                caller_table_region,
            )
            .with_reference_bind_profile(profile);
        let request = RuntimeFormulaRequest::new(source, query_bundle)
            .with_backend(EvaluationBackend::OxFuncBacked);
        let result =
            environment
                .execute(request)
                .map_err(|detail| GridRefError::OxfmlEvaluationFailed {
                    address: address.clone(),
                    detail,
                })?;
        let mut trace = tracing_provider.finish();
        trace.volatile = result.semantic_plan.execution_profile.volatility
            != oxfml_core::semantics::FormulaVolatilityClass::Stable;
        trace.add_external_subscriptions_from_runtime_result(&result);
        // F1: classify metadata-only runtime-realized consumption over an
        // independently rebound copy of the formula's bound tree (semantic
        // structure, not text) so ROWS/COLUMNS/ROW/COLUMN-only consumers of
        // an INDIRECT/OFFSET realized reference get invalidation-only
        // `ReferenceMetadata` overlay edges instead of value edges.
        let bound = bind_grid_formula_for_transform(formula, address, profile, self.bounds);
        trace.runtime_realized_dependencies_are_metadata_only =
            grid_formula_runtime_realized_dependencies_are_metadata_only(&bound);
        // G5(b): the structural feeder only sees AxisVisibility dependencies
        // for statically-walkable reference arguments, so a hidden-row-
        // sensitive aggregate over a text-realized target (e.g.
        // `SUBTOTAL(109,INDIRECT(C1))`) gets no AxisVisibility coverage from
        // that path. When the bound tree contains SUBTOTAL/AGGREGATE
        // anywhere, derive AxisVisibility dependencies from whatever the
        // trace actually realized this evaluation and fold them into the
        // overlay-replacement input so they install/retarget exactly like
        // any other runtime-realized dependency.
        if grid_bound_formula_contains_hidden_sensitive_function(&bound) {
            trace.realized_dependencies.extend(
                grid_axis_visibility_overlay_dependencies_from_trace(&trace, self.bounds),
            );
        }
        Ok(GridFormulaEvaluationOutcome {
            value: result.published_calc_value(),
            trace,
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
        self.overlays
            .spill_facts
            .values()
            .find(|fact| fact.extent.contains(address))
            .map(|fact| fact.anchor.clone())
    }
}
