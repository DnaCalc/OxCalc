//! Grid scale scenarios - text and lookup repeated-R1C1 profiles: the text,
//! INDEX, MATCH, and VLOOKUP function families. Each runs a profile and
//! asserts its P-register counters. Invoked by the scale dispatch; shares
//! harness helpers via `use super::*`.

use super::*;

pub(super) fn text_function_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 5 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "text-function-r1c1-1m requires at least 5 columns".to_string(),
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
    sheet.put_dense_literal_region_with(dense_rect, |_address| {
        CalcValue::text(ExcelText::from_interop_assignment("RowGrid"))
    })?;
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
        GridFormulaCell::new("=LEN(RC[-1])", "excel.grid.v1:r1c1-template:LEN(RC[-1])")
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
        GridFormulaCell::new(
            "=LEFT(RC[-2],3)",
            "excel.grid.v1:r1c1-template:LEFT(RC[-2],3)",
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
            "=RIGHT(RC[-3],4)",
            "excel.grid.v1:r1c1-template:RIGHT(RC[-3],4)",
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
            "=CONCAT(RC[-2],RC[-1])",
            "excel.grid.v1:r1c1-template:CONCAT(RC[-2],RC[-1])",
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
    let expected_formula_cells = u64::from(options.rows).saturating_mul(4);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_numeric_cells = u64::from(options.rows);
    let first_input_value = valuation.read_cell(&address(1, 1)).computed;
    let first_len_value = valuation.read_cell(&address(1, 2)).computed;
    let first_left_value = valuation.read_cell(&address(1, 3)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_right_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let last_concat_value = valuation.read_cell(&address(options.rows, 5)).computed;

    let counters = json_object([
        ("dense_columns", json!(1)),
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
            "dense_value_region_bytes",
            json!(byte_report.dense_value_region_bytes),
        ),
        (
            "repeated_formula_region_bytes",
            json!(byte_report.repeated_formula_region_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "dense_bytes_per_cell_micros",
            json!(byte_report.dense_bytes_per_cell_micros()),
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
            "first_len_value",
            json!(calc_value_display_text(&first_len_value)),
        ),
        (
            "first_left_value",
            json!(calc_value_display_text(&first_left_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_right_value",
            json!(calc_value_display_text(&middle_right_value)),
        ),
        (
            "last_concat_value",
            json!(calc_value_display_text(&last_concat_value)),
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
            "text-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "text-function-r1c1-1M keeps uniform dense text, repeated formulas, and blanks within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
        ),
        register_assertion(
            "P-11",
            "text-function-r1c1-1M prepares four R1C1 text templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 4
                && recalc.compiled_formula_plans_cached == 4
        ),
        register_assertion(
            "P-14",
            "text-function-r1c1-1M formula plan cache misses once per template and hits for remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 4
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(4)
                && recalc.compiled_formula_plan_cache_misses == 4
        ),
        register_assertion(
            "P-19",
            "unchanged text-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "text-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 4
        ),
        register_assertion(
            "GRID-TEXT-FUNCTION-R1C1-1M",
            "text-function-r1c1-1M evaluates LEN/LEFT/RIGHT/CONCAT over R1C1 refs as dense output without sparse fallback",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 4
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 5
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_numeric_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_input_value) == "RowGrid"
                && calc_value_display_text(&first_len_value) == "7"
                && calc_value_display_text(&first_left_value) == "Row"
                && calc_value_display_text(&middle_right_value) == "Grid"
                && calc_value_display_text(&last_concat_value) == "RowGrid"
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn index_function_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 6 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "index-function-r1c1-1m requires at least 6 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    sheet.put_dense_number_region_with(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            1,
            options.rows,
            1,
            bounds,
        )?,
        |address| f64::from(address.row) * 10.0,
    )?;
    sheet.put_dense_literal_region_with(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            2,
            options.rows,
            2,
            bounds,
        )?,
        |_address| CalcValue::text(ExcelText::from_interop_assignment("Index")),
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
        GridFormulaCell::new(
            "=INDEX(RC[-2]:RC[-1],1,1)",
            "excel.grid.v1:r1c1-template:INDEX(RC[-2]:RC[-1],1,1)",
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
            "=INDEX(RC[-3]:RC[-2],1,2)",
            "excel.grid.v1:r1c1-template:INDEX(RC[-3]:RC[-2],1,2)",
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
            "=INDEX(R1C1:RC1,ROW(),1)",
            "excel.grid.v1:r1c1-template:INDEX(R1C1:RC1,ROW(),1)",
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
            "=INDEX(RC[-5]:RC[-4],2,1)",
            "excel.grid.v1:r1c1-template:INDEX(RC[-5]:RC[-4],2,1)",
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
    let expected_numeric_cells = u64::from(options.rows).saturating_mul(3);
    let first_numeric_lookup = valuation.read_cell(&address(1, 3)).computed;
    let first_text_lookup = valuation.read_cell(&address(1, 4)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_dynamic_lookup = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_dynamic_lookup = valuation.read_cell(&address(options.rows, 5)).computed;
    let first_ref_error = valuation.read_cell(&address(1, 6)).computed;
    let expected_ref_display = calc_value_display_text(&CalcValue::error(WorksheetErrorCode::Ref));
    let expected_middle_dynamic_display =
        calc_value_display_text(&CalcValue::number(f64::from(middle_row) * 10.0));
    let expected_last_dynamic_display =
        calc_value_display_text(&CalcValue::number(f64::from(options.rows) * 10.0));

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
            "dense_value_region_bytes",
            json!(byte_report.dense_value_region_bytes),
        ),
        (
            "repeated_formula_region_bytes",
            json!(byte_report.repeated_formula_region_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "dense_bytes_per_cell_micros",
            json!(byte_report.dense_bytes_per_cell_micros()),
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
            "first_numeric_lookup_value",
            json!(calc_value_display_text(&first_numeric_lookup)),
        ),
        (
            "first_text_lookup_value",
            json!(calc_value_display_text(&first_text_lookup)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_dynamic_lookup_value",
            json!(calc_value_display_text(&middle_dynamic_lookup)),
        ),
        (
            "last_dynamic_lookup_value",
            json!(calc_value_display_text(&last_dynamic_lookup)),
        ),
        (
            "first_ref_error_value",
            json!(calc_value_display_text(&first_ref_error)),
        ),
        (
            "expected_middle_dynamic_lookup_value",
            json!(expected_middle_dynamic_display.clone()),
        ),
        (
            "expected_last_dynamic_lookup_value",
            json!(expected_last_dynamic_display.clone()),
        ),
        ("expected_ref_error", json!(expected_ref_display.clone())),
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
            "index-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "index-function-r1c1-1M keeps dense values, repeated formulas, and blanks within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
        ),
        register_assertion(
            "P-11",
            "index-function-r1c1-1M prepares four R1C1 INDEX templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 4
                && recalc.compiled_formula_plans_cached == 4
        ),
        register_assertion(
            "P-14",
            "index-function-r1c1-1M formula plan cache misses once per template and hits for remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 4
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(4)
                && recalc.compiled_formula_plan_cache_misses == 4
        ),
        register_assertion(
            "P-19",
            "unchanged index-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "index-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 2
                && partition.repeated_formula_regions == 4
        ),
        register_assertion(
            "GRID-INDEX-FUNCTION-R1C1-1M",
            "index-function-r1c1-1M evaluates INDEX over R1C1 ranges as dense numeric, text, and #REF! output without sparse fallback",
            stats.dense_value_regions == 2
                && stats.repeated_formula_regions == 4
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 6
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_numeric_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_numeric_lookup) == "10"
                && calc_value_display_text(&first_text_lookup) == "Index"
                && calc_value_display_text(&middle_dynamic_lookup)
                    == expected_middle_dynamic_display
                && calc_value_display_text(&last_dynamic_lookup) == expected_last_dynamic_display
                && calc_value_display_text(&first_ref_error) == expected_ref_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn match_function_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 6 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "match-function-r1c1-1m requires at least 6 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    sheet.put_dense_number_region_with(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            1,
            options.rows,
            3,
            bounds,
        )?,
        |address| f64::from(address.row) * 10.0 + f64::from(address.col) - 1.0,
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
            "=MATCH(RC[-2],RC[-3]:RC[-1],0)",
            "excel.grid.v1:r1c1-template:MATCH(RC[-2],RC[-3]:RC[-1],0)",
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
            "=INDEX(RC[-4]:RC[-2],1,MATCH(RC[-3],RC[-4]:RC[-2],0))",
            "excel.grid.v1:r1c1-template:INDEX(RC[-4]:RC[-2],1,MATCH(RC[-3],RC[-4]:RC[-2],0))",
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
            "=MATCH(999999999,RC[-5]:RC[-3],0)",
            "excel.grid.v1:r1c1-template:MATCH(999999999,RC[-5]:RC[-3],0)",
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
    let expected_formula_cells = u64::from(options.rows).saturating_mul(3);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_numeric_cells = u64::from(options.rows).saturating_mul(5);
    let first_match_position = valuation.read_cell(&address(1, 4)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_index_match = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_no_match = valuation.read_cell(&address(options.rows, 6)).computed;
    let expected_middle_index_display =
        calc_value_display_text(&CalcValue::number(f64::from(middle_row) * 10.0 + 1.0));
    let expected_no_match_display =
        calc_value_display_text(&CalcValue::error(WorksheetErrorCode::NA));

    let counters = json_object([
        ("dense_columns", json!(3)),
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
            "dense_value_region_bytes",
            json!(byte_report.dense_value_region_bytes),
        ),
        (
            "repeated_formula_region_bytes",
            json!(byte_report.repeated_formula_region_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "dense_bytes_per_cell_micros",
            json!(byte_report.dense_bytes_per_cell_micros()),
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
            "first_match_position_value",
            json!(calc_value_display_text(&first_match_position)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_index_match_value",
            json!(calc_value_display_text(&middle_index_match)),
        ),
        (
            "last_no_match_value",
            json!(calc_value_display_text(&last_no_match)),
        ),
        (
            "expected_middle_index_match_value",
            json!(expected_middle_index_display.clone()),
        ),
        (
            "expected_no_match",
            json!(expected_no_match_display.clone()),
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
            "match-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "match-function-r1c1-1M keeps dense values, repeated formulas, and blanks within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
        ),
        register_assertion(
            "P-11",
            "match-function-r1c1-1M prepares three R1C1 MATCH/INDEX templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 3
                && recalc.compiled_formula_plans_cached == 3
        ),
        register_assertion(
            "P-14",
            "match-function-r1c1-1M formula plan cache misses once per template and hits for remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 3
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(3)
                && recalc.compiled_formula_plan_cache_misses == 3
        ),
        register_assertion(
            "P-19",
            "unchanged match-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "match-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 3
        ),
        register_assertion(
            "GRID-MATCH-FUNCTION-R1C1-1M",
            "match-function-r1c1-1M evaluates exact MATCH and nested INDEX/MATCH over R1C1 ranges as dense numeric and #N/A output without sparse fallback",
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
                && calc_value_display_text(&first_match_position) == "2"
                && calc_value_display_text(&middle_index_match) == expected_middle_index_display
                && calc_value_display_text(&last_no_match) == expected_no_match_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn vlookup_function_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 7 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "vlookup-function-r1c1-1m requires at least 7 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    sheet.put_dense_number_region_with(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            1,
            options.rows,
            1,
            bounds,
        )?,
        |address| f64::from(address.row) * 10.0,
    )?;
    sheet.put_dense_literal_region_with(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            2,
            options.rows,
            2,
            bounds,
        )?,
        |_address| CalcValue::text(ExcelText::from_interop_assignment("Lookup")),
    )?;
    sheet.put_dense_number_region_with(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        |address| f64::from(address.row) * 100.0,
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
            "=VLOOKUP(RC[-3],RC[-3]:RC[-1],2,FALSE)",
            "excel.grid.v1:r1c1-template:VLOOKUP(RC[-3],RC[-3]:RC[-1],2,FALSE)",
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
            "=VLOOKUP(RC[-4],RC[-4]:RC[-2],3,0)",
            "excel.grid.v1:r1c1-template:VLOOKUP(RC[-4],RC[-4]:RC[-2],3,0)",
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
            "=VLOOKUP(999999999,RC[-5]:RC[-3],2,FALSE)",
            "excel.grid.v1:r1c1-template:VLOOKUP(999999999,RC[-5]:RC[-3],2,FALSE)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            7,
            options.rows,
            7,
            bounds,
        )?,
        GridFormulaCell::new(
            "=VLOOKUP(RC[-6],RC[-6]:RC[-4],4,FALSE)",
            "excel.grid.v1:r1c1-template:VLOOKUP(RC[-6],RC[-6]:RC[-4],4,FALSE)",
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
    let expected_formula_cells = u64::from(options.rows).saturating_mul(4);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_numeric_cells = u64::from(options.rows).saturating_mul(3);
    let first_text_lookup = valuation.read_cell(&address(1, 4)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_numeric_lookup = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_no_match = valuation.read_cell(&address(options.rows, 6)).computed;
    let first_ref_error = valuation.read_cell(&address(1, 7)).computed;
    let expected_middle_numeric_display =
        calc_value_display_text(&CalcValue::number(f64::from(middle_row) * 100.0));
    let expected_no_match_display =
        calc_value_display_text(&CalcValue::error(WorksheetErrorCode::NA));
    let expected_ref_display = calc_value_display_text(&CalcValue::error(WorksheetErrorCode::Ref));

    let counters = json_object([
        ("dense_columns", json!(3)),
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
            "dense_value_region_bytes",
            json!(byte_report.dense_value_region_bytes),
        ),
        (
            "repeated_formula_region_bytes",
            json!(byte_report.repeated_formula_region_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "dense_bytes_per_cell_micros",
            json!(byte_report.dense_bytes_per_cell_micros()),
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
            "first_text_lookup_value",
            json!(calc_value_display_text(&first_text_lookup)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_numeric_lookup_value",
            json!(calc_value_display_text(&middle_numeric_lookup)),
        ),
        (
            "last_no_match_value",
            json!(calc_value_display_text(&last_no_match)),
        ),
        (
            "first_ref_error_value",
            json!(calc_value_display_text(&first_ref_error)),
        ),
        (
            "expected_middle_numeric_lookup_value",
            json!(expected_middle_numeric_display.clone()),
        ),
        (
            "expected_no_match",
            json!(expected_no_match_display.clone()),
        ),
        ("expected_ref_error", json!(expected_ref_display.clone())),
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
            "vlookup-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "vlookup-function-r1c1-1M keeps dense values, repeated formulas, and blanks within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
        ),
        register_assertion(
            "P-11",
            "vlookup-function-r1c1-1M prepares four R1C1 exact VLOOKUP templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 4
                && recalc.compiled_formula_plans_cached == 4
        ),
        register_assertion(
            "P-14",
            "vlookup-function-r1c1-1M formula plan cache misses once per template and hits for remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 4
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(4)
                && recalc.compiled_formula_plan_cache_misses == 4
        ),
        register_assertion(
            "P-19",
            "unchanged vlookup-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "vlookup-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 3
                && partition.repeated_formula_regions == 4
        ),
        register_assertion(
            "GRID-VLOOKUP-FUNCTION-R1C1-1M",
            "vlookup-function-r1c1-1M evaluates exact VLOOKUP over R1C1 ranges as dense text, numeric, #N/A, and #REF! output without sparse fallback",
            stats.dense_value_regions == 3
                && stats.repeated_formula_regions == 4
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 7
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_numeric_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_text_lookup) == "Lookup"
                && calc_value_display_text(&middle_numeric_lookup)
                    == expected_middle_numeric_display
                && calc_value_display_text(&last_no_match) == expected_no_match_display
                && calc_value_display_text(&first_ref_error) == expected_ref_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}
