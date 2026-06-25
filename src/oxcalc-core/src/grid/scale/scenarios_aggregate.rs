//! Grid scale scenarios - row-aggregate and error repeated-R1C1 profiles:
//! SUM/SUMSQ/COUNT/PRODUCT/AVERAGE/MIN-MAX rows, the windowed sum, and the
//! division-error, error-propagation, and aggregate-error families. Each runs
//! a profile and asserts its P-register counters. Invoked by the scale
//! dispatch; shares harness helpers via `use super::*`.

use super::*;

pub(super) fn sum_row_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "sum-row-r1c1-1m requires at least 4 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        f64::from(address.row) * f64::from(address.col)
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        4,
        options.rows,
        4,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=SUM(RC[-3]:RC[-1])",
            "excel.grid.v1:r1c1-template:SUM(RC[-3]:RC[-1])",
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
    let expected_dense_cells = u64::from(options.rows).saturating_mul(3);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_formula_value = valuation.read_cell(&address(1, 4)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_formula_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let expected_last_formula_display = integer_display(f64::from(options.rows) * 6.0);

    let counters = json_object([
        ("dense_columns", json!(3)),
        ("formula_columns", json!(1)),
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
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
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
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
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
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "sum-row-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "sum-row-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "sum-row-r1c1-1M prepares one R1C1 SUM range template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "sum-row-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged sum-row-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "sum-row-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-SUM-ROW-R1C1-1M",
            "sum-row-r1c1-1M publishes dense formula output for SUM(RC[-3]:RC[-1])",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_formula_value) == "6"
                && calc_value_display_text(&last_formula_value) == expected_last_formula_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn sumsq_row_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "sumsq-row-r1c1-1m requires at least 4 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        f64::from(address.row) * f64::from(address.col)
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        4,
        options.rows,
        4,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=SUMSQ(RC[-3]:RC[-1])",
            "excel.grid.v1:r1c1-template:SUMSQ(RC[-3]:RC[-1])",
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
    let expected_dense_cells = u64::from(options.rows).saturating_mul(3);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_formula_value = |row: u32| {
        let row = f64::from(row);
        CalcValue::number(14.0 * row * row)
    };
    let first_formula_value = valuation.read_cell(&address(1, 4)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_formula_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let expected_last_formula_display =
        integer_display(14.0 * f64::from(options.rows) * f64::from(options.rows));

    let counters = json_object([
        ("dense_columns", json!(3)),
        ("formula_columns", json!(1)),
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
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
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
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
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
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        ("middle_row", json!(middle_row)),
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
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "sumsq-row-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "sumsq-row-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "sumsq-row-r1c1-1M prepares one R1C1 SUMSQ range template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "sumsq-row-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged sumsq-row-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "sumsq-row-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-SUMSQ-ROW-R1C1-1M",
            "sumsq-row-r1c1-1M publishes dense formula output for SUMSQ(RC[-3]:RC[-1])",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.sparse_computed_cells() == 0
                && first_formula_value == expected_formula_value(1)
                && middle_formula_value == expected_formula_value(middle_row)
                && last_formula_value == expected_formula_value(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn count_row_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "count-row-r1c1-1m requires at least 4 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        f64::from(address.row) * f64::from(address.col)
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        4,
        options.rows,
        4,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=COUNT(RC[-3]:RC[-1])",
            "excel.grid.v1:r1c1-template:COUNT(RC[-3]:RC[-1])",
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
    let expected_dense_cells = u64::from(options.rows).saturating_mul(3);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_formula_value = valuation.read_cell(&address(1, 4)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_formula_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 4)).computed;

    let counters = json_object([
        ("dense_columns", json!(3)),
        ("formula_columns", json!(1)),
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
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
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
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
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
        ("expected_formula_value", json!("3")),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "count-row-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "count-row-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "count-row-r1c1-1M prepares one R1C1 COUNT range template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "count-row-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged count-row-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "count-row-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-COUNT-ROW-R1C1-1M",
            "count-row-r1c1-1M publishes dense formula output for COUNT(RC[-3]:RC[-1])",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_formula_value) == "3"
                && calc_value_display_text(&middle_formula_value) == "3"
                && calc_value_display_text(&last_formula_value) == "3"
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn product_row_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "product-row-r1c1-1m requires at least 4 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| f64::from(address.col))?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        4,
        options.rows,
        4,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=PRODUCT(RC[-3]:RC[-1])",
            "excel.grid.v1:r1c1-template:PRODUCT(RC[-3]:RC[-1])",
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
    let expected_dense_cells = u64::from(options.rows).saturating_mul(3);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_formula_value = valuation.read_cell(&address(1, 4)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_formula_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 4)).computed;

    let counters = json_object([
        ("dense_columns", json!(3)),
        ("formula_columns", json!(1)),
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
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
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
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
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
        ("expected_formula_value", json!("6")),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "product-row-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "product-row-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "product-row-r1c1-1M prepares one R1C1 PRODUCT range template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "product-row-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged product-row-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "product-row-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-PRODUCT-ROW-R1C1-1M",
            "product-row-r1c1-1M publishes dense formula output for PRODUCT(RC[-3]:RC[-1])",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_formula_value) == "6"
                && calc_value_display_text(&middle_formula_value) == "6"
                && calc_value_display_text(&last_formula_value) == "6"
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn average_row_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "average-row-r1c1-1m requires at least 4 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        f64::from(address.row) * f64::from(address.col)
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        4,
        options.rows,
        4,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=AVERAGE(RC[-3]:RC[-1])",
            "excel.grid.v1:r1c1-template:AVERAGE(RC[-3]:RC[-1])",
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
    let expected_dense_cells = u64::from(options.rows).saturating_mul(3);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_formula_value = valuation.read_cell(&address(1, 4)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_formula_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let expected_last_formula_display = integer_display(f64::from(options.rows) * 2.0);

    let counters = json_object([
        ("dense_columns", json!(3)),
        ("formula_columns", json!(1)),
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
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
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
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
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
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "average-row-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "average-row-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "average-row-r1c1-1M prepares one R1C1 AVERAGE range template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "average-row-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged average-row-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "average-row-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-AVERAGE-ROW-R1C1-1M",
            "average-row-r1c1-1M publishes dense formula output for AVERAGE(RC[-3]:RC[-1])",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_formula_value) == "2"
                && calc_value_display_text(&last_formula_value) == expected_last_formula_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn min_max_row_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 5 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "min-max-row-r1c1-1m requires at least 5 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        f64::from(address.row) * f64::from(address.col)
    })?;
    let min_formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        4,
        options.rows,
        4,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        min_formula_rect,
        GridFormulaCell::new(
            "=MIN(RC[-3]:RC[-1])",
            "excel.grid.v1:r1c1-template:MIN(RC[-3]:RC[-1])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let max_formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        5,
        options.rows,
        5,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        max_formula_rect,
        GridFormulaCell::new(
            "=MAX(RC[-4]:RC[-2])",
            "excel.grid.v1:r1c1-template:MAX(RC[-4]:RC[-2])",
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
    let expected_dense_cells = u64::from(options.rows).saturating_mul(3);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(2);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_min_value = valuation.read_cell(&address(1, 4)).computed;
    let first_max_value = valuation.read_cell(&address(1, 5)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_min_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let middle_max_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_min_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let last_max_value = valuation.read_cell(&address(options.rows, 5)).computed;
    let expected_last_min_display = integer_display(f64::from(options.rows));
    let expected_last_max_display = integer_display(f64::from(options.rows) * 3.0);

    let counters = json_object([
        ("dense_columns", json!(3)),
        ("formula_columns", json!(2)),
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
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
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
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
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
            "first_min_formula_value",
            json!(calc_value_display_text(&first_min_value)),
        ),
        (
            "first_max_formula_value",
            json!(calc_value_display_text(&first_max_value)),
        ),
        (
            "middle_min_formula_value",
            json!(calc_value_display_text(&middle_min_value)),
        ),
        (
            "middle_max_formula_value",
            json!(calc_value_display_text(&middle_max_value)),
        ),
        (
            "last_min_formula_value",
            json!(calc_value_display_text(&last_min_value)),
        ),
        (
            "last_max_formula_value",
            json!(calc_value_display_text(&last_max_value)),
        ),
        (
            "expected_last_min_formula_value",
            json!(expected_last_min_display.clone()),
        ),
        (
            "expected_last_max_formula_value",
            json!(expected_last_max_display.clone()),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "min-max-row-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "min-max-row-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "min-max-row-r1c1-1M prepares two R1C1 MIN/MAX range templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 2
                && recalc.compiled_formula_plans_cached == 2
        ),
        register_assertion(
            "P-14",
            "min-max-row-r1c1-1M formula plan cache misses twice and hits for remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 2
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(2)
                && recalc.compiled_formula_plan_cache_misses == 2
        ),
        register_assertion(
            "P-19",
            "unchanged min-max-row-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "min-max-row-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 2
        ),
        register_assertion(
            "GRID-MIN-MAX-ROW-R1C1-1M",
            "min-max-row-r1c1-1M publishes dense formula output for MIN/MAX row ranges",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 2
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 3
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_min_value) == "1"
                && calc_value_display_text(&first_max_value) == "3"
                && calc_value_display_text(&last_min_value) == expected_last_min_display
                && calc_value_display_text(&last_max_value) == expected_last_max_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn sum_window_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.rows < 3 || options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "sum-window-r1c1-1m requires at least 3 rows and 2 columns".to_string(),
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
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        3,
        2,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=SUM(R[-2]C[-1]:RC[-1])",
            "excel.grid.v1:r1c1-template:SUM(R[-2]C[-1]:RC[-1])",
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
    let expected_dense_cells = u64::from(options.rows);
    let expected_formula_cells = u64::from(options.rows.saturating_sub(2));
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_formula_value = valuation.read_cell(&address(3, 2)).computed;
    let middle_row = (options.rows / 2).max(3);
    let middle_formula_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 2)).computed;
    let expected_middle_formula_display = integer_display(f64::from(middle_row) * 3.0 - 3.0);
    let expected_last_formula_display = integer_display(f64::from(options.rows) * 3.0 - 3.0);

    let counters = json_object([
        ("dense_columns", json!(1)),
        ("formula_columns", json!(1)),
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
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
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
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
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
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        (
            "expected_middle_formula_value",
            json!(expected_middle_formula_display.clone()),
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
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "sum-window-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "sum-window-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "sum-window-r1c1-1M prepares one R1C1 SUM range template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "sum-window-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged sum-window-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "sum-window-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-SUM-WINDOW-R1C1-1M",
            "sum-window-r1c1-1M publishes dense formula output for SUM(R[-2]C[-1]:RC[-1])",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_formula_value) == "6"
                && calc_value_display_text(&middle_formula_value)
                    == expected_middle_formula_display
                && calc_value_display_text(&last_formula_value) == expected_last_formula_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn division_error_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "division-error-r1c1-1m requires at least 2 columns".to_string(),
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
    sheet.put_dense_number_region_with(dense_rect, |address| f64::from(address.row) * 2.0)?;
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
        GridFormulaCell::new("=RC[-1]/0", "excel.grid.v1:r1c1-template:RC[-1]/0")
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
    let expected_dense_cells = u64::from(options.rows);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_formula_value = valuation.read_cell(&address(1, 2)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_formula_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 2)).computed;
    let expected_error_display =
        calc_value_display_text(&CalcValue::error(WorksheetErrorCode::Div0));

    let counters = json_object([
        ("dense_columns", json!(1)),
        ("formula_columns", json!(1)),
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
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
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
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
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
            "expected_formula_error",
            json!(expected_error_display.clone()),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "division-error-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "division-error-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "division-error-r1c1-1M prepares one R1C1 template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "division-error-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged division-error-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "division-error-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-DIVISION-ERROR-R1C1-1M",
            "division-error-r1c1-1M publishes dense formula output for RC[-1]/0",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_formula_value) == expected_error_display
                && calc_value_display_text(&middle_formula_value) == expected_error_display
                && calc_value_display_text(&last_formula_value) == expected_error_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn division_error_propagation_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "division-error-propagation-r1c1-1m requires at least 3 columns".to_string(),
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
    sheet.put_dense_number_region_with(dense_rect, |address| f64::from(address.row) * 2.0)?;
    let division_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        2,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        division_rect,
        GridFormulaCell::new("=RC[-1]/0", "excel.grid.v1:r1c1-template:RC[-1]/0")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let propagation_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        3,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        propagation_rect,
        GridFormulaCell::new("=RC[-1]+1", "excel.grid.v1:r1c1-template:RC[-1]+1")
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
    let expected_dense_cells = u64::from(options.rows);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(2);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_direct_error = valuation.read_cell(&address(1, 2)).computed;
    let first_propagated_error = valuation.read_cell(&address(1, 3)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_propagated_error = valuation.read_cell(&address(middle_row, 3)).computed;
    let last_propagated_error = valuation.read_cell(&address(options.rows, 3)).computed;
    let expected_error_display =
        calc_value_display_text(&CalcValue::error(WorksheetErrorCode::Div0));

    let counters = json_object([
        ("dense_columns", json!(1)),
        ("formula_columns", json!(2)),
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
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
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
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
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
            "first_direct_error_value",
            json!(calc_value_display_text(&first_direct_error)),
        ),
        (
            "first_propagated_error_value",
            json!(calc_value_display_text(&first_propagated_error)),
        ),
        (
            "middle_propagated_error_value",
            json!(calc_value_display_text(&middle_propagated_error)),
        ),
        (
            "last_propagated_error_value",
            json!(calc_value_display_text(&last_propagated_error)),
        ),
        (
            "expected_formula_error",
            json!(expected_error_display.clone()),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "division-error-propagation-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "division-error-propagation-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "division-error-propagation-r1c1-1M prepares two R1C1 templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 2
                && recalc.compiled_formula_plans_cached == 2
        ),
        register_assertion(
            "P-14",
            "division-error-propagation-r1c1-1M formula plan cache misses once per template and hits for remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 2
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(2)
                && recalc.compiled_formula_plan_cache_misses == 2
        ),
        register_assertion(
            "P-19",
            "unchanged division-error-propagation-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "division-error-propagation-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 2
        ),
        register_assertion(
            "GRID-DIVISION-ERROR-PROPAGATION-R1C1-1M",
            "division-error-propagation-r1c1-1M keeps direct and propagated Div0 outputs dense",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 2
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 3
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_direct_error) == expected_error_display
                && calc_value_display_text(&first_propagated_error) == expected_error_display
                && calc_value_display_text(&middle_propagated_error) == expected_error_display
                && calc_value_display_text(&last_propagated_error) == expected_error_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn aggregate_error_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "aggregate-error-r1c1-1m requires at least 4 columns".to_string(),
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
    sheet.put_dense_number_region_with(dense_rect, |address| f64::from(address.row) * 2.0)?;
    let direct_error_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        2,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        direct_error_rect,
        GridFormulaCell::new("=RC[-1]/0", "excel.grid.v1:r1c1-template:RC[-1]/0")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let aggregate_error_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        3,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        aggregate_error_rect,
        GridFormulaCell::new(
            "=SUM(RC[-2]:RC[-1])",
            "excel.grid.v1:r1c1-template:SUM(RC[-2]:RC[-1])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let recovered_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        4,
        options.rows,
        4,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        recovered_rect,
        GridFormulaCell::new(
            "=IFERROR(SUM(RC[-3]:RC[-2]),RC[-3])",
            "excel.grid.v1:r1c1-template:IFERROR(SUM(RC[-3]:RC[-2]),RC[-3])",
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
    let expected_dense_cells = u64::from(options.rows);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(3);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_numeric_cells = u64::from(options.rows).saturating_mul(2);
    let first_direct_error = valuation.read_cell(&address(1, 2)).computed;
    let first_aggregate_error = valuation.read_cell(&address(1, 3)).computed;
    let first_recovered_value = valuation.read_cell(&address(1, 4)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_aggregate_error = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_recovered_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let last_aggregate_error = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_recovered_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let expected_error_display =
        calc_value_display_text(&CalcValue::error(WorksheetErrorCode::Div0));
    let expected_middle_recovered_display =
        calc_value_display_text(&CalcValue::number(f64::from(middle_row) * 2.0));
    let expected_last_recovered_display =
        calc_value_display_text(&CalcValue::number(f64::from(options.rows) * 2.0));

    let counters = json_object([
        ("dense_columns", json!(1)),
        ("formula_columns", json!(3)),
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
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
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
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
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
            "computed_dense_logical_packed_cells",
            json!(valuation.dense_computed_logical_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_direct_error_value",
            json!(calc_value_display_text(&first_direct_error)),
        ),
        (
            "first_aggregate_error_value",
            json!(calc_value_display_text(&first_aggregate_error)),
        ),
        (
            "middle_aggregate_error_value",
            json!(calc_value_display_text(&middle_aggregate_error)),
        ),
        (
            "last_aggregate_error_value",
            json!(calc_value_display_text(&last_aggregate_error)),
        ),
        (
            "first_recovered_value",
            json!(calc_value_display_text(&first_recovered_value)),
        ),
        (
            "middle_recovered_value",
            json!(calc_value_display_text(&middle_recovered_value)),
        ),
        (
            "last_recovered_value",
            json!(calc_value_display_text(&last_recovered_value)),
        ),
        (
            "expected_formula_error",
            json!(expected_error_display.clone()),
        ),
        (
            "expected_middle_recovered_value",
            json!(expected_middle_recovered_display.clone()),
        ),
        (
            "expected_last_recovered_value",
            json!(expected_last_recovered_display.clone()),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "aggregate-error-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "aggregate-error-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "aggregate-error-r1c1-1M prepares three R1C1 templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 3
                && recalc.compiled_formula_plans_cached == 3
        ),
        register_assertion(
            "P-14",
            "aggregate-error-r1c1-1M formula plan cache misses once per template and hits for remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 3
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(3)
                && recalc.compiled_formula_plan_cache_misses == 3
        ),
        register_assertion(
            "P-19",
            "unchanged aggregate-error-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "aggregate-error-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 3
        ),
        register_assertion(
            "GRID-AGGREGATE-ERROR-R1C1-1M",
            "aggregate-error-r1c1-1M propagates range-aggregate errors and recovers with IFERROR without sparse output",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 3
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 4
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_numeric_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_direct_error) == expected_error_display
                && calc_value_display_text(&first_aggregate_error) == expected_error_display
                && calc_value_display_text(&middle_aggregate_error) == expected_error_display
                && calc_value_display_text(&last_aggregate_error) == expected_error_display
                && calc_value_display_text(&first_recovered_value) == "2"
                && calc_value_display_text(&middle_recovered_value)
                    == expected_middle_recovered_display
                && calc_value_display_text(&last_recovered_value)
                    == expected_last_recovered_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}
