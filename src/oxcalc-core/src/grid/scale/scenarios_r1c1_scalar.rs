//! Grid scale scenarios - scalar and math-function repeated-R1C1 profiles:
//! direct/unary/two-left/absolute/division/decimal/recursive-binary refs and
//! the argument-aggregate, math, mod, rounding, integer, log, trig, and angle
//! function families. Each runs a profile and asserts its P-register
//! counters. Invoked by the scale dispatch; shares helpers via `use super::*`.

use super::*;

pub(super) fn direct_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "direct-r1c1-1m requires at least 3 columns".to_string(),
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
    sheet.put_dense_number_region_with(dense_rect, |address| f64::from(address.row) * 10.0)?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            2,
            options.rows,
            2,
            bounds,
        )?,
        GridFormulaCell::new("=RC[-1]", "excel.grid.v1:r1c1-template:RC[-1]")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new("=(RC[-2])", "excel.grid.v1:r1c1-template:(RC[-2])")
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
    let expected_value = |row: u32| CalcValue::number(f64::from(row) * 10.0);
    let middle_row = (options.rows / 2).max(1);
    let first_direct_value = valuation.read_cell(&address(1, 2)).computed;
    let first_parenthesized_value = valuation.read_cell(&address(1, 3)).computed;
    let middle_direct_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let middle_parenthesized_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let last_direct_value = valuation.read_cell(&address(options.rows, 2)).computed;
    let last_parenthesized_value = valuation.read_cell(&address(options.rows, 3)).computed;

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
            "first_direct_value",
            json!(calc_value_display_text(&first_direct_value)),
        ),
        (
            "first_parenthesized_value",
            json!(calc_value_display_text(&first_parenthesized_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_direct_value",
            json!(calc_value_display_text(&middle_direct_value)),
        ),
        (
            "middle_parenthesized_value",
            json!(calc_value_display_text(&middle_parenthesized_value)),
        ),
        (
            "last_direct_value",
            json!(calc_value_display_text(&last_direct_value)),
        ),
        (
            "last_parenthesized_value",
            json!(calc_value_display_text(&last_parenthesized_value)),
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
            "direct-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "direct-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "direct-r1c1-1M prepares two direct scalar R1C1 templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 2
                && recalc.compiled_formula_plans_cached == 2
        ),
        register_assertion(
            "P-14",
            "direct-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 2
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(2)
                && recalc.compiled_formula_plan_cache_misses == 2
        ),
        register_assertion(
            "P-19",
            "unchanged direct-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "direct-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 2
        ),
        register_assertion(
            "GRID-DIRECT-R1C1-1M",
            "direct-r1c1-1M publishes dense output for direct scalar and parenthesized direct scalar R1C1 formulas",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 2
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 3
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.sparse_computed_cells() == 0
                && first_direct_value == expected_value(1)
                && first_parenthesized_value == expected_value(1)
                && middle_direct_value == expected_value(middle_row)
                && middle_parenthesized_value == expected_value(middle_row)
                && last_direct_value == expected_value(options.rows)
                && last_parenthesized_value == expected_value(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn unary_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "unary-r1c1-1m requires at least 4 columns".to_string(),
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
    sheet.put_dense_number_region_with(dense_rect, |address| f64::from(address.row) * 10.0)?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            2,
            options.rows,
            2,
            bounds,
        )?,
        GridFormulaCell::new("=-RC[-1]", "excel.grid.v1:r1c1-template:-RC[-1]")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new("=-(RC[-2]+5)", "excel.grid.v1:r1c1-template:-(RC[-2]+5)")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new("=-RC[-3]*2+1", "excel.grid.v1:r1c1-template:-RC[-3]*2+1")
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
    let direct_value = |row: u32| CalcValue::number(-(f64::from(row) * 10.0));
    let parenthesized_value = |row: u32| CalcValue::number(-(f64::from(row) * 10.0 + 5.0));
    let arithmetic_value = |row: u32| CalcValue::number(-(f64::from(row) * 10.0) * 2.0 + 1.0);
    let middle_row = (options.rows / 2).max(1);
    let first_direct_value = valuation.read_cell(&address(1, 2)).computed;
    let first_parenthesized_value = valuation.read_cell(&address(1, 3)).computed;
    let first_arithmetic_value = valuation.read_cell(&address(1, 4)).computed;
    let middle_direct_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let middle_parenthesized_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_arithmetic_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let last_direct_value = valuation.read_cell(&address(options.rows, 2)).computed;
    let last_parenthesized_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_arithmetic_value = valuation.read_cell(&address(options.rows, 4)).computed;

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
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_direct_value",
            json!(calc_value_display_text(&first_direct_value)),
        ),
        (
            "first_parenthesized_value",
            json!(calc_value_display_text(&first_parenthesized_value)),
        ),
        (
            "first_arithmetic_value",
            json!(calc_value_display_text(&first_arithmetic_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_direct_value",
            json!(calc_value_display_text(&middle_direct_value)),
        ),
        (
            "middle_parenthesized_value",
            json!(calc_value_display_text(&middle_parenthesized_value)),
        ),
        (
            "middle_arithmetic_value",
            json!(calc_value_display_text(&middle_arithmetic_value)),
        ),
        (
            "last_direct_value",
            json!(calc_value_display_text(&last_direct_value)),
        ),
        (
            "last_parenthesized_value",
            json!(calc_value_display_text(&last_parenthesized_value)),
        ),
        (
            "last_arithmetic_value",
            json!(calc_value_display_text(&last_arithmetic_value)),
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
            "unary-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "unary-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "unary-r1c1-1M prepares three unary scalar R1C1 templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 3
                && recalc.compiled_formula_plans_cached == 3
        ),
        register_assertion(
            "P-14",
            "unary-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 3
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(3)
                && recalc.compiled_formula_plan_cache_misses == 3
        ),
        register_assertion(
            "P-19",
            "unchanged unary-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "unary-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 3
        ),
        register_assertion(
            "GRID-UNARY-R1C1-1M",
            "unary-r1c1-1M publishes dense output for direct unary, parenthesized unary, and unary arithmetic R1C1 formulas",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 3
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 4
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.sparse_computed_cells() == 0
                && first_direct_value == direct_value(1)
                && first_parenthesized_value == parenthesized_value(1)
                && first_arithmetic_value == arithmetic_value(1)
                && middle_direct_value == direct_value(middle_row)
                && middle_parenthesized_value == parenthesized_value(middle_row)
                && middle_arithmetic_value == arithmetic_value(middle_row)
                && last_direct_value == direct_value(options.rows)
                && last_parenthesized_value == parenthesized_value(options.rows)
                && last_arithmetic_value == arithmetic_value(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn argument_aggregate_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 8 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "argument-aggregate-r1c1-1m requires at least 8 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| match address.col {
        1 => f64::from(address.row) * 10.0,
        2 => f64::from(address.row),
        _ => unreachable!("dense input region has two columns"),
    })?;
    for (col, source, normal_form) in [
        (
            3,
            "=SUM(RC[-2],RC[-1],5)",
            "excel.grid.v1:r1c1-template:SUM(RC[-2],RC[-1],5)",
        ),
        (
            4,
            "=COUNT(RC[-3],RC[-2],5)",
            "excel.grid.v1:r1c1-template:COUNT(RC[-3],RC[-2],5)",
        ),
        (
            5,
            "=PRODUCT(RC[-4],RC[-3],2)",
            "excel.grid.v1:r1c1-template:PRODUCT(RC[-4],RC[-3],2)",
        ),
        (
            6,
            "=AVERAGE(RC[-5],RC[-4],5)",
            "excel.grid.v1:r1c1-template:AVERAGE(RC[-5],RC[-4],5)",
        ),
        (
            7,
            "=MIN(RC[-6],RC[-5],5)",
            "excel.grid.v1:r1c1-template:MIN(RC[-6],RC[-5],5)",
        ),
        (
            8,
            "=MAX(RC[-7],RC[-6],5)",
            "excel.grid.v1:r1c1-template:MAX(RC[-7],RC[-6],5)",
        ),
    ] {
        sheet.put_repeated_formula_region(
            GridRect::new(
                "book:grid-scale",
                "sheet:grid-scale",
                1,
                col,
                options.rows,
                col,
                bounds,
            )?,
            GridFormulaCell::new(source, normal_form)
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
        )?;
    }

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
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(6);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let sum_value = |row: u32| CalcValue::number(f64::from(row) * 11.0 + 5.0);
    let count_value = |_row: u32| CalcValue::number(3.0);
    let product_value = |row: u32| {
        let row = f64::from(row);
        CalcValue::number(row * 10.0 * row * 2.0)
    };
    let average_value = |row: u32| CalcValue::number((f64::from(row) * 11.0 + 5.0) / 3.0);
    let min_value = |row: u32| CalcValue::number(f64::from(row).min(5.0));
    let max_value = |row: u32| CalcValue::number(f64::from(row) * 10.0);
    let middle_row = (options.rows / 2).max(1);
    let first_sum_value = valuation.read_cell(&address(1, 3)).computed;
    let first_count_value = valuation.read_cell(&address(1, 4)).computed;
    let first_product_value = valuation.read_cell(&address(1, 5)).computed;
    let first_average_value = valuation.read_cell(&address(1, 6)).computed;
    let first_min_value = valuation.read_cell(&address(1, 7)).computed;
    let first_max_value = valuation.read_cell(&address(1, 8)).computed;
    let middle_sum_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_product_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let middle_average_value = valuation.read_cell(&address(middle_row, 6)).computed;
    let last_sum_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_product_value = valuation.read_cell(&address(options.rows, 5)).computed;
    let last_average_value = valuation.read_cell(&address(options.rows, 6)).computed;
    let last_min_value = valuation.read_cell(&address(options.rows, 7)).computed;
    let last_max_value = valuation.read_cell(&address(options.rows, 8)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
        ("formula_columns", json!(6)),
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
            "first_sum_value",
            json!(calc_value_display_text(&first_sum_value)),
        ),
        (
            "first_count_value",
            json!(calc_value_display_text(&first_count_value)),
        ),
        (
            "first_product_value",
            json!(calc_value_display_text(&first_product_value)),
        ),
        (
            "first_average_value",
            json!(calc_value_display_text(&first_average_value)),
        ),
        (
            "first_min_value",
            json!(calc_value_display_text(&first_min_value)),
        ),
        (
            "first_max_value",
            json!(calc_value_display_text(&first_max_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_sum_value",
            json!(calc_value_display_text(&middle_sum_value)),
        ),
        (
            "middle_product_value",
            json!(calc_value_display_text(&middle_product_value)),
        ),
        (
            "middle_average_value",
            json!(calc_value_display_text(&middle_average_value)),
        ),
        (
            "last_sum_value",
            json!(calc_value_display_text(&last_sum_value)),
        ),
        (
            "last_product_value",
            json!(calc_value_display_text(&last_product_value)),
        ),
        (
            "last_average_value",
            json!(calc_value_display_text(&last_average_value)),
        ),
        (
            "last_min_value",
            json!(calc_value_display_text(&last_min_value)),
        ),
        (
            "last_max_value",
            json!(calc_value_display_text(&last_max_value)),
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
            "argument-aggregate-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "argument-aggregate-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "argument-aggregate-r1c1-1M prepares six aggregate argument templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 6
                && recalc.compiled_formula_plans_cached == 6
        ),
        register_assertion(
            "P-14",
            "argument-aggregate-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 6
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(6)
                && recalc.compiled_formula_plan_cache_misses == 6
        ),
        register_assertion(
            "P-19",
            "unchanged argument-aggregate-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "argument-aggregate-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 6
        ),
        register_assertion(
            "GRID-ARGUMENT-AGGREGATE-R1C1-1M",
            "argument-aggregate-r1c1-1M publishes dense output for SUM/COUNT/PRODUCT/AVERAGE/MIN/MAX argument lists",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 6
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 7
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.sparse_computed_cells() == 0
                && first_sum_value == sum_value(1)
                && first_count_value == count_value(1)
                && first_product_value == product_value(1)
                && first_average_value == average_value(1)
                && first_min_value == min_value(1)
                && first_max_value == max_value(1)
                && middle_sum_value == sum_value(middle_row)
                && middle_product_value == product_value(middle_row)
                && middle_average_value == average_value(middle_row)
                && last_sum_value == sum_value(options.rows)
                && last_product_value == product_value(options.rows)
                && last_average_value == average_value(options.rows)
                && last_min_value == min_value(options.rows)
                && last_max_value == max_value(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn math_function_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 5 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "math-function-r1c1-1m requires at least 5 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| match address.col {
        1 if address.row % 2 == 0 => f64::from(address.row),
        1 => -f64::from(address.row),
        2 => {
            let row = f64::from(address.row);
            row * row
        }
        _ => unreachable!("dense input region has two columns"),
    })?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new("=ABS(RC[-2])", "excel.grid.v1:r1c1-template:ABS(RC[-2])")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new("=SQRT(RC[-2])", "excel.grid.v1:r1c1-template:SQRT(RC[-2])")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            5,
            options.rows,
            5,
            bounds,
        )?,
        GridFormulaCell::new(
            "=POWER(ABS(RC[-4]),2)",
            "excel.grid.v1:r1c1-template:POWER(ABS(RC[-4]),2)",
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
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(3);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_abs = |row: u32| CalcValue::number(f64::from(row));
    let expected_sqrt = |row: u32| CalcValue::number(f64::from(row));
    let expected_power = |row: u32| {
        let row = f64::from(row);
        CalcValue::number(row * row)
    };
    let middle_row = (options.rows / 2).max(1);
    let first_abs_value = valuation.read_cell(&address(1, 3)).computed;
    let first_sqrt_value = valuation.read_cell(&address(1, 4)).computed;
    let first_power_value = valuation.read_cell(&address(1, 5)).computed;
    let middle_abs_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_sqrt_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let middle_power_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_abs_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_sqrt_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let last_power_value = valuation.read_cell(&address(options.rows, 5)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
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
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_abs_value",
            json!(calc_value_display_text(&first_abs_value)),
        ),
        (
            "first_sqrt_value",
            json!(calc_value_display_text(&first_sqrt_value)),
        ),
        (
            "first_power_value",
            json!(calc_value_display_text(&first_power_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_abs_value",
            json!(calc_value_display_text(&middle_abs_value)),
        ),
        (
            "middle_sqrt_value",
            json!(calc_value_display_text(&middle_sqrt_value)),
        ),
        (
            "middle_power_value",
            json!(calc_value_display_text(&middle_power_value)),
        ),
        (
            "last_abs_value",
            json!(calc_value_display_text(&last_abs_value)),
        ),
        (
            "last_sqrt_value",
            json!(calc_value_display_text(&last_sqrt_value)),
        ),
        (
            "last_power_value",
            json!(calc_value_display_text(&last_power_value)),
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
            "math-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "math-function-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "math-function-r1c1-1M prepares three scalar math function templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 3
                && recalc.compiled_formula_plans_cached == 3
        ),
        register_assertion(
            "P-14",
            "math-function-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 3
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(3)
                && recalc.compiled_formula_plan_cache_misses == 3
        ),
        register_assertion(
            "P-19",
            "unchanged math-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "math-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 3
        ),
        register_assertion(
            "GRID-MATH-FUNCTION-R1C1-1M",
            "math-function-r1c1-1M publishes dense output for ABS/SQRT/POWER scalar functions",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 3
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 4
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.sparse_computed_cells() == 0
                && first_abs_value == expected_abs(1)
                && first_sqrt_value == expected_sqrt(1)
                && first_power_value == expected_power(1)
                && middle_abs_value == expected_abs(middle_row)
                && middle_sqrt_value == expected_sqrt(middle_row)
                && middle_power_value == expected_power(middle_row)
                && last_abs_value == expected_abs(options.rows)
                && last_sqrt_value == expected_sqrt(options.rows)
                && last_power_value == expected_power(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn mod_function_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 5 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "mod-function-r1c1-1m requires at least 5 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| match address.col {
        1 => f64::from(address.row),
        2 => 7.0,
        _ => unreachable!("dense input region has two columns"),
    })?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new(
            "=MOD(RC[-2],RC[-1])",
            "excel.grid.v1:r1c1-template:MOD(RC[-2],RC[-1])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new(
            "=IF(MOD(RC[-3],2)=0,RC[-3]/2,RC[-3]*3)",
            "excel.grid.v1:r1c1-template:IF(MOD(RC[-3],2)=0,RC[-3]/2,RC[-3]*3)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            5,
            options.rows,
            5,
            bounds,
        )?,
        GridFormulaCell::new(
            "=MOD(POWER(RC[-4],2),RC[-3])",
            "excel.grid.v1:r1c1-template:MOD(POWER(RC[-4],2),RC[-3])",
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
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(3);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_mod = |row: u32| CalcValue::number(f64::from(row % 7));
    let expected_if = |row: u32| {
        if row % 2 == 0 {
            CalcValue::number(f64::from(row) / 2.0)
        } else {
            CalcValue::number(f64::from(row) * 3.0)
        }
    };
    let expected_power_mod = |row: u32| {
        let remainder = row % 7;
        CalcValue::number(f64::from((remainder * remainder) % 7))
    };
    let middle_row = (options.rows / 2).max(1);
    let first_mod_value = valuation.read_cell(&address(1, 3)).computed;
    let first_if_value = valuation.read_cell(&address(1, 4)).computed;
    let first_power_mod_value = valuation.read_cell(&address(1, 5)).computed;
    let middle_mod_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_if_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let middle_power_mod_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_mod_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_if_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let last_power_mod_value = valuation.read_cell(&address(options.rows, 5)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
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
            "first_mod_value",
            json!(calc_value_display_text(&first_mod_value)),
        ),
        (
            "first_if_value",
            json!(calc_value_display_text(&first_if_value)),
        ),
        (
            "first_power_mod_value",
            json!(calc_value_display_text(&first_power_mod_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_mod_value",
            json!(calc_value_display_text(&middle_mod_value)),
        ),
        (
            "middle_if_value",
            json!(calc_value_display_text(&middle_if_value)),
        ),
        (
            "middle_power_mod_value",
            json!(calc_value_display_text(&middle_power_mod_value)),
        ),
        (
            "last_mod_value",
            json!(calc_value_display_text(&last_mod_value)),
        ),
        (
            "last_if_value",
            json!(calc_value_display_text(&last_if_value)),
        ),
        (
            "last_power_mod_value",
            json!(calc_value_display_text(&last_power_mod_value)),
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
            "mod-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "mod-function-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "mod-function-r1c1-1M prepares three MOD/scalar function templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 3
                && recalc.compiled_formula_plans_cached == 3
        ),
        register_assertion(
            "P-14",
            "mod-function-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 3
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(3)
                && recalc.compiled_formula_plan_cache_misses == 3
        ),
        register_assertion(
            "P-19",
            "unchanged mod-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "mod-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 3
        ),
        register_assertion(
            "GRID-MOD-FUNCTION-R1C1-1M",
            "mod-function-r1c1-1M publishes dense numeric output for MOD and MOD-driven IF templates",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 3
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 4
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && first_mod_value == expected_mod(1)
                && first_if_value == expected_if(1)
                && first_power_mod_value == expected_power_mod(1)
                && middle_mod_value == expected_mod(middle_row)
                && middle_if_value == expected_if(middle_row)
                && middle_power_mod_value == expected_power_mod(middle_row)
                && last_mod_value == expected_mod(options.rows)
                && last_if_value == expected_if(options.rows)
                && last_power_mod_value == expected_power_mod(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn rounding_function_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 5 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "rounding-function-r1c1-1m requires at least 5 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| match address.col {
        1 => f64::from(address.row) + 0.5,
        2 => 0.0,
        _ => unreachable!("dense input region has two columns"),
    })?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new(
            "=ROUND(RC[-2],RC[-1])",
            "excel.grid.v1:r1c1-template:ROUND(RC[-2],RC[-1])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new(
            "=ROUNDUP(RC[-3],RC[-2])",
            "excel.grid.v1:r1c1-template:ROUNDUP(RC[-3],RC[-2])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            5,
            options.rows,
            5,
            bounds,
        )?,
        GridFormulaCell::new(
            "=ROUNDDOWN(RC[-4],RC[-3])",
            "excel.grid.v1:r1c1-template:ROUNDDOWN(RC[-4],RC[-3])",
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
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(3);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_round = |row: u32| CalcValue::number(f64::from(row) + 1.0);
    let expected_roundup = expected_round;
    let expected_rounddown = |row: u32| CalcValue::number(f64::from(row));
    let middle_row = (options.rows / 2).max(1);
    let first_round_value = valuation.read_cell(&address(1, 3)).computed;
    let first_roundup_value = valuation.read_cell(&address(1, 4)).computed;
    let first_rounddown_value = valuation.read_cell(&address(1, 5)).computed;
    let middle_round_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_roundup_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let middle_rounddown_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_round_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_roundup_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let last_rounddown_value = valuation.read_cell(&address(options.rows, 5)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
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
            "first_round_value",
            json!(calc_value_display_text(&first_round_value)),
        ),
        (
            "first_roundup_value",
            json!(calc_value_display_text(&first_roundup_value)),
        ),
        (
            "first_rounddown_value",
            json!(calc_value_display_text(&first_rounddown_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_round_value",
            json!(calc_value_display_text(&middle_round_value)),
        ),
        (
            "middle_roundup_value",
            json!(calc_value_display_text(&middle_roundup_value)),
        ),
        (
            "middle_rounddown_value",
            json!(calc_value_display_text(&middle_rounddown_value)),
        ),
        (
            "last_round_value",
            json!(calc_value_display_text(&last_round_value)),
        ),
        (
            "last_roundup_value",
            json!(calc_value_display_text(&last_roundup_value)),
        ),
        (
            "last_rounddown_value",
            json!(calc_value_display_text(&last_rounddown_value)),
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
            "rounding-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "rounding-function-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "rounding-function-r1c1-1M prepares three ROUND-family templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 3
                && recalc.compiled_formula_plans_cached == 3
        ),
        register_assertion(
            "P-14",
            "rounding-function-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 3
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(3)
                && recalc.compiled_formula_plan_cache_misses == 3
        ),
        register_assertion(
            "P-19",
            "unchanged rounding-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "rounding-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 3
        ),
        register_assertion(
            "GRID-ROUNDING-FUNCTION-R1C1-1M",
            "rounding-function-r1c1-1M publishes dense numeric output for ROUND/ROUNDUP/ROUNDDOWN templates",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 3
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 4
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && first_round_value == expected_round(1)
                && first_roundup_value == expected_roundup(1)
                && first_rounddown_value == expected_rounddown(1)
                && middle_round_value == expected_round(middle_row)
                && middle_roundup_value == expected_roundup(middle_row)
                && middle_rounddown_value == expected_rounddown(middle_row)
                && last_round_value == expected_round(options.rows)
                && last_roundup_value == expected_roundup(options.rows)
                && last_rounddown_value == expected_rounddown(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn integer_function_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 5 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "integer-function-r1c1-1m requires at least 5 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| match address.col {
        1 => f64::from(address.row) + 0.9,
        2 => -1.0,
        _ => unreachable!("dense input region has two columns"),
    })?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new("=INT(RC[-2])", "excel.grid.v1:r1c1-template:INT(RC[-2])")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new(
            "=TRUNC(RC[-3])",
            "excel.grid.v1:r1c1-template:TRUNC(RC[-3])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            5,
            options.rows,
            5,
            bounds,
        )?,
        GridFormulaCell::new(
            "=TRUNC(RC[-4],RC[-3])",
            "excel.grid.v1:r1c1-template:TRUNC(RC[-4],RC[-3])",
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
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(3);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_int = |row: u32| CalcValue::number(f64::from(row));
    let expected_trunc = expected_int;
    let expected_trunc_tens = |row: u32| CalcValue::number(f64::from((row / 10) * 10));
    let middle_row = (options.rows / 2).max(1);
    let first_int_value = valuation.read_cell(&address(1, 3)).computed;
    let first_trunc_value = valuation.read_cell(&address(1, 4)).computed;
    let first_trunc_tens_value = valuation.read_cell(&address(1, 5)).computed;
    let middle_int_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_trunc_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let middle_trunc_tens_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_int_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_trunc_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let last_trunc_tens_value = valuation.read_cell(&address(options.rows, 5)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
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
            "first_int_value",
            json!(calc_value_display_text(&first_int_value)),
        ),
        (
            "first_trunc_value",
            json!(calc_value_display_text(&first_trunc_value)),
        ),
        (
            "first_trunc_tens_value",
            json!(calc_value_display_text(&first_trunc_tens_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_int_value",
            json!(calc_value_display_text(&middle_int_value)),
        ),
        (
            "middle_trunc_value",
            json!(calc_value_display_text(&middle_trunc_value)),
        ),
        (
            "middle_trunc_tens_value",
            json!(calc_value_display_text(&middle_trunc_tens_value)),
        ),
        (
            "last_int_value",
            json!(calc_value_display_text(&last_int_value)),
        ),
        (
            "last_trunc_value",
            json!(calc_value_display_text(&last_trunc_value)),
        ),
        (
            "last_trunc_tens_value",
            json!(calc_value_display_text(&last_trunc_tens_value)),
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
            "integer-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "integer-function-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "integer-function-r1c1-1M prepares three INT/TRUNC templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 3
                && recalc.compiled_formula_plans_cached == 3
        ),
        register_assertion(
            "P-14",
            "integer-function-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 3
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(3)
                && recalc.compiled_formula_plan_cache_misses == 3
        ),
        register_assertion(
            "P-19",
            "unchanged integer-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "integer-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 3
        ),
        register_assertion(
            "GRID-INTEGER-FUNCTION-R1C1-1M",
            "integer-function-r1c1-1M publishes dense numeric output for INT and one/two-arg TRUNC templates",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 3
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 4
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && first_int_value == expected_int(1)
                && first_trunc_value == expected_trunc(1)
                && first_trunc_tens_value == expected_trunc_tens(1)
                && middle_int_value == expected_int(middle_row)
                && middle_trunc_value == expected_trunc(middle_row)
                && middle_trunc_tens_value == expected_trunc_tens(middle_row)
                && last_int_value == expected_int(options.rows)
                && last_trunc_value == expected_trunc(options.rows)
                && last_trunc_tens_value == expected_trunc_tens(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn log_function_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 6 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "log-function-r1c1-1m requires at least 6 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| match address.col {
        1 => 1.0,
        2 => 0.0,
        _ => unreachable!("dense input region has two columns"),
    })?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new("=EXP(RC[-1])", "excel.grid.v1:r1c1-template:EXP(RC[-1])")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new("=LN(RC[-3])", "excel.grid.v1:r1c1-template:LN(RC[-3])")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            5,
            options.rows,
            5,
            bounds,
        )?,
        GridFormulaCell::new(
            "=LOG10(RC[-4]*100)",
            "excel.grid.v1:r1c1-template:LOG10(RC[-4]*100)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            6,
            options.rows,
            6,
            bounds,
        )?,
        GridFormulaCell::new(
            "=LOG(RC[-5]*100,10)",
            "excel.grid.v1:r1c1-template:LOG(RC[-5]*100,10)",
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
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(4);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_exp = CalcValue::number(1.0);
    let expected_ln = CalcValue::number(0.0);
    let expected_log10 = CalcValue::number(2.0);
    let expected_log = CalcValue::number(2.0);
    let middle_row = (options.rows / 2).max(1);
    let first_exp_value = valuation.read_cell(&address(1, 3)).computed;
    let first_ln_value = valuation.read_cell(&address(1, 4)).computed;
    let first_log10_value = valuation.read_cell(&address(1, 5)).computed;
    let first_log_value = valuation.read_cell(&address(1, 6)).computed;
    let middle_exp_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_ln_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let middle_log10_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let middle_log_value = valuation.read_cell(&address(middle_row, 6)).computed;
    let last_exp_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_ln_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let last_log10_value = valuation.read_cell(&address(options.rows, 5)).computed;
    let last_log_value = valuation.read_cell(&address(options.rows, 6)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
        ("formula_columns", json!(4)),
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
            "first_exp_value",
            json!(calc_value_display_text(&first_exp_value)),
        ),
        (
            "first_ln_value",
            json!(calc_value_display_text(&first_ln_value)),
        ),
        (
            "first_log10_value",
            json!(calc_value_display_text(&first_log10_value)),
        ),
        (
            "first_log_value",
            json!(calc_value_display_text(&first_log_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_exp_value",
            json!(calc_value_display_text(&middle_exp_value)),
        ),
        (
            "middle_ln_value",
            json!(calc_value_display_text(&middle_ln_value)),
        ),
        (
            "middle_log10_value",
            json!(calc_value_display_text(&middle_log10_value)),
        ),
        (
            "middle_log_value",
            json!(calc_value_display_text(&middle_log_value)),
        ),
        (
            "last_exp_value",
            json!(calc_value_display_text(&last_exp_value)),
        ),
        (
            "last_ln_value",
            json!(calc_value_display_text(&last_ln_value)),
        ),
        (
            "last_log10_value",
            json!(calc_value_display_text(&last_log10_value)),
        ),
        (
            "last_log_value",
            json!(calc_value_display_text(&last_log_value)),
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
            "log-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "log-function-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "log-function-r1c1-1M prepares four EXP/LN/LOG templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 4
                && recalc.compiled_formula_plans_cached == 4
        ),
        register_assertion(
            "P-14",
            "log-function-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 4
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(4)
                && recalc.compiled_formula_plan_cache_misses == 4
        ),
        register_assertion(
            "P-19",
            "unchanged log-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "log-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 4
        ),
        register_assertion(
            "GRID-LOG-FUNCTION-R1C1-1M",
            "log-function-r1c1-1M publishes dense numeric output for EXP/LN/LOG10/LOG templates",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 4
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 5
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && first_exp_value == expected_exp
                && first_ln_value == expected_ln
                && first_log10_value == expected_log10
                && first_log_value == expected_log
                && middle_exp_value == expected_exp
                && middle_ln_value == expected_ln
                && middle_log10_value == expected_log10
                && middle_log_value == expected_log
                && last_exp_value == expected_exp
                && last_ln_value == expected_ln
                && last_log10_value == expected_log10
                && last_log_value == expected_log
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn trig_function_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "trig-function-r1c1-1m requires at least 4 columns".to_string(),
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
    sheet.put_dense_number_region_with(dense_rect, |_| 0.0)?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            2,
            options.rows,
            2,
            bounds,
        )?,
        GridFormulaCell::new("=SIN(RC[-1])", "excel.grid.v1:r1c1-template:SIN(RC[-1])")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new("=COS(RC[-2])", "excel.grid.v1:r1c1-template:COS(RC[-2])")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new("=TAN(RC[-3])", "excel.grid.v1:r1c1-template:TAN(RC[-3])")
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
    let expected_sin = CalcValue::number(0.0);
    let expected_cos = CalcValue::number(1.0);
    let expected_tan = CalcValue::number(0.0);
    let middle_row = (options.rows / 2).max(1);
    let first_sin_value = valuation.read_cell(&address(1, 2)).computed;
    let first_cos_value = valuation.read_cell(&address(1, 3)).computed;
    let first_tan_value = valuation.read_cell(&address(1, 4)).computed;
    let middle_sin_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let middle_cos_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_tan_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let last_sin_value = valuation.read_cell(&address(options.rows, 2)).computed;
    let last_cos_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_tan_value = valuation.read_cell(&address(options.rows, 4)).computed;

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
            "first_sin_value",
            json!(calc_value_display_text(&first_sin_value)),
        ),
        (
            "first_cos_value",
            json!(calc_value_display_text(&first_cos_value)),
        ),
        (
            "first_tan_value",
            json!(calc_value_display_text(&first_tan_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_sin_value",
            json!(calc_value_display_text(&middle_sin_value)),
        ),
        (
            "middle_cos_value",
            json!(calc_value_display_text(&middle_cos_value)),
        ),
        (
            "middle_tan_value",
            json!(calc_value_display_text(&middle_tan_value)),
        ),
        (
            "last_sin_value",
            json!(calc_value_display_text(&last_sin_value)),
        ),
        (
            "last_cos_value",
            json!(calc_value_display_text(&last_cos_value)),
        ),
        (
            "last_tan_value",
            json!(calc_value_display_text(&last_tan_value)),
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
            "trig-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "trig-function-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "trig-function-r1c1-1M prepares three SIN/COS/TAN templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 3
                && recalc.compiled_formula_plans_cached == 3
        ),
        register_assertion(
            "P-14",
            "trig-function-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 3
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(3)
                && recalc.compiled_formula_plan_cache_misses == 3
        ),
        register_assertion(
            "P-19",
            "unchanged trig-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "trig-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 3
        ),
        register_assertion(
            "GRID-TRIG-FUNCTION-R1C1-1M",
            "trig-function-r1c1-1M publishes dense numeric output for SIN/COS/TAN templates",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 3
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 4
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && first_sin_value == expected_sin
                && first_cos_value == expected_cos
                && first_tan_value == expected_tan
                && middle_sin_value == expected_sin
                && middle_cos_value == expected_cos
                && middle_tan_value == expected_tan
                && last_sin_value == expected_sin
                && last_cos_value == expected_cos
                && last_tan_value == expected_tan
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn angle_function_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 6 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "angle-function-r1c1-1m requires at least 6 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |_| 0.0)?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new(
            "=RADIANS(RC[-2])",
            "excel.grid.v1:r1c1-template:RADIANS(RC[-2])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new(
            "=DEGREES(RC[-2])",
            "excel.grid.v1:r1c1-template:DEGREES(RC[-2])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            5,
            options.rows,
            5,
            bounds,
        )?,
        GridFormulaCell::new(
            "=SIN(RADIANS(RC[-4]))",
            "excel.grid.v1:r1c1-template:SIN(RADIANS(RC[-4]))",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            6,
            options.rows,
            6,
            bounds,
        )?,
        GridFormulaCell::new("=PI()", "excel.grid.v1:r1c1-template:PI()")
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
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(4);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_radians = CalcValue::number(0.0);
    let expected_degrees = CalcValue::number(0.0);
    let expected_sin_degrees = CalcValue::number(0.0);
    let expected_pi = CalcValue::number(std::f64::consts::PI);
    let middle_row = (options.rows / 2).max(1);
    let first_radians_value = valuation.read_cell(&address(1, 3)).computed;
    let first_degrees_value = valuation.read_cell(&address(1, 4)).computed;
    let first_sin_degrees_value = valuation.read_cell(&address(1, 5)).computed;
    let first_pi_value = valuation.read_cell(&address(1, 6)).computed;
    let middle_radians_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_degrees_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let middle_sin_degrees_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let middle_pi_value = valuation.read_cell(&address(middle_row, 6)).computed;
    let last_radians_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_degrees_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let last_sin_degrees_value = valuation.read_cell(&address(options.rows, 5)).computed;
    let last_pi_value = valuation.read_cell(&address(options.rows, 6)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
        ("formula_columns", json!(4)),
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
            "first_radians_value",
            json!(calc_value_display_text(&first_radians_value)),
        ),
        (
            "first_degrees_value",
            json!(calc_value_display_text(&first_degrees_value)),
        ),
        (
            "first_sin_degrees_value",
            json!(calc_value_display_text(&first_sin_degrees_value)),
        ),
        (
            "first_pi_value",
            json!(calc_value_display_text(&first_pi_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_radians_value",
            json!(calc_value_display_text(&middle_radians_value)),
        ),
        (
            "middle_degrees_value",
            json!(calc_value_display_text(&middle_degrees_value)),
        ),
        (
            "middle_sin_degrees_value",
            json!(calc_value_display_text(&middle_sin_degrees_value)),
        ),
        (
            "middle_pi_value",
            json!(calc_value_display_text(&middle_pi_value)),
        ),
        (
            "last_radians_value",
            json!(calc_value_display_text(&last_radians_value)),
        ),
        (
            "last_degrees_value",
            json!(calc_value_display_text(&last_degrees_value)),
        ),
        (
            "last_sin_degrees_value",
            json!(calc_value_display_text(&last_sin_degrees_value)),
        ),
        (
            "last_pi_value",
            json!(calc_value_display_text(&last_pi_value)),
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
            "angle-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "angle-function-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "angle-function-r1c1-1M prepares four RADIANS/DEGREES/PI templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 4
                && recalc.compiled_formula_plans_cached == 4
        ),
        register_assertion(
            "P-14",
            "angle-function-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 4
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(4)
                && recalc.compiled_formula_plan_cache_misses == 4
        ),
        register_assertion(
            "P-19",
            "unchanged angle-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "angle-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 4
        ),
        register_assertion(
            "GRID-ANGLE-FUNCTION-R1C1-1M",
            "angle-function-r1c1-1M publishes dense numeric output for RADIANS/DEGREES/PI templates",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 4
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 5
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && first_radians_value == expected_radians
                && first_degrees_value == expected_degrees
                && first_sin_degrees_value == expected_sin_degrees
                && first_pi_value == expected_pi
                && middle_radians_value == expected_radians
                && middle_degrees_value == expected_degrees
                && middle_sin_degrees_value == expected_sin_degrees
                && middle_pi_value == expected_pi
                && last_radians_value == expected_radians
                && last_degrees_value == expected_degrees
                && last_sin_degrees_value == expected_sin_degrees
                && last_pi_value == expected_pi
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}
