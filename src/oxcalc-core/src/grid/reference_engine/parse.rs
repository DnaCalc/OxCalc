//! A1/R1C1 reference parsing and rendering for the strict-excel-grid
//! profile: parsing cell/range atoms and axis fragments, the
//! R1C1-relative normal-form key, channel rendering, defined-name keys,
//! and column<->index conversions. Internal to the reference engine;
//! shares the parent module's types via `use super::*`.

use super::*;

pub(super) fn parse_a1_cell_reference(
    atom_text: &str,
    request: &ReferenceAtomBindRequest,
    bounds: ExcelGridBounds,
) -> Option<ParsedExcelGridAtom> {
    let mut rest = atom_text;
    let col_absolute = rest.starts_with('$');
    if col_absolute {
        rest = &rest[1..];
    }

    let col_len = rest
        .chars()
        .take_while(|ch| ch.is_ascii_alphabetic())
        .map(char::len_utf8)
        .sum::<usize>();
    if col_len == 0 {
        return None;
    }
    let col_text = &rest[..col_len];
    rest = &rest[col_len..];

    let row_absolute = rest.starts_with('$');
    if row_absolute {
        rest = &rest[1..];
    }

    let row_len = rest
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .map(char::len_utf8)
        .sum::<usize>();
    if row_len == 0 || row_len != rest.len() {
        return None;
    }
    let row_text = &rest[..row_len];
    let Some(col_index) = column_to_index(col_text) else {
        return Some(ParsedExcelGridAtom::InvalidStatic(format!(
            "A1 column '{col_text}' is outside strict Excel grid bounds"
        )));
    };
    let Ok(row_index) = row_text.parse::<u32>() else {
        return Some(ParsedExcelGridAtom::InvalidStatic(format!(
            "A1 row '{row_text}' is outside strict Excel grid bounds"
        )));
    };
    if !bounds.contains_row(row_index) || !bounds.contains_col(col_index) {
        return Some(ParsedExcelGridAtom::InvalidStatic(format!(
            "A1 reference '{atom_text}' is outside strict Excel grid bounds {}x{}",
            bounds.max_rows, bounds.max_cols
        )));
    }

    let row = if row_absolute {
        ExcelGridAxisRef::Absolute(row_index)
    } else {
        ExcelGridAxisRef::Relative(axis_delta(row_index, request.caller_row))
    };
    let col = if col_absolute {
        ExcelGridAxisRef::Absolute(col_index)
    } else {
        ExcelGridAxisRef::Relative(axis_delta(col_index, request.caller_col))
    };

    Some(ParsedExcelGridAtom::Bound(
        ExcelGridReference::Cell {
            workbook_id: request.workbook_id.clone(),
            sheet_id: request.sheet_id.clone(),
            row,
            col,
            source_style: ExcelGridReferenceStyle::A1,
            source_text: request.source_text.clone(),
            parsed_qualifier: request.parsed_qualifier.clone(),
        },
        ReferenceValidity::ValidAfterInstantiation,
    ))
}

pub(super) fn parse_r1c1_cell_reference(
    atom_text: &str,
    request: &ReferenceAtomBindRequest,
    bounds: ExcelGridBounds,
) -> Option<ParsedExcelGridAtom> {
    let (row, rest) = parse_r1c1_axis(atom_text, 'R')?;
    let (col, rest) = parse_r1c1_axis(rest, 'C')?;
    if !rest.is_empty() {
        return None;
    }

    if let ExcelGridAxisRef::Absolute(row) = row
        && !bounds.contains_row(row)
    {
        return Some(ParsedExcelGridAtom::InvalidStatic(format!(
            "R1C1 row '{row}' is outside strict Excel grid bounds"
        )));
    }
    if let ExcelGridAxisRef::Absolute(col) = col
        && !bounds.contains_col(col)
    {
        return Some(ParsedExcelGridAtom::InvalidStatic(format!(
            "R1C1 column '{col}' is outside strict Excel grid bounds"
        )));
    }

    let validity = if axis_valid_for_current_placement(row, request.caller_row, bounds.max_rows)
        && axis_valid_for_current_placement(col, request.caller_col, bounds.max_cols)
    {
        ReferenceValidity::ValidAfterInstantiation
    } else {
        ReferenceValidity::InvalidForCurrentPlacement
    };

    Some(ParsedExcelGridAtom::Bound(
        ExcelGridReference::Cell {
            workbook_id: request.workbook_id.clone(),
            sheet_id: request.sheet_id.clone(),
            row,
            col,
            source_style: ExcelGridReferenceStyle::R1C1,
            source_text: request.source_text.clone(),
            parsed_qualifier: request.parsed_qualifier.clone(),
        },
        validity,
    ))
}

/// The `{workbook}` component a bound range record carries (W062 D2 §5/§10,
/// R3.7). For an ordinary in-workbook range this is the caller's own workbook
/// id; for an **external** range (`[Book2]Sheet1!A:A`) the endpoints carry an
/// `external_target_id` (the bracket alias), and the component becomes the
/// dormant-external identity token `extbook:{normalized_alias}` — the §10
/// additive shape that keeps the key case-stable and rename/path-immune while
/// the sibling is unloaded. The alias catalog (not the key) is what a later
/// sibling load routes against.
///
/// Both endpoints share one qualifier (OxFml's `harmonize_simple_reference_fragments`
/// only emits a range for external endpoints when the bracket aliases match),
/// so keying off `left` is sufficient; `right` carries the identical alias.
pub(super) fn range_workbook_component(request: &ReferenceRangeBindRequest) -> String {
    match &request.left.external_target_id {
        Some(alias) => ExternalBookToken::from_alias(alias).as_str().to_string(),
        None => request.workbook_id.clone(),
    }
}

pub(super) fn parse_a1_whole_axis_range_reference(
    request: &ReferenceRangeBindRequest,
    bounds: ExcelGridBounds,
) -> Option<ParsedExcelGridAtom> {
    let sheet_id = range_sheet_id(request)?;
    let workbook_id = range_workbook_component(request);
    let left_row =
        parse_a1_row_axis_fragment(&request.left.target_text, request.caller_row, bounds);
    let right_row =
        parse_a1_row_axis_fragment(&request.right.target_text, request.caller_row, bounds);
    if let Some(parsed) = whole_row_range_reference(
        request,
        workbook_id.as_str(),
        sheet_id.as_str(),
        left_row,
        right_row,
        ExcelGridReferenceStyle::A1,
        bounds,
    ) {
        return Some(parsed);
    }

    let left_col =
        parse_a1_col_axis_fragment(&request.left.target_text, request.caller_col, bounds);
    let right_col =
        parse_a1_col_axis_fragment(&request.right.target_text, request.caller_col, bounds);
    whole_column_range_reference(
        request,
        workbook_id.as_str(),
        sheet_id.as_str(),
        left_col,
        right_col,
        ExcelGridReferenceStyle::A1,
        bounds,
    )
}

pub(super) fn parse_r1c1_whole_axis_range_reference(
    request: &ReferenceRangeBindRequest,
    bounds: ExcelGridBounds,
) -> Option<ParsedExcelGridAtom> {
    let sheet_id = range_sheet_id(request)?;
    let workbook_id = range_workbook_component(request);
    let left_row = parse_r1c1_row_axis_fragment(&request.left.target_text, bounds);
    let right_row = parse_r1c1_row_axis_fragment(&request.right.target_text, bounds);
    if let Some(parsed) = whole_row_range_reference(
        request,
        workbook_id.as_str(),
        sheet_id.as_str(),
        left_row,
        right_row,
        ExcelGridReferenceStyle::R1C1,
        bounds,
    ) {
        return Some(parsed);
    }

    let left_col = parse_r1c1_col_axis_fragment(&request.left.target_text, bounds);
    let right_col = parse_r1c1_col_axis_fragment(&request.right.target_text, bounds);
    whole_column_range_reference(
        request,
        workbook_id.as_str(),
        sheet_id.as_str(),
        left_col,
        right_col,
        ExcelGridReferenceStyle::R1C1,
        bounds,
    )
}

pub(super) fn whole_row_range_reference(
    request: &ReferenceRangeBindRequest,
    workbook_id: &str,
    sheet_id: &str,
    left: Option<ParsedExcelGridAxis>,
    right: Option<ParsedExcelGridAxis>,
    source_style: ExcelGridReferenceStyle,
    bounds: ExcelGridBounds,
) -> Option<ParsedExcelGridAtom> {
    match parsed_axis_pair(left, right)? {
        Ok((start_row, end_row)) => {
            let (start_row, end_row) = canonical_axis_pair(start_row, end_row, request.caller_row);
            Some(ParsedExcelGridAtom::Bound(
                ExcelGridReference::WholeRow {
                    workbook_id: workbook_id.to_string(),
                    sheet_id: sheet_id.to_string(),
                    start_row,
                    end_row,
                    source_style,
                    source_text: request.source_text.clone(),
                    parsed_qualifier: common_range_qualifier(request),
                },
                range_validity(
                    request,
                    start_row,
                    end_row,
                    request.caller_row,
                    bounds.max_rows,
                ),
            ))
        }
        Err(reason) => Some(ParsedExcelGridAtom::InvalidStatic(reason)),
    }
}

pub(super) fn whole_column_range_reference(
    request: &ReferenceRangeBindRequest,
    workbook_id: &str,
    sheet_id: &str,
    left: Option<ParsedExcelGridAxis>,
    right: Option<ParsedExcelGridAxis>,
    source_style: ExcelGridReferenceStyle,
    bounds: ExcelGridBounds,
) -> Option<ParsedExcelGridAtom> {
    match parsed_axis_pair(left, right)? {
        Ok((start_col, end_col)) => {
            let (start_col, end_col) = canonical_axis_pair(start_col, end_col, request.caller_col);
            Some(ParsedExcelGridAtom::Bound(
                ExcelGridReference::WholeColumn {
                    workbook_id: workbook_id.to_string(),
                    sheet_id: sheet_id.to_string(),
                    start_col,
                    end_col,
                    source_style,
                    source_text: request.source_text.clone(),
                    parsed_qualifier: common_range_qualifier(request),
                },
                range_validity(
                    request,
                    start_col,
                    end_col,
                    request.caller_col,
                    bounds.max_cols,
                ),
            ))
        }
        Err(reason) => Some(ParsedExcelGridAtom::InvalidStatic(reason)),
    }
}

/// The validity a bound whole-axis range record carries. For an **external**
/// range (`[Book2]Sheet1!A:A`) this is always
/// [`ReferenceValidity::DynamicOrHostSensitive`] (W062 D2 §5, R3.7): the
/// reference is valid-or-`#REF!` purely as a function of host/context state —
/// whether a sibling workspace is loaded under that alias — not of the axis
/// geometry, so it is never a static-placement fact. An ordinary in-workbook
/// range keeps its geometry-derived [`range_axis_validity`].
fn range_validity(
    request: &ReferenceRangeBindRequest,
    left: ExcelGridAxisRef,
    right: ExcelGridAxisRef,
    caller: u32,
    axis_max: u32,
) -> ReferenceValidity {
    if request.left.external_target_id.is_some() || request.right.external_target_id.is_some() {
        ReferenceValidity::DynamicOrHostSensitive
    } else {
        range_axis_validity(left, right, caller, axis_max)
    }
}

pub(super) fn canonical_axis_pair(
    left: ExcelGridAxisRef,
    right: ExcelGridAxisRef,
    caller: u32,
) -> (ExcelGridAxisRef, ExcelGridAxisRef) {
    if axis_resolved_for_order(left, caller) <= axis_resolved_for_order(right, caller) {
        (left, right)
    } else {
        (right, left)
    }
}

pub(super) fn axis_resolved_for_order(axis: ExcelGridAxisRef, caller: u32) -> i64 {
    match axis {
        ExcelGridAxisRef::Absolute(index) => i64::from(index),
        ExcelGridAxisRef::Relative(delta) => i64::from(caller) + i64::from(delta),
    }
}

pub(super) fn parsed_axis_pair(
    left: Option<ParsedExcelGridAxis>,
    right: Option<ParsedExcelGridAxis>,
) -> Option<Result<(ExcelGridAxisRef, ExcelGridAxisRef), String>> {
    match (left?, right?) {
        (ParsedExcelGridAxis::Bound(left), ParsedExcelGridAxis::Bound(right)) => {
            Some(Ok((left, right)))
        }
        (ParsedExcelGridAxis::InvalidStatic(reason), _)
        | (_, ParsedExcelGridAxis::InvalidStatic(reason)) => Some(Err(reason)),
    }
}

pub(super) fn parse_a1_row_axis_fragment(
    text: &str,
    caller_row: u32,
    bounds: ExcelGridBounds,
) -> Option<ParsedExcelGridAxis> {
    let (absolute, digits) = strip_optional_dollar(text);
    if digits.is_empty() || !digits.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let Ok(row_index) = digits.parse::<u32>() else {
        return Some(ParsedExcelGridAxis::InvalidStatic(format!(
            "A1 row '{text}' is outside strict Excel grid bounds"
        )));
    };
    if !bounds.contains_row(row_index) {
        return Some(ParsedExcelGridAxis::InvalidStatic(format!(
            "A1 row '{text}' is outside strict Excel grid bounds {}x{}",
            bounds.max_rows, bounds.max_cols
        )));
    }
    Some(ParsedExcelGridAxis::Bound(if absolute {
        ExcelGridAxisRef::Absolute(row_index)
    } else {
        ExcelGridAxisRef::Relative(axis_delta(row_index, caller_row))
    }))
}

pub(super) fn parse_a1_col_axis_fragment(
    text: &str,
    caller_col: u32,
    bounds: ExcelGridBounds,
) -> Option<ParsedExcelGridAxis> {
    let (absolute, letters) = strip_optional_dollar(text);
    if letters.is_empty() || !letters.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }
    let Some(col_index) = column_to_index(letters) else {
        return Some(ParsedExcelGridAxis::InvalidStatic(format!(
            "A1 column '{text}' is outside strict Excel grid bounds"
        )));
    };
    if !bounds.contains_col(col_index) {
        return Some(ParsedExcelGridAxis::InvalidStatic(format!(
            "A1 column '{text}' is outside strict Excel grid bounds {}x{}",
            bounds.max_rows, bounds.max_cols
        )));
    }
    Some(ParsedExcelGridAxis::Bound(if absolute {
        ExcelGridAxisRef::Absolute(col_index)
    } else {
        ExcelGridAxisRef::Relative(axis_delta(col_index, caller_col))
    }))
}

pub(super) fn parse_r1c1_row_axis_fragment(
    text: &str,
    bounds: ExcelGridBounds,
) -> Option<ParsedExcelGridAxis> {
    let (axis, rest) = parse_r1c1_axis(text, 'R')?;
    if !rest.is_empty() {
        return None;
    }
    validate_r1c1_axis_fragment(text, axis, bounds.max_rows, "row")
}

pub(super) fn parse_r1c1_col_axis_fragment(
    text: &str,
    bounds: ExcelGridBounds,
) -> Option<ParsedExcelGridAxis> {
    let (axis, rest) = parse_r1c1_axis(text, 'C')?;
    if !rest.is_empty() {
        return None;
    }
    validate_r1c1_axis_fragment(text, axis, bounds.max_cols, "column")
}

pub(super) fn validate_r1c1_axis_fragment(
    text: &str,
    axis: ExcelGridAxisRef,
    max: u32,
    axis_name: &str,
) -> Option<ParsedExcelGridAxis> {
    if let ExcelGridAxisRef::Absolute(index) = axis
        && !(1 <= index && index <= max)
    {
        return Some(ParsedExcelGridAxis::InvalidStatic(format!(
            "R1C1 {axis_name} '{text}' is outside strict Excel grid bounds"
        )));
    }
    Some(ParsedExcelGridAxis::Bound(axis))
}

pub(super) fn strip_optional_dollar(text: &str) -> (bool, &str) {
    text.strip_prefix('$')
        .map_or((false, text), |rest| (true, rest))
}

pub(super) fn range_axis_validity(
    start: ExcelGridAxisRef,
    end: ExcelGridAxisRef,
    caller: u32,
    max: u32,
) -> ReferenceValidity {
    if axis_valid_for_current_placement(start, caller, max)
        && axis_valid_for_current_placement(end, caller, max)
    {
        ReferenceValidity::ValidAfterInstantiation
    } else {
        ReferenceValidity::InvalidForCurrentPlacement
    }
}

pub(super) fn range_sheet_id(request: &ReferenceRangeBindRequest) -> Option<String> {
    (request.left.sheet_id == request.right.sheet_id).then(|| request.left.sheet_id.clone())
}

pub(super) fn common_range_qualifier(request: &ReferenceRangeBindRequest) -> Option<String> {
    match (
        &request.left.parsed_qualifier,
        &request.right.parsed_qualifier,
    ) {
        (Some(left), Some(right)) if left == right => Some(left.clone()),
        (Some(left), None) => Some(left.clone()),
        (None, Some(right)) => Some(right.clone()),
        _ => None,
    }
}

pub(super) fn parse_r1c1_axis(text: &str, axis_kind: char) -> Option<(ExcelGridAxisRef, &str)> {
    let rest = text.strip_prefix(axis_kind)?;
    if let Some(relative) = rest.strip_prefix('[') {
        let close_index = relative.find(']')?;
        let delta = relative[..close_index].parse::<i32>().ok()?;
        return Some((
            ExcelGridAxisRef::Relative(delta),
            &relative[close_index + 1..],
        ));
    }

    let digit_len = rest
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .map(char::len_utf8)
        .sum::<usize>();
    if digit_len == 0 {
        return Some((ExcelGridAxisRef::Relative(0), rest));
    }
    let absolute = rest[..digit_len].parse::<u32>().ok()?;
    Some((ExcelGridAxisRef::Absolute(absolute), &rest[digit_len..]))
}

pub(super) fn normal_form_key_for_reference(
    profile_id: &str,
    reference: &ExcelGridReference,
) -> ReferenceNormalFormKey {
    match reference {
        ExcelGridReference::Cell {
            workbook_id,
            sheet_id,
            row,
            col,
            ..
        } => ReferenceNormalFormKey(format!(
            "{profile_id}:cell:{}:{}:{}{}",
            key_component(workbook_id),
            key_component(sheet_id),
            axis_key("R", *row),
            axis_key("C", *col)
        )),
        ExcelGridReference::Area {
            workbook_id,
            sheet_id,
            start_row,
            start_col,
            end_row,
            end_col,
            ..
        } => ReferenceNormalFormKey(format!(
            "{profile_id}:area:{}:{}:{}{}:{}{}",
            key_component(workbook_id),
            key_component(sheet_id),
            axis_key("R", *start_row),
            axis_key("C", *start_col),
            axis_key("R", *end_row),
            axis_key("C", *end_col)
        )),
        ExcelGridReference::WholeRow {
            workbook_id,
            sheet_id,
            start_row,
            end_row,
            ..
        } => ReferenceNormalFormKey(format!(
            "{profile_id}:whole-row:{}:{}:{}:{}",
            key_component(workbook_id),
            key_component(sheet_id),
            axis_key("R", *start_row),
            axis_key("R", *end_row)
        )),
        ExcelGridReference::WholeColumn {
            workbook_id,
            sheet_id,
            start_col,
            end_col,
            ..
        } => ReferenceNormalFormKey(format!(
            "{profile_id}:whole-column:{}:{}:{}:{}",
            key_component(workbook_id),
            key_component(sheet_id),
            axis_key("C", *start_col),
            axis_key("C", *end_col)
        )),
        ExcelGridReference::SpillAnchor {
            workbook_id,
            sheet_id,
            anchor_key,
            ..
        } => ReferenceNormalFormKey(format!(
            "{profile_id}:spill:{}:{}:{}",
            key_component(workbook_id),
            key_component(sheet_id),
            key_component(anchor_key)
        )),
        ExcelGridReference::StructuredReference {
            workbook_id,
            sheet_id,
            source_text,
            ..
        } => ReferenceNormalFormKey(format!(
            "{profile_id}:structured:{}:{}:{}",
            key_component(workbook_id),
            key_component(sheet_id),
            key_component(source_text)
        )),
        ExcelGridReference::Name {
            workbook_id,
            sheet_id,
            name,
            ..
        } => ReferenceNormalFormKey(format!(
            "{profile_id}:name:{}:{}:{}",
            key_component(workbook_id),
            key_component(sheet_id),
            key_component(name)
        )),
        // W062 D2 §10 additive shape:
        // `{profile}:sheetspan:{workbook}:{start_sheet}:{end_sheet}:{rect}`.
        // Endpoints are rename-immune identity tokens; `{rect}` is the
        // sheet-agnostic authored target text (§4.2 ignore-rule). The key
        // deliberately does NOT enumerate member sheets — expansion is
        // closure-time (R4.12) — so a sheet insert/move/delete inside the span
        // never touches the key.
        ExcelGridReference::SheetSpan {
            workbook_id,
            start_sheet,
            end_sheet,
            target,
            ..
        } => ReferenceNormalFormKey(format!(
            "{profile_id}:sheetspan:{}:{}:{}:{}",
            key_component(workbook_id),
            key_component(start_sheet),
            key_component(end_sheet),
            key_component(target)
        )),
        ExcelGridReference::RefError {
            workbook_id,
            sheet_id,
            source_text,
            reason,
        } => ReferenceNormalFormKey(format!(
            "{profile_id}:ref-error:{}:{}:{}:{}",
            key_component(workbook_id),
            key_component(sheet_id),
            key_component(source_text),
            key_component(reason)
        )),
    }
}

pub(super) fn render_reference_for_channel(
    reference: &ExcelGridReference,
    channel: FormulaChannelKind,
    anchor: Option<&ExcelGridFormulaAnchor>,
) -> Option<String> {
    match reference {
        ExcelGridReference::Cell {
            row,
            col,
            source_style,
            parsed_qualifier,
            ..
        } => render_cell_reference(*row, *col, *source_style, channel, parsed_qualifier, anchor),
        ExcelGridReference::Area {
            start_row,
            start_col,
            end_row,
            end_col,
            source_style,
            parsed_qualifier,
            ..
        } => {
            let start = render_cell_reference(
                *start_row,
                *start_col,
                *source_style,
                channel,
                &None,
                anchor,
            )?;
            let end =
                render_cell_reference(*end_row, *end_col, *source_style, channel, &None, anchor)?;
            Some(with_optional_qualifier(
                parsed_qualifier.as_deref(),
                &format!("{start}:{end}"),
            ))
        }
        ExcelGridReference::WholeRow {
            start_row,
            end_row,
            source_style,
            parsed_qualifier,
            ..
        } => {
            let start = render_row_axis(*start_row, *source_style, channel, anchor)?;
            let end = render_row_axis(*end_row, *source_style, channel, anchor)?;
            Some(with_optional_qualifier(
                parsed_qualifier.as_deref(),
                &format!("{start}:{end}"),
            ))
        }
        ExcelGridReference::WholeColumn {
            start_col,
            end_col,
            source_style,
            parsed_qualifier,
            ..
        } => {
            let start = render_col_axis(*start_col, *source_style, channel, anchor)?;
            let end = render_col_axis(*end_col, *source_style, channel, anchor)?;
            Some(with_optional_qualifier(
                parsed_qualifier.as_deref(),
                &format!("{start}:{end}"),
            ))
        }
        ExcelGridReference::RefError { .. } => Some("#REF!".to_string()),
        ExcelGridReference::SpillAnchor { source_text, .. }
        | ExcelGridReference::StructuredReference { source_text, .. }
        | ExcelGridReference::Name { source_text, .. }
        // A 3D span round-trips its authored text: the sheet endpoints are
        // identity-token semantics, not axis geometry, so there is no channel
        // re-rendering to perform (mirrors Name/Structured). R4.12 owns any
        // future post-membership-change rerender.
        | ExcelGridReference::SheetSpan { source_text, .. } => Some(source_text.clone()),
    }
}

pub(super) fn render_cell_reference(
    row: ExcelGridAxisRef,
    col: ExcelGridAxisRef,
    source_style: ExcelGridReferenceStyle,
    channel: FormulaChannelKind,
    qualifier: &Option<String>,
    anchor: Option<&ExcelGridFormulaAnchor>,
) -> Option<String> {
    let style = if channel == FormulaChannelKind::WorksheetR1C1 {
        ExcelGridReferenceStyle::R1C1
    } else {
        source_style
    };
    let local = match style {
        ExcelGridReferenceStyle::A1 => {
            let anchor = anchor?;
            format!(
                "{}{}",
                render_col_axis(col, style, channel, Some(anchor))?,
                render_row_axis(row, style, channel, Some(anchor))?
            )
        }
        ExcelGridReferenceStyle::R1C1 => {
            format!(
                "{}{}",
                render_row_axis(row, style, channel, anchor)?,
                render_col_axis(col, style, channel, anchor)?
            )
        }
    };
    Some(with_optional_qualifier(qualifier.as_deref(), &local))
}

pub(super) fn render_row_axis(
    row: ExcelGridAxisRef,
    source_style: ExcelGridReferenceStyle,
    channel: FormulaChannelKind,
    anchor: Option<&ExcelGridFormulaAnchor>,
) -> Option<String> {
    let style = if channel == FormulaChannelKind::WorksheetR1C1 {
        ExcelGridReferenceStyle::R1C1
    } else {
        source_style
    };
    match style {
        ExcelGridReferenceStyle::A1 => match row {
            ExcelGridAxisRef::Absolute(index) => Some(format!("${index}")),
            ExcelGridAxisRef::Relative(delta) => {
                let anchor = anchor?;
                Some((i64::from(anchor.row) + i64::from(delta)).to_string())
            }
        },
        ExcelGridReferenceStyle::R1C1 => Some(match row {
            ExcelGridAxisRef::Absolute(index) => format!("R{index}"),
            ExcelGridAxisRef::Relative(0) => "R".to_string(),
            ExcelGridAxisRef::Relative(delta) => format!("R[{delta}]"),
        }),
    }
}

pub(super) fn render_col_axis(
    col: ExcelGridAxisRef,
    source_style: ExcelGridReferenceStyle,
    channel: FormulaChannelKind,
    anchor: Option<&ExcelGridFormulaAnchor>,
) -> Option<String> {
    let style = if channel == FormulaChannelKind::WorksheetR1C1 {
        ExcelGridReferenceStyle::R1C1
    } else {
        source_style
    };
    match style {
        ExcelGridReferenceStyle::A1 => match col {
            ExcelGridAxisRef::Absolute(index) => Some(format!("${}", index_to_column(index)?)),
            ExcelGridAxisRef::Relative(delta) => {
                let anchor = anchor?;
                let index = u32::try_from(i64::from(anchor.col) + i64::from(delta)).ok()?;
                index_to_column(index)
            }
        },
        ExcelGridReferenceStyle::R1C1 => Some(match col {
            ExcelGridAxisRef::Absolute(index) => format!("C{index}"),
            ExcelGridAxisRef::Relative(0) => "C".to_string(),
            ExcelGridAxisRef::Relative(delta) => format!("C[{delta}]"),
        }),
    }
}

pub(super) fn with_optional_qualifier(qualifier: Option<&str>, local: &str) -> String {
    qualifier.map_or_else(
        || local.to_string(),
        |qualifier| format!("{qualifier}!{local}"),
    )
}

pub(super) fn key_component(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'_' | b'-' => {
                escaped.push(byte as char);
            }
            _ => escaped.push_str(&format!("%{byte:02X}")),
        }
    }
    escaped
}

pub(super) fn axis_key(prefix: &str, axis: ExcelGridAxisRef) -> String {
    match axis {
        ExcelGridAxisRef::Absolute(index) => format!("{prefix}{index}"),
        ExcelGridAxisRef::Relative(delta) => format!("{prefix}[{delta}]"),
    }
}

pub(super) fn atom_text_without_qualifier(source_text: &str) -> &str {
    source_text
        .rsplit_once('!')
        .map_or(source_text, |(_, atom)| atom)
}

#[must_use]
pub fn excel_grid_defined_name_key(name: &str, bounds: ExcelGridBounds) -> Option<String> {
    let name = name.trim();
    if name.is_empty()
        || name.contains('!')
        || name.contains(':')
        || name.contains(' ')
        || looks_like_a1_reference_name(name)
        || parse_textual_a1_point(name, bounds).is_some()
        || looks_like_r1c1_reference_name(name)
    {
        return None;
    }

    let mut chars = name.chars();
    let first = chars.next()?;
    if !(first.is_ascii_alphabetic() || first == '_' || first == '\\') {
        return None;
    }
    if !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '.') {
        return None;
    }

    let uppercase = name.to_ascii_uppercase();
    if matches!(uppercase.as_str(), "TRUE" | "FALSE") {
        return None;
    }

    Some(uppercase)
}

#[must_use]
pub fn excel_grid_sheet_defined_name_key(
    workbook_id: &str,
    sheet_id: &str,
    name: &str,
    bounds: ExcelGridBounds,
) -> Option<String> {
    let name_key = excel_grid_defined_name_key(name, bounds)?;
    Some(format!(
        "{}:scoped-name:{}:{}:{}",
        EXCEL_GRID_PROFILE_ID,
        key_component(workbook_id),
        key_component(sheet_id),
        key_component(&name_key)
    ))
}

#[must_use]
pub fn excel_grid_defined_name_key_is_scoped(key: &str) -> bool {
    key.starts_with(&format!("{EXCEL_GRID_PROFILE_ID}:scoped-name:"))
}

#[must_use]
pub fn excel_grid_defined_name_seed_keys(
    name_or_key: &str,
    bounds: ExcelGridBounds,
) -> Option<Vec<String>> {
    let trimmed = name_or_key.trim();
    if excel_grid_defined_name_key_is_scoped(trimmed) {
        return Some(vec![trimmed.to_string()]);
    }
    excel_grid_defined_name_key(trimmed, bounds).map(|key| vec![key])
}

pub(super) fn looks_like_a1_reference_name(name: &str) -> bool {
    let mut rest = name.trim();
    if let Some(after_dollar) = rest.strip_prefix('$') {
        rest = after_dollar;
    }
    let col_len = rest
        .chars()
        .take_while(|ch| ch.is_ascii_alphabetic())
        .map(char::len_utf8)
        .sum::<usize>();
    if col_len == 0 {
        return false;
    }
    rest = &rest[col_len..];
    if let Some(after_dollar) = rest.strip_prefix('$') {
        rest = after_dollar;
    }
    let row_len = rest
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .map(char::len_utf8)
        .sum::<usize>();
    row_len > 0 && row_len == rest.len()
}

pub(super) fn looks_like_r1c1_reference_name(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    if matches!(upper.as_str(), "R" | "C" | "RC") {
        return true;
    }
    let Some(after_row) = upper.strip_prefix('R') else {
        return false;
    };
    let Some((row, col)) = after_row.split_once('C') else {
        return false;
    };
    (row.is_empty() || is_r1c1_axis_fragment(row)) && (col.is_empty() || is_r1c1_axis_fragment(col))
}

pub(super) fn is_r1c1_axis_fragment(fragment: &str) -> bool {
    if fragment.chars().all(|ch| ch.is_ascii_digit()) {
        return true;
    }
    let Some(inner) = fragment
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return false;
    };
    let digits = inner
        .strip_prefix('+')
        .or_else(|| inner.strip_prefix('-'))
        .unwrap_or(inner);
    !digits.is_empty() && digits.chars().all(|ch| ch.is_ascii_digit())
}

pub(super) fn axis_delta(target: u32, caller: u32) -> i32 {
    i32::try_from(i64::from(target) - i64::from(caller))
        .expect("strict Excel grid axis delta fits i32")
}

pub(super) fn axis_valid_for_current_placement(
    axis: ExcelGridAxisRef,
    caller: u32,
    max: u32,
) -> bool {
    match axis {
        ExcelGridAxisRef::Absolute(index) => 1 <= index && index <= max,
        ExcelGridAxisRef::Relative(delta) => {
            let resolved = i64::from(caller) + i64::from(delta);
            1 <= resolved && resolved <= i64::from(max)
        }
    }
}

pub(super) fn column_to_index(text: &str) -> Option<u32> {
    let mut result = 0u32;
    for ch in text.chars() {
        let upper = ch.to_ascii_uppercase();
        if !upper.is_ascii_alphabetic() {
            return None;
        }
        result = result
            .checked_mul(26)?
            .checked_add((upper as u32) - ('A' as u32) + 1)?;
    }
    Some(result)
}

pub(super) fn index_to_column(mut index: u32) -> Option<String> {
    if index == 0 {
        return None;
    }
    let mut chars = Vec::new();
    while index > 0 {
        index -= 1;
        chars.push(char::from_u32(u32::from(b'A') + (index % 26))?);
        index /= 26;
    }
    chars.reverse();
    Some(chars.into_iter().collect())
}
