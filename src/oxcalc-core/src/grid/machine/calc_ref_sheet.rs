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
    pub(super) overlays: GridOverlaySet,
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
        self.overlays.spill_facts.remove(address);
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
        &self.overlays.table_overlays
    }

    pub fn set_table_overlay(&mut self, table: GridTableOverlay) -> Result<(), GridRefError> {
        table.check_sheet(&self.workbook_id, &self.sheet_id, self.bounds)?;
        let table_key = table_key_for_name(&table.table_name, self.bounds)?;
        let table_range = table.table_range.clone();
        if let Some(old_table) = self.overlays.table_overlays.get(&table_key) {
            remove_table_overlay_feature_regions(
                &mut self.overlays.feature_rendered_regions,
                &old_table.table_range,
            );
        }
        self.overlays.table_overlays.insert(table_key, table);
        self.add_feature_rendered_region(table_range, "table-overlay", false);
        Ok(())
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
        let feature_regions_removed = remove_table_overlay_feature_regions(
            &mut self.overlays.feature_rendered_regions,
            &old_table.table_range,
        );
        let table_range = table.table_range.clone();
        self.overlays
            .table_overlays
            .insert(table_key.clone(), table);
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

        let old_defined_names = std::mem::take(&mut self.defined_names);
        for (name_key, rect) in old_defined_names {
            let (Some(rect), _) = transform_rect_for_edit(&rect, edit, self.bounds)? else {
                continue;
            };
            self.defined_names.insert(name_key, rect);
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
            || self.overlays.spill_facts == *base_spill_facts
            || !authored_contains_grid_spill_reference(authored, profile, self.bounds)
        {
            return Ok(());
        }

        report.spill_repair_converged = false;
        for _ in 0..formula_cells {
            let spill_facts_before = self.overlays.spill_facts.clone();
            report.spill_repair_passes += 1;

            for (address, cell) in authored {
                let GridAuthoredCell::Formula(formula) = cell else {
                    continue;
                };
                report.spill_repair_formula_evaluations += 1;
                let value = self.evaluate_formula_with_spill_repair(address, formula, profile)?;
                self.publish_formula_value(address.clone(), value, authored);
            }

            if self.overlays.spill_facts == spill_facts_before {
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

    fn clear_formula_output_for_anchor(&mut self, anchor: &ExcelGridCellAddress) {
        if let Some(fact) = self.overlays.spill_facts.remove(anchor) {
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
            self.overlays.spill_facts.insert(
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
            self.overlays.spill_facts.insert(
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
        self.overlays.spill_facts.insert(
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
        .with_borrowed_cells(&self.computed);
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
            provider = provider.with_defined_name(name, rect.clone());
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
        self.overlays
            .spill_facts
            .values()
            .find(|fact| fact.extent.contains(address))
            .map(|fact| fact.anchor.clone())
    }
}
