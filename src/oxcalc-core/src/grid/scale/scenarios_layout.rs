//! Grid scale scenarios - storage and layout profiles: sparse whole-column,
//! full column, sparse singletons, zig-zag, dense values, the repeated/
//! fill-down/Pascal R1C1 seeds, and the boring 1Mx10 baseline. Each runs a
//! profile and asserts its P-register counters. Invoked by the scale
//! dispatch; shares harness helpers via `use super::*`.

use super::*;

pub(super) fn sparse_whole_column_scale(
    _options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = ExcelGridBounds::strict_excel();
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let middle_row = bounds.max_rows / 2;
    sheet.set_literal(address(1, 1), CalcValue::number(5.0))?;
    sheet.set_literal(address(middle_row, 1), CalcValue::number(7.0))?;
    sheet.set_literal(address(bounds.max_rows, 1), CalcValue::number(11.0))?;
    sheet.set_formula(
        address(1, 2),
        GridFormulaCell::new("=SUM(A:A)", "excel.grid.v1:sum-whole-column:C1"),
    )?;

    let report =
        sheet.run_engine_mode_with_oxfml(GridEngineMode::Optimized, [address(1, 2)], 100_000)?;
    let p20_reports =
        sheet.optimized_formula_reference_enumeration_reports(&address(1, 2), 100_000)?;
    let Some(p20) = p20_reports.first() else {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "sparse whole-column scale produced no P-20 enumeration report".to_string(),
        });
    };
    let computed_sum = report
        .optimized
        .as_ref()
        .and_then(|run| run.readout.first())
        .map(|readout| readout.computed.clone())
        .unwrap_or_else(CalcValue::empty);
    let byte_report = sheet.storage_byte_report();

    Ok(json!({
        "counters": {
            "declared_cell_count": p20.declared_cell_count,
            "defined_cell_count": p20.defined_cell_count,
            "slots_visited": p20.slots_visited(),
            "sparse_value_cells_visited": p20.sparse_value_cells_visited,
            "authored_storage_bytes": byte_report.authored_storage_bytes,
            "sparse_point_bytes": byte_report.sparse_point_bytes,
            "sparse_point_bytes_per_cell_micros": byte_report.sparse_point_bytes_per_cell_micros(),
            "grid_cell_capacity": byte_report.grid_cell_capacity,
            "blank_cells": byte_report.blank_cells,
            "blank_cell_bytes": byte_report.blank_cell_bytes,
            "computed_sum": calc_value_display_text(&computed_sum)
        },
        "register_assertions": [
            register_assertion(
                "P-10",
                "blank cells require zero authored storage bytes",
                byte_report.p10_blank_cells_zero_bytes_holds()
            ),
            register_assertion(
                "P-20",
                "strict A:A provider enumeration visits occupied slots only",
                p20.p20_occupied_slots_holds()
                    && p20.declared_cell_count == usize::try_from(bounds.max_rows).unwrap()
                    && p20.defined_cell_count == 3
                    && p20.slots_visited() == 3
            )
        ]
    }))
}

pub(super) fn full_column_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "full-column-1m requires at least 2 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| f64::from(address.row))?;
    sheet.set_formula(
        address(1, 2),
        GridFormulaCell::new("=SUM(A:A)", "excel.grid.v1:sum-whole-column:C1"),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let p20_reports = sheet
        .optimized_formula_reference_enumeration_reports(&address(1, 2), materialization_limit)?;
    let Some(p20) = p20_reports.first() else {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "full-column-1m scale produced no P-20 enumeration report".to_string(),
        });
    };
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let computed_sum = valuation.read_cell(&address(1, 2)).computed;
    let expected_sum =
        u64::from(options.rows).saturating_mul(u64::from(options.rows).saturating_add(1)) / 2;
    let expected_sum_display = expected_sum.to_string();

    Ok(json!({
        "counters": {
            "declared_cell_count": p20.declared_cell_count,
            "defined_cell_count": p20.defined_cell_count,
            "slots_visited": p20.slots_visited(),
            "dense_value_cells_visited": p20.dense_value_cells_visited,
            "sparse_value_cells_visited": p20.sparse_value_cells_visited,
            "compact_regions_intersected": p20.compact_regions_intersected,
            "dense_value_regions": stats.dense_value_regions,
            "dense_value_cells": stats.dense_value_cells,
            "dense_numeric_packed_cells": byte_report.dense_numeric_packed_cells,
            "sparse_point_cells": stats.sparse_point_cells,
            "authored_storage_bytes": byte_report.authored_storage_bytes,
            "dense_value_region_bytes": byte_report.dense_value_region_bytes,
            "dense_bytes_per_cell_micros": byte_report.dense_bytes_per_cell_micros(),
            "authored_bytes_per_cell_micros": byte_report.authored_bytes_per_cell_micros(),
            "blank_cells": byte_report.blank_cells,
            "blank_cell_bytes": byte_report.blank_cell_bytes,
            "occupied_cells": recalc.occupied_cells,
            "literal_cells": recalc.literal_cells,
            "formula_cells": recalc.formula_cells,
            "formula_evaluations": recalc.formula_evaluations,
            "computed_dense_value_regions": recalc.computed_dense_value_regions,
            "computed_sparse_cells": valuation.sparse_computed_cells(),
            "computed_sum": calc_value_display_text(&computed_sum),
            "expected_sum": expected_sum_display.clone(),
            "warm_cells_visited": warm.cells_visited,
            "warm_formula_evaluations": warm.formula_evaluations
        },
        "register_assertions": [
            register_assertion(
                "P-00",
                "full-column-1M recalc visits each occupied cell once",
                recalc.p00_primary_exact_once_holds()
                    && recalc.literal_cells == u64::from(options.rows)
                    && recalc.formula_cells == 1
            ),
            register_assertion(
                "P-10",
                "full-column-1M dense numeric authored values stay within 17 B/cell and blanks cost zero bytes",
                byte_report.p10_dense_value_budget_holds()
                    && byte_report.p10_blank_cells_zero_bytes_holds()
            ),
            register_assertion(
                "P-19",
                "unchanged full-column-1M sheet hits warm no-op cache",
                warm.p19_warm_noop_holds()
            ),
            register_assertion(
                "P-20",
                "full-column-1M SUM(A:A) visits occupied dense slots only",
                p20.p20_occupied_slots_holds()
                    && p20.declared_cell_count == usize::try_from(options.rows).unwrap_or(usize::MAX)
                    && p20.defined_cell_count == usize::try_from(options.rows).unwrap_or(usize::MAX)
                    && p20.slots_visited() == u64::from(options.rows)
                    && p20.dense_value_cells_visited == u64::from(options.rows)
                    && p20.sparse_value_cells_visited == 0
                    && p20.compact_regions_intersected == 1
            ),
            register_assertion(
                "GRID-FULL-COLUMN-1M",
                "full-column-1M keeps the occupied column dense and computes the expected aggregate",
                stats.dense_value_regions == 1
                    && stats.dense_value_cells == u64::from(options.rows)
                    && byte_report.dense_numeric_packed_cells == u64::from(options.rows)
                    && stats.sparse_point_cells == 1
                    && calc_value_display_text(&computed_sum) == expected_sum_display
            )
        ]
    }))
}

pub(super) fn sparse_singletons_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    for row in 1..=options.rows {
        let col = ((row - 1) % options.cols) + 1;
        sheet.set_literal(address(row, col), CalcValue::number(f64::from(row)))?;
    }
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let first_value = sheet
        .authored_cell_at(&address(1, 1))
        .and_then(|readout| readout.authored)
        .unwrap_or_else(|| GridAuthoredCell::Literal(CalcValue::empty()));
    let middle_row = (options.rows / 2).max(1);
    let middle_col = ((middle_row - 1) % options.cols) + 1;
    let middle_value = sheet
        .authored_cell_at(&address(middle_row, middle_col))
        .and_then(|readout| readout.authored)
        .unwrap_or_else(|| GridAuthoredCell::Literal(CalcValue::empty()));
    let last_col = ((options.rows - 1) % options.cols) + 1;
    let last_value = sheet
        .authored_cell_at(&address(options.rows, last_col))
        .and_then(|readout| readout.authored)
        .unwrap_or_else(|| GridAuthoredCell::Literal(CalcValue::empty()));

    Ok(json!({
        "counters": {
            "sparse_point_cells": stats.sparse_point_cells,
            "authored_cells_upper_bound": stats.authored_cells_upper_bound,
            "authored_storage_bytes": byte_report.authored_storage_bytes,
            "sparse_point_bytes": byte_report.sparse_point_bytes,
            "sparse_point_bytes_per_cell_micros": byte_report.sparse_point_bytes_per_cell_micros(),
            "grid_cell_capacity": byte_report.grid_cell_capacity,
            "blank_cells": byte_report.blank_cells,
            "blank_cell_bytes": byte_report.blank_cell_bytes,
            "sample_first_value": authored_cell_display_text(&first_value),
            "sample_middle_value": authored_cell_display_text(&middle_value),
            "sample_last_value": authored_cell_display_text(&last_value)
        },
        "register_assertions": [
            register_assertion(
                "P-10",
                "sparse numeric singletons stay within 85 B/cell and blanks cost zero bytes",
                byte_report.p10_sparse_singleton_budget_holds()
                    && byte_report.p10_blank_cells_zero_bytes_holds()
                    && stats.sparse_point_cells == usize::try_from(options.rows).unwrap_or(usize::MAX)
                    && stats.authored_cells_upper_bound == u64::from(options.rows)
            )
        ]
    }))
}

pub(super) fn zig_zag_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    for row in 1..=options.rows {
        let col = ((row - 1) % options.cols) + 1;
        sheet.set_literal(address(row, col), CalcValue::number(f64::from(row)))?;
    }
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let middle_row = (options.rows / 2).max(1);
    let first_col = zig_zag_col(1, options.cols);
    let middle_col = zig_zag_col(middle_row, options.cols);
    let last_col = zig_zag_col(options.rows, options.cols);
    let first_value = sheet
        .authored_cell_at(&address(1, first_col))
        .and_then(|readout| readout.authored)
        .unwrap_or_else(|| GridAuthoredCell::Literal(CalcValue::empty()));
    let middle_value = sheet
        .authored_cell_at(&address(middle_row, middle_col))
        .and_then(|readout| readout.authored)
        .unwrap_or_else(|| GridAuthoredCell::Literal(CalcValue::empty()));
    let last_value = sheet
        .authored_cell_at(&address(options.rows, last_col))
        .and_then(|readout| readout.authored)
        .unwrap_or_else(|| GridAuthoredCell::Literal(CalcValue::empty()));

    Ok(json!({
        "counters": {
            "sparse_point_cells": stats.sparse_point_cells,
            "authored_cells_upper_bound": stats.authored_cells_upper_bound,
            "authored_storage_bytes": byte_report.authored_storage_bytes,
            "sparse_point_bytes": byte_report.sparse_point_bytes,
            "sparse_point_bytes_per_cell_micros": byte_report.sparse_point_bytes_per_cell_micros(),
            "grid_cell_capacity": byte_report.grid_cell_capacity,
            "blank_cells": byte_report.blank_cells,
            "blank_cell_bytes": byte_report.blank_cell_bytes,
            "columns_spanned": options.cols,
            "sample_first_col": first_col,
            "sample_middle_col": middle_col,
            "sample_last_col": last_col,
            "sample_first_value": authored_cell_display_text(&first_value),
            "sample_middle_value": authored_cell_display_text(&middle_value),
            "sample_last_value": authored_cell_display_text(&last_value),
            "partition_sparse_point_cells": partition.sparse_point_cells,
            "partition_dense_value_regions": partition.dense_value_regions,
            "partition_repeated_formula_regions": partition.repeated_formula_regions,
            "partition_dense_value_pair_checks": partition.dense_value_pair_checks,
            "partition_repeated_formula_pair_checks": partition.repeated_formula_pair_checks,
            "partition_dense_value_overlap_count": partition.dense_value_overlap_count,
            "partition_repeated_formula_overlap_count": partition.repeated_formula_overlap_count,
            "partition_max_parallelism_bound": partition.max_parallelism_bound
        },
        "register_assertions": [
            register_assertion(
                "P-10",
                "zig-zag-1M sparse diagonal singletons stay within 85 B/cell and blanks cost zero bytes",
                byte_report.p10_sparse_singleton_budget_holds()
                    && byte_report.p10_blank_cells_zero_bytes_holds()
                    && stats.sparse_point_cells == usize::try_from(options.rows).unwrap_or(usize::MAX)
                    && stats.authored_cells_upper_bound == u64::from(options.rows)
            ),
            register_assertion(
                "P-18",
                "zig-zag-1M sparse singleton partition witness is valid and records parallelism bound",
                partition.p18_partition_witness_holds()
                    && partition.sparse_point_cells == usize::try_from(options.rows).unwrap_or(usize::MAX)
                    && partition.max_parallelism_bound == u64::from(options.rows)
            ),
            register_assertion(
                "GRID-ZIG-ZAG-1M",
                "zig-zag-1M spans the configured column width with one sparse point per row",
                options.cols == bounds.max_cols
                    && stats.sparse_point_cells == usize::try_from(options.rows).unwrap_or(usize::MAX)
                    && authored_cell_display_text(&first_value) == "1"
                    && authored_cell_display_text(&middle_value) == middle_row.to_string()
                    && authored_cell_display_text(&last_value) == options.rows.to_string()
            )
        ]
    }))
}

pub(super) fn dense_values_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        options.cols,
        bounds,
    )?;
    sheet.put_dense_literal_region_with(rect, |address| {
        CalcValue::number(f64::from(address.row) + f64::from(address.col) / 1000.0)
    })?;
    let (valuation, recalc) = sheet.recalculate_mark_all_dirty_compact_with_oxfml(100_000)?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let expected_cells = u64::from(options.rows) * u64::from(options.cols);

    Ok(json!({
        "counters": {
            "dense_value_regions": stats.dense_value_regions,
            "dense_value_cells": stats.dense_value_cells,
            "dense_numeric_packed_cells": byte_report.dense_numeric_packed_cells,
            "authored_storage_bytes": byte_report.authored_storage_bytes,
            "dense_value_region_bytes": byte_report.dense_value_region_bytes,
            "dense_bytes_per_cell_micros": byte_report.dense_bytes_per_cell_micros(),
            "authored_bytes_per_cell_micros": byte_report.authored_bytes_per_cell_micros(),
            "blank_cells": byte_report.blank_cells,
            "blank_cell_bytes": byte_report.blank_cell_bytes,
            "sparse_point_cells": stats.sparse_point_cells,
            "computed_dense_value_regions": recalc.computed_dense_value_regions,
            "computed_sparse_cells": valuation.sparse_computed_cells(),
            "literal_cells": recalc.literal_cells,
            "occupied_cells": recalc.occupied_cells
        },
        "register_assertions": [
            register_assertion("P-00", "dense value recalc visits each occupied value once", recalc.p00_primary_exact_once_holds()),
            register_assertion("P-10", "dense numeric authored values stay within 17 B/cell and blanks cost zero bytes", byte_report.p10_dense_value_budget_holds() && byte_report.p10_blank_cells_zero_bytes_holds()),
            register_assertion(
                "GRID-DENSE-VALUES-REGION",
                "dense values remain region-backed without sparse point expansion",
                stats.dense_value_regions == 1
                    && stats.dense_value_cells == expected_cells
                    && byte_report.dense_numeric_packed_cells == expected_cells
                    && stats.sparse_point_cells == 0
                    && recalc.computed_dense_value_regions == 1
                    && valuation.sparse_computed_cells() == 0
            )
        ]
    }))
}

pub(super) fn repeated_r1c1_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_literal_region_with(dense_rect, |address| {
        CalcValue::number(f64::from(address.row))
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        2,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(100_000)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let last_value = valuation.read_cell(&address(options.rows, 2)).computed;

    Ok(json!({
        "counters": {
            "repeated_formula_regions": stats.repeated_formula_regions,
            "repeated_formula_cells": stats.repeated_formula_cells,
            "distinct_repeated_formula_templates": stats.distinct_repeated_formula_templates,
            "dense_numeric_packed_cells": byte_report.dense_numeric_packed_cells,
            "authored_storage_bytes": byte_report.authored_storage_bytes,
            "dense_value_region_bytes": byte_report.dense_value_region_bytes,
            "repeated_formula_region_bytes": byte_report.repeated_formula_region_bytes,
            "dense_bytes_per_cell_micros": byte_report.dense_bytes_per_cell_micros(),
            "repeated_formula_bytes_per_cell_micros": byte_report.repeated_formula_bytes_per_cell_micros(),
            "authored_bytes_per_cell_micros": byte_report.authored_bytes_per_cell_micros(),
            "blank_cells": byte_report.blank_cells,
            "blank_cell_bytes": byte_report.blank_cell_bytes,
            "formula_templates_prepared": recalc.formula_templates_prepared,
            "distinct_formula_templates": recalc.distinct_formula_templates,
            "formula_cells": recalc.formula_cells,
            "formula_plan_cache_lookups": recalc.formula_plan_cache_lookups(),
            "formula_plan_cache_hits": recalc.formula_plan_cache_hits,
            "formula_plan_cache_misses": recalc.formula_plan_cache_misses,
            "formula_plan_cache_hit_rate_micros": recalc.formula_plan_cache_hit_rate_micros(),
            "computed_dense_value_regions": recalc.computed_dense_value_regions,
            "computed_sparse_cells": valuation.sparse_computed_cells(),
            "last_formula_value": calc_value_display_text(&last_value),
            "warm_cells_visited": warm.cells_visited,
            "warm_formula_evaluations": warm.formula_evaluations
        },
        "register_assertions": [
            register_assertion("P-00", "repeated R1C1 recalc visits each occupied cell once", recalc.p00_primary_exact_once_holds()),
            register_assertion("P-10", "repeated R1C1 formulas share authored bytes and dense inputs stay packed", byte_report.p10_dense_value_budget_holds() && byte_report.p10_repeated_formula_budget_holds() && byte_report.p10_blank_cells_zero_bytes_holds()),
            register_assertion("P-11", "repeated R1C1 prepares one template for the region", recalc.p11_template_prepare_once_holds() && recalc.formula_templates_prepared == 1),
            register_assertion(
                "P-14",
                "repeated R1C1 formula plan cache misses once and hits for the remaining formula cells",
                recalc.p14_plan_cache_hit_floor_holds()
                    && recalc.formula_plan_cache_misses == 1
                    && recalc.formula_plan_cache_hits == recalc.formula_cells.saturating_sub(1)
            ),
            register_assertion("P-19", "unchanged repeated R1C1 sheet hits warm no-op cache", warm.p19_warm_noop_holds())
        ]
    }))
}

pub(super) fn fill_down_r1c1_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.rows < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "fill-down-r1c1 requires at least 2 rows".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    sheet.set_literal(address(1, 1), CalcValue::number(1.0))?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        2,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new("=R[-1]C+1", "excel.grid.v1:r1c1-template:R[-1]C+1")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let first_value = valuation.read_cell(&address(1, 1)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_value = valuation.read_cell(&address(middle_row, 1)).computed;
    let last_value = valuation.read_cell(&address(options.rows, 1)).computed;
    let expected_formula_cells = u64::from(options.rows - 1);

    Ok(json!({
        "counters": {
            "repeated_formula_regions": stats.repeated_formula_regions,
            "repeated_formula_cells": stats.repeated_formula_cells,
            "distinct_repeated_formula_templates": stats.distinct_repeated_formula_templates,
            "sparse_point_cells": stats.sparse_point_cells,
            "authored_storage_bytes": byte_report.authored_storage_bytes,
            "sparse_point_bytes": byte_report.sparse_point_bytes,
            "repeated_formula_region_bytes": byte_report.repeated_formula_region_bytes,
            "repeated_formula_bytes_per_cell_micros": byte_report.repeated_formula_bytes_per_cell_micros(),
            "authored_bytes_per_cell_micros": byte_report.authored_bytes_per_cell_micros(),
            "blank_cells": byte_report.blank_cells,
            "blank_cell_bytes": byte_report.blank_cell_bytes,
            "formula_templates_prepared": recalc.formula_templates_prepared,
            "distinct_formula_templates": recalc.distinct_formula_templates,
            "formula_cells": recalc.formula_cells,
            "formula_evaluations": recalc.formula_evaluations,
            "formula_plan_cache_lookups": recalc.formula_plan_cache_lookups(),
            "formula_plan_cache_hits": recalc.formula_plan_cache_hits,
            "formula_plan_cache_misses": recalc.formula_plan_cache_misses,
            "formula_plan_cache_hit_rate_micros": recalc.formula_plan_cache_hit_rate_micros(),
            "last_formula_value": calc_value_display_text(&last_value),
            "middle_formula_value": calc_value_display_text(&middle_value),
            "first_value": calc_value_display_text(&first_value),
            "computed_dense_value_regions": recalc.computed_dense_value_regions,
            "computed_sparse_cells": valuation.sparse_computed_cells(),
            "warm_cells_visited": warm.cells_visited,
            "warm_formula_evaluations": warm.formula_evaluations
        },
        "register_assertions": [
            register_assertion("P-00", "fill-down R1C1 recalc visits each occupied cell once", recalc.p00_primary_exact_once_holds()),
            register_assertion("P-10", "fill-down R1C1 formulas share authored bytes and blanks cost zero bytes", byte_report.p10_repeated_formula_budget_holds() && byte_report.p10_blank_cells_zero_bytes_holds()),
            register_assertion(
                "P-11",
                "fill-down R1C1 prepares one template for the repeated region",
                recalc.p11_template_prepare_once_holds()
                    && recalc.formula_templates_prepared == 1
                    && stats.repeated_formula_cells == expected_formula_cells
            ),
            register_assertion(
                "P-14",
                "fill-down R1C1 formula plan cache misses once and hits for the remaining formula cells",
                recalc.p14_plan_cache_hit_floor_holds()
                    && recalc.formula_plan_cache_misses == 1
                    && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
            ),
            register_assertion("P-19", "unchanged fill-down R1C1 sheet hits warm no-op cache", warm.p19_warm_noop_holds()),
            register_assertion(
                "GRID-FILL-DOWN-R1C1",
                "fill-down R1C1 produces the expected first/middle/last values",
                calc_value_display_text(&first_value) == "1"
                    && calc_value_display_text(&middle_value) == middle_row.to_string()
                    && calc_value_display_text(&last_value) == options.rows.to_string()
                    && recalc.computed_dense_value_regions == 1
                    && valuation.sparse_computed_cells() == 1
            )
        ]
    }))
}

pub(super) fn pascal_r1c1_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.rows < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "pascal-r1c1-1m requires at least 2 rows".to_string(),
        });
    }
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "pascal-r1c1-1m requires at least 2 columns".to_string(),
        });
    }

    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let left_boundary = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_number_region_with(left_boundary, |_address| 1.0)?;
    for col in 2..=options.cols {
        sheet.set_literal(address(1, col), CalcValue::number(1.0))?;
    }

    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        2,
        2,
        options.rows,
        options.cols,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=RC[-1]+R[-1]C",
            "excel.grid.v1:r1c1-template:RC[-1]+R[-1]C",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_formula_cells =
        u64::from(options.rows.saturating_sub(1)).saturating_mul(u64::from(options.cols - 1));
    let expected_occupied_cells = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let expected_sparse_boundary_cells = usize::try_from(options.cols - 1).unwrap_or(usize::MAX);
    let middle_row = (options.rows / 2).max(2);
    let first_formula_value = valuation.read_cell(&address(2, 2)).computed;
    let middle_formula_value = valuation
        .read_cell(&address(middle_row, options.cols))
        .computed;
    let last_formula_value = valuation
        .read_cell(&address(options.rows, options.cols))
        .computed;
    let expected_first_formula = CalcValue::number(pascal_r1c1_expected_value(2, 2));
    let expected_middle_formula =
        CalcValue::number(pascal_r1c1_expected_value(middle_row, options.cols));
    let expected_last_formula =
        CalcValue::number(pascal_r1c1_expected_value(options.rows, options.cols));

    let counters = json_object([
        ("boundary_dense_value_cells", json!(stats.dense_value_cells)),
        (
            "boundary_sparse_point_cells",
            json!(stats.sparse_point_cells),
        ),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "distinct_repeated_formula_templates",
            json!(stats.distinct_repeated_formula_templates),
        ),
        (
            "authored_cells_upper_bound",
            json!(stats.authored_cells_upper_bound),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "dense_value_region_bytes",
            json!(byte_report.dense_value_region_bytes),
        ),
        ("sparse_point_bytes", json!(byte_report.sparse_point_bytes)),
        (
            "repeated_formula_region_bytes",
            json!(byte_report.repeated_formula_region_bytes),
        ),
        (
            "dense_bytes_per_cell_micros",
            json!(byte_report.dense_bytes_per_cell_micros()),
        ),
        (
            "sparse_point_bytes_per_cell_micros",
            json!(byte_report.sparse_point_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        ("blank_cells", json!(byte_report.blank_cells)),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "formula_plan_cache_hit_rate_micros",
            json!(recalc.formula_plan_cache_hit_rate_micros()),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        ("sample_middle_row", json!(middle_row)),
        (
            "sample_first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        (
            "sample_middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        (
            "sample_last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        (
            "expected_first_formula_value",
            json!(calc_value_display_text(&expected_first_formula)),
        ),
        (
            "expected_middle_formula_value",
            json!(calc_value_display_text(&expected_middle_formula)),
        ),
        (
            "expected_last_formula_value",
            json!(calc_value_display_text(&expected_last_formula)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_sparse_point_cells",
            json!(partition.sparse_point_cells),
        ),
        (
            "partition_dense_value_regions",
            json!(partition.dense_value_regions),
        ),
        (
            "partition_repeated_formula_regions",
            json!(partition.repeated_formula_regions),
        ),
        (
            "partition_dense_value_pair_checks",
            json!(partition.dense_value_pair_checks),
        ),
        (
            "partition_repeated_formula_pair_checks",
            json!(partition.repeated_formula_pair_checks),
        ),
        (
            "partition_dense_value_overlap_count",
            json!(partition.dense_value_overlap_count),
        ),
        (
            "partition_repeated_formula_overlap_count",
            json!(partition.repeated_formula_overlap_count),
        ),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "pascal-r1c1-1M recalc visits each occupied boundary/formula cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "pascal-r1c1-1M keeps dense boundary, sparse boundary, repeated formulas, and blanks within byte floors",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_sparse_singleton_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
        ),
        register_assertion(
            "P-11",
            "pascal-r1c1-1M prepares one R1C1 template for the 2D repeated formula region",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && stats.repeated_formula_cells == expected_formula_cells
        ),
        register_assertion(
            "P-14",
            "pascal-r1c1-1M formula plan cache misses once and hits for remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
        ),
        register_assertion(
            "P-18",
            "pascal-r1c1-1M compact boundary and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "P-19",
            "unchanged pascal-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "GRID-PASCAL-R1C1-1M",
            "pascal-r1c1-1M publishes a two-dimensional R1C1 recurrence as dense output with expected sampled values",
            stats.dense_value_cells == u64::from(options.rows)
                && stats.sparse_point_cells == expected_sparse_boundary_cells
                && stats.repeated_formula_regions == 1
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == expected_sparse_boundary_cells
                && first_formula_value == expected_first_formula
                && middle_formula_value == expected_middle_formula
                && last_formula_value == expected_last_formula
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn boring_1mx10_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "boring-1mx10 requires at least 2 columns".to_string(),
        });
    }
    let formula_cols = if options.cols >= 10 {
        2
    } else {
        (options.cols / 2).max(1)
    };
    let dense_cols = options.cols - formula_cols;
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        dense_cols,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        f64::from(address.row) * 1000.0 + f64::from(address.col)
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        dense_cols + 1,
        options.rows,
        options.cols,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows) * u64::from(dense_cols);
    let expected_formula_cells = u64::from(options.rows) * u64::from(formula_cols);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_dense_value = valuation.read_cell(&address(1, 1)).computed;
    let first_formula_value = valuation.read_cell(&address(1, dense_cols + 1)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_formula_value = valuation
        .read_cell(&address(middle_row, options.cols))
        .computed;
    let last_formula_value = valuation
        .read_cell(&address(options.rows, options.cols))
        .computed;
    let expected_last_formula = (f64::from(options.rows) * 1000.0 + f64::from(dense_cols))
        * 2_f64.powi(formula_cols as i32);
    let expected_last_formula_display = integer_display(expected_last_formula);

    let counters = json_object([
        ("dense_columns", json!(dense_cols)),
        ("formula_columns", json!(formula_cols)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "distinct_repeated_formula_templates",
            json!(stats.distinct_repeated_formula_templates),
        ),
        (
            "dense_numeric_packed_cells",
            json!(byte_report.dense_numeric_packed_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "dense_value_region_bytes",
            json!(byte_report.dense_value_region_bytes),
        ),
        (
            "repeated_formula_region_bytes",
            json!(byte_report.repeated_formula_region_bytes),
        ),
        (
            "dense_bytes_per_cell_micros",
            json!(byte_report.dense_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        ("blank_cells", json!(byte_report.blank_cells)),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "formula_plan_cache_hit_rate_micros",
            json!(recalc.formula_plan_cache_hit_rate_micros()),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_dense_value",
            json!(calc_value_display_text(&first_dense_value)),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        (
            "expected_last_formula_value",
            json!(expected_last_formula_display.clone()),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_sparse_point_cells",
            json!(partition.sparse_point_cells),
        ),
        (
            "partition_dense_value_regions",
            json!(partition.dense_value_regions),
        ),
        (
            "partition_repeated_formula_regions",
            json!(partition.repeated_formula_regions),
        ),
        (
            "partition_dense_value_pair_checks",
            json!(partition.dense_value_pair_checks),
        ),
        (
            "partition_repeated_formula_pair_checks",
            json!(partition.repeated_formula_pair_checks),
        ),
        (
            "partition_dense_value_overlap_count",
            json!(partition.dense_value_overlap_count),
        ),
        (
            "partition_repeated_formula_overlap_count",
            json!(partition.repeated_formula_overlap_count),
        ),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "boring-1Mx10 recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "boring-1Mx10 dense values and repeated formulas stay within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "boring-1Mx10 prepares one R1C1 template for the repeated formula block",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && stats.repeated_formula_cells == expected_formula_cells
        ),
        register_assertion(
            "P-14",
            "boring-1Mx10 formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
        ),
        register_assertion(
            "P-19",
            "unchanged boring-1Mx10 sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "boring-1Mx10 compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
                && partition.max_parallelism_bound == 2
        ),
        register_assertion(
            "GRID-BORING-1MX10",
            "boring-1Mx10 keeps authored values/formulas compact and produces expected sampled values",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_dense_value) == "1001"
                && calc_value_display_text(&last_formula_value) == expected_last_formula_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}
