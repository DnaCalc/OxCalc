#![forbid(unsafe_code)]

//! Seed corpus runner for the W061 strict Excel grid floor.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use oxfunc_core::value::{CalcValue, WorksheetErrorCode};
use serde_json::{Value, json};
use thiserror::Error;

use crate::coordinator::calc_value_display_text;
use crate::grid::authored::GridFormulaCell;
use crate::grid::coords::{ExcelGridBounds, ExcelGridCellAddress};
use crate::grid::error::GridRefError;
use crate::grid::geometry::GridRect;
use crate::grid::machine::{
    GRID_CALC_REF_DEFAULT_MATERIALIZATION_LIMIT, GridAxisValueDependency,
    GridAxisVisibilityDependency, GridDependency, GridDifferentialMismatch,
    GridDifferentialRunReport, GridEngineCellReadout, GridEngineMode, GridEngineRecalcReport,
    GridEngineRunReport, GridInvalidationNamespaceLifecycleOperation,
    GridInvalidationNamespaceLifecycleReport, GridInvalidationRef,
    GridInvalidationStructuralEditReport, GridNameDependency,
    GridOptimizedFormulaReferenceEnumerationReport, GridOptimizedSheet, GridSpillBlockerDependency,
    GridSpillDependency, GridSpillFact, GridTableColumn, GridTableDependency, GridTableOverlay,
};
use crate::grid::machine::{GridAxis, GridAxisEdit, GridAxisEditKind};

const GRID_RUN_REPORT_SCHEMA_V1: &str = "oxcalc.grid.seed_corpus_run.v1";
const GRID_CASE_INDEX_SCHEMA_V1: &str = "oxcalc.grid.seed_corpus_case_index.v1";
const GRID_CASE_RESULT_SCHEMA_V1: &str = "oxcalc.grid.seed_corpus_case_result.v1";
const GRID_WORKBOOK_ID: &str = "book:grid-seed";
const GRID_SHEET_ID: &str = "sheet:grid-seed";

#[derive(Debug, Error)]
pub enum GridRunnerError {
    #[error("unknown grid engine mode {value}; expected reference, optimized, or both")]
    UnknownEngineMode { value: String },
    #[error(transparent)]
    Grid(#[from] GridRefError),
    #[error("failed to create grid artifact directory {path}: {source}")]
    CreateDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to remove existing grid artifact directory {path}: {source}")]
    RemoveDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to write grid artifact {path}: {source}")]
    WriteFile {
        path: String,
        source: std::io::Error,
    },
}

impl GridEngineMode {
    pub fn from_engine_arg(value: &str) -> Result<Self, GridRunnerError> {
        match value {
            "reference" => Ok(Self::Reference),
            "optimized" => Ok(Self::Optimized),
            "both" => Ok(Self::Both),
            other => Err(GridRunnerError::UnknownEngineMode {
                value: other.to_string(),
            }),
        }
    }

    #[must_use]
    pub const fn engine_arg(self) -> &'static str {
        match self {
            Self::Reference => "reference",
            Self::Optimized => "optimized",
            Self::Both => "both",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridExpectedCellValue {
    pub address: ExcelGridCellAddress,
    pub expected: CalcValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridExpectedValueMismatch {
    pub address: ExcelGridCellAddress,
    pub expected: CalcValue,
    pub reference: Option<CalcValue>,
    pub optimized: Option<CalcValue>,
}

impl GridExpectedValueMismatch {
    #[must_use]
    pub fn expected_display(&self) -> String {
        calc_value_display_text(&self.expected)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridInvalidationCellDependencies {
    pub dependent: ExcelGridCellAddress,
    pub dependencies: Vec<GridDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridDirtyClosureCheck {
    pub check_id: String,
    pub seeds: Vec<ExcelGridCellAddress>,
    pub expected_dirty: Vec<ExcelGridCellAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridDynamicClosureCheck {
    pub check_id: String,
    pub request_key: String,
    pub expected_dirty: Vec<ExcelGridCellAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSpillClosureCheck {
    pub check_id: String,
    pub dependency: GridSpillDependency,
    pub expected_dirty: Vec<ExcelGridCellAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSpillBlockerClosureCheck {
    pub check_id: String,
    pub dependency: GridSpillBlockerDependency,
    pub expected_dirty: Vec<ExcelGridCellAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridAxisVisibilityClosureCheck {
    pub check_id: String,
    pub dependency: GridAxisVisibilityDependency,
    pub expected_dirty: Vec<ExcelGridCellAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridNamespaceLifecycleOperation {
    RenameDefinedName {
        old_name: String,
        new_name: String,
    },
    DeleteDefinedName {
        name: String,
    },
    RenameTable {
        old_table: String,
        new_table: String,
    },
    DeleteTable {
        table: String,
    },
    ResizeTable {
        table: String,
        new_extent: GridRect,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridNamespaceLifecycleCheck {
    pub check_id: String,
    pub operation: GridNamespaceLifecycleOperation,
    pub expected_dirty: Vec<ExcelGridCellAddress>,
    pub dirty_closure_checks: Vec<GridDirtyClosureCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridStructuralInvalidationCheck {
    pub check_id: String,
    pub edit: GridAxisEdit,
    pub dirty_closure_checks: Vec<GridDirtyClosureCheck>,
    pub dynamic_closure_checks: Vec<GridDynamicClosureCheck>,
    pub spill_closure_checks: Vec<GridSpillClosureCheck>,
    pub spill_blocker_closure_checks: Vec<GridSpillBlockerClosureCheck>,
    pub axis_visibility_closure_checks: Vec<GridAxisVisibilityClosureCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridInvalidationScenario {
    pub dependencies: Vec<GridInvalidationCellDependencies>,
    pub dirty_closure_checks: Vec<GridDirtyClosureCheck>,
    pub dynamic_closure_checks: Vec<GridDynamicClosureCheck>,
    pub spill_closure_checks: Vec<GridSpillClosureCheck>,
    pub spill_blocker_closure_checks: Vec<GridSpillBlockerClosureCheck>,
    pub axis_visibility_closure_checks: Vec<GridAxisVisibilityClosureCheck>,
    pub namespace_lifecycle_checks: Vec<GridNamespaceLifecycleCheck>,
    pub structural_edit_checks: Vec<GridStructuralInvalidationCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridInstalledDependencyReport {
    pub dependent: ExcelGridCellAddress,
    pub scalar_edge_count: usize,
    pub scalar_dependencies: Vec<ExcelGridCellAddress>,
    pub dynamic_dependencies: Vec<String>,
    pub spill_dependencies: Vec<GridSpillDependency>,
    pub spill_blocker_dependencies: Vec<GridSpillBlockerDependency>,
    pub axis_value_dependencies: Vec<GridAxisValueDependency>,
    pub axis_visibility_dependencies: Vec<GridAxisVisibilityDependency>,
    pub name_dependencies: Vec<GridNameDependency>,
    pub table_dependencies: Vec<GridTableDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridDirtyClosureReport {
    pub check_id: String,
    pub seeds: Vec<ExcelGridCellAddress>,
    pub expected_dirty: Vec<ExcelGridCellAddress>,
    pub actual_dirty: Vec<ExcelGridCellAddress>,
    pub matched: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridDynamicClosureReport {
    pub check_id: String,
    pub request_key: String,
    pub expected_dirty: Vec<ExcelGridCellAddress>,
    pub actual_dirty: Vec<ExcelGridCellAddress>,
    pub matched: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSpillClosureReport {
    pub check_id: String,
    pub dependency: GridSpillDependency,
    pub expected_dirty: Vec<ExcelGridCellAddress>,
    pub actual_dirty: Vec<ExcelGridCellAddress>,
    pub matched: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSpillBlockerClosureReport {
    pub check_id: String,
    pub dependency: GridSpillBlockerDependency,
    pub expected_dirty: Vec<ExcelGridCellAddress>,
    pub actual_dirty: Vec<ExcelGridCellAddress>,
    pub matched: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridAxisVisibilityClosureReport {
    pub check_id: String,
    pub dependency: GridAxisVisibilityDependency,
    pub expected_dirty: Vec<ExcelGridCellAddress>,
    pub actual_dirty: Vec<ExcelGridCellAddress>,
    pub matched: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridStructuralInvalidationReport {
    pub check_id: String,
    pub edit: GridAxisEdit,
    pub edit_report: GridInvalidationStructuralEditReport,
    pub dirty_closures: Vec<GridDirtyClosureReport>,
    pub dynamic_closures: Vec<GridDynamicClosureReport>,
    pub spill_closures: Vec<GridSpillClosureReport>,
    pub spill_blocker_closures: Vec<GridSpillBlockerClosureReport>,
    pub axis_visibility_closures: Vec<GridAxisVisibilityClosureReport>,
    pub mismatch_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridNamespaceLifecycleReport {
    pub check_id: String,
    pub operation: GridNamespaceLifecycleOperation,
    pub lifecycle_report: GridInvalidationNamespaceLifecycleReport,
    pub expected_dirty: Vec<ExcelGridCellAddress>,
    pub actual_dirty: Vec<ExcelGridCellAddress>,
    pub matched: bool,
    pub dirty_closures: Vec<GridDirtyClosureReport>,
    pub mismatch_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridInvalidationRunReport {
    pub scalar_edge_count: usize,
    pub spill_blocker_edge_count: usize,
    pub axis_value_edge_count: usize,
    pub name_edge_count: usize,
    pub table_edge_count: usize,
    pub installed_dependencies: Vec<GridInstalledDependencyReport>,
    pub dirty_closures: Vec<GridDirtyClosureReport>,
    pub dynamic_closures: Vec<GridDynamicClosureReport>,
    pub spill_closures: Vec<GridSpillClosureReport>,
    pub spill_blocker_closures: Vec<GridSpillBlockerClosureReport>,
    pub axis_visibility_closures: Vec<GridAxisVisibilityClosureReport>,
    pub namespace_lifecycle: Vec<GridNamespaceLifecycleReport>,
    pub structural_edits: Vec<GridStructuralInvalidationReport>,
    pub mismatch_count: usize,
}

impl GridInvalidationRunReport {
    #[must_use]
    pub const fn matched(&self) -> bool {
        self.mismatch_count == 0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridCorpusScenario {
    pub case_id: String,
    pub description: String,
    pub tags: Vec<String>,
    pub sheet: GridOptimizedSheet,
    pub probes: Vec<ExcelGridCellAddress>,
    pub expected_values: Vec<GridExpectedCellValue>,
    pub invalidation: Option<GridInvalidationScenario>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridCorpusCaseReport {
    pub case_id: String,
    pub description: String,
    pub tags: Vec<String>,
    pub engine_report: GridDifferentialRunReport,
    pub expected_values: Vec<GridExpectedCellValue>,
    pub expectation_mismatches: Vec<GridExpectedValueMismatch>,
    pub invalidation_report: Option<GridInvalidationRunReport>,
    pub p20_report: Option<GridP20RunReport>,
}

impl GridCorpusCaseReport {
    #[must_use]
    pub fn matched(&self) -> bool {
        self.engine_report.mismatches.is_empty()
            && self.expectation_mismatches.is_empty()
            && self
                .invalidation_report
                .as_ref()
                .is_none_or(GridInvalidationRunReport::matched)
            && self
                .p20_report
                .as_ref()
                .is_none_or(GridP20RunReport::matched)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridP20RunReport {
    pub formula_enumerations: Vec<GridOptimizedFormulaReferenceEnumerationReport>,
    pub mismatch_count: usize,
}

impl GridP20RunReport {
    #[must_use]
    pub fn matched(&self) -> bool {
        self.mismatch_count == 0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridCorpusRunReport {
    pub schema_version: String,
    pub engine_mode: GridEngineMode,
    pub case_count: usize,
    pub expectation_mismatch_count: usize,
    pub differential_mismatch_count: usize,
    pub invalidation_mismatch_count: usize,
    pub p20_mismatch_count: usize,
    pub cases: Vec<GridCorpusCaseReport>,
}

impl GridCorpusRunReport {
    #[must_use]
    pub fn matched(&self) -> bool {
        self.expectation_mismatch_count == 0
            && self.differential_mismatch_count == 0
            && self.invalidation_mismatch_count == 0
            && self.p20_mismatch_count == 0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GridCorpusRunner {
    materialization_limit: u64,
}

impl GridCorpusRunner {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            materialization_limit: GRID_CALC_REF_DEFAULT_MATERIALIZATION_LIMIT,
        }
    }

    #[must_use]
    pub const fn with_materialization_limit(mut self, materialization_limit: u64) -> Self {
        self.materialization_limit = materialization_limit;
        self
    }

    pub fn run_seed_corpus(
        &self,
        engine_mode: GridEngineMode,
    ) -> Result<GridCorpusRunReport, GridRunnerError> {
        let mut cases = Vec::new();
        let mut expectation_mismatch_count = 0;
        let mut differential_mismatch_count = 0;
        let mut invalidation_mismatch_count = 0;
        let mut p20_mismatch_count = 0;
        for scenario in seed_corpus_scenarios()? {
            let engine_report = scenario.sheet.run_engine_mode_with_oxfml(
                engine_mode,
                scenario.probes.clone(),
                self.materialization_limit,
            )?;
            let expectation_mismatches =
                compare_expected_values(&scenario.expected_values, &engine_report);
            let invalidation_report = scenario
                .invalidation
                .as_ref()
                .map(run_invalidation_scenario)
                .transpose()?;
            let p20_report =
                run_p20_report_if_applicable(&scenario, engine_mode, self.materialization_limit)?;
            expectation_mismatch_count += expectation_mismatches.len();
            differential_mismatch_count += engine_report.mismatches.len();
            invalidation_mismatch_count += invalidation_report
                .as_ref()
                .map_or(0, |report| report.mismatch_count);
            p20_mismatch_count += p20_report
                .as_ref()
                .map_or(0, |report| report.mismatch_count);
            cases.push(GridCorpusCaseReport {
                case_id: scenario.case_id,
                description: scenario.description,
                tags: scenario.tags,
                engine_report,
                expected_values: scenario.expected_values,
                expectation_mismatches,
                invalidation_report,
                p20_report,
            });
        }

        Ok(GridCorpusRunReport {
            schema_version: GRID_RUN_REPORT_SCHEMA_V1.to_string(),
            engine_mode,
            case_count: cases.len(),
            expectation_mismatch_count,
            differential_mismatch_count,
            invalidation_mismatch_count,
            p20_mismatch_count,
            cases,
        })
    }

    pub fn run_seed_corpus_arg(
        &self,
        engine_arg: &str,
    ) -> Result<GridCorpusRunReport, GridRunnerError> {
        self.run_seed_corpus(GridEngineMode::from_engine_arg(engine_arg)?)
    }

    pub fn execute_seed_corpus(
        &self,
        repo_root: &Path,
        run_id: &str,
        engine_arg: &str,
    ) -> Result<GridCorpusRunReport, GridRunnerError> {
        let report = self.run_seed_corpus_arg(engine_arg)?;
        let artifact_root = repo_root.join(grid_artifact_root_relative(run_id));
        reset_artifact_root(&artifact_root)?;
        create_directory(&artifact_root)?;
        create_directory(&artifact_root.join("cases"))?;

        let mut case_index = Vec::new();
        for case in &report.cases {
            let case_directory = artifact_root.join("cases").join(&case.case_id);
            create_directory(&case_directory)?;
            write_json(
                &case_directory.join("result.json"),
                &case_result_json(&report, case),
            )?;
            case_index.push(json!({
                "case_id": &case.case_id,
                "path": format!("cases/{}/result.json", case.case_id),
                "tags": &case.tags,
                "status": if case.matched() { "matched" } else { "mismatched" },
                "expectation_mismatch_count": case.expectation_mismatches.len(),
                "differential_mismatch_count": case.engine_report.mismatches.len(),
                "invalidation_mismatch_count": case.invalidation_report.as_ref().map_or(0, |report| report.mismatch_count),
                "p20_mismatch_count": case.p20_report.as_ref().map_or(0, |report| report.mismatch_count)
            }));
        }

        write_json(
            &artifact_root.join("case_index.json"),
            &json!({
                "schema_version": GRID_CASE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "engine_mode": report.engine_mode.engine_arg(),
                "cases": case_index
            }),
        )?;
        write_json(
            &artifact_root.join("run_summary.json"),
            &run_summary_json(run_id, &report),
        )?;
        Ok(report)
    }
}

fn run_p20_report_if_applicable(
    scenario: &GridCorpusScenario,
    engine_mode: GridEngineMode,
    materialization_limit: u64,
) -> Result<Option<GridP20RunReport>, GridRunnerError> {
    if scenario.case_id != "grid_seed_whole_axis_value_invalidation_001"
        || engine_mode == GridEngineMode::Reference
    {
        return Ok(None);
    }

    let mut formula_enumerations = Vec::new();
    for formula_address in [address(2, 4), address(2, 5)] {
        formula_enumerations.extend(
            scenario
                .sheet
                .optimized_formula_reference_enumeration_reports(
                    &formula_address,
                    materialization_limit,
                )?,
        );
    }
    let mismatch_count = formula_enumerations
        .iter()
        .filter(|report| !report.p20_occupied_slots_holds())
        .count();
    Ok(Some(GridP20RunReport {
        formula_enumerations,
        mismatch_count,
    }))
}

impl Default for GridCorpusRunner {
    fn default() -> Self {
        Self::new()
    }
}

fn run_summary_json(run_id: &str, report: &GridCorpusRunReport) -> Value {
    json!({
        "schema_version": &report.schema_version,
        "run_id": run_id,
        "artifact_root": grid_artifact_root_relative(run_id),
        "engine_mode": report.engine_mode.engine_arg(),
        "case_count": report.case_count,
        "expectation_mismatch_count": report.expectation_mismatch_count,
        "differential_mismatch_count": report.differential_mismatch_count,
        "invalidation_mismatch_count": report.invalidation_mismatch_count,
        "p20_mismatch_count": report.p20_mismatch_count,
        "matched": report.matched(),
        "cases": report.cases.iter().map(|case| {
            json!({
                "case_id": &case.case_id,
                "status": if case.matched() { "matched" } else { "mismatched" },
                "tags": &case.tags,
                "expectation_mismatch_count": case.expectation_mismatches.len(),
                "differential_mismatch_count": case.engine_report.mismatches.len(),
                "invalidation_mismatch_count": case.invalidation_report.as_ref().map_or(0, |report| report.mismatch_count),
                "p20_mismatch_count": case.p20_report.as_ref().map_or(0, |report| report.mismatch_count)
            })
        }).collect::<Vec<_>>()
    })
}

fn case_result_json(report: &GridCorpusRunReport, case: &GridCorpusCaseReport) -> Value {
    json!({
        "schema_version": GRID_CASE_RESULT_SCHEMA_V1,
        "case_id": &case.case_id,
        "description": &case.description,
        "tags": &case.tags,
        "status": if case.matched() { "matched" } else { "mismatched" },
        "engine_mode": report.engine_mode.engine_arg(),
        "expected_values": case.expected_values.iter().map(expected_value_json).collect::<Vec<_>>(),
        "expectation_mismatches": case.expectation_mismatches.iter().map(expectation_mismatch_json).collect::<Vec<_>>(),
        "differential_mismatches": case.engine_report.mismatches.iter().map(differential_mismatch_json).collect::<Vec<_>>(),
        "invalidation": case.invalidation_report.as_ref().map(invalidation_report_json),
        "p20": case.p20_report.as_ref().map(p20_report_json),
        "reference": case.engine_report.reference.as_ref().map(engine_run_json),
        "optimized": case.engine_report.optimized.as_ref().map(engine_run_json)
    })
}

fn p20_report_json(report: &GridP20RunReport) -> Value {
    json!({
        "mismatch_count": report.mismatch_count,
        "matched": report.matched(),
        "formula_enumerations": report.formula_enumerations.iter().map(|enumeration| {
            json!({
                "formula_address": address_json(&enumeration.formula_address),
                "reference_source_text": &enumeration.reference_source_text,
                "declared_cell_count": enumeration.declared_cell_count,
                "defined_cell_count": enumeration.defined_cell_count,
                "dense_value_cells_visited": enumeration.dense_value_cells_visited,
                "sparse_value_cells_visited": enumeration.sparse_value_cells_visited,
                "slots_visited": enumeration.slots_visited(),
                "compact_regions_intersected": enumeration.compact_regions_intersected,
                "p20_occupied_slots_holds": enumeration.p20_occupied_slots_holds()
            })
        }).collect::<Vec<_>>()
    })
}

fn engine_run_json(report: &GridEngineRunReport) -> Value {
    json!({
        "mode": report.mode.engine_arg(),
        "recalc": recalc_report_json(&report.recalc),
        "readout": report.readout.iter().map(readout_json).collect::<Vec<_>>(),
        "warm_noop": report.warm_noop.as_ref().map(warm_noop_json)
    })
}

fn warm_noop_json(report: &crate::grid::machine::GridEngineWarmNoOpReport) -> Value {
    json!({
        "recalc": {
            "cache_hit": report.recalc.cache_hit,
            "cached_occupied_cells": report.recalc.cached_occupied_cells,
            "cached_formula_cells": report.recalc.cached_formula_cells,
            "cells_visited": report.recalc.cells_visited,
            "formula_evaluations": report.recalc.formula_evaluations,
            "p19_warm_noop_holds": report.recalc.p19_warm_noop_holds()
        },
        "readout": report.readout.iter().map(readout_json).collect::<Vec<_>>()
    })
}

fn recalc_report_json(report: &GridEngineRecalcReport) -> Value {
    match report {
        GridEngineRecalcReport::Reference(report) => json!({
            "engine": "reference",
            "occupied_cells": report.occupied_cells,
            "literal_cells": report.literal_cells,
            "formula_cells": report.formula_cells,
            "cells_evaluated": report.cells_evaluated,
            "formula_evaluations": report.formula_evaluations,
            "spill_repair_passes": report.spill_repair_passes,
            "spill_repair_formula_evaluations": report.spill_repair_formula_evaluations,
            "spill_repair_converged": report.spill_repair_converged,
            "spill_facts_published": report.spill_facts_published,
            "spill_facts_blocked": report.spill_facts_blocked,
            "spill_ghost_cells_published": report.spill_ghost_cells_published,
            "visited_cells": report.visited_cells.iter().map(address_json).collect::<Vec<_>>(),
            "p00_non_spill_exact_once_holds": report.p00_non_spill_exact_once_holds()
        }),
        GridEngineRecalcReport::Optimized(report) => json!({
            "engine": "optimized",
            "occupied_cells": report.occupied_cells,
            "literal_cells": report.literal_cells,
            "formula_cells": report.formula_cells,
            "cells_evaluated": report.cells_evaluated,
            "formula_evaluations": report.formula_evaluations,
            "sparse_literal_cells": report.sparse_literal_cells,
            "sparse_formula_cells": report.sparse_formula_cells,
            "dense_value_region_cells": report.dense_value_region_cells,
            "repeated_formula_region_cells": report.repeated_formula_region_cells,
            "formula_templates_prepared": report.formula_templates_prepared,
            "distinct_formula_templates": report.distinct_formula_templates,
            "spill_repair_passes": report.spill_repair_passes,
            "spill_repair_formula_evaluations": report.spill_repair_formula_evaluations,
            "spill_repair_converged": report.spill_repair_converged,
            "computed_dense_value_regions": report.computed_dense_value_regions,
            "computed_sparse_cells": report.computed_sparse_cells,
            "spill_facts_published": report.spill_facts_published,
            "spill_facts_blocked": report.spill_facts_blocked,
            "spill_ghost_cells_published": report.spill_ghost_cells_published,
            "p00_primary_exact_once_holds": report.p00_primary_exact_once_holds(),
            "p11_template_prepare_once_holds": report.p11_template_prepare_once_holds()
        }),
    }
}

fn expected_value_json(expected: &GridExpectedCellValue) -> Value {
    json!({
        "address": address_json(&expected.address),
        "expected": calc_value_display_text(&expected.expected)
    })
}

fn expectation_mismatch_json(mismatch: &GridExpectedValueMismatch) -> Value {
    json!({
        "address": address_json(&mismatch.address),
        "expected": calc_value_display_text(&mismatch.expected),
        "reference": mismatch.reference.as_ref().map(calc_value_display_text),
        "optimized": mismatch.optimized.as_ref().map(calc_value_display_text)
    })
}

fn differential_mismatch_json(mismatch: &GridDifferentialMismatch) -> Value {
    json!({
        "address": address_json(&mismatch.address),
        "reference": calc_value_display_text(&mismatch.reference),
        "optimized": calc_value_display_text(&mismatch.optimized)
    })
}

fn invalidation_report_json(report: &GridInvalidationRunReport) -> Value {
    json!({
        "scalar_edge_count": report.scalar_edge_count,
        "spill_blocker_edge_count": report.spill_blocker_edge_count,
        "axis_value_edge_count": report.axis_value_edge_count,
        "name_edge_count": report.name_edge_count,
        "table_edge_count": report.table_edge_count,
        "mismatch_count": report.mismatch_count,
        "matched": report.matched(),
        "installed_dependencies": report.installed_dependencies.iter().map(installed_dependency_json).collect::<Vec<_>>(),
        "dirty_closures": report.dirty_closures.iter().map(dirty_closure_json).collect::<Vec<_>>(),
        "dynamic_closures": report.dynamic_closures.iter().map(dynamic_closure_json).collect::<Vec<_>>(),
        "spill_closures": report.spill_closures.iter().map(spill_closure_json).collect::<Vec<_>>(),
        "spill_blocker_closures": report.spill_blocker_closures.iter().map(spill_blocker_closure_json).collect::<Vec<_>>(),
        "axis_visibility_closures": report.axis_visibility_closures.iter().map(axis_visibility_closure_json).collect::<Vec<_>>(),
        "namespace_lifecycle": report.namespace_lifecycle.iter().map(namespace_lifecycle_json).collect::<Vec<_>>(),
        "structural_edits": report.structural_edits.iter().map(structural_invalidation_json).collect::<Vec<_>>()
    })
}

fn installed_dependency_json(report: &GridInstalledDependencyReport) -> Value {
    json!({
        "dependent": address_json(&report.dependent),
        "scalar_edge_count": report.scalar_edge_count,
        "scalar_dependencies": report.scalar_dependencies.iter().map(address_json).collect::<Vec<_>>(),
        "dynamic_dependencies": &report.dynamic_dependencies,
        "spill_dependencies": report.spill_dependencies.iter().map(spill_dependency_json).collect::<Vec<_>>(),
        "spill_blocker_dependencies": report.spill_blocker_dependencies.iter().map(spill_blocker_dependency_json).collect::<Vec<_>>(),
        "axis_value_dependencies": report.axis_value_dependencies.iter().map(axis_value_dependency_json).collect::<Vec<_>>(),
        "axis_visibility_dependencies": report.axis_visibility_dependencies.iter().map(axis_visibility_dependency_json).collect::<Vec<_>>(),
        "name_dependencies": report.name_dependencies.iter().map(name_dependency_json).collect::<Vec<_>>(),
        "table_dependencies": report.table_dependencies.iter().map(table_dependency_json).collect::<Vec<_>>()
    })
}

fn dirty_closure_json(report: &GridDirtyClosureReport) -> Value {
    json!({
        "check_id": &report.check_id,
        "seeds": report.seeds.iter().map(address_json).collect::<Vec<_>>(),
        "expected_dirty": report.expected_dirty.iter().map(address_json).collect::<Vec<_>>(),
        "actual_dirty": report.actual_dirty.iter().map(address_json).collect::<Vec<_>>(),
        "matched": report.matched
    })
}

fn dynamic_closure_json(report: &GridDynamicClosureReport) -> Value {
    json!({
        "check_id": &report.check_id,
        "request_key": &report.request_key,
        "expected_dirty": report.expected_dirty.iter().map(address_json).collect::<Vec<_>>(),
        "actual_dirty": report.actual_dirty.iter().map(address_json).collect::<Vec<_>>(),
        "matched": report.matched
    })
}

fn spill_closure_json(report: &GridSpillClosureReport) -> Value {
    json!({
        "check_id": &report.check_id,
        "dependency": spill_dependency_json(&report.dependency),
        "expected_dirty": report.expected_dirty.iter().map(address_json).collect::<Vec<_>>(),
        "actual_dirty": report.actual_dirty.iter().map(address_json).collect::<Vec<_>>(),
        "matched": report.matched
    })
}

fn spill_dependency_json(dependency: &GridSpillDependency) -> Value {
    json!({
        "anchor": address_json(&dependency.anchor)
    })
}

fn spill_blocker_closure_json(report: &GridSpillBlockerClosureReport) -> Value {
    json!({
        "check_id": &report.check_id,
        "dependency": spill_blocker_dependency_json(&report.dependency),
        "expected_dirty": report.expected_dirty.iter().map(address_json).collect::<Vec<_>>(),
        "actual_dirty": report.actual_dirty.iter().map(address_json).collect::<Vec<_>>(),
        "matched": report.matched
    })
}

fn spill_blocker_dependency_json(dependency: &GridSpillBlockerDependency) -> Value {
    json!({
        "extent": rect_json(&dependency.extent)
    })
}

fn name_dependency_json(dependency: &GridNameDependency) -> Value {
    json!({
        "name_key": dependency.name_key,
        "extent": rect_json(&dependency.extent)
    })
}

fn table_dependency_json(dependency: &GridTableDependency) -> Value {
    json!({
        "table_key": dependency.table_key,
        "extent": rect_json(&dependency.extent)
    })
}

fn axis_value_dependency_json(dependency: &GridAxisValueDependency) -> Value {
    json!({
        "axis": match dependency.axis {
            GridAxis::Row => "row",
            GridAxis::Column => "column",
        },
        "first": dependency.first,
        "last": dependency.last
    })
}

fn axis_visibility_closure_json(report: &GridAxisVisibilityClosureReport) -> Value {
    json!({
        "check_id": &report.check_id,
        "dependency": axis_visibility_dependency_json(&report.dependency),
        "expected_dirty": report.expected_dirty.iter().map(address_json).collect::<Vec<_>>(),
        "actual_dirty": report.actual_dirty.iter().map(address_json).collect::<Vec<_>>(),
        "matched": report.matched
    })
}

fn axis_visibility_dependency_json(dependency: &GridAxisVisibilityDependency) -> Value {
    json!({
        "axis": match dependency.axis {
            GridAxis::Row => "row",
            GridAxis::Column => "column",
        },
        "first": dependency.first,
        "last": dependency.last
    })
}

fn namespace_lifecycle_json(report: &GridNamespaceLifecycleReport) -> Value {
    json!({
        "check_id": &report.check_id,
        "operation": namespace_lifecycle_operation_json(&report.operation),
        "lifecycle_report": invalidation_namespace_lifecycle_report_json(&report.lifecycle_report),
        "expected_dirty": report.expected_dirty.iter().map(address_json).collect::<Vec<_>>(),
        "actual_dirty": report.actual_dirty.iter().map(address_json).collect::<Vec<_>>(),
        "matched": report.matched,
        "dirty_closures": report.dirty_closures.iter().map(dirty_closure_json).collect::<Vec<_>>(),
        "mismatch_count": report.mismatch_count
    })
}

fn namespace_lifecycle_operation_json(operation: &GridNamespaceLifecycleOperation) -> Value {
    match operation {
        GridNamespaceLifecycleOperation::RenameDefinedName { old_name, new_name } => json!({
            "kind": "rename_defined_name",
            "old_name": old_name,
            "new_name": new_name
        }),
        GridNamespaceLifecycleOperation::DeleteDefinedName { name } => json!({
            "kind": "delete_defined_name",
            "name": name
        }),
        GridNamespaceLifecycleOperation::RenameTable {
            old_table,
            new_table,
        } => json!({
            "kind": "rename_table",
            "old_table": old_table,
            "new_table": new_table
        }),
        GridNamespaceLifecycleOperation::DeleteTable { table } => json!({
            "kind": "delete_table",
            "table": table
        }),
        GridNamespaceLifecycleOperation::ResizeTable { table, new_extent } => json!({
            "kind": "resize_table",
            "table": table,
            "new_extent": rect_json(new_extent)
        }),
    }
}

fn invalidation_namespace_lifecycle_report_json(
    report: &GridInvalidationNamespaceLifecycleReport,
) -> Value {
    json!({
        "operation": invalidation_namespace_lifecycle_operation_json(&report.operation),
        "dirty_closure": report.dirty_closure.iter().map(address_json).collect::<Vec<_>>(),
        "semantic_dependencies_kept": report.semantic_dependencies_kept,
        "semantic_dependencies_dropped": report.semantic_dependencies_dropped,
        "name_edges_before": report.name_edges_before,
        "name_edges_after": report.name_edges_after,
        "table_edges_before": report.table_edges_before,
        "table_edges_after": report.table_edges_after
    })
}

fn invalidation_namespace_lifecycle_operation_json(
    operation: &GridInvalidationNamespaceLifecycleOperation,
) -> Value {
    match operation {
        GridInvalidationNamespaceLifecycleOperation::RenameName {
            old_name_key,
            new_name_key,
        } => json!({
            "kind": "rename_name",
            "old_name_key": old_name_key,
            "new_name_key": new_name_key
        }),
        GridInvalidationNamespaceLifecycleOperation::DeleteName { name_key } => json!({
            "kind": "delete_name",
            "name_key": name_key
        }),
        GridInvalidationNamespaceLifecycleOperation::RenameTable {
            old_table_key,
            new_table_key,
        } => json!({
            "kind": "rename_table",
            "old_table_key": old_table_key,
            "new_table_key": new_table_key
        }),
        GridInvalidationNamespaceLifecycleOperation::DeleteTable { table_key } => json!({
            "kind": "delete_table",
            "table_key": table_key
        }),
        GridInvalidationNamespaceLifecycleOperation::ResizeTable { table_key } => json!({
            "kind": "resize_table",
            "table_key": table_key
        }),
    }
}

fn structural_invalidation_json(report: &GridStructuralInvalidationReport) -> Value {
    json!({
        "check_id": &report.check_id,
        "edit": axis_edit_json(report.edit),
        "edit_report": invalidation_structural_edit_report_json(&report.edit_report),
        "dirty_closures": report.dirty_closures.iter().map(dirty_closure_json).collect::<Vec<_>>(),
        "dynamic_closures": report.dynamic_closures.iter().map(dynamic_closure_json).collect::<Vec<_>>(),
        "spill_closures": report.spill_closures.iter().map(spill_closure_json).collect::<Vec<_>>(),
        "spill_blocker_closures": report.spill_blocker_closures.iter().map(spill_blocker_closure_json).collect::<Vec<_>>(),
        "axis_visibility_closures": report.axis_visibility_closures.iter().map(axis_visibility_closure_json).collect::<Vec<_>>(),
        "mismatch_count": report.mismatch_count,
        "matched": report.mismatch_count == 0
    })
}

fn invalidation_structural_edit_report_json(
    report: &GridInvalidationStructuralEditReport,
) -> Value {
    json!({
        "edit": axis_edit_json(report.edit),
        "dependent_cells_kept": report.dependent_cells_kept,
        "dependent_cells_dropped": report.dependent_cells_dropped,
        "semantic_dependencies_kept": report.semantic_dependencies_kept,
        "semantic_dependencies_dropped": report.semantic_dependencies_dropped,
        "scalar_edges_before": report.scalar_edges_before,
        "scalar_edges_after": report.scalar_edges_after,
        "spill_edges_before": report.spill_edges_before,
        "spill_edges_after": report.spill_edges_after,
        "spill_blocker_edges_before": report.spill_blocker_edges_before,
        "spill_blocker_edges_after": report.spill_blocker_edges_after,
        "axis_value_edges_before": report.axis_value_edges_before,
        "axis_value_edges_after": report.axis_value_edges_after,
        "name_edges_before": report.name_edges_before,
        "name_edges_after": report.name_edges_after,
        "table_edges_before": report.table_edges_before,
        "table_edges_after": report.table_edges_after,
        "dynamic_edges_before": report.dynamic_edges_before,
        "dynamic_edges_after": report.dynamic_edges_after
    })
}

fn axis_edit_json(edit: GridAxisEdit) -> Value {
    let axis = match edit.axis {
        GridAxis::Row => "row",
        GridAxis::Column => "column",
    };
    match edit.kind {
        GridAxisEditKind::Insert { before, count } => json!({
            "axis": axis,
            "kind": "insert",
            "before": before,
            "count": count
        }),
        GridAxisEditKind::Delete { first, count } => json!({
            "axis": axis,
            "kind": "delete",
            "first": first,
            "count": count
        }),
    }
}

fn readout_json(readout: &GridEngineCellReadout) -> Value {
    json!({
        "address": address_json(&readout.address),
        "computed": calc_value_display_text(&readout.computed)
    })
}

fn address_json(address: &ExcelGridCellAddress) -> Value {
    json!({
        "workbook_id": &address.workbook_id,
        "sheet_id": &address.sheet_id,
        "row": address.row,
        "col": address.col,
        "r1c1": format!("R{}C{}", address.row, address.col)
    })
}

fn rect_json(rect: &GridRect) -> Value {
    json!({
        "workbook_id": &rect.workbook_id,
        "sheet_id": &rect.sheet_id,
        "top_row": rect.top_row,
        "left_col": rect.left_col,
        "bottom_row": rect.bottom_row,
        "right_col": rect.right_col,
        "a1": format!("R{}C{}:R{}C{}", rect.top_row, rect.left_col, rect.bottom_row, rect.right_col)
    })
}

fn reset_artifact_root(path: &Path) -> Result<(), GridRunnerError> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|source| GridRunnerError::RemoveDirectory {
            path: path.display().to_string(),
            source,
        })?;
    }
    Ok(())
}

fn create_directory(path: &Path) -> Result<(), GridRunnerError> {
    fs::create_dir_all(path).map_err(|source| GridRunnerError::CreateDirectory {
        path: path.display().to_string(),
        source,
    })
}

fn write_json(path: &Path, value: &Value) -> Result<(), GridRunnerError> {
    let text = serde_json::to_string_pretty(value).expect("grid artifact JSON should serialize");
    fs::write(path, text).map_err(|source| GridRunnerError::WriteFile {
        path: path.display().to_string(),
        source,
    })
}

fn grid_artifact_root_relative(run_id: &str) -> String {
    format!("docs/test-runs/core-engine/grid-seed/{run_id}")
}

fn run_invalidation_scenario(
    scenario: &GridInvalidationScenario,
) -> Result<GridInvalidationRunReport, GridRefError> {
    let mut invalidation = GridInvalidationRef::new(bounds());
    let mut installed_dependencies = Vec::new();
    let mut scalar_edge_count = 0;
    let mut spill_blocker_edge_count = 0;
    let mut axis_value_edge_count = 0;
    let mut name_edge_count = 0;
    let mut table_edge_count = 0;

    for dependencies in &scenario.dependencies {
        let edge_count = invalidation.set_cell_dependencies(
            dependencies.dependent.clone(),
            dependencies.dependencies.clone(),
        )?;
        scalar_edge_count += edge_count;
        let spill_blocker_dependencies = invalidation
            .spill_blocker_dependencies_for(&dependencies.dependent)
            .into_iter()
            .collect::<Vec<_>>();
        spill_blocker_edge_count += spill_blocker_dependencies.len();
        let axis_value_dependencies = invalidation
            .axis_value_dependencies_for(&dependencies.dependent)
            .into_iter()
            .collect::<Vec<_>>();
        axis_value_edge_count += axis_value_dependencies.len();
        let name_dependencies = invalidation
            .name_dependencies_for(&dependencies.dependent)
            .into_iter()
            .collect::<Vec<_>>();
        name_edge_count += name_dependencies.len();
        let table_dependencies = invalidation
            .table_dependencies_for(&dependencies.dependent)
            .into_iter()
            .collect::<Vec<_>>();
        table_edge_count += table_dependencies.len();
        installed_dependencies.push(GridInstalledDependencyReport {
            dependent: dependencies.dependent.clone(),
            scalar_edge_count: edge_count,
            scalar_dependencies: invalidation
                .dependencies_for(&dependencies.dependent)
                .into_iter()
                .collect(),
            dynamic_dependencies: invalidation
                .dynamic_dependencies_for(&dependencies.dependent)
                .into_iter()
                .collect(),
            spill_dependencies: invalidation
                .spill_dependencies_for(&dependencies.dependent)
                .into_iter()
                .collect(),
            spill_blocker_dependencies,
            axis_value_dependencies,
            name_dependencies,
            table_dependencies,
            axis_visibility_dependencies: invalidation
                .axis_visibility_dependencies_for(&dependencies.dependent)
                .into_iter()
                .collect(),
        });
    }

    let dirty_closures = scenario
        .dirty_closure_checks
        .iter()
        .map(|check| {
            let actual_dirty = invalidation
                .dirty_closure(check.seeds.iter().cloned())
                .into_iter()
                .collect::<Vec<_>>();
            let expected_dirty = sorted_addresses(check.expected_dirty.clone());
            let matched = actual_dirty == expected_dirty;
            GridDirtyClosureReport {
                check_id: check.check_id.clone(),
                seeds: sorted_addresses(check.seeds.clone()),
                expected_dirty,
                actual_dirty,
                matched,
            }
        })
        .collect::<Vec<_>>();

    let dynamic_closures = scenario
        .dynamic_closure_checks
        .iter()
        .map(|check| {
            let actual_dirty = invalidation
                .dirty_closure_for_dynamic_request(&check.request_key)
                .into_iter()
                .collect::<Vec<_>>();
            let expected_dirty = sorted_addresses(check.expected_dirty.clone());
            let matched = actual_dirty == expected_dirty;
            GridDynamicClosureReport {
                check_id: check.check_id.clone(),
                request_key: check.request_key.clone(),
                expected_dirty,
                actual_dirty,
                matched,
            }
        })
        .collect::<Vec<_>>();

    let spill_closures = scenario
        .spill_closure_checks
        .iter()
        .map(|check| {
            let actual_dirty = invalidation
                .dirty_closure_for_spill_fact(check.dependency.clone())?
                .into_iter()
                .collect::<Vec<_>>();
            let expected_dirty = sorted_addresses(check.expected_dirty.clone());
            let matched = actual_dirty == expected_dirty;
            Ok(GridSpillClosureReport {
                check_id: check.check_id.clone(),
                dependency: check.dependency.clone(),
                expected_dirty,
                actual_dirty,
                matched,
            })
        })
        .collect::<Result<Vec<_>, GridRefError>>()?;

    let spill_blocker_closures = scenario
        .spill_blocker_closure_checks
        .iter()
        .map(|check| {
            let actual_dirty = invalidation
                .dirty_closure_for_spill_blocker(check.dependency.clone())?
                .into_iter()
                .collect::<Vec<_>>();
            let expected_dirty = sorted_addresses(check.expected_dirty.clone());
            let matched = actual_dirty == expected_dirty;
            Ok(GridSpillBlockerClosureReport {
                check_id: check.check_id.clone(),
                dependency: check.dependency.clone(),
                expected_dirty,
                actual_dirty,
                matched,
            })
        })
        .collect::<Result<Vec<_>, GridRefError>>()?;

    let axis_visibility_closures = scenario
        .axis_visibility_closure_checks
        .iter()
        .map(|check| {
            let actual_dirty = invalidation
                .dirty_closure_for_axis_visibility(check.dependency.clone())?
                .into_iter()
                .collect::<Vec<_>>();
            let expected_dirty = sorted_addresses(check.expected_dirty.clone());
            let matched = actual_dirty == expected_dirty;
            Ok(GridAxisVisibilityClosureReport {
                check_id: check.check_id.clone(),
                dependency: check.dependency.clone(),
                expected_dirty,
                actual_dirty,
                matched,
            })
        })
        .collect::<Result<Vec<_>, GridRefError>>()?;

    let mut namespace_lifecycle = Vec::new();
    for check in &scenario.namespace_lifecycle_checks {
        let lifecycle_report = match &check.operation {
            GridNamespaceLifecycleOperation::RenameDefinedName { old_name, new_name } => {
                invalidation.rename_defined_name(old_name, new_name)?
            }
            GridNamespaceLifecycleOperation::DeleteDefinedName { name } => {
                invalidation.delete_defined_name(name)?
            }
            GridNamespaceLifecycleOperation::RenameTable {
                old_table,
                new_table,
            } => invalidation.rename_table(old_table, new_table)?,
            GridNamespaceLifecycleOperation::DeleteTable { table } => {
                invalidation.delete_table(table)?
            }
            GridNamespaceLifecycleOperation::ResizeTable { table, new_extent } => {
                invalidation.resize_table(table, new_extent.clone())?
            }
        };
        let actual_dirty = sorted_addresses(lifecycle_report.dirty_closure.clone());
        let expected_dirty = sorted_addresses(check.expected_dirty.clone());
        let matched = actual_dirty == expected_dirty;
        let dirty_closures = check
            .dirty_closure_checks
            .iter()
            .map(|check| {
                let actual_dirty = invalidation
                    .dirty_closure(check.seeds.iter().cloned())
                    .into_iter()
                    .collect::<Vec<_>>();
                let expected_dirty = sorted_addresses(check.expected_dirty.clone());
                let matched = actual_dirty == expected_dirty;
                GridDirtyClosureReport {
                    check_id: check.check_id.clone(),
                    seeds: sorted_addresses(check.seeds.clone()),
                    expected_dirty,
                    actual_dirty,
                    matched,
                }
            })
            .collect::<Vec<_>>();
        let mismatch_count = usize::from(!matched)
            + dirty_closures
                .iter()
                .filter(|report| !report.matched)
                .count();
        namespace_lifecycle.push(GridNamespaceLifecycleReport {
            check_id: check.check_id.clone(),
            operation: check.operation.clone(),
            lifecycle_report,
            expected_dirty,
            actual_dirty,
            matched,
            dirty_closures,
            mismatch_count,
        });
    }

    let mut structural_edits = Vec::new();
    for check in &scenario.structural_edit_checks {
        let edit_report = invalidation.apply_axis_edit(check.edit)?;
        let dirty_closures = check
            .dirty_closure_checks
            .iter()
            .map(|check| {
                let actual_dirty = invalidation
                    .dirty_closure(check.seeds.iter().cloned())
                    .into_iter()
                    .collect::<Vec<_>>();
                let expected_dirty = sorted_addresses(check.expected_dirty.clone());
                let matched = actual_dirty == expected_dirty;
                GridDirtyClosureReport {
                    check_id: check.check_id.clone(),
                    seeds: sorted_addresses(check.seeds.clone()),
                    expected_dirty,
                    actual_dirty,
                    matched,
                }
            })
            .collect::<Vec<_>>();
        let dynamic_closures = check
            .dynamic_closure_checks
            .iter()
            .map(|check| {
                let actual_dirty = invalidation
                    .dirty_closure_for_dynamic_request(&check.request_key)
                    .into_iter()
                    .collect::<Vec<_>>();
                let expected_dirty = sorted_addresses(check.expected_dirty.clone());
                let matched = actual_dirty == expected_dirty;
                GridDynamicClosureReport {
                    check_id: check.check_id.clone(),
                    request_key: check.request_key.clone(),
                    expected_dirty,
                    actual_dirty,
                    matched,
                }
            })
            .collect::<Vec<_>>();
        let spill_closures = check
            .spill_closure_checks
            .iter()
            .map(|check| {
                let actual_dirty = invalidation
                    .dirty_closure_for_spill_fact(check.dependency.clone())?
                    .into_iter()
                    .collect::<Vec<_>>();
                let expected_dirty = sorted_addresses(check.expected_dirty.clone());
                let matched = actual_dirty == expected_dirty;
                Ok(GridSpillClosureReport {
                    check_id: check.check_id.clone(),
                    dependency: check.dependency.clone(),
                    expected_dirty,
                    actual_dirty,
                    matched,
                })
            })
            .collect::<Result<Vec<_>, GridRefError>>()?;
        let spill_blocker_closures = check
            .spill_blocker_closure_checks
            .iter()
            .map(|check| {
                let actual_dirty = invalidation
                    .dirty_closure_for_spill_blocker(check.dependency.clone())?
                    .into_iter()
                    .collect::<Vec<_>>();
                let expected_dirty = sorted_addresses(check.expected_dirty.clone());
                let matched = actual_dirty == expected_dirty;
                Ok(GridSpillBlockerClosureReport {
                    check_id: check.check_id.clone(),
                    dependency: check.dependency.clone(),
                    expected_dirty,
                    actual_dirty,
                    matched,
                })
            })
            .collect::<Result<Vec<_>, GridRefError>>()?;
        let axis_visibility_closures = check
            .axis_visibility_closure_checks
            .iter()
            .map(|check| {
                let actual_dirty = invalidation
                    .dirty_closure_for_axis_visibility(check.dependency.clone())?
                    .into_iter()
                    .collect::<Vec<_>>();
                let expected_dirty = sorted_addresses(check.expected_dirty.clone());
                let matched = actual_dirty == expected_dirty;
                Ok(GridAxisVisibilityClosureReport {
                    check_id: check.check_id.clone(),
                    dependency: check.dependency.clone(),
                    expected_dirty,
                    actual_dirty,
                    matched,
                })
            })
            .collect::<Result<Vec<_>, GridRefError>>()?;
        let mismatch_count = dirty_closures
            .iter()
            .filter(|report| !report.matched)
            .count()
            + dynamic_closures
                .iter()
                .filter(|report| !report.matched)
                .count()
            + spill_closures
                .iter()
                .filter(|report| !report.matched)
                .count()
            + spill_blocker_closures
                .iter()
                .filter(|report| !report.matched)
                .count()
            + axis_visibility_closures
                .iter()
                .filter(|report| !report.matched)
                .count();
        structural_edits.push(GridStructuralInvalidationReport {
            check_id: check.check_id.clone(),
            edit: check.edit,
            edit_report,
            dirty_closures,
            dynamic_closures,
            spill_closures,
            spill_blocker_closures,
            axis_visibility_closures,
            mismatch_count,
        });
    }

    let mismatch_count = dirty_closures
        .iter()
        .filter(|report| !report.matched)
        .count()
        + dynamic_closures
            .iter()
            .filter(|report| !report.matched)
            .count()
        + spill_closures
            .iter()
            .filter(|report| !report.matched)
            .count()
        + spill_blocker_closures
            .iter()
            .filter(|report| !report.matched)
            .count()
        + axis_visibility_closures
            .iter()
            .filter(|report| !report.matched)
            .count()
        + namespace_lifecycle
            .iter()
            .map(|report| report.mismatch_count)
            .sum::<usize>()
        + structural_edits
            .iter()
            .map(|report| report.mismatch_count)
            .sum::<usize>();

    Ok(GridInvalidationRunReport {
        scalar_edge_count,
        spill_blocker_edge_count,
        axis_value_edge_count,
        name_edge_count,
        table_edge_count,
        installed_dependencies,
        dirty_closures,
        dynamic_closures,
        spill_closures,
        spill_blocker_closures,
        axis_visibility_closures,
        namespace_lifecycle,
        structural_edits,
        mismatch_count,
    })
}

fn sorted_addresses(
    addresses: impl IntoIterator<Item = ExcelGridCellAddress>,
) -> Vec<ExcelGridCellAddress> {
    addresses
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn seed_corpus_scenarios() -> Result<Vec<GridCorpusScenario>, GridRefError> {
    Ok(vec![
        sparse_formula_chain()?,
        dense_values_only()?,
        dense_values_sparse_sum_formula()?,
        defined_name_value_resolution()?,
        table_overlay_structured_reference_value_resolution()?,
        table_overlay_current_row_structured_reference_context()?,
        table_overlay_structured_sections_and_escaped_columns()?,
        repeated_r1c1_formula_region()?,
        optimized_structural_edit_repeated_r1c1_region()?,
        hidden_row_subtotal_visibility_invalidation()?,
        whole_axis_value_invalidation()?,
        spill_anchor_ledger_invalidation()?,
        dynamic_sequence_spill_publication()?,
        dynamic_sequence_spill_repair()?,
        mutual_sequence_spill_blockage()?,
        table_overlay_spill_blockage()?,
        merged_region_spill_blockage()?,
        dynamic_invalidation_request()?,
    ])
}

fn sparse_formula_chain() -> Result<GridCorpusScenario, GridRefError> {
    let mut sheet = sheet();
    sheet.set_literal(address(1, 1), CalcValue::number(7.0))?;
    sheet.set_formula(
        address(1, 2),
        GridFormulaCell::new("=A1*3", "excel.grid.v1:cell:R[0]C[-1]*3"),
    )?;
    sheet.set_formula(
        address(1, 3),
        GridFormulaCell::new("=SUM(A1:B1)", "excel.grid.v1:sum:R[0]C[-2]:R[0]C[-1]"),
    )?;
    let probes = vec![address(1, 1), address(1, 2), address(1, 3)];
    Ok(GridCorpusScenario {
        case_id: "grid_seed_sparse_formula_chain_001".to_string(),
        description: "Sparse point literals and formulas share values across reference and optimized engines."
            .to_string(),
        tags: strings(["sparse", "a1", "formula"]),
        sheet,
        probes: probes.clone(),
        expected_values: vec![
            expected(address(1, 1), CalcValue::number(7.0)),
            expected(address(1, 2), CalcValue::number(21.0)),
            expected(address(1, 3), CalcValue::number(28.0)),
        ],
        invalidation: Some(GridInvalidationScenario {
            dependencies: vec![
                cell_dependencies(address(1, 2), [GridDependency::Cell(address(1, 1))]),
                cell_dependencies(
                    address(1, 3),
                    [GridDependency::Range(rect(1, 1, 1, 2)?)],
                ),
            ],
            dirty_closure_checks: vec![
                dirty_check(
                    "edit-a1-dirties-chain",
                    [address(1, 1)],
                    [address(1, 1), address(1, 2), address(1, 3)],
                ),
                dirty_check(
                    "edit-b1-dirties-sum",
                    [address(1, 2)],
                    [address(1, 2), address(1, 3)],
                ),
            ],
            dynamic_closure_checks: Vec::new(),
            spill_closure_checks: Vec::new(),
            spill_blocker_closure_checks: Vec::new(),
            axis_visibility_closure_checks: Vec::new(),
            namespace_lifecycle_checks: Vec::new(),
            structural_edit_checks: vec![GridStructuralInvalidationCheck {
                check_id: "insert-column-before-chain-shifts-closure".to_string(),
                edit: GridAxisEdit::insert_columns(1, 1),
                dirty_closure_checks: vec![dirty_check(
                    "post-insert-edit-b1-dirties-shifted-chain",
                    [address(1, 2)],
                    [address(1, 2), address(1, 3), address(1, 4)],
                )],
                dynamic_closure_checks: Vec::new(),
                spill_closure_checks: Vec::new(),
                spill_blocker_closure_checks: Vec::new(),
                axis_visibility_closure_checks: Vec::new(),
            }],
        }),
    })
}

fn dense_values_only() -> Result<GridCorpusScenario, GridRefError> {
    let mut sheet = sheet();
    sheet.put_dense_literal_region(
        rect(1, 1, 4, 4)?,
        (1..=4)
            .flat_map(|row| (1..=4).map(move |col| CalcValue::number(f64::from((row * 100) + col))))
            .collect(),
    )?;
    let probes = vec![address(1, 1), address(2, 3), address(4, 4)];
    Ok(GridCorpusScenario {
        case_id: "grid_seed_dense_values_only_001".to_string(),
        description: "Dense row-major values stay region-backed in the optimized valuation."
            .to_string(),
        tags: strings(["dense-values", "values-only"]),
        sheet,
        probes: probes.clone(),
        expected_values: vec![
            expected(address(1, 1), CalcValue::number(101.0)),
            expected(address(2, 3), CalcValue::number(203.0)),
            expected(address(4, 4), CalcValue::number(404.0)),
        ],
        invalidation: Some(GridInvalidationScenario {
            dependencies: Vec::new(),
            dirty_closure_checks: vec![dirty_check(
                "literal-edit-dirties-only-self",
                [address(2, 3)],
                [address(2, 3)],
            )],
            dynamic_closure_checks: Vec::new(),
            spill_closure_checks: Vec::new(),
            spill_blocker_closure_checks: Vec::new(),
            axis_visibility_closure_checks: Vec::new(),
            namespace_lifecycle_checks: Vec::new(),
            structural_edit_checks: Vec::new(),
        }),
    })
}

fn dense_values_sparse_sum_formula() -> Result<GridCorpusScenario, GridRefError> {
    let mut sheet = sheet();
    sheet.put_dense_literal_region(
        rect(1, 1, 4, 1)?,
        vec![
            CalcValue::number(2.0),
            CalcValue::number(3.0),
            CalcValue::number(5.0),
            CalcValue::number(7.0),
        ],
    )?;
    sheet.set_formula(
        address(1, 2),
        GridFormulaCell::new("=SUM(A1:A4)", "excel.grid.v1:sum:R[0]C[-1]:R[3]C[-1]"),
    )?;
    let probes = vec![address(1, 1), address(4, 1), address(1, 2)];
    Ok(GridCorpusScenario {
        case_id: "grid_seed_dense_values_sparse_sum_001".to_string(),
        description:
            "A sparse aggregate formula reads a dense value region through the grid provider."
                .to_string(),
        tags: strings(["dense-values", "sparse-formula", "a1"]),
        sheet,
        probes: probes.clone(),
        expected_values: vec![
            expected(address(1, 1), CalcValue::number(2.0)),
            expected(address(4, 1), CalcValue::number(7.0)),
            expected(address(1, 2), CalcValue::number(17.0)),
        ],
        invalidation: Some(GridInvalidationScenario {
            dependencies: vec![cell_dependencies(
                address(1, 2),
                [GridDependency::Range(rect(1, 1, 4, 1)?)],
            )],
            dirty_closure_checks: vec![
                dirty_check(
                    "edit-a2-dirties-aggregate",
                    [address(2, 1)],
                    [address(2, 1), address(1, 2)],
                ),
                dirty_check(
                    "edit-a4-dirties-aggregate",
                    [address(4, 1)],
                    [address(4, 1), address(1, 2)],
                ),
            ],
            dynamic_closure_checks: Vec::new(),
            spill_closure_checks: Vec::new(),
            spill_blocker_closure_checks: Vec::new(),
            axis_visibility_closure_checks: Vec::new(),
            namespace_lifecycle_checks: Vec::new(),
            structural_edit_checks: vec![GridStructuralInvalidationCheck {
                check_id: "insert-row-inside-aggregate-range-expands-closure".to_string(),
                edit: GridAxisEdit::insert_rows(2, 1),
                dirty_closure_checks: vec![dirty_check(
                    "post-insert-edit-a2-dirties-expanded-aggregate",
                    [address(2, 1)],
                    [address(2, 1), address(1, 2)],
                )],
                dynamic_closure_checks: Vec::new(),
                spill_closure_checks: Vec::new(),
                spill_blocker_closure_checks: Vec::new(),
                axis_visibility_closure_checks: Vec::new(),
            }],
        }),
    })
}

fn defined_name_value_resolution() -> Result<GridCorpusScenario, GridRefError> {
    let mut sheet = sheet();
    let input_range = rect(1, 1, 3, 1)?;
    sheet.put_dense_literal_region(
        input_range.clone(),
        vec![
            CalcValue::number(2.0),
            CalcValue::number(4.0),
            CalcValue::number(6.0),
        ],
    )?;
    sheet.set_defined_name("InputRange", input_range)?;
    sheet.set_formula(
        address(1, 2),
        GridFormulaCell::new("=SUM(InputRange)", "excel.grid.v1:sum-name:InputRange"),
    )?;
    sheet.set_formula(
        address(1, 3),
        GridFormulaCell::new(
            "=SUM(INDIRECT(\"InputRange\"))",
            "excel.grid.v1:sum-indirect-name:InputRange",
        ),
    )?;
    let probes = vec![address(1, 2), address(1, 3)];
    Ok(GridCorpusScenario {
        case_id: "grid_seed_defined_name_value_resolution_001".to_string(),
        description:
            "Defined names resolve through the grid provider for symbolic formulas and INDIRECT text."
                .to_string(),
        tags: strings(["defined-name", "name-namespace", "indirect", "sparse-formula"]),
        sheet,
        probes: probes.clone(),
        expected_values: vec![
            expected(address(1, 2), CalcValue::number(12.0)),
            expected(address(1, 3), CalcValue::number(12.0)),
        ],
        invalidation: Some(GridInvalidationScenario {
            dependencies: vec![
                cell_dependencies(
                    address(1, 2),
                    [GridDependency::Name(GridNameDependency::new(
                        "InputRange",
                        rect(1, 1, 3, 1)?,
                        bounds(),
                    )?)],
                ),
                cell_dependencies(
                    address(1, 3),
                    [GridDependency::Name(GridNameDependency::new(
                        "InputRange",
                        rect(1, 1, 3, 1)?,
                        bounds(),
                    )?)],
                ),
            ],
            dirty_closure_checks: vec![dirty_check(
                "edit-a2-dirties-defined-name-consumers",
                [address(2, 1)],
                [address(2, 1), address(1, 2), address(1, 3)],
            )],
            dynamic_closure_checks: Vec::new(),
            spill_closure_checks: Vec::new(),
            spill_blocker_closure_checks: Vec::new(),
            axis_visibility_closure_checks: Vec::new(),
            namespace_lifecycle_checks: vec![
                namespace_lifecycle_check(
                    "rename-inputrange-retargets-name-dependencies",
                    GridNamespaceLifecycleOperation::RenameDefinedName {
                        old_name: "InputRange".to_string(),
                        new_name: "DataRange".to_string(),
                    },
                    [address(1, 2), address(1, 3)],
                    [dirty_check(
                        "post-rename-edit-a2-dirties-data-range-consumers",
                        [address(2, 1)],
                        [address(2, 1), address(1, 2), address(1, 3)],
                    )],
                ),
                namespace_lifecycle_check(
                    "delete-datarange-drops-name-dependencies",
                    GridNamespaceLifecycleOperation::DeleteDefinedName {
                        name: "DataRange".to_string(),
                    },
                    [address(1, 2), address(1, 3)],
                    [dirty_check(
                        "post-delete-edit-a2-no-longer-dirties-name-consumers",
                        [address(2, 1)],
                        [address(2, 1)],
                    )],
                ),
            ],
            structural_edit_checks: Vec::new(),
        }),
    })
}

fn table_overlay_structured_reference_value_resolution() -> Result<GridCorpusScenario, GridRefError>
{
    let mut sheet = sheet();
    sheet.put_dense_literal_region(
        rect(1, 1, 4, 2)?,
        vec![
            CalcValue::empty(),
            CalcValue::empty(),
            CalcValue::empty(),
            CalcValue::number(2.0),
            CalcValue::empty(),
            CalcValue::number(4.0),
            CalcValue::empty(),
            CalcValue::number(6.0),
        ],
    )?;
    let amount_data = rect(2, 2, 4, 2)?;
    sheet.set_table_overlay(
        GridTableOverlay::new(
            "table:grid-seed:table1",
            "Table1",
            rect(1, 1, 4, 2)?,
            vec![
                GridTableColumn::new("table1:label", "Label", 1, rect(2, 1, 4, 1)?),
                GridTableColumn::new("table1:amount", "Amount", 2, amount_data.clone()),
            ],
        )
        .with_header_rect(rect(1, 1, 1, 2)?),
    )?;
    sheet.set_formula(
        address(1, 3),
        GridFormulaCell::new(
            "=SUM(Table1[Amount])",
            "excel.grid.v1:sum-table:Table1[Amount]",
        ),
    )?;
    sheet.set_formula(
        address(1, 4),
        GridFormulaCell::new(
            "=SUM(INDIRECT(\"Table1[Amount]\"))",
            "excel.grid.v1:sum-indirect-table:Table1[Amount]",
        ),
    )?;
    let probes = vec![address(1, 3), address(1, 4)];
    Ok(GridCorpusScenario {
        case_id: "grid_seed_table_overlay_structured_reference_001".to_string(),
        description:
            "Table overlays resolve explicit structured references through the grid provider and text resolver."
                .to_string(),
        tags: strings([
            "table-overlay",
            "structured-reference",
            "table-dependency",
            "indirect",
            "sparse-formula",
        ]),
        sheet,
        probes: probes.clone(),
        expected_values: vec![
            expected(address(1, 3), CalcValue::number(12.0)),
            expected(address(1, 4), CalcValue::number(12.0)),
        ],
        invalidation: Some(GridInvalidationScenario {
            dependencies: vec![
                cell_dependencies(
                    address(1, 3),
                    [GridDependency::Table(GridTableDependency::new(
                        "Table1",
                        amount_data.clone(),
                        bounds(),
                    )?)],
                ),
                cell_dependencies(
                    address(1, 4),
                    [GridDependency::Table(GridTableDependency::new(
                        "Table1",
                        amount_data,
                        bounds(),
                    )?)],
                ),
            ],
            dirty_closure_checks: vec![dirty_check(
                "edit-b3-dirties-table-consumers",
                [address(3, 2)],
                [address(3, 2), address(1, 3), address(1, 4)],
            )],
            dynamic_closure_checks: Vec::new(),
            spill_closure_checks: Vec::new(),
            spill_blocker_closure_checks: Vec::new(),
            axis_visibility_closure_checks: Vec::new(),
            namespace_lifecycle_checks: vec![
                namespace_lifecycle_check(
                    "rename-table1-retargets-table-dependencies",
                    GridNamespaceLifecycleOperation::RenameTable {
                        old_table: "Table1".to_string(),
                        new_table: "Sales".to_string(),
                    },
                    [address(1, 3), address(1, 4)],
                    Vec::new(),
                ),
                namespace_lifecycle_check(
                    "resize-sales-rebuilds-table-scalar-edges",
                    GridNamespaceLifecycleOperation::ResizeTable {
                        table: "Sales".to_string(),
                        new_extent: rect(2, 2, 5, 2)?,
                    },
                    [address(1, 3), address(1, 4)],
                    [dirty_check(
                        "post-resize-edit-b5-dirties-table-consumers",
                        [address(5, 2)],
                        [address(5, 2), address(1, 3), address(1, 4)],
                    )],
                ),
                namespace_lifecycle_check(
                    "delete-sales-drops-table-dependencies",
                    GridNamespaceLifecycleOperation::DeleteTable {
                        table: "Sales".to_string(),
                    },
                    [address(1, 3), address(1, 4)],
                    [dirty_check(
                        "post-delete-edit-b3-no-longer-dirties-table-consumers",
                        [address(3, 2)],
                        [address(3, 2)],
                    )],
                ),
            ],
            structural_edit_checks: Vec::new(),
        }),
    })
}

fn table_overlay_current_row_structured_reference_context()
-> Result<GridCorpusScenario, GridRefError> {
    let mut sheet = sheet();
    sheet.put_dense_literal_region(
        rect(1, 1, 4, 2)?,
        vec![
            CalcValue::empty(),
            CalcValue::empty(),
            CalcValue::empty(),
            CalcValue::number(2.0),
            CalcValue::empty(),
            CalcValue::number(4.0),
            CalcValue::empty(),
            CalcValue::number(6.0),
        ],
    )?;
    sheet.set_table_overlay(
        GridTableOverlay::new(
            "table:grid-seed:current-row",
            "Table1",
            rect(1, 1, 4, 4)?,
            vec![
                GridTableColumn::new("table1:label", "Label", 1, rect(2, 1, 4, 1)?),
                GridTableColumn::new("table1:amount", "Amount", 2, rect(2, 2, 4, 2)?),
                GridTableColumn::new("table1:double", "Double", 3, rect(2, 3, 4, 3)?),
                GridTableColumn::new("table1:amount-total", "AmountTotal", 4, rect(2, 4, 4, 4)?),
            ],
        )
        .with_header_rect(rect(1, 1, 1, 4)?),
    )?;
    sheet.put_repeated_formula_region(
        rect(2, 3, 4, 3)?,
        GridFormulaCell::new("=[@Amount]*2", "excel.grid.v1:table-current-row:Amount*2"),
    )?;
    sheet.put_repeated_formula_region(
        rect(2, 4, 4, 4)?,
        GridFormulaCell::new("=SUM([Amount])", "excel.grid.v1:table-omitted:sum:Amount"),
    )?;
    let probes = vec![
        address(2, 3),
        address(3, 3),
        address(4, 3),
        address(2, 4),
        address(3, 4),
        address(4, 4),
    ];
    Ok(GridCorpusScenario {
        case_id: "grid_seed_table_overlay_current_row_structured_reference_001".to_string(),
        description:
            "Table formulas inside overlays receive caller table context for current-row structured references."
                .to_string(),
        tags: strings([
            "table-overlay",
            "structured-reference",
            "current-row",
            "caller-context",
            "omitted-table",
            "repeated-formula",
        ]),
        sheet,
        probes: probes.clone(),
        expected_values: vec![
            expected(address(2, 3), CalcValue::number(4.0)),
            expected(address(3, 3), CalcValue::number(8.0)),
            expected(address(4, 3), CalcValue::number(12.0)),
            expected(address(2, 4), CalcValue::number(12.0)),
            expected(address(3, 4), CalcValue::number(12.0)),
            expected(address(4, 4), CalcValue::number(12.0)),
        ],
        invalidation: Some(GridInvalidationScenario {
            dependencies: vec![
                cell_dependencies(address(2, 3), [GridDependency::Cell(address(2, 2))]),
                cell_dependencies(address(3, 3), [GridDependency::Cell(address(3, 2))]),
                cell_dependencies(address(4, 3), [GridDependency::Cell(address(4, 2))]),
                cell_dependencies(
                    address(2, 4),
                    [GridDependency::Table(GridTableDependency::new(
                        "Table1",
                        rect(2, 2, 4, 2)?,
                        bounds(),
                    )?)],
                ),
                cell_dependencies(
                    address(3, 4),
                    [GridDependency::Table(GridTableDependency::new(
                        "Table1",
                        rect(2, 2, 4, 2)?,
                        bounds(),
                    )?)],
                ),
                cell_dependencies(
                    address(4, 4),
                    [GridDependency::Table(GridTableDependency::new(
                        "Table1",
                        rect(2, 2, 4, 2)?,
                        bounds(),
                    )?)],
                ),
            ],
            dirty_closure_checks: vec![
                dirty_check(
                    "edit-b2-dirties-current-row-formula",
                    [address(2, 2)],
                    [address(2, 2), address(2, 3), address(2, 4), address(3, 4), address(4, 4)],
                ),
                dirty_check(
                    "edit-b3-dirties-current-row-and-omitted-formulas",
                    [address(3, 2)],
                    [address(3, 2), address(3, 3), address(2, 4), address(3, 4), address(4, 4)],
                ),
            ],
            dynamic_closure_checks: Vec::new(),
            spill_closure_checks: Vec::new(),
            spill_blocker_closure_checks: Vec::new(),
            axis_visibility_closure_checks: Vec::new(),
            namespace_lifecycle_checks: Vec::new(),
            structural_edit_checks: Vec::new(),
        }),
    })
}

fn table_overlay_structured_sections_and_escaped_columns()
-> Result<GridCorpusScenario, GridRefError> {
    let mut sheet = sheet();
    sheet.put_dense_literal_region(
        rect(1, 1, 5, 3)?,
        vec![
            CalcValue::empty(),
            CalcValue::empty(),
            CalcValue::empty(),
            CalcValue::empty(),
            CalcValue::number(2.0),
            CalcValue::number(1.0),
            CalcValue::empty(),
            CalcValue::number(4.0),
            CalcValue::number(2.0),
            CalcValue::empty(),
            CalcValue::number(6.0),
            CalcValue::number(3.0),
            CalcValue::empty(),
            CalcValue::number(12.0),
            CalcValue::number(6.0),
        ],
    )?;
    sheet.set_table_overlay(
        GridTableOverlay::new(
            "table:grid-seed:table1-wide",
            "Table1",
            rect(1, 1, 5, 3)?,
            vec![
                GridTableColumn::new("table1:label", "Label", 1, rect(2, 1, 4, 1)?),
                GridTableColumn::new("table1:amount", "Amount", 2, rect(2, 2, 4, 2)?),
                GridTableColumn::new("table1:tax", "Tax", 3, rect(2, 3, 4, 3)?),
            ],
        )
        .with_header_rect(rect(1, 1, 1, 3)?)
        .with_totals_rect(rect(5, 1, 5, 3)?),
    )?;
    sheet.put_dense_literal_region(
        rect(6, 1, 9, 3)?,
        vec![
            CalcValue::empty(),
            CalcValue::empty(),
            CalcValue::empty(),
            CalcValue::empty(),
            CalcValue::number(2.0),
            CalcValue::number(1.0),
            CalcValue::empty(),
            CalcValue::number(4.0),
            CalcValue::number(2.0),
            CalcValue::empty(),
            CalcValue::number(6.0),
            CalcValue::number(3.0),
        ],
    )?;
    sheet.set_table_overlay(
        GridTableOverlay::new(
            "table:grid-seed:escaped",
            "TableEsc",
            rect(6, 1, 9, 3)?,
            vec![
                GridTableColumn::new("table-esc:label", "Label", 1, rect(7, 1, 9, 1)?),
                GridTableColumn::new("table-esc:hash-data", "#Data", 2, rect(7, 2, 9, 2)?),
                GridTableColumn::new(
                    "table-esc:gross-margin",
                    "Gross]Margin",
                    3,
                    rect(7, 3, 9, 3)?,
                ),
            ],
        )
        .with_header_rect(rect(6, 1, 6, 3)?),
    )?;
    sheet.set_formula(
        address(1, 4),
        GridFormulaCell::new(
            "=SUM(Table1[[#Data],[Amount]:[Tax]])",
            "excel.grid.v1:sum-table-data:Table1[Amount:Tax]",
        ),
    )?;
    sheet.set_formula(
        address(2, 4),
        GridFormulaCell::new(
            "=SUM(INDIRECT(\"Table1[[#Data],[Amount]:[Tax]]\"))",
            "excel.grid.v1:sum-indirect-table-data:Table1[Amount:Tax]",
        ),
    )?;
    sheet.set_formula(
        address(3, 4),
        GridFormulaCell::new(
            "=SUM(Table1[[#Totals],[Amount]:[Tax]])",
            "excel.grid.v1:sum-table-totals:Table1[Amount:Tax]",
        ),
    )?;
    sheet.set_formula(
        address(4, 4),
        GridFormulaCell::new(
            "=SUM(Table1[[#All],[Amount]:[Tax]])",
            "excel.grid.v1:sum-table-all:Table1[Amount:Tax]",
        ),
    )?;
    sheet.set_formula(
        address(5, 4),
        GridFormulaCell::new(
            "=SUM(TableEsc['#Data])",
            "excel.grid.v1:sum-table-escaped:TableEsc[#Data]",
        ),
    )?;
    sheet.set_formula(
        address(6, 4),
        GridFormulaCell::new(
            "=SUM(TableEsc[[#Data],['#Data]:[Gross']Margin]])",
            "excel.grid.v1:sum-table-escaped-range:TableEsc[#Data:GrossMargin]",
        ),
    )?;
    sheet.set_formula(
        address(7, 4),
        GridFormulaCell::new(
            "=SUM(Table1[[#Headers],[#Totals],[Amount]:[Tax]])",
            "excel.grid.v1:sum-table-union:Table1[Headers+Totals,Amount:Tax]",
        ),
    )?;
    sheet.set_formula(
        address(8, 4),
        GridFormulaCell::new(
            "=SUM(INDIRECT(\"Table1[[#Headers],[#Totals],[Amount]:[Tax]]\"))",
            "excel.grid.v1:sum-indirect-table-union:Table1[Headers+Totals,Amount:Tax]",
        ),
    )?;
    let probes = vec![
        address(1, 4),
        address(2, 4),
        address(3, 4),
        address(4, 4),
        address(5, 4),
        address(6, 4),
        address(7, 4),
        address(8, 4),
    ];
    Ok(GridCorpusScenario {
        case_id: "grid_seed_table_overlay_structured_sections_001".to_string(),
        description:
            "Table overlays resolve section-qualified, multi-column, and escaped structured references."
                .to_string(),
        tags: strings([
            "table-overlay",
            "structured-reference",
            "section-qualified",
            "multi-area",
            "escaped-column",
            "table-dependency",
            "indirect",
        ]),
        sheet,
        probes: probes.clone(),
        expected_values: vec![
            expected(address(1, 4), CalcValue::number(18.0)),
            expected(address(2, 4), CalcValue::number(18.0)),
            expected(address(3, 4), CalcValue::number(18.0)),
            expected(address(4, 4), CalcValue::number(36.0)),
            expected(address(5, 4), CalcValue::number(12.0)),
            expected(address(6, 4), CalcValue::number(18.0)),
            expected(address(7, 4), CalcValue::number(18.0)),
            expected(address(8, 4), CalcValue::number(18.0)),
        ],
        invalidation: Some(GridInvalidationScenario {
            dependencies: vec![
                cell_dependencies(
                    address(1, 4),
                    [GridDependency::Table(GridTableDependency::new(
                        "Table1",
                        rect(2, 2, 4, 3)?,
                        bounds(),
                    )?)],
                ),
                cell_dependencies(
                    address(2, 4),
                    [GridDependency::Table(GridTableDependency::new(
                        "Table1",
                        rect(2, 2, 4, 3)?,
                        bounds(),
                    )?)],
                ),
                cell_dependencies(
                    address(3, 4),
                    [GridDependency::Table(GridTableDependency::new(
                        "Table1",
                        rect(5, 2, 5, 3)?,
                        bounds(),
                    )?)],
                ),
                cell_dependencies(
                    address(4, 4),
                    [GridDependency::Table(GridTableDependency::new(
                        "Table1",
                        rect(1, 2, 5, 3)?,
                        bounds(),
                    )?)],
                ),
                cell_dependencies(
                    address(5, 4),
                    [GridDependency::Table(GridTableDependency::new(
                        "TableEsc",
                        rect(7, 2, 9, 2)?,
                        bounds(),
                    )?)],
                ),
                cell_dependencies(
                    address(6, 4),
                    [GridDependency::Table(GridTableDependency::new(
                        "TableEsc",
                        rect(7, 2, 9, 3)?,
                        bounds(),
                    )?)],
                ),
                cell_dependencies(
                    address(7, 4),
                    [
                        GridDependency::Table(GridTableDependency::new(
                            "Table1",
                            rect(1, 2, 1, 3)?,
                            bounds(),
                        )?),
                        GridDependency::Table(GridTableDependency::new(
                            "Table1",
                            rect(5, 2, 5, 3)?,
                            bounds(),
                        )?),
                    ],
                ),
                cell_dependencies(
                    address(8, 4),
                    [
                        GridDependency::Table(GridTableDependency::new(
                            "Table1",
                            rect(1, 2, 1, 3)?,
                            bounds(),
                        )?),
                        GridDependency::Table(GridTableDependency::new(
                            "Table1",
                            rect(5, 2, 5, 3)?,
                            bounds(),
                        )?),
                    ],
                ),
            ],
            dirty_closure_checks: vec![
                dirty_check(
                    "edit-c3-dirties-data-qualified-table-consumers",
                    [address(3, 3)],
                    [address(3, 3), address(1, 4), address(2, 4), address(4, 4)],
                ),
                dirty_check(
                    "edit-b5-dirties-totals-and-all-table-consumers",
                    [address(5, 2)],
                    [
                        address(5, 2),
                        address(3, 4),
                        address(4, 4),
                        address(7, 4),
                        address(8, 4),
                    ],
                ),
                dirty_check(
                    "edit-b1-dirties-header-total-union-table-consumers",
                    [address(1, 2)],
                    [address(1, 2), address(4, 4), address(7, 4), address(8, 4)],
                ),
                dirty_check(
                    "edit-b8-dirties-escaped-table-consumers",
                    [address(8, 2)],
                    [address(8, 2), address(5, 4), address(6, 4)],
                ),
            ],
            dynamic_closure_checks: Vec::new(),
            spill_closure_checks: Vec::new(),
            spill_blocker_closure_checks: Vec::new(),
            axis_visibility_closure_checks: Vec::new(),
            namespace_lifecycle_checks: Vec::new(),
            structural_edit_checks: Vec::new(),
        }),
    })
}

fn repeated_r1c1_formula_region() -> Result<GridCorpusScenario, GridRefError> {
    let mut sheet = sheet();
    sheet.put_dense_literal_region(
        rect(1, 1, 4, 1)?,
        vec![
            CalcValue::number(10.0),
            CalcValue::number(20.0),
            CalcValue::number(30.0),
            CalcValue::number(40.0),
        ],
    )?;
    sheet.put_repeated_formula_region(
        rect(1, 2, 4, 2)?,
        GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
            .with_source_channel(oxfml_core::source::FormulaChannelKind::WorksheetR1C1),
    )?;
    let probes = vec![address(1, 2), address(2, 2), address(4, 2)];
    Ok(GridCorpusScenario {
        case_id: "grid_seed_repeated_r1c1_formula_region_001".to_string(),
        description: "A repeated R1C1 formula region evaluates against dense left-neighbor values."
            .to_string(),
        tags: strings(["dense-values", "repeated-formula", "r1c1"]),
        sheet,
        probes: probes.clone(),
        expected_values: vec![
            expected(address(1, 2), CalcValue::number(20.0)),
            expected(address(2, 2), CalcValue::number(40.0)),
            expected(address(4, 2), CalcValue::number(80.0)),
        ],
        invalidation: Some(GridInvalidationScenario {
            dependencies: vec![
                cell_dependencies(address(1, 2), [GridDependency::Cell(address(1, 1))]),
                cell_dependencies(address(2, 2), [GridDependency::Cell(address(2, 1))]),
                cell_dependencies(address(3, 2), [GridDependency::Cell(address(3, 1))]),
                cell_dependencies(address(4, 2), [GridDependency::Cell(address(4, 1))]),
            ],
            dirty_closure_checks: vec![
                dirty_check(
                    "edit-a2-dirties-r1c1-row-peer",
                    [address(2, 1)],
                    [address(2, 1), address(2, 2)],
                ),
                dirty_check(
                    "edit-a4-dirties-r1c1-row-peer",
                    [address(4, 1)],
                    [address(4, 1), address(4, 2)],
                ),
            ],
            dynamic_closure_checks: Vec::new(),
            spill_closure_checks: Vec::new(),
            spill_blocker_closure_checks: Vec::new(),
            axis_visibility_closure_checks: Vec::new(),
            namespace_lifecycle_checks: Vec::new(),
            structural_edit_checks: Vec::new(),
        }),
    })
}

fn optimized_structural_edit_repeated_r1c1_region() -> Result<GridCorpusScenario, GridRefError> {
    let mut sheet = sheet();
    sheet.put_dense_literal_region(
        rect(1, 1, 4, 1)?,
        vec![
            CalcValue::number(10.0),
            CalcValue::number(20.0),
            CalcValue::number(30.0),
            CalcValue::number(40.0),
        ],
    )?;
    sheet.put_repeated_formula_region(
        rect(1, 2, 4, 2)?,
        GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
            .with_source_channel(oxfml_core::source::FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.apply_axis_edit(GridAxisEdit::insert_rows(3, 1))?;
    let probes = vec![
        address(1, 2),
        address(2, 2),
        address(3, 2),
        address(4, 2),
        address(5, 2),
    ];
    Ok(GridCorpusScenario {
        case_id: "grid_seed_optimized_structural_edit_repeated_r1c1_001".to_string(),
        description:
            "Optimized compact dense and repeated R1C1 regions split across an inserted row and still match reference projection."
                .to_string(),
        tags: strings(["structural-edit", "dense-values", "repeated-formula", "r1c1"]),
        sheet,
        probes: probes.clone(),
        expected_values: vec![
            expected(address(1, 2), CalcValue::number(20.0)),
            expected(address(2, 2), CalcValue::number(40.0)),
            expected(address(3, 2), CalcValue::empty()),
            expected(address(4, 2), CalcValue::number(60.0)),
            expected(address(5, 2), CalcValue::number(80.0)),
        ],
        invalidation: Some(GridInvalidationScenario {
            dependencies: Vec::new(),
            dirty_closure_checks: Vec::new(),
            dynamic_closure_checks: Vec::new(),
            spill_closure_checks: Vec::new(),
            spill_blocker_closure_checks: Vec::new(),
            axis_visibility_closure_checks: Vec::new(),
            namespace_lifecycle_checks: Vec::new(),
            structural_edit_checks: Vec::new(),
        }),
    })
}

fn hidden_row_subtotal_visibility_invalidation() -> Result<GridCorpusScenario, GridRefError> {
    let mut sheet = sheet();
    sheet.put_dense_literal_region(
        rect(1, 1, 3, 1)?,
        vec![
            CalcValue::number(10.0),
            CalcValue::number(20.0),
            CalcValue::number(30.0),
        ],
    )?;
    sheet.axis_state_mut().set_row(
        2,
        crate::grid::machine::GridAxisProps {
            hidden_manual: true,
            ..crate::grid::machine::GridAxisProps::visible()
        },
    );
    sheet.axis_state_mut().set_row(
        3,
        crate::grid::machine::GridAxisProps {
            hidden_filter: true,
            ..crate::grid::machine::GridAxisProps::visible()
        },
    );
    sheet.set_formula(
        address(1, 2),
        GridFormulaCell::new(
            "=SUBTOTAL(109,A1:A3)",
            "excel.grid.v1:subtotal109:R[0]C[-1]:R[2]C[-1]",
        ),
    )?;
    sheet.set_formula(
        address(1, 3),
        GridFormulaCell::new("=B1+1", "excel.grid.v1:cell:R[0]C[-1]+1"),
    )?;
    let probes = vec![address(1, 2), address(1, 3)];
    Ok(GridCorpusScenario {
        case_id: "grid_seed_hidden_row_subtotal_visibility_001".to_string(),
        description:
            "Hidden-sensitive SUBTOTAL evaluates through AxisState and visibility changes dirty the aggregate chain."
                .to_string(),
        tags: strings(["hidden-row", "axis-visibility", "subtotal", "invalidation"]),
        sheet,
        probes: probes.clone(),
        expected_values: vec![
            expected(address(1, 2), CalcValue::number(10.0)),
            expected(address(1, 3), CalcValue::number(11.0)),
        ],
        invalidation: Some(GridInvalidationScenario {
            dependencies: vec![
                cell_dependencies(
                    address(1, 2),
                    [
                        GridDependency::Range(rect(1, 1, 3, 1)?),
                        GridDependency::AxisVisibility(GridAxisVisibilityDependency::rows(1, 3)),
                    ],
                ),
                cell_dependencies(address(1, 3), [GridDependency::Cell(address(1, 2))]),
            ],
            dirty_closure_checks: vec![dirty_check(
                "edit-a1-dirties-hidden-sensitive-chain",
                [address(1, 1)],
                [address(1, 1), address(1, 2), address(1, 3)],
            )],
            dynamic_closure_checks: Vec::new(),
            spill_closure_checks: Vec::new(),
            spill_blocker_closure_checks: Vec::new(),
            axis_visibility_closure_checks: vec![axis_visibility_check(
                "hide-row2-dirties-hidden-sensitive-chain",
                GridAxisVisibilityDependency::rows(2, 2),
                [address(1, 2), address(1, 3)],
            )],
            namespace_lifecycle_checks: Vec::new(),
            structural_edit_checks: Vec::new(),
        }),
    })
}

fn whole_axis_value_invalidation() -> Result<GridCorpusScenario, GridRefError> {
    let mut sheet = sheet();
    sheet.set_literal(address(1, 1), CalcValue::number(5.0))?;
    sheet.set_literal(address(1, 3), CalcValue::number(7.0))?;
    sheet.set_literal(address(3, 2), CalcValue::number(11.0))?;
    sheet.set_formula(
        address(2, 4),
        GridFormulaCell::new("=SUM(1:1)", "excel.grid.v1:sum-whole-row:R1"),
    )?;
    sheet.set_formula(
        address(2, 5),
        GridFormulaCell::new("=SUM(A:B)", "excel.grid.v1:sum-whole-column:C1:C2"),
    )?;
    sheet.set_formula(
        address(2, 6),
        GridFormulaCell::new("=D2+E2", "excel.grid.v1:cell:R[0]C[-2]+R[0]C[-1]"),
    )?;
    let probes = vec![address(2, 4), address(2, 5), address(2, 6)];
    Ok(GridCorpusScenario {
        case_id: "grid_seed_whole_axis_value_invalidation_001".to_string(),
        description:
            "Whole-row and whole-column references evaluate sparsely and dirty through axis-value indexes."
                .to_string(),
        tags: strings([
            "whole-axis",
            "whole-row",
            "whole-column",
            "sparse-formula",
            "invalidation",
        ]),
        sheet,
        probes: probes.clone(),
        expected_values: vec![
            expected(address(2, 4), CalcValue::number(12.0)),
            expected(address(2, 5), CalcValue::number(16.0)),
            expected(address(2, 6), CalcValue::number(28.0)),
        ],
        invalidation: Some(GridInvalidationScenario {
            dependencies: vec![
                cell_dependencies(
                    address(2, 4),
                    [GridDependency::AxisValue(GridAxisValueDependency::rows(1, 1))],
                ),
                cell_dependencies(
                    address(2, 5),
                    [GridDependency::AxisValue(GridAxisValueDependency::columns(1, 2))],
                ),
                cell_dependencies(
                    address(2, 6),
                    [
                        GridDependency::Cell(address(2, 4)),
                        GridDependency::Cell(address(2, 5)),
                    ],
                ),
            ],
            dirty_closure_checks: vec![
                dirty_check(
                    "edit-c1-dirties-whole-row-chain",
                    [address(1, 3)],
                    [address(1, 3), address(2, 4), address(2, 6)],
                ),
                dirty_check(
                    "edit-b3-dirties-whole-column-chain",
                    [address(3, 2)],
                    [address(3, 2), address(2, 5), address(2, 6)],
                ),
            ],
            dynamic_closure_checks: Vec::new(),
            spill_closure_checks: Vec::new(),
            spill_blocker_closure_checks: Vec::new(),
            axis_visibility_closure_checks: Vec::new(),
            namespace_lifecycle_checks: Vec::new(),
            structural_edit_checks: vec![GridStructuralInvalidationCheck {
                check_id: "insert-row-before-whole-row-reference-shifts-axis-index".to_string(),
                edit: GridAxisEdit::insert_rows(1, 1),
                dirty_closure_checks: vec![dirty_check(
                    "post-insert-edit-c2-dirties-shifted-whole-row-chain",
                    [address(2, 3)],
                    [address(2, 3), address(3, 4), address(3, 6)],
                )],
                dynamic_closure_checks: Vec::new(),
                spill_closure_checks: Vec::new(),
                spill_blocker_closure_checks: Vec::new(),
                axis_visibility_closure_checks: Vec::new(),
            }],
        }),
    })
}

fn spill_anchor_ledger_invalidation() -> Result<GridCorpusScenario, GridRefError> {
    let mut sheet = sheet();
    sheet.put_dense_literal_region(
        rect(1, 1, 3, 1)?,
        vec![
            CalcValue::number(4.0),
            CalcValue::number(8.0),
            CalcValue::number(16.0),
        ],
    )?;
    sheet.set_spill_fact(GridSpillFact {
        anchor: address(1, 1),
        extent: rect(1, 1, 3, 1)?,
        blocked: false,
    })?;
    sheet.set_formula(
        address(1, 2),
        GridFormulaCell::new("=SUM(A1#)", "excel.grid.v1:sum-spill:R[0]C[-1]#"),
    )?;
    sheet.set_formula(
        address(1, 3),
        GridFormulaCell::new("=B1+1", "excel.grid.v1:cell:R[0]C[-1]+1"),
    )?;
    let probes = vec![address(1, 2), address(1, 3)];
    Ok(GridCorpusScenario {
        case_id: "grid_seed_spill_anchor_ledger_001".to_string(),
        description:
            "A committed spill extent feeds A1# through both engines, and spill-shape dirties consumers separately from member-cell edits."
                .to_string(),
        tags: strings(["spill", "a1-hash", "sparse-formula", "invalidation"]),
        sheet,
        probes: probes.clone(),
        expected_values: vec![
            expected(address(1, 2), CalcValue::number(28.0)),
            expected(address(1, 3), CalcValue::number(29.0)),
        ],
        invalidation: Some(GridInvalidationScenario {
            dependencies: vec![
                cell_dependencies(
                    address(1, 2),
                    [
                        GridDependency::Range(rect(1, 1, 3, 1)?),
                        GridDependency::SpillFact(GridSpillDependency::anchor(address(1, 1))),
                    ],
                ),
                cell_dependencies(address(1, 3), [GridDependency::Cell(address(1, 2))]),
            ],
            dirty_closure_checks: vec![dirty_check(
                "edit-a2-dirties-spill-consumer-chain",
                [address(2, 1)],
                [address(2, 1), address(1, 2), address(1, 3)],
            )],
            dynamic_closure_checks: Vec::new(),
            spill_closure_checks: vec![spill_check(
                "spill-a1-shape-dirties-consumer-chain",
                GridSpillDependency::anchor(address(1, 1)),
                [address(1, 2), address(1, 3)],
            )],
            spill_blocker_closure_checks: Vec::new(),
            axis_visibility_closure_checks: Vec::new(),
            namespace_lifecycle_checks: Vec::new(),
            structural_edit_checks: vec![GridStructuralInvalidationCheck {
                check_id: "insert-row-inside-spill-extent-keeps-anchor-shape-dependency"
                    .to_string(),
                edit: GridAxisEdit::insert_rows(2, 1),
                dirty_closure_checks: Vec::new(),
                dynamic_closure_checks: Vec::new(),
                spill_closure_checks: vec![spill_check(
                    "post-insert-spill-a1-shape-dirties-consumer-chain",
                    GridSpillDependency::anchor(address(1, 1)),
                    [address(1, 2), address(1, 3)],
                )],
                spill_blocker_closure_checks: Vec::new(),
                axis_visibility_closure_checks: Vec::new(),
            }],
        }),
    })
}

fn dynamic_sequence_spill_publication() -> Result<GridCorpusScenario, GridRefError> {
    let mut sheet = sheet();
    sheet.set_formula(
        address(1, 1),
        GridFormulaCell::new("=SEQUENCE(3)", "excel.grid.v1:sequence:R1C1#"),
    )?;
    sheet.set_formula(
        address(1, 2),
        GridFormulaCell::new("=SUM(A1#)", "excel.grid.v1:sum-spill:R[0]C[-1]#"),
    )?;
    sheet.set_formula(
        address(1, 3),
        GridFormulaCell::new("=B1+1", "excel.grid.v1:cell:R[0]C[-1]+1"),
    )?;
    let probes = vec![
        address(1, 1),
        address(2, 1),
        address(3, 1),
        address(1, 2),
        address(1, 3),
    ];
    Ok(GridCorpusScenario {
        case_id: "grid_seed_dynamic_sequence_spill_001".to_string(),
        description:
            "A dynamic-array SEQUENCE formula publishes ghost cells and feeds an A1# consumer chain in reference and optimized engines."
                .to_string(),
        tags: strings(["spill", "dynamic-array", "sequence", "a1-hash", "invalidation"]),
        sheet,
        probes: probes.clone(),
        expected_values: vec![
            expected(address(1, 1), CalcValue::number(1.0)),
            expected(address(2, 1), CalcValue::number(2.0)),
            expected(address(3, 1), CalcValue::number(3.0)),
            expected(address(1, 2), CalcValue::number(6.0)),
            expected(address(1, 3), CalcValue::number(7.0)),
        ],
        invalidation: Some(GridInvalidationScenario {
            dependencies: vec![
                cell_dependencies(
                    address(1, 2),
                    [
                        GridDependency::Range(rect(1, 1, 3, 1)?),
                        GridDependency::SpillFact(GridSpillDependency::anchor(address(1, 1))),
                    ],
                ),
                cell_dependencies(address(1, 3), [GridDependency::Cell(address(1, 2))]),
            ],
            dirty_closure_checks: vec![dirty_check(
                "edit-spill-ghost-a2-dirties-sequence-consumer-chain",
                [address(2, 1)],
                [address(2, 1), address(1, 2), address(1, 3)],
            )],
            dynamic_closure_checks: Vec::new(),
            spill_closure_checks: vec![spill_check(
                "sequence-a1-shape-dirties-consumer-chain",
                GridSpillDependency::anchor(address(1, 1)),
                [address(1, 2), address(1, 3)],
            )],
            spill_blocker_closure_checks: Vec::new(),
            axis_visibility_closure_checks: Vec::new(),
            namespace_lifecycle_checks: Vec::new(),
            structural_edit_checks: Vec::new(),
        }),
    })
}

fn dynamic_sequence_spill_repair() -> Result<GridCorpusScenario, GridRefError> {
    let mut sheet = sheet();
    sheet.set_formula(
        address(1, 1),
        GridFormulaCell::new("=SUM(B1#)", "excel.grid.v1:sum-late-spill:RC[1]#"),
    )?;
    sheet.set_formula(
        address(1, 2),
        GridFormulaCell::new("=SEQUENCE(3)", "excel.grid.v1:sequence:R1C2#"),
    )?;
    let probes = vec![address(1, 1), address(1, 2), address(2, 2), address(3, 2)];
    Ok(GridCorpusScenario {
        case_id: "grid_seed_dynamic_sequence_spill_repair_001".to_string(),
        description:
            "A B1 dynamic-array spill publishes after its earlier A1 B1# consumer and converges through the bounded spill repair phase."
                .to_string(),
        tags: strings([
            "spill",
            "dynamic-array",
            "sequence",
            "a1-hash",
            "repair",
            "invalidation",
        ]),
        sheet,
        probes: probes.clone(),
        expected_values: vec![
            expected(address(1, 1), CalcValue::number(6.0)),
            expected(address(1, 2), CalcValue::number(1.0)),
            expected(address(2, 2), CalcValue::number(2.0)),
            expected(address(3, 2), CalcValue::number(3.0)),
        ],
        invalidation: Some(GridInvalidationScenario {
            dependencies: vec![cell_dependencies(
                address(1, 1),
                [
                    GridDependency::Range(rect(1, 2, 3, 2)?),
                    GridDependency::SpillFact(GridSpillDependency::anchor(address(1, 2))),
                ],
            )],
            dirty_closure_checks: vec![dirty_check(
                "edit-late-spill-ghost-b2-dirties-repaired-consumer",
                [address(2, 2)],
                [address(2, 2), address(1, 1)],
            )],
            dynamic_closure_checks: Vec::new(),
            spill_closure_checks: vec![spill_check(
                "late-sequence-b1-shape-dirties-repaired-consumer",
                GridSpillDependency::anchor(address(1, 2)),
                [address(1, 1)],
            )],
            spill_blocker_closure_checks: Vec::new(),
            axis_visibility_closure_checks: Vec::new(),
            namespace_lifecycle_checks: Vec::new(),
            structural_edit_checks: Vec::new(),
        }),
    })
}

fn mutual_sequence_spill_blockage() -> Result<GridCorpusScenario, GridRefError> {
    let mut sheet = sheet();
    sheet.set_formula(
        address(1, 1),
        GridFormulaCell::new("=SEQUENCE(3)", "excel.grid.v1:sequence:R1C1#"),
    )?;
    sheet.set_formula(
        address(2, 1),
        GridFormulaCell::new("=SEQUENCE(3)", "excel.grid.v1:sequence:R2C1#"),
    )?;
    let probes = vec![address(1, 1), address(2, 1), address(3, 1), address(4, 1)];
    Ok(GridCorpusScenario {
        case_id: "grid_seed_mutual_sequence_spill_blockage_001".to_string(),
        description:
            "Neighboring dynamic-array SEQUENCE anchors mutually block when a later anchor lies inside an earlier blocked spill extent."
                .to_string(),
        tags: strings(["spill", "dynamic-array", "sequence", "blockage", "mutual"]),
        sheet,
        probes: probes.clone(),
        expected_values: vec![
            expected(address(1, 1), CalcValue::error(WorksheetErrorCode::Spill)),
            expected(address(2, 1), CalcValue::error(WorksheetErrorCode::Spill)),
            expected(address(3, 1), CalcValue::empty()),
            expected(address(4, 1), CalcValue::empty()),
        ],
        invalidation: Some(GridInvalidationScenario {
            dependencies: Vec::new(),
            dirty_closure_checks: Vec::new(),
            dynamic_closure_checks: Vec::new(),
            spill_closure_checks: Vec::new(),
            spill_blocker_closure_checks: Vec::new(),
            axis_visibility_closure_checks: Vec::new(),
            namespace_lifecycle_checks: Vec::new(),
            structural_edit_checks: Vec::new(),
        }),
    })
}

fn table_overlay_spill_blockage() -> Result<GridCorpusScenario, GridRefError> {
    let mut sheet = sheet();
    sheet.set_formula(
        address(1, 1),
        GridFormulaCell::new("=SEQUENCE(3)", "excel.grid.v1:sequence:R1C1#"),
    )?;
    sheet.add_feature_rendered_region(rect(2, 1, 3, 1)?, "table-overlay", false)?;
    let probes = vec![address(1, 1), address(2, 1), address(3, 1)];
    Ok(GridCorpusScenario {
        case_id: "grid_seed_table_overlay_spill_blockage_001".to_string(),
        description:
            "A table-overlay feature region blocks a dynamic-array spill extent in reference and optimized engines."
                .to_string(),
        tags: strings(["spill", "dynamic-array", "table-overlay", "blockage"]),
        sheet,
        probes: probes.clone(),
        expected_values: vec![
            expected(address(1, 1), CalcValue::error(WorksheetErrorCode::Spill)),
            expected(address(2, 1), CalcValue::empty()),
            expected(address(3, 1), CalcValue::empty()),
        ],
        invalidation: Some(GridInvalidationScenario {
            dependencies: Vec::new(),
            dirty_closure_checks: Vec::new(),
            dynamic_closure_checks: Vec::new(),
            spill_closure_checks: Vec::new(),
            spill_blocker_closure_checks: Vec::new(),
            axis_visibility_closure_checks: Vec::new(),
            namespace_lifecycle_checks: Vec::new(),
            structural_edit_checks: Vec::new(),
        }),
    })
}

fn merged_region_spill_blockage() -> Result<GridCorpusScenario, GridRefError> {
    let mut sheet = sheet();
    sheet.set_formula(
        address(1, 1),
        GridFormulaCell::new("=SEQUENCE(3)", "excel.grid.v1:sequence:R1C1#"),
    )?;
    sheet.add_merged_region(rect(2, 1, 3, 1)?)?;
    let probes = vec![address(1, 1), address(2, 1), address(3, 1)];
    Ok(GridCorpusScenario {
        case_id: "grid_seed_merged_region_spill_blockage_001".to_string(),
        description:
            "A merged region blocks a dynamic-array spill extent in reference and optimized engines."
                .to_string(),
        tags: strings([
            "spill",
            "dynamic-array",
            "merged-region",
            "blockage",
            "blocker-invalidation",
        ]),
        sheet,
        probes: probes.clone(),
        expected_values: vec![
            expected(address(1, 1), CalcValue::error(WorksheetErrorCode::Spill)),
            expected(address(2, 1), CalcValue::empty()),
            expected(address(3, 1), CalcValue::empty()),
        ],
        invalidation: Some(GridInvalidationScenario {
            dependencies: vec![cell_dependencies(
                address(1, 1),
                [GridDependency::SpillBlocker(
                    GridSpillBlockerDependency::extent(rect(1, 1, 3, 1)?),
                )],
            )],
            dirty_closure_checks: Vec::new(),
            dynamic_closure_checks: Vec::new(),
            spill_closure_checks: Vec::new(),
            spill_blocker_closure_checks: vec![spill_blocker_check(
                "merged-region-clearance-dirties-blocked-spill-anchor",
                GridSpillBlockerDependency::extent(rect(2, 1, 2, 1)?),
                [address(1, 1)],
            )],
            axis_visibility_closure_checks: Vec::new(),
            namespace_lifecycle_checks: Vec::new(),
            structural_edit_checks: vec![GridStructuralInvalidationCheck {
                check_id: "insert-row-inside-blocked-spill-extent-keeps-blocker-watch"
                    .to_string(),
                edit: GridAxisEdit::insert_rows(2, 1),
                dirty_closure_checks: Vec::new(),
                dynamic_closure_checks: Vec::new(),
                spill_closure_checks: Vec::new(),
                spill_blocker_closure_checks: vec![spill_blocker_check(
                    "post-insert-blocker-clearance-dirties-blocked-spill-anchor",
                    GridSpillBlockerDependency::extent(rect(4, 1, 4, 1)?),
                    [address(1, 1)],
                )],
                axis_visibility_closure_checks: Vec::new(),
            }],
        }),
    })
}

fn dynamic_invalidation_request() -> Result<GridCorpusScenario, GridRefError> {
    let mut sheet = sheet();
    sheet.set_literal(address(1, 1), CalcValue::number(11.0))?;
    sheet.set_literal(address(1, 2), CalcValue::number(22.0))?;
    sheet.set_formula(
        address(1, 3),
        GridFormulaCell::new("=B1+1", "excel.grid.v1:cell:R[0]C[-1]+1"),
    )?;
    let probes = vec![address(1, 1), address(1, 2), address(1, 3)];
    Ok(GridCorpusScenario {
        case_id: "grid_seed_dynamic_invalidation_request_001".to_string(),
        description: "GridInvalidation-Ref keeps host-sensitive dynamic requests separate from scalar cell edges."
            .to_string(),
        tags: strings(["dynamic-invalidation", "scalar-closure"]),
        sheet,
        probes: probes.clone(),
        expected_values: vec![
            expected(address(1, 1), CalcValue::number(11.0)),
            expected(address(1, 2), CalcValue::number(22.0)),
            expected(address(1, 3), CalcValue::number(23.0)),
        ],
        invalidation: Some(GridInvalidationScenario {
            dependencies: vec![
                cell_dependencies(
                    address(1, 2),
                    [GridDependency::DynamicRequest("indirect:Sheet1!A1".to_string())],
                ),
                cell_dependencies(address(1, 3), [GridDependency::Cell(address(1, 2))]),
            ],
            dirty_closure_checks: vec![dirty_check(
                "edit-b1-dirties-downstream",
                [address(1, 2)],
                [address(1, 2), address(1, 3)],
            )],
            dynamic_closure_checks: vec![dynamic_check(
                "dynamic-request-dirties-dependent-chain",
                "indirect:Sheet1!A1",
                [address(1, 2), address(1, 3)],
            )],
            spill_closure_checks: Vec::new(),
            spill_blocker_closure_checks: Vec::new(),
            axis_visibility_closure_checks: Vec::new(),
            namespace_lifecycle_checks: Vec::new(),
            structural_edit_checks: Vec::new(),
        }),
    })
}

fn compare_expected_values(
    expected_values: &[GridExpectedCellValue],
    engine_report: &GridDifferentialRunReport,
) -> Vec<GridExpectedValueMismatch> {
    expected_values
        .iter()
        .filter_map(|expected| {
            let reference = engine_report
                .reference
                .as_ref()
                .and_then(|report| readout_value(&report.readout, &expected.address));
            let optimized = engine_report
                .optimized
                .as_ref()
                .and_then(|report| readout_value(&report.readout, &expected.address));
            let reference_matches = reference
                .as_ref()
                .is_none_or(|value| value == &expected.expected);
            let optimized_matches = optimized
                .as_ref()
                .is_none_or(|value| value == &expected.expected);
            (!reference_matches || !optimized_matches).then(|| GridExpectedValueMismatch {
                address: expected.address.clone(),
                expected: expected.expected.clone(),
                reference,
                optimized,
            })
        })
        .collect()
}

fn readout_value(
    readout: &[GridEngineCellReadout],
    address: &ExcelGridCellAddress,
) -> Option<CalcValue> {
    readout
        .iter()
        .find(|entry| &entry.address == address)
        .map(|entry| entry.computed.clone())
}

fn expected(address: ExcelGridCellAddress, expected: CalcValue) -> GridExpectedCellValue {
    GridExpectedCellValue { address, expected }
}

fn cell_dependencies(
    dependent: ExcelGridCellAddress,
    dependencies: impl IntoIterator<Item = GridDependency>,
) -> GridInvalidationCellDependencies {
    GridInvalidationCellDependencies {
        dependent,
        dependencies: dependencies.into_iter().collect(),
    }
}

fn dirty_check(
    check_id: impl Into<String>,
    seeds: impl IntoIterator<Item = ExcelGridCellAddress>,
    expected_dirty: impl IntoIterator<Item = ExcelGridCellAddress>,
) -> GridDirtyClosureCheck {
    GridDirtyClosureCheck {
        check_id: check_id.into(),
        seeds: seeds.into_iter().collect(),
        expected_dirty: expected_dirty.into_iter().collect(),
    }
}

fn dynamic_check(
    check_id: impl Into<String>,
    request_key: impl Into<String>,
    expected_dirty: impl IntoIterator<Item = ExcelGridCellAddress>,
) -> GridDynamicClosureCheck {
    GridDynamicClosureCheck {
        check_id: check_id.into(),
        request_key: request_key.into(),
        expected_dirty: expected_dirty.into_iter().collect(),
    }
}

fn spill_check(
    check_id: impl Into<String>,
    dependency: GridSpillDependency,
    expected_dirty: impl IntoIterator<Item = ExcelGridCellAddress>,
) -> GridSpillClosureCheck {
    GridSpillClosureCheck {
        check_id: check_id.into(),
        dependency,
        expected_dirty: expected_dirty.into_iter().collect(),
    }
}

fn spill_blocker_check(
    check_id: impl Into<String>,
    dependency: GridSpillBlockerDependency,
    expected_dirty: impl IntoIterator<Item = ExcelGridCellAddress>,
) -> GridSpillBlockerClosureCheck {
    GridSpillBlockerClosureCheck {
        check_id: check_id.into(),
        dependency,
        expected_dirty: expected_dirty.into_iter().collect(),
    }
}

fn axis_visibility_check(
    check_id: impl Into<String>,
    dependency: GridAxisVisibilityDependency,
    expected_dirty: impl IntoIterator<Item = ExcelGridCellAddress>,
) -> GridAxisVisibilityClosureCheck {
    GridAxisVisibilityClosureCheck {
        check_id: check_id.into(),
        dependency,
        expected_dirty: expected_dirty.into_iter().collect(),
    }
}

fn namespace_lifecycle_check(
    check_id: impl Into<String>,
    operation: GridNamespaceLifecycleOperation,
    expected_dirty: impl IntoIterator<Item = ExcelGridCellAddress>,
    dirty_closure_checks: impl IntoIterator<Item = GridDirtyClosureCheck>,
) -> GridNamespaceLifecycleCheck {
    GridNamespaceLifecycleCheck {
        check_id: check_id.into(),
        operation,
        expected_dirty: expected_dirty.into_iter().collect(),
        dirty_closure_checks: dirty_closure_checks.into_iter().collect(),
    }
}

fn sheet() -> GridOptimizedSheet {
    GridOptimizedSheet::new(GRID_WORKBOOK_ID, GRID_SHEET_ID, bounds())
}

fn rect(
    top_row: u32,
    left_col: u32,
    bottom_row: u32,
    right_col: u32,
) -> Result<GridRect, GridRefError> {
    GridRect::new(
        GRID_WORKBOOK_ID,
        GRID_SHEET_ID,
        top_row,
        left_col,
        bottom_row,
        right_col,
        bounds(),
    )
}

fn address(row: u32, col: u32) -> ExcelGridCellAddress {
    ExcelGridCellAddress::new(GRID_WORKBOOK_ID, GRID_SHEET_ID, row, col)
}

const fn bounds() -> ExcelGridBounds {
    ExcelGridBounds {
        max_rows: 128,
        max_cols: 32,
    }
}

fn strings(values: impl IntoIterator<Item = &'static str>) -> Vec<String> {
    values.into_iter().map(str::to_string).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::machine::GridEngineRecalcReport;

    #[test]
    fn grid_engine_mode_parses_runner_arg_surface() {
        assert_eq!(
            GridEngineMode::from_engine_arg("reference").unwrap(),
            GridEngineMode::Reference
        );
        assert_eq!(
            GridEngineMode::from_engine_arg("optimized").unwrap(),
            GridEngineMode::Optimized
        );
        assert_eq!(
            GridEngineMode::from_engine_arg("both").unwrap(),
            GridEngineMode::Both
        );
        assert_eq!(GridEngineMode::Both.engine_arg(), "both");
        assert!(matches!(
            GridEngineMode::from_engine_arg("fast"),
            Err(GridRunnerError::UnknownEngineMode { value }) if value == "fast"
        ));
    }

    #[test]
    fn grid_runner_seed_corpus_both_mode_matches_reference_and_optimized() {
        let report = GridCorpusRunner::new()
            .run_seed_corpus(GridEngineMode::Both)
            .unwrap();

        assert_eq!(report.schema_version, GRID_RUN_REPORT_SCHEMA_V1);
        assert_eq!(report.engine_mode, GridEngineMode::Both);
        assert_eq!(report.case_count, 18);
        assert_eq!(report.expectation_mismatch_count, 0);
        assert_eq!(report.differential_mismatch_count, 0);
        assert_eq!(report.invalidation_mismatch_count, 0);
        assert!(report.matched());
        assert!(
            report
                .cases
                .iter()
                .any(|case| case.tags.iter().any(|tag| tag == "sparse"))
        );
        assert!(
            report
                .cases
                .iter()
                .any(|case| case.tags.iter().any(|tag| tag == "dense-values"))
        );
        assert!(
            report
                .cases
                .iter()
                .any(|case| case.tags.iter().any(|tag| tag == "repeated-formula"))
        );
        assert!(report.cases.iter().any(|case| {
            case.case_id == "grid_seed_defined_name_value_resolution_001"
                && case.tags.iter().any(|tag| tag == "defined-name")
        }));
        assert!(report.cases.iter().any(|case| {
            case.case_id == "grid_seed_table_overlay_structured_reference_001"
                && case.tags.iter().any(|tag| tag == "structured-reference")
        }));
        assert!(report.cases.iter().any(|case| {
            case.case_id == "grid_seed_table_overlay_current_row_structured_reference_001"
                && case.tags.iter().any(|tag| tag == "current-row")
                && case.tags.iter().any(|tag| tag == "caller-context")
                && case.tags.iter().any(|tag| tag == "omitted-table")
        }));
        assert!(report.cases.iter().any(|case| {
            case.case_id == "grid_seed_table_overlay_structured_sections_001"
                && case.tags.iter().any(|tag| tag == "section-qualified")
                && case.tags.iter().any(|tag| tag == "escaped-column")
        }));
        assert!(
            report
                .cases
                .iter()
                .any(|case| case.tags.iter().any(|tag| tag == "dynamic-invalidation"))
        );
        assert!(report.cases.iter().any(|case| {
            case.case_id == "grid_seed_optimized_structural_edit_repeated_r1c1_001"
                && case.tags.iter().any(|tag| tag == "structural-edit")
        }));
        assert!(report.cases.iter().any(|case| {
            case.case_id == "grid_seed_hidden_row_subtotal_visibility_001"
                && case.tags.iter().any(|tag| tag == "hidden-row")
        }));
        assert!(report.cases.iter().any(|case| {
            case.case_id == "grid_seed_whole_axis_value_invalidation_001"
                && case.tags.iter().any(|tag| tag == "whole-axis")
        }));
        assert!(report.cases.iter().any(|case| {
            case.case_id == "grid_seed_spill_anchor_ledger_001"
                && case.tags.iter().any(|tag| tag == "spill")
        }));
        assert!(report.cases.iter().any(|case| {
            case.case_id == "grid_seed_dynamic_sequence_spill_001"
                && case.tags.iter().any(|tag| tag == "dynamic-array")
        }));
        assert!(report.cases.iter().any(|case| {
            case.case_id == "grid_seed_dynamic_sequence_spill_repair_001"
                && case.tags.iter().any(|tag| tag == "repair")
        }));
        assert!(report.cases.iter().any(|case| {
            case.case_id == "grid_seed_mutual_sequence_spill_blockage_001"
                && case.tags.iter().any(|tag| tag == "mutual")
        }));
        assert!(report.cases.iter().any(|case| {
            case.case_id == "grid_seed_table_overlay_spill_blockage_001"
                && case.tags.iter().any(|tag| tag == "table-overlay")
        }));
        assert!(report.cases.iter().any(|case| {
            case.case_id == "grid_seed_merged_region_spill_blockage_001"
                && case.tags.iter().any(|tag| tag == "merged-region")
                && case.tags.iter().any(|tag| tag == "blocker-invalidation")
        }));
        assert!(report.cases.iter().all(|case| {
            case.engine_report.reference.is_some()
                && case.engine_report.optimized.is_some()
                && case
                    .invalidation_report
                    .as_ref()
                    .is_some_and(|report| report.matched())
                && case.matched()
        }));
        let dynamic_case = report
            .cases
            .iter()
            .find(|case| case.case_id == "grid_seed_dynamic_invalidation_request_001")
            .unwrap();
        let invalidation = dynamic_case.invalidation_report.as_ref().unwrap();
        assert_eq!(invalidation.dynamic_closures.len(), 1);
        assert_eq!(
            invalidation.installed_dependencies[0].dynamic_dependencies,
            vec!["indirect:Sheet1!A1".to_string()]
        );
        let sparse_case = report
            .cases
            .iter()
            .find(|case| case.case_id == "grid_seed_sparse_formula_chain_001")
            .unwrap();
        let structural_edits = &sparse_case
            .invalidation_report
            .as_ref()
            .unwrap()
            .structural_edits;
        assert_eq!(structural_edits.len(), 1);
        assert_eq!(
            structural_edits[0].check_id,
            "insert-column-before-chain-shifts-closure"
        );
        assert_eq!(structural_edits[0].mismatch_count, 0);
        let defined_name_case = report
            .cases
            .iter()
            .find(|case| case.case_id == "grid_seed_defined_name_value_resolution_001")
            .unwrap();
        let optimized = defined_name_case.engine_report.optimized.as_ref().unwrap();
        assert_eq!(optimized.readout[0].computed, CalcValue::number(12.0));
        assert_eq!(optimized.readout[1].computed, CalcValue::number(12.0));
        let defined_name_invalidation = defined_name_case.invalidation_report.as_ref().unwrap();
        assert_eq!(defined_name_invalidation.name_edge_count, 2);
        assert_eq!(defined_name_invalidation.namespace_lifecycle.len(), 2);
        assert!(
            defined_name_invalidation
                .namespace_lifecycle
                .iter()
                .all(|report| report.matched && report.mismatch_count == 0)
        );
        assert_eq!(
            defined_name_invalidation.namespace_lifecycle[0].actual_dirty,
            sorted_addresses([address(1, 2), address(1, 3)])
        );
        assert_eq!(
            defined_name_invalidation.namespace_lifecycle[0]
                .lifecycle_report
                .name_edges_before,
            2
        );
        assert_eq!(
            defined_name_invalidation.namespace_lifecycle[0]
                .lifecycle_report
                .name_edges_after,
            2
        );
        assert_eq!(
            defined_name_invalidation.namespace_lifecycle[1]
                .lifecycle_report
                .name_edges_before,
            2
        );
        assert_eq!(
            defined_name_invalidation.namespace_lifecycle[1]
                .lifecycle_report
                .name_edges_after,
            0
        );
        assert_eq!(
            defined_name_invalidation.namespace_lifecycle[1].dirty_closures[0].actual_dirty,
            sorted_addresses([address(2, 1)])
        );
        let table_case = report
            .cases
            .iter()
            .find(|case| case.case_id == "grid_seed_table_overlay_structured_reference_001")
            .unwrap();
        let optimized = table_case.engine_report.optimized.as_ref().unwrap();
        assert_eq!(optimized.readout[0].computed, CalcValue::number(12.0));
        assert_eq!(optimized.readout[1].computed, CalcValue::number(12.0));
        let table_invalidation = table_case.invalidation_report.as_ref().unwrap();
        assert_eq!(table_invalidation.table_edge_count, 2);
        assert_eq!(
            table_invalidation.installed_dependencies[0].table_dependencies[0].table_key,
            "TABLE1"
        );
        assert_eq!(table_invalidation.namespace_lifecycle.len(), 3);
        assert!(
            table_invalidation
                .namespace_lifecycle
                .iter()
                .all(|report| report.matched && report.mismatch_count == 0)
        );
        assert_eq!(
            table_invalidation.namespace_lifecycle[0]
                .lifecycle_report
                .table_edges_before,
            2
        );
        assert_eq!(
            table_invalidation.namespace_lifecycle[0]
                .lifecycle_report
                .table_edges_after,
            2
        );
        assert_eq!(
            table_invalidation.namespace_lifecycle[1]
                .lifecycle_report
                .table_edges_before,
            2
        );
        assert_eq!(
            table_invalidation.namespace_lifecycle[1]
                .lifecycle_report
                .table_edges_after,
            2
        );
        assert_eq!(
            table_invalidation.namespace_lifecycle[1].dirty_closures[0].actual_dirty,
            sorted_addresses([address(5, 2), address(1, 3), address(1, 4)])
        );
        assert_eq!(
            table_invalidation.namespace_lifecycle[2]
                .lifecycle_report
                .table_edges_before,
            2
        );
        assert_eq!(
            table_invalidation.namespace_lifecycle[2]
                .lifecycle_report
                .table_edges_after,
            0
        );
        let table_current_row_case = report
            .cases
            .iter()
            .find(|case| {
                case.case_id == "grid_seed_table_overlay_current_row_structured_reference_001"
            })
            .unwrap();
        assert_eq!(
            table_current_row_case
                .engine_report
                .optimized
                .as_ref()
                .unwrap()
                .readout[4]
                .computed,
            CalcValue::number(12.0)
        );
        let table_current_row_invalidation =
            table_current_row_case.invalidation_report.as_ref().unwrap();
        assert_eq!(table_current_row_invalidation.scalar_edge_count, 12);
        assert_eq!(table_current_row_invalidation.table_edge_count, 3);
        assert_eq!(
            table_current_row_invalidation.dirty_closures[1].actual_dirty,
            sorted_addresses([
                address(3, 2),
                address(3, 3),
                address(2, 4),
                address(3, 4),
                address(4, 4)
            ])
        );
        let table_sections_case = report
            .cases
            .iter()
            .find(|case| case.case_id == "grid_seed_table_overlay_structured_sections_001")
            .unwrap();
        let optimized = table_sections_case
            .engine_report
            .optimized
            .as_ref()
            .unwrap();
        assert_eq!(optimized.readout[0].computed, CalcValue::number(18.0));
        assert_eq!(optimized.readout[3].computed, CalcValue::number(36.0));
        assert_eq!(optimized.readout[5].computed, CalcValue::number(18.0));
        assert_eq!(optimized.readout[6].computed, CalcValue::number(18.0));
        assert_eq!(optimized.readout[7].computed, CalcValue::number(18.0));
        assert_eq!(
            table_sections_case
                .invalidation_report
                .as_ref()
                .unwrap()
                .table_edge_count,
            10
        );
        let hidden_case = report
            .cases
            .iter()
            .find(|case| case.case_id == "grid_seed_hidden_row_subtotal_visibility_001")
            .unwrap();
        let hidden_invalidation = hidden_case.invalidation_report.as_ref().unwrap();
        assert_eq!(hidden_invalidation.axis_visibility_closures.len(), 1);
        assert!(hidden_invalidation.axis_visibility_closures[0].matched);
        let spill_case = report
            .cases
            .iter()
            .find(|case| case.case_id == "grid_seed_spill_anchor_ledger_001")
            .unwrap();
        let spill_invalidation = spill_case.invalidation_report.as_ref().unwrap();
        assert_eq!(spill_invalidation.spill_closures.len(), 1);
        assert!(spill_invalidation.spill_closures[0].matched);
        assert_eq!(
            spill_invalidation.installed_dependencies[0].spill_dependencies,
            vec![GridSpillDependency::anchor(address(1, 1))]
        );
        let whole_axis_case = report
            .cases
            .iter()
            .find(|case| case.case_id == "grid_seed_whole_axis_value_invalidation_001")
            .unwrap();
        let whole_axis_invalidation = whole_axis_case.invalidation_report.as_ref().unwrap();
        assert_eq!(whole_axis_invalidation.axis_value_edge_count, 2);
        assert_eq!(
            whole_axis_invalidation.installed_dependencies[0].axis_value_dependencies,
            vec![GridAxisValueDependency::rows(1, 1)]
        );
        assert_eq!(
            whole_axis_invalidation.installed_dependencies[1].axis_value_dependencies,
            vec![GridAxisValueDependency::columns(1, 2)]
        );
        assert_eq!(whole_axis_invalidation.structural_edits.len(), 1);
        assert_eq!(
            whole_axis_invalidation.structural_edits[0]
                .edit_report
                .axis_value_edges_after,
            2
        );
        let p20 = whole_axis_case.p20_report.as_ref().unwrap();
        assert!(p20.matched());
        assert_eq!(p20.formula_enumerations.len(), 2);
        assert_eq!(p20.formula_enumerations[0].reference_source_text, "1:1");
        assert_eq!(p20.formula_enumerations[0].declared_cell_count, 32);
        assert_eq!(p20.formula_enumerations[0].defined_cell_count, 2);
        assert_eq!(p20.formula_enumerations[0].slots_visited(), 2);
        assert_eq!(p20.formula_enumerations[1].reference_source_text, "A:B");
        assert_eq!(p20.formula_enumerations[1].declared_cell_count, 256);
        assert_eq!(p20.formula_enumerations[1].defined_cell_count, 2);
        assert_eq!(p20.formula_enumerations[1].slots_visited(), 2);
        let sequence_spill_case = report
            .cases
            .iter()
            .find(|case| case.case_id == "grid_seed_dynamic_sequence_spill_001")
            .unwrap();
        let optimized = sequence_spill_case
            .engine_report
            .optimized
            .as_ref()
            .unwrap();
        assert!(matches!(
            &optimized.recalc,
            GridEngineRecalcReport::Optimized(recalc)
                if recalc.spill_facts_published == 1
                    && recalc.spill_ghost_cells_published == 2
        ));
        assert_eq!(
            sequence_spill_case
                .invalidation_report
                .as_ref()
                .unwrap()
                .spill_closures
                .len(),
            1
        );
        let sequence_repair_case = report
            .cases
            .iter()
            .find(|case| case.case_id == "grid_seed_dynamic_sequence_spill_repair_001")
            .unwrap();
        let optimized = sequence_repair_case
            .engine_report
            .optimized
            .as_ref()
            .unwrap();
        assert!(matches!(
            &optimized.recalc,
            GridEngineRecalcReport::Optimized(recalc)
                if recalc.spill_repair_passes == 1
                    && recalc.spill_repair_formula_evaluations == 2
                    && recalc.spill_repair_converged
                    && recalc.spill_facts_published == 1
                    && recalc.spill_ghost_cells_published == 2
        ));
        assert_eq!(
            sequence_repair_case
                .invalidation_report
                .as_ref()
                .unwrap()
                .spill_closures
                .len(),
            1
        );
        let mutual_spill_case = report
            .cases
            .iter()
            .find(|case| case.case_id == "grid_seed_mutual_sequence_spill_blockage_001")
            .unwrap();
        let optimized = mutual_spill_case.engine_report.optimized.as_ref().unwrap();
        assert_eq!(
            optimized.readout[0].computed,
            CalcValue::error(WorksheetErrorCode::Spill)
        );
        assert_eq!(
            optimized.readout[1].computed,
            CalcValue::error(WorksheetErrorCode::Spill)
        );
        assert!(matches!(
            &optimized.recalc,
            GridEngineRecalcReport::Optimized(recalc)
                if recalc.spill_facts_blocked == 2
                    && recalc.spill_facts_published == 0
                    && recalc.spill_ghost_cells_published == 0
        ));
        let table_overlay_case = report
            .cases
            .iter()
            .find(|case| case.case_id == "grid_seed_table_overlay_spill_blockage_001")
            .unwrap();
        let optimized = table_overlay_case.engine_report.optimized.as_ref().unwrap();
        assert_eq!(
            optimized.readout[0].computed,
            CalcValue::error(WorksheetErrorCode::Spill)
        );
        assert!(matches!(
            &optimized.recalc,
            GridEngineRecalcReport::Optimized(recalc)
                if recalc.spill_facts_blocked == 1
                    && recalc.spill_facts_published == 0
                    && recalc.spill_ghost_cells_published == 0
                    && recalc.spill_repair_passes == 0
        ));
        let merged_region_case = report
            .cases
            .iter()
            .find(|case| case.case_id == "grid_seed_merged_region_spill_blockage_001")
            .unwrap();
        let optimized = merged_region_case.engine_report.optimized.as_ref().unwrap();
        assert_eq!(
            optimized.readout[0].computed,
            CalcValue::error(WorksheetErrorCode::Spill)
        );
        assert!(matches!(
            &optimized.recalc,
            GridEngineRecalcReport::Optimized(recalc)
                if recalc.spill_facts_blocked == 1
                    && recalc.spill_facts_published == 0
                    && recalc.spill_ghost_cells_published == 0
                    && recalc.spill_repair_passes == 0
        ));
        let merged_invalidation = merged_region_case.invalidation_report.as_ref().unwrap();
        assert_eq!(merged_invalidation.spill_blocker_edge_count, 1);
        assert_eq!(merged_invalidation.spill_blocker_closures.len(), 1);
        assert!(merged_invalidation.spill_blocker_closures[0].matched);
        assert_eq!(
            merged_invalidation.installed_dependencies[0].spill_blocker_dependencies,
            vec![GridSpillBlockerDependency::extent(
                rect(1, 1, 3, 1).unwrap()
            )]
        );
        assert_eq!(merged_invalidation.structural_edits.len(), 1);
        assert_eq!(
            merged_invalidation.structural_edits[0]
                .edit_report
                .spill_blocker_edges_after,
            1
        );
    }

    #[test]
    fn grid_runner_optimized_mode_keeps_reference_non_claiming() {
        let report = GridCorpusRunner::new()
            .run_seed_corpus_arg("optimized")
            .unwrap();

        assert_eq!(report.engine_mode, GridEngineMode::Optimized);
        assert!(report.matched());
        assert!(report.cases.iter().all(|case| {
            case.engine_report.reference.is_none() && case.engine_report.optimized.is_some()
        }));
    }

    #[test]
    fn grid_runner_seed_corpus_reports_optimized_repeated_template_counter() {
        let report = GridCorpusRunner::new()
            .run_seed_corpus(GridEngineMode::Both)
            .unwrap();
        let repeated_case = report
            .cases
            .iter()
            .find(|case| case.case_id == "grid_seed_repeated_r1c1_formula_region_001")
            .unwrap();
        let optimized = repeated_case.engine_report.optimized.as_ref().unwrap();

        assert!(matches!(
            &optimized.recalc,
            GridEngineRecalcReport::Optimized(recalc)
                if recalc.formula_cells == 4
                    && recalc.formula_templates_prepared == 1
                    && recalc.p11_template_prepare_once_holds()
        ));
        let warm_noop = optimized.warm_noop.as_ref().unwrap();
        assert!(warm_noop.recalc.p19_warm_noop_holds());
        assert_eq!(warm_noop.recalc.cached_occupied_cells, 8);
        assert_eq!(warm_noop.recalc.cached_formula_cells, 4);
        assert_eq!(warm_noop.readout, optimized.readout);
    }

    #[test]
    fn grid_runner_execute_seed_corpus_emits_artifacts() {
        let temp_root = std::env::temp_dir().join(format!(
            "oxcalc-grid-runner-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let run_id = "unit-grid-seed-run";
        let report = GridCorpusRunner::new()
            .execute_seed_corpus(&temp_root, run_id, "both")
            .unwrap();
        let artifact_root = temp_root.join(grid_artifact_root_relative(run_id));

        assert!(report.matched());
        assert!(artifact_root.join("run_summary.json").exists());
        assert!(artifact_root.join("case_index.json").exists());

        let summary: Value = serde_json::from_str(
            &std::fs::read_to_string(artifact_root.join("run_summary.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(summary["schema_version"], GRID_RUN_REPORT_SCHEMA_V1);
        assert_eq!(summary["engine_mode"], "both");
        assert_eq!(summary["case_count"], 18);
        assert_eq!(summary["invalidation_mismatch_count"], 0);
        assert_eq!(summary["matched"], true);

        let index: Value = serde_json::from_str(
            &std::fs::read_to_string(artifact_root.join("case_index.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(index["schema_version"], GRID_CASE_INDEX_SCHEMA_V1);
        assert_eq!(index["cases"].as_array().unwrap().len(), 18);

        let repeated: Value = serde_json::from_str(
            &std::fs::read_to_string(
                artifact_root
                    .join("cases")
                    .join("grid_seed_repeated_r1c1_formula_region_001")
                    .join("result.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(repeated["schema_version"], GRID_CASE_RESULT_SCHEMA_V1);
        assert_eq!(repeated["status"], "matched");
        assert_eq!(
            repeated["optimized"]["recalc"]["formula_templates_prepared"],
            1
        );
        assert_eq!(
            repeated["optimized"]["recalc"]["p11_template_prepare_once_holds"],
            true
        );
        assert_eq!(
            repeated["optimized"]["warm_noop"]["recalc"]["p19_warm_noop_holds"],
            true
        );
        assert_eq!(
            repeated["optimized"]["warm_noop"]["recalc"]["cells_visited"],
            0
        );
        assert_eq!(
            repeated["optimized"]["warm_noop"]["readout"],
            repeated["optimized"]["readout"]
        );
        assert_eq!(repeated["invalidation"]["matched"], true);
        assert_eq!(repeated["invalidation"]["scalar_edge_count"], 4);
        assert!(repeated["reference"].is_object());

        let defined_name: Value = serde_json::from_str(
            &std::fs::read_to_string(
                artifact_root
                    .join("cases")
                    .join("grid_seed_defined_name_value_resolution_001")
                    .join("result.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(defined_name["status"], "matched");
        assert_eq!(defined_name["invalidation"]["name_edge_count"], 2);
        assert_eq!(
            defined_name["invalidation"]["namespace_lifecycle"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            defined_name["invalidation"]["namespace_lifecycle"][0]["matched"],
            true
        );
        assert_eq!(
            defined_name["invalidation"]["namespace_lifecycle"][1]["lifecycle_report"]["name_edges_after"],
            0
        );

        let table_reference: Value = serde_json::from_str(
            &std::fs::read_to_string(
                artifact_root
                    .join("cases")
                    .join("grid_seed_table_overlay_structured_reference_001")
                    .join("result.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(table_reference["status"], "matched");
        assert_eq!(table_reference["invalidation"]["table_edge_count"], 2);
        assert_eq!(
            table_reference["invalidation"]["namespace_lifecycle"]
                .as_array()
                .unwrap()
                .len(),
            3
        );
        assert_eq!(
            table_reference["invalidation"]["namespace_lifecycle"][1]["dirty_closures"][0]["matched"],
            true
        );
        assert_eq!(
            table_reference["invalidation"]["namespace_lifecycle"][2]["lifecycle_report"]["table_edges_after"],
            0
        );

        let structural: Value = serde_json::from_str(
            &std::fs::read_to_string(
                artifact_root
                    .join("cases")
                    .join("grid_seed_optimized_structural_edit_repeated_r1c1_001")
                    .join("result.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(structural["status"], "matched");
        assert_eq!(structural["expected_values"][2]["expected"], "");
        assert_eq!(structural["optimized"]["readout"][2]["computed"], "");
        assert_eq!(
            structural["optimized"]["recalc"]["formula_templates_prepared"],
            1
        );

        let hidden: Value = serde_json::from_str(
            &std::fs::read_to_string(
                artifact_root
                    .join("cases")
                    .join("grid_seed_hidden_row_subtotal_visibility_001")
                    .join("result.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(hidden["status"], "matched");
        assert_eq!(hidden["optimized"]["readout"][0]["computed"], "10");
        assert_eq!(hidden["optimized"]["readout"][1]["computed"], "11");
        assert_eq!(
            hidden["invalidation"]["axis_visibility_closures"][0]["matched"],
            true
        );
        assert_eq!(
            hidden["invalidation"]["installed_dependencies"][0]["axis_visibility_dependencies"][0]
                ["axis"],
            "row"
        );

        let whole_axis: Value = serde_json::from_str(
            &std::fs::read_to_string(
                artifact_root
                    .join("cases")
                    .join("grid_seed_whole_axis_value_invalidation_001")
                    .join("result.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(whole_axis["status"], "matched");
        assert_eq!(whole_axis["optimized"]["readout"][0]["computed"], "12");
        assert_eq!(whole_axis["optimized"]["readout"][1]["computed"], "16");
        assert_eq!(whole_axis["optimized"]["readout"][2]["computed"], "28");
        assert_eq!(whole_axis["invalidation"]["axis_value_edge_count"], 2);
        assert_eq!(
            whole_axis["invalidation"]["installed_dependencies"][0]["axis_value_dependencies"][0]["axis"],
            "row"
        );
        assert_eq!(
            whole_axis["invalidation"]["installed_dependencies"][1]["axis_value_dependencies"][0]["axis"],
            "column"
        );
        assert_eq!(
            whole_axis["invalidation"]["structural_edits"][0]["edit_report"]["axis_value_edges_after"],
            2
        );
        assert_eq!(whole_axis["p20"]["matched"], true);
        assert_eq!(whole_axis["p20"]["mismatch_count"], 0);
        assert_eq!(
            whole_axis["p20"]["formula_enumerations"][0]["reference_source_text"],
            "1:1"
        );
        assert_eq!(
            whole_axis["p20"]["formula_enumerations"][0]["declared_cell_count"],
            32
        );
        assert_eq!(
            whole_axis["p20"]["formula_enumerations"][0]["slots_visited"],
            2
        );
        assert_eq!(
            whole_axis["p20"]["formula_enumerations"][0]["p20_occupied_slots_holds"],
            true
        );
        assert_eq!(
            whole_axis["p20"]["formula_enumerations"][1]["reference_source_text"],
            "A:B"
        );
        assert_eq!(
            whole_axis["p20"]["formula_enumerations"][1]["declared_cell_count"],
            256
        );
        assert_eq!(
            whole_axis["p20"]["formula_enumerations"][1]["slots_visited"],
            2
        );
        assert_eq!(
            whole_axis["p20"]["formula_enumerations"][1]["p20_occupied_slots_holds"],
            true
        );

        let spill: Value = serde_json::from_str(
            &std::fs::read_to_string(
                artifact_root
                    .join("cases")
                    .join("grid_seed_spill_anchor_ledger_001")
                    .join("result.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(spill["status"], "matched");
        assert_eq!(spill["optimized"]["readout"][0]["computed"], "28");
        assert_eq!(spill["optimized"]["readout"][1]["computed"], "29");
        assert_eq!(spill["invalidation"]["spill_closures"][0]["matched"], true);
        assert_eq!(
            spill["invalidation"]["installed_dependencies"][0]["spill_dependencies"][0]["anchor"]["r1c1"],
            "R1C1"
        );

        let sequence_spill: Value = serde_json::from_str(
            &std::fs::read_to_string(
                artifact_root
                    .join("cases")
                    .join("grid_seed_dynamic_sequence_spill_001")
                    .join("result.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(sequence_spill["status"], "matched");
        assert_eq!(sequence_spill["optimized"]["readout"][0]["computed"], "1");
        assert_eq!(sequence_spill["optimized"]["readout"][1]["computed"], "2");
        assert_eq!(sequence_spill["optimized"]["readout"][2]["computed"], "3");
        assert_eq!(sequence_spill["optimized"]["readout"][3]["computed"], "6");
        assert_eq!(sequence_spill["optimized"]["readout"][4]["computed"], "7");
        assert_eq!(
            sequence_spill["optimized"]["recalc"]["spill_facts_published"],
            1
        );
        assert_eq!(
            sequence_spill["optimized"]["recalc"]["spill_ghost_cells_published"],
            2
        );
        assert_eq!(
            sequence_spill["invalidation"]["spill_closures"][0]["matched"],
            true
        );

        let sequence_repair: Value = serde_json::from_str(
            &std::fs::read_to_string(
                artifact_root
                    .join("cases")
                    .join("grid_seed_dynamic_sequence_spill_repair_001")
                    .join("result.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(sequence_repair["status"], "matched");
        assert_eq!(sequence_repair["optimized"]["readout"][0]["computed"], "6");
        assert_eq!(sequence_repair["optimized"]["readout"][1]["computed"], "1");
        assert_eq!(sequence_repair["optimized"]["readout"][2]["computed"], "2");
        assert_eq!(sequence_repair["optimized"]["readout"][3]["computed"], "3");
        assert_eq!(
            sequence_repair["optimized"]["recalc"]["spill_repair_passes"],
            1
        );
        assert_eq!(
            sequence_repair["optimized"]["recalc"]["spill_repair_formula_evaluations"],
            2
        );
        assert_eq!(
            sequence_repair["optimized"]["recalc"]["spill_repair_converged"],
            true
        );
        assert_eq!(
            sequence_repair["invalidation"]["spill_closures"][0]["matched"],
            true
        );

        let mutual_spill: Value = serde_json::from_str(
            &std::fs::read_to_string(
                artifact_root
                    .join("cases")
                    .join("grid_seed_mutual_sequence_spill_blockage_001")
                    .join("result.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(mutual_spill["status"], "matched");
        assert_eq!(mutual_spill["optimized"]["readout"][0]["computed"], "Spill");
        assert_eq!(mutual_spill["optimized"]["readout"][1]["computed"], "Spill");
        assert_eq!(mutual_spill["optimized"]["readout"][2]["computed"], "");
        assert_eq!(mutual_spill["optimized"]["readout"][3]["computed"], "");
        assert_eq!(
            mutual_spill["optimized"]["recalc"]["spill_facts_blocked"],
            2
        );
        assert_eq!(
            mutual_spill["optimized"]["recalc"]["spill_facts_published"],
            0
        );

        let table_overlay: Value = serde_json::from_str(
            &std::fs::read_to_string(
                artifact_root
                    .join("cases")
                    .join("grid_seed_table_overlay_spill_blockage_001")
                    .join("result.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(table_overlay["status"], "matched");
        assert_eq!(
            table_overlay["optimized"]["readout"][0]["computed"],
            "Spill"
        );
        assert_eq!(table_overlay["optimized"]["readout"][1]["computed"], "");
        assert_eq!(table_overlay["optimized"]["readout"][2]["computed"], "");
        assert_eq!(
            table_overlay["optimized"]["recalc"]["spill_facts_blocked"],
            1
        );
        assert_eq!(
            table_overlay["optimized"]["recalc"]["spill_facts_published"],
            0
        );
        assert_eq!(
            table_overlay["optimized"]["recalc"]["spill_ghost_cells_published"],
            0
        );
        assert_eq!(
            table_overlay["optimized"]["recalc"]["spill_repair_passes"],
            0
        );

        let merged_region: Value = serde_json::from_str(
            &std::fs::read_to_string(
                artifact_root
                    .join("cases")
                    .join("grid_seed_merged_region_spill_blockage_001")
                    .join("result.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(merged_region["status"], "matched");
        assert_eq!(
            merged_region["optimized"]["readout"][0]["computed"],
            "Spill"
        );
        assert_eq!(merged_region["optimized"]["readout"][1]["computed"], "");
        assert_eq!(merged_region["optimized"]["readout"][2]["computed"], "");
        assert_eq!(
            merged_region["optimized"]["recalc"]["spill_facts_blocked"],
            1
        );
        assert_eq!(
            merged_region["optimized"]["recalc"]["spill_facts_published"],
            0
        );
        assert_eq!(
            merged_region["optimized"]["recalc"]["spill_ghost_cells_published"],
            0
        );
        assert_eq!(
            merged_region["optimized"]["recalc"]["spill_repair_passes"],
            0
        );
        assert_eq!(merged_region["invalidation"]["spill_blocker_edge_count"], 1);
        assert_eq!(
            merged_region["invalidation"]["installed_dependencies"][0]["spill_blocker_dependencies"]
                [0]["extent"]["a1"],
            "R1C1:R3C1"
        );
        assert_eq!(
            merged_region["invalidation"]["spill_blocker_closures"][0]["matched"],
            true
        );
        assert_eq!(
            merged_region["invalidation"]["structural_edits"][0]["edit_report"]["spill_blocker_edges_after"],
            1
        );

        let dynamic: Value = serde_json::from_str(
            &std::fs::read_to_string(
                artifact_root
                    .join("cases")
                    .join("grid_seed_dynamic_invalidation_request_001")
                    .join("result.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(dynamic["status"], "matched");
        assert_eq!(dynamic["invalidation"]["matched"], true);
        assert_eq!(
            dynamic["invalidation"]["installed_dependencies"][0]["dynamic_dependencies"][0],
            "indirect:Sheet1!A1"
        );
        assert_eq!(
            dynamic["invalidation"]["dynamic_closures"][0]["matched"],
            true
        );

        std::fs::remove_dir_all(temp_root).unwrap();
    }
}
