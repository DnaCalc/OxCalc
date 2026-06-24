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

pub(super) fn transform_excel_grid_reference(
    reference: &ExcelGridReference,
    original_record: &ProfileReferenceRecord,
    payload: &ExcelGridReferenceTransformPayload,
    bounds: ExcelGridBounds,
) -> Result<ReferenceTransformResult, String> {
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
    let max = axis_max_for_edit_axis(edit.axis, bounds);
    match edit.kind {
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
        | ExcelGridReference::RefError { source_text, .. } => *source_text = text,
    }
}
