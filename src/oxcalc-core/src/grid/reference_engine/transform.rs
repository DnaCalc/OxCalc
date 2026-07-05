//! Structural-edit reference transforms for the strict-excel-grid bind
//! profile: applying row/column insert and delete edits to a bound
//! reference, re-rendering it, and classifying the outcome. Internal to
//! the reference engine; relies on the parent module's parse/render
//! helpers and shared types via `use super::*`.

use super::*;

pub(super) fn transformed_parsed_qualifier(reference: &ExcelGridReference) -> Option<String> {
    match reference {
        ExcelGridReference::Cell {
            parsed_qualifier, ..
        }
        | ExcelGridReference::Area {
            parsed_qualifier, ..
        }
        | ExcelGridReference::WholeRow {
            parsed_qualifier, ..
        }
        | ExcelGridReference::WholeColumn {
            parsed_qualifier, ..
        }
        | ExcelGridReference::SpillAnchor {
            parsed_qualifier, ..
        }
        | ExcelGridReference::StructuredReference {
            parsed_qualifier, ..
        }
        | ExcelGridReference::Name {
            parsed_qualifier, ..
        }
        | ExcelGridReference::SheetSpan {
            parsed_qualifier, ..
        } => parsed_qualifier.clone(),
        ExcelGridReference::RefError { .. } => None,
    }
}

pub(super) fn decode_excel_grid_transform_payload(
    payload: &ProfilePayload,
) -> Option<ExcelGridReferenceTransformPayload> {
    if payload.payload_kind != EXCEL_GRID_STRUCTURAL_EDIT_PAYLOAD_KIND || payload.encoding != "json"
    {
        return None;
    }
    serde_json::from_str(&payload.data).ok()
}

/// The container (`workbook_id`, `sheet_id`) a bound grid reference targets, for
/// every reference variant. A `RefError` record has no live target, so it
/// reports `None` and stays `#REF!` regardless of edit.
fn reference_container(reference: &ExcelGridReference) -> Option<(&str, &str)> {
    match reference {
        ExcelGridReference::Cell {
            workbook_id,
            sheet_id,
            ..
        }
        | ExcelGridReference::Area {
            workbook_id,
            sheet_id,
            ..
        }
        | ExcelGridReference::WholeRow {
            workbook_id,
            sheet_id,
            ..
        }
        | ExcelGridReference::WholeColumn {
            workbook_id,
            sheet_id,
            ..
        }
        | ExcelGridReference::SpillAnchor {
            workbook_id,
            sheet_id,
            ..
        }
        | ExcelGridReference::StructuredReference {
            workbook_id,
            sheet_id,
            ..
        }
        | ExcelGridReference::Name {
            workbook_id,
            sheet_id,
            ..
        } => Some((workbook_id, sheet_id)),
        // A 3D sheet-span has two sheet endpoints, not one target sheet, so it
        // has no single container to report. In practice this arm is never hit:
        // `transform_excel_grid_reference` intercepts spans before any
        // container-based transform. Endpoint delete/shrink is R4.12.
        ExcelGridReference::SheetSpan { .. } | ExcelGridReference::RefError { .. } => None,
    }
}

/// Apply a sheet-deletion structural edit to a bound reference (W062 D2 §6,
/// contract V7). Strict-excel policy is `HardRefError`: a reference whose
/// *target sheet* is the deleted sheet becomes a hard `#REF!` — a destructive
/// [`ReferenceTransformOutcome::FullyInvalid`] transform carrying a `RefError`
/// record, from which D4's readout renders `#REF!`. A reference targeting any
/// *other* sheet is [`ReferenceTransformOutcome::Unchanged`]. There is no
/// heal-on-recreate: because the record itself is rewritten to `RefError`,
/// recreating a same-named sheet cannot resurrect it; only revision restore
/// (undo) brings back the pre-transform record.
fn transform_sheet_deletion(
    reference: &ExcelGridReference,
    original_record: &ProfileReferenceRecord,
    edit: &ExcelGridStructuralEdit,
) -> ReferenceTransformResult {
    // A record already `#REF!` stays `#REF!`.
    let Some((workbook_id, sheet_id)) = reference_container(reference) else {
        return ReferenceTransformResult {
            outcome: ReferenceTransformOutcome::FullyInvalid,
            reference: Some(original_record.clone()),
            diagnostics: Vec::new(),
        };
    };
    if !sheet_deletion_targets_reference_sheet(edit, workbook_id, sheet_id) {
        // References into surviving sheets are untouched by the deletion.
        return unchanged_transform(original_record);
    }
    // The target sheet is gone: destructive `#REF!` transform of the record.
    // No formula anchor is needed — `#REF!` renders without one.
    invalid_reference_transform(
        original_record,
        workbook_id,
        sheet_id,
        "reference target sheet deleted by structural edit",
        None,
    )
}

/// Apply a sheet-deletion structural edit to a bound 3D sheet-span reference
/// (`Sheet1:Sheet3!A1`, W062 D2 §6 / V7, R4.12) — the endpoint delete/shrink
/// transform.
///
/// Excel-faithful behavior (V7):
/// - Deleting a sheet **outside** the span's interval — no change.
/// - Deleting a **strictly interior** member sheet — no record rewrite; the
///   stored span simply re-expands against the new order (membership shrinks).
/// - Deleting an **endpoint** sheet — the endpoint moves to the nearest
///   surviving sheet that was inside the old span (the span shrinks, staying
///   valid). Both endpoints being the deleted sheet, or the span having no
///   surviving member at all, collapses to a destructive `#REF!`.
///
/// The nearest-surviving-interior computation needs the pre-deletion sheet
/// order, which a single reference record cannot carry — it rides in the
/// payload's [`ExcelGridReferenceTransformPayload::sheet_order_before_edit`]. If
/// that order is absent (empty), only the identity-resolvable terminal cases are
/// handled (a degenerate single-sheet span whose sole sheet is deleted collapses
/// to `#REF!`; otherwise the span is left `Unchanged`, deferring the shrink to
/// the closure-time re-expansion against the new order).
fn transform_sheet_span_deletion(
    original_record: &ProfileReferenceRecord,
    workbook_id: &str,
    start_sheet: &str,
    end_sheet: &str,
    target: &str,
    payload: &ExcelGridReferenceTransformPayload,
) -> ReferenceTransformResult {
    use crate::structural::fold_name_case_insensitive as fold;

    let edit = &payload.edit;
    if edit.workbook_id != workbook_id {
        return unchanged_transform(original_record);
    }
    let deleted = edit.sheet_id.as_str();
    let deletes_start = fold(deleted) == fold(start_sheet);
    let deletes_end = fold(deleted) == fold(end_sheet);

    // Degenerate single-sheet span whose only sheet is deleted ⇒ `#REF!`. This
    // is resolvable from identity alone (no order needed).
    if fold(start_sheet) == fold(end_sheet) {
        if deletes_start {
            return invalid_reference_transform(
                original_record,
                workbook_id,
                start_sheet,
                "3D sheet-span collapsed: its only member sheet was deleted",
                None,
            );
        }
        return unchanged_transform(original_record);
    }

    // Deleting a non-endpoint sheet: no record rewrite. Whether it was interior
    // to the span or entirely outside it, the stored endpoints are still valid
    // sheets and the span re-expands against the new order (interior membership
    // shrinks for free; an outside deletion changes nothing).
    if !deletes_start && !deletes_end {
        return unchanged_transform(original_record);
    }

    // An endpoint was deleted: shrink it to the nearest surviving sheet that was
    // inside the old span. This needs the pre-deletion order.
    let order = &payload.sheet_order_before_edit;
    if order.is_empty() {
        // No order available: cannot compute the shrink target. Leave the record
        // Unchanged; the closure-time re-expansion still shrinks membership, and
        // a spurious `#REF!` here would be a silently-wrong destructive rewrite.
        return unchanged_transform(original_record);
    }
    let position_of = |name: &str| order.iter().position(|sheet| fold(sheet) == fold(name));
    let (Some(start_pos), Some(end_pos)) = (position_of(start_sheet), position_of(end_sheet))
    else {
        // An endpoint already absent from the pre-edit order — leave Unchanged.
        return unchanged_transform(original_record);
    };
    let (low, high) = (start_pos.min(end_pos), start_pos.max(end_pos));

    // The surviving members of the old interval, excluding the deleted sheet, in
    // order. If none survive the span collapses to `#REF!`.
    let surviving: Vec<&String> = order[low..=high]
        .iter()
        .filter(|sheet| fold(sheet) != fold(deleted))
        .collect();
    let (Some(first), Some(last)) = (surviving.first(), surviving.last()) else {
        return invalid_reference_transform(
            original_record,
            workbook_id,
            start_sheet,
            "3D sheet-span collapsed: no member sheet survived the deletion",
            None,
        );
    };

    // The shrunk span keeps the same authored orientation (start before end in
    // sheet order maps to the low/high survivors). If `start_sheet` was the
    // lower endpoint, the new start is the first survivor and the new end the
    // last; if the author wrote the endpoints reversed, preserve that.
    let (new_start, new_end) = if start_pos <= end_pos {
        ((*first).clone(), (*last).clone())
    } else {
        ((*last).clone(), (*first).clone())
    };

    let shrunk = ExcelGridReference::SheetSpan {
        workbook_id: workbook_id.to_string(),
        start_sheet: new_start.clone(),
        end_sheet: new_end.clone(),
        target: target.to_string(),
        source_text: format!("{new_start}:{new_end}!{target}"),
        parsed_qualifier: None,
    };
    transformed_reference_result(
        original_record,
        shrunk,
        ReferenceTransformOutcome::Shifted,
        None,
    )
}

pub(super) fn transform_excel_grid_reference(
    reference: &ExcelGridReference,
    original_record: &ProfileReferenceRecord,
    payload: &ExcelGridReferenceTransformPayload,
    bounds: ExcelGridBounds,
) -> Result<ReferenceTransformResult, String> {
    // A 3D sheet-span reference (`Sheet1:Sheet3!A1`) is immune to axis (row/
    // column) edits — its endpoints are sheet identities and its target is
    // sheet-agnostic authored text, so a row/column insert/delete never shifts
    // it. The one edit that transforms a span is deleting a *sheet* (endpoint
    // delete/shrink, W062 D2 §6 / V7, R4.12): deleting an interior sheet needs
    // no record rewrite (the stored span re-expands against the new order, which
    // shrinks membership for free), but deleting an *endpoint* sheet must move
    // the endpoint to the nearest surviving sheet that was inside the old span,
    // and a span whose only member is the deleted sheet collapses to `#REF!`.
    if let ExcelGridReference::SheetSpan {
        workbook_id,
        start_sheet,
        end_sheet,
        target,
        ..
    } = reference
    {
        if matches!(payload.edit.kind, ExcelGridStructuralEditKind::SheetDeleted) {
            return Ok(transform_sheet_span_deletion(
                original_record,
                workbook_id,
                start_sheet,
                end_sheet,
                target,
                payload,
            ));
        }
        // Any non-deletion structural edit leaves the span untouched.
        return Ok(unchanged_transform(original_record));
    }

    // Sheet deletion is a container-level edit dispatched before the axis-based
    // per-reference transforms: every reference variant collapses to `#REF!`
    // when its target sheet is the deleted one, none needs axis/anchor math
    // (W062 D2 §6, contract V7).
    if matches!(payload.edit.kind, ExcelGridStructuralEditKind::SheetDeleted) {
        return Ok(transform_sheet_deletion(
            reference,
            original_record,
            &payload.edit,
        ));
    }

    let anchor_after = transformed_formula_anchor(payload, bounds)?;
    let anchor_before = payload.formula_anchor_before.as_ref();
    let anchor_after_ref = anchor_after.as_ref();

    match reference {
        ExcelGridReference::Cell {
            workbook_id,
            sheet_id,
            row,
            col,
            source_style,
            parsed_qualifier,
            ..
        } => {
            if !edit_targets_reference_sheet(&payload.edit, workbook_id, sheet_id) {
                return Ok(unchanged_transform(original_record));
            }
            let row_result = transform_reference_axis(
                *row,
                ExcelGridStructuralEditAxis::Row,
                anchor_before,
                anchor_after_ref,
                &payload.edit,
                bounds,
            )?;
            let col_result = transform_reference_axis(
                *col,
                ExcelGridStructuralEditAxis::Column,
                anchor_before,
                anchor_after_ref,
                &payload.edit,
                bounds,
            )?;
            let (Some(row), Some(col)) = (row_result.axis, col_result.axis) else {
                return Ok(invalid_reference_transform(
                    original_record,
                    workbook_id,
                    sheet_id,
                    "cell reference target deleted by structural edit",
                    anchor_after_ref,
                ));
            };
            let transformed = ExcelGridReference::Cell {
                workbook_id: workbook_id.clone(),
                sheet_id: sheet_id.clone(),
                row,
                col,
                source_style: *source_style,
                source_text: String::new(),
                parsed_qualifier: parsed_qualifier.clone(),
            };
            let outcome = combine_transform_outcomes(row_result.outcome, col_result.outcome);
            Ok(transformed_reference_result(
                original_record,
                transformed,
                outcome,
                anchor_after_ref,
            ))
        }
        ExcelGridReference::Area {
            workbook_id,
            sheet_id,
            start_row,
            start_col,
            end_row,
            end_col,
            source_style,
            parsed_qualifier,
            ..
        } => {
            if !edit_targets_reference_sheet(&payload.edit, workbook_id, sheet_id) {
                return Ok(unchanged_transform(original_record));
            }
            let row_result = transform_reference_axis_range(
                *start_row,
                *end_row,
                ExcelGridStructuralEditAxis::Row,
                anchor_before,
                anchor_after_ref,
                &payload.edit,
                bounds,
            )?;
            let col_result = transform_reference_axis_range(
                *start_col,
                *end_col,
                ExcelGridStructuralEditAxis::Column,
                anchor_before,
                anchor_after_ref,
                &payload.edit,
                bounds,
            )?;
            let (Some((start_row, end_row)), Some((start_col, end_col))) =
                (row_result.axes, col_result.axes)
            else {
                return Ok(invalid_reference_transform(
                    original_record,
                    workbook_id,
                    sheet_id,
                    "area reference target deleted by structural edit",
                    anchor_after_ref,
                ));
            };
            let transformed = ExcelGridReference::Area {
                workbook_id: workbook_id.clone(),
                sheet_id: sheet_id.clone(),
                start_row,
                start_col,
                end_row,
                end_col,
                source_style: *source_style,
                source_text: String::new(),
                parsed_qualifier: parsed_qualifier.clone(),
            };
            let outcome = combine_transform_outcomes(row_result.outcome, col_result.outcome);
            Ok(transformed_reference_result(
                original_record,
                transformed,
                outcome,
                anchor_after_ref,
            ))
        }
        ExcelGridReference::WholeRow {
            workbook_id,
            sheet_id,
            start_row,
            end_row,
            source_style,
            parsed_qualifier,
            ..
        } => {
            if !edit_targets_reference_sheet(&payload.edit, workbook_id, sheet_id)
                || payload.edit.axis != ExcelGridStructuralEditAxis::Row
            {
                return Ok(unchanged_transform(original_record));
            }
            let row_result = transform_reference_axis_range(
                *start_row,
                *end_row,
                ExcelGridStructuralEditAxis::Row,
                anchor_before,
                anchor_after_ref,
                &payload.edit,
                bounds,
            )?;
            let Some((start_row, end_row)) = row_result.axes else {
                return Ok(invalid_reference_transform(
                    original_record,
                    workbook_id,
                    sheet_id,
                    "whole-row reference target deleted by structural edit",
                    anchor_after_ref,
                ));
            };
            let transformed = ExcelGridReference::WholeRow {
                workbook_id: workbook_id.clone(),
                sheet_id: sheet_id.clone(),
                start_row,
                end_row,
                source_style: *source_style,
                source_text: String::new(),
                parsed_qualifier: parsed_qualifier.clone(),
            };
            Ok(transformed_reference_result(
                original_record,
                transformed,
                row_result.outcome,
                anchor_after_ref,
            ))
        }
        ExcelGridReference::WholeColumn {
            workbook_id,
            sheet_id,
            start_col,
            end_col,
            source_style,
            parsed_qualifier,
            ..
        } => {
            if !edit_targets_reference_sheet(&payload.edit, workbook_id, sheet_id)
                || payload.edit.axis != ExcelGridStructuralEditAxis::Column
            {
                return Ok(unchanged_transform(original_record));
            }
            let col_result = transform_reference_axis_range(
                *start_col,
                *end_col,
                ExcelGridStructuralEditAxis::Column,
                anchor_before,
                anchor_after_ref,
                &payload.edit,
                bounds,
            )?;
            let Some((start_col, end_col)) = col_result.axes else {
                return Ok(invalid_reference_transform(
                    original_record,
                    workbook_id,
                    sheet_id,
                    "whole-column reference target deleted by structural edit",
                    anchor_after_ref,
                ));
            };
            let transformed = ExcelGridReference::WholeColumn {
                workbook_id: workbook_id.clone(),
                sheet_id: sheet_id.clone(),
                start_col,
                end_col,
                source_style: *source_style,
                source_text: String::new(),
                parsed_qualifier: parsed_qualifier.clone(),
            };
            Ok(transformed_reference_result(
                original_record,
                transformed,
                col_result.outcome,
                anchor_after_ref,
            ))
        }
        ExcelGridReference::RefError { .. } => Ok(ReferenceTransformResult {
            outcome: ReferenceTransformOutcome::FullyInvalid,
            reference: Some(original_record.clone()),
            diagnostics: Vec::new(),
        }),
        ExcelGridReference::SpillAnchor { .. }
        | ExcelGridReference::StructuredReference { .. }
        | ExcelGridReference::Name { .. } => Ok(ReferenceTransformResult {
            outcome: ReferenceTransformOutcome::DynamicOrHostSensitive,
            reference: Some(original_record.clone()),
            diagnostics: vec![
                "strict grid structural transform for spill, name, and structured references requires host namespace/ledger context"
                    .to_string(),
            ],
        }),
        // Unreachable: spans are intercepted at the top of
        // `transform_excel_grid_reference` (endpoint delete/shrink is R4.12).
        // Kept for exhaustiveness and defends the same Unchanged outcome.
        ExcelGridReference::SheetSpan { .. } => Ok(unchanged_transform(original_record)),
    }
}

fn unchanged_transform(original_record: &ProfileReferenceRecord) -> ReferenceTransformResult {
    ReferenceTransformResult {
        outcome: ReferenceTransformOutcome::Unchanged,
        reference: Some(original_record.clone()),
        diagnostics: Vec::new(),
    }
}

fn transformed_reference_result(
    original_record: &ProfileReferenceRecord,
    mut reference: ExcelGridReference,
    outcome: ReferenceTransformOutcome,
    anchor_after: Option<&ExcelGridFormulaAnchor>,
) -> ReferenceTransformResult {
    let rendered = render_reference_for_channel(
        &reference,
        original_record.source_info.source_channel,
        anchor_after,
    )
    .unwrap_or_else(|| normal_form_key_for_reference(&original_record.profile_id, &reference).0);
    set_reference_source_text(&mut reference, rendered);
    let record = profile_record_for_transformed_reference(
        original_record,
        reference,
        ReferenceValidity::ValidAfterInstantiation,
        anchor_after,
    );
    ReferenceTransformResult {
        outcome,
        reference: Some(record),
        diagnostics: Vec::new(),
    }
}

fn invalid_reference_transform(
    original_record: &ProfileReferenceRecord,
    workbook_id: &str,
    sheet_id: &str,
    reason: &str,
    anchor_after: Option<&ExcelGridFormulaAnchor>,
) -> ReferenceTransformResult {
    let reference = ExcelGridReference::RefError {
        workbook_id: workbook_id.to_string(),
        sheet_id: sheet_id.to_string(),
        source_text: "#REF!".to_string(),
        reason: reason.to_string(),
    };
    let record = profile_record_for_transformed_reference(
        original_record,
        reference,
        ReferenceValidity::InvalidStatic,
        anchor_after,
    );
    ReferenceTransformResult {
        outcome: ReferenceTransformOutcome::FullyInvalid,
        reference: Some(record),
        diagnostics: Vec::new(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AxisTransformResult {
    axis: Option<ExcelGridAxisRef>,
    outcome: ReferenceTransformOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AxisRangeTransformResult {
    axes: Option<(ExcelGridAxisRef, ExcelGridAxisRef)>,
    outcome: ReferenceTransformOutcome,
}

fn transform_reference_axis(
    axis_ref: ExcelGridAxisRef,
    axis: ExcelGridStructuralEditAxis,
    anchor_before: Option<&ExcelGridFormulaAnchor>,
    anchor_after: Option<&ExcelGridFormulaAnchor>,
    edit: &ExcelGridStructuralEdit,
    bounds: ExcelGridBounds,
) -> Result<AxisTransformResult, String> {
    if edit.axis != axis {
        return Ok(AxisTransformResult {
            axis: Some(axis_ref),
            outcome: ReferenceTransformOutcome::Unchanged,
        });
    }
    let old_anchor = anchor_axis(anchor_before, axis)?;
    let new_anchor = anchor_axis(anchor_after, axis)?;
    let old_index = resolve_axis_ref(axis_ref, old_anchor, axis_max_for_edit_axis(axis, bounds))?;
    let Some(new_index) = transform_structural_axis_index(
        old_index,
        edit.kind,
        axis_max_for_edit_axis(axis, bounds),
    )?
    else {
        return Ok(AxisTransformResult {
            axis: None,
            outcome: ReferenceTransformOutcome::FullyInvalid,
        });
    };
    let transformed_axis = reencode_axis_ref(axis_ref, new_index, new_anchor);
    Ok(AxisTransformResult {
        axis: Some(transformed_axis),
        outcome: if new_index == old_index {
            ReferenceTransformOutcome::Unchanged
        } else {
            ReferenceTransformOutcome::Shifted
        },
    })
}

fn transform_reference_axis_range(
    start: ExcelGridAxisRef,
    end: ExcelGridAxisRef,
    axis: ExcelGridStructuralEditAxis,
    anchor_before: Option<&ExcelGridFormulaAnchor>,
    anchor_after: Option<&ExcelGridFormulaAnchor>,
    edit: &ExcelGridStructuralEdit,
    bounds: ExcelGridBounds,
) -> Result<AxisRangeTransformResult, String> {
    if edit.axis != axis {
        return Ok(AxisRangeTransformResult {
            axes: Some((start, end)),
            outcome: ReferenceTransformOutcome::Unchanged,
        });
    }
    let old_anchor = anchor_axis(anchor_before, axis)?;
    let new_anchor = anchor_axis(anchor_after, axis)?;
    let max = axis_max_for_edit_axis(axis, bounds);
    let start_index = resolve_axis_ref(start, old_anchor, max)?;
    let end_index = resolve_axis_ref(end, old_anchor, max)?;
    let Some((new_start, new_end, outcome)) = transform_structural_axis_range(
        start_index.min(end_index),
        start_index.max(end_index),
        edit.kind,
        max,
    )?
    else {
        return Ok(AxisRangeTransformResult {
            axes: None,
            outcome: ReferenceTransformOutcome::FullyInvalid,
        });
    };
    Ok(AxisRangeTransformResult {
        axes: Some((
            reencode_axis_ref(start, new_start, new_anchor),
            reencode_axis_ref(end, new_end, new_anchor),
        )),
        outcome,
    })
}

fn edit_targets_reference_sheet(
    edit: &ExcelGridStructuralEdit,
    workbook_id: &str,
    sheet_id: &str,
) -> bool {
    edit.workbook_id == workbook_id && edit.sheet_id == sheet_id
}

/// Sheet-deletion targeting: like [`edit_targets_reference_sheet`] but the sheet
/// component compares **case-insensitively**, with the same lowercase fold the
/// catalog's name routing uses (`NormalizedSheetName::from_symbol`). Excel sheet
/// names are case-insensitive, so a bound record whose authored qualifier is a
/// case variant of the deleted sheet's display name (`=sheet2!A1` vs `Sheet2`)
/// still targets the deleted sheet and must harden to `#REF!` (W062 D2 §6 / V7;
/// calc-5kqg.43 fresh-eyes M1: the consumer's selection pass routes names
/// case-insensitively, so the transform decision must fold identically or a
/// case-variant reference is selected but silently left un-rewritten — a
/// dangling live name that would heal on recreate, violating the contract).
/// Axis edits keep the exact-match helper: their sheet ids come from the same
/// engine-internal address the formula anchor carries, never from user-authored
/// qualifier text.
fn sheet_deletion_targets_reference_sheet(
    edit: &ExcelGridStructuralEdit,
    workbook_id: &str,
    sheet_id: &str,
) -> bool {
    use crate::structural::fold_name_case_insensitive as fold;
    edit.workbook_id == workbook_id && fold(&edit.sheet_id) == fold(sheet_id)
}

fn transformed_formula_anchor(
    payload: &ExcelGridReferenceTransformPayload,
    bounds: ExcelGridBounds,
) -> Result<Option<ExcelGridFormulaAnchor>, String> {
    if let Some(anchor_after) = &payload.formula_anchor_after {
        return Ok(Some(anchor_after.clone()));
    }
    let Some(anchor_before) = &payload.formula_anchor_before else {
        return Ok(None);
    };
    // A sheet deletion never shifts a cell along an axis: the formula anchor is
    // carried through unchanged (its own placement is untouched by removing a
    // *different* container; a formula ON the deleted sheet is removed by the
    // caller's structural edit, not re-anchored here). (W062 D2 §6.)
    if matches!(payload.edit.kind, ExcelGridStructuralEditKind::SheetDeleted) {
        return Ok(Some(anchor_before.clone()));
    }
    if anchor_before.workbook_id != payload.edit.workbook_id
        || anchor_before.sheet_id != payload.edit.sheet_id
    {
        return Ok(Some(anchor_before.clone()));
    }
    let max = axis_max_for_edit_axis(payload.edit.axis, bounds);
    let index = match payload.edit.axis {
        ExcelGridStructuralEditAxis::Row => anchor_before.row,
        ExcelGridStructuralEditAxis::Column => anchor_before.col,
    };
    let Some(new_index) = transform_structural_axis_index(index, payload.edit.kind, max)? else {
        return Err("formula anchor is deleted by structural edit".to_string());
    };
    let mut anchor_after = anchor_before.clone();
    match payload.edit.axis {
        ExcelGridStructuralEditAxis::Row => anchor_after.row = new_index,
        ExcelGridStructuralEditAxis::Column => anchor_after.col = new_index,
    }
    Ok(Some(anchor_after))
}

fn anchor_axis(
    anchor: Option<&ExcelGridFormulaAnchor>,
    axis: ExcelGridStructuralEditAxis,
) -> Result<u32, String> {
    let Some(anchor) = anchor else {
        return Err(
            "strict grid structural transform requires formula anchor context for relative axes"
                .to_string(),
        );
    };
    Ok(match axis {
        ExcelGridStructuralEditAxis::Row => anchor.row,
        ExcelGridStructuralEditAxis::Column => anchor.col,
    })
}

fn resolve_axis_ref(axis_ref: ExcelGridAxisRef, anchor: u32, max: u32) -> Result<u32, String> {
    match axis_ref {
        ExcelGridAxisRef::Absolute(index) => {
            if 1 <= index && index <= max {
                Ok(index)
            } else {
                Err(format!("absolute grid axis {index} is outside 1..={max}"))
            }
        }
        ExcelGridAxisRef::Relative(delta) => {
            let resolved = i64::from(anchor) + i64::from(delta);
            if 1 <= resolved && resolved <= i64::from(max) {
                Ok(u32::try_from(resolved).expect("validated grid axis fits u32"))
            } else {
                Err(format!(
                    "relative grid axis R/C[{delta}] from anchor {anchor} is outside 1..={max}"
                ))
            }
        }
    }
}

fn reencode_axis_ref(
    original: ExcelGridAxisRef,
    new_index: u32,
    new_anchor: u32,
) -> ExcelGridAxisRef {
    match original {
        ExcelGridAxisRef::Absolute(_) => ExcelGridAxisRef::Absolute(new_index),
        ExcelGridAxisRef::Relative(_) => {
            ExcelGridAxisRef::Relative(axis_delta(new_index, new_anchor))
        }
    }
}

fn transform_structural_axis_index(
    index: u32,
    kind: ExcelGridStructuralEditKind,
    max: u32,
) -> Result<Option<u32>, String> {
    match kind {
        // Sheet deletion has no axis band; it is dispatched before any axis
        // transform (W062 D2 §6) and never reaches here.
        ExcelGridStructuralEditKind::SheetDeleted => {
            Err("sheet-deletion edit has no axis transform".to_string())
        }
        ExcelGridStructuralEditKind::Insert { before, count } => {
            if index < before {
                return Ok(Some(index));
            }
            let Some(new_index) = index.checked_add(count) else {
                return Ok(None);
            };
            Ok((new_index <= max).then_some(new_index))
        }
        ExcelGridStructuralEditKind::Delete { first, count } => {
            let last = structural_delete_last(first, count)?;
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

fn transform_structural_axis_range(
    start: u32,
    end: u32,
    kind: ExcelGridStructuralEditKind,
    max: u32,
) -> Result<Option<(u32, u32, ReferenceTransformOutcome)>, String> {
    match kind {
        // Sheet deletion has no axis band; it is dispatched before any axis
        // transform (W062 D2 §6) and never reaches here.
        ExcelGridStructuralEditKind::SheetDeleted => {
            Err("sheet-deletion edit has no axis transform".to_string())
        }
        ExcelGridStructuralEditKind::Insert { before, count } => {
            if before > end {
                return Ok(Some((start, end, ReferenceTransformOutcome::Unchanged)));
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
                    ReferenceTransformOutcome::Shrunk
                } else {
                    ReferenceTransformOutcome::Shifted
                };
                return Ok(Some((new_start, new_end, outcome)));
            }

            let unclipped_end = end.saturating_add(count);
            let new_end = unclipped_end.min(max);
            let outcome = if new_end > end {
                ReferenceTransformOutcome::Expanded
            } else {
                ReferenceTransformOutcome::Unchanged
            };
            Ok(Some((start, new_end, outcome)))
        }
        ExcelGridStructuralEditKind::Delete { first, count } => {
            let last = structural_delete_last(first, count)?;
            if last < start {
                return Ok(Some((
                    start - count,
                    end - count,
                    ReferenceTransformOutcome::Shifted,
                )));
            }
            if first > end {
                return Ok(Some((start, end, ReferenceTransformOutcome::Unchanged)));
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
                ReferenceTransformOutcome::Shrunk,
            )))
        }
    }
}

pub(super) fn validate_structural_edit(
    edit: &ExcelGridStructuralEdit,
    bounds: ExcelGridBounds,
) -> Result<(), String> {
    // A sheet deletion is a container-level edit with no axis band to validate:
    // the target sheet is `edit.sheet_id` and the axis field is a don't-care
    // (W062 D2 §6, contract V7). Nothing to bound-check.
    if matches!(edit.kind, ExcelGridStructuralEditKind::SheetDeleted) {
        return Ok(());
    }
    let max = axis_max_for_edit_axis(edit.axis, bounds);
    match edit.kind {
        // Handled by the early return above; unreachable here but kept for
        // match exhaustiveness.
        ExcelGridStructuralEditKind::SheetDeleted => {}
        ExcelGridStructuralEditKind::Insert { before, count } => {
            if count == 0 || before == 0 || before > max.saturating_add(1) {
                return Err(format!(
                    "insert {:?} before {before} count {count} outside 1..={}",
                    edit.axis,
                    max.saturating_add(1)
                ));
            }
        }
        ExcelGridStructuralEditKind::Delete { first, count } => {
            if count == 0 || first == 0 {
                return Err(format!(
                    "delete {:?} first {first} count {count} is invalid",
                    edit.axis
                ));
            }
            let last = structural_delete_last(first, count)?;
            if first > max || last > max {
                return Err(format!(
                    "delete {:?} first {first} count {count} outside 1..={max}",
                    edit.axis
                ));
            }
        }
    }
    Ok(())
}

fn structural_delete_last(first: u32, count: u32) -> Result<u32, String> {
    first
        .checked_add(count.saturating_sub(1))
        .ok_or_else(|| format!("delete first {first} count {count} overflows axis"))
}

const fn axis_max_for_edit_axis(axis: ExcelGridStructuralEditAxis, bounds: ExcelGridBounds) -> u32 {
    match axis {
        ExcelGridStructuralEditAxis::Row => bounds.max_rows,
        ExcelGridStructuralEditAxis::Column => bounds.max_cols,
    }
}

fn combine_transform_outcomes(
    left: ReferenceTransformOutcome,
    right: ReferenceTransformOutcome,
) -> ReferenceTransformOutcome {
    use ReferenceTransformOutcome::{
        DynamicOrHostSensitive, Expanded, FullyInvalid, GeometryCoupledOpaqueConflict,
        PartiallyInvalid, Shifted, Shrunk, Split, Unchanged, Unsupported,
    };
    for candidate in [
        GeometryCoupledOpaqueConflict,
        Unsupported,
        FullyInvalid,
        Split,
        PartiallyInvalid,
        DynamicOrHostSensitive,
        Expanded,
        Shrunk,
        Shifted,
    ] {
        if left == candidate || right == candidate {
            return candidate;
        }
    }
    Unchanged
}

fn set_reference_source_text(reference: &mut ExcelGridReference, text: String) {
    match reference {
        ExcelGridReference::Cell { source_text, .. }
        | ExcelGridReference::Area { source_text, .. }
        | ExcelGridReference::WholeRow { source_text, .. }
        | ExcelGridReference::WholeColumn { source_text, .. }
        | ExcelGridReference::SpillAnchor { source_text, .. }
        | ExcelGridReference::StructuredReference { source_text, .. }
        | ExcelGridReference::Name { source_text, .. }
        | ExcelGridReference::SheetSpan { source_text, .. }
        | ExcelGridReference::RefError { source_text, .. } => *source_text = text,
    }
}
