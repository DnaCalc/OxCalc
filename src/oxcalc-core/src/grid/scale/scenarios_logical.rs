//! Grid scale scenarios - logical and comparison repeated-R1C1 profiles:
//! reference and logical functions, IF / IF-branch / nested-IF / IFERROR, and
//! the comparison / comparison-expression / comparison-IFERROR families. Each
//! runs a profile and asserts its P-register counters. Invoked by the scale
//! dispatch; shares harness helpers via `use super::*`.

use super::*;

pub(super) fn reference_function_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 6 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "reference-function-r1c1-1m requires at least 6 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    for (col, source, normal_form) in [
        (1, "=ROW()", "excel.grid.v1:r1c1-template:ROW()"),
        (2, "=COLUMN()", "excel.grid.v1:r1c1-template:COLUMN()"),
        (3, "=ROW(RC[-2])", "excel.grid.v1:r1c1-template:ROW(RC[-2])"),
        (
            4,
            "=COLUMN(RC[-3])",
            "excel.grid.v1:r1c1-template:COLUMN(RC[-3])",
        ),
        (
            5,
            "=ROWS(R1C1:R3C1)",
            "excel.grid.v1:r1c1-template:ROWS(R1C1:R3C1)",
        ),
        (
            6,
            "=COLUMNS(RC[-5]:RC[-3])",
            "excel.grid.v1:r1c1-template:COLUMNS(RC[-5]:RC[-3])",
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
    let expected_dense_cells = 0_u64;
    let expected_formula_cells = u64::from(options.rows).saturating_mul(6);
    let expected_occupied_cells = expected_formula_cells;
    let middle_row = (options.rows / 2).max(1);
    let first_row_value = valuation.read_cell(&address(1, 1)).computed;
    let first_column_value = valuation.read_cell(&address(1, 2)).computed;
    let first_ref_row_value = valuation.read_cell(&address(1, 3)).computed;
    let first_ref_column_value = valuation.read_cell(&address(1, 4)).computed;
    let first_rows_value = valuation.read_cell(&address(1, 5)).computed;
    let first_columns_value = valuation.read_cell(&address(1, 6)).computed;
    let middle_row_value = valuation.read_cell(&address(middle_row, 1)).computed;
    let middle_ref_row_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let last_row_value = valuation.read_cell(&address(options.rows, 1)).computed;
    let last_ref_row_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_rows_value = valuation.read_cell(&address(options.rows, 5)).computed;
    let last_columns_value = valuation.read_cell(&address(options.rows, 6)).computed;
    let expected_first_row = CalcValue::number(1.0);
    let expected_middle_row = CalcValue::number(f64::from(middle_row));
    let expected_last_row = CalcValue::number(f64::from(options.rows));
    let expected_current_column = CalcValue::number(2.0);
    let expected_reference_column = CalcValue::number(1.0);
    let expected_rows = CalcValue::number(3.0);
    let expected_columns = CalcValue::number(3.0);

    let counters = json_object([
        ("dense_columns", json!(0)),
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
            "first_row_value",
            json!(calc_value_display_text(&first_row_value)),
        ),
        (
            "first_column_value",
            json!(calc_value_display_text(&first_column_value)),
        ),
        (
            "first_ref_row_value",
            json!(calc_value_display_text(&first_ref_row_value)),
        ),
        (
            "first_ref_column_value",
            json!(calc_value_display_text(&first_ref_column_value)),
        ),
        (
            "first_rows_value",
            json!(calc_value_display_text(&first_rows_value)),
        ),
        (
            "first_columns_value",
            json!(calc_value_display_text(&first_columns_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_row_value",
            json!(calc_value_display_text(&middle_row_value)),
        ),
        (
            "middle_ref_row_value",
            json!(calc_value_display_text(&middle_ref_row_value)),
        ),
        (
            "last_row_value",
            json!(calc_value_display_text(&last_row_value)),
        ),
        (
            "last_ref_row_value",
            json!(calc_value_display_text(&last_ref_row_value)),
        ),
        (
            "last_rows_value",
            json!(calc_value_display_text(&last_rows_value)),
        ),
        (
            "last_columns_value",
            json!(calc_value_display_text(&last_columns_value)),
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
            "reference-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "reference-function-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "reference-function-r1c1-1M prepares six ROW/COLUMN reference-function templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 6
                && recalc.compiled_formula_plans_cached == 6
        ),
        register_assertion(
            "P-14",
            "reference-function-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 6
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(6)
                && recalc.compiled_formula_plan_cache_misses == 6
        ),
        register_assertion(
            "P-19",
            "unchanged reference-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "reference-function-r1c1-1M compact repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 0
                && partition.repeated_formula_regions == 6
        ),
        register_assertion(
            "GRID-REFERENCE-FUNCTION-R1C1-1M",
            "reference-function-r1c1-1M publishes dense numeric output for ROW/COLUMN/ROWS/COLUMNS without dereferencing values",
            stats.dense_value_regions == 0
                && stats.repeated_formula_regions == 6
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 6
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.sparse_computed_cells() == 0
                && first_row_value == expected_first_row
                && first_column_value == expected_current_column
                && first_ref_row_value == expected_first_row
                && first_ref_column_value == expected_reference_column
                && first_rows_value == expected_rows
                && first_columns_value == expected_columns
                && middle_row_value == expected_middle_row
                && middle_ref_row_value == expected_middle_row
                && last_row_value == expected_last_row
                && last_ref_row_value == expected_last_row
                && last_rows_value == expected_rows
                && last_columns_value == expected_columns
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn logical_function_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 5 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "logical-function-r1c1-1m requires at least 5 columns".to_string(),
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
        1 if address.row % 2 == 1 => f64::from(address.row),
        1 => -f64::from(address.row),
        2 if address.row % 3 == 0 => f64::from(address.row),
        2 => -f64::from(address.row),
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
            "=AND(RC[-2]>0,RC[-1]>0)",
            "excel.grid.v1:r1c1-template:AND(RC[-2]>0,RC[-1]>0)",
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
            "=OR(RC[-3]>0,RC[-2]>0)",
            "excel.grid.v1:r1c1-template:OR(RC[-3]>0,RC[-2]>0)",
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
            "=NOT(AND(RC[-4]>0,RC[-3]>0))",
            "excel.grid.v1:r1c1-template:NOT(AND(RC[-4]>0,RC[-3]>0))",
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
    let expected_and = |row: u32| CalcValue::logical(row % 2 == 1 && row % 3 == 0);
    let expected_or = |row: u32| CalcValue::logical(row % 2 == 1 || row % 3 == 0);
    let expected_not = |row: u32| CalcValue::logical(!(row % 2 == 1 && row % 3 == 0));
    let middle_row = (options.rows / 2).max(1);
    let true_sample_row = if options.rows >= 3 { 3 } else { 1 };
    let first_and_value = valuation.read_cell(&address(1, 3)).computed;
    let first_or_value = valuation.read_cell(&address(1, 4)).computed;
    let first_not_value = valuation.read_cell(&address(1, 5)).computed;
    let true_sample_and_value = valuation.read_cell(&address(true_sample_row, 3)).computed;
    let true_sample_or_value = valuation.read_cell(&address(true_sample_row, 4)).computed;
    let true_sample_not_value = valuation.read_cell(&address(true_sample_row, 5)).computed;
    let middle_and_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_or_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let middle_not_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_and_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_or_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let last_not_value = valuation.read_cell(&address(options.rows, 5)).computed;

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
            "first_and_value",
            json!(calc_value_display_text(&first_and_value)),
        ),
        (
            "first_or_value",
            json!(calc_value_display_text(&first_or_value)),
        ),
        (
            "first_not_value",
            json!(calc_value_display_text(&first_not_value)),
        ),
        ("true_sample_row", json!(true_sample_row)),
        (
            "true_sample_and_value",
            json!(calc_value_display_text(&true_sample_and_value)),
        ),
        (
            "true_sample_or_value",
            json!(calc_value_display_text(&true_sample_or_value)),
        ),
        (
            "true_sample_not_value",
            json!(calc_value_display_text(&true_sample_not_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_and_value",
            json!(calc_value_display_text(&middle_and_value)),
        ),
        (
            "middle_or_value",
            json!(calc_value_display_text(&middle_or_value)),
        ),
        (
            "middle_not_value",
            json!(calc_value_display_text(&middle_not_value)),
        ),
        (
            "last_and_value",
            json!(calc_value_display_text(&last_and_value)),
        ),
        (
            "last_or_value",
            json!(calc_value_display_text(&last_or_value)),
        ),
        (
            "last_not_value",
            json!(calc_value_display_text(&last_not_value)),
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
            "logical-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "logical-function-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "logical-function-r1c1-1M prepares three logical function templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 3
                && recalc.compiled_formula_plans_cached == 3
        ),
        register_assertion(
            "P-14",
            "logical-function-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 3
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(3)
                && recalc.compiled_formula_plan_cache_misses == 3
        ),
        register_assertion(
            "P-19",
            "unchanged logical-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "logical-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 3
        ),
        register_assertion(
            "GRID-LOGICAL-FUNCTION-R1C1-1M",
            "logical-function-r1c1-1M publishes dense logical output for AND/OR/NOT over R1C1 comparisons",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 3
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 4
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_dense_cells
                && valuation.dense_computed_logical_packed_cells() == expected_formula_cells
                && valuation.sparse_computed_cells() == 0
                && first_and_value == expected_and(1)
                && first_or_value == expected_or(1)
                && first_not_value == expected_not(1)
                && true_sample_and_value == expected_and(true_sample_row)
                && true_sample_or_value == expected_or(true_sample_row)
                && true_sample_not_value == expected_not(true_sample_row)
                && middle_and_value == expected_and(middle_row)
                && middle_or_value == expected_or(middle_row)
                && middle_not_value == expected_not(middle_row)
                && last_and_value == expected_and(options.rows)
                && last_or_value == expected_or(options.rows)
                && last_not_value == expected_not(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn if_logical_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 5 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "if-logical-r1c1-1m requires at least 5 columns".to_string(),
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
        1 if address.row % 2 == 1 => f64::from(address.row),
        1 => -f64::from(address.row),
        2 if address.row % 3 == 0 => f64::from(address.row),
        2 => -f64::from(address.row),
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
            "=IF(AND(RC[-2]>0,RC[-1]>0),RC[-2]+RC[-1],0)",
            "excel.grid.v1:r1c1-template:IF(AND(RC[-2]>0,RC[-1]>0),RC[-2]+RC[-1],0)",
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
            "=IF(OR(RC[-3]>0,RC[-2]>0),RC[-3]-RC[-2],0)",
            "excel.grid.v1:r1c1-template:IF(OR(RC[-3]>0,RC[-2]>0),RC[-3]-RC[-2],0)",
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
            "=IF(NOT(AND(RC[-4]>0,RC[-3]>0)),ABS(RC[-4]),0)",
            "excel.grid.v1:r1c1-template:IF(NOT(AND(RC[-4]>0,RC[-3]>0)),ABS(RC[-4]),0)",
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
    let input_one = |row: u32| {
        if row % 2 == 1 {
            f64::from(row)
        } else {
            -f64::from(row)
        }
    };
    let input_two = |row: u32| {
        if row % 3 == 0 {
            f64::from(row)
        } else {
            -f64::from(row)
        }
    };
    let expected_and_if = |row: u32| {
        if row % 2 == 1 && row % 3 == 0 {
            CalcValue::number(input_one(row) + input_two(row))
        } else {
            CalcValue::number(0.0)
        }
    };
    let expected_or_if = |row: u32| {
        if row % 2 == 1 || row % 3 == 0 {
            CalcValue::number(input_one(row) - input_two(row))
        } else {
            CalcValue::number(0.0)
        }
    };
    let expected_not_if = |row: u32| {
        if !(row % 2 == 1 && row % 3 == 0) {
            CalcValue::number(input_one(row).abs())
        } else {
            CalcValue::number(0.0)
        }
    };
    let middle_row = (options.rows / 2).max(1);
    let true_sample_row = if options.rows >= 3 { 3 } else { 1 };
    let first_and_if_value = valuation.read_cell(&address(1, 3)).computed;
    let first_or_if_value = valuation.read_cell(&address(1, 4)).computed;
    let first_not_if_value = valuation.read_cell(&address(1, 5)).computed;
    let true_sample_and_if_value = valuation.read_cell(&address(true_sample_row, 3)).computed;
    let true_sample_or_if_value = valuation.read_cell(&address(true_sample_row, 4)).computed;
    let true_sample_not_if_value = valuation.read_cell(&address(true_sample_row, 5)).computed;
    let middle_and_if_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_or_if_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let middle_not_if_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_and_if_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_or_if_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let last_not_if_value = valuation.read_cell(&address(options.rows, 5)).computed;

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
            "first_and_if_value",
            json!(calc_value_display_text(&first_and_if_value)),
        ),
        (
            "first_or_if_value",
            json!(calc_value_display_text(&first_or_if_value)),
        ),
        (
            "first_not_if_value",
            json!(calc_value_display_text(&first_not_if_value)),
        ),
        ("true_sample_row", json!(true_sample_row)),
        (
            "true_sample_and_if_value",
            json!(calc_value_display_text(&true_sample_and_if_value)),
        ),
        (
            "true_sample_or_if_value",
            json!(calc_value_display_text(&true_sample_or_if_value)),
        ),
        (
            "true_sample_not_if_value",
            json!(calc_value_display_text(&true_sample_not_if_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_and_if_value",
            json!(calc_value_display_text(&middle_and_if_value)),
        ),
        (
            "middle_or_if_value",
            json!(calc_value_display_text(&middle_or_if_value)),
        ),
        (
            "middle_not_if_value",
            json!(calc_value_display_text(&middle_not_if_value)),
        ),
        (
            "last_and_if_value",
            json!(calc_value_display_text(&last_and_if_value)),
        ),
        (
            "last_or_if_value",
            json!(calc_value_display_text(&last_or_if_value)),
        ),
        (
            "last_not_if_value",
            json!(calc_value_display_text(&last_not_if_value)),
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
            "if-logical-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "if-logical-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "if-logical-r1c1-1M prepares three IF logical-condition templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 3
                && recalc.compiled_formula_plans_cached == 3
        ),
        register_assertion(
            "P-14",
            "if-logical-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 3
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(3)
                && recalc.compiled_formula_plan_cache_misses == 3
        ),
        register_assertion(
            "P-19",
            "unchanged if-logical-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "if-logical-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 3
        ),
        register_assertion(
            "GRID-IF-LOGICAL-R1C1-1M",
            "if-logical-r1c1-1M publishes dense numeric IF output for AND/OR/NOT R1C1 conditions",
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
                && first_and_if_value == expected_and_if(1)
                && first_or_if_value == expected_or_if(1)
                && first_not_if_value == expected_not_if(1)
                && true_sample_and_if_value == expected_and_if(true_sample_row)
                && true_sample_or_if_value == expected_or_if(true_sample_row)
                && true_sample_not_if_value == expected_not_if(true_sample_row)
                && middle_and_if_value == expected_and_if(middle_row)
                && middle_or_if_value == expected_or_if(middle_row)
                && middle_not_if_value == expected_not_if(middle_row)
                && last_and_if_value == expected_and_if(options.rows)
                && last_or_if_value == expected_or_if(options.rows)
                && last_not_if_value == expected_not_if(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn two_left_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "two-left-r1c1-1m requires at least 4 columns".to_string(),
        });
    }
    let formula_cols = 2_u32;
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
        GridFormulaCell::new(
            "=RC[-2]+RC[-1]",
            "excel.grid.v1:r1c1-template:RC[-2]+RC[-1]",
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
    let expected_dense_cells = u64::from(options.rows) * u64::from(dense_cols);
    let expected_formula_cells = u64::from(options.rows) * u64::from(formula_cols);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_formula_value = valuation.read_cell(&address(1, dense_cols + 1)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_formula_value = valuation
        .read_cell(&address(middle_row, options.cols))
        .computed;
    let last_formula_value = valuation
        .read_cell(&address(options.rows, options.cols))
        .computed;
    let expected_first_formula =
        (1_000.0 + f64::from(dense_cols - 1)) + (1_000.0 + f64::from(dense_cols));
    let expected_last_formula = f64::from(options.rows) * 3000.0 + f64::from(dense_cols * 3 - 1);
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
            "two-left-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "two-left-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "two-left-r1c1-1M prepares one R1C1 template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "two-left-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged two-left-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "two-left-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-TWO-LEFT-R1C1-1M",
            "two-left-r1c1-1M publishes dense formula output for RC[-2]+RC[-1]",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_formula_value)
                    == integer_display(expected_first_formula)
                && calc_value_display_text(&last_formula_value) == expected_last_formula_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn absolute_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "absolute-r1c1-1m requires at least 3 columns".to_string(),
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
    sheet.put_dense_number_region_with(dense_rect, |address| {
        f64::from(address.row) * f64::from(address.col)
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        3,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new("=RC[-1]+R1C1", "excel.grid.v1:r1c1-template:RC[-1]+R1C1")
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
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_formula_value = valuation.read_cell(&address(1, 3)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_formula_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let expected_middle_formula_display = integer_display(f64::from(middle_row) * 2.0 + 1.0);
    let expected_last_formula_display = integer_display(f64::from(options.rows) * 2.0 + 1.0);

    let counters = json_object([
        ("dense_columns", json!(2)),
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
            "absolute-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "absolute-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "absolute-r1c1-1M prepares one mixed absolute/relative R1C1 template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "absolute-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged absolute-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "absolute-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-ABSOLUTE-R1C1-1M",
            "absolute-r1c1-1M publishes dense formula output for RC[-1]+R1C1",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_formula_value) == "3"
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

pub(super) fn division_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "division-r1c1-1m requires at least 2 columns".to_string(),
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
        GridFormulaCell::new("=RC[-1]/2", "excel.grid.v1:r1c1-template:RC[-1]/2")
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
    let expected_last_formula_display = integer_display(f64::from(options.rows));

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
            "division-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "division-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "division-r1c1-1M prepares one R1C1 template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "division-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged division-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "division-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-DIVISION-R1C1-1M",
            "division-r1c1-1M publishes dense formula output for RC[-1]/2",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_formula_value) == "1"
                && calc_value_display_text(&last_formula_value) == expected_last_formula_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn decimal_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "decimal-r1c1-1m requires at least 2 columns".to_string(),
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
        GridFormulaCell::new("=RC[-1]*0.5", "excel.grid.v1:r1c1-template:RC[-1]*0.5")
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
    let expected_last_formula_display = integer_display(f64::from(options.rows));

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
            "decimal-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "decimal-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "decimal-r1c1-1M prepares one R1C1 template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "decimal-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged decimal-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "decimal-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-DECIMAL-R1C1-1M",
            "decimal-r1c1-1M publishes dense formula output for RC[-1]*0.5",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_formula_value) == "1"
                && calc_value_display_text(&last_formula_value) == expected_last_formula_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn recursive_binary_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 5 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "recursive-binary-r1c1-1m requires at least 5 columns".to_string(),
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
    sheet.put_dense_number_region_with(dense_rect, |address| match address.col {
        1 => f64::from(address.row),
        2 => f64::from(address.row) * 10.0,
        _ => 2.0,
    })?;
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
            "=RC[-3]+RC[-2]*RC[-1]",
            "excel.grid.v1:r1c1-template:RC[-3]+RC[-2]*RC[-1]",
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
            "=(RC[-4]+RC[-3])*RC[-2]",
            "excel.grid.v1:r1c1-template:(RC[-4]+RC[-3])*RC[-2]",
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
    let precedence_value = |row: u32| CalcValue::number(f64::from(row) * 21.0);
    let parenthesized_value = |row: u32| CalcValue::number(f64::from(row) * 22.0);
    let middle_row = (options.rows / 2).max(1);
    let first_precedence_value = valuation.read_cell(&address(1, 4)).computed;
    let first_parenthesized_value = valuation.read_cell(&address(1, 5)).computed;
    let middle_precedence_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let middle_parenthesized_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_precedence_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let last_parenthesized_value = valuation.read_cell(&address(options.rows, 5)).computed;

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
            "first_precedence_value",
            json!(calc_value_display_text(&first_precedence_value)),
        ),
        (
            "first_parenthesized_value",
            json!(calc_value_display_text(&first_parenthesized_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_precedence_value",
            json!(calc_value_display_text(&middle_precedence_value)),
        ),
        (
            "middle_parenthesized_value",
            json!(calc_value_display_text(&middle_parenthesized_value)),
        ),
        (
            "last_precedence_value",
            json!(calc_value_display_text(&last_precedence_value)),
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
            "recursive-binary-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "recursive-binary-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "recursive-binary-r1c1-1M prepares two recursive R1C1 binary templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 2
                && recalc.compiled_formula_plans_cached == 2
        ),
        register_assertion(
            "P-14",
            "recursive-binary-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 2
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(2)
                && recalc.compiled_formula_plan_cache_misses == 2
        ),
        register_assertion(
            "P-19",
            "unchanged recursive-binary-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "recursive-binary-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 2
        ),
        register_assertion(
            "GRID-RECURSIVE-BINARY-R1C1-1M",
            "recursive-binary-r1c1-1M publishes dense output for precedence and parenthesized arithmetic expressions",
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
                && first_precedence_value == precedence_value(1)
                && first_parenthesized_value == parenthesized_value(1)
                && middle_precedence_value == precedence_value(middle_row)
                && middle_parenthesized_value == parenthesized_value(middle_row)
                && last_precedence_value == precedence_value(options.rows)
                && last_parenthesized_value == parenthesized_value(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn if_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "if-r1c1-1m requires at least 2 columns".to_string(),
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
    sheet.put_dense_number_region_with(dense_rect, |address| {
        if address.row % 2 == 0 {
            -f64::from(address.row)
        } else {
            f64::from(address.row)
        }
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
        GridFormulaCell::new(
            "=IF(RC[-1]>0,RC[-1],0)",
            "excel.grid.v1:r1c1-template:IF(RC[-1]>0,RC[-1],0)",
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
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let middle_row = (options.rows / 2).max(1);
    let positive_tail_row = if options.rows % 2 == 1 {
        options.rows
    } else {
        options.rows.saturating_sub(1).max(1)
    };
    let first_input_value = valuation.read_cell(&address(1, 1)).computed;
    let first_formula_value = valuation.read_cell(&address(1, 2)).computed;
    let middle_input_value = valuation.read_cell(&address(middle_row, 1)).computed;
    let middle_formula_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let positive_tail_formula_value = valuation.read_cell(&address(positive_tail_row, 2)).computed;
    let last_input_value = valuation.read_cell(&address(options.rows, 1)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 2)).computed;

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
            "first_input_value",
            json!(calc_value_display_text(&first_input_value)),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_input_value",
            json!(calc_value_display_text(&middle_input_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        ("positive_tail_row", json!(positive_tail_row)),
        (
            "positive_tail_formula_value",
            json!(calc_value_display_text(&positive_tail_formula_value)),
        ),
        (
            "last_input_value",
            json!(calc_value_display_text(&last_input_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
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
            "if-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "if-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "if-r1c1-1M prepares one R1C1 IF template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "if-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged if-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "if-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-IF-R1C1-1M",
            "if-r1c1-1M publishes dense formula output for IF(RC[-1]>0,RC[-1],0)",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && first_input_value == CalcValue::number(1.0)
                && first_formula_value == CalcValue::number(1.0)
                && positive_tail_formula_value == CalcValue::number(f64::from(positive_tail_row))
                && last_formula_value
                    == if options.rows % 2 == 0 {
                        CalcValue::number(0.0)
                    } else {
                        CalcValue::number(f64::from(options.rows))
                    }
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn if_branch_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "if-branch-r1c1-1m requires at least 2 columns".to_string(),
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
    sheet.put_dense_number_region_with(dense_rect, |address| {
        if address.row % 2 == 0 {
            -f64::from(address.row)
        } else {
            f64::from(address.row)
        }
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
        GridFormulaCell::new(
            "=IF(RC[-1]>0,RC[-1]*2,RC[-1]/2)",
            "excel.grid.v1:r1c1-template:IF(RC[-1]>0,RC[-1]*2,RC[-1]/2)",
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
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_formula_value = |row: u32| {
        let input = if row % 2 == 0 {
            -f64::from(row)
        } else {
            f64::from(row)
        };
        if input > 0.0 {
            CalcValue::number(input * 2.0)
        } else {
            CalcValue::number(input / 2.0)
        }
    };
    let middle_row = (options.rows / 2).max(1);
    let positive_tail_row = if options.rows % 2 == 1 {
        options.rows
    } else {
        options.rows.saturating_sub(1).max(1)
    };
    let first_input_value = valuation.read_cell(&address(1, 1)).computed;
    let first_formula_value = valuation.read_cell(&address(1, 2)).computed;
    let middle_input_value = valuation.read_cell(&address(middle_row, 1)).computed;
    let middle_formula_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let positive_tail_formula_value = valuation.read_cell(&address(positive_tail_row, 2)).computed;
    let last_input_value = valuation.read_cell(&address(options.rows, 1)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 2)).computed;

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
            "first_input_value",
            json!(calc_value_display_text(&first_input_value)),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_input_value",
            json!(calc_value_display_text(&middle_input_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        ("positive_tail_row", json!(positive_tail_row)),
        (
            "positive_tail_formula_value",
            json!(calc_value_display_text(&positive_tail_formula_value)),
        ),
        (
            "last_input_value",
            json!(calc_value_display_text(&last_input_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
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
            "if-branch-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "if-branch-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "if-branch-r1c1-1M prepares one R1C1 IF branch-expression template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "if-branch-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged if-branch-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "if-branch-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-IF-BRANCH-R1C1-1M",
            "if-branch-r1c1-1M publishes dense formula output for IF branches with scalar arithmetic",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.sparse_computed_cells() == 0
                && first_formula_value == expected_formula_value(1)
                && middle_formula_value == expected_formula_value(middle_row)
                && positive_tail_formula_value == expected_formula_value(positive_tail_row)
                && last_formula_value == expected_formula_value(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn nested_if_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "nested-if-r1c1-1m requires at least 3 columns".to_string(),
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
        _ if address.row % 2 == 0 => 1.0,
        _ => -1.0,
    })?;
    let threshold = (options.rows / 2).max(1);
    let formula_text = format!(
        "=IF(RC[-2]>{threshold},IF(RC[-1]>0,RC[-2]*2,RC[-2]*3),IF(RC[-1]>0,RC[-2]+1,RC[-2]-1))"
    );
    let normal_form_key = format!(
        "excel.grid.v1:r1c1-template:IF(RC[-2]>{threshold},IF(RC[-1]>0,RC[-2]*2,RC[-2]*3),IF(RC[-1]>0,RC[-2]+1,RC[-2]-1))"
    );
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
        GridFormulaCell::new(formula_text, normal_form_key)
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
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_formula_value = |row: u32| {
        let primary = f64::from(row);
        let selector = if row % 2 == 0 { 1.0 } else { -1.0 };
        if primary > f64::from(threshold) {
            if selector > 0.0 {
                CalcValue::number(primary * 2.0)
            } else {
                CalcValue::number(primary * 3.0)
            }
        } else if selector > 0.0 {
            CalcValue::number(primary + 1.0)
        } else {
            CalcValue::number(primary - 1.0)
        }
    };
    let middle_row = (options.rows / 2).max(1);
    let first_formula_value = valuation.read_cell(&address(1, 3)).computed;
    let middle_formula_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let after_threshold_row = threshold.saturating_add(1).min(options.rows).max(1);
    let after_threshold_formula_value = valuation
        .read_cell(&address(after_threshold_row, 3))
        .computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 3)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
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
        ("threshold_row", json!(threshold)),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        ("after_threshold_row", json!(after_threshold_row)),
        (
            "after_threshold_formula_value",
            json!(calc_value_display_text(&after_threshold_formula_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
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
            "nested-if-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "nested-if-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "nested-if-r1c1-1M prepares one nested R1C1 IF template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "nested-if-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged nested-if-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "nested-if-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-NESTED-IF-R1C1-1M",
            "nested-if-r1c1-1M publishes dense output for nested scalar IF branches",
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
                && after_threshold_formula_value == expected_formula_value(after_threshold_row)
                && last_formula_value == expected_formula_value(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn iferror_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "iferror-r1c1-1m requires at least 3 columns".to_string(),
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
    sheet.put_dense_number_region_with(dense_rect, |address| {
        if address.col == 1 {
            f64::from(address.row) * 2.0
        } else if address.row % 2 == 0 {
            0.0
        } else {
            2.0
        }
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        3,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=IFERROR(RC[-2]/RC[-1],0)",
            "excel.grid.v1:r1c1-template:IFERROR(RC[-2]/RC[-1],0)",
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
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let middle_row = (options.rows / 2).max(1);
    let positive_tail_row = if options.rows % 2 == 1 {
        options.rows
    } else {
        options.rows.saturating_sub(1).max(1)
    };
    let first_denominator_value = valuation.read_cell(&address(1, 2)).computed;
    let first_formula_value = valuation.read_cell(&address(1, 3)).computed;
    let middle_denominator_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let middle_formula_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let positive_tail_formula_value = valuation.read_cell(&address(positive_tail_row, 3)).computed;
    let last_denominator_value = valuation.read_cell(&address(options.rows, 2)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 3)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
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
            "first_denominator_value",
            json!(calc_value_display_text(&first_denominator_value)),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_denominator_value",
            json!(calc_value_display_text(&middle_denominator_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        ("positive_tail_row", json!(positive_tail_row)),
        (
            "positive_tail_formula_value",
            json!(calc_value_display_text(&positive_tail_formula_value)),
        ),
        (
            "last_denominator_value",
            json!(calc_value_display_text(&last_denominator_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
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
            "iferror-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "iferror-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "iferror-r1c1-1M prepares one R1C1 IFERROR template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "iferror-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged iferror-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "iferror-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-IFERROR-R1C1-1M",
            "iferror-r1c1-1M publishes dense formula output for IFERROR(RC[-2]/RC[-1],0)",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && first_denominator_value == CalcValue::number(2.0)
                && first_formula_value == CalcValue::number(1.0)
                && middle_denominator_value
                    == if middle_row % 2 == 0 {
                        CalcValue::number(0.0)
                    } else {
                        CalcValue::number(2.0)
                    }
                && middle_formula_value
                    == if middle_row % 2 == 0 {
                        CalcValue::number(0.0)
                    } else {
                        CalcValue::number(f64::from(middle_row))
                    }
                && positive_tail_formula_value == CalcValue::number(f64::from(positive_tail_row))
                && last_formula_value
                    == if options.rows % 2 == 0 {
                        CalcValue::number(0.0)
                    } else {
                        CalcValue::number(f64::from(options.rows))
                    }
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn comparison_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "comparison-r1c1-1m requires at least 2 columns".to_string(),
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
    sheet.put_dense_number_region_with(dense_rect, |address| {
        if address.row % 2 == 0 {
            -f64::from(address.row)
        } else {
            f64::from(address.row)
        }
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
        GridFormulaCell::new("=RC[-1]>0", "excel.grid.v1:r1c1-template:RC[-1]>0")
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
    let middle_row = (options.rows / 2).max(1);
    let positive_tail_row = if options.rows % 2 == 1 {
        options.rows
    } else {
        options.rows.saturating_sub(1).max(1)
    };
    let first_input_value = valuation.read_cell(&address(1, 1)).computed;
    let first_formula_value = valuation.read_cell(&address(1, 2)).computed;
    let middle_input_value = valuation.read_cell(&address(middle_row, 1)).computed;
    let middle_formula_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let positive_tail_formula_value = valuation.read_cell(&address(positive_tail_row, 2)).computed;
    let last_input_value = valuation.read_cell(&address(options.rows, 1)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 2)).computed;

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
            "first_input_value",
            json!(calc_value_display_text(&first_input_value)),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_input_value",
            json!(calc_value_display_text(&middle_input_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        ("positive_tail_row", json!(positive_tail_row)),
        (
            "positive_tail_formula_value",
            json!(calc_value_display_text(&positive_tail_formula_value)),
        ),
        (
            "last_input_value",
            json!(calc_value_display_text(&last_input_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
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
            "comparison-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "comparison-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "comparison-r1c1-1M prepares one R1C1 comparison template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "comparison-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged comparison-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "comparison-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-COMPARISON-R1C1-1M",
            "comparison-r1c1-1M publishes dense logical formula output for RC[-1]>0",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_dense_cells
                && valuation.dense_computed_logical_packed_cells() == expected_formula_cells
                && valuation.sparse_computed_cells() == 0
                && first_input_value == CalcValue::number(1.0)
                && first_formula_value == CalcValue::logical(true)
                && middle_formula_value == CalcValue::logical(middle_row % 2 == 1)
                && positive_tail_formula_value == CalcValue::logical(true)
                && last_formula_value == CalcValue::logical(options.rows % 2 == 1)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn comparison_expression_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "comparison-expression-r1c1-1m requires at least 3 columns".to_string(),
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
    sheet.put_dense_number_region_with(dense_rect, |address| {
        if address.col == 1 {
            if address.row % 2 == 0 {
                -f64::from(address.row)
            } else {
                f64::from(address.row)
            }
        } else {
            f64::from(address.row)
        }
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        3,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=RC[-2]*2>RC[-1]+1",
            "excel.grid.v1:r1c1-template:RC[-2]*2>RC[-1]+1",
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
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_formula_value = |row: u32| {
        let left = if row % 2 == 0 {
            -f64::from(row)
        } else {
            f64::from(row)
        };
        CalcValue::logical(left * 2.0 > f64::from(row) + 1.0)
    };
    let middle_row = (options.rows / 2).max(1);
    let positive_tail_row = if options.rows % 2 == 1 {
        options.rows
    } else {
        options.rows.saturating_sub(1).max(1)
    };
    let first_left_input_value = valuation.read_cell(&address(1, 1)).computed;
    let first_right_input_value = valuation.read_cell(&address(1, 2)).computed;
    let first_formula_value = valuation.read_cell(&address(1, 3)).computed;
    let middle_left_input_value = valuation.read_cell(&address(middle_row, 1)).computed;
    let middle_right_input_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let middle_formula_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let positive_tail_formula_value = valuation.read_cell(&address(positive_tail_row, 3)).computed;
    let last_left_input_value = valuation.read_cell(&address(options.rows, 1)).computed;
    let last_right_input_value = valuation.read_cell(&address(options.rows, 2)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 3)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
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
            "computed_dense_logical_packed_cells",
            json!(valuation.dense_computed_logical_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_left_input_value",
            json!(calc_value_display_text(&first_left_input_value)),
        ),
        (
            "first_right_input_value",
            json!(calc_value_display_text(&first_right_input_value)),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_left_input_value",
            json!(calc_value_display_text(&middle_left_input_value)),
        ),
        (
            "middle_right_input_value",
            json!(calc_value_display_text(&middle_right_input_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        ("positive_tail_row", json!(positive_tail_row)),
        (
            "positive_tail_formula_value",
            json!(calc_value_display_text(&positive_tail_formula_value)),
        ),
        (
            "last_left_input_value",
            json!(calc_value_display_text(&last_left_input_value)),
        ),
        (
            "last_right_input_value",
            json!(calc_value_display_text(&last_right_input_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
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
            "comparison-expression-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "comparison-expression-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "comparison-expression-r1c1-1M prepares one scalar-expression comparison template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "comparison-expression-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged comparison-expression-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "comparison-expression-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-COMPARISON-EXPRESSION-R1C1-1M",
            "comparison-expression-r1c1-1M publishes dense logical output for scalar-expression comparison operands",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_dense_cells
                && valuation.dense_computed_logical_packed_cells() == expected_formula_cells
                && valuation.sparse_computed_cells() == 0
                && first_left_input_value == CalcValue::number(1.0)
                && first_right_input_value == CalcValue::number(1.0)
                && first_formula_value == expected_formula_value(1)
                && middle_formula_value == expected_formula_value(middle_row)
                && positive_tail_formula_value == expected_formula_value(positive_tail_row)
                && last_formula_value == expected_formula_value(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn comparison_iferror_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "comparison-iferror-r1c1-1m requires at least 3 columns".to_string(),
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
    sheet.put_dense_number_region_with(dense_rect, |address| {
        if address.col == 1 {
            if address.row % 2 == 0 {
                -f64::from(address.row)
            } else {
                f64::from(address.row)
            }
        } else if address.row % 2 == 0 {
            0.0
        } else {
            1.0
        }
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        3,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=IFERROR(RC[-2]/RC[-1],0)>0",
            "excel.grid.v1:r1c1-template:IFERROR(RC[-2]/RC[-1],0)>0",
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
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_formula_value = |row: u32| CalcValue::logical(row % 2 == 1);
    let middle_row = (options.rows / 2).max(1);
    let positive_tail_row = if options.rows % 2 == 1 {
        options.rows
    } else {
        options.rows.saturating_sub(1).max(1)
    };
    let first_left_input_value = valuation.read_cell(&address(1, 1)).computed;
    let first_right_input_value = valuation.read_cell(&address(1, 2)).computed;
    let first_formula_value = valuation.read_cell(&address(1, 3)).computed;
    let middle_left_input_value = valuation.read_cell(&address(middle_row, 1)).computed;
    let middle_right_input_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let middle_formula_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let positive_tail_formula_value = valuation.read_cell(&address(positive_tail_row, 3)).computed;
    let last_left_input_value = valuation.read_cell(&address(options.rows, 1)).computed;
    let last_right_input_value = valuation.read_cell(&address(options.rows, 2)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 3)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
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
            "computed_dense_logical_packed_cells",
            json!(valuation.dense_computed_logical_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_left_input_value",
            json!(calc_value_display_text(&first_left_input_value)),
        ),
        (
            "first_right_input_value",
            json!(calc_value_display_text(&first_right_input_value)),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_left_input_value",
            json!(calc_value_display_text(&middle_left_input_value)),
        ),
        (
            "middle_right_input_value",
            json!(calc_value_display_text(&middle_right_input_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        ("positive_tail_row", json!(positive_tail_row)),
        (
            "positive_tail_formula_value",
            json!(calc_value_display_text(&positive_tail_formula_value)),
        ),
        (
            "last_left_input_value",
            json!(calc_value_display_text(&last_left_input_value)),
        ),
        (
            "last_right_input_value",
            json!(calc_value_display_text(&last_right_input_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
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
            "comparison-iferror-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "comparison-iferror-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "comparison-iferror-r1c1-1M prepares one nested IFERROR comparison template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "comparison-iferror-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged comparison-iferror-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "comparison-iferror-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-COMPARISON-IFERROR-R1C1-1M",
            "comparison-iferror-r1c1-1M publishes dense logical output for nested IFERROR comparison operands",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_dense_cells
                && valuation.dense_computed_logical_packed_cells() == expected_formula_cells
                && valuation.sparse_computed_cells() == 0
                && first_formula_value == expected_formula_value(1)
                && middle_formula_value == expected_formula_value(middle_row)
                && positive_tail_formula_value == expected_formula_value(positive_tail_row)
                && last_formula_value == expected_formula_value(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}
