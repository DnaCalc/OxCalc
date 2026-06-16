#![forbid(unsafe_code)]

//! Strict Excel-grid reference bind profile for the OxFml/OxCalc profile seam.
//!
//! Provenance note: the behavioral authority order for this profile is Excel
//! observation first, public file/formula specifications second, and Foundation
//! guidance as the local architecture map. The grid bounds mirror
//! `docs/spec/core-engine/CORE_ENGINE_GRID_MODEL.md` Section 4.1 and the
//! Foundation reference-resolution doctrine in
//! `../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md`.

use std::collections::BTreeMap;

use oxfml_core::binding::{
    ProfilePayload, ProfileReferenceRecord, ProfileVersion, ReferenceAtomBindRequest,
    ReferenceAtomBindResult, ReferenceBindProfile, ReferenceDependencyEnvelope,
    ReferenceFingerprintPolicy, ReferenceNormalFormKey, ReferenceOperatorCapabilities,
    ReferencePolicy, ReferenceProfileFingerprint, ReferenceProfileFingerprintContext,
    ReferenceRangeBindRequest, ReferenceRangeBindResult, ReferenceSourceInfo, ReferenceValidity,
};
use oxfml_core::source::FormulaChannelKind;
use oxfunc_core::resolver::{
    CallerContext, ReferenceDereferenceRequest, ReferenceEnumerationRequest, ReferenceFacts,
    ReferenceFactsRequest, ReferenceResolutionError, ReferenceSystemError, ReferenceSystemProvider,
    ResolvedReferenceCell, ResolvedReferenceExtent, ResolvedReferenceValues,
    materialize_resolved_reference_values, reference_facts,
};
use oxfunc_core::value::{
    CalcValue, ExcelText, ReferenceDisplay, ReferenceHandle, ReferenceHandleId, ReferenceIdentity,
    ReferenceLike, ReferenceSystemId,
};
use serde::{Deserialize, Serialize};

pub const EXCEL_GRID_PROFILE_ID: &str = "excel.grid.v1";
pub const STRICT_EXCEL_GRID_PROFILE_ALIAS: &str = "strict-excel-grid";
pub const STRICT_EXCEL_MAX_ROWS: u32 = 1_048_576;
pub const STRICT_EXCEL_MAX_COLS: u32 = 16_384;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExcelGridBounds {
    pub max_rows: u32,
    pub max_cols: u32,
}

impl ExcelGridBounds {
    #[must_use]
    pub const fn strict_excel() -> Self {
        Self {
            max_rows: STRICT_EXCEL_MAX_ROWS,
            max_cols: STRICT_EXCEL_MAX_COLS,
        }
    }

    #[must_use]
    pub const fn contains_row(self, row: u32) -> bool {
        1 <= row && row <= self.max_rows
    }

    #[must_use]
    pub const fn contains_col(self, col: u32) -> bool {
        1 <= col && col <= self.max_cols
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExcelGridReferenceStyle {
    A1,
    R1C1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExcelGridAxisRef {
    Absolute(u32),
    Relative(i32),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExcelGridReference {
    Cell {
        workbook_id: String,
        sheet_id: String,
        row: ExcelGridAxisRef,
        col: ExcelGridAxisRef,
        source_style: ExcelGridReferenceStyle,
        source_text: String,
        parsed_qualifier: Option<String>,
    },
    Area {
        workbook_id: String,
        sheet_id: String,
        start_row: ExcelGridAxisRef,
        start_col: ExcelGridAxisRef,
        end_row: ExcelGridAxisRef,
        end_col: ExcelGridAxisRef,
        source_style: ExcelGridReferenceStyle,
        source_text: String,
        parsed_qualifier: Option<String>,
    },
    WholeRow {
        workbook_id: String,
        sheet_id: String,
        start_row: ExcelGridAxisRef,
        end_row: ExcelGridAxisRef,
        source_style: ExcelGridReferenceStyle,
        source_text: String,
        parsed_qualifier: Option<String>,
    },
    WholeColumn {
        workbook_id: String,
        sheet_id: String,
        start_col: ExcelGridAxisRef,
        end_col: ExcelGridAxisRef,
        source_style: ExcelGridReferenceStyle,
        source_text: String,
        parsed_qualifier: Option<String>,
    },
    SpillAnchor {
        workbook_id: String,
        sheet_id: String,
        anchor_key: String,
        source_text: String,
        parsed_qualifier: Option<String>,
    },
    StructuredReference {
        workbook_id: String,
        sheet_id: String,
        source_text: String,
        parsed_qualifier: Option<String>,
    },
    Name {
        workbook_id: String,
        sheet_id: String,
        name: String,
        source_text: String,
        parsed_qualifier: Option<String>,
    },
    RefError {
        workbook_id: String,
        sheet_id: String,
        source_text: String,
        reason: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrictExcelGridReferenceProfile {
    bounds: ExcelGridBounds,
}

impl Default for StrictExcelGridReferenceProfile {
    fn default() -> Self {
        Self::new()
    }
}

impl StrictExcelGridReferenceProfile {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            bounds: ExcelGridBounds::strict_excel(),
        }
    }

    #[must_use]
    pub const fn with_bounds(bounds: ExcelGridBounds) -> Self {
        Self { bounds }
    }

    #[must_use]
    pub const fn bounds(self) -> ExcelGridBounds {
        self.bounds
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExcelGridCellAddress {
    pub workbook_id: String,
    pub sheet_id: String,
    pub row: u32,
    pub col: u32,
}

impl ExcelGridCellAddress {
    #[must_use]
    pub fn new(
        workbook_id: impl Into<String>,
        sheet_id: impl Into<String>,
        row: u32,
        col: u32,
    ) -> Self {
        Self {
            workbook_id: workbook_id.into(),
            sheet_id: sheet_id.into(),
            row,
            col,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExcelGridReferenceSystemProvider {
    workbook_id: String,
    sheet_id: String,
    caller_row: u32,
    caller_col: u32,
    bounds: ExcelGridBounds,
    cells: BTreeMap<ExcelGridCellAddress, CalcValue>,
}

impl ExcelGridReferenceSystemProvider {
    #[must_use]
    pub fn new(
        workbook_id: impl Into<String>,
        sheet_id: impl Into<String>,
        caller_row: u32,
        caller_col: u32,
    ) -> Self {
        Self {
            workbook_id: workbook_id.into(),
            sheet_id: sheet_id.into(),
            caller_row,
            caller_col,
            bounds: ExcelGridBounds::strict_excel(),
            cells: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn with_bounds(mut self, bounds: ExcelGridBounds) -> Self {
        self.bounds = bounds;
        self
    }

    #[must_use]
    pub fn with_cell_value(
        mut self,
        workbook_id: impl Into<String>,
        sheet_id: impl Into<String>,
        row: u32,
        col: u32,
        value: CalcValue,
    ) -> Self {
        self.cells.insert(
            ExcelGridCellAddress::new(workbook_id, sheet_id, row, col),
            value,
        );
        self
    }

    fn cell_value(&self, address: &ExcelGridCellAddress) -> CalcValue {
        self.cells
            .get(address)
            .cloned()
            .unwrap_or_else(CalcValue::empty)
    }
}

impl ReferenceBindProfile for StrictExcelGridReferenceProfile {
    fn profile_id(&self) -> &str {
        EXCEL_GRID_PROFILE_ID
    }

    fn profile_version(&self) -> ProfileVersion {
        ProfileVersion::v1()
    }

    fn reference_policy(&self) -> ReferencePolicy {
        ReferencePolicy::ProfileSymbolic
    }

    fn fingerprint_policy(&self) -> ReferenceFingerprintPolicy {
        ReferenceFingerprintPolicy::IncludeCallerAnchor
    }

    fn fingerprint(
        &self,
        context: &ReferenceProfileFingerprintContext,
    ) -> ReferenceProfileFingerprint {
        ReferenceProfileFingerprint(format!(
            "{}:{}:book={}:sheet={}:bounds={}x{}:structure={}",
            self.profile_id(),
            self.profile_version().0,
            context.workbook_id,
            context.sheet_id,
            self.bounds.max_rows,
            self.bounds.max_cols,
            context.structure_context_version
        ))
    }

    fn operator_capabilities(&self) -> ReferenceOperatorCapabilities {
        ReferenceOperatorCapabilities {
            range: true,
            union: true,
            intersection: true,
            spill: true,
        }
    }

    fn bind_atom(&self, request: &ReferenceAtomBindRequest) -> ReferenceAtomBindResult {
        let atom_text = atom_text_without_qualifier(&request.source_text);
        let parsed = match request.source_channel {
            FormulaChannelKind::WorksheetR1C1 => {
                parse_r1c1_cell_reference(atom_text, request, self.bounds)
            }
            FormulaChannelKind::WorksheetA1
            | FormulaChannelKind::ConditionalFormatting
            | FormulaChannelKind::DataValidation => {
                parse_a1_cell_reference(atom_text, request, self.bounds)
            }
        };

        let Some(parsed) = parsed else {
            return ReferenceAtomBindResult::LegacyCompatibility;
        };

        match parsed {
            ParsedExcelGridAtom::Bound(reference, validity) => ReferenceAtomBindResult::Bound(
                profile_record_for_reference(self.profile_id(), request, reference, validity),
            ),
            ParsedExcelGridAtom::InvalidStatic(reason) => ReferenceAtomBindResult::Rejected {
                validity: ReferenceValidity::InvalidStatic,
                message: reason,
            },
        }
    }

    fn bind_range(&self, request: &ReferenceRangeBindRequest) -> ReferenceRangeBindResult {
        if request.left.external_target_id.is_some() || request.right.external_target_id.is_some() {
            return ReferenceRangeBindResult::LegacyCompatibility;
        }

        let parsed = match request.source_channel {
            FormulaChannelKind::WorksheetR1C1 => {
                parse_r1c1_whole_axis_range_reference(request, self.bounds)
            }
            FormulaChannelKind::WorksheetA1
            | FormulaChannelKind::ConditionalFormatting
            | FormulaChannelKind::DataValidation => {
                parse_a1_whole_axis_range_reference(request, self.bounds)
            }
        };

        let Some(parsed) = parsed else {
            return ReferenceRangeBindResult::LegacyCompatibility;
        };

        match parsed {
            ParsedExcelGridAtom::Bound(reference, validity) => ReferenceRangeBindResult::Bound(
                profile_record_for_range_reference(self.profile_id(), request, reference, validity),
            ),
            ParsedExcelGridAtom::InvalidStatic(reason) => ReferenceRangeBindResult::Rejected {
                validity: ReferenceValidity::InvalidStatic,
                message: reason,
            },
        }
    }

    fn dependency_hints(
        &self,
        reference: &ProfileReferenceRecord,
        _context: &ReferenceProfileFingerprintContext,
    ) -> ReferenceDependencyEnvelope {
        match reference.validity {
            ReferenceValidity::ValidNow | ReferenceValidity::ValidAfterInstantiation => {
                ReferenceDependencyEnvelope::Static {
                    profile_id: self.profile_id().to_string(),
                    dependency_key: reference.normal_form_key.0.clone(),
                }
            }
            ReferenceValidity::InvalidForCurrentPlacement => ReferenceDependencyEnvelope::Static {
                profile_id: self.profile_id().to_string(),
                dependency_key: format!("invalid-placement:{}", reference.normal_form_key.0),
            },
            ReferenceValidity::DynamicOrHostSensitive => ReferenceDependencyEnvelope::Dynamic {
                profile_id: self.profile_id().to_string(),
                request_key: reference.normal_form_key.0.clone(),
            },
            ReferenceValidity::InvalidStatic | ReferenceValidity::Unsupported => {
                ReferenceDependencyEnvelope::Unsupported {
                    reason: format!("reference validity is {:?}", reference.validity),
                }
            }
        }
    }
}

impl ReferenceSystemProvider for ExcelGridReferenceSystemProvider {
    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        let rect = self.reference_rect(&request.reference)?;
        if rect.row_count() == 1 && rect.col_count() == 1 {
            return Ok(self.cell_value(&ExcelGridCellAddress::new(
                rect.workbook_id,
                rect.sheet_id,
                rect.top_row,
                rect.left_col,
            )));
        }

        let values = self.resolved_values_for_rect(&rect)?;
        if values.declared_extent.declared_cell_count() > MAX_MATERIALIZED_GRID_CELLS {
            return Err(ReferenceResolutionError::ProviderFailure {
                detail: "excel_grid_reference_requires_sparse_enumeration".to_string(),
            });
        }
        materialize_resolved_reference_values(&values).map(CalcValue::array)
    }

    fn enumerate_values(
        &self,
        request: &ReferenceEnumerationRequest,
    ) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
        if request.reference.system.0 != EXCEL_GRID_PROFILE_ID {
            return Ok(None);
        }
        let rect = self.reference_rect(&request.reference)?;
        self.resolved_values_for_rect(&rect).map(Some)
    }

    fn facts(
        &self,
        request: &ReferenceFactsRequest,
    ) -> Result<ReferenceFacts, ReferenceSystemError> {
        Ok(reference_facts(&request.reference))
    }

    fn caller_context(&self) -> Option<CallerContext> {
        Some(CallerContext {
            prefix: Some(format!("{}!{}", self.workbook_id, self.sheet_id)),
            row: usize::try_from(self.caller_row).unwrap_or(usize::MAX),
            col: usize::try_from(self.caller_col).unwrap_or(usize::MAX),
        })
    }
}

const MAX_MATERIALIZED_GRID_CELLS: usize = 100_000;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExcelGridRect {
    workbook_id: String,
    sheet_id: String,
    top_row: u32,
    left_col: u32,
    bottom_row: u32,
    right_col: u32,
}

impl ExcelGridRect {
    fn row_count(&self) -> u32 {
        self.bottom_row - self.top_row + 1
    }

    fn col_count(&self) -> u32 {
        self.right_col - self.left_col + 1
    }

    fn contains(&self, address: &ExcelGridCellAddress) -> bool {
        address.workbook_id == self.workbook_id
            && address.sheet_id == self.sheet_id
            && self.top_row <= address.row
            && address.row <= self.bottom_row
            && self.left_col <= address.col
            && address.col <= self.right_col
    }
}

impl ExcelGridReferenceSystemProvider {
    fn reference_rect(
        &self,
        reference: &ReferenceLike,
    ) -> Result<ExcelGridRect, ReferenceResolutionError> {
        if reference.system.0 != EXCEL_GRID_PROFILE_ID {
            return Err(ReferenceResolutionError::UnresolvedReference {
                target: reference.target().to_string(),
            });
        }
        let Some(key) = opaque_reference_key(reference) else {
            return Err(ReferenceResolutionError::UnresolvedReference {
                target: reference.target().to_string(),
            });
        };
        parse_excel_grid_reference_key(&key, self).ok_or_else(|| {
            ReferenceResolutionError::UnresolvedReference {
                target: reference.target().to_string(),
            }
        })
    }

    fn resolved_values_for_rect(
        &self,
        rect: &ExcelGridRect,
    ) -> Result<ResolvedReferenceValues, ReferenceResolutionError> {
        let rows = usize::try_from(rect.row_count()).map_err(|_| {
            ReferenceResolutionError::ProviderFailure {
                detail: "excel_grid_reference_extent_overflow".to_string(),
            }
        })?;
        let cols = usize::try_from(rect.col_count()).map_err(|_| {
            ReferenceResolutionError::ProviderFailure {
                detail: "excel_grid_reference_extent_overflow".to_string(),
            }
        })?;
        let mut cells = self
            .cells
            .iter()
            .filter(|(address, _)| rect.contains(address))
            .map(|(address, value)| {
                let row = usize::try_from(address.row - rect.top_row + 1).unwrap_or(usize::MAX);
                let col = usize::try_from(address.col - rect.left_col + 1).unwrap_or(usize::MAX);
                ResolvedReferenceCell::new(row, col, value.clone())
            })
            .collect::<Vec<_>>();
        cells.sort_by_key(|cell| (cell.row, cell.col));
        Ok(ResolvedReferenceValues::new(
            ResolvedReferenceExtent::new(rows, cols),
            cells,
            Some(format!(
                "excel-grid:v1:{}:{}:R{}C{}:R{}C{}",
                key_component(&rect.workbook_id),
                key_component(&rect.sheet_id),
                rect.top_row,
                rect.left_col,
                rect.bottom_row,
                rect.right_col
            )),
        ))
    }
}

fn opaque_reference_key(reference: &ReferenceLike) -> Option<String> {
    match &reference.identity {
        ReferenceIdentity::Opaque(handle) => String::from_utf8(handle.id.bytes.clone()).ok(),
        _ => None,
    }
}

fn parse_excel_grid_reference_key(
    key: &str,
    provider: &ExcelGridReferenceSystemProvider,
) -> Option<ExcelGridRect> {
    let parts = key.split(':').collect::<Vec<_>>();
    if parts.first().copied()? != EXCEL_GRID_PROFILE_ID {
        return None;
    }
    match parts.as_slice() {
        [_, "cell", workbook_id, sheet_id, axes] => {
            let (row, rest) = parse_r1c1_axis(axes, 'R')?;
            let (col, rest) = parse_r1c1_axis(rest, 'C')?;
            if !rest.is_empty() {
                return None;
            }
            let row = instantiate_axis(row, provider.caller_row, provider.bounds.max_rows)?;
            let col = instantiate_axis(col, provider.caller_col, provider.bounds.max_cols)?;
            Some(ExcelGridRect {
                workbook_id: unkey_component(workbook_id)?,
                sheet_id: unkey_component(sheet_id)?,
                top_row: row,
                left_col: col,
                bottom_row: row,
                right_col: col,
            })
        }
        [_, "area", workbook_id, sheet_id, start_axes, end_axes] => {
            let (start_row, start_col) = instantiate_cell_axes(start_axes, provider)?;
            let (end_row, end_col) = instantiate_cell_axes(end_axes, provider)?;
            Some(ExcelGridRect {
                workbook_id: unkey_component(workbook_id)?,
                sheet_id: unkey_component(sheet_id)?,
                top_row: start_row.min(end_row),
                left_col: start_col.min(end_col),
                bottom_row: start_row.max(end_row),
                right_col: start_col.max(end_col),
            })
        }
        [_, "whole-row", workbook_id, sheet_id, start_row, end_row] => {
            let start_row = instantiate_axis_key(
                start_row,
                'R',
                provider.caller_row,
                provider.bounds.max_rows,
            )?;
            let end_row =
                instantiate_axis_key(end_row, 'R', provider.caller_row, provider.bounds.max_rows)?;
            Some(ExcelGridRect {
                workbook_id: unkey_component(workbook_id)?,
                sheet_id: unkey_component(sheet_id)?,
                top_row: start_row.min(end_row),
                left_col: 1,
                bottom_row: start_row.max(end_row),
                right_col: provider.bounds.max_cols,
            })
        }
        [_, "whole-column", workbook_id, sheet_id, start_col, end_col] => {
            let start_col = instantiate_axis_key(
                start_col,
                'C',
                provider.caller_col,
                provider.bounds.max_cols,
            )?;
            let end_col =
                instantiate_axis_key(end_col, 'C', provider.caller_col, provider.bounds.max_cols)?;
            Some(ExcelGridRect {
                workbook_id: unkey_component(workbook_id)?,
                sheet_id: unkey_component(sheet_id)?,
                top_row: 1,
                left_col: start_col.min(end_col),
                bottom_row: provider.bounds.max_rows,
                right_col: start_col.max(end_col),
            })
        }
        _ => None,
    }
}

fn instantiate_cell_axes(
    axes: &str,
    provider: &ExcelGridReferenceSystemProvider,
) -> Option<(u32, u32)> {
    let (row, rest) = parse_r1c1_axis(axes, 'R')?;
    let (col, rest) = parse_r1c1_axis(rest, 'C')?;
    if !rest.is_empty() {
        return None;
    }
    Some((
        instantiate_axis(row, provider.caller_row, provider.bounds.max_rows)?,
        instantiate_axis(col, provider.caller_col, provider.bounds.max_cols)?,
    ))
}

fn instantiate_axis_key(text: &str, axis_kind: char, caller: u32, max: u32) -> Option<u32> {
    let (axis, rest) = parse_r1c1_axis(text, axis_kind)?;
    if rest.is_empty() {
        instantiate_axis(axis, caller, max)
    } else {
        None
    }
}

fn instantiate_axis(axis: ExcelGridAxisRef, caller: u32, max: u32) -> Option<u32> {
    let resolved = match axis {
        ExcelGridAxisRef::Absolute(index) => i64::from(index),
        ExcelGridAxisRef::Relative(delta) => i64::from(caller) + i64::from(delta),
    };
    (1 <= resolved && resolved <= i64::from(max)).then(|| u32::try_from(resolved).ok())?
}

fn unkey_component(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut index = 0;
    let mut decoded = Vec::with_capacity(bytes.len());
    while index < bytes.len() {
        if bytes[index] != b'%' {
            decoded.push(bytes[index]);
            index += 1;
            continue;
        }
        if index + 2 >= bytes.len() {
            return None;
        }
        let high = hex_value(bytes[index + 1])?;
        let low = hex_value(bytes[index + 2])?;
        decoded.push((high << 4) | low);
        index += 3;
    }
    String::from_utf8(decoded).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[must_use]
pub fn decode_excel_grid_reference_payload(payload: &ProfilePayload) -> Option<ExcelGridReference> {
    if payload.payload_kind != "excel-grid-reference" || payload.encoding != "json" {
        return None;
    }
    serde_json::from_str(&payload.data).ok()
}

#[must_use]
pub fn excel_grid_reference_like_from_profile_record(
    record: &ProfileReferenceRecord,
) -> Option<ReferenceLike> {
    if record.profile_id != EXCEL_GRID_PROFILE_ID {
        return None;
    }
    let reference = decode_excel_grid_reference_payload(&record.profile_payload)?;
    if record.normal_form_key != normal_form_key_for_reference(&record.profile_id, &reference) {
        return None;
    }
    let display_text = record
        .render_hint
        .clone()
        .unwrap_or_else(|| record.normal_form_key.0.clone());
    Some(ReferenceLike::opaque(
        ReferenceSystemId(EXCEL_GRID_PROFILE_ID.to_string()),
        ReferenceHandle {
            id: ReferenceHandleId::from_bytes(record.normal_form_key.0.clone().into_bytes()),
        },
        Some(ReferenceDisplay {
            text: ExcelText::from_interop_assignment(&display_text),
        }),
    ))
}

enum ParsedExcelGridAtom {
    Bound(ExcelGridReference, ReferenceValidity),
    InvalidStatic(String),
}

enum ParsedExcelGridAxis {
    Bound(ExcelGridAxisRef),
    InvalidStatic(String),
}

fn profile_record_for_reference(
    profile_id: &str,
    request: &ReferenceAtomBindRequest,
    reference: ExcelGridReference,
    validity: ReferenceValidity,
) -> ProfileReferenceRecord {
    let normal_form_key = normal_form_key_for_reference(profile_id, &reference);
    let payload_data =
        serde_json::to_string(&reference).expect("excel grid reference payload serializes");
    ProfileReferenceRecord {
        profile_id: profile_id.to_string(),
        profile_version: ProfileVersion::v1(),
        source_info: ReferenceSourceInfo {
            source_channel: request.source_channel,
            source_span: request.source_span,
            source_text: request.source_text.clone(),
            parsed_qualifier: request.parsed_qualifier.clone(),
            address_fidelity: Some(request.source_text.clone()),
        },
        profile_payload: ProfilePayload {
            payload_kind: "excel-grid-reference".to_string(),
            encoding: "json".to_string(),
            data: payload_data,
        },
        normal_form_key,
        render_hint: Some(request.source_text.clone()),
        validity,
    }
}

fn profile_record_for_range_reference(
    profile_id: &str,
    request: &ReferenceRangeBindRequest,
    reference: ExcelGridReference,
    validity: ReferenceValidity,
) -> ProfileReferenceRecord {
    let normal_form_key = normal_form_key_for_reference(profile_id, &reference);
    let payload_data =
        serde_json::to_string(&reference).expect("excel grid reference payload serializes");
    ProfileReferenceRecord {
        profile_id: profile_id.to_string(),
        profile_version: ProfileVersion::v1(),
        source_info: ReferenceSourceInfo {
            source_channel: request.source_channel,
            source_span: request.source_span,
            source_text: request.source_text.clone(),
            parsed_qualifier: common_range_qualifier(request),
            address_fidelity: Some(request.source_text.clone()),
        },
        profile_payload: ProfilePayload {
            payload_kind: "excel-grid-reference".to_string(),
            encoding: "json".to_string(),
            data: payload_data,
        },
        normal_form_key,
        render_hint: Some(request.source_text.clone()),
        validity,
    }
}

fn parse_a1_cell_reference(
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

fn parse_r1c1_cell_reference(
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

fn parse_a1_whole_axis_range_reference(
    request: &ReferenceRangeBindRequest,
    bounds: ExcelGridBounds,
) -> Option<ParsedExcelGridAtom> {
    let sheet_id = range_sheet_id(request)?;
    let left_row =
        parse_a1_row_axis_fragment(&request.left.target_text, request.caller_row, bounds);
    let right_row =
        parse_a1_row_axis_fragment(&request.right.target_text, request.caller_row, bounds);
    if let Some(parsed) = whole_row_range_reference(
        request,
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
        sheet_id.as_str(),
        left_col,
        right_col,
        ExcelGridReferenceStyle::A1,
        bounds,
    )
}

fn parse_r1c1_whole_axis_range_reference(
    request: &ReferenceRangeBindRequest,
    bounds: ExcelGridBounds,
) -> Option<ParsedExcelGridAtom> {
    let sheet_id = range_sheet_id(request)?;
    let left_row = parse_r1c1_row_axis_fragment(&request.left.target_text, bounds);
    let right_row = parse_r1c1_row_axis_fragment(&request.right.target_text, bounds);
    if let Some(parsed) = whole_row_range_reference(
        request,
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
        sheet_id.as_str(),
        left_col,
        right_col,
        ExcelGridReferenceStyle::R1C1,
        bounds,
    )
}

fn whole_row_range_reference(
    request: &ReferenceRangeBindRequest,
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
                    workbook_id: request.workbook_id.clone(),
                    sheet_id: sheet_id.to_string(),
                    start_row,
                    end_row,
                    source_style,
                    source_text: request.source_text.clone(),
                    parsed_qualifier: common_range_qualifier(request),
                },
                range_axis_validity(start_row, end_row, request.caller_row, bounds.max_rows),
            ))
        }
        Err(reason) => Some(ParsedExcelGridAtom::InvalidStatic(reason)),
    }
}

fn whole_column_range_reference(
    request: &ReferenceRangeBindRequest,
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
                    workbook_id: request.workbook_id.clone(),
                    sheet_id: sheet_id.to_string(),
                    start_col,
                    end_col,
                    source_style,
                    source_text: request.source_text.clone(),
                    parsed_qualifier: common_range_qualifier(request),
                },
                range_axis_validity(start_col, end_col, request.caller_col, bounds.max_cols),
            ))
        }
        Err(reason) => Some(ParsedExcelGridAtom::InvalidStatic(reason)),
    }
}

fn canonical_axis_pair(
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

fn axis_resolved_for_order(axis: ExcelGridAxisRef, caller: u32) -> i64 {
    match axis {
        ExcelGridAxisRef::Absolute(index) => i64::from(index),
        ExcelGridAxisRef::Relative(delta) => i64::from(caller) + i64::from(delta),
    }
}

fn parsed_axis_pair(
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

fn parse_a1_row_axis_fragment(
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

fn parse_a1_col_axis_fragment(
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

fn parse_r1c1_row_axis_fragment(
    text: &str,
    bounds: ExcelGridBounds,
) -> Option<ParsedExcelGridAxis> {
    let (axis, rest) = parse_r1c1_axis(text, 'R')?;
    if !rest.is_empty() {
        return None;
    }
    validate_r1c1_axis_fragment(text, axis, bounds.max_rows, "row")
}

fn parse_r1c1_col_axis_fragment(
    text: &str,
    bounds: ExcelGridBounds,
) -> Option<ParsedExcelGridAxis> {
    let (axis, rest) = parse_r1c1_axis(text, 'C')?;
    if !rest.is_empty() {
        return None;
    }
    validate_r1c1_axis_fragment(text, axis, bounds.max_cols, "column")
}

fn validate_r1c1_axis_fragment(
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

fn strip_optional_dollar(text: &str) -> (bool, &str) {
    text.strip_prefix('$')
        .map_or((false, text), |rest| (true, rest))
}

fn range_axis_validity(
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

fn range_sheet_id(request: &ReferenceRangeBindRequest) -> Option<String> {
    (request.left.sheet_id == request.right.sheet_id).then(|| request.left.sheet_id.clone())
}

fn common_range_qualifier(request: &ReferenceRangeBindRequest) -> Option<String> {
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

fn parse_r1c1_axis(text: &str, axis_kind: char) -> Option<(ExcelGridAxisRef, &str)> {
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

fn normal_form_key_for_reference(
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

fn key_component(text: &str) -> String {
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

fn axis_key(prefix: &str, axis: ExcelGridAxisRef) -> String {
    match axis {
        ExcelGridAxisRef::Absolute(index) => format!("{prefix}{index}"),
        ExcelGridAxisRef::Relative(delta) => format!("{prefix}[{delta}]"),
    }
}

fn atom_text_without_qualifier(source_text: &str) -> &str {
    source_text
        .rsplit_once('!')
        .map_or(source_text, |(_, atom)| atom)
}

fn axis_delta(target: u32, caller: u32) -> i32 {
    i32::try_from(i64::from(target) - i64::from(caller))
        .expect("strict Excel grid axis delta fits i32")
}

fn axis_valid_for_current_placement(axis: ExcelGridAxisRef, caller: u32, max: u32) -> bool {
    match axis {
        ExcelGridAxisRef::Absolute(index) => 1 <= index && index <= max,
        ExcelGridAxisRef::Relative(delta) => {
            let resolved = i64::from(caller) + i64::from(delta);
            1 <= resolved && resolved <= i64::from(max)
        }
    }
}

fn column_to_index(text: &str) -> Option<u32> {
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

#[cfg(test)]
mod tests {
    use oxfml_core::binding::{
        BindContext, BindRequest, BoundExpr, BoundFormula, NormalizedReference,
        ReferenceBindProfile, ReferenceExpr,
    };
    use oxfml_core::consumer::editor::{EditorAnalysisStage, EditorEditService, EditorEnvironment};
    use oxfml_core::red::project_red_view;
    use oxfml_core::source::{
        FormulaChannelKind, FormulaSourceRecord, FormulaToken, StructureContextVersion,
    };
    use oxfml_core::syntax::parser::{ParseRequest, parse_formula};
    use oxfml_core::{bind_formula, bind_formula_incremental};

    use super::*;

    #[test]
    fn strict_profile_binds_a1_cells_with_dollar_fidelity() {
        let profile = StrictExcelGridReferenceProfile::new();
        let bound = bind_for(
            "strict-a1-fidelity",
            "=A1+$A1+A$1+$A$1",
            FormulaChannelKind::WorksheetA1,
            5,
            3,
            &profile,
        );

        assert_eq!(bound.normalized_references.len(), 4);
        assert_cell_ref(
            &bound.normalized_references[0],
            ExcelGridAxisRef::Relative(-4),
            ExcelGridAxisRef::Relative(-2),
            "A1",
        );
        assert_cell_ref(
            &bound.normalized_references[1],
            ExcelGridAxisRef::Relative(-4),
            ExcelGridAxisRef::Absolute(1),
            "$A1",
        );
        assert_cell_ref(
            &bound.normalized_references[2],
            ExcelGridAxisRef::Absolute(1),
            ExcelGridAxisRef::Relative(-2),
            "A$1",
        );
        assert_cell_ref(
            &bound.normalized_references[3],
            ExcelGridAxisRef::Absolute(1),
            ExcelGridAxisRef::Absolute(1),
            "$A$1",
        );
    }

    #[test]
    fn strict_profile_surfaces_editor_reference_info_through_oxfml_profile_seam() {
        let profile = StrictExcelGridReferenceProfile::new();
        let source = FormulaSourceRecord::new("strict-editor-info", 1, "=A1")
            .with_formula_channel_kind(FormulaChannelKind::WorksheetA1);
        let service = EditorEditService::new(
            EditorEnvironment::new(BindContext {
                caller_row: 5,
                caller_col: 3,
                formula_token: FormulaToken("strict-editor-info-token".to_string()),
                structure_context_version: StructureContextVersion(
                    "strict-excel-grid-struct-v1".to_string(),
                ),
                ..BindContext::default()
            })
            .with_reference_bind_profile(&profile),
        );

        let opened = service.apply_edit(source, None, EditorAnalysisStage::SyntaxAndBind, None);
        let info = service
            .reference_info_at_cursor(&opened.document, 2, None)
            .expect("strict profile reference should be visible to editor info");

        assert_eq!(
            info.source_span,
            oxfml_core::syntax::token::TextSpan::new(1, 2)
        );
        assert_eq!(info.source_text, "A1");
        assert_eq!(info.profile_record.profile_id, EXCEL_GRID_PROFILE_ID);
        assert_eq!(info.profile_record.render_hint.as_deref(), Some("A1"));
        assert_eq!(info.rendered_text.as_deref(), Some("A1"));
        assert!(info.diagnostics.is_empty());
        match decode_excel_grid_reference_payload(&info.profile_record.profile_payload)
            .expect("strict editor info should carry grid profile payload")
        {
            ExcelGridReference::Cell { row, col, .. } => {
                assert_eq!(row, ExcelGridAxisRef::Relative(-4));
                assert_eq!(col, ExcelGridAxisRef::Relative(-2));
            }
            other => panic!("expected strict cell reference payload, got {other:?}"),
        }
    }

    #[test]
    fn strict_profile_r1c1_template_identity_is_caller_independent() {
        let profile = StrictExcelGridReferenceProfile::new();
        let source = FormulaSourceRecord::new("strict-r1c1", 1, "=R[-1]C[2]")
            .with_formula_channel_kind(FormulaChannelKind::WorksheetR1C1);
        let first = bind_request(source.clone(), 5, 3, &profile, None);
        let second = bind_request(source, 20, 9, &profile, Some(&first.bound_formula));

        assert_eq!(
            first.bound_formula.formula_template_identity,
            second.bound_formula.formula_template_identity
        );
        assert_ne!(
            first.bound_formula.placed_formula_identity,
            second.bound_formula.placed_formula_identity
        );
        assert!(!second.reused_bound_formula);
        assert_cell_ref(
            &second.bound_formula.normalized_references[0],
            ExcelGridAxisRef::Relative(-1),
            ExcelGridAxisRef::Relative(2),
            "R[-1]C[2]",
        );
    }

    #[test]
    fn strict_profile_a1_incremental_bind_rebinds_when_caller_anchor_changes() {
        let profile = StrictExcelGridReferenceProfile::new();
        let source = FormulaSourceRecord::new("strict-a1-rebind", 1, "=A1")
            .with_formula_channel_kind(FormulaChannelKind::WorksheetA1);
        let first = bind_request(source.clone(), 5, 3, &profile, None);
        let second = bind_request(source, 20, 9, &profile, Some(&first.bound_formula));

        assert!(!second.reused_bound_formula);
        assert_ne!(
            first.bound_formula.formula_template_identity,
            second.bound_formula.formula_template_identity
        );
        assert_cell_ref(
            &second.bound_formula.normalized_references[0],
            ExcelGridAxisRef::Relative(-19),
            ExcelGridAxisRef::Relative(-8),
            "A1",
        );
    }

    #[test]
    fn strict_profile_rejects_absolute_a1_out_of_bounds() {
        let profile = StrictExcelGridReferenceProfile::new();
        let bound = bind_for(
            "strict-a1-oob",
            "=XFE1",
            FormulaChannelKind::WorksheetA1,
            1,
            1,
            &profile,
        );

        assert!(bound.normalized_references.is_empty());
        assert!(bound.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("outside strict Excel grid bounds")
        }));
    }

    #[test]
    fn strict_profile_marks_relative_r1c1_out_of_current_placement() {
        let profile = StrictExcelGridReferenceProfile::new();
        let bound = bind_for(
            "strict-r1c1-placement-oob",
            "=R[-1]C",
            FormulaChannelKind::WorksheetR1C1,
            1,
            4,
            &profile,
        );

        let record = profile_record(&bound.normalized_references[0]);
        assert_eq!(
            record.validity,
            ReferenceValidity::InvalidForCurrentPlacement
        );
        assert!(bound.diagnostics.is_empty());
    }

    #[test]
    fn strict_profile_binds_qualified_a1_to_target_sheet_payload() {
        let profile = StrictExcelGridReferenceProfile::new();
        let bound = bind_for(
            "strict-qualified-a1",
            "=Sheet2!$B$4",
            FormulaChannelKind::WorksheetA1,
            8,
            8,
            &profile,
        );

        let reference = decoded_reference(&bound.normalized_references[0]);
        match reference {
            ExcelGridReference::Cell {
                sheet_id,
                row,
                col,
                parsed_qualifier,
                ..
            } => {
                assert_eq!(sheet_id, "Sheet2");
                assert_eq!(row, ExcelGridAxisRef::Absolute(4));
                assert_eq!(col, ExcelGridAxisRef::Absolute(2));
                assert_eq!(parsed_qualifier.as_deref(), Some("Sheet2"));
            }
            other => panic!("expected cell reference payload, got {other:?}"),
        }
    }

    #[test]
    fn strict_profile_binds_a1_whole_row_and_column_ranges() {
        let profile = StrictExcelGridReferenceProfile::new();
        let rows = bind_for(
            "strict-a1-whole-rows",
            "=1:3",
            FormulaChannelKind::WorksheetA1,
            5,
            5,
            &profile,
        );
        assert_whole_row_ref(
            &rows.normalized_references[0],
            ExcelGridAxisRef::Relative(-4),
            ExcelGridAxisRef::Relative(-2),
            "1:3",
        );

        let reversed_columns = bind_for(
            "strict-a1-whole-columns-reversed",
            "=C:A",
            FormulaChannelKind::WorksheetA1,
            5,
            5,
            &profile,
        );
        assert_whole_column_ref(
            &reversed_columns.normalized_references[0],
            ExcelGridAxisRef::Relative(-4),
            ExcelGridAxisRef::Relative(-2),
            "C:A",
        );

        let mixed_columns = bind_for(
            "strict-a1-whole-columns-mixed",
            "=$A:C",
            FormulaChannelKind::WorksheetA1,
            5,
            5,
            &profile,
        );
        assert_whole_column_ref(
            &mixed_columns.normalized_references[0],
            ExcelGridAxisRef::Absolute(1),
            ExcelGridAxisRef::Relative(-2),
            "$A:C",
        );
    }

    #[test]
    fn strict_profile_binds_r1c1_whole_row_and_column_ranges() {
        let profile = StrictExcelGridReferenceProfile::new();
        let rows = bind_for(
            "strict-r1c1-whole-rows",
            "=R[-1]:R[1]",
            FormulaChannelKind::WorksheetR1C1,
            5,
            5,
            &profile,
        );
        assert_whole_row_ref(
            &rows.normalized_references[0],
            ExcelGridAxisRef::Relative(-1),
            ExcelGridAxisRef::Relative(1),
            "R[-1]:R[1]",
        );

        let columns = bind_for(
            "strict-r1c1-whole-columns",
            "=C1:C3",
            FormulaChannelKind::WorksheetR1C1,
            5,
            5,
            &profile,
        );
        assert_whole_column_ref(
            &columns.normalized_references[0],
            ExcelGridAxisRef::Absolute(1),
            ExcelGridAxisRef::Absolute(3),
            "C1:C3",
        );
    }

    #[test]
    fn strict_profile_binds_qualified_a1_whole_column_range_to_target_sheet() {
        let profile = StrictExcelGridReferenceProfile::new();
        let bound = bind_for(
            "strict-qualified-whole-column",
            "=Sheet2!A:C",
            FormulaChannelKind::WorksheetA1,
            5,
            5,
            &profile,
        );

        match decoded_reference(&bound.normalized_references[0]) {
            ExcelGridReference::WholeColumn {
                sheet_id,
                start_col,
                end_col,
                parsed_qualifier,
                ..
            } => {
                assert_eq!(sheet_id, "Sheet2");
                assert_eq!(start_col, ExcelGridAxisRef::Relative(-4));
                assert_eq!(end_col, ExcelGridAxisRef::Relative(-2));
                assert_eq!(parsed_qualifier.as_deref(), Some("Sheet2"));
            }
            other => panic!("expected qualified whole column payload, got {other:?}"),
        }
    }

    #[test]
    fn strict_profile_keeps_cell_ranges_as_reference_expression_composition() {
        let profile = StrictExcelGridReferenceProfile::new();
        let bound = bind_for(
            "strict-cell-range-composition",
            "=A1:B2",
            FormulaChannelKind::WorksheetA1,
            5,
            3,
            &profile,
        );

        assert_eq!(bound.normalized_references.len(), 2);
        match &bound.root {
            BoundExpr::Reference(ReferenceExpr::Range { start, end }) => {
                assert_profile_symbolic_expr(
                    start,
                    "excel.grid.v1:cell:book%3Adefault:sheet%3Adefault:R[-4]C[-2]",
                );
                assert_profile_symbolic_expr(
                    end,
                    "excel.grid.v1:cell:book%3Adefault:sheet%3Adefault:R[-3]C[-1]",
                );
            }
            other => panic!("expected symbolic cell range expression, got {other:?}"),
        }
    }

    #[test]
    fn strict_profile_rejects_absolute_whole_column_out_of_bounds() {
        let profile = StrictExcelGridReferenceProfile::new();
        let bound = bind_for(
            "strict-a1-whole-column-oob",
            "=XFE:XFE",
            FormulaChannelKind::WorksheetA1,
            1,
            1,
            &profile,
        );

        assert!(bound.normalized_references.is_empty());
        assert!(bound.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("outside strict Excel grid bounds")
        }));
    }

    #[test]
    fn strict_profile_record_lowering_rejects_payload_key_mismatch() {
        let profile = StrictExcelGridReferenceProfile::new();
        let bound = bind_for(
            "strict-record-mismatch",
            "=A1",
            FormulaChannelKind::WorksheetA1,
            5,
            3,
            &profile,
        );
        let mut record = profile_record(&bound.normalized_references[0]).clone();
        record.normal_form_key =
            ReferenceNormalFormKey("excel.grid.v1:cell:other:sheet:R1C1".to_string());

        assert_eq!(excel_grid_reference_like_from_profile_record(&record), None);
    }

    #[test]
    fn strict_grid_provider_dereferences_symbolic_cell() {
        let profile = StrictExcelGridReferenceProfile::new();
        let bound = bind_for(
            "strict-provider-cell",
            "=A1",
            FormulaChannelKind::WorksheetA1,
            5,
            3,
            &profile,
        );
        let reference = excel_grid_reference_like_from_profile_record(profile_record(
            &bound.normalized_references[0],
        ))
        .expect("strict profile record should lower to ReferenceLike");
        let provider = ExcelGridReferenceSystemProvider::new("book:default", "sheet:default", 5, 3)
            .with_cell_value(
                "book:default",
                "sheet:default",
                1,
                1,
                CalcValue::number(42.0),
            );

        let value = provider
            .dereference(&ReferenceDereferenceRequest { reference })
            .expect("cell reference should dereference");

        assert_eq!(value, CalcValue::number(42.0));
    }

    #[test]
    fn strict_grid_provider_enumerates_sparse_whole_row_range() {
        let profile = StrictExcelGridReferenceProfile::new();
        let bound = bind_for(
            "strict-provider-whole-row",
            "=1:2",
            FormulaChannelKind::WorksheetA1,
            5,
            3,
            &profile,
        );
        let reference = excel_grid_reference_like_from_profile_record(profile_record(
            &bound.normalized_references[0],
        ))
        .expect("strict whole-row record should lower to ReferenceLike");
        let provider = ExcelGridReferenceSystemProvider::new("book:default", "sheet:default", 5, 3)
            .with_cell_value(
                "book:default",
                "sheet:default",
                1,
                2,
                CalcValue::number(12.0),
            )
            .with_cell_value(
                "book:default",
                "sheet:default",
                2,
                4,
                CalcValue::number(24.0),
            );

        let values = provider
            .enumerate_values(&ReferenceEnumerationRequest { reference })
            .expect("whole-row reference should enumerate")
            .expect("strict grid provider should return sparse values");

        assert_eq!(
            values.declared_extent,
            ResolvedReferenceExtent::new(2, STRICT_EXCEL_MAX_COLS as usize)
        );
        assert_eq!(
            values.defined_cells,
            vec![
                ResolvedReferenceCell::new(1, 2, CalcValue::number(12.0)),
                ResolvedReferenceCell::new(2, 4, CalcValue::number(24.0)),
            ]
        );
    }

    #[test]
    fn strict_grid_provider_ignores_non_grid_enumeration_requests() {
        let provider = ExcelGridReferenceSystemProvider::new("book:default", "sheet:default", 5, 3);
        let reference = ReferenceLike::opaque(
            ReferenceSystemId("other.reference.v1".to_string()),
            ReferenceHandle {
                id: ReferenceHandleId::from_bytes(b"other".to_vec()),
            },
            None,
        );

        let values = provider
            .enumerate_values(&ReferenceEnumerationRequest { reference })
            .expect("non-grid enumeration should not fail");

        assert_eq!(values, None);
    }

    fn bind_for(
        stable_id: &str,
        formula: &str,
        channel: FormulaChannelKind,
        caller_row: u32,
        caller_col: u32,
        profile: &dyn ReferenceBindProfile,
    ) -> BoundFormula {
        let source = FormulaSourceRecord::new(stable_id, 1, formula.to_string())
            .with_formula_channel_kind(channel);
        bind_request(source, caller_row, caller_col, profile, None).bound_formula
    }

    fn bind_request(
        source: FormulaSourceRecord,
        caller_row: u32,
        caller_col: u32,
        profile: &dyn ReferenceBindProfile,
        previous: Option<&BoundFormula>,
    ) -> oxfml_core::IncrementalBindResult {
        let parse = parse_formula(ParseRequest {
            source: source.clone(),
        });
        let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
        let request = BindRequest {
            source,
            green_tree: parse.green_tree,
            red_projection: red,
            context: BindContext {
                caller_row,
                caller_col,
                formula_token: FormulaToken("strict-excel-template-token".to_string()),
                structure_context_version: StructureContextVersion(
                    "strict-excel-grid-struct-v1".to_string(),
                ),
                ..BindContext::default()
            },
            host_name_resolver: None,
            reference_bind_profile: Some(profile),
        };

        match previous {
            Some(previous) => bind_formula_incremental(request, Some(previous)),
            None => {
                let bind = bind_formula(request);
                oxfml_core::IncrementalBindResult {
                    bound_formula: bind.bound_formula,
                    reused_bound_formula: false,
                }
            }
        }
    }

    fn assert_cell_ref(
        normalized: &NormalizedReference,
        expected_row: ExcelGridAxisRef,
        expected_col: ExcelGridAxisRef,
        expected_source_text: &str,
    ) {
        let record = profile_record(normalized);
        assert_eq!(record.profile_id, EXCEL_GRID_PROFILE_ID);
        assert_eq!(
            record.source_info.address_fidelity.as_deref(),
            Some(expected_source_text)
        );
        assert_eq!(record.validity, ReferenceValidity::ValidAfterInstantiation);
        match decode_excel_grid_reference_payload(&record.profile_payload)
            .expect("excel grid payload")
        {
            ExcelGridReference::Cell {
                row,
                col,
                source_text,
                ..
            } => {
                assert_eq!(row, expected_row);
                assert_eq!(col, expected_col);
                assert_eq!(source_text, expected_source_text);
            }
            other => panic!("expected cell reference payload, got {other:?}"),
        }
    }

    fn assert_whole_row_ref(
        normalized: &NormalizedReference,
        expected_start_row: ExcelGridAxisRef,
        expected_end_row: ExcelGridAxisRef,
        expected_source_text: &str,
    ) {
        let record = profile_record(normalized);
        assert_eq!(record.profile_id, EXCEL_GRID_PROFILE_ID);
        assert_eq!(
            record.source_info.address_fidelity.as_deref(),
            Some(expected_source_text)
        );
        assert_eq!(record.validity, ReferenceValidity::ValidAfterInstantiation);
        match decode_excel_grid_reference_payload(&record.profile_payload)
            .expect("excel grid payload")
        {
            ExcelGridReference::WholeRow {
                start_row,
                end_row,
                source_text,
                ..
            } => {
                assert_eq!(start_row, expected_start_row);
                assert_eq!(end_row, expected_end_row);
                assert_eq!(source_text, expected_source_text);
            }
            other => panic!("expected whole row reference payload, got {other:?}"),
        }
    }

    fn assert_whole_column_ref(
        normalized: &NormalizedReference,
        expected_start_col: ExcelGridAxisRef,
        expected_end_col: ExcelGridAxisRef,
        expected_source_text: &str,
    ) {
        let record = profile_record(normalized);
        assert_eq!(record.profile_id, EXCEL_GRID_PROFILE_ID);
        assert_eq!(
            record.source_info.address_fidelity.as_deref(),
            Some(expected_source_text)
        );
        assert_eq!(record.validity, ReferenceValidity::ValidAfterInstantiation);
        match decode_excel_grid_reference_payload(&record.profile_payload)
            .expect("excel grid payload")
        {
            ExcelGridReference::WholeColumn {
                start_col,
                end_col,
                source_text,
                ..
            } => {
                assert_eq!(start_col, expected_start_col);
                assert_eq!(end_col, expected_end_col);
                assert_eq!(source_text, expected_source_text);
            }
            other => panic!("expected whole column reference payload, got {other:?}"),
        }
    }

    fn assert_profile_symbolic_expr(reference: &ReferenceExpr, expected_key: &str) {
        match reference {
            ReferenceExpr::Atom(NormalizedReference::ProfileSymbolic(record)) => {
                assert_eq!(record.normal_form_key.0, expected_key);
            }
            other => panic!("expected profile symbolic reference expr, got {other:?}"),
        }
    }

    fn decoded_reference(normalized: &NormalizedReference) -> ExcelGridReference {
        decode_excel_grid_reference_payload(&profile_record(normalized).profile_payload)
            .expect("excel grid payload")
    }

    fn profile_record(normalized: &NormalizedReference) -> &ProfileReferenceRecord {
        match normalized {
            NormalizedReference::ProfileSymbolic(record) => record,
            other => panic!("expected profile symbolic reference, got {other:?}"),
        }
    }
}
