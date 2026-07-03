//! GridOptimizedSheet: the optimized grid engine's orchestrator. Owns the
//! compact authored state and drives mutate / recalc / commit, spill
//! publication, the warm no-op fast path, repeated-R1C1 plan compilation,
//! and structural-edit application, producing a GridOptimizedValuation the
//! differential harness checks against the GridCalc-Ref oracle. Internal to
//! the machine; shares the machine's types via `use super::*`.

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct GridOptimizedSheet {
    pub(super) workbook_id: String,
    pub(super) sheet_id: String,
    pub(super) bounds: ExcelGridBounds,
    pub(super) next_revision: u64,
    pub(super) axis_state: GridAxisState,
    pub(super) sparse_points: BTreeMap<GridCellCoord, GridVersionedAuthoredCell>,
    pub(super) dense_value_regions: Vec<GridDenseValueRegion>,
    pub(super) repeated_formula_regions: Vec<GridRepeatedFormulaRegion>,
    pub(super) spill_value_fingerprints: BTreeMap<ExcelGridCellAddress, String>,
    pub(super) spill_epoch_ledger: GridSpillEpochLedger,
    pub(super) defined_names: BTreeMap<String, GridRect>,
    pub(super) dynamic_defined_names: BTreeMap<String, GridDynamicDefinedName>,
    pub(super) dynamic_defined_name_extents: BTreeMap<String, GridRect>,
    pub(super) overlays: GridOverlaySet,
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
            spill_value_fingerprints: BTreeMap::new(),
            spill_epoch_ledger: GridSpillEpochLedger::default(),
            defined_names: BTreeMap::new(),
            dynamic_defined_names: BTreeMap::new(),
            dynamic_defined_name_extents: BTreeMap::new(),
            overlays: GridOverlaySet::default(),
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
            self.overlays.spill_facts.values(),
            &self.defined_names,
            // Committed sheet state has no evaluated dynamic-name extents
            // (those only exist inside a live valuation); the recalc-path
            // provider below supplies them from `valuation` instead.
            std::iter::empty(),
            self.overlays.table_overlays.values(),
            &self.axis_state,
        )
    }

    pub(super) fn empty_valuation_with_committed_spill_state(&self) -> GridOptimizedValuation {
        GridOptimizedValuation::with_spill_state(
            self.workbook_id.clone(),
            self.sheet_id.clone(),
            self.bounds,
            self.overlays.spill_facts.clone(),
            self.spill_value_fingerprints.clone(),
            self.spill_epoch_ledger.clone(),
        )
        .with_defined_names(self.defined_names.clone())
        .with_dynamic_defined_name_keys(self.dynamic_defined_names.keys().cloned().collect())
        .with_dynamic_defined_name_extents(self.dynamic_defined_name_extents.clone())
        .with_table_overlays(self.overlays.table_overlays.clone())
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
        &self.overlays.spill_facts
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

        let previous_spill_fact_entries = self.overlays.spill_facts.len();
        let previous_spill_fingerprint_entries = self.spill_value_fingerprints.len();
        let previous_epoch_anchors = self.spill_epoch_ledger.entries().len();

        self.overlays.spill_facts = valuation.spill_facts.clone();
        self.spill_value_fingerprints = valuation.spill_value_fingerprints.clone();
        let ledger_update = self.refresh_spill_epoch_ledger();

        Ok(GridOptimizedSpillPublicationCommitReport {
            previous_spill_fact_entries,
            committed_spill_fact_entries: self.overlays.spill_facts.len(),
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

    #[must_use]
    pub fn dynamic_defined_names(&self) -> &BTreeMap<String, GridDynamicDefinedName> {
        &self.dynamic_defined_names
    }

    #[must_use]
    pub fn dynamic_defined_name_extents(&self) -> &BTreeMap<String, GridRect> {
        &self.dynamic_defined_name_extents
    }

    pub fn set_defined_name(
        &mut self,
        name: impl AsRef<str>,
        rect: GridRect,
    ) -> Result<GridNameLifecycleReport, GridRefError> {
        self.check_rect(&rect)?;
        let name_key = defined_name_key_for_name(name.as_ref(), self.bounds)?;
        let removed_dynamic_name = self.dynamic_defined_names.remove(&name_key).is_some();
        let removed_dynamic_extent = self
            .dynamic_defined_name_extents
            .remove(&name_key)
            .is_some();
        let replaced_static = self.defined_names.insert(name_key.clone(), rect).is_some();
        let operation = if replaced_static || removed_dynamic_name || removed_dynamic_extent {
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
        self.check_rect(&rect)?;
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
        let replaced_static = self.defined_names.insert(name_key.clone(), rect).is_some();
        let operation = if replaced_static || removed_dynamic_name || removed_dynamic_extent {
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
        let operation = if removed_static || replaced_dynamic || removed_dynamic_extent {
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
        let operation = if removed_static || replaced_dynamic || removed_dynamic_extent {
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
        if rect.is_none() && dynamic_name.is_none() && dynamic_extent.is_none() {
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
        let stats = transform_sparse_point_formulas_for_defined_name_rename(
            &mut self.sparse_points,
            &self.workbook_id,
            &self.sheet_id,
            &old_key,
            new_name,
            self.bounds,
            shadowed_by_scope,
        )?;
        let repeated_stats = transform_repeated_formula_regions_for_defined_name_rename(
            &mut self.repeated_formula_regions,
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
        if !removed_static && !removed_dynamic && !removed_dynamic_extent {
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
            shadowed_by_scope,
        )?;
        let repeated_stats = transform_repeated_formula_regions_for_defined_name_delete(
            &mut self.repeated_formula_regions,
            &name_key,
            self.bounds,
            shadowed_by_scope,
        )?;
        let mut dirty_seed_keys = vec![name_key.clone()];
        if shadowed_by_scope && let Some(scoped_key) = scoped_shadow_key {
            dirty_seed_keys.push(scoped_key);
        }
        Ok(GridNameLifecycleReport {
            operation: GridNameLifecycleOperation::Delete,
            old_name_key: Some(name_key.clone()),
            new_name_key: None,
            dirty_seeds: grid_name_lifecycle_dirty_seeds(dirty_seed_keys),
            formula_cells_transformed: stats.formula_cells_transformed
                + repeated_stats.formula_cells_transformed,
            formula_reference_transforms: stats.formula_reference_transforms
                + repeated_stats.formula_reference_transforms,
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
        if !removed_static && !removed_dynamic && !removed_dynamic_extent {
            return Err(GridRefError::DefinedNameNotFound {
                name: name.to_string(),
            });
        }
        // The deleted scoped name may have been shadowing a same-text
        // global entry; a consumer bound to the global key before the
        // scope ever existed must also be dirtied. See D2.
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
        self.add_feature_rendered_region(table_range.clone(), "table-overlay", false)?;
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
        self.add_feature_rendered_region(table_range.clone(), "table-overlay", false)?;
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
            dirty_seeds: grid_table_lifecycle_dirty_seeds(
                [old_name.to_string(), new_name.to_string()],
                [table_range],
            ),
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
            dirty_seeds: grid_table_lifecycle_dirty_seeds([table_name.to_string()], [table_range]),
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
        &self.overlays.merged_regions
    }

    pub fn add_merged_region(&mut self, rect: GridRect) -> Result<(), GridRefError> {
        self.check_rect(&rect)?;
        self.overlays.merged_regions.push(GridMergedRegion { rect });
        Ok(())
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
    ) -> Result<(), GridRefError> {
        self.check_rect(&rect)?;
        self.overlays
            .feature_rendered_regions
            .push(FeatureRenderedRegion {
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

        // Fail-fast: a feature-rendered region that refuses an intersecting axis
        // edit (the pivot family) aborts the whole edit before any mutation, via
        // the overlay admission check - identical to the legacy pre-mutation
        // refusal (same FeatureRenderedRegionEditRefused detail string).
        for region in &self.overlays.feature_rendered_regions {
            if let EditAdmission::Refuse { detail } =
                GridOverlay::FeatureRendered(region.clone()).admit_axis_edit(edit)?
            {
                return Err(GridRefError::FeatureRenderedRegionEditRefused {
                    feature_kind: region.feature_kind.clone(),
                    detail,
                });
            }
        }

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
            mut sparse_formula_cells_transformed,
            mut sparse_formula_reference_transforms,
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

        // Defined names (non-overlay state; silent filter on drop).
        let old_defined_names = std::mem::take(&mut self.defined_names);
        for (name_key, rect) in old_defined_names {
            let (Some(rect), _) = transform_rect_for_edit(&rect, edit, self.bounds)? else {
                continue;
            };
            self.defined_names.insert(name_key, rect);
        }

        // The sheet-side dynamic-name extent cache mirrors the valuation's
        // calc-time realized extent state: it is cleared here (not shifted)
        // per the axis-edit rule and rebuilt on the next dynamic-name
        // refresh.
        self.dynamic_defined_name_extents.clear();

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
            sparse_formula_cells_transformed += stats.formula_cells_transformed;
            sparse_formula_reference_transforms += stats.formula_reference_transforms;
            self.dynamic_defined_names
                .insert(name_key, GridDynamicDefinedName::new(formula, new_anchor));
        }

        // Unified overlay transform: tables, merged regions, feature regions, and
        // spill facts all flow through GridOverlay::transform_for_axis_edit, then
        // redistribute to their per-kind storage (tables/spills re-keyed,
        // merged/features re-appended in their original order). Feature refusal
        // was handled fail-fast above, so transform here never errors on refusal.
        let mut overlays = Vec::new();
        for table in std::mem::take(&mut self.overlays.table_overlays).into_values() {
            overlays.push(GridOverlay::Table(table));
        }
        for region in std::mem::take(&mut self.overlays.merged_regions) {
            overlays.push(GridOverlay::Merged(region));
        }
        for region in std::mem::take(&mut self.overlays.feature_rendered_regions) {
            overlays.push(GridOverlay::FeatureRendered(region));
        }
        for fact in std::mem::take(&mut self.overlays.spill_facts).into_values() {
            overlays.push(GridOverlay::Spill(fact));
        }

        let mut spill_facts_kept = 0;
        let mut spill_facts_dropped = 0;
        let mut merged_regions_kept = 0;
        let mut merged_regions_dropped = 0;
        let mut feature_regions_kept = 0;
        let mut feature_regions_dropped = 0;
        let mut feature_regions_marked_needs_refresh = 0;
        for overlay in &overlays {
            let feature_was_marked =
                matches!(overlay, GridOverlay::FeatureRendered(region) if region.needs_refresh);
            match overlay.transform_for_axis_edit(edit, self.bounds)? {
                None => match overlay.kind() {
                    OverlayKind::Spill => spill_facts_dropped += 1,
                    OverlayKind::Merged => merged_regions_dropped += 1,
                    OverlayKind::FeatureRendered => feature_regions_dropped += 1,
                    _ => {}
                },
                Some(GridOverlay::Table(table)) => {
                    // The transform preserves table_name, and the map key is a
                    // pure function of the name, so re-deriving the key yields the
                    // same key the table was stored under (the `?` is unreachable
                    // for any table that was validly inserted).
                    let table_key = table_key_for_name(&table.table_name, self.bounds)?;
                    self.overlays.table_overlays.insert(table_key, table);
                }
                Some(GridOverlay::Merged(region)) => {
                    self.overlays.merged_regions.push(region);
                    merged_regions_kept += 1;
                }
                Some(GridOverlay::FeatureRendered(region)) => {
                    if region.needs_refresh && !feature_was_marked {
                        feature_regions_marked_needs_refresh += 1;
                    }
                    self.overlays.feature_rendered_regions.push(region);
                    feature_regions_kept += 1;
                }
                Some(GridOverlay::Spill(fact)) => {
                    self.overlays.spill_facts.insert(fact.anchor.clone(), fact);
                    spill_facts_kept += 1;
                }
                Some(GridOverlay::Extension(_)) => {
                    // OVL-6 seam: extension overlays have no GridOverlaySet
                    // storage, so the redistribution set - built only from the
                    // four stored claimers above - never yields one. CSE-1 /
                    // CF-1 / RICH-1 add storage and a real arm here.
                    unreachable!("extension overlays are not sheet-stored (OVL-6)")
                }
            }
        }

        // Spill auxiliary state, refreshed after the spill facts are updated.
        self.spill_value_fingerprints = transform_spill_value_fingerprints_for_edit(
            std::mem::take(&mut self.spill_value_fingerprints),
            edit,
            self.bounds,
        )?;
        self.refresh_spill_epoch_ledger();

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
            feature_regions_kept,
            feature_regions_dropped,
            feature_regions_marked_needs_refresh,
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
            spill_facts: self.overlays.spill_facts.len(),
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
                    overlay_blockage_mismatches: Vec::new(),
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
                    overlay_blockage_mismatches: Vec::new(),
                })
            }
            GridEngineMode::Both => {
                let reference =
                    self.run_reference_engine_with_oxfml(&probes, materialization_limit)?;
                let optimized =
                    self.run_optimized_engine_with_oxfml(&probes, materialization_limit)?;
                let mismatches =
                    compare_grid_engine_readouts(&reference.readout, &optimized.readout);
                // Permanent-pair overlay invariant: the optimized overlay-set
                // blockage probe (which produced `optimized.spill_facts`) must
                // agree with the reference brute-force blockage (which produced
                // `reference.spill_facts`).
                let overlay_blockage_mismatches =
                    compare_grid_overlay_blockage(&reference.spill_facts, &optimized.spill_facts);
                Ok(GridDifferentialRunReport {
                    mode,
                    reference: Some(reference),
                    optimized: Some(optimized),
                    mismatches,
                    overlay_blockage_mismatches,
                })
            }
        }
    }

    pub(super) fn run_reference_engine_with_oxfml(
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
            spill_facts: reference.spill_facts().values().cloned().collect(),
        })
    }

    pub(super) fn run_optimized_engine_with_oxfml(
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
        let warm_noop = self.recalculate_warm_noop_compact_with_oxfml(&cache).map(
            |(warm_valuation, warm_report)| {
                let warm_readout = probes
                    .iter()
                    .map(|address| GridEngineCellReadout {
                        address: address.clone(),
                        computed: warm_valuation.read_cell(address).computed,
                    })
                    .collect();
                GridEngineWarmNoOpReport {
                    recalc: warm_report,
                    readout: warm_readout,
                }
            },
        );
        Ok(GridEngineRunReport {
            mode: GridEngineMode::Optimized,
            recalc: GridEngineRecalcReport::Optimized(report),
            readout,
            warm_noop,
            spill_facts: valuation.spill_facts().values().cloned().collect(),
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
            structural_dependency_edges: 0,
            overlay_dependency_edges: 0,
            dynamic_defined_name_evaluations: 0,
            external_subscription_updates: Vec::new(),
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

    pub fn recalculate_dirty_compact_with_oxfml(
        &self,
        previous: &GridOptimizedValuation,
        seeds: impl IntoIterator<Item = GridDirtySeed>,
        materialization_limit: u64,
    ) -> Result<(GridOptimizedValuation, GridOptimizedRecalcReport), GridRefError> {
        self.check_valuation_identity(previous)?;
        // A visible-first (`GridOptimizedValuationCoverage::VisibleProjection`)
        // valuation only evaluated an upstream cone; it is indistinguishable
        // from a full valuation by shape alone (cone formulas install real
        // structural/overlay edges, so the `graph_edge_count == 0` guard
        // below does not fire). Seeding a dirty recalc from it would
        // silently under-recalculate everything outside the cone. Escalate
        // to a full mark-all instead of trusting it.
        if !previous.is_full_coverage() {
            let mut formula_plan_cache = GridOptimizedFormulaPlanCache::default();
            return self.recalculate_mark_all_dirty_compact_with_oxfml_using_formula_plan_cache(
                materialization_limit,
                &mut formula_plan_cache,
            );
        }
        let seeds = seeds.into_iter().collect::<BTreeSet<_>>();
        let force_volatile_dynamic_names = seeds.contains(&GridDirtySeed::Volatile);
        let force_external_dynamic_names = seeds.contains(&GridDirtySeed::External);
        let mut formula_plan_cache = GridOptimizedFormulaPlanCache::default();
        let mut valuation = previous.clone();
        valuation.defined_names = self.defined_names.clone();
        valuation
            .dynamic_defined_name_extents
            .retain(|name_key, _| self.dynamic_defined_names.contains_key(name_key));
        let all_dynamic_name_keys = self.dynamic_defined_names.keys().cloned().collect();
        valuation
            .dynamic_defined_name_dependencies
            .retain_names(&all_dynamic_name_keys);
        valuation
            .volatile_dynamic_defined_names
            .retain(|name_key| self.dynamic_defined_names.contains_key(name_key));
        valuation
            .external_pending_dynamic_defined_names
            .retain(|name_key| self.dynamic_defined_names.contains_key(name_key));
        valuation.table_overlays = self.overlays.table_overlays.clone();

        let formula_cells = self.final_formula_cell_count(materialization_limit)?;
        // `!valuation.graph_installed` (mirrors `GridCalcRefSheet::graph_installed`,
        // see calc_ref_sheet.rs) covers "no mark-all has ever populated this
        // valuation's graph" without inferring it from a zero edge count: a
        // sheet whose formulas legitimately have no dependencies (`=1+1`,
        // `=NOW()`) has a correctly-installed, zero-edge graph after its
        // first mark-all and must take the incremental path afterward.
        if formula_cells > 0 && !valuation.graph_installed {
            return self.recalculate_mark_all_dirty_compact_with_oxfml_using_formula_plan_cache(
                materialization_limit,
                &mut formula_plan_cache,
            );
        }

        let initial_closure = valuation
            .runtime_dependencies
            .dirty_closure_for_seeds(seeds.clone())?;
        let mut pending = BTreeSet::new();
        let mut applied_literals = BTreeSet::new();
        let mut report = GridOptimizedRecalcReport::empty();
        let mut prepared_templates = BTreeSet::new();
        let dynamic_names_to_refresh = dynamic_defined_name_keys_to_refresh(
            &all_dynamic_name_keys,
            &valuation.dynamic_defined_name_dependencies,
            &valuation.volatile_dynamic_defined_names,
            &valuation.external_pending_dynamic_defined_names,
            &seeds,
            self.bounds,
            force_volatile_dynamic_names,
            force_external_dynamic_names,
        )?;
        self.apply_dirty_cells_to_optimized_worklist(
            &mut valuation,
            &initial_closure.dirty_cells,
            &mut applied_literals,
            &mut pending,
            &mut report,
        )?;

        let dynamic_name_report = self.refresh_dynamic_defined_names_with_oxfml(
            &mut valuation,
            materialization_limit,
            Some(&dynamic_names_to_refresh),
            force_volatile_dynamic_names,
            force_external_dynamic_names,
        )?;
        report.dynamic_defined_name_evaluations += dynamic_name_report.evaluations;
        report
            .external_subscription_updates
            .extend(dynamic_name_report.external_subscription_updates.clone());
        if !dynamic_name_report.dirty_seeds.is_empty() {
            let dirty_cells = valuation
                .runtime_dependencies
                .dirty_closure_for_seeds(dynamic_name_report.dirty_seeds)?
                .dirty_cells;
            self.apply_dirty_cells_to_optimized_worklist(
                &mut valuation,
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
            let address = if let Some(address) = valuation
                .runtime_dependencies
                .next_ready_dirty_formula(&pending)
            {
                address
            } else if let Some(address) = valuation
                .runtime_dependencies
                .first_pending_with_overlay_dependencies(&pending)
            {
                address
            } else {
                let cycle = pending
                    .iter()
                    .find_map(|address| {
                        valuation
                            .runtime_dependencies
                            .effective_dependency_cycle_from(address, &pending)
                    })
                    .unwrap_or_else(|| pending.iter().cloned().collect());
                return Err(GridRefError::EffectiveDependencyCycleDetected { cycle });
            };
            pending.remove(&address);
            let Some((revision, GridAuthoredCell::Formula(formula), source)) =
                self.versioned_authored_cell_at(&address)
            else {
                continue;
            };

            formula_iterations += 1;
            if formula_iterations > iteration_limit {
                return Err(GridRefError::IncrementalRecalcDidNotConverge { iteration_limit });
            }

            count_optimized_dirty_formula_evaluation(&mut report, source);
            register_formula_plan_cache_access(
                &mut prepared_templates,
                &mut formula_plan_cache,
                &formula,
                None,
                &mut report,
                1,
            );
            self.install_optimized_structural_dependencies_for_formula(
                &mut valuation,
                &address,
                &formula,
                materialization_limit,
            )?;
            let outcome = self.evaluate_optimized_formula_with_spill_repair_outcome(
                &address,
                &formula,
                &valuation,
                materialization_limit,
            )?;
            report.external_subscription_updates.push(
                GridExternalAvailabilitySubscriptionUpdate::formula_root(
                    address.clone(),
                    outcome.trace.external_subscriptions.clone(),
                ),
            );
            let overlay_update = valuation
                .runtime_dependencies
                .replace_overlay_dependencies_from_trace(address.clone(), &outcome.trace)?;
            if let Some(cycle) = valuation
                .runtime_dependencies
                .effective_dependency_cycle_from(&address, &pending)
            {
                return Err(GridRefError::EffectiveDependencyCycleDetected { cycle });
            }
            let publication_delta = self.publish_formula_value_to_valuation(
                &mut valuation,
                address.clone(),
                revision,
                outcome.value,
                source,
            );
            let spill_blocker_update = valuation
                .runtime_dependencies
                .refresh_overlay_spill_blocker_dependency(
                    address.clone(),
                    publication_delta.current_spill_blocker_extent.clone(),
                )?;
            report.spill_facts_published += publication_delta.facts_published;
            report.spill_facts_blocked += publication_delta.facts_blocked;
            report.spill_ghost_cells_published += publication_delta.ghost_cells_published;

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
                    valuation
                        .runtime_dependencies
                        .dirty_closure_for_seeds(overlay_update.dirty_seeds)?
                        .dirty_cells,
                );
            }
            if !spill_blocker_update.dirty_seeds.is_empty() {
                dirty_cells.extend(
                    valuation
                        .runtime_dependencies
                        .dirty_closure_for_seeds(spill_blocker_update.dirty_seeds)?
                        .dirty_cells,
                );
            }
            dirty_cells.extend(
                valuation
                    .runtime_dependencies
                    .dirty_closure_for_seeds(publication_dirty_seeds)?
                    .dirty_cells,
            );
            let dynamic_names_to_refresh = dynamic_defined_name_keys_to_refresh(
                &self.dynamic_defined_names.keys().cloned().collect(),
                &valuation.dynamic_defined_name_dependencies,
                &valuation.volatile_dynamic_defined_names,
                &valuation.external_pending_dynamic_defined_names,
                &dynamic_refresh_seeds,
                self.bounds,
                false,
                false,
            )?;
            let dynamic_name_report = self.refresh_dynamic_defined_names_with_oxfml(
                &mut valuation,
                materialization_limit,
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
                    valuation
                        .runtime_dependencies
                        .dirty_closure_for_seeds(dynamic_name_report.dirty_seeds)?
                        .dirty_cells,
                );
            }
            dirty_cells.remove(&address);
            self.apply_dirty_cells_to_optimized_worklist(
                &mut valuation,
                &dirty_cells,
                &mut applied_literals,
                &mut pending,
                &mut report,
            )?;
        }

        report.formula_templates_prepared = prepared_templates.len();
        report.distinct_formula_templates = prepared_templates.len();
        formula_plan_cache.prune_to_templates(&prepared_templates);
        report.compiled_formula_plans_cached = formula_plan_cache.cached_compiled_plan_count();
        report.computed_dense_value_regions = valuation.dense_value_regions().len();
        report.computed_sparse_cells = valuation.sparse_computed_cells();
        report.structural_dependency_edges = valuation
            .runtime_dependencies
            .semantic_dependency_count_for_layer(GridDependencyLayer::Structural);
        report.overlay_dependency_edges = valuation
            .runtime_dependencies
            .semantic_dependency_count_for_layer(GridDependencyLayer::CalcOverlay);
        valuation.refresh_spill_epoch_ledger();
        self.refresh_optimized_report_spill_counters(&valuation, &mut report);
        Ok((valuation, report))
    }

    fn refresh_dynamic_defined_names_with_oxfml(
        &self,
        valuation: &mut GridOptimizedValuation,
        materialization_limit: u64,
        names_to_refresh: Option<&BTreeSet<String>>,
        force_volatile: bool,
        force_external: bool,
    ) -> Result<GridDynamicDefinedNameRefreshReport, GridRefError> {
        if self.dynamic_defined_names.is_empty() {
            valuation.dynamic_defined_name_keys.clear();
            valuation.dynamic_defined_name_extents.clear();
            valuation.dynamic_defined_name_dependencies.clear();
            valuation.volatile_dynamic_defined_names.clear();
            valuation.external_pending_dynamic_defined_names.clear();
            return Ok(GridDynamicDefinedNameRefreshReport::default());
        }
        let active_names = self
            .dynamic_defined_names
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        valuation.dynamic_defined_name_keys = active_names.clone();
        valuation
            .dynamic_defined_name_extents
            .retain(|name_key, _| self.dynamic_defined_names.contains_key(name_key));
        valuation
            .dynamic_defined_name_dependencies
            .retain_names(&active_names);
        let mut dirty_names = BTreeSet::new();
        let mut evaluations = 0;
        let mut external_subscription_updates = Vec::new();
        let mut volatile_names = valuation.volatile_dynamic_defined_names.clone();
        let mut external_pending_names = valuation.external_pending_dynamic_defined_names.clone();
        volatile_names.retain(|name_key| self.dynamic_defined_names.contains_key(name_key));
        external_pending_names.retain(|name_key| self.dynamic_defined_names.contains_key(name_key));
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
            let Some(definition) = self.dynamic_defined_names.get(&name_key) else {
                continue;
            };
            evaluations += 1;
            if evaluations > iteration_limit {
                let cycle = valuation
                    .dynamic_defined_name_dependencies
                    .dynamic_name_cycle(&active_names)
                    .unwrap_or_else(|| active_names.iter().cloned().collect());
                return Err(GridRefError::DynamicDefinedNameCycleDetected { cycle });
            }
            let old_extent = valuation
                .dynamic_defined_name_extents
                .get(&name_key)
                .cloned();
            let was_volatile = volatile_names.contains(&name_key);
            let was_external_pending = external_pending_names.contains(&name_key);
            let new_extent = self.evaluate_dynamic_defined_name_extent_with_oxfml(
                valuation,
                definition,
                materialization_limit,
                was_volatile,
                was_external_pending,
            )?;
            external_subscription_updates.push(
                GridExternalAvailabilitySubscriptionUpdate::dynamic_defined_name(
                    name_key.clone(),
                    new_extent.external_subscriptions.clone(),
                ),
            );
            valuation
                .dynamic_defined_name_dependencies
                .set_dependencies(name_key.clone(), new_extent.formula_dependencies);
            evaluated_this_pass.insert(name_key.clone());
            if let Some(cycle) = valuation
                .dynamic_defined_name_dependencies
                .dynamic_name_cycle(&evaluated_this_pass)
            {
                return Err(GridRefError::DynamicDefinedNameCycleDetected { cycle });
            }
            volatile_names.remove(&name_key);
            external_pending_names.remove(&name_key);
            if new_extent.volatile {
                volatile_names.insert(name_key.clone());
                if force_volatile {
                    dirty_names.insert(name_key.clone());
                }
            }
            if new_extent.external_pending {
                external_pending_names.insert(name_key.clone());
                if force_external {
                    dirty_names.insert(name_key.clone());
                }
            }
            if old_extent == new_extent.extent {
                continue;
            }
            if let Some(extent) = new_extent.extent {
                valuation
                    .dynamic_defined_name_extents
                    .insert(name_key.clone(), extent);
            } else {
                valuation.dynamic_defined_name_extents.remove(&name_key);
            }
            dirty_names.insert(name_key.clone());
            pending.extend(
                valuation
                    .dynamic_defined_name_dependencies
                    .dependent_names_for_name(&name_key, &active_names),
            );
        }
        valuation.volatile_dynamic_defined_names = volatile_names;
        valuation.external_pending_dynamic_defined_names = external_pending_names;
        Ok(GridDynamicDefinedNameRefreshReport {
            dirty_seeds: grid_name_lifecycle_dirty_seeds(dirty_names),
            evaluations,
            external_subscription_updates,
        })
    }

    fn evaluate_dynamic_defined_name_extent_with_oxfml(
        &self,
        valuation: &GridOptimizedValuation,
        definition: &GridDynamicDefinedName,
        materialization_limit: u64,
        was_volatile: bool,
        was_external_pending: bool,
    ) -> Result<GridDynamicDefinedNameEvaluationOutcome, GridRefError> {
        let structural_dependencies = {
            let provider = valuation.reference_system_provider_with_dense_materialization_limit(
                definition.anchor.row,
                definition.anchor.col,
                materialization_limit,
            );
            let profile = StrictExcelGridReferenceProfile::with_bounds(self.bounds);
            grid_structural_dependencies_for_formula(
                &definition.formula,
                &definition.anchor,
                &profile,
                self.bounds,
                &provider,
            )
        };
        let structural_dependencies_vec =
            structural_dependencies.iter().cloned().collect::<Vec<_>>();
        let outcome = match self.evaluate_optimized_formula_with_oxfml(
            &definition.anchor,
            &definition.formula,
            valuation,
            materialization_limit,
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
        previous: &GridOptimizedValuation,
        seeds: impl IntoIterator<Item = GridDirtySeed>,
        probes: impl IntoIterator<Item = ExcelGridCellAddress>,
        materialization_limit: u64,
    ) -> Result<GridDirtyRecalcDifferentialRunReport, GridRefError> {
        let seeds = seeds.into_iter().collect::<Vec<_>>();
        let probes = probes.into_iter().collect::<Vec<_>>();

        let (dirty, dirty_report) =
            self.recalculate_dirty_compact_with_oxfml(previous, seeds, materialization_limit)?;
        let dirty_readout = probes
            .iter()
            .map(|address| GridEngineCellReadout {
                address: address.clone(),
                computed: dirty.read_cell(address).computed,
            })
            .collect::<Vec<_>>();
        let dirty_spill_facts = dirty.spill_facts().values().cloned().collect::<Vec<_>>();
        let dirty_dependencies = dirty.runtime_dependency_graph().clone();
        let dirty_dynamic_defined_names = GridDynamicDefinedNameRuntimeSnapshot::new(
            dirty.dynamic_defined_name_keys().clone(),
            dirty.dynamic_defined_name_extents().clone(),
            dirty.dynamic_defined_name_dependencies().clone(),
            dirty.volatile_dynamic_defined_names().clone(),
            dirty.external_pending_dynamic_defined_names().clone(),
        );

        let (mark_all, mark_all_report) =
            self.recalculate_mark_all_dirty_compact_with_oxfml(materialization_limit)?;
        let mark_all_readout = probes
            .iter()
            .map(|address| GridEngineCellReadout {
                address: address.clone(),
                computed: mark_all.read_cell(address).computed,
            })
            .collect::<Vec<_>>();
        let mark_all_spill_facts = mark_all.spill_facts().values().cloned().collect::<Vec<_>>();
        let mark_all_dependencies = mark_all.runtime_dependency_graph().clone();
        let mark_all_dynamic_defined_names = GridDynamicDefinedNameRuntimeSnapshot::new(
            mark_all.dynamic_defined_name_keys().clone(),
            mark_all.dynamic_defined_name_extents().clone(),
            mark_all.dynamic_defined_name_dependencies().clone(),
            mark_all.volatile_dynamic_defined_names().clone(),
            mark_all.external_pending_dynamic_defined_names().clone(),
        );

        let dirty_spill_epoch_ledger = dirty.spill_epoch_ledger().clone();
        let mark_all_spill_epoch_ledger = mark_all.spill_epoch_ledger().clone();

        // FIX 4: seed the registry-effect axis with a real (not skipped)
        // comparison. `GridExternalAvailabilityTopicRegistry` is host-owned,
        // not sheet-owned (see design doc: hosts apply subscription updates
        // to their own registry instance), so neither `self` nor `previous`
        // carries a "prior registry state" this function could pull a richer
        // seed from. An empty seed is still a REAL, non-vacuous comparison,
        // unlike `None` (which short-circuits `compare_grid_dirty_recalc_
        // registry_effect` entirely and forces `registry_effect_equal =
        // true` unconditionally): both clones below start from the same
        // empty registry and apply only their own run's
        // `external_subscription_updates`, so this still catches a run that
        // emits a different (or missing) subscription-update list than its
        // counterpart for the same root/topic.
        let registry_effect_seed = GridExternalAvailabilityTopicRegistry::default();

        Ok(build_grid_dirty_recalc_differential_report(
            GridEngineMode::Optimized,
            GridEngineRecalcReport::Optimized(dirty_report.clone()),
            GridEngineRecalcReport::Optimized(mark_all_report.clone()),
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
            structural_dependency_edges: 0,
            overlay_dependency_edges: 0,
            dynamic_defined_name_evaluations: 0,
            external_subscription_updates: Vec::new(),
        };
        let mut prepared_templates = BTreeSet::new();
        self.populate_compact_literal_valuation(
            &mut valuation,
            &mut report,
            materialization_limit,
        )?;
        let dynamic_name_report = self.refresh_dynamic_defined_names_with_oxfml(
            &mut valuation,
            materialization_limit,
            None,
            false,
            false,
        )?;
        report.dynamic_defined_name_evaluations += dynamic_name_report.evaluations;
        report
            .external_subscription_updates
            .extend(dynamic_name_report.external_subscription_updates);

        if let Some(formula_cells) = self.final_formula_cell_count_if_worklist_sized(
            &valuation,
            materialization_limit,
            10_000,
        )? {
            let mut pending = BTreeSet::new();
            for (coord, point) in &self.sparse_points {
                let address = self.address_from_coord(*coord);
                if !self.final_source_matches(&address, GridOptimizedCellSource::SparsePoint) {
                    continue;
                }
                let Some(formula) = point.cell.formula_ref() else {
                    continue;
                };
                self.install_optimized_structural_dependencies_for_formula(
                    &mut valuation,
                    &address,
                    formula,
                    materialization_limit,
                )?;
                pending.insert(address);
            }
            for (region_index, region) in self.repeated_formula_regions.iter().enumerate() {
                let source = GridOptimizedCellSource::RepeatedFormulaRegion { region_index };
                for address in region.rect.scalar_cells(materialization_limit)? {
                    if !self.final_source_matches(&address, source) {
                        continue;
                    }
                    self.install_optimized_structural_dependencies_for_formula(
                        &mut valuation,
                        &address,
                        &region.formula,
                        materialization_limit,
                    )?;
                    pending.insert(address);
                }
            }

            let iteration_limit = formula_cells
                .max(1)
                .saturating_mul(formula_cells.max(1))
                .saturating_mul(4);
            let mut formula_iterations = 0usize;
            while !pending.is_empty() {
                let address = if let Some(address) = valuation
                    .runtime_dependencies
                    .next_ready_dirty_formula(&pending)
                {
                    address
                } else if let Some(address) = valuation
                    .runtime_dependencies
                    .first_pending_with_overlay_dependencies(&pending)
                {
                    address
                } else {
                    let cycle = pending
                        .iter()
                        .find_map(|address| {
                            valuation
                                .runtime_dependencies
                                .effective_dependency_cycle_from(address, &pending)
                        })
                        .unwrap_or_else(|| pending.iter().cloned().collect());
                    return Err(GridRefError::EffectiveDependencyCycleDetected { cycle });
                };
                pending.remove(&address);
                let Some((revision, GridAuthoredCell::Formula(formula), source)) =
                    self.versioned_authored_cell_at(&address)
                else {
                    continue;
                };

                formula_iterations += 1;
                if formula_iterations > iteration_limit {
                    return Err(GridRefError::IncrementalRecalcDidNotConverge { iteration_limit });
                }

                count_optimized_dirty_formula_evaluation(&mut report, source);
                register_formula_plan_cache_access(
                    &mut prepared_templates,
                    formula_plan_cache,
                    &formula,
                    None,
                    &mut report,
                    1,
                );
                self.install_optimized_structural_dependencies_for_formula(
                    &mut valuation,
                    &address,
                    &formula,
                    materialization_limit,
                )?;
                let outcome = self.evaluate_optimized_formula_with_spill_repair_outcome(
                    &address,
                    &formula,
                    &valuation,
                    materialization_limit,
                )?;
                report.external_subscription_updates.push(
                    GridExternalAvailabilitySubscriptionUpdate::formula_root(
                        address.clone(),
                        outcome.trace.external_subscriptions.clone(),
                    ),
                );
                valuation
                    .runtime_dependencies
                    .replace_overlay_dependencies_from_trace(address.clone(), &outcome.trace)?;
                if let Some(cycle) = valuation
                    .runtime_dependencies
                    .effective_dependency_cycle_from(&address, &pending)
                {
                    return Err(GridRefError::EffectiveDependencyCycleDetected { cycle });
                }
                let publication_delta = self.publish_formula_value_to_valuation(
                    &mut valuation,
                    address.clone(),
                    revision,
                    outcome.value,
                    source,
                );
                valuation
                    .runtime_dependencies
                    .refresh_overlay_spill_blocker_dependency(
                        address.clone(),
                        publication_delta.current_spill_blocker_extent.clone(),
                    )?;
                report.spill_facts_published += publication_delta.facts_published;
                report.spill_facts_blocked += publication_delta.facts_blocked;
                report.spill_ghost_cells_published += publication_delta.ghost_cells_published;

                let publication_dirty_seeds = publication_delta.dirty_seeds();
                let mut dirty_cells = valuation
                    .runtime_dependencies
                    .dirty_closure_for_seeds(publication_dirty_seeds.clone())?
                    .dirty_cells;
                let dynamic_names_to_refresh = dynamic_defined_name_keys_to_refresh(
                    &self.dynamic_defined_names.keys().cloned().collect(),
                    &valuation.dynamic_defined_name_dependencies,
                    &valuation.volatile_dynamic_defined_names,
                    &valuation.external_pending_dynamic_defined_names,
                    &publication_dirty_seeds,
                    self.bounds,
                    false,
                    false,
                )?;
                let dynamic_name_report = self.refresh_dynamic_defined_names_with_oxfml(
                    &mut valuation,
                    materialization_limit,
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
                        valuation
                            .runtime_dependencies
                            .dirty_closure_for_seeds(dynamic_name_report.dirty_seeds)?
                            .dirty_cells,
                    );
                }
                dirty_cells.remove(&address);
                for dirty_cell in dirty_cells {
                    if matches!(
                        self.versioned_authored_cell_at(&dirty_cell),
                        Some((_, GridAuthoredCell::Formula(_), _))
                    ) {
                        pending.insert(dirty_cell);
                    }
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
            report.structural_dependency_edges = valuation
                .runtime_dependencies
                .semantic_dependency_count_for_layer(GridDependencyLayer::Structural);
            report.overlay_dependency_edges = valuation
                .runtime_dependencies
                .semantic_dependency_count_for_layer(GridDependencyLayer::CalcOverlay);
            valuation.refresh_spill_epoch_ledger();
            self.refresh_optimized_report_spill_counters(&valuation, &mut report);
            // A full mark-all pass (this guarded sparse/small-repeated
            // fast path included) just committed a graph consistent with
            // every formula on the sheet, even if it has zero edges (a
            // sheet of only dependency-free formulas like `=1+1`/`=NOW()`).
            valuation.graph_installed = true;
            return Ok((valuation, report));
        }

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
            self.install_optimized_structural_dependencies_for_formula(
                &mut valuation,
                &address,
                formula,
                materialization_limit,
            )?;
            let outcome = self.evaluate_optimized_formula_with_spill_repair_outcome(
                &address,
                formula,
                &valuation,
                materialization_limit,
            )?;
            report.external_subscription_updates.push(
                GridExternalAvailabilitySubscriptionUpdate::formula_root(
                    address.clone(),
                    outcome.trace.external_subscriptions.clone(),
                ),
            );
            valuation
                .runtime_dependencies
                .replace_overlay_dependencies_from_trace(address.clone(), &outcome.trace)?;
            let publication_delta = self.publish_formula_value_to_valuation(
                &mut valuation,
                address.clone(),
                point.revision,
                outcome.value,
                GridOptimizedCellSource::SparsePoint,
            );
            valuation
                .runtime_dependencies
                .refresh_overlay_spill_blocker_dependency(
                    address.clone(),
                    publication_delta.current_spill_blocker_extent.clone(),
                )?;
            report.spill_facts_published += publication_delta.facts_published;
            report.spill_facts_blocked += publication_delta.facts_blocked;
            report.spill_ghost_cells_published += publication_delta.ghost_cells_published;
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
                self.install_optimized_structural_dependencies_for_formula(
                    &mut valuation,
                    &address,
                    &region.formula,
                    materialization_limit,
                )?;
                let outcome = self.evaluate_optimized_formula_with_spill_repair_outcome(
                    &address,
                    &region.formula,
                    &valuation,
                    materialization_limit,
                )?;
                report.external_subscription_updates.push(
                    GridExternalAvailabilitySubscriptionUpdate::formula_root(
                        address.clone(),
                        outcome.trace.external_subscriptions.clone(),
                    ),
                );
                valuation
                    .runtime_dependencies
                    .replace_overlay_dependencies_from_trace(address.clone(), &outcome.trace)?;
                let publication_delta = self.publish_formula_value_to_valuation(
                    &mut valuation,
                    address.clone(),
                    region.revision,
                    outcome.value,
                    source,
                );
                valuation
                    .runtime_dependencies
                    .refresh_overlay_spill_blocker_dependency(
                        address,
                        publication_delta.current_spill_blocker_extent.clone(),
                    )?;
                report.spill_facts_published += publication_delta.facts_published;
                report.spill_facts_blocked += publication_delta.facts_blocked;
                report.spill_ghost_cells_published += publication_delta.ghost_cells_published;
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
        report.structural_dependency_edges = valuation
            .runtime_dependencies
            .semantic_dependency_count_for_layer(GridDependencyLayer::Structural);
        report.overlay_dependency_edges = valuation
            .runtime_dependencies
            .semantic_dependency_count_for_layer(GridDependencyLayer::CalcOverlay);
        valuation.refresh_spill_epoch_ledger();
        self.refresh_optimized_report_spill_counters(&valuation, &mut report);
        // See the guarded fast-path return above: a completed mark-all
        // pass always commits a trustworthy graph, zero edges or not.
        valuation.graph_installed = true;
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
            structural_dependency_edges: 0,
            overlay_dependency_edges: 0,
            dynamic_defined_name_evaluations: 0,
            external_subscription_updates: Vec::new(),
        };
        let mut prepared_templates = BTreeSet::new();
        let mut formula_plan_cache = GridOptimizedFormulaPlanCache::default();

        // Phase 1: project every in-cone LITERAL first (sparse literals,
        // then dense-region literal subrects) so any in-cone formula reads a
        // fully-populated cone rather than a partially-blank one. An
        // in-cone formula reading an in-cone dense literal must see that
        // literal's value, not a blank precedent.
        for (coord, point) in &self.sparse_points {
            let address = self.address_from_coord(*coord);
            if !upstream_rect.contains(&address)
                || !self.final_source_matches(&address, GridOptimizedCellSource::SparsePoint)
            {
                continue;
            }
            let Some(value) = point.cell.literal_value() else {
                continue;
            };
            recalc.occupied_cells += 1;
            recalc.cells_evaluated += 1;
            recalc.literal_cells += 1;
            recalc.sparse_literal_cells += 1;
            valuation.insert_sparse_value(
                address.clone(),
                point.revision,
                value,
                GridOptimizedCellSource::SparsePoint,
            );
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

        // Phase 2: evaluate in-cone FORMULAS (sparse, then repeated-region
        // fallback) now that every in-cone literal is already installed.
        for (coord, point) in &self.sparse_points {
            let address = self.address_from_coord(*coord);
            if !upstream_rect.contains(&address)
                || !self.final_source_matches(&address, GridOptimizedCellSource::SparsePoint)
            {
                continue;
            }
            let Some(formula) = point.cell.formula_ref() else {
                continue;
            };
            recalc.occupied_cells += 1;
            recalc.cells_evaluated += 1;
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
            self.install_optimized_structural_dependencies_for_formula(
                &mut valuation,
                &address,
                formula,
                materialization_limit,
            )?;
            let outcome = self.evaluate_optimized_formula_with_spill_repair_outcome(
                &address,
                formula,
                &valuation,
                materialization_limit,
            )?;
            recalc.external_subscription_updates.push(
                GridExternalAvailabilitySubscriptionUpdate::formula_root(
                    address.clone(),
                    outcome.trace.external_subscriptions.clone(),
                ),
            );
            valuation
                .runtime_dependencies
                .replace_overlay_dependencies_from_trace(address.clone(), &outcome.trace)?;
            if let Some(cycle) = valuation
                .runtime_dependencies
                .effective_dependency_cycle_from(&address, &BTreeSet::new())
            {
                return Err(GridRefError::EffectiveDependencyCycleDetected { cycle });
            }
            let publication_delta = self.publish_formula_value_to_valuation(
                &mut valuation,
                address.clone(),
                point.revision,
                outcome.value,
                GridOptimizedCellSource::SparsePoint,
            );
            valuation
                .runtime_dependencies
                .refresh_overlay_spill_blocker_dependency(
                    address.clone(),
                    publication_delta.current_spill_blocker_extent.clone(),
                )?;
            recalc.spill_facts_published += publication_delta.facts_published;
            recalc.spill_facts_blocked += publication_delta.facts_blocked;
            recalc.spill_ghost_cells_published += publication_delta.ghost_cells_published;
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
                self.install_optimized_structural_dependencies_for_formula(
                    &mut valuation,
                    &address,
                    &region.formula,
                    materialization_limit,
                )?;
                let outcome = self.evaluate_optimized_formula_with_spill_repair_outcome(
                    &address,
                    &region.formula,
                    &valuation,
                    materialization_limit,
                )?;
                recalc.external_subscription_updates.push(
                    GridExternalAvailabilitySubscriptionUpdate::formula_root(
                        address.clone(),
                        outcome.trace.external_subscriptions.clone(),
                    ),
                );
                valuation
                    .runtime_dependencies
                    .replace_overlay_dependencies_from_trace(address.clone(), &outcome.trace)?;
                if let Some(cycle) = valuation
                    .runtime_dependencies
                    .effective_dependency_cycle_from(&address, &BTreeSet::new())
                {
                    return Err(GridRefError::EffectiveDependencyCycleDetected { cycle });
                }
                let publication_delta = self.publish_formula_value_to_valuation(
                    &mut valuation,
                    address.clone(),
                    region.revision,
                    outcome.value,
                    source,
                );
                valuation
                    .runtime_dependencies
                    .refresh_overlay_spill_blocker_dependency(
                        address.clone(),
                        publication_delta.current_spill_blocker_extent.clone(),
                    )?;
                recalc.spill_facts_published += publication_delta.facts_published;
                recalc.spill_facts_blocked += publication_delta.facts_blocked;
                recalc.spill_ghost_cells_published += publication_delta.ghost_cells_published;
            }
        }

        recalc.formula_templates_prepared = prepared_templates.len();
        recalc.distinct_formula_templates = prepared_templates.len();
        formula_plan_cache.prune_to_templates(&prepared_templates);
        recalc.compiled_formula_plans_cached = formula_plan_cache.cached_compiled_plan_count();
        recalc.computed_dense_value_regions = valuation.dense_value_regions().len();
        recalc.computed_sparse_cells = valuation.sparse_computed_cells();
        recalc.structural_dependency_edges = valuation
            .runtime_dependencies
            .semantic_dependency_count_for_layer(GridDependencyLayer::Structural);
        recalc.overlay_dependency_edges = valuation
            .runtime_dependencies
            .semantic_dependency_count_for_layer(GridDependencyLayer::CalcOverlay);
        valuation.refresh_spill_epoch_ledger();
        self.refresh_optimized_report_spill_counters(&valuation, &mut recalc);
        valuation.set_visible_projection_coverage(upstream_rect.clone());

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
        // A partial (visible-projection) valuation never warm-skips: it does
        // not reflect the whole sheet, so it must never be handed back as a
        // trusted "nothing changed" result.
        if !cache.valuation.is_full_coverage() {
            return None;
        }
        if cache.valuation.runtime_dependencies.has_volatile_roots() {
            return None;
        }
        if !cache.valuation.volatile_dynamic_defined_names.is_empty() {
            return None;
        }
        if cache
            .valuation
            .runtime_dependencies
            .has_external_pending_roots()
        {
            return None;
        }
        if !cache
            .valuation
            .external_pending_dynamic_defined_names
            .is_empty()
        {
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

    pub(super) fn repair_optimized_spills_with_oxfml(
        &self,
        valuation: &mut GridOptimizedValuation,
        report: &mut GridOptimizedRecalcReport,
        materialization_limit: u64,
    ) -> Result<(), GridRefError> {
        let formula_cells = usize::try_from(report.formula_cells).unwrap_or(usize::MAX);
        if formula_cells == 0
            || valuation.spill_facts == self.overlays.spill_facts
            || !self.contains_grid_spill_reference_formula(materialization_limit)?
        {
            return Ok(());
        }

        report.spill_repair_converged = false;
        for _ in 0..formula_cells {
            let spill_facts_before = valuation.spill_facts.clone();
            // B5 (reference-engine twin fix applied here too): a pass can
            // leave spill_facts unchanged while still publishing a changed
            // *plain* value (e.g. a scalar formula that reads a later
            // spill-anchor's contents and only settles once that anchor
            // itself has been repaired earlier in the same pass).
            // Convergence must require both spill_facts stability AND
            // published-value stability across the whole pass; otherwise a
            // later precedent's fresh value can go unread by an
            // already-finalized plain consumer.
            let sparse_before = valuation.sparse.clone();
            // B5 parity fix: see the fuller rationale at the bottom of this
            // loop where it is compared. Captured here alongside the other
            // two "before" snapshots so all three reflect the same instant.
            let spill_value_fingerprints_before = valuation.spill_value_fingerprints.clone();
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
                let outcome = self.evaluate_optimized_formula_with_spill_repair_outcome(
                    &address,
                    formula,
                    valuation,
                    materialization_limit,
                )?;
                report.external_subscription_updates.push(
                    GridExternalAvailabilitySubscriptionUpdate::formula_root(
                        address.clone(),
                        outcome.trace.external_subscriptions.clone(),
                    ),
                );
                valuation
                    .runtime_dependencies
                    .replace_overlay_dependencies_from_trace(address.clone(), &outcome.trace)?;
                let publication_delta = self.publish_formula_value_to_valuation(
                    valuation,
                    address.clone(),
                    point.revision,
                    outcome.value,
                    GridOptimizedCellSource::SparsePoint,
                );
                valuation
                    .runtime_dependencies
                    .refresh_overlay_spill_blocker_dependency(
                        address.clone(),
                        publication_delta.current_spill_blocker_extent,
                    )?;
            }

            for (region_index, region) in self.repeated_formula_regions.iter().enumerate() {
                let source = GridOptimizedCellSource::RepeatedFormulaRegion { region_index };
                for address in region.rect.scalar_cells(materialization_limit)? {
                    if !self.final_source_matches(&address, source) {
                        continue;
                    }
                    report.spill_repair_formula_evaluations += 1;
                    let outcome = self.evaluate_optimized_formula_with_spill_repair_outcome(
                        &address,
                        &region.formula,
                        valuation,
                        materialization_limit,
                    )?;
                    report.external_subscription_updates.push(
                        GridExternalAvailabilitySubscriptionUpdate::formula_root(
                            address.clone(),
                            outcome.trace.external_subscriptions.clone(),
                        ),
                    );
                    valuation
                        .runtime_dependencies
                        .replace_overlay_dependencies_from_trace(address.clone(), &outcome.trace)?;
                    let publication_delta = self.publish_formula_value_to_valuation(
                        valuation,
                        address.clone(),
                        region.revision,
                        outcome.value,
                        source,
                    );
                    valuation
                        .runtime_dependencies
                        .refresh_overlay_spill_blocker_dependency(
                            address,
                            publication_delta.current_spill_blocker_extent,
                        )?;
                }
            }

            // B5 parity fix: `spill_facts` alone (anchor/extent/blocked) does
            // NOT capture a same-extent, same-blocked-status VALUE change to
            // a published spill array — `GridSpillFact` carries no value
            // fingerprint. A repeated-region/dense formula that republishes
            // a spilling array at an unchanged extent (e.g. reading a
            // later-settling precedent through `#`, mirroring the
            // same-extent value-only case `grid_calc_ref_dirty_recalc_
            // reaches_spill_anchor_consumer_on_same_extent_value_change`
            // covers for ordinary dirty recalc) writes through
            // `push_dense_value_payload` into `dense_value_regions`, not
            // through `insert_sparse_value` into `sparse`, so neither half
            // of the original `spill_facts`/`sparse` check would see it,
            // and the reference-engine twin (which compares the FULL
            // `computed` map) could diverge from a premature "converged"
            // verdict here. `spill_value_fingerprints` already carries a
            // per-anchor value hash for every published array (see
            // `publish_formula_value_to_valuation`), updated on every
            // publish regardless of extent stability, so comparing it is as
            // cheap as the existing two BTreeMap/struct comparisons and
            // closes the gap without a full cell-by-cell readout. This can
            // only make convergence MORE conservative (worst case, one more
            // pass), still bounded by the same `formula_cells` pass limit
            // the reference engine's `computed`-equality check uses.
            if valuation.spill_facts == spill_facts_before
                && valuation.sparse == sparse_before
                && valuation.spill_value_fingerprints == spill_value_fingerprints_before
            {
                report.spill_repair_converged = true;
                break;
            }
        }

        Ok(())
    }

    pub(super) fn contains_grid_spill_reference_formula(
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

    pub(super) fn visible_same_row_left_upstream_rect(
        &self,
        visible_rect: &GridRect,
    ) -> Result<GridRect, GridRefError> {
        let mut left_col = visible_rect.left_col;
        for region in &self.repeated_formula_regions {
            if !grid_rects_overlap(&region.rect, visible_rect) {
                continue;
            }
            // Derive the upstream requirement from the COMPILED plan (same
            // gate `try_evaluate_repeated_formula_visible_subrect` uses to
            // decide whether it can fast-path this region), not from a
            // literal source_text comparison: a whitespace/case variant of
            // the same formula (e.g. "=RC[-1] * 2") compiles to the same
            // plan and must size the cone identically, or the fast path
            // below silently evaluates with a truncated upstream input.
            if same_row_left_input_cols(&region.formula) > 0 {
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

    pub(super) fn try_evaluate_repeated_formula_visible_subrect(
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
        for address in subrect.scalar_cells(materialization_limit)? {
            self.install_optimized_structural_dependencies_for_formula(
                valuation,
                &address,
                &region.formula,
                materialization_limit,
            )?;
        }
        valuation.push_dense_value_payload(
            subrect.clone(),
            GridDenseValuePayload::from_numbers(values),
            region.revision,
            source,
        );
        Ok(true)
    }

    pub(super) fn try_evaluate_repeated_formula_region_fast_path(
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
        for address in region.rect.scalar_cells(materialization_limit)? {
            self.install_optimized_structural_dependencies_for_formula(
                valuation,
                &address,
                &region.formula,
                materialization_limit,
            )?;
        }
        valuation.push_dense_value_payload(
            region.rect.clone(),
            GridDenseValuePayload::from_calc_values(values),
            region.revision,
            source,
        );
        Ok(true)
    }

    pub(super) fn evaluate_optimized_formula_with_spill_repair_outcome(
        &self,
        address: &ExcelGridCellAddress,
        formula: &GridFormulaCell,
        valuation: &GridOptimizedValuation,
        materialization_limit: u64,
    ) -> Result<GridFormulaEvaluationOutcome, GridRefError> {
        if let Some(value) = evaluate_optimized_formula_fast_path(address, formula, valuation) {
            return Ok(GridFormulaEvaluationOutcome {
                value,
                trace: GridRuntimeDependencyTrace::default(),
            });
        }
        match self.evaluate_optimized_formula_with_oxfml(
            address,
            formula,
            valuation,
            materialization_limit,
        ) {
            Ok(outcome) => Ok(outcome),
            Err(error) => {
                let profile = StrictExcelGridReferenceProfile::with_bounds(self.bounds);
                if formula_contains_grid_spill_reference(formula, address, &profile, self.bounds) {
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

    fn install_optimized_structural_dependencies_for_formula(
        &self,
        valuation: &mut GridOptimizedValuation,
        address: &ExcelGridCellAddress,
        formula: &GridFormulaCell,
        materialization_limit: u64,
    ) -> Result<(), GridRefError> {
        let structural_dependencies = {
            let provider = valuation.reference_system_provider_with_dense_materialization_limit(
                address.row,
                address.col,
                materialization_limit,
            );
            let profile = StrictExcelGridReferenceProfile::with_bounds(self.bounds);
            grid_structural_dependencies_for_formula(
                formula,
                address,
                &profile,
                self.bounds,
                &provider,
            )
        };
        valuation
            .runtime_dependencies
            .set_structural_dependencies(address.clone(), structural_dependencies)
            .map(|_| ())
    }

    fn apply_dirty_cells_to_optimized_worklist(
        &self,
        valuation: &mut GridOptimizedValuation,
        dirty_cells: &BTreeSet<ExcelGridCellAddress>,
        applied_literals: &mut BTreeSet<ExcelGridCellAddress>,
        pending: &mut BTreeSet<ExcelGridCellAddress>,
        report: &mut GridOptimizedRecalcReport,
    ) -> Result<(), GridRefError> {
        let mut vacated_dirty_seeds = BTreeSet::new();
        for address in dirty_cells {
            match self.versioned_authored_cell_at(address) {
                Some((revision, GridAuthoredCell::Literal(value), source)) => {
                    if applied_literals.insert(address.clone()) {
                        let clear_report = valuation.clear_formula_output_for_anchor(address);
                        if clear_report.had_spill_fact {
                            vacated_dirty_seeds.extend(grid_vacated_spill_extent_dirty_seeds(
                                address,
                                &clear_report.old_extent,
                            ));
                        }
                        valuation.insert_sparse_value(address.clone(), revision, value, source);
                        valuation
                            .runtime_dependencies
                            .set_structural_dependencies(address.clone(), Vec::new())?;
                        valuation
                            .runtime_dependencies
                            .clear_overlay_dependencies(address)?;
                        count_optimized_dirty_literal_evaluation(report, source);
                    }
                }
                Some((_, GridAuthoredCell::Formula(_), _)) => {
                    pending.insert(address.clone());
                }
                None => {}
            }
        }
        if !vacated_dirty_seeds.is_empty() {
            let vacated_dirty_cells = valuation
                .runtime_dependencies
                .dirty_closure_for_seeds(vacated_dirty_seeds)?
                .dirty_cells;
            self.apply_dirty_cells_to_optimized_worklist(
                valuation,
                &vacated_dirty_cells,
                applied_literals,
                pending,
                report,
            )?;
        }
        Ok(())
    }

    fn versioned_authored_cell_at(
        &self,
        address: &ExcelGridCellAddress,
    ) -> Option<(u64, GridAuthoredCell, GridOptimizedCellSource)> {
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

        Some((best_revision, best_cell?, best_source?))
    }

    fn final_formula_cell_count(&self, materialization_limit: u64) -> Result<usize, GridRefError> {
        let mut count = 0usize;
        for (coord, point) in &self.sparse_points {
            let address = self.address_from_coord(*coord);
            if point.cell.formula_ref().is_some()
                && self.final_source_matches(&address, GridOptimizedCellSource::SparsePoint)
            {
                count = count.saturating_add(1);
            }
        }
        for (region_index, region) in self.repeated_formula_regions.iter().enumerate() {
            let source = GridOptimizedCellSource::RepeatedFormulaRegion { region_index };
            for address in region.rect.scalar_cells(materialization_limit)? {
                if self.final_source_matches(&address, source) {
                    count = count.saturating_add(1);
                }
            }
        }
        Ok(count)
    }

    fn final_formula_cell_count_if_worklist_sized(
        &self,
        valuation: &GridOptimizedValuation,
        materialization_limit: u64,
        worklist_limit: usize,
    ) -> Result<Option<usize>, GridRefError> {
        let mut count = 0usize;
        let mut pending = BTreeSet::new();
        // Mirrors the actual compact execution order of
        // `recalculate_mark_all_dirty_compact_with_oxfml_using_formula_plan_cache`:
        // ALL final sparse-formula addresses in coord order, THEN each
        // repeated region's cells in rect order (region by region), NOT
        // interleaved global address order. The probe below must simulate
        // this exact sequence, not `pending`'s BTreeSet (address) order.
        let mut compact_execution_order = Vec::new();
        let mut has_sparse_formula = false;
        let mut has_repeated_formula_region = false;
        for (coord, point) in &self.sparse_points {
            let address = self.address_from_coord(*coord);
            if point.cell.formula_ref().is_some()
                && self.final_source_matches(&address, GridOptimizedCellSource::SparsePoint)
            {
                has_sparse_formula = true;
                count = count.saturating_add(1);
                if count > worklist_limit {
                    return Ok(None);
                }
                pending.insert(address.clone());
                compact_execution_order.push(address);
            }
        }
        for (region_index, region) in self.repeated_formula_regions.iter().enumerate() {
            if region.rect.cell_count() > materialization_limit
                || region.rect.cell_count() > u64::try_from(worklist_limit).unwrap_or(u64::MAX)
            {
                return Ok(None);
            }
            let source = GridOptimizedCellSource::RepeatedFormulaRegion { region_index };
            for address in region.rect.scalar_cells(materialization_limit)? {
                if !self.final_source_matches(&address, source) {
                    continue;
                }
                has_repeated_formula_region = true;
                count = count.saturating_add(1);
                if count > worklist_limit {
                    return Ok(None);
                }
                pending.insert(address.clone());
                compact_execution_order.push(address);
            }
        }
        if !has_repeated_formula_region {
            return Ok(Some(count));
        }

        let mut probe = valuation.clone();
        for address in &pending {
            let Some((_, GridAuthoredCell::Formula(formula), _)) =
                self.versioned_authored_cell_at(address)
            else {
                continue;
            };
            self.install_optimized_structural_dependencies_for_formula(
                &mut probe,
                address,
                &formula,
                materialization_limit,
            )?;
        }

        // Walk the ACTUAL compact execution sequence and check, at each
        // step, whether the next cell the compact path would evaluate is
        // actually ready (no outstanding pending precedent) given what the
        // compact path has evaluated so far. Any out-of-order evaluation —
        // e.g. a sparse formula depending on a not-yet-evaluated
        // repeated-region cell, or a repeated-region cell depending on a
        // later cell in the same or another region — disapproves compact.
        let mut remaining_pending: BTreeSet<ExcelGridCellAddress> = compact_execution_order
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        for next_in_execution_order in &compact_execution_order {
            if probe
                .runtime_dependencies
                .is_formula_ready(next_in_execution_order, &remaining_pending)
            {
                remaining_pending.remove(next_in_execution_order);
                continue;
            }
            return if !has_sparse_formula
                && self
                    .repeated_formula_regions_use_only_reference_metadata(materialization_limit)?
            {
                Ok(None)
            } else {
                Ok(Some(count))
            };
        }
        Ok(None)
    }

    fn repeated_formula_regions_use_only_reference_metadata(
        &self,
        materialization_limit: u64,
    ) -> Result<bool, GridRefError> {
        let mut saw_repeated_formula = false;
        for (region_index, region) in self.repeated_formula_regions.iter().enumerate() {
            let source = GridOptimizedCellSource::RepeatedFormulaRegion { region_index };
            let mut has_final_formula_cell = false;
            for address in region.rect.scalar_cells(materialization_limit)? {
                if self.final_source_matches(&address, source) {
                    has_final_formula_cell = true;
                    break;
                }
            }
            if !has_final_formula_cell {
                continue;
            }
            saw_repeated_formula = true;
            let Some(plan) = GridOptimizedCompiledFormulaPlan::compile(&region.formula) else {
                return Ok(false);
            };
            if !r1c1_compiled_plan_uses_only_reference_metadata(&plan) {
                return Ok(false);
            }
        }
        Ok(saw_repeated_formula)
    }

    fn check_valuation_identity(
        &self,
        valuation: &GridOptimizedValuation,
    ) -> Result<(), GridRefError> {
        if valuation.workbook_id == self.workbook_id
            && valuation.sheet_id == self.sheet_id
            && valuation.bounds == self.bounds
        {
            return Ok(());
        }
        Err(GridRefError::ValuationGridIdentityMismatch {
            expected_workbook_id: self.workbook_id.clone(),
            expected_sheet_id: self.sheet_id.clone(),
            expected_bounds: self.bounds,
            actual_workbook_id: valuation.workbook_id.clone(),
            actual_sheet_id: valuation.sheet_id.clone(),
            actual_bounds: valuation.bounds,
        })
    }

    pub(super) fn refresh_optimized_report_spill_counters(
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
        reference.overlays.spill_facts = self.overlays.spill_facts.clone();
        reference.spill_value_fingerprints = self.spill_value_fingerprints.clone();
        reference.spill_epoch_ledger = self.spill_epoch_ledger.clone();
        reference.defined_names = self.defined_names.clone();
        reference.dynamic_defined_names = self.dynamic_defined_names.clone();
        reference.dynamic_defined_name_extents = self.dynamic_defined_name_extents.clone();
        reference.overlays.table_overlays = self.overlays.table_overlays.clone();
        reference.overlays.merged_regions = self.overlays.merged_regions.clone();
        reference.overlays.feature_rendered_regions =
            self.overlays.feature_rendered_regions.clone();
        for (address, cell) in authored {
            match cell.cell.to_authored() {
                GridAuthoredCell::Literal(value) => reference.set_literal(address, value)?,
                GridAuthoredCell::Formula(formula) => reference.set_formula(address, formula)?,
            }
        }
        Ok(reference)
    }

    pub(super) fn populate_compact_literal_valuation(
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

    pub(super) fn evaluate_optimized_formula_with_oxfml(
        &self,
        address: &ExcelGridCellAddress,
        formula: &GridFormulaCell,
        valuation: &GridOptimizedValuation,
        materialization_limit: u64,
    ) -> Result<GridFormulaEvaluationOutcome, GridRefError> {
        let provider = valuation.reference_system_provider_with_dense_materialization_limit(
            address.row,
            address.col,
            materialization_limit,
        );
        let tracing_provider = GridTracingReferenceSystemProvider::new(&provider);
        // Built from the LIVE in-progress `valuation` (not
        // `self.overlays`, which is the pre-recalc committed baseline):
        // hidden-row-aware aggregates (SUBTOTAL/AGGREGATE) over a spill
        // reference, defined name, dynamic-name extent, or structured/table
        // reference must see facts as they are published mid-recalc, the
        // same way the reference engine's `self` (which mutates in place)
        // does. `GridHostInfoProvider::new` skips blocked spill facts
        // internally.
        let host_info = GridHostInfoProvider::new(
            self.workbook_id.clone(),
            self.sheet_id.clone(),
            address.row,
            address.col,
            self.bounds,
            valuation.spill_facts.values(),
            &valuation.defined_names,
            &valuation.dynamic_defined_name_extents,
            valuation.table_overlays.values(),
            &self.axis_state,
        );
        let query_bundle = TypedContextQueryBundle::new(
            Some(&host_info as &dyn HostInfoProvider),
            None,
            None,
            None,
            None,
        )
        .with_reference_system_provider(Some(&tracing_provider as &dyn ReferenceSystemProvider));
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
                "grid-optimized:{}:{}:{}x{}",
                self.workbook_id, self.sheet_id, self.bounds.max_rows, self.bounds.max_cols
            )))
            .with_table_context(
                grid_table_descriptor_catalog(self.overlays.table_overlays.values()),
                enclosing_table_ref,
                caller_table_region,
            )
            .with_reference_bind_profile(&profile);
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
        let bound = bind_grid_formula_for_transform(formula, address, &profile, self.bounds);
        trace.runtime_realized_dependencies_are_metadata_only =
            grid_formula_runtime_realized_dependencies_are_metadata_only(&bound);
        // G5(b): see the matching comment in calc_ref_sheet.rs. A hidden-
        // row-sensitive aggregate over a text-realized target gets no
        // AxisVisibility coverage from the structural feeder, so derive it
        // from the trace's realized dependencies when the bound tree
        // contains SUBTOTAL/AGGREGATE anywhere.
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

    pub(super) fn publish_formula_value_to_valuation(
        &self,
        valuation: &mut GridOptimizedValuation,
        address: ExcelGridCellAddress,
        revision: u64,
        value: CalcValue,
        source: GridOptimizedCellSource,
    ) -> GridValuePublicationDelta {
        let old_fact = valuation.spill_facts.get(&address).cloned();
        let old_readout = valuation.read_cell(&address);
        let old_effective_cells = grid_formula_output_cells_before_publication(
            &address,
            old_fact.as_ref(),
            old_readout.source.is_some(),
        );
        valuation.clear_formula_output_for_anchor(&address);

        let Some(array) = value.as_array() else {
            valuation.insert_sparse_value(address.clone(), revision, value, source);
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
            valuation.insert_sparse_value(
                address.clone(),
                revision,
                CalcValue::error(WorksheetErrorCode::Spill),
                source,
            );
            valuation
                .spill_value_fingerprints
                .insert(address.clone(), blocked_spill_value_fingerprint(array));
            let new_fact = GridSpillFact {
                anchor: address.clone(),
                extent: anchor_cell_rect(&address, self.bounds),
                blocked: true,
            };
            valuation
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
            let new_fact = GridSpillFact {
                anchor: address.clone(),
                extent,
                blocked: true,
            };
            valuation
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

        let new_effective_cells = grid_formula_output_cells_for_extent(&extent);
        valuation.push_dense_value_payload(
            extent.clone(),
            GridDenseValuePayload::from_calc_array(array),
            revision,
            source,
        );
        let new_fact = GridSpillFact {
            anchor: address.clone(),
            extent,
            blocked: false,
        };
        valuation
            .spill_facts
            .insert(address.clone(), new_fact.clone());
        valuation
            .spill_value_fingerprints
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

    pub(super) fn optimized_spill_extent_is_blocked(
        &self,
        anchor: &ExcelGridCellAddress,
        extent: &GridRect,
        valuation: &GridOptimizedValuation,
    ) -> bool {
        self.overlay_set_blockage_probe(anchor, extent, &valuation.spill_facts)
            .is_ok_and(|report| report.blocked)
    }

    pub fn optimized_spill_blockage_probe_report(
        &self,
        anchor: &ExcelGridCellAddress,
        extent: &GridRect,
    ) -> Result<GridOptimizedSpillBlockageProbeReport, GridRefError> {
        self.overlay_set_blockage_probe(anchor, extent, &self.overlays.spill_facts)
    }

    /// The legacy per-type blockage probe, retained as the **reference oracle**
    /// for the overlay-blockage equivalence guard (the OVL-2/3 test). Production
    /// blockage now routes through
    /// [`overlay_set_blockage_probe`](Self::overlay_set_blockage_probe); this
    /// brute-force per-type version stays as the differential reference the
    /// unified probe is checked against.
    #[allow(dead_code)]
    pub(super) fn optimized_spill_blockage_probe_report_with_facts(
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

        for region in &self.overlays.merged_regions {
            if grid_rects_overlap(&region.rect, extent)
                && rects_overlap_outside_anchor(&region.rect, extent, anchor)
            {
                report.merged_region_candidates += 1;
                report.blocked = true;
            }
        }

        for region in &self.overlays.feature_rendered_regions {
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

    /// The overlay set for spill-blockage probing: every committed table, merged
    /// region, and feature-rendered region on the sheet, plus the supplied spill
    /// facts. (Sparse points, dense and repeated-formula regions are value
    /// storage, not overlays; OVL-3 folds them in via an internal shim.)
    pub(super) fn overlay_set_for_spill_facts(
        &self,
        spill_facts: &BTreeMap<ExcelGridCellAddress, GridSpillFact>,
    ) -> Vec<GridOverlay> {
        let mut overlays = Vec::new();
        for table in self.overlays.table_overlays.values() {
            overlays.push(GridOverlay::Table(table.clone()));
        }
        for region in &self.overlays.merged_regions {
            overlays.push(GridOverlay::Merged(region.clone()));
        }
        for region in &self.overlays.feature_rendered_regions {
            overlays.push(GridOverlay::FeatureRendered(region.clone()));
        }
        for fact in spill_facts.values() {
            overlays.push(GridOverlay::Spill(fact.clone()));
        }
        overlays
    }

    /// The production spill-blockage probe (OVL-3): an overlay-set re-expression
    /// of the legacy per-type probe, checked against it by the equivalence guard
    /// ([`optimized_spill_blockage_probe_report_with_facts`](Self::optimized_spill_blockage_probe_report_with_facts)
    /// is the retained reference oracle). The blocked-formula anchor-containment
    /// pre-pass and the value-storage payload loops stay inline; only the merged
    /// / feature / unblocked-spill blockers route through the unified overlay set.
    pub(super) fn overlay_set_blockage_probe(
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

        // Blocked-formula anchor-containment pre-pass (a spill-only geometry, not
        // an overlap-outside-anchor test).
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

        // Value-storage payload (kept inline; OVL-3 folds these in via a shim).
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

        // The unified overlay set: merged regions, spill-blocking features, and
        // other live spills. A table blocks only through its companion feature
        // region (`SpillBlock::None`), so it contributes nothing here.
        for overlay in self.overlay_set_for_spill_facts(spill_facts) {
            if overlay.blocks_spill() == SpillBlock::None {
                continue;
            }
            // A published spill never blocks its own re-evaluation.
            if let GridOverlay::Spill(fact) = &overlay {
                if fact.anchor == *anchor {
                    continue;
                }
            }
            let blocks = overlay.claimed_rects().iter().any(|rect| {
                grid_rects_overlap(rect, extent)
                    && rects_overlap_outside_anchor(rect, extent, anchor)
            });
            if blocks {
                match overlay.kind() {
                    OverlayKind::Merged => report.merged_region_candidates += 1,
                    OverlayKind::FeatureRendered => report.feature_rendered_region_candidates += 1,
                    OverlayKind::Spill => report.unblocked_spill_fact_candidates += 1,
                    _ => {}
                }
                report.blocked = true;
            }
        }

        Ok(report)
    }

    pub(super) fn overlay_dense_regions(
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

    pub(super) fn overlay_repeated_formula_regions(
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

    pub(super) fn final_source_matches(
        &self,
        address: &ExcelGridCellAddress,
        source: GridOptimizedCellSource,
    ) -> bool {
        self.authored_cell_at(address)
            .map_or(false, |readout| readout.source == Some(source))
    }

    pub(super) fn address_from_coord(&self, coord: GridCellCoord) -> ExcelGridCellAddress {
        ExcelGridCellAddress::new(
            self.workbook_id.clone(),
            self.sheet_id.clone(),
            coord.row,
            coord.col,
        )
    }

    pub(super) fn warm_noop_token(&self, materialization_limit: u64) -> GridOptimizedWarmNoOpToken {
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
            merged_regions: self.overlays.merged_regions.clone(),
            feature_rendered_regions: self.overlays.feature_rendered_regions.clone(),
            spill_facts: self
                .overlays
                .spill_facts
                .iter()
                .map(|(address, fact)| (address.clone(), fact.clone()))
                .collect(),
            defined_names: self
                .defined_names
                .iter()
                .map(|(name, rect)| (name.clone(), rect.clone()))
                .collect(),
            dynamic_defined_names: self
                .dynamic_defined_names
                .iter()
                .map(|(name, definition)| (name.clone(), definition.clone()))
                .collect(),
            dynamic_defined_name_extents: self
                .dynamic_defined_name_extents
                .iter()
                .map(|(name, rect)| (name.clone(), rect.clone()))
                .collect(),
            table_overlays: self
                .overlays
                .table_overlays
                .iter()
                .map(|(table, overlay)| (table.clone(), overlay.clone()))
                .collect(),
        }
    }

    pub(super) fn allocate_revision(&mut self) -> u64 {
        let revision = self.next_revision;
        self.next_revision = self.next_revision.saturating_add(1);
        revision
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

/// Number of same-row, one-column-left upstream input columns a repeated
/// R1C1 formula region needs, derived from its COMPILED plan rather than a
/// raw source_text comparison. This is the same plan shape gate
/// `try_evaluate_repeated_formula_visible_subrect` uses to decide whether it
/// can fast-path a region (`plan == GridOptimizedCompiledFormulaPlan::
/// r1c1_double_left()`); whitespace/case variants of the formula compile to
/// the same plan and must be recognized identically here so the visible-cone
/// sizing and the fast-path evaluator never disagree about what upstream
/// input the region needs. Returns 0 when the formula either does not
/// compile to a recognized plan or does not need a same-row-left input.
fn same_row_left_input_cols(formula: &GridFormulaCell) -> u32 {
    if formula.source_channel != FormulaChannelKind::WorksheetR1C1 {
        return 0;
    }
    match GridOptimizedCompiledFormulaPlan::compile(formula) {
        Some(plan) if plan == GridOptimizedCompiledFormulaPlan::r1c1_double_left() => 1,
        _ => 0,
    }
}

fn r1c1_compiled_plan_uses_only_reference_metadata(
    plan: &GridOptimizedCompiledFormulaPlan,
) -> bool {
    match plan {
        GridOptimizedCompiledFormulaPlan::R1C1Scalar(expression) => {
            r1c1_scalar_expression_uses_only_reference_metadata(expression)
        }
        GridOptimizedCompiledFormulaPlan::R1C1Binary(plan) => {
            r1c1_scalar_expression_uses_only_reference_metadata(&plan.left)
                && r1c1_scalar_expression_uses_only_reference_metadata(&plan.right)
        }
        GridOptimizedCompiledFormulaPlan::R1C1If(plan) => {
            r1c1_logical_expression_uses_only_reference_metadata(&plan.condition)
                && r1c1_scalar_expression_uses_only_reference_metadata(&plan.when_true)
                && r1c1_scalar_expression_uses_only_reference_metadata(&plan.when_false)
        }
        GridOptimizedCompiledFormulaPlan::R1C1IfError(plan) => {
            r1c1_scalar_expression_uses_only_reference_metadata(&plan.value)
                && r1c1_scalar_expression_uses_only_reference_metadata(&plan.fallback)
        }
        GridOptimizedCompiledFormulaPlan::R1C1LogicalFunction(plan) => plan
            .arguments
            .iter()
            .all(r1c1_logical_expression_uses_only_reference_metadata),
        GridOptimizedCompiledFormulaPlan::R1C1Comparison(plan) => {
            r1c1_comparison_uses_only_reference_metadata(&plan.comparison)
        }
        GridOptimizedCompiledFormulaPlan::R1C1RangeAggregate(_)
        | GridOptimizedCompiledFormulaPlan::R1C1TextFunction(_)
        | GridOptimizedCompiledFormulaPlan::R1C1Index(_) => false,
    }
}

fn r1c1_scalar_expression_uses_only_reference_metadata(
    expression: &GridOptimizedR1C1ScalarExpression,
) -> bool {
    match expression {
        GridOptimizedR1C1ScalarExpression::Operand(GridOptimizedR1C1Operand::Number(_)) => true,
        GridOptimizedR1C1ScalarExpression::Operand(GridOptimizedR1C1Operand::Ref(_)) => false,
        GridOptimizedR1C1ScalarExpression::UnaryMinus(plan) => {
            r1c1_scalar_expression_uses_only_reference_metadata(&plan.value)
        }
        GridOptimizedR1C1ScalarExpression::Binary(plan) => {
            r1c1_scalar_expression_uses_only_reference_metadata(&plan.left)
                && r1c1_scalar_expression_uses_only_reference_metadata(&plan.right)
        }
        GridOptimizedR1C1ScalarExpression::ScalarFunction(plan) => plan
            .arguments
            .iter()
            .all(r1c1_scalar_expression_uses_only_reference_metadata),
        GridOptimizedR1C1ScalarExpression::ReferenceFunction(_) => true,
        GridOptimizedR1C1ScalarExpression::If(plan) => {
            r1c1_logical_expression_uses_only_reference_metadata(&plan.condition)
                && r1c1_scalar_expression_uses_only_reference_metadata(&plan.when_true)
                && r1c1_scalar_expression_uses_only_reference_metadata(&plan.when_false)
        }
        GridOptimizedR1C1ScalarExpression::IfError(plan) => {
            r1c1_scalar_expression_uses_only_reference_metadata(&plan.value)
                && r1c1_scalar_expression_uses_only_reference_metadata(&plan.fallback)
        }
        GridOptimizedR1C1ScalarExpression::RangeAggregate(_)
        | GridOptimizedR1C1ScalarExpression::ArgumentAggregate(_)
        | GridOptimizedR1C1ScalarExpression::TextFunction(_)
        | GridOptimizedR1C1ScalarExpression::Index(_)
        | GridOptimizedR1C1ScalarExpression::Match(_)
        | GridOptimizedR1C1ScalarExpression::VLookup(_) => false,
    }
}

fn r1c1_logical_expression_uses_only_reference_metadata(
    expression: &GridOptimizedR1C1LogicalExpression,
) -> bool {
    match expression {
        GridOptimizedR1C1LogicalExpression::Comparison(comparison) => {
            r1c1_comparison_uses_only_reference_metadata(comparison)
        }
        GridOptimizedR1C1LogicalExpression::Function(plan) => plan
            .arguments
            .iter()
            .all(r1c1_logical_expression_uses_only_reference_metadata),
    }
}

fn r1c1_comparison_uses_only_reference_metadata(comparison: &GridOptimizedR1C1Comparison) -> bool {
    r1c1_scalar_expression_uses_only_reference_metadata(&comparison.left)
        && r1c1_scalar_expression_uses_only_reference_metadata(&comparison.right)
}

pub(super) fn overlay_versioned_cell(
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

pub(super) fn register_formula_plan_cache_access(
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

pub(super) fn count_optimized_dirty_literal_evaluation(
    report: &mut GridOptimizedRecalcReport,
    source: GridOptimizedCellSource,
) {
    report.occupied_cells = report.occupied_cells.saturating_add(1);
    report.cells_evaluated = report.cells_evaluated.saturating_add(1);
    report.literal_cells = report.literal_cells.saturating_add(1);
    match source {
        GridOptimizedCellSource::SparsePoint => {
            report.sparse_literal_cells = report.sparse_literal_cells.saturating_add(1);
        }
        GridOptimizedCellSource::DenseValueRegion { .. } => {
            report.dense_value_region_cells = report.dense_value_region_cells.saturating_add(1);
        }
        GridOptimizedCellSource::RepeatedFormulaRegion { .. } => {}
    }
}

pub(super) fn count_optimized_dirty_formula_evaluation(
    report: &mut GridOptimizedRecalcReport,
    source: GridOptimizedCellSource,
) {
    report.occupied_cells = report.occupied_cells.saturating_add(1);
    report.cells_evaluated = report.cells_evaluated.saturating_add(1);
    report.formula_cells = report.formula_cells.saturating_add(1);
    report.formula_evaluations = report.formula_evaluations.saturating_add(1);
    match source {
        GridOptimizedCellSource::SparsePoint => {
            report.sparse_formula_cells = report.sparse_formula_cells.saturating_add(1);
        }
        GridOptimizedCellSource::DenseValueRegion { .. } => {}
        GridOptimizedCellSource::RepeatedFormulaRegion { .. } => {
            report.repeated_formula_region_cells =
                report.repeated_formula_region_cells.saturating_add(1);
        }
    }
}

pub(super) fn normalized_r1c1_expression(source_text: &str) -> Option<String> {
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

pub(super) fn compile_r1c1_range_aggregate_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1RangeAggregatePlan> {
    GridOptimizedR1C1RangeAggregateFunction::ALL
        .into_iter()
        .find_map(|function| compile_r1c1_range_function_expression(expression, function))
}

pub(super) fn compile_r1c1_range_function_expression(
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

pub(super) fn compile_r1c1_argument_aggregate_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1ArgumentAggregatePlan> {
    GridOptimizedR1C1RangeAggregateFunction::ALL
        .into_iter()
        .find_map(|function| {
            compile_r1c1_argument_aggregate_function_expression(expression, function)
        })
}

pub(super) fn compile_r1c1_argument_aggregate_function_expression(
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

pub(super) fn parse_r1c1_aggregate_argument(
    text: &str,
) -> Option<GridOptimizedR1C1AggregateArgument> {
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

pub(super) fn compile_r1c1_scalar_function_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1ScalarFunctionPlan> {
    GridOptimizedR1C1ScalarFunction::ALL
        .into_iter()
        .find_map(|function| compile_r1c1_scalar_function_call(expression, function))
}

pub(super) fn compile_r1c1_scalar_function_call(
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

pub(super) fn compile_r1c1_reference_function_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1ReferenceFunctionPlan> {
    GridOptimizedR1C1ReferenceFunction::ALL
        .into_iter()
        .find_map(|function| compile_r1c1_reference_function_call(expression, function))
}

pub(super) fn compile_r1c1_index_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1IndexPlan> {
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

pub(super) fn compile_r1c1_match_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1MatchPlan> {
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

pub(super) fn parse_r1c1_exact_match_type(text: &str) -> Option<()> {
    let value = text.parse::<f64>().ok()?;
    (value == 0.0).then_some(())
}

pub(super) fn compile_r1c1_vlookup_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1VLookupPlan> {
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

pub(super) fn parse_r1c1_exact_lookup_mode(text: &str) -> Option<()> {
    if text == "FALSE" {
        return Some(());
    }
    parse_r1c1_exact_match_type(text)
}

pub(super) fn compile_r1c1_text_function_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1TextFunctionPlan> {
    compile_r1c1_len_function_expression(expression)
        .or_else(|| compile_r1c1_left_function_expression(expression))
        .or_else(|| compile_r1c1_right_function_expression(expression))
        .or_else(|| compile_r1c1_concat_function_expression(expression))
}

pub(super) fn compile_r1c1_len_function_expression(
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

pub(super) fn compile_r1c1_left_function_expression(
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

pub(super) fn compile_r1c1_right_function_expression(
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

pub(super) fn compile_r1c1_concat_function_expression(
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

pub(super) fn compile_r1c1_reference_function_call(
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

pub(super) fn parse_r1c1_reference_function_argument(
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

pub(super) fn parse_r1c1_text_reference_argument(text: &str) -> Option<GridOptimizedR1C1Ref> {
    let mut text = text;
    while let Some(inner) = strip_outer_r1c1_parens(text) {
        text = inner;
    }
    parse_r1c1_reference(text)
}

pub(super) fn parse_r1c1_range_or_single_reference(
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

pub(super) fn compile_r1c1_iferror_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1IfErrorPlan> {
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

pub(super) fn compile_r1c1_if_expression(expression: &str) -> Option<GridOptimizedR1C1IfPlan> {
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

pub(super) fn compile_r1c1_logical_function_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1LogicalFunctionPlan> {
    GridOptimizedR1C1LogicalFunction::ALL
        .into_iter()
        .find_map(|function| compile_r1c1_logical_function_call(expression, function))
}

pub(super) fn compile_r1c1_logical_function_call(
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

pub(super) fn parse_r1c1_logical_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1LogicalExpression> {
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

pub(super) fn parse_r1c1_scalar_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1ScalarExpression> {
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

pub(super) fn strip_outer_r1c1_parens(expression: &str) -> Option<&str> {
    let inner = expression.strip_prefix('(')?.strip_suffix(')')?;
    if r1c1_outer_parens_enclose_expression(expression) {
        Some(inner)
    } else {
        None
    }
}

pub(super) fn r1c1_outer_parens_enclose_expression(expression: &str) -> bool {
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

pub(super) fn split_top_level_commas(expression: &str) -> Option<Vec<&str>> {
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

pub(super) fn parse_r1c1_comparison(expression: &str) -> Option<GridOptimizedR1C1Comparison> {
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

pub(super) fn compile_r1c1_comparison_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1ComparisonPlan> {
    Some(GridOptimizedR1C1ComparisonPlan {
        comparison: parse_r1c1_comparison(expression)?,
    })
}

pub(super) fn find_r1c1_comparison_operator(
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

pub(super) fn find_r1c1_range_separator(expression: &str) -> Option<usize> {
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

pub(super) fn compile_r1c1_binary_expression(
    expression: &str,
) -> Option<GridOptimizedR1C1BinaryPlan> {
    let (operator_index, op) = find_r1c1_binary_operator(expression)?;
    let left = expression.get(..operator_index)?;
    let right = expression.get(operator_index + 1..)?;
    Some(GridOptimizedR1C1BinaryPlan {
        left: Box::new(parse_r1c1_scalar_expression(left)?),
        op,
        right: Box::new(parse_r1c1_scalar_expression(right)?),
    })
}

pub(super) fn compile_r1c1_unary_minus_expression(
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

pub(super) fn find_r1c1_binary_operator(
    expression: &str,
) -> Option<(usize, GridOptimizedR1C1BinaryOp)> {
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

pub(super) fn find_r1c1_binary_operator_in_precedence_group(
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

pub(super) fn is_unary_r1c1_minus(expression: &str, index: usize) -> bool {
    expression
        .get(..index)
        .and_then(|prefix| prefix.chars().next_back())
        .is_none_or(|previous| matches!(previous, '+' | '-' | '*' | '/' | '(' | ','))
}

pub(super) fn parse_r1c1_operand(text: &str) -> Option<GridOptimizedR1C1Operand> {
    if let Ok(value) = text.parse::<f64>() {
        return GridOptimizedR1C1NumberLiteral::new(value).map(GridOptimizedR1C1Operand::Number);
    }
    parse_r1c1_reference_operand(text)
}

pub(super) fn parse_r1c1_reference_operand(text: &str) -> Option<GridOptimizedR1C1Operand> {
    parse_r1c1_reference(text).map(GridOptimizedR1C1Operand::Ref)
}

pub(super) fn parse_r1c1_reference(text: &str) -> Option<GridOptimizedR1C1Ref> {
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

pub(super) fn parse_r1c1_axis_ref(
    text: &str,
    index: &mut usize,
) -> Option<GridOptimizedR1C1AxisRef> {
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

pub(super) fn add_i32_to_u32(value: u32, delta: i32) -> Option<u32> {
    let result = i64::from(value).checked_add(i64::from(delta))?;
    u32::try_from(result).ok().filter(|result| *result >= 1)
}

pub(super) fn evaluate_optimized_formula_fast_path(
    address: &ExcelGridCellAddress,
    formula: &GridFormulaCell,
    valuation: &GridOptimizedValuation,
) -> Option<CalcValue> {
    GridOptimizedCompiledFormulaPlan::compile(formula)?.evaluate_single_cell(address, valuation)
}

pub(super) fn number_from_calc_value(value: &CalcValue) -> Option<f64> {
    match value.core {
        CoreValue::Number(number) => Some(number),
        _ => None,
    }
}

pub(super) fn aggregate_optimized_r1c1_rect(
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

pub(super) fn accumulate_optimized_r1c1_rect(
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
pub(super) struct GridOptimizedR1C1AggregateState {
    pub(super) sum: f64,
    pub(super) sum_sq: f64,
    pub(super) product: f64,
    pub(super) count: u64,
    pub(super) extreme: Option<f64>,
}

impl GridOptimizedR1C1AggregateState {
    pub(super) const fn new() -> Self {
        Self {
            sum: 0.0,
            sum_sq: 0.0,
            product: 1.0,
            count: 0,
            extreme: None,
        }
    }

    pub(super) fn accumulate(
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

    pub(super) fn finish(self, function: GridOptimizedR1C1RangeAggregateFunction) -> CalcValue {
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

pub(super) fn optimized_r1c1_value_for_cell(
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

pub(super) fn optimized_r1c1_calc_value_for_cell(
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

pub(super) fn optimized_r1c1_value_from_calc_value(
    value: &CalcValue,
) -> Option<GridOptimizedR1C1Value> {
    match value.core {
        CoreValue::Number(number) => Some(GridOptimizedR1C1Value::Number(number)),
        CoreValue::Error(error) => Some(GridOptimizedR1C1Value::Error(error)),
        _ => None,
    }
}

pub(super) fn optimized_r1c1_calc_value_from_value(
    value: GridOptimizedR1C1Value,
) -> Option<CalcValue> {
    match value {
        GridOptimizedR1C1Value::Number(number) => Some(CalcValue::number(number)),
        GridOptimizedR1C1Value::Error(error) => Some(CalcValue::error(error)),
    }
}

pub(super) fn optimized_r1c1_text_from_calc_value(
    value: CalcValue,
) -> Result<ExcelText, CalcValue> {
    match value.core {
        CoreValue::Text(text) => Ok(text),
        CoreValue::Error(error) => Err(CalcValue::error(error)),
        _ => Err(CalcValue::error(WorksheetErrorCode::Value)),
    }
}

pub(super) fn optimized_r1c1_text_count_from_value(
    value: GridOptimizedR1C1Value,
) -> Result<usize, CalcValue> {
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

pub(super) fn optimized_r1c1_index_from_value(
    value: GridOptimizedR1C1Value,
) -> Result<usize, CalcValue> {
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

pub(super) fn optimized_r1c1_text_slice(
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

pub(super) fn negate_optimized_r1c1_value(value: GridOptimizedR1C1Value) -> CalcValue {
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

#[derive(Debug, Clone, Copy)]
pub struct GridFormulaEvaluationRequest<'a> {
    pub address: &'a ExcelGridCellAddress,
    pub formula: &'a GridFormulaCell,
    pub authored: &'a BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
    pub previous_computed: &'a BTreeMap<ExcelGridCellAddress, CalcValue>,
}

impl GridRect {
    pub(super) fn check_sheet(&self, sheet: &GridCalcRefSheet) -> Result<(), GridRefError> {
        self.check_workbook_sheet(&sheet.workbook_id, &sheet.sheet_id)
    }
}

pub(super) fn scalar_cells_unchecked(rect: &GridRect) -> Vec<ExcelGridCellAddress> {
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

pub(super) fn compressed_range_block_for_cell(row: u32, col: u32) -> (u32, u32) {
    (
        (row.saturating_sub(1)) / GRID_INVALIDATION_COMPRESSED_RANGE_INDEX_BLOCK_SIZE,
        (col.saturating_sub(1)) / GRID_INVALIDATION_COMPRESSED_RANGE_INDEX_BLOCK_SIZE,
    )
}

pub(super) fn compressed_range_blocks_for_rect(rect: &GridRect) -> Vec<(u32, u32)> {
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

pub(super) fn axis_visibility_block_for_index(axis: GridAxis, index: u32) -> (GridAxis, u32) {
    (
        axis,
        (index.saturating_sub(1)) / GRID_INVALIDATION_COMPRESSED_RANGE_INDEX_BLOCK_SIZE,
    )
}

pub(super) fn axis_visibility_blocks_for_dependency(
    dependency: &GridAxisVisibilityDependency,
) -> Vec<(GridAxis, u32)> {
    let (_, first_block) = axis_visibility_block_for_index(dependency.axis, dependency.first);
    let (_, last_block) = axis_visibility_block_for_index(dependency.axis, dependency.last);
    (first_block..=last_block)
        .map(|block| (dependency.axis, block))
        .collect()
}

pub(super) fn axis_visibility_dependencies_intersect(
    lhs: &GridAxisVisibilityDependency,
    rhs: &GridAxisVisibilityDependency,
) -> bool {
    lhs.axis == rhs.axis && lhs.first <= rhs.last && rhs.first <= lhs.last
}

pub(super) fn spill_epoch_snapshot_map(
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

pub(super) fn validate_spill_epoch_snapshot(
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

pub(super) fn spill_epoch_change_kind(
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

pub(super) fn transform_cell_map<T>(
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

pub(super) fn transform_authored_cell_map_for_edit(
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

pub(super) fn transform_optimized_sparse_points_for_edit(
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

pub(super) fn transform_dense_value_region_for_edit(
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
pub(super) struct GridRepeatedFormulaRegionTransformOutput {
    pub(super) regions: Vec<GridRepeatedFormulaRegion>,
    pub(super) formula_segments_transformed: usize,
    pub(super) formula_reference_transforms: usize,
}

pub(super) fn transform_repeated_formula_region_for_edit(
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

pub(super) fn dense_values_for_subrect(
    region: &GridDenseValueRegion,
    subrect: &GridRect,
) -> Vec<CalcValue> {
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

pub(super) fn bytes_per_cell_micros(bytes: u64, cells: u64) -> u64 {
    if cells == 0 {
        return 0;
    }
    bytes
        .saturating_mul(1_000_000)
        .saturating_add(cells.saturating_sub(1))
        / cells
}

pub(super) fn estimated_grid_cell_coord_bytes(_coord: GridCellCoord) -> u64 {
    u64::try_from(std::mem::size_of::<GridCellCoord>()).unwrap_or(u64::MAX)
}

pub(super) fn estimated_grid_rect_heap_bytes(rect: &GridRect) -> u64 {
    u64::try_from(rect.workbook_id.len())
        .unwrap_or(u64::MAX)
        .saturating_add(u64::try_from(rect.sheet_id.len()).unwrap_or(u64::MAX))
}

pub(super) fn estimated_versioned_authored_cell_bytes(cell: &GridVersionedAuthoredCell) -> u64 {
    u64::try_from(std::mem::size_of::<u64>())
        .unwrap_or(u64::MAX)
        .saturating_add(cell.cell.estimated_authored_bytes())
}

pub(super) fn estimated_calc_value_bytes(value: &CalcValue) -> u64 {
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

pub(super) fn estimated_calc_value_frame_payload_bytes(value: &CalcValue) -> u64 {
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

pub(super) fn estimated_formula_cell_bytes(formula: &GridFormulaCell) -> u64 {
    u64::try_from(std::mem::size_of::<GridFormulaCell>())
        .unwrap_or(u64::MAX)
        .saturating_add(u64::try_from(formula.source_text.len()).unwrap_or(u64::MAX))
        .saturating_add(u64::try_from(formula.normal_form_key.len()).unwrap_or(u64::MAX))
}

pub(super) fn estimated_repeated_formula_region_bytes(region: &GridRepeatedFormulaRegion) -> u64 {
    u64::try_from(std::mem::size_of::<GridRepeatedFormulaRegion>())
        .unwrap_or(u64::MAX)
        .saturating_add(estimated_grid_rect_heap_bytes(&region.rect))
        .saturating_add(estimated_formula_cell_bytes(&region.formula))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct GridAxisEditSegment {
    pub(super) old_start: u32,
    pub(super) old_end: u32,
    pub(super) new_start: u32,
    pub(super) new_end: u32,
}

pub(super) fn rect_segments_for_axis_edit(
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

pub(super) fn axis_segments_for_edit(
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
pub(super) struct GridFormulaStructuralTransformStats {
    pub(super) formula_cells_transformed: usize,
    pub(super) formula_reference_transforms: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FormulaSourceReplacement {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) replacement: String,
    pub(super) transformed_reference: bool,
}

pub(super) fn transform_formula_cell_for_axis_edit(
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

pub(super) fn transform_authored_formulas_for_table_rename(
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

pub(super) fn transform_authored_formulas_for_defined_name_rename(
    authored: &mut BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
    old_name_key: &str,
    new_name: &str,
    bounds: ExcelGridBounds,
    skip_if_shadowed_by_scope: bool,
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
            skip_if_shadowed_by_scope,
        )?;
        *formula = transformed;
        total.formula_cells_transformed += stats.formula_cells_transformed;
        total.formula_reference_transforms += stats.formula_reference_transforms;
    }
    Ok(total)
}

pub(super) fn transform_authored_formulas_for_defined_name_delete(
    authored: &mut BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
    deleted_name_key: &str,
    bounds: ExcelGridBounds,
    skip_if_shadowed_by_scope: bool,
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
            skip_if_shadowed_by_scope,
        )?;
        *formula = transformed;
        total.formula_cells_transformed += stats.formula_cells_transformed;
        total.formula_reference_transforms += stats.formula_reference_transforms;
    }
    Ok(total)
}

pub(super) fn transform_authored_formulas_for_table_delete(
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

pub(super) fn transform_sparse_point_formulas_for_defined_name_rename(
    sparse_points: &mut BTreeMap<GridCellCoord, GridVersionedAuthoredCell>,
    workbook_id: &str,
    sheet_id: &str,
    old_name_key: &str,
    new_name: &str,
    bounds: ExcelGridBounds,
    skip_if_shadowed_by_scope: bool,
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
            skip_if_shadowed_by_scope,
        )?;
        *formula = transformed;
        total.formula_cells_transformed += stats.formula_cells_transformed;
        total.formula_reference_transforms += stats.formula_reference_transforms;
    }
    Ok(total)
}

pub(super) fn transform_sparse_point_formulas_for_defined_name_delete(
    sparse_points: &mut BTreeMap<GridCellCoord, GridVersionedAuthoredCell>,
    workbook_id: &str,
    sheet_id: &str,
    deleted_name_key: &str,
    bounds: ExcelGridBounds,
    skip_if_shadowed_by_scope: bool,
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
            skip_if_shadowed_by_scope,
        )?;
        *formula = transformed;
        total.formula_cells_transformed += stats.formula_cells_transformed;
        total.formula_reference_transforms += stats.formula_reference_transforms;
    }
    Ok(total)
}

pub(super) fn transform_sparse_point_formulas_for_table_rename(
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

pub(super) fn transform_sparse_point_formulas_for_table_delete(
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

pub(super) fn transform_repeated_formula_regions_for_defined_name_rename(
    regions: &mut [GridRepeatedFormulaRegion],
    old_name_key: &str,
    new_name: &str,
    bounds: ExcelGridBounds,
    skip_if_shadowed_by_scope: bool,
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
            skip_if_shadowed_by_scope,
        )?;
        region.formula = transformed;
        total.formula_cells_transformed += stats.formula_cells_transformed;
        total.formula_reference_transforms += stats.formula_reference_transforms;
    }
    Ok(total)
}

pub(super) fn transform_repeated_formula_regions_for_defined_name_delete(
    regions: &mut [GridRepeatedFormulaRegion],
    deleted_name_key: &str,
    bounds: ExcelGridBounds,
    skip_if_shadowed_by_scope: bool,
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
            skip_if_shadowed_by_scope,
        )?;
        region.formula = transformed;
        total.formula_cells_transformed += stats.formula_cells_transformed;
        total.formula_reference_transforms += stats.formula_reference_transforms;
    }
    Ok(total)
}

pub(super) fn transform_repeated_formula_regions_for_table_rename(
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

pub(super) fn transform_repeated_formula_regions_for_table_delete(
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

pub(super) fn transform_formula_cell_for_table_rename(
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

pub(super) fn transform_formula_cell_for_defined_name_rename(
    formula: GridFormulaCell,
    address: &ExcelGridCellAddress,
    old_name_key: &str,
    new_name: &str,
    bounds: ExcelGridBounds,
    skip_if_shadowed_by_scope: bool,
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
        // A same-text sheet-scoped name shadows the global name being
        // renamed: a reference bound here resolved to the scoped entry,
        // not the global one being renamed, so rewriting it would corrupt
        // a binding that legitimately targets the scoped name. See D3.
        if skip_if_shadowed_by_scope {
            continue;
        }
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

pub(super) fn transform_formula_cell_for_defined_name_delete(
    formula: GridFormulaCell,
    address: &ExcelGridCellAddress,
    deleted_name_key: &str,
    bounds: ExcelGridBounds,
    skip_if_shadowed_by_scope: bool,
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
        // A same-text sheet-scoped name shadows the global name being
        // deleted: a reference bound here resolved to the scoped entry, so
        // deleting the global name must not rewrite it to #NAME?. See D3.
        if skip_if_shadowed_by_scope {
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

pub(super) fn transform_formula_cell_for_table_delete(
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

pub(super) fn rewrite_defined_name_reference(
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

pub(super) fn defined_name_reference_has_key(
    source_text: &str,
    name_key: &str,
    bounds: ExcelGridBounds,
) -> bool {
    let local_start = source_text.rfind('!').map_or(0, |index| index + 1);
    let name = source_text[local_start..].trim();
    defined_name_key_for_name(name, bounds).is_ok_and(|key| key == name_key)
}

pub(super) fn rewrite_structured_reference_table_name(
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

pub(super) fn structured_reference_has_explicit_table_key(
    source_text: &str,
    table_key: &str,
) -> bool {
    let Some(bracket_index) = source_text.find('[') else {
        return false;
    };
    let local_start = source_text[..bracket_index]
        .rfind('!')
        .map_or(0, |index| index + 1);
    let table_name = source_text[local_start..bracket_index].trim();
    !table_name.is_empty() && table_name.to_ascii_uppercase() == table_key
}

pub(super) fn bind_grid_formula_for_transform(
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

pub(super) fn excel_grid_structural_edit_from_axis_edit(
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

pub(super) fn select_non_overlapping_replacements(
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

pub(super) fn apply_formula_source_replacements(
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

pub(super) fn transform_diagnostics(diagnostics: &[String]) -> String {
    if diagnostics.is_empty() {
        "no diagnostics".to_string()
    } else {
        diagnostics.join("; ")
    }
}

pub(super) fn transform_address_for_edit(
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

pub(super) fn transform_spill_value_fingerprints_for_edit(
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

pub(super) fn transform_rect_for_edit(
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
pub(super) struct FeatureRenderedRegionTransformBatch {
    pub(super) regions: Vec<FeatureRenderedRegion>,
    pub(super) kept: usize,
    pub(super) dropped: usize,
    pub(super) marked_needs_refresh: usize,
}

pub(super) fn transform_feature_rendered_regions_for_axis_edit(
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

pub(super) fn feature_rendered_region_axis_edit_refused(
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

pub(super) fn transform_dependency_for_axis_edit(
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
        GridDependency::NameIdentity(dependency) => {
            Ok(Some(GridDependency::NameIdentity(dependency.clone())))
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
        GridDependency::TableIdentity(dependency) => {
            Ok(Some(GridDependency::TableIdentity(dependency.clone())))
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
        GridDependency::ReferenceMetadata(dependency) => Ok(transform_dependency_for_axis_edit(
            dependency, edit, bounds,
        )?
        .map(|dependency| GridDependency::ReferenceMetadata(Box::new(dependency)))),
        GridDependency::DynamicRequest(request_key) => {
            Ok(Some(GridDependency::DynamicRequest(request_key.clone())))
        }
    }
}

pub(super) fn transform_axis_visibility_dependency_for_edit(
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

pub(super) fn transform_axis_value_dependency_for_edit(
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

pub(super) fn transform_axis_range(
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

pub(super) fn transform_axis_index(
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

pub(super) fn validate_axis_edit(
    edit: GridAxisEdit,
    bounds: ExcelGridBounds,
) -> Result<(), GridRefError> {
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

pub(super) fn validate_axis_visibility_dependency(
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

pub(super) fn validate_axis_value_dependency(
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

pub(super) fn delete_last(first: u32, count: u32) -> Result<u32, GridRefError> {
    first
        .checked_add(count.saturating_sub(1))
        .ok_or_else(|| GridRefError::InvalidStructuralEdit {
            detail: format!("delete first {first} count {count} overflows axis"),
        })
}

pub(super) const fn axis_max(axis: GridAxis, bounds: ExcelGridBounds) -> u32 {
    match axis {
        GridAxis::Row => bounds.max_rows,
        GridAxis::Column => bounds.max_cols,
    }
}

pub(super) const fn address_axis_index(address: &ExcelGridCellAddress, axis: GridAxis) -> u32 {
    match axis {
        GridAxis::Row => address.row,
        GridAxis::Column => address.col,
    }
}

pub(super) const fn rect_axis_range(rect: &GridRect, axis: GridAxis) -> (u32, u32) {
    match axis {
        GridAxis::Row => (rect.top_row, rect.bottom_row),
        GridAxis::Column => (rect.left_col, rect.right_col),
    }
}
