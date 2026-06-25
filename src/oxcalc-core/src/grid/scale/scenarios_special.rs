//! Grid scale scenarios - structural churn, streaming/viewport, and spill
//! register profiles: insert storm, COW retention, publication delta, tile
//! stream, viewport, range invalidation/query, sum pyramid, dirty rect, hide
//! storm, aggregate context, and the spill anchor/epoch/filter/sequence/
//! blockage profiles. Each runs a profile and asserts its P-register
//! counters. Invoked by the scale dispatch; shares harness helpers via
//! `use super::*`.

use super::*;

pub(super) fn plan_cache_rounds_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "plan-cache-rounds-1m requires at least 2 columns".to_string(),
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

    let rounds = 3_usize;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let cache_report = sheet.persistent_formula_plan_cache_report(rounds, materialization_limit)?;
    let expected_formula_cells = u64::from(options.rows) * u64::from(formula_cols);
    let expected_total_lookups = expected_formula_cells.saturating_mul(rounds as u64);
    let expected_total_hits = expected_total_lookups.saturating_sub(1);
    let round_reports = cache_report
        .round_reports
        .iter()
        .map(|round| {
            json!({
                "round_index": round.round_index,
                "formula_cells": round.formula_cells,
                "distinct_formula_templates": round.distinct_formula_templates,
                "formula_plan_cache_lookups": round.formula_plan_cache_lookups(),
                "formula_plan_cache_hits": round.formula_plan_cache_hits,
                "formula_plan_cache_misses": round.formula_plan_cache_misses,
                "formula_plan_cache_hit_rate_micros": round.formula_plan_cache_hit_rate_micros(),
                "compiled_formula_plan_cache_hits": round.compiled_formula_plan_cache_hits,
                "compiled_formula_plan_cache_misses": round.compiled_formula_plan_cache_misses,
                "cached_template_count_after_round": round.cached_template_count_after_round,
                "cached_compiled_plan_count_after_round": round.cached_compiled_plan_count_after_round,
            })
        })
        .collect::<Vec<_>>();
    let first_round_hits = cache_report
        .round_reports
        .first()
        .map_or(0, |round| round.formula_plan_cache_hits);
    let second_round_misses = cache_report
        .round_reports
        .get(1)
        .map_or(0, |round| round.formula_plan_cache_misses);
    let third_round_misses = cache_report
        .round_reports
        .get(2)
        .map_or(0, |round| round.formula_plan_cache_misses);
    let first_round_compiled_plan_misses = cache_report
        .round_reports
        .first()
        .map_or(0, |round| round.compiled_formula_plan_cache_misses);
    let later_round_compiled_plan_misses = cache_report
        .round_reports
        .iter()
        .skip(1)
        .map(|round| round.compiled_formula_plan_cache_misses)
        .sum::<u64>();

    let counters = json_object([
        ("rounds", json!(cache_report.rounds)),
        ("dense_columns", json!(dense_cols)),
        ("formula_columns", json!(formula_cols)),
        (
            "formula_cells_per_round",
            json!(cache_report.formula_cells_per_round),
        ),
        (
            "distinct_formula_templates",
            json!(cache_report.distinct_formula_templates),
        ),
        (
            "cached_template_count",
            json!(cache_report.cached_template_count),
        ),
        (
            "cached_compiled_plan_count",
            json!(cache_report.cached_compiled_plan_count),
        ),
        ("first_round_misses", json!(cache_report.first_round_misses)),
        ("later_round_misses", json!(cache_report.later_round_misses)),
        ("total_lookups", json!(cache_report.total_lookups())),
        ("total_hits", json!(cache_report.total_hits)),
        ("total_misses", json!(cache_report.total_misses)),
        (
            "total_compiled_plan_hits",
            json!(cache_report.total_compiled_plan_hits),
        ),
        (
            "total_compiled_plan_misses",
            json!(cache_report.total_compiled_plan_misses),
        ),
        (
            "first_round_compiled_plan_misses",
            json!(first_round_compiled_plan_misses),
        ),
        (
            "later_round_compiled_plan_misses",
            json!(later_round_compiled_plan_misses),
        ),
        ("hit_rate_micros", json!(cache_report.hit_rate_micros())),
        ("first_round_hits", json!(first_round_hits)),
        ("second_round_misses", json!(second_round_misses)),
        ("third_round_misses", json!(third_round_misses)),
        ("round_reports", json!(round_reports)),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-14",
            "persistent formula plan cache misses only during the first dirty recalc round",
            cache_report.p14_persistent_plan_cache_holds()
                && cache_report.formula_cells_per_round == expected_formula_cells
                && cache_report.total_hits == expected_total_hits
                && cache_report.total_misses == 1
        ),
        register_assertion(
            "GRID-PLAN-CACHE-ROUNDS-1M",
            "plan-cache-rounds-1M preserves one cached R1C1 template across three dirty recalc rounds",
            cache_report.rounds == rounds
                && cache_report.distinct_formula_templates == 1
                && cache_report.cached_template_count == 1
                && cache_report.cached_compiled_plan_count == 1
                && cache_report.first_round_misses == 1
                && cache_report.later_round_misses == 0
                && cache_report.total_compiled_plan_misses == 1
                && cache_report.total_compiled_plan_hits == rounds.saturating_sub(1) as u64
                && cache_report.total_lookups() == expected_total_lookups
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn insert_storm_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.rows < 16 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "insert-storm-1m requires at least 16 rows".to_string(),
        });
    }
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "insert-storm-1m requires at least 2 columns".to_string(),
        });
    }

    let formula_cols = if options.cols >= 10 {
        2
    } else {
        (options.cols / 2).max(1)
    };
    let dense_cols = options.cols - formula_cols;
    let occupied_rows = options.rows - 8;
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        occupied_rows,
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
        occupied_rows,
        options.cols,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let initial_stats = sheet.storage_stats();
    let initial_byte_report = sheet.storage_byte_report();
    let edit_rows = insert_storm_rows(occupied_rows);
    let row_edit_pairs = edit_rows.len();
    let mut edit_reports = Vec::with_capacity(edit_rows.len() * 2);
    for row in edit_rows {
        edit_reports.push(sheet.apply_axis_edit(GridAxisEdit::insert_rows(row, 1))?);
        edit_reports.push(sheet.apply_axis_edit(GridAxisEdit::delete_rows(row + 2, 1))?);
    }
    let final_stats = sheet.storage_stats();
    let final_byte_report = sheet.storage_byte_report();
    let storm = InsertStormReport::from_reports(&edit_reports);
    let initial_authored_cells = initial_stats
        .dense_value_cells
        .saturating_add(initial_stats.repeated_formula_cells);
    let final_authored_cells = final_stats
        .dense_value_cells
        .saturating_add(final_stats.repeated_formula_cells);
    let expected_deleted_cells =
        u64::try_from(row_edit_pairs).unwrap_or(u64::MAX) * u64::from(options.cols);
    let naive_cell_rewrite_floor = initial_authored_cells
        .saturating_mul(u64::try_from(edit_reports.len()).unwrap_or(u64::MAX));
    let compact_metadata_touch_ratio_micros = micros_ratio(
        storm.compact_region_metadata_touches,
        naive_cell_rewrite_floor,
    );

    Ok(json!({
        "counters": {
            "occupied_rows_before": occupied_rows,
            "dense_columns": dense_cols,
            "formula_columns": formula_cols,
            "row_edit_pairs": row_edit_pairs,
            "edits_applied": edit_reports.len(),
            "dense_regions_initial": initial_stats.dense_value_regions,
            "repeated_formula_regions_initial": initial_stats.repeated_formula_regions,
            "dense_regions_final": final_stats.dense_value_regions,
            "repeated_formula_regions_final": final_stats.repeated_formula_regions,
            "max_dense_regions_after_edit": storm.max_dense_regions_after,
            "max_repeated_formula_regions_after_edit": storm.max_repeated_formula_regions_after,
            "dense_region_metadata_visits": storm.dense_region_metadata_visits,
            "repeated_formula_region_metadata_visits": storm.repeated_formula_region_metadata_visits,
            "compact_region_metadata_touches": storm.compact_region_metadata_touches,
            "naive_cell_rewrite_floor": naive_cell_rewrite_floor,
            "compact_metadata_touch_ratio_micros": compact_metadata_touch_ratio_micros,
            "dense_value_cells_initial": initial_stats.dense_value_cells,
            "repeated_formula_cells_initial": initial_stats.repeated_formula_cells,
            "dense_value_cells_final": final_stats.dense_value_cells,
            "repeated_formula_cells_final": final_stats.repeated_formula_cells,
            "authored_cells_initial": initial_authored_cells,
            "authored_cells_final": final_authored_cells,
            "expected_deleted_cells": expected_deleted_cells,
            "actual_deleted_cells": initial_authored_cells.saturating_sub(final_authored_cells),
            "sparse_point_cells_final": final_stats.sparse_point_cells,
            "dense_value_regions_dropped": storm.dense_value_regions_dropped,
            "repeated_formula_regions_dropped": storm.repeated_formula_regions_dropped,
            "repeated_formula_segments_transformed": storm.repeated_formula_segments_transformed,
            "repeated_formula_reference_transforms": storm.repeated_formula_reference_transforms,
            "authored_storage_bytes_initial": initial_byte_report.authored_storage_bytes,
            "authored_storage_bytes_final": final_byte_report.authored_storage_bytes,
            "blank_cell_bytes_final": final_byte_report.blank_cell_bytes
        },
        "register_assertions": [
            register_assertion(
                "P-10",
                "insert-storm compact storage remains region-backed and blanks cost zero bytes",
                final_byte_report.p10_dense_value_budget_holds()
                    && final_byte_report.p10_repeated_formula_budget_holds()
                    && final_byte_report.p10_blank_cells_zero_bytes_holds()
                    && final_stats.sparse_point_cells == 0
            ),
            register_assertion(
                "P-17",
                "row insert/delete storm touches compact region metadata rather than rewriting all authored cells",
                storm.compact_region_metadata_touches
                    <= u64::try_from(edit_reports.len()).unwrap_or(u64::MAX).saturating_mul(16)
                    && storm.compact_region_metadata_touches < naive_cell_rewrite_floor
                    && compact_metadata_touch_ratio_micros <= 20_000
            ),
            register_assertion(
                "GRID-INSERT-STORM-1M",
                "insert/delete row storm preserves compact dense and repeated formula regions with only deleted rows removed",
                final_stats.sparse_point_cells == 0
                    && initial_stats.dense_value_regions == 1
                    && initial_stats.repeated_formula_regions == 1
                    && final_stats.dense_value_regions <= edit_reports.len() + 1
                    && final_stats.repeated_formula_regions <= edit_reports.len() + 1
                    && initial_authored_cells.saturating_sub(final_authored_cells) == expected_deleted_cells
            )
        ]
    }))
}

pub(super) fn cow_retention_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.rows < 16 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "cow-retention-1m requires at least 16 rows".to_string(),
        });
    }
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "cow-retention-1m requires at least 2 columns".to_string(),
        });
    }

    let formula_cols = if options.cols >= 10 {
        2
    } else {
        (options.cols / 2).max(1)
    };
    let dense_cols = options.cols - formula_cols;
    let occupied_rows = options.rows - 8;
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        occupied_rows,
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
        occupied_rows,
        options.cols,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let initial_stats = sheet.storage_stats();
    let initial_byte_report = sheet.storage_byte_report();
    let edit_rows = insert_storm_rows(occupied_rows);
    let mut retained_roots = vec![sheet.clone()];
    let mut edit_reports = Vec::with_capacity(edit_rows.len() * 2);
    for row in edit_rows {
        edit_reports.push(sheet.apply_axis_edit(GridAxisEdit::insert_rows(row, 1))?);
        retained_roots.push(sheet.clone());
        edit_reports.push(sheet.apply_axis_edit(GridAxisEdit::delete_rows(row + 2, 1))?);
        retained_roots.push(sheet.clone());
    }
    let final_stats = sheet.storage_stats();
    let final_byte_report = sheet.storage_byte_report();
    let retention = GridOptimizedSheet::cow_retention_report(&retained_roots);
    let storm = InsertStormReport::from_reports(&edit_reports);
    let initial_authored_cells = initial_stats
        .dense_value_cells
        .saturating_add(initial_stats.repeated_formula_cells);
    let final_authored_cells = final_stats
        .dense_value_cells
        .saturating_add(final_stats.repeated_formula_cells);
    let expected_deleted_cells =
        u64::try_from(edit_rows.len()).unwrap_or(u64::MAX) * u64::from(options.cols);
    let naive_cell_rewrite_floor = initial_authored_cells
        .saturating_mul(u64::try_from(edit_reports.len()).unwrap_or(u64::MAX));

    Ok(json!({
        "counters": {
            "occupied_rows_before": occupied_rows,
            "dense_columns": dense_cols,
            "formula_columns": formula_cols,
            "edits_applied": edit_reports.len(),
            "retained_revision_count": retention.retained_revision_count,
            "unique_dense_payloads": retention.unique_dense_payloads,
            "unique_dense_payload_bytes": retention.unique_dense_payload_bytes,
            "dense_region_metadata_bytes_retained": retention.dense_region_metadata_bytes,
            "repeated_formula_region_bytes_retained": retention.repeated_formula_region_bytes,
            "sparse_point_bytes_retained": retention.sparse_point_bytes,
            "sheet_root_metadata_bytes_retained": retention.sheet_root_metadata_bytes,
            "retained_compact_regions": retention.retained_compact_regions,
            "cow_retained_bytes": retention.cow_retained_bytes,
            "naive_full_snapshot_retention_bytes_floor": retention.naive_full_snapshot_retention_bytes_floor,
            "retained_to_naive_ratio_micros": retention.retained_to_naive_ratio_micros,
            "compact_region_metadata_touches": storm.compact_region_metadata_touches,
            "naive_cell_rewrite_floor": naive_cell_rewrite_floor,
            "dense_value_cells_initial": initial_stats.dense_value_cells,
            "repeated_formula_cells_initial": initial_stats.repeated_formula_cells,
            "dense_value_cells_final": final_stats.dense_value_cells,
            "repeated_formula_cells_final": final_stats.repeated_formula_cells,
            "authored_cells_initial": initial_authored_cells,
            "authored_cells_final": final_authored_cells,
            "expected_deleted_cells": expected_deleted_cells,
            "actual_deleted_cells": initial_authored_cells.saturating_sub(final_authored_cells),
            "dense_regions_final": final_stats.dense_value_regions,
            "repeated_formula_regions_final": final_stats.repeated_formula_regions,
            "sparse_point_cells_final": final_stats.sparse_point_cells,
            "authored_storage_bytes_initial": initial_byte_report.authored_storage_bytes,
            "authored_storage_bytes_final": final_byte_report.authored_storage_bytes,
            "blank_cell_bytes_final": final_byte_report.blank_cell_bytes
        },
        "register_assertions": [
            register_assertion(
                "P-21",
                "retained COW roots share dense payload bytes and grow with compact touched regions rather than full snapshots",
                retention.p21_cow_retention_holds()
                    && retention.retained_revision_count == edit_reports.len() + 1
                    && retention.unique_dense_payloads == 1
                    && retention.cow_retained_bytes < retention.naive_full_snapshot_retention_bytes_floor
                    && retention.retained_to_naive_ratio_micros <= 500_000
                    && storm.compact_region_metadata_touches < naive_cell_rewrite_floor
            ),
            register_assertion(
                "GRID-COW-RETENTION-1M",
                "COW retention preserves compact edited roots without sparse materialization or full dense payload duplication",
                final_stats.sparse_point_cells == 0
                    && final_byte_report.p10_blank_cells_zero_bytes_holds()
                    && initial_authored_cells.saturating_sub(final_authored_cells) == expected_deleted_cells
                    && final_stats.dense_value_regions <= edit_reports.len() + 1
                    && final_stats.repeated_formula_regions <= edit_reports.len() + 1
            )
        ]
    }))
}

pub(super) fn publication_delta_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "publication-delta-1m requires at least 2 columns".to_string(),
        });
    }
    let changed_row = (options.rows / 2).max(1);
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let previous_sheet = publication_delta_sheet(options, bounds, None)?;
    let current_sheet = publication_delta_sheet(options, bounds, Some(changed_row))?;
    let (previous_valuation, previous_recalc) =
        previous_sheet.recalculate_mark_all_dirty_compact_with_oxfml(materialization_limit)?;
    let (current_valuation, current_recalc) =
        current_sheet.recalculate_mark_all_dirty_compact_with_oxfml(materialization_limit)?;
    let delta = current_valuation.publication_delta_report_since(&previous_valuation);
    let publication_entries_total = delta.publication_entries_total();
    let publication_entry_ratio_micros = micros_ratio(
        u64::try_from(publication_entries_total).unwrap_or(u64::MAX),
        delta.naive_current_computed_cell_publication_floor,
    );
    let previous_changed_input = previous_valuation
        .read_cell(&address(changed_row, 1))
        .computed;
    let current_changed_input = current_valuation
        .read_cell(&address(changed_row, 1))
        .computed;
    let previous_changed_formula = previous_valuation
        .read_cell(&address(changed_row, 2))
        .computed;
    let current_changed_formula = current_valuation
        .read_cell(&address(changed_row, 2))
        .computed;

    Ok(json!({
        "counters": {
            "changed_row": changed_row,
            "previous_occupied_cells": previous_recalc.occupied_cells,
            "current_occupied_cells": current_recalc.occupied_cells,
            "previous_formula_cells": previous_recalc.formula_cells,
            "current_formula_cells": current_recalc.formula_cells,
            "previous_computed_dense_value_regions": previous_recalc.computed_dense_value_regions,
            "current_computed_dense_value_regions": current_recalc.computed_dense_value_regions,
            "previous_computed_sparse_cells": previous_valuation.sparse_computed_cells(),
            "current_computed_sparse_cells": current_valuation.sparse_computed_cells(),
            "same_grid_identity": delta.same_grid_identity,
            "previous_sparse_cells": delta.previous_sparse_cells,
            "current_sparse_cells": delta.current_sparse_cells,
            "previous_dense_region_entries": delta.previous_dense_region_entries,
            "current_dense_region_entries": delta.current_dense_region_entries,
            "previous_dense_cells": delta.previous_dense_cells,
            "current_dense_cells": delta.current_dense_cells,
            "previous_spill_fact_entries": delta.previous_spill_fact_entries,
            "current_spill_fact_entries": delta.current_spill_fact_entries,
            "sparse_entries_added": delta.sparse_entries_added,
            "sparse_entries_changed": delta.sparse_entries_changed,
            "sparse_entries_removed": delta.sparse_entries_removed,
            "dense_region_entries_added": delta.dense_region_entries_added,
            "dense_region_entries_changed": delta.dense_region_entries_changed,
            "dense_region_entries_removed": delta.dense_region_entries_removed,
            "dense_region_entries_unchanged": delta.dense_region_entries_unchanged,
            "dense_region_cells_changed": delta.dense_region_cells_changed,
            "spill_fact_entries_added": delta.spill_fact_entries_added,
            "spill_fact_entries_changed": delta.spill_fact_entries_changed,
            "spill_fact_entries_removed": delta.spill_fact_entries_removed,
            "publication_entries_total": publication_entries_total,
            "naive_current_computed_cell_publication_floor": delta.naive_current_computed_cell_publication_floor,
            "naive_full_grid_publication_floor": delta.naive_full_grid_publication_floor,
            "publication_entry_ratio_micros": publication_entry_ratio_micros,
            "previous_changed_input": calc_value_display_text(&previous_changed_input),
            "current_changed_input": calc_value_display_text(&current_changed_input),
            "previous_changed_formula": calc_value_display_text(&previous_changed_formula),
            "current_changed_formula": calc_value_display_text(&current_changed_formula)
        },
        "register_assertions": [
            register_assertion(
                "P-22",
                "one dense input edit plus one dependent repeated-formula output publishes compact region entries rather than all cells",
                delta.same_grid_identity
                    && publication_entries_total == 2
                    && delta.dense_region_entries_changed == 2
                    && delta.sparse_entries_added == 0
                    && delta.sparse_entries_changed == 0
                    && delta.sparse_entries_removed == 0
                    && delta.spill_fact_entries_added == 0
                    && delta.spill_fact_entries_changed == 0
                    && delta.spill_fact_entries_removed == 0
                    && u64::try_from(publication_entries_total).unwrap_or(u64::MAX)
                        < delta.naive_current_computed_cell_publication_floor
                    && publication_entry_ratio_micros <= 1_000
            ),
            register_assertion(
                "GRID-PUBLICATION-DELTA-1M",
                "the publication delta lane changes the edited dense input and dependent repeated-formula output while retaining dense computed storage",
                previous_recalc.computed_dense_value_regions == 2
                    && current_recalc.computed_dense_value_regions == 2
                    && previous_valuation.sparse_computed_cells() == 0
                    && current_valuation.sparse_computed_cells() == 0
                    && previous_changed_input != current_changed_input
                    && previous_changed_formula != current_changed_formula
            )
        ]
    }))
}

pub(super) fn publication_delta_sheet(
    options: &GridScaleOptions,
    bounds: ExcelGridBounds,
    changed_row: Option<u32>,
) -> Result<GridOptimizedSheet, GridScaleRunnerError> {
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let input_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_number_region_with(input_rect, |address| {
        let base = f64::from(address.row);
        if changed_row == Some(address.row) {
            base + 10_000.0
        } else {
            base
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
        GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    Ok(sheet)
}

pub(super) fn tile_stream_64k_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    const TILE_ROWS: u32 = 200;
    const TILE_COLS: u32 = 320;
    const UNRELATED_SPARSE_CELLS: u32 = 1_000;
    const MAX_FRAME_BYTES_PER_SUBSCRIBED_CELL: u64 = 64;
    if options.rows < TILE_ROWS.saturating_mul(2) {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "tile-stream-64k requires at least 400 rows".to_string(),
        });
    }
    if options.cols < TILE_COLS {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "tile-stream-64k requires at least 320 columns".to_string(),
        });
    }

    let tile_top = (options.rows / 2).saturating_sub(TILE_ROWS / 2).max(1);
    let tile_bottom = tile_top + TILE_ROWS - 1;
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let tile_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        tile_top,
        1,
        tile_bottom,
        TILE_COLS,
        bounds,
    )?;
    sheet.put_dense_number_region_with(tile_rect.clone(), |address| {
        f64::from(address.row) * 1000.0 + f64::from(address.col)
    })?;
    for offset in 0..UNRELATED_SPARSE_CELLS {
        let row = options.rows.saturating_sub(offset);
        if row < 1 || (tile_top <= row && row <= tile_bottom) {
            continue;
        }
        sheet.set_literal(
            address(row, options.cols),
            CalcValue::number(f64::from(row)),
        )?;
    }

    let materialization_limit = u64::from(TILE_ROWS).saturating_mul(u64::from(TILE_COLS));
    let (valuation, recalc) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml(materialization_limit)?;
    let report = valuation.tile_snapshot_report(tile_rect)?;
    let full_grid_to_tile_cell_ratio_micros =
        micros_ratio(report.subscribed_cell_count, report.full_grid_cell_floor);

    Ok(json!({
        "counters": {
            "tile_rows": TILE_ROWS,
            "tile_cols": TILE_COLS,
            "tile_top": tile_top,
            "tile_bottom": tile_bottom,
            "tile_subscribed_cells": report.subscribed_cell_count,
            "tile_defined_cells": report.defined_cell_count,
            "tile_blank_cells": report.blank_cell_count,
            "dense_value_cells_visited": report.dense_value_cells_visited,
            "sparse_value_cells_visited": report.sparse_value_cells_visited,
            "compact_regions_intersected": report.compact_regions_intersected,
            "estimated_value_payload_bytes": report.estimated_value_payload_bytes,
            "estimated_frame_bytes": report.estimated_frame_bytes,
            "frame_bytes_per_subscribed_cell_micros": report.frame_bytes_per_subscribed_cell_micros(),
            "max_frame_bytes_per_subscribed_cell": MAX_FRAME_BYTES_PER_SUBSCRIBED_CELL,
            "full_grid_cell_floor": report.full_grid_cell_floor,
            "full_grid_dense_numeric_bytes_floor": report.full_grid_dense_numeric_bytes_floor,
            "full_grid_to_tile_cell_ratio_micros": full_grid_to_tile_cell_ratio_micros,
            "unrelated_sparse_cells": valuation.sparse_computed_cells(),
            "computed_dense_value_regions": recalc.computed_dense_value_regions,
            "computed_sparse_cells": valuation.sparse_computed_cells(),
            "occupied_cells": recalc.occupied_cells
        },
        "register_assertions": [
            register_assertion(
                "P-15",
                "tile-stream-64K frame bytes are bounded by subscribed cells and do not scale with grid capacity or unrelated sparse changes",
                report.p15_tile_streaming_holds(MAX_FRAME_BYTES_PER_SUBSCRIBED_CELL)
                    && report.subscribed_cell_count == u64::from(TILE_ROWS).saturating_mul(u64::from(TILE_COLS))
                    && report.estimated_frame_bytes < report.full_grid_dense_numeric_bytes_floor
                    && valuation.sparse_computed_cells() == usize::try_from(UNRELATED_SPARSE_CELLS).unwrap_or(usize::MAX)
                    && report.sparse_value_cells_visited == 0
            ),
            register_assertion(
                "GRID-TILE-STREAM-64K",
                "a 320x200 tile over a large grid visits the intersecting dense tile region and no unrelated sparse cells",
                report.defined_cell_count == usize::try_from(report.subscribed_cell_count).unwrap_or(usize::MAX)
                    && report.blank_cell_count == 0
                    && report.dense_value_cells_visited == report.subscribed_cell_count
                    && report.compact_regions_intersected == 1
                    && recalc.computed_dense_value_regions == 1
            )
        ]
    }))
}

pub(super) fn viewport_64k_of_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    const VISIBLE_ROWS: u32 = 64_000;
    const DENSE_COLS: u32 = 8;
    const FORMULA_COLS: u32 = 2;
    const VISIBLE_COL: u32 = DENSE_COLS + FORMULA_COLS;
    if options.rows < VISIBLE_ROWS {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "viewport-64k-of-1m requires at least 64,000 rows".to_string(),
        });
    }
    if options.cols < VISIBLE_COL {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "viewport-64k-of-1m requires at least 10 columns".to_string(),
        });
    }

    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        DENSE_COLS,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        f64::from(address.row) * 1000.0 + f64::from(address.col)
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        DENSE_COLS + 1,
        options.rows,
        VISIBLE_COL,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let visible_top = (options.rows / 2).saturating_sub(VISIBLE_ROWS / 2).max(1);
    let visible_bottom = visible_top + VISIBLE_ROWS - 1;
    let visible_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        visible_top,
        VISIBLE_COL,
        visible_bottom,
        VISIBLE_COL,
        bounds,
    )?;
    let materialization_limit = u64::from(VISIBLE_ROWS).saturating_mul(3);
    let (valuation, visible_report) = sheet
        .recalculate_visible_rect_compact_with_oxfml(visible_rect.clone(), materialization_limit)?;
    let snapshot = valuation.tile_snapshot_report(visible_rect.clone())?;

    let middle_row = visible_top + (VISIBLE_ROWS / 2);
    let top_value = valuation
        .read_cell(&address(visible_top, VISIBLE_COL))
        .computed;
    let middle_value = valuation
        .read_cell(&address(middle_row, VISIBLE_COL))
        .computed;
    let bottom_value = valuation
        .read_cell(&address(visible_bottom, VISIBLE_COL))
        .computed;
    let expected_bottom_value = (f64::from(visible_bottom) * 1000.0 + f64::from(DENSE_COLS)) * 4.0;
    let expected_bottom_display = integer_display(expected_bottom_value);

    Ok(json!({
        "counters": {
            "visible_rows": VISIBLE_ROWS,
            "visible_cols": 1,
            "visible_top": visible_top,
            "visible_bottom": visible_bottom,
            "visible_left": VISIBLE_COL,
            "visible_right": VISIBLE_COL,
            "visible_cell_count": visible_report.visible_cell_count,
            "visible_upstream_top": visible_report.upstream_rect.top_row,
            "visible_upstream_bottom": visible_report.upstream_rect.bottom_row,
            "visible_upstream_left": visible_report.upstream_rect.left_col,
            "visible_upstream_right": visible_report.upstream_rect.right_col,
            "visible_upstream_cell_count": visible_report.visible_upstream_cell_count,
            "cells_evaluated_before_visible_complete": visible_report.cells_evaluated_before_visible_complete,
            "formula_evaluations_before_visible_complete": visible_report.formula_evaluations_before_visible_complete,
            "dense_value_cells_projected": visible_report.dense_value_cells_projected,
            "repeated_formula_cells_projected": visible_report.repeated_formula_cells_projected,
            "sparse_point_cells_projected": visible_report.sparse_point_cells_projected,
            "computed_dense_value_regions": visible_report.computed_dense_value_regions,
            "computed_sparse_cells": visible_report.computed_sparse_cells,
            "full_recalc_occupied_cell_floor": visible_report.full_recalc_occupied_cell_floor,
            "full_grid_cell_floor": visible_report.full_grid_cell_floor,
            "visible_eval_to_full_occupied_ratio_micros": visible_report.evaluated_to_full_occupied_ratio_micros(),
            "upstream_to_full_occupied_ratio_micros": micros_ratio(visible_report.visible_upstream_cell_count, visible_report.full_recalc_occupied_cell_floor),
            "snapshot_subscribed_cells": snapshot.subscribed_cell_count,
            "snapshot_defined_cells": snapshot.defined_cell_count,
            "snapshot_dense_value_cells_visited": snapshot.dense_value_cells_visited,
            "snapshot_sparse_value_cells_visited": snapshot.sparse_value_cells_visited,
            "snapshot_estimated_frame_bytes": snapshot.estimated_frame_bytes,
            "top_visible_value": calc_value_display_text(&top_value),
            "middle_visible_value": calc_value_display_text(&middle_value),
            "bottom_visible_value": calc_value_display_text(&bottom_value),
            "expected_bottom_visible_value": expected_bottom_display.clone()
        },
        "register_assertions": [
            register_assertion(
                "P-16",
                "viewport-64K visible-first recalc evaluates only the visible same-row upstream cone before the viewport is clean",
                visible_report.p16_visible_first_holds()
                    && visible_report.visible_cell_count == u64::from(VISIBLE_ROWS)
                    && visible_report.visible_upstream_cell_count == u64::from(VISIBLE_ROWS).saturating_mul(3)
                    && visible_report.cells_evaluated_before_visible_complete == visible_report.visible_upstream_cell_count
                    && visible_report.cells_evaluated_before_visible_complete < visible_report.full_recalc_occupied_cell_floor
                    && visible_report.computed_sparse_cells == 0
            ),
            register_assertion(
                "GRID-VIEWPORT-64K",
                "viewport-64K produces the visible formula column from compact dense and repeated-R1C1 regions",
                visible_report.dense_value_cells_projected == u64::from(VISIBLE_ROWS)
                    && visible_report.repeated_formula_cells_projected == u64::from(VISIBLE_ROWS).saturating_mul(2)
                    && visible_report.formula_evaluations_before_visible_complete == u64::from(VISIBLE_ROWS).saturating_mul(2)
                    && snapshot.subscribed_cell_count == u64::from(VISIBLE_ROWS)
                    && snapshot.defined_cell_count == usize::try_from(VISIBLE_ROWS).unwrap_or(usize::MAX)
                    && snapshot.dense_value_cells_visited == u64::from(VISIBLE_ROWS)
                    && snapshot.sparse_value_cells_visited == 0
                    && calc_value_display_text(&bottom_value) == expected_bottom_display
            )
        ]
    }))
}

pub(super) fn range_invalidation_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    const SCALARIZATION_LIMIT: u64 = 1_024;
    if u64::from(options.rows) <= SCALARIZATION_LIMIT {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "range-invalidation-1m requires rows above the scalarization limit".to_string(),
        });
    }
    if options.cols < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "range-invalidation-1m requires at least 3 columns".to_string(),
        });
    }

    let range = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    let dependent = address(1, 2);
    let downstream = address(1, 3);
    let seed = address((options.rows / 2).max(1), 1);
    let mut invalidation =
        GridInvalidationRef::with_scalarization_limit(bounds, SCALARIZATION_LIMIT);
    let installed_range_scalar_edges = invalidation
        .set_cell_dependencies(dependent.clone(), [GridDependency::Range(range.clone())])?;
    invalidation.set_cell_dependencies(
        downstream.clone(),
        [GridDependency::Cell(dependent.clone())],
    )?;
    let dirty_closure = invalidation.dirty_closure([seed.clone()]);
    let compressed_range_dependencies_for_dependent = invalidation
        .compressed_range_dependencies_for(&dependent)
        .len();
    let expanded_scalar_edge_floor = range.cell_count();
    let compressed_range_edges = invalidation.compressed_range_edge_count();
    let scalar_edges = invalidation.scalar_edge_count();
    let semantic_dependencies = invalidation.semantic_dependencies_for(&dependent).len();
    let compressed_support_edge_ratio_micros = micros_ratio(
        u64::try_from(compressed_range_edges).unwrap_or(u64::MAX),
        expanded_scalar_edge_floor,
    );

    Ok(json!({
        "counters": {
            "dependency_rows": options.rows,
            "dependency_cols": 1,
            "scalarization_limit": SCALARIZATION_LIMIT,
            "expanded_scalar_edge_floor": expanded_scalar_edge_floor,
            "installed_range_scalar_edges": installed_range_scalar_edges,
            "scalar_edges_total": scalar_edges,
            "compressed_range_edges": compressed_range_edges,
            "compressed_range_dependencies_for_dependent": compressed_range_dependencies_for_dependent,
            "semantic_dependencies_for_dependent": semantic_dependencies,
            "compressed_support_edge_ratio_micros": compressed_support_edge_ratio_micros,
            "dirty_seed_row": seed.row,
            "dirty_closure_size": dirty_closure.len(),
            "dirty_closure_contains_dependent": dirty_closure.contains(&dependent),
            "dirty_closure_contains_downstream": dirty_closure.contains(&downstream)
        },
        "register_assertions": [
            register_assertion(
                "P-13",
                "finite range invalidation keeps one compressed reverse edge instead of expanding a 1M-row range",
                compressed_range_edges == 1
                    && installed_range_scalar_edges == 0
                    && scalar_edges == 1
                    && compressed_range_dependencies_for_dependent == 1
                    && expanded_scalar_edge_floor == u64::from(options.rows)
                    && compressed_support_edge_ratio_micros < 1_000_000
                    && dirty_closure.contains(&dependent)
                    && dirty_closure.contains(&downstream)
            ),
            register_assertion(
                "GRID-RANGE-INVALIDATION-1M",
                "a cell edit inside a compressed 1M-row range reaches the range formula and its downstream dependent",
                dirty_closure.len() == 3
                    && dirty_closure.contains(&seed)
                    && dirty_closure.contains(&dependent)
                    && dirty_closure.contains(&downstream)
            )
        ]
    }))
}

pub(super) fn range_query_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    const SCALARIZATION_LIMIT: u64 = 64;
    const RANGE_COUNT: u32 = 1_000;
    if options.rows < RANGE_COUNT {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "range-query-1m requires at least 1000 rows".to_string(),
        });
    }
    if options.cols < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "range-query-1m requires at least 3 columns".to_string(),
        });
    }

    let range_height = options.rows / RANGE_COUNT;
    let covered_rows = range_height * RANGE_COUNT;
    let selected_range_index = RANGE_COUNT / 2;
    let selected_start = selected_range_index * range_height + 1;
    let selected_seed_row = selected_start + (range_height / 2).min(range_height - 1);
    let selected_dependent = address(selected_range_index + 1, 2);
    let downstream = address(1, 3);
    let seed = address(selected_seed_row, 1);

    let mut invalidation =
        GridInvalidationRef::with_scalarization_limit(bounds, SCALARIZATION_LIMIT);
    for range_index in 0..RANGE_COUNT {
        let start_row = range_index * range_height + 1;
        let end_row = start_row + range_height - 1;
        let range = GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            start_row,
            1,
            end_row,
            1,
            bounds,
        )?;
        invalidation
            .set_cell_dependencies(address(range_index + 1, 2), [GridDependency::Range(range)])?;
    }
    invalidation.set_cell_dependencies(
        downstream.clone(),
        [GridDependency::Cell(selected_dependent.clone())],
    )?;

    let query = invalidation.compressed_range_query_report(seed.clone())?;
    let dirty_closure = invalidation.dirty_closure([seed.clone()]);
    let total_compressed_range_edges = invalidation.compressed_range_edge_count();
    let naive_candidate_floor = total_compressed_range_edges;
    let indexed_candidate_ratio_micros = micros_ratio(
        u64::try_from(query.indexed_candidate_count).unwrap_or(u64::MAX),
        u64::try_from(naive_candidate_floor).unwrap_or(u64::MAX),
    );

    Ok(json!({
        "counters": {
            "range_count": RANGE_COUNT,
            "range_height": range_height,
            "covered_rows": covered_rows,
            "scalarization_limit": SCALARIZATION_LIMIT,
            "total_compressed_range_edges": total_compressed_range_edges,
            "naive_candidate_floor": naive_candidate_floor,
            "indexed_candidate_count": query.indexed_candidate_count,
            "matched_dependent_count": query.matched_dependent_count,
            "indexed_candidate_ratio_micros": indexed_candidate_ratio_micros,
            "dirty_seed_row": seed.row,
            "dirty_closure_size": dirty_closure.len(),
            "dirty_closure_contains_selected_dependent": dirty_closure.contains(&selected_dependent),
            "dirty_closure_contains_downstream": dirty_closure.contains(&downstream)
        },
        "register_assertions": [
            register_assertion(
                "P-12",
                "compressed range invalidation uses the block interval index instead of scanning every range edge",
                total_compressed_range_edges == usize::try_from(RANGE_COUNT).unwrap_or(usize::MAX)
                    && query.indexed_candidate_count < naive_candidate_floor
                    && query.indexed_candidate_count <= 4
                    && query.matched_dependent_count == 1
                    && indexed_candidate_ratio_micros <= 4_000
                    && dirty_closure.contains(&selected_dependent)
                    && dirty_closure.contains(&downstream)
            ),
            register_assertion(
                "GRID-RANGE-QUERY-1M",
                "a seed inside one of 1000 compressed ranges dirties only the matching range chain",
                dirty_closure.len() == 3
                    && dirty_closure.contains(&seed)
                    && dirty_closure.contains(&selected_dependent)
                    && dirty_closure.contains(&downstream)
            )
        ]
    }))
}

pub(super) fn sum_pyramid_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    const SCALARIZATION_LIMIT: u64 = 4;
    const LEVEL1_COUNT: u32 = 1_000;
    const GROUP_SIZE: u32 = 10;
    const LEVEL2_COUNT: u32 = LEVEL1_COUNT / GROUP_SIZE;
    const LEVEL3_COUNT: u32 = LEVEL2_COUNT / GROUP_SIZE;
    if options.rows < LEVEL1_COUNT * (u32::try_from(SCALARIZATION_LIMIT).unwrap() + 1) {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "sum-pyramid-1m requires enough rows for compressed leaf ranges".to_string(),
        });
    }
    if options.cols < 6 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "sum-pyramid-1m requires at least 6 columns".to_string(),
        });
    }

    let level1_height = options.rows / LEVEL1_COUNT;
    let covered_rows = level1_height * LEVEL1_COUNT;
    let mut invalidation =
        GridInvalidationRef::with_scalarization_limit(bounds, SCALARIZATION_LIMIT);
    let mut expanded_range_edge_floor = 0_u64;
    let mut installed_range_scalar_edges = 0_usize;

    for level1_index in 0..LEVEL1_COUNT {
        let start_row = sum_pyramid_row_for_level1(level1_index, level1_height);
        let end_row = start_row + level1_height - 1;
        let range = GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            start_row,
            1,
            end_row,
            1,
            bounds,
        )?;
        expanded_range_edge_floor = expanded_range_edge_floor.saturating_add(range.cell_count());
        installed_range_scalar_edges = installed_range_scalar_edges.saturating_add(
            invalidation
                .set_cell_dependencies(address(start_row, 2), [GridDependency::Range(range)])?,
        );
    }

    for level2_index in 0..LEVEL2_COUNT {
        let first_level1_index = level2_index * GROUP_SIZE;
        let last_level1_index = first_level1_index + GROUP_SIZE - 1;
        let start_row = sum_pyramid_row_for_level1(first_level1_index, level1_height);
        let end_row = sum_pyramid_row_for_level1(last_level1_index, level1_height);
        let range = GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            start_row,
            2,
            end_row,
            2,
            bounds,
        )?;
        expanded_range_edge_floor = expanded_range_edge_floor.saturating_add(range.cell_count());
        installed_range_scalar_edges = installed_range_scalar_edges.saturating_add(
            invalidation
                .set_cell_dependencies(address(start_row, 3), [GridDependency::Range(range)])?,
        );
    }

    for level3_index in 0..LEVEL3_COUNT {
        let first_level2_index = level3_index * GROUP_SIZE;
        let last_level2_index = first_level2_index + GROUP_SIZE - 1;
        let start_row = sum_pyramid_row_for_level1(first_level2_index * GROUP_SIZE, level1_height);
        let end_row = sum_pyramid_row_for_level1(last_level2_index * GROUP_SIZE, level1_height);
        let range = GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            start_row,
            3,
            end_row,
            3,
            bounds,
        )?;
        expanded_range_edge_floor = expanded_range_edge_floor.saturating_add(range.cell_count());
        installed_range_scalar_edges = installed_range_scalar_edges.saturating_add(
            invalidation
                .set_cell_dependencies(address(start_row, 4), [GridDependency::Range(range)])?,
        );
    }

    let final_dependent = address(1, 5);
    let final_range_end_row =
        sum_pyramid_row_for_level1((LEVEL3_COUNT - 1) * GROUP_SIZE * GROUP_SIZE, level1_height);
    let final_range = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        4,
        final_range_end_row,
        4,
        bounds,
    )?;
    expanded_range_edge_floor = expanded_range_edge_floor.saturating_add(final_range.cell_count());
    installed_range_scalar_edges =
        installed_range_scalar_edges.saturating_add(invalidation.set_cell_dependencies(
            final_dependent.clone(),
            [GridDependency::Range(final_range)],
        )?);
    let downstream = address(1, 6);
    let downstream_scalar_edges = invalidation.set_cell_dependencies(
        downstream.clone(),
        [GridDependency::Cell(final_dependent.clone())],
    )?;

    let selected_level1_index = LEVEL1_COUNT / 2;
    let selected_level1_row = sum_pyramid_row_for_level1(selected_level1_index, level1_height);
    let selected_seed_row = selected_level1_row + (level1_height / 2).min(level1_height - 1);
    let seed = address(selected_seed_row, 1);
    let level1_dependent = address(selected_level1_row, 2);
    let selected_level2_index = selected_level1_index / GROUP_SIZE;
    let selected_level2_row =
        sum_pyramid_row_for_level1(selected_level2_index * GROUP_SIZE, level1_height);
    let level2_dependent = address(selected_level2_row, 3);
    let selected_level3_index = selected_level2_index / GROUP_SIZE;
    let selected_level3_row = sum_pyramid_row_for_level1(
        selected_level3_index * GROUP_SIZE * GROUP_SIZE,
        level1_height,
    );
    let level3_dependent = address(selected_level3_row, 4);

    let leaf_query = invalidation.compressed_range_query_report(seed.clone())?;
    let level1_query = invalidation.compressed_range_query_report(level1_dependent.clone())?;
    let level2_query = invalidation.compressed_range_query_report(level2_dependent.clone())?;
    let level3_query = invalidation.compressed_range_query_report(level3_dependent.clone())?;
    let indexed_candidate_sum = leaf_query
        .indexed_candidate_count
        .saturating_add(level1_query.indexed_candidate_count)
        .saturating_add(level2_query.indexed_candidate_count)
        .saturating_add(level3_query.indexed_candidate_count);
    let matched_dependent_sum = leaf_query
        .matched_dependent_count
        .saturating_add(level1_query.matched_dependent_count)
        .saturating_add(level2_query.matched_dependent_count)
        .saturating_add(level3_query.matched_dependent_count);
    let dirty_closure = invalidation.dirty_closure([seed.clone()]);
    let total_compressed_range_edges = invalidation.compressed_range_edge_count();
    let compressed_support_edge_ratio_micros = micros_ratio(
        u64::try_from(total_compressed_range_edges).unwrap_or(u64::MAX),
        expanded_range_edge_floor,
    );
    let indexed_candidate_ratio_micros = micros_ratio(
        u64::try_from(indexed_candidate_sum).unwrap_or(u64::MAX),
        u64::try_from(total_compressed_range_edges).unwrap_or(u64::MAX),
    );

    Ok(json!({
        "counters": {
            "level1_count": LEVEL1_COUNT,
            "level2_count": LEVEL2_COUNT,
            "level3_count": LEVEL3_COUNT,
            "level1_height": level1_height,
            "covered_rows": covered_rows,
            "scalarization_limit": SCALARIZATION_LIMIT,
            "expanded_range_edge_floor": expanded_range_edge_floor,
            "installed_range_scalar_edges": installed_range_scalar_edges,
            "downstream_scalar_edges": downstream_scalar_edges,
            "scalar_edges_total": invalidation.scalar_edge_count(),
            "total_compressed_range_edges": total_compressed_range_edges,
            "compressed_support_edge_ratio_micros": compressed_support_edge_ratio_micros,
            "leaf_indexed_candidate_count": leaf_query.indexed_candidate_count,
            "level1_indexed_candidate_count": level1_query.indexed_candidate_count,
            "level2_indexed_candidate_count": level2_query.indexed_candidate_count,
            "level3_indexed_candidate_count": level3_query.indexed_candidate_count,
            "indexed_candidate_sum": indexed_candidate_sum,
            "indexed_candidate_ratio_micros": indexed_candidate_ratio_micros,
            "matched_dependent_sum": matched_dependent_sum,
            "dirty_seed_row": seed.row,
            "dirty_closure_size": dirty_closure.len(),
            "dirty_closure_contains_level1": dirty_closure.contains(&level1_dependent),
            "dirty_closure_contains_level2": dirty_closure.contains(&level2_dependent),
            "dirty_closure_contains_level3": dirty_closure.contains(&level3_dependent),
            "dirty_closure_contains_final": dirty_closure.contains(&final_dependent),
            "dirty_closure_contains_downstream": dirty_closure.contains(&downstream)
        },
        "register_assertions": [
            register_assertion(
                "P-12",
                "sum-pyramid-1M compressed range queries use the block index across every aggregation level",
                total_compressed_range_edges == usize::try_from(LEVEL1_COUNT + LEVEL2_COUNT + LEVEL3_COUNT + 1).unwrap_or(usize::MAX)
                    && indexed_candidate_sum < total_compressed_range_edges
                    && indexed_candidate_ratio_micros <= 250_000
                    && matched_dependent_sum == 4
                    && dirty_closure.contains(&downstream)
            ),
            register_assertion(
                "P-13",
                "sum-pyramid-1M keeps pyramid range support compressed instead of expanding range cells",
                installed_range_scalar_edges == 0
                    && downstream_scalar_edges == 1
                    && invalidation.scalar_edge_count() == 1
                    && expanded_range_edge_floor > u64::try_from(total_compressed_range_edges).unwrap_or(u64::MAX)
                    && compressed_support_edge_ratio_micros < 1_000_000
            ),
            register_assertion(
                "GRID-SUM-PYRAMID-1M",
                "a leaf edit in the compressed sum pyramid dirties exactly the selected aggregation chain and downstream dependent",
                dirty_closure.len() == 6
                    && dirty_closure.contains(&seed)
                    && dirty_closure.contains(&level1_dependent)
                    && dirty_closure.contains(&level2_dependent)
                    && dirty_closure.contains(&level3_dependent)
                    && dirty_closure.contains(&final_dependent)
                    && dirty_closure.contains(&downstream)
            )
        ]
    }))
}

pub(super) fn dirty_rect_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    const SCALARIZATION_LIMIT: u64 = 4;
    const DEPENDENCY_COUNT: u32 = 1_000;
    if options.rows < DEPENDENCY_COUNT * (u32::try_from(SCALARIZATION_LIMIT).unwrap() + 1) {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "dirty-rect-1m requires enough rows for compressed range dependencies"
                .to_string(),
        });
    }
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "dirty-rect-1m requires at least 4 columns".to_string(),
        });
    }

    let range_height = options.rows / DEPENDENCY_COUNT;
    let selected_range_index = DEPENDENCY_COUNT / 2;
    let selected_start_row = selected_range_index * range_height + 1;
    let selected_scalar_dependency_row =
        selected_start_row + (range_height / 2).min(range_height - 1);
    let mut invalidation =
        GridInvalidationRef::with_scalarization_limit(bounds, SCALARIZATION_LIMIT);
    let mut expanded_range_edge_floor = 0_u64;
    let mut installed_range_scalar_edges = 0_usize;

    for range_index in 0..DEPENDENCY_COUNT {
        let start_row = range_index * range_height + 1;
        let end_row = start_row + range_height - 1;
        let range_dependent = address(range_index + 1, 2);
        let scalar_dependent = address(range_index + 1, 3);
        let scalar_dependency_row = start_row + (range_height / 2).min(range_height - 1);
        let range = GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            start_row,
            1,
            end_row,
            1,
            bounds,
        )?;
        expanded_range_edge_floor = expanded_range_edge_floor.saturating_add(range.cell_count());
        installed_range_scalar_edges = installed_range_scalar_edges.saturating_add(
            invalidation.set_cell_dependencies(range_dependent, [GridDependency::Range(range)])?,
        );
        invalidation.set_cell_dependencies(
            scalar_dependent,
            [GridDependency::Cell(address(scalar_dependency_row, 1))],
        )?;
    }

    let selected_range_dependent = address(selected_range_index + 1, 2);
    let selected_scalar_dependent = address(selected_range_index + 1, 3);
    let downstream = address(1, 4);
    invalidation.set_cell_dependencies(
        downstream.clone(),
        [
            GridDependency::Cell(selected_range_dependent.clone()),
            GridDependency::Cell(selected_scalar_dependent.clone()),
        ],
    )?;

    let dirty_rect_top = selected_scalar_dependency_row.saturating_sub(5).max(1);
    let dirty_rect_bottom = selected_scalar_dependency_row
        .saturating_add(5)
        .min(options.rows);
    let dirty_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        dirty_rect_top,
        1,
        dirty_rect_bottom,
        1,
        bounds,
    )?;
    let report = invalidation.dirty_rect_query_report(dirty_rect)?;
    let indexed_candidate_sum = report
        .indexed_scalar_candidate_count
        .saturating_add(report.indexed_compressed_range_candidate_count);
    let total_edge_count = report
        .total_scalar_edges
        .saturating_add(report.total_compressed_range_edges);
    let indexed_candidate_ratio_micros = micros_ratio(
        u64::try_from(indexed_candidate_sum).unwrap_or(u64::MAX),
        u64::try_from(total_edge_count).unwrap_or(u64::MAX),
    );

    Ok(json!({
        "counters": {
            "dependency_count": DEPENDENCY_COUNT,
            "range_height": range_height,
            "dirty_rect_top": dirty_rect_top,
            "dirty_rect_bottom": dirty_rect_bottom,
            "dirty_rect_cell_count": report.seed_rect_cell_count,
            "selected_scalar_dependency_row": selected_scalar_dependency_row,
            "expanded_range_edge_floor": expanded_range_edge_floor,
            "installed_range_scalar_edges": installed_range_scalar_edges,
            "total_scalar_edges": report.total_scalar_edges,
            "total_compressed_range_edges": report.total_compressed_range_edges,
            "total_edge_count": total_edge_count,
            "indexed_scalar_candidate_count": report.indexed_scalar_candidate_count,
            "matched_scalar_dependent_count": report.matched_scalar_dependent_count,
            "indexed_compressed_range_candidate_count": report.indexed_compressed_range_candidate_count,
            "matched_compressed_range_dependent_count": report.matched_compressed_range_dependent_count,
            "indexed_candidate_sum": indexed_candidate_sum,
            "indexed_candidate_ratio_micros": indexed_candidate_ratio_micros,
            "direct_dependent_count": report.direct_dependents.len(),
            "dirty_closure_size": report.dirty_closure.len(),
            "dirty_closure_contains_range_dependent": report.dirty_closure.contains(&selected_range_dependent),
            "dirty_closure_contains_scalar_dependent": report.dirty_closure.contains(&selected_scalar_dependent),
            "dirty_closure_contains_downstream": report.dirty_closure.contains(&downstream)
        },
        "register_assertions": [
            register_assertion(
                "P-12",
                "dirty-rect-1M queries scalar and compressed range consumers through block indexes without expanding the rectangle or scanning all edges",
                report.seed_rect_cell_count == 11
                    && installed_range_scalar_edges == 0
                    && report.indexed_scalar_candidate_count < report.total_scalar_edges
                    && report.indexed_compressed_range_candidate_count < report.total_compressed_range_edges
                    && report.matched_scalar_dependent_count == 1
                    && report.matched_compressed_range_dependent_count == 1
                    && indexed_candidate_ratio_micros <= 10_000
                    && report.dirty_closure.contains(&downstream)
            ),
            register_assertion(
                "GRID-DIRTY-RECT-1M",
                "an 11-cell dirty rectangle dirties the selected range consumer, scalar consumer, and downstream dependent",
                report.direct_dependents.len() == 2
                    && report.dirty_closure.len() == 3
                    && report.dirty_closure.contains(&selected_range_dependent)
                    && report.dirty_closure.contains(&selected_scalar_dependent)
                    && report.dirty_closure.contains(&downstream)
            )
        ]
    }))
}

pub(super) fn hide_storm_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    const VISIBILITY_DEPENDENCY_COUNT: u32 = 1_000;
    if options.rows < VISIBILITY_DEPENDENCY_COUNT {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "hide-storm-1m requires at least 1000 rows".to_string(),
        });
    }
    if options.cols < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "hide-storm-1m requires at least 3 columns".to_string(),
        });
    }

    let range_height = options.rows / VISIBILITY_DEPENDENCY_COUNT;
    let covered_rows = range_height * VISIBILITY_DEPENDENCY_COUNT;
    let selected_range_index = VISIBILITY_DEPENDENCY_COUNT / 2;
    let selected_start = selected_range_index * range_height + 1;
    let selected_seed_row = selected_start + (range_height / 2).min(range_height - 1);
    let selected_dependent = address(selected_range_index + 1, 2);
    let downstream = address(1, 3);
    let seed_dependency = GridAxisVisibilityDependency::rows(selected_seed_row, selected_seed_row);

    let mut invalidation = GridInvalidationRef::new(bounds);
    for range_index in 0..VISIBILITY_DEPENDENCY_COUNT {
        let start_row = range_index * range_height + 1;
        let end_row = start_row + range_height - 1;
        invalidation.set_cell_dependencies(
            address(range_index + 1, 2),
            [GridDependency::AxisVisibility(
                GridAxisVisibilityDependency::rows(start_row, end_row),
            )],
        )?;
    }
    invalidation.set_cell_dependencies(
        downstream.clone(),
        [GridDependency::Cell(selected_dependent.clone())],
    )?;

    let query = invalidation.axis_visibility_query_report(seed_dependency.clone())?;
    let dirty_closure = invalidation.dirty_closure_for_axis_visibility(seed_dependency.clone())?;
    let total_axis_visibility_edges = invalidation.axis_visibility_edge_count();
    let naive_candidate_floor = total_axis_visibility_edges;
    let indexed_candidate_ratio_micros = micros_ratio(
        u64::try_from(query.indexed_candidate_count).unwrap_or(u64::MAX),
        u64::try_from(naive_candidate_floor).unwrap_or(u64::MAX),
    );

    Ok(json!({
        "counters": {
            "visibility_dependency_count": VISIBILITY_DEPENDENCY_COUNT,
            "visibility_range_height": range_height,
            "covered_rows": covered_rows,
            "total_axis_visibility_edges": total_axis_visibility_edges,
            "naive_candidate_floor": naive_candidate_floor,
            "indexed_candidate_count": query.indexed_candidate_count,
            "matched_dependent_count": query.matched_dependent_count,
            "indexed_candidate_ratio_micros": indexed_candidate_ratio_micros,
            "visibility_seed_row": selected_seed_row,
            "dirty_closure_size": dirty_closure.len(),
            "dirty_closure_contains_selected_dependent": dirty_closure.contains(&selected_dependent),
            "dirty_closure_contains_downstream": dirty_closure.contains(&downstream)
        },
        "register_assertions": [
            register_assertion(
                "P-24",
                "hidden-row visibility invalidation uses the block index instead of scanning every hidden-sensitive range",
                total_axis_visibility_edges
                    == usize::try_from(VISIBILITY_DEPENDENCY_COUNT).unwrap_or(usize::MAX)
                    && query.indexed_candidate_count < naive_candidate_floor
                    && query.indexed_candidate_count <= 4
                    && query.matched_dependent_count == 1
                    && indexed_candidate_ratio_micros <= 4_000
                    && dirty_closure.contains(&selected_dependent)
                    && dirty_closure.contains(&downstream)
            ),
            register_assertion(
                "GRID-HIDE-STORM-1M",
                "a hidden-row change inside one of 1000 hidden-sensitive row bands dirties only the matching aggregate chain",
                dirty_closure.len() == 2
                    && dirty_closure.contains(&selected_dependent)
                    && dirty_closure.contains(&downstream)
            )
        ]
    }))
}

pub(super) fn aggregate_context_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.rows < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "aggregate-context-1m requires at least 3 rows".to_string(),
        });
    }
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "aggregate-context-1m requires at least 2 columns".to_string(),
        });
    }

    let manual_hidden_row = (options.rows / 3).max(2);
    let mut filtered_hidden_row = ((options.rows.saturating_mul(2)) / 3).max(3);
    if filtered_hidden_row == manual_hidden_row {
        filtered_hidden_row = filtered_hidden_row.saturating_add(1).min(options.rows);
    }

    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    sheet.axis_state_mut().set_row(
        manual_hidden_row,
        GridAxisProps {
            hidden_manual: true,
            ..GridAxisProps::visible()
        },
    );
    sheet.axis_state_mut().set_row(
        filtered_hidden_row,
        GridAxisProps {
            hidden_filter: true,
            ..GridAxisProps::visible()
        },
    );
    let provider = sheet.host_info_provider(1, 2);
    let reference = ReferenceLike::new(ReferenceKind::Area, format!("A1:A{}", options.rows));
    let report = provider
        .aggregate_context_query_report(&reference)
        .map_err(|source| GridScaleRunnerError::InvalidOptions {
            detail: format!("aggregate-context provider report failed: {source:?}"),
        })?;
    let axis_run_ratio_micros = micros_ratio(
        u64::try_from(report.axis_run_probe_count).unwrap_or(u64::MAX),
        u64::try_from(report.declared_cell_count).unwrap_or(u64::MAX),
    );

    Ok(json!({
        "counters": {
            "aggregate_reference_declared_cells": report.declared_cell_count,
            "aggregate_reference_rows": report.rows,
            "aggregate_reference_cols": report.cols,
            "manual_hidden_row": manual_hidden_row,
            "filtered_hidden_row": filtered_hidden_row,
            "explicit_axis_row_entries_visited": report.explicit_axis_row_entries_visited,
            "default_row_runs": report.default_row_runs,
            "row_context_runs": report.row_context_runs,
            "axis_run_probe_count": report.axis_run_probe_count,
            "axis_run_probe_ratio_micros": axis_run_ratio_micros,
            "per_cell_context_expansion_count": report.per_cell_context_expansion_count,
            "manually_hidden_rows": report.manually_hidden_rows,
            "filtered_hidden_rows": report.filtered_hidden_rows
        },
        "register_assertions": [
            register_assertion(
                "P-28",
                "aggregate host context provider probes AxisState row runs instead of every referenced cell",
                report.declared_cell_count == usize::try_from(options.rows).unwrap_or(usize::MAX)
                    && report.explicit_axis_row_entries_visited == 2
                    && report.default_row_runs == 3
                    && report.axis_run_probe_count == 5
                    && report.axis_run_probe_count < report.declared_cell_count
                    && report.per_cell_context_expansion_count == report.declared_cell_count
                    && report.manually_hidden_rows == 1
                    && report.filtered_hidden_rows == 1
            ),
            register_assertion(
                "GRID-AGGREGATE-CONTEXT-1M",
                "a 1M-row SUBTOTAL-style context query reads five row-context runs and records the current per-cell seam expansion",
                report.rows == usize::try_from(options.rows).unwrap_or(usize::MAX)
                    && report.cols == 1
                    && report.axis_run_probe_count == 5
                    && axis_run_ratio_micros <= 5
            )
        ]
    }))
}

pub(super) fn spill_anchor_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "spill-anchor-1m requires at least 2 columns".to_string(),
        });
    }

    let middle_row = (options.rows / 2).max(1);
    let provider =
        ExcelGridReferenceSystemProvider::new("book:grid-scale", "sheet:grid-scale", 1, 2)
            .with_bounds(bounds)
            .with_cell_value(
                "book:grid-scale",
                "sheet:grid-scale",
                1,
                1,
                CalcValue::number(1.0),
            )
            .with_cell_value(
                "book:grid-scale",
                "sheet:grid-scale",
                middle_row,
                1,
                CalcValue::number(f64::from(middle_row)),
            )
            .with_cell_value(
                "book:grid-scale",
                "sheet:grid-scale",
                options.rows,
                1,
                CalcValue::number(f64::from(options.rows)),
            )
            .with_spill_extent(
                "book:grid-scale",
                "sheet:grid-scale",
                1,
                1,
                GridRect {
                    workbook_id: "book:grid-scale".to_string(),
                    sheet_id: "sheet:grid-scale".to_string(),
                    top_row: 1,
                    left_col: 1,
                    bottom_row: options.rows,
                    right_col: 1,
                },
            );
    let report = provider
        .spill_anchor_dereference_report(&ReferenceLike::new(ReferenceKind::SpillAnchor, "A1#"))
        .map_err(|source| GridScaleRunnerError::InvalidOptions {
            detail: format!("spill-anchor provider report failed: {source:?}"),
        })?;

    Ok(json!({
        "counters": {
            "spill_extent_declared_cells": report.declared_cell_count,
            "spill_ledger_probe_count": report.ledger_probe_count,
            "spill_extent_cells_scanned_for_ledger": report.extent_cells_scanned_for_ledger,
            "provider_value_entries_scanned": report.value_entries_scanned,
            "defined_cells_returned": report.defined_cells_returned,
            "anchor_row": report.anchor.row,
            "anchor_col": report.anchor.col
        },
        "register_assertions": [
            register_assertion(
                "P-25",
                "A1# resolves its spill extent with one ledger probe and no extent scan",
                report.declared_cell_count == usize::try_from(options.rows).unwrap_or(usize::MAX)
                    && report.ledger_probe_count == 1
                    && report.extent_cells_scanned_for_ledger == 0
                    && report.value_entries_scanned == 3
                    && report.defined_cells_returned == 3
            ),
            register_assertion(
                "GRID-SPILL-ANCHOR-1M",
                "1M-row A1# provider report remains occupancy-proportional for stored values",
                report.declared_cell_count == usize::try_from(options.rows).unwrap_or(usize::MAX)
                    && report.value_entries_scanned < report.declared_cell_count
                    && report.anchor == address(1, 1)
            )
        ]
    }))
}

pub(super) fn spill_epoch_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.rows < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "spill-epoch-1m requires at least 4 rows".to_string(),
        });
    }
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "spill-epoch-1m requires at least 4 columns".to_string(),
        });
    }

    let a1_anchor = address(1, 1);
    let unrelated_anchor = address(2, 2);
    let a1_consumer = address(1, 3);
    let downstream = address(1, 4);
    let a1_extent = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    let a1_shrunk_extent = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows - 1,
        1,
        bounds,
    )?;
    let unrelated_extent =
        GridRect::new("book:grid-scale", "sheet:grid-scale", 2, 2, 3, 2, bounds)?;
    let spill_facts = BTreeMap::from([
        (
            a1_anchor.clone(),
            GridSpillFact {
                anchor: a1_anchor.clone(),
                extent: a1_extent.clone(),
                blocked: false,
            },
        ),
        (
            unrelated_anchor.clone(),
            GridSpillFact {
                anchor: unrelated_anchor.clone(),
                extent: unrelated_extent.clone(),
                blocked: false,
            },
        ),
    ]);
    let mut base_ledger = GridSpillEpochLedger::default();
    let base_ledger_update = base_ledger.update_from_spill_facts(&spill_facts, |fact| {
        if fact.anchor == a1_anchor {
            "a1:v1".to_string()
        } else {
            "unrelated:v1".to_string()
        }
    });
    let base_snapshots = base_ledger.snapshots();

    let mut unchanged_ledger = base_ledger.clone();
    let unchanged_ledger_update = unchanged_ledger.update_from_spill_facts(&spill_facts, |fact| {
        if fact.anchor == a1_anchor {
            "a1:v1".to_string()
        } else {
            "unrelated:v1".to_string()
        }
    });

    let mut unrelated_churn_ledger = base_ledger.clone();
    let unrelated_churn_ledger_update =
        unrelated_churn_ledger.update_from_spill_facts(&spill_facts, |fact| {
            if fact.anchor == a1_anchor {
                "a1:v1".to_string()
            } else {
                "unrelated:v2".to_string()
            }
        });

    let mut a1_only_facts = BTreeMap::new();
    a1_only_facts.insert(
        a1_anchor.clone(),
        spill_facts.get(&a1_anchor).cloned().unwrap(),
    );
    let mut a1_only_ledger = GridSpillEpochLedger::default();
    a1_only_ledger.update_from_spill_facts(&a1_only_facts, |_| "a1:v1".to_string());
    let a1_only_snapshots = a1_only_ledger.snapshots();

    let mut value_change_ledger = a1_only_ledger.clone();
    let value_change_ledger_update =
        value_change_ledger.update_from_spill_facts(&a1_only_facts, |_| "a1:v2".to_string());

    let mut shrunk_facts = BTreeMap::new();
    shrunk_facts.insert(
        a1_anchor.clone(),
        GridSpillFact {
            anchor: a1_anchor.clone(),
            extent: a1_shrunk_extent,
            blocked: false,
        },
    );
    let mut extent_change_ledger = a1_only_ledger.clone();
    let extent_change_ledger_update =
        extent_change_ledger.update_from_spill_facts(&shrunk_facts, |_| "a1:v1".to_string());

    let mut invalidation = GridInvalidationRef::new(bounds);
    invalidation.set_cell_dependencies(
        a1_consumer.clone(),
        [GridDependency::SpillFact(GridSpillDependency::anchor(
            a1_anchor.clone(),
        ))],
    )?;
    invalidation.set_cell_dependencies(
        downstream.clone(),
        [GridDependency::Cell(a1_consumer.clone())],
    )?;

    let unchanged = invalidation.dirty_closure_for_spill_epoch_changes(
        base_snapshots.values().cloned(),
        unchanged_ledger.snapshots().values().cloned(),
    )?;
    let unrelated_churn = invalidation.dirty_closure_for_spill_epoch_changes(
        base_snapshots.values().cloned(),
        unrelated_churn_ledger.snapshots().values().cloned(),
    )?;
    let value_change = invalidation.dirty_closure_for_spill_epoch_changes(
        a1_only_snapshots.values().cloned(),
        value_change_ledger.snapshots().values().cloned(),
    )?;
    let extent_change = invalidation.dirty_closure_for_spill_epoch_changes(
        a1_only_snapshots.values().cloned(),
        extent_change_ledger.snapshots().values().cloned(),
    )?;

    let mut optimized_commit_sheet =
        GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let mut optimized_commit_valuation =
        GridOptimizedValuation::new("book:grid-scale", "sheet:grid-scale", bounds);
    optimized_commit_valuation.set_spill_fact(spill_facts.get(&a1_anchor).cloned().ok_or_else(
        || GridScaleRunnerError::InvalidOptions {
            detail: "spill-epoch commit setup missing A1 fact".to_string(),
        },
    )?)?;
    let optimized_commit_first = optimized_commit_sheet
        .commit_spill_publication_from_valuation(&optimized_commit_valuation)?;
    let optimized_commit_second = optimized_commit_sheet
        .commit_spill_publication_from_valuation(&optimized_commit_valuation)?;

    let mut optimized_commit_shrunk =
        GridOptimizedValuation::new("book:grid-scale", "sheet:grid-scale", bounds);
    optimized_commit_shrunk.set_spill_fact(shrunk_facts.get(&a1_anchor).cloned().ok_or_else(
        || GridScaleRunnerError::InvalidOptions {
            detail: "spill-epoch commit setup missing shrunk A1 fact".to_string(),
        },
    )?)?;
    let optimized_commit_extent =
        optimized_commit_sheet.commit_spill_publication_from_valuation(&optimized_commit_shrunk)?;

    Ok(json!({
        "counters": {
            "spill_extent_declared_cells": a1_extent.cell_count(),
            "spill_epoch_base_added": base_ledger_update.anchors_added,
            "spill_epoch_unchanged_preserved": unchanged_ledger_update.epochs_preserved,
            "spill_epoch_unrelated_value_changed": unrelated_churn_ledger_update.value_changed_anchors,
            "spill_epoch_a1_value_changed": value_change_ledger_update.value_changed_anchors,
            "spill_epoch_a1_extent_changed": extent_change_ledger_update.extent_changed_anchors,
            "optimized_spill_commit_first_added": optimized_commit_first.ledger_update.anchors_added,
            "optimized_spill_commit_first_committed_anchors": optimized_commit_first.committed_epoch_anchors,
            "optimized_spill_commit_second_preserved": optimized_commit_second.ledger_update.epochs_preserved,
            "optimized_spill_commit_extent_changed": optimized_commit_extent.ledger_update.extent_changed_anchors,
            "optimized_spill_commit_current_epoch_anchors": optimized_commit_sheet.spill_epoch_ledger().entries().len(),
            "spill_dependency_edges": invalidation.spill_edge_count(),
            "unchanged_anchors_compared": unchanged.anchors_compared,
            "unchanged_changed_anchors": unchanged.changed_anchors.len(),
            "unchanged_dirty_closure_size": unchanged.dirty_closure.len(),
            "unrelated_changed_anchors": unrelated_churn.changed_anchors.len(),
            "unrelated_value_epoch_changed_anchors": unrelated_churn.value_epoch_changed_anchors,
            "unrelated_dirty_closure_size": unrelated_churn.dirty_closure.len(),
            "value_changed_anchors": value_change.changed_anchors.len(),
            "value_epoch_changed_anchors": value_change.value_epoch_changed_anchors,
            "value_dirty_closure_size": value_change.dirty_closure.len(),
            "value_dirty_closure_contains_consumer": value_change.dirty_closure.contains(&a1_consumer),
            "value_dirty_closure_contains_downstream": value_change.dirty_closure.contains(&downstream),
            "extent_changed_anchors": extent_change.changed_anchors.len(),
            "extent_epoch_changed_anchors": extent_change.extent_epoch_changed_anchors,
            "extent_dirty_closure_size": extent_change.dirty_closure.len(),
            "extent_dirty_closure_contains_consumer": extent_change.dirty_closure.contains(&a1_consumer),
            "extent_dirty_closure_contains_downstream": extent_change.dirty_closure.contains(&downstream)
        },
        "register_assertions": [
            register_assertion(
                "P-27",
                "A1# consumers dirty only when their spill anchor extent or value epoch changes",
                a1_extent.cell_count() == u64::from(options.rows)
                    && base_ledger_update.anchors_added == 2
                    && unchanged_ledger_update.epochs_preserved == 2
                    && unrelated_churn_ledger_update.value_changed_anchors == 1
                    && value_change_ledger_update.value_changed_anchors == 1
                    && extent_change_ledger_update.extent_changed_anchors == 1
                    && optimized_commit_first.ledger_update.anchors_added == 1
                    && optimized_commit_second.ledger_update.epochs_preserved == 1
                    && optimized_commit_extent.ledger_update.extent_changed_anchors == 1
                    && optimized_commit_sheet.spill_epoch_ledger().entries().len() == 1
                    && invalidation.spill_edge_count() == 1
                    && unchanged.dirty_closure.is_empty()
                    && unrelated_churn.changed_anchors.len() == 1
                    && unrelated_churn.dirty_closure.is_empty()
                    && value_change.value_epoch_changed_anchors == 1
                    && value_change.dirty_closure.len() == 2
                    && value_change.dirty_closure.contains(&a1_consumer)
                    && value_change.dirty_closure.contains(&downstream)
                    && extent_change.extent_epoch_changed_anchors == 1
                    && extent_change.dirty_closure.len() == 2
                    && extent_change.dirty_closure.contains(&a1_consumer)
                    && extent_change.dirty_closure.contains(&downstream)
            ),
            register_assertion(
                "GRID-SPILL-EPOCH-1M",
                "a 1M-row A1# extent has epoch-precise invalidation for unchanged, unrelated, value, and extent changes",
                unchanged.anchors_compared == 2
                    && unchanged.changed_anchors.is_empty()
                    && unrelated_churn.dirty_closure.is_empty()
                    && value_change.dirty_closure == extent_change.dirty_closure
                    && value_change.dirty_closure.len() == 2
            )
        ]
    }))
}

pub(super) fn filter_spill_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.rows < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "filter-spill-1m requires at least 4 rows".to_string(),
        });
    }
    if options.cols < 5 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "filter-spill-1m requires at least 5 columns".to_string(),
        });
    }

    let anchor = address(1, 1);
    let old_extent = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    let new_bottom_row = (options.rows / 2).max(1);
    let new_extent = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        new_bottom_row,
        1,
        bounds,
    )?;
    let mut old_spill_rows = vec![1, (options.rows / 2).max(1), options.rows];
    old_spill_rows.sort_unstable();
    old_spill_rows.dedup();
    let mut new_spill_rows = vec![1, (new_bottom_row / 2).max(1), new_bottom_row];
    new_spill_rows.sort_unstable();
    new_spill_rows.dedup();
    let unrelated_sparse_value_count = options.rows.min(1_000);

    let mut valuation = GridOptimizedValuation::new("book:grid-scale", "sheet:grid-scale", bounds);
    valuation.set_spill_fact(GridSpillFact {
        anchor: anchor.clone(),
        extent: old_extent.clone(),
        blocked: false,
    })?;
    for row in &old_spill_rows {
        valuation.insert_sparse_computed_value(
            address(*row, 1),
            u64::from(*row),
            CalcValue::number(f64::from(*row)),
            GridOptimizedCellSource::SparsePoint,
        )?;
    }
    for row in 1..=unrelated_sparse_value_count {
        valuation.insert_sparse_computed_value(
            address(row, 2),
            u64::from(row),
            CalcValue::number(f64::from(row * 10)),
            GridOptimizedCellSource::SparsePoint,
        )?;
    }

    let sparse_values_before_clear = valuation.sparse_computed_cells();
    let clear_report = valuation.clear_formula_output_for_anchor_report(&anchor)?;
    let sparse_values_after_clear = valuation.sparse_computed_cells();
    valuation.set_spill_fact(GridSpillFact {
        anchor: anchor.clone(),
        extent: new_extent.clone(),
        blocked: false,
    })?;
    for row in &new_spill_rows {
        valuation.insert_sparse_computed_value(
            address(*row, 1),
            u64::from(*row).saturating_add(10_000),
            CalcValue::number(f64::from(*row)),
            GridOptimizedCellSource::SparsePoint,
        )?;
    }
    let sparse_values_after_respill = valuation.sparse_computed_cells();

    let grid_cell_capacity = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let old_new_extent_cell_budget = old_extent.cell_count();
    let respill_sparse_cells_touched = clear_report
        .indexed_candidate_count
        .saturating_add(new_spill_rows.len());
    let indexed_clear_ratio_micros = micros_ratio(
        u64::try_from(clear_report.indexed_candidate_count).unwrap_or(u64::MAX),
        u64::try_from(clear_report.naive_sparse_value_scan_floor).unwrap_or(u64::MAX),
    );
    let respill_touch_ratio_micros = micros_ratio(
        u64::try_from(respill_sparse_cells_touched).unwrap_or(u64::MAX),
        grid_cell_capacity,
    );
    let old_tail_cleared =
        valuation.read_cell(&address(options.rows, 1)).computed == CalcValue::empty();
    let new_tail_written = valuation.read_cell(&address(new_bottom_row, 1)).computed
        == CalcValue::number(f64::from(new_bottom_row));
    let unrelated_tail_kept = valuation
        .read_cell(&address(unrelated_sparse_value_count, 2))
        .computed
        == CalcValue::number(f64::from(unrelated_sparse_value_count * 10));

    let mut filter_sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let source_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    let include_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        3,
        options.rows,
        3,
        bounds,
    )?;
    filter_sheet.put_dense_number_region_with(source_rect, |address| {
        f64::from(address.row) * 100.0 + f64::from(address.col)
    })?;
    filter_sheet.put_dense_literal_region_with(include_rect, |address| {
        CalcValue::logical(address.row <= new_bottom_row)
    })?;
    filter_sheet.set_formula(
        address(1, 4),
        GridFormulaCell::new(
            format!("=FILTER(A1:B{},C1:C{})", options.rows, options.rows),
            format!(
                "excel.grid.v1:filter-spill:R1C1:R{}C2:R1C3:R{}C3",
                options.rows, options.rows
            ),
        ),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (filter_valuation, filter_committed) = filter_sheet
        .recalculate_mark_all_dirty_compact_with_oxfml_and_commit_spill_publication(
            materialization_limit,
        )?;
    let filter_recalc = &filter_committed.recalc;
    let filter_spill_commit = &filter_committed.spill_commit;
    let filter_anchor = address(1, 4);
    let filter_spill_fact = filter_valuation.spill_facts().get(&filter_anchor);
    let filter_spill_extent_declared_cells =
        filter_spill_fact.map_or(0, |fact| fact.extent.cell_count());
    let filter_spill_extent_rows = filter_spill_fact.map_or(0, |fact| fact.extent.row_count());
    let filter_spill_extent_cols = filter_spill_fact.map_or(0, |fact| fact.extent.col_count());
    let filter_sheet_committed_spill_fact_entries = filter_sheet.spill_facts().len();
    let filter_sheet_committed_epoch_anchors = filter_sheet.spill_epoch_ledger().entries().len();
    let middle_output_row = (new_bottom_row / 2).max(1);
    let filter_first_left_value = filter_valuation.read_cell(&address(1, 4)).computed;
    let filter_first_right_value = filter_valuation.read_cell(&address(1, 5)).computed;
    let filter_middle_left_value = filter_valuation
        .read_cell(&address(middle_output_row, 4))
        .computed;
    let filter_middle_right_value = filter_valuation
        .read_cell(&address(middle_output_row, 5))
        .computed;
    let filter_last_left_value = filter_valuation
        .read_cell(&address(new_bottom_row, 4))
        .computed;
    let filter_last_right_value = filter_valuation
        .read_cell(&address(new_bottom_row, 5))
        .computed;
    let filter_vacated_left_value = filter_valuation
        .read_cell(&address(
            new_bottom_row.saturating_add(1).min(options.rows),
            4,
        ))
        .computed;
    let filter_vacated_right_value = filter_valuation
        .read_cell(&address(
            new_bottom_row.saturating_add(1).min(options.rows),
            5,
        ))
        .computed;

    filter_sheet.set_literal(address(new_bottom_row, 3), CalcValue::logical(false))?;
    let (filter_lifecycle_valuation, filter_lifecycle_committed) = filter_sheet
        .recalculate_mark_all_dirty_compact_with_oxfml_and_commit_spill_publication(
            materialization_limit,
        )?;
    let filter_lifecycle_recalc = &filter_lifecycle_committed.recalc;
    let filter_lifecycle_spill_commit = &filter_lifecycle_committed.spill_commit;
    let filter_lifecycle_rows = new_bottom_row.saturating_sub(1).max(1);
    let filter_lifecycle_spill_fact = filter_lifecycle_valuation.spill_facts().get(&filter_anchor);
    let filter_lifecycle_spill_extent_declared_cells =
        filter_lifecycle_spill_fact.map_or(0, |fact| fact.extent.cell_count());
    let filter_lifecycle_spill_extent_rows =
        filter_lifecycle_spill_fact.map_or(0, |fact| fact.extent.row_count());
    let filter_lifecycle_spill_extent_cols =
        filter_lifecycle_spill_fact.map_or(0, |fact| fact.extent.col_count());
    let filter_lifecycle_sheet_committed_spill_fact_entries = filter_sheet.spill_facts().len();
    let filter_lifecycle_sheet_committed_epoch_anchors =
        filter_sheet.spill_epoch_ledger().entries().len();
    let filter_lifecycle_epoch = filter_sheet
        .spill_epoch_ledger()
        .snapshot_for(&filter_anchor)
        .map_or(0, |snapshot| snapshot.value_epoch);
    let filter_lifecycle_first_left_value = filter_lifecycle_valuation
        .read_cell(&address(1, 4))
        .computed;
    let filter_lifecycle_first_right_value = filter_lifecycle_valuation
        .read_cell(&address(1, 5))
        .computed;
    let filter_lifecycle_last_left_value = filter_lifecycle_valuation
        .read_cell(&address(filter_lifecycle_rows, 4))
        .computed;
    let filter_lifecycle_last_right_value = filter_lifecycle_valuation
        .read_cell(&address(filter_lifecycle_rows, 5))
        .computed;
    let filter_lifecycle_vacated_left_value = filter_lifecycle_valuation
        .read_cell(&address(new_bottom_row, 4))
        .computed;
    let filter_lifecycle_vacated_right_value = filter_lifecycle_valuation
        .read_cell(&address(new_bottom_row, 5))
        .computed;

    let horizontal_source_rows = options.rows.saturating_sub(1);
    let mut column_filter_sheet =
        GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let horizontal_source_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        horizontal_source_rows,
        3,
        bounds,
    )?;
    let horizontal_include_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        options.rows,
        1,
        options.rows,
        3,
        bounds,
    )?;
    column_filter_sheet.put_dense_number_region_with(horizontal_source_rect, |address| {
        f64::from(address.row) * 100.0 + f64::from(address.col)
    })?;
    column_filter_sheet.put_dense_literal_region_with(horizontal_include_rect, |address| {
        CalcValue::logical(address.col != 2)
    })?;
    column_filter_sheet.set_formula(
        address(1, 4),
        GridFormulaCell::new(
            format!(
                "=FILTER(A1:C{},A{}:C{})",
                horizontal_source_rows, options.rows, options.rows
            ),
            format!(
                "excel.grid.v1:filter-spill-columns:R1C1:R{}C3:R{}C1:R{}C3",
                horizontal_source_rows, options.rows, options.rows
            ),
        ),
    )?;
    let (column_filter_valuation, column_filter_committed) = column_filter_sheet
        .recalculate_mark_all_dirty_compact_with_oxfml_and_commit_spill_publication(
            materialization_limit,
        )?;
    let column_filter_recalc = &column_filter_committed.recalc;
    let column_filter_spill_commit = &column_filter_committed.spill_commit;
    let column_filter_anchor = address(1, 4);
    let column_filter_spill_fact = column_filter_valuation
        .spill_facts()
        .get(&column_filter_anchor);
    let column_filter_spill_extent_declared_cells =
        column_filter_spill_fact.map_or(0, |fact| fact.extent.cell_count());
    let column_filter_spill_extent_rows =
        column_filter_spill_fact.map_or(0, |fact| fact.extent.row_count());
    let column_filter_spill_extent_cols =
        column_filter_spill_fact.map_or(0, |fact| fact.extent.col_count());
    let column_filter_sheet_committed_spill_fact_entries = column_filter_sheet.spill_facts().len();
    let column_filter_sheet_committed_epoch_anchors =
        column_filter_sheet.spill_epoch_ledger().entries().len();
    let horizontal_middle_output_row = (horizontal_source_rows / 2).max(1);
    let column_filter_first_left_value = column_filter_valuation.read_cell(&address(1, 4)).computed;
    let column_filter_first_right_value =
        column_filter_valuation.read_cell(&address(1, 5)).computed;
    let column_filter_middle_left_value = column_filter_valuation
        .read_cell(&address(horizontal_middle_output_row, 4))
        .computed;
    let column_filter_middle_right_value = column_filter_valuation
        .read_cell(&address(horizontal_middle_output_row, 5))
        .computed;
    let column_filter_last_left_value = column_filter_valuation
        .read_cell(&address(horizontal_source_rows, 4))
        .computed;
    let column_filter_last_right_value = column_filter_valuation
        .read_cell(&address(horizontal_source_rows, 5))
        .computed;

    let counters = json_object([
        ("grid_cell_capacity", json!(grid_cell_capacity)),
        (
            "old_spill_extent_declared_cells",
            json!(clear_report.old_extent_cell_count),
        ),
        (
            "new_spill_extent_declared_cells",
            json!(new_extent.cell_count()),
        ),
        (
            "old_new_extent_cell_budget",
            json!(old_new_extent_cell_budget),
        ),
        ("old_sparse_spill_values", json!(old_spill_rows.len())),
        (
            "new_sparse_spill_values_written",
            json!(new_spill_rows.len()),
        ),
        (
            "unrelated_sparse_values",
            json!(unrelated_sparse_value_count),
        ),
        (
            "sparse_values_before_clear",
            json!(sparse_values_before_clear),
        ),
        (
            "sparse_values_after_clear",
            json!(sparse_values_after_clear),
        ),
        (
            "sparse_values_after_respill",
            json!(sparse_values_after_respill),
        ),
        (
            "naive_sparse_value_scan_floor",
            json!(clear_report.naive_sparse_value_scan_floor),
        ),
        (
            "indexed_clear_candidate_count",
            json!(clear_report.indexed_candidate_count),
        ),
        (
            "sparse_values_removed",
            json!(clear_report.sparse_values_removed),
        ),
        (
            "indexed_clear_ratio_micros",
            json!(indexed_clear_ratio_micros),
        ),
        (
            "respill_sparse_cells_touched",
            json!(respill_sparse_cells_touched),
        ),
        (
            "respill_touch_ratio_micros",
            json!(respill_touch_ratio_micros),
        ),
        ("old_tail_cleared", json!(old_tail_cleared)),
        ("new_tail_written", json!(new_tail_written)),
        ("unrelated_tail_kept", json!(unrelated_tail_kept)),
        (
            "filter_formula_spill_extent_declared_cells",
            json!(filter_spill_extent_declared_cells),
        ),
        (
            "filter_formula_spill_facts_published",
            json!(filter_recalc.spill_facts_published),
        ),
        (
            "filter_formula_spill_facts_blocked",
            json!(filter_recalc.spill_facts_blocked),
        ),
        (
            "filter_formula_spill_ghost_cells_published",
            json!(filter_recalc.spill_ghost_cells_published),
        ),
        (
            "filter_formula_spill_commit_previous_fact_entries",
            json!(filter_spill_commit.previous_spill_fact_entries),
        ),
        (
            "filter_formula_spill_commit_committed_fact_entries",
            json!(filter_spill_commit.committed_spill_fact_entries),
        ),
        (
            "filter_formula_spill_commit_anchors_added",
            json!(filter_spill_commit.ledger_update.anchors_added),
        ),
        (
            "filter_formula_spill_commit_current_epoch_anchors",
            json!(filter_sheet_committed_epoch_anchors),
        ),
        (
            "filter_formula_sheet_committed_spill_fact_entries",
            json!(filter_sheet_committed_spill_fact_entries),
        ),
        (
            "filter_formula_spill_extent_declared_rows",
            json!(filter_spill_extent_rows),
        ),
        (
            "filter_formula_spill_extent_declared_cols",
            json!(filter_spill_extent_cols),
        ),
        (
            "filter_formula_computed_dense_value_regions",
            json!(filter_recalc.computed_dense_value_regions),
        ),
        (
            "filter_formula_computed_dense_cells",
            json!(filter_valuation.dense_computed_cells()),
        ),
        (
            "filter_formula_computed_dense_numeric_packed_cells",
            json!(filter_valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "filter_formula_computed_sparse_cells",
            json!(filter_valuation.sparse_computed_cells()),
        ),
        (
            "filter_formula_first_left_value",
            json!(calc_value_display_text(&filter_first_left_value)),
        ),
        (
            "filter_formula_first_right_value",
            json!(calc_value_display_text(&filter_first_right_value)),
        ),
        (
            "filter_formula_middle_left_value",
            json!(calc_value_display_text(&filter_middle_left_value)),
        ),
        (
            "filter_formula_middle_right_value",
            json!(calc_value_display_text(&filter_middle_right_value)),
        ),
        (
            "filter_formula_last_left_value",
            json!(calc_value_display_text(&filter_last_left_value)),
        ),
        (
            "filter_formula_last_right_value",
            json!(calc_value_display_text(&filter_last_right_value)),
        ),
        (
            "filter_formula_vacated_left_value",
            json!(calc_value_display_text(&filter_vacated_left_value)),
        ),
        (
            "filter_formula_vacated_right_value",
            json!(calc_value_display_text(&filter_vacated_right_value)),
        ),
        ("filter_lifecycle_sparse_mask_overrides", json!(1)),
        (
            "filter_lifecycle_spill_extent_declared_cells",
            json!(filter_lifecycle_spill_extent_declared_cells),
        ),
        (
            "filter_lifecycle_spill_extent_declared_rows",
            json!(filter_lifecycle_spill_extent_rows),
        ),
        (
            "filter_lifecycle_spill_extent_declared_cols",
            json!(filter_lifecycle_spill_extent_cols),
        ),
        (
            "filter_lifecycle_spill_facts_published",
            json!(filter_lifecycle_recalc.spill_facts_published),
        ),
        (
            "filter_lifecycle_spill_facts_blocked",
            json!(filter_lifecycle_recalc.spill_facts_blocked),
        ),
        (
            "filter_lifecycle_spill_ghost_cells_published",
            json!(filter_lifecycle_recalc.spill_ghost_cells_published),
        ),
        (
            "filter_lifecycle_spill_commit_previous_fact_entries",
            json!(filter_lifecycle_spill_commit.previous_spill_fact_entries),
        ),
        (
            "filter_lifecycle_spill_commit_committed_fact_entries",
            json!(filter_lifecycle_spill_commit.committed_spill_fact_entries),
        ),
        (
            "filter_lifecycle_spill_commit_anchors_added",
            json!(filter_lifecycle_spill_commit.ledger_update.anchors_added),
        ),
        (
            "filter_lifecycle_spill_commit_anchors_changed",
            json!(filter_lifecycle_spill_commit.ledger_update.anchors_changed),
        ),
        (
            "filter_lifecycle_spill_commit_extent_changed_anchors",
            json!(
                filter_lifecycle_spill_commit
                    .ledger_update
                    .extent_changed_anchors
            ),
        ),
        (
            "filter_lifecycle_spill_commit_value_changed_anchors",
            json!(
                filter_lifecycle_spill_commit
                    .ledger_update
                    .value_changed_anchors
            ),
        ),
        (
            "filter_lifecycle_spill_commit_blocked_changed_anchors",
            json!(
                filter_lifecycle_spill_commit
                    .ledger_update
                    .blocked_changed_anchors
            ),
        ),
        (
            "filter_lifecycle_spill_commit_current_epoch_anchors",
            json!(filter_lifecycle_sheet_committed_epoch_anchors),
        ),
        (
            "filter_lifecycle_sheet_committed_spill_fact_entries",
            json!(filter_lifecycle_sheet_committed_spill_fact_entries),
        ),
        (
            "filter_lifecycle_committed_value_epoch",
            json!(filter_lifecycle_epoch),
        ),
        (
            "filter_lifecycle_computed_dense_value_regions",
            json!(filter_lifecycle_recalc.computed_dense_value_regions),
        ),
        (
            "filter_lifecycle_computed_dense_cells",
            json!(filter_lifecycle_valuation.dense_computed_cells()),
        ),
        (
            "filter_lifecycle_computed_dense_numeric_packed_cells",
            json!(filter_lifecycle_valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "filter_lifecycle_computed_sparse_cells",
            json!(filter_lifecycle_valuation.sparse_computed_cells()),
        ),
        (
            "filter_lifecycle_first_left_value",
            json!(calc_value_display_text(&filter_lifecycle_first_left_value)),
        ),
        (
            "filter_lifecycle_first_right_value",
            json!(calc_value_display_text(&filter_lifecycle_first_right_value)),
        ),
        (
            "filter_lifecycle_last_left_value",
            json!(calc_value_display_text(&filter_lifecycle_last_left_value)),
        ),
        (
            "filter_lifecycle_last_right_value",
            json!(calc_value_display_text(&filter_lifecycle_last_right_value)),
        ),
        (
            "filter_lifecycle_vacated_left_value",
            json!(calc_value_display_text(
                &filter_lifecycle_vacated_left_value
            )),
        ),
        (
            "filter_lifecycle_vacated_right_value",
            json!(calc_value_display_text(
                &filter_lifecycle_vacated_right_value
            )),
        ),
        ("column_filter_source_rows", json!(horizontal_source_rows)),
        (
            "column_filter_spill_extent_declared_cells",
            json!(column_filter_spill_extent_declared_cells),
        ),
        (
            "column_filter_spill_facts_published",
            json!(column_filter_recalc.spill_facts_published),
        ),
        (
            "column_filter_spill_facts_blocked",
            json!(column_filter_recalc.spill_facts_blocked),
        ),
        (
            "column_filter_spill_ghost_cells_published",
            json!(column_filter_recalc.spill_ghost_cells_published),
        ),
        (
            "column_filter_spill_commit_previous_fact_entries",
            json!(column_filter_spill_commit.previous_spill_fact_entries),
        ),
        (
            "column_filter_spill_commit_committed_fact_entries",
            json!(column_filter_spill_commit.committed_spill_fact_entries),
        ),
        (
            "column_filter_spill_commit_anchors_added",
            json!(column_filter_spill_commit.ledger_update.anchors_added),
        ),
        (
            "column_filter_spill_commit_current_epoch_anchors",
            json!(column_filter_sheet_committed_epoch_anchors),
        ),
        (
            "column_filter_sheet_committed_spill_fact_entries",
            json!(column_filter_sheet_committed_spill_fact_entries),
        ),
        (
            "column_filter_spill_extent_declared_rows",
            json!(column_filter_spill_extent_rows),
        ),
        (
            "column_filter_spill_extent_declared_cols",
            json!(column_filter_spill_extent_cols),
        ),
        (
            "column_filter_computed_dense_value_regions",
            json!(column_filter_recalc.computed_dense_value_regions),
        ),
        (
            "column_filter_computed_dense_cells",
            json!(column_filter_valuation.dense_computed_cells()),
        ),
        (
            "column_filter_computed_dense_numeric_packed_cells",
            json!(column_filter_valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "column_filter_computed_sparse_cells",
            json!(column_filter_valuation.sparse_computed_cells()),
        ),
        (
            "column_filter_first_left_value",
            json!(calc_value_display_text(&column_filter_first_left_value)),
        ),
        (
            "column_filter_first_right_value",
            json!(calc_value_display_text(&column_filter_first_right_value)),
        ),
        (
            "column_filter_middle_left_value",
            json!(calc_value_display_text(&column_filter_middle_left_value)),
        ),
        (
            "column_filter_middle_right_value",
            json!(calc_value_display_text(&column_filter_middle_right_value)),
        ),
        (
            "column_filter_last_left_value",
            json!(calc_value_display_text(&column_filter_last_left_value)),
        ),
        (
            "column_filter_last_right_value",
            json!(calc_value_display_text(&column_filter_last_right_value)),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-23",
            "re-spill old-output cleanup uses the sparse index over the old spill extent instead of scanning the whole sparse value map or sheet",
            clear_report.had_spill_fact
                && clear_report.old_extent_cell_count == u64::from(options.rows)
                && new_extent.cell_count() < clear_report.old_extent_cell_count
                && old_new_extent_cell_budget == clear_report.old_extent_cell_count
                && clear_report.naive_sparse_value_scan_floor == sparse_values_before_clear
                && clear_report.indexed_candidate_count == old_spill_rows.len()
                && clear_report.sparse_values_removed == old_spill_rows.len()
                && clear_report.indexed_candidate_count
                    < clear_report.naive_sparse_value_scan_floor
                && u64::try_from(respill_sparse_cells_touched).unwrap_or(u64::MAX)
                    < grid_cell_capacity
                && sparse_values_after_clear
                    == usize::try_from(unrelated_sparse_value_count).unwrap_or(usize::MAX)
                && old_tail_cleared
                && new_tail_written
                && unrelated_tail_kept
        ),
        register_assertion(
            "GRID-FILTER-SPILL-1M",
            "a 1M-row re-spill resize clears only old spill outputs, preserves unrelated sparse values, and a real two-column FILTER formula publishes dense output",
            old_extent.cell_count() == u64::from(options.rows)
                && clear_report.indexed_candidate_count == old_spill_rows.len()
                && sparse_values_after_respill
                    == usize::try_from(unrelated_sparse_value_count).unwrap_or(usize::MAX)
                        + new_spill_rows.len()
                && clear_report.old_extent == old_extent
                && valuation
                    .spill_facts()
                    .get(&anchor)
                    .is_some_and(|fact| fact.extent == new_extent)
                && filter_spill_extent_declared_cells
                    == u64::from(new_bottom_row).saturating_mul(2)
                && filter_spill_extent_rows == new_bottom_row
                && filter_spill_extent_cols == 2
                && filter_recalc.spill_facts_published == 1
                && filter_recalc.spill_facts_blocked == 0
                && filter_recalc.spill_ghost_cells_published
                    == usize::try_from(
                        u64::from(new_bottom_row)
                            .saturating_mul(2)
                            .saturating_sub(1)
                    )
                    .unwrap_or(usize::MAX)
                && filter_spill_commit.previous_spill_fact_entries == 0
                && filter_spill_commit.committed_spill_fact_entries == 1
                && filter_spill_commit.ledger_update.anchors_added == 1
                && filter_sheet_committed_spill_fact_entries == 1
                && filter_sheet_committed_epoch_anchors == 1
                && filter_valuation.sparse_computed_cells() == 0
                && filter_first_left_value == CalcValue::number(101.0)
                && filter_first_right_value == CalcValue::number(102.0)
                && filter_middle_left_value
                    == CalcValue::number(f64::from(middle_output_row) * 100.0 + 1.0)
                && filter_middle_right_value
                    == CalcValue::number(f64::from(middle_output_row) * 100.0 + 2.0)
                && filter_last_left_value
                    == CalcValue::number(f64::from(new_bottom_row) * 100.0 + 1.0)
                && filter_last_right_value
                    == CalcValue::number(f64::from(new_bottom_row) * 100.0 + 2.0)
                && filter_vacated_left_value == CalcValue::empty()
                && filter_vacated_right_value == CalcValue::empty()
        ),
        register_assertion(
            "GRID-FILTER-LIFECYCLE-1M",
            "a later committed optimized FILTER recalc shrinks the dense output, clears vacated ghosts, and advances the spill epoch",
            filter_lifecycle_spill_extent_declared_cells
                == u64::from(filter_lifecycle_rows).saturating_mul(2)
                && filter_lifecycle_spill_extent_rows == filter_lifecycle_rows
                && filter_lifecycle_spill_extent_cols == 2
                && filter_lifecycle_recalc.spill_facts_published == 1
                && filter_lifecycle_recalc.spill_facts_blocked == 0
                && filter_lifecycle_recalc.spill_ghost_cells_published
                    == usize::try_from(
                        u64::from(filter_lifecycle_rows)
                            .saturating_mul(2)
                            .saturating_sub(1)
                    )
                    .unwrap_or(usize::MAX)
                && filter_lifecycle_spill_commit.previous_spill_fact_entries == 1
                && filter_lifecycle_spill_commit.committed_spill_fact_entries == 1
                && filter_lifecycle_spill_commit.ledger_update.anchors_added == 0
                && filter_lifecycle_spill_commit.ledger_update.anchors_changed == 1
                && filter_lifecycle_spill_commit
                    .ledger_update
                    .extent_changed_anchors
                    == 1
                && filter_lifecycle_spill_commit
                    .ledger_update
                    .value_changed_anchors
                    == 1
                && filter_lifecycle_spill_commit
                    .ledger_update
                    .blocked_changed_anchors
                    == 0
                && filter_lifecycle_sheet_committed_spill_fact_entries == 1
                && filter_lifecycle_sheet_committed_epoch_anchors == 1
                && filter_lifecycle_epoch == 2
                && filter_lifecycle_valuation.sparse_computed_cells() == 1
                && filter_lifecycle_first_left_value == CalcValue::number(101.0)
                && filter_lifecycle_first_right_value == CalcValue::number(102.0)
                && filter_lifecycle_last_left_value
                    == CalcValue::number(f64::from(filter_lifecycle_rows) * 100.0 + 1.0)
                && filter_lifecycle_last_right_value
                    == CalcValue::number(f64::from(filter_lifecycle_rows) * 100.0 + 2.0)
                && filter_lifecycle_vacated_left_value == CalcValue::empty()
                && filter_lifecycle_vacated_right_value == CalcValue::empty()
        ),
        register_assertion(
            "GRID-FILTER-COLUMN-SPILL-1M",
            "a 1M-row three-column source with a horizontal include row publishes and commits a dense two-column FILTER output",
            column_filter_spill_extent_declared_cells
                == u64::from(horizontal_source_rows).saturating_mul(2)
                && column_filter_spill_extent_rows == horizontal_source_rows
                && column_filter_spill_extent_cols == 2
                && column_filter_recalc.spill_facts_published == 1
                && column_filter_recalc.spill_facts_blocked == 0
                && column_filter_recalc.spill_ghost_cells_published
                    == usize::try_from(
                        u64::from(horizontal_source_rows)
                            .saturating_mul(2)
                            .saturating_sub(1)
                    )
                    .unwrap_or(usize::MAX)
                && column_filter_spill_commit.previous_spill_fact_entries == 0
                && column_filter_spill_commit.committed_spill_fact_entries == 1
                && column_filter_spill_commit.ledger_update.anchors_added == 1
                && column_filter_sheet_committed_spill_fact_entries == 1
                && column_filter_sheet_committed_epoch_anchors == 1
                && column_filter_valuation.sparse_computed_cells() == 0
                && column_filter_first_left_value == CalcValue::number(101.0)
                && column_filter_first_right_value == CalcValue::number(103.0)
                && column_filter_middle_left_value
                    == CalcValue::number(f64::from(horizontal_middle_output_row) * 100.0 + 1.0)
                && column_filter_middle_right_value
                    == CalcValue::number(f64::from(horizontal_middle_output_row) * 100.0 + 3.0)
                && column_filter_last_left_value
                    == CalcValue::number(f64::from(horizontal_source_rows) * 100.0 + 1.0)
                && column_filter_last_right_value
                    == CalcValue::number(f64::from(horizontal_source_rows) * 100.0 + 3.0)
        )
    ]);
    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

pub(super) fn sequence_spill_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 1 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "sequence-spill-1m requires at least 1 column".to_string(),
        });
    }

    let anchor = address(1, 1);
    let middle_row = (options.rows / 2).max(1);
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    sheet.set_formula(
        anchor.clone(),
        GridFormulaCell::new(
            format!("=SEQUENCE({})", options.rows),
            format!("excel.grid.v1:sequence:R1C1:{}#", options.rows),
        ),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, committed) = sheet
        .recalculate_mark_all_dirty_compact_with_oxfml_and_commit_spill_publication(
            materialization_limit,
        )?;
    let recalc = &committed.recalc;
    let spill_commit = &committed.spill_commit;
    let first_value = valuation.read_cell(&address(1, 1)).computed;
    let middle_value = valuation.read_cell(&address(middle_row, 1)).computed;
    let last_value = valuation.read_cell(&address(options.rows, 1)).computed;
    let spill_fact = valuation.spill_facts().get(&anchor);
    let spill_extent_cell_count = spill_fact.map_or(0, |fact| fact.extent.cell_count());
    let committed_spill_fact_entries = sheet.spill_facts().len();
    let committed_epoch_anchors = sheet.spill_epoch_ledger().entries().len();

    Ok(json!({
        "counters": {
            "spill_extent_declared_cells": spill_extent_cell_count,
            "spill_facts_published": recalc.spill_facts_published,
            "spill_facts_blocked": recalc.spill_facts_blocked,
            "spill_ghost_cells_published": recalc.spill_ghost_cells_published,
            "spill_commit_previous_fact_entries": spill_commit.previous_spill_fact_entries,
            "spill_commit_committed_fact_entries": spill_commit.committed_spill_fact_entries,
            "spill_commit_anchors_added": spill_commit.ledger_update.anchors_added,
            "spill_commit_current_epoch_anchors": committed_epoch_anchors,
            "sheet_committed_spill_fact_entries": committed_spill_fact_entries,
            "computed_dense_value_regions": recalc.computed_dense_value_regions,
            "computed_dense_cells": valuation.dense_computed_cells(),
            "computed_dense_numeric_packed_cells": valuation.dense_computed_numeric_packed_cells(),
            "computed_sparse_cells": valuation.sparse_computed_cells(),
            "formula_cells": recalc.formula_cells,
            "formula_evaluations": recalc.formula_evaluations,
            "sample_first_value": calc_value_display_text(&first_value),
            "sample_middle_row": middle_row,
            "sample_middle_value": calc_value_display_text(&middle_value),
            "sample_last_value": calc_value_display_text(&last_value)
        },
        "register_assertions": [
            register_assertion(
                "P-23",
                "successful dynamic-array spill publication stores a 1M SEQUENCE payload as one dense computed region instead of sparse cells",
                recalc.spill_facts_published == 1
                    && recalc.spill_facts_blocked == 0
                    && recalc.spill_ghost_cells_published == usize::try_from(options.rows.saturating_sub(1)).unwrap_or(usize::MAX)
                    && spill_commit.previous_spill_fact_entries == 0
                    && spill_commit.committed_spill_fact_entries == 1
                    && spill_commit.ledger_update.anchors_added == 1
                    && committed_spill_fact_entries == 1
                    && committed_epoch_anchors == 1
                    && recalc.computed_dense_value_regions == 1
                    && valuation.dense_computed_cells() == u64::from(options.rows)
                    && valuation.dense_computed_numeric_packed_cells() == u64::from(options.rows)
                    && valuation.sparse_computed_cells() == 0
            ),
            register_assertion(
                "GRID-SEQUENCE-SPILL-1M",
                "a 1M-row SEQUENCE spill remains dense-region backed and preserves sampled values",
                spill_extent_cell_count == u64::from(options.rows)
                    && first_value == CalcValue::number(1.0)
                    && middle_value == CalcValue::number(f64::from(middle_row))
                    && last_value == CalcValue::number(f64::from(options.rows))
            )
        ]
    }))
}

pub(super) fn spill_blockage_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "spill-blockage-1m requires at least 2 columns".to_string(),
        });
    }

    let anchor = address(1, 1);
    let extent = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;

    let empty_sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let empty_report = empty_sheet.optimized_spill_blockage_probe_report(&anchor, &extent)?;

    let mut blocked_sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let far_blocker = address(options.rows, 1);
    blocked_sheet.set_literal(far_blocker.clone(), CalcValue::number(99.0))?;
    let blocked_report = blocked_sheet.optimized_spill_blockage_probe_report(&anchor, &extent)?;

    let empty_compact_probe_count =
        u64::try_from(empty_report.compact_blocker_probe_count()).unwrap_or(u64::MAX);
    let blocked_compact_probe_count =
        u64::try_from(blocked_report.compact_blocker_probe_count()).unwrap_or(u64::MAX);
    let empty_probe_ratio_micros = micros_ratio(
        empty_compact_probe_count,
        empty_report.naive_extent_cell_probe_floor,
    );
    let blocked_probe_ratio_micros = micros_ratio(
        blocked_compact_probe_count,
        blocked_report.naive_extent_cell_probe_floor,
    );

    Ok(json!({
        "counters": {
            "spill_extent_declared_cells": empty_report.extent_cell_count,
            "empty_naive_extent_cell_probe_floor": empty_report.naive_extent_cell_probe_floor,
            "empty_compact_blocker_probe_count": empty_compact_probe_count,
            "empty_probe_ratio_micros": empty_probe_ratio_micros,
            "empty_blocked": empty_report.blocked,
            "far_blocker_row": far_blocker.row,
            "far_blocker_naive_extent_cell_probe_floor": blocked_report.naive_extent_cell_probe_floor,
            "far_blocker_compact_blocker_probe_count": blocked_compact_probe_count,
            "far_blocker_sparse_point_candidates": blocked_report.sparse_point_candidates,
            "far_blocker_probe_ratio_micros": blocked_probe_ratio_micros,
            "far_blocker_blocked": blocked_report.blocked
        },
        "register_assertions": [
            register_assertion(
                "P-26",
                "spill blockage probes compact occupied candidates and never empty cells across a 1M-row intended extent",
                empty_report.extent_cell_count == u64::from(options.rows)
                    && empty_report.naive_extent_cell_probe_floor == u64::from(options.rows)
                    && empty_compact_probe_count == 0
                    && !empty_report.blocked
                    && blocked_report.naive_extent_cell_probe_floor == u64::from(options.rows)
                    && blocked_compact_probe_count == 1
                    && blocked_report.sparse_point_candidates == 1
                    && blocked_report.blocked
                    && blocked_compact_probe_count < blocked_report.naive_extent_cell_probe_floor
            ),
            register_assertion(
                "GRID-SPILL-BLOCKAGE-1M",
                "a far sparse blocker in a 1M-row spill extent is found by one compact probe instead of a row scan",
                blocked_report.extent_cell_count == u64::from(options.rows)
                    && blocked_report.blocked
                    && blocked_compact_probe_count < blocked_report.naive_extent_cell_probe_floor
            )
        ]
    }))
}
