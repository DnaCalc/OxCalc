#![forbid(unsafe_code)]

//! Strict Excel-grid reference bind profile for the OxFml/OxCalc profile seam.
//!
//! Provenance note: the behavioral authority order for this profile is Excel
//! observation first, public file/formula specifications second, and Foundation
//! guidance as the local architecture map. The grid bounds mirror
//! `docs/spec/core-engine/CORE_ENGINE_GRID_MODEL.md` Section 4.1 and the
//! Foundation reference-resolution doctrine in
//! `../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md`.

use std::borrow::Cow;
use std::collections::BTreeMap;

use oxfml_core::binding::{
    ProfilePayload, ProfileReferenceRecord, ProfileVersion, ReferenceAtomBindRequest,
    ReferenceAtomBindResult, ReferenceBindProfile, ReferenceDependencyEnvelope,
    ReferenceFingerprintPolicy, ReferenceNameBindRequest, ReferenceNormalFormKey,
    ReferenceOperatorCapabilities, ReferencePolicy, ReferenceProfileFingerprint,
    ReferenceProfileFingerprintContext, ReferenceRangeBindRequest, ReferenceRangeBindResult,
    ReferenceSourceInfo, ReferenceStructuredBindRequest, ReferenceTransformKind,
    ReferenceTransformOutcome, ReferenceTransformRequest, ReferenceTransformResult,
    ReferenceValidity,
};
use oxfml_core::source::FormulaChannelKind;
use oxfunc_core::resolver::{
    CallerContext, ReferenceComposeOperation, ReferenceComposeRequest, ReferenceDereferenceRequest,
    ReferenceEnumerationRequest, ReferenceFacts, ReferenceFactsRequest, ReferenceResolutionError,
    ReferenceSystemError, ReferenceSystemOperation, ReferenceSystemProvider,
    ReferenceTextResolutionMode, ReferenceTextResolveRequest,
    ReferenceTransformKind as EvalTransformKind, ReferenceTransformRequest as EvalTransformRequest,
    ResolvedReferenceCell, ResolvedReferenceExtent, ResolvedReferenceValues,
    materialize_resolved_reference_values, reference_facts,
};
use oxfunc_core::value::{
    CalcValue, ExcelText, ReferenceDisplay, ReferenceHandle, ReferenceHandleId, ReferenceIdentity,
    ReferenceKind, ReferenceLike, ReferenceSystemId,
};
// Profile-pure coordinate primitives now live in `crate::grid::coords`; this
// facade preserves the historical `excel_grid_reference::` paths during the
// grid module decomposition.
pub use crate::grid::coords::{
    ExcelGridAxisRef, ExcelGridBounds, ExcelGridCellAddress, ExcelGridReferenceStyle,
    STRICT_EXCEL_MAX_COLS, STRICT_EXCEL_MAX_ROWS,
};

pub const EXCEL_GRID_PROFILE_ID: &str = "excel.grid.v1";
pub const STRICT_EXCEL_GRID_PROFILE_ALIAS: &str = "strict-excel-grid";

// The grid reference AST and structural-edit description types now live in
// `crate::grid::ast`; this facade preserves the historical paths during the
// decomposition.
pub use crate::grid::ast::{
    EXCEL_GRID_STRUCTURAL_EDIT_PAYLOAD_KIND, ExcelGridFormulaAnchor, ExcelGridReference,
    ExcelGridReferenceTransformPayload, ExcelGridStructuralEdit, ExcelGridStructuralEditAxis,
    ExcelGridStructuralEditKind,
};

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

// Profile-pure grid geometry (resolved rectangles and the structured-table
// descriptions built from them) now lives in `crate::grid::geometry`; this
// facade preserves the historical paths during the decomposition.
use crate::grid::geometry::GridRect;
pub use crate::grid::geometry::{
    ExcelGridResolvedRect, ExcelGridStructuredTable, ExcelGridStructuredTableColumn,
};

#[derive(Debug, Clone)]
pub struct ExcelGridReferenceSystemProvider<'a> {
    workbook_id: String,
    sheet_id: String,
    caller_row: u32,
    caller_col: u32,
    bounds: ExcelGridBounds,
    cells: Cow<'a, BTreeMap<ExcelGridCellAddress, CalcValue>>,
    spill_extents: BTreeMap<ExcelGridCellAddress, GridRect>,
    defined_names: BTreeMap<String, GridRect>,
    structured_references: BTreeMap<String, GridRect>,
    structured_tables: BTreeMap<String, ExcelGridStructuredTable>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcelGridSpillAnchorDereferenceReport {
    pub anchor: ExcelGridCellAddress,
    pub declared_cell_count: usize,
    pub ledger_probe_count: usize,
    pub extent_cells_scanned_for_ledger: usize,
    pub value_entries_scanned: usize,
    pub defined_cells_returned: usize,
}

impl<'a> ExcelGridReferenceSystemProvider<'a> {
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
            cells: Cow::Owned(BTreeMap::new()),
            spill_extents: BTreeMap::new(),
            defined_names: BTreeMap::new(),
            structured_references: BTreeMap::new(),
            structured_tables: BTreeMap::new(),
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
        self.cells.to_mut().insert(
            ExcelGridCellAddress::new(workbook_id, sheet_id, row, col),
            value,
        );
        self
    }

    /// Borrow an externally-owned value store instead of cloning it in.
    ///
    /// The recalc hot path rebuilds a provider per formula cell but the grid's
    /// computed value store is invariant within that single construction, so the
    /// caller lends it here to avoid an O(cells) deep clone per formula cell.
    #[must_use]
    pub fn with_borrowed_cells(
        mut self,
        cells: &'a BTreeMap<ExcelGridCellAddress, CalcValue>,
    ) -> Self {
        self.cells = Cow::Borrowed(cells);
        self
    }

    #[must_use]
    pub fn with_spill_extent(
        mut self,
        anchor_workbook_id: impl Into<String>,
        anchor_sheet_id: impl Into<String>,
        anchor_row: u32,
        anchor_col: u32,
        extent: ExcelGridResolvedRect,
    ) -> Self {
        self.spill_extents.insert(
            ExcelGridCellAddress::new(anchor_workbook_id, anchor_sheet_id, anchor_row, anchor_col),
            GridRect {
                workbook_id: extent.workbook_id,
                sheet_id: extent.sheet_id,
                top_row: extent.top_row,
                left_col: extent.left_col,
                bottom_row: extent.bottom_row,
                right_col: extent.right_col,
            },
        );
        self
    }

    #[must_use]
    pub fn with_defined_name(
        mut self,
        name: impl AsRef<str>,
        extent: ExcelGridResolvedRect,
    ) -> Self {
        if let Some(name_key) = excel_grid_defined_name_key(name.as_ref(), self.bounds) {
            self.defined_names.insert(
                name_key,
                GridRect {
                    workbook_id: extent.workbook_id,
                    sheet_id: extent.sheet_id,
                    top_row: extent.top_row,
                    left_col: extent.left_col,
                    bottom_row: extent.bottom_row,
                    right_col: extent.right_col,
                },
            );
        }
        self
    }

    #[must_use]
    pub fn with_structured_reference_text(
        mut self,
        text: impl AsRef<str>,
        extent: ExcelGridResolvedRect,
    ) -> Self {
        self.structured_references.insert(
            excel_grid_structured_reference_key(text.as_ref()),
            GridRect {
                workbook_id: extent.workbook_id,
                sheet_id: extent.sheet_id,
                top_row: extent.top_row,
                left_col: extent.left_col,
                bottom_row: extent.bottom_row,
                right_col: extent.right_col,
            },
        );
        self
    }

    #[must_use]
    pub fn with_structured_table(mut self, mut table: ExcelGridStructuredTable) -> Self {
        table.columns.sort_by_key(|column| column.ordinal);
        self.structured_tables.insert(
            excel_grid_structured_reference_key(&table.table_name),
            table,
        );
        self
    }

    pub fn spill_anchor_dereference_report(
        &self,
        reference: &ReferenceLike,
    ) -> Result<ExcelGridSpillAnchorDereferenceReport, ReferenceResolutionError> {
        if reference.system.0 != EXCEL_GRID_PROFILE_ID
            || reference.kind() != ReferenceKind::SpillAnchor
        {
            return Err(ReferenceResolutionError::UnresolvedReference {
                target: reference.target().to_string(),
            });
        }
        let anchor = self.spill_anchor_address(reference)?;
        let rect = self.spill_extents.get(&anchor).cloned().ok_or_else(|| {
            ReferenceResolutionError::UnresolvedReference {
                target: reference.target().to_string(),
            }
        })?;
        let values = self.resolved_values_for_rect(&rect)?;
        Ok(ExcelGridSpillAnchorDereferenceReport {
            anchor,
            declared_cell_count: values.declared_extent.declared_cell_count(),
            ledger_probe_count: 1,
            extent_cells_scanned_for_ledger: 0,
            value_entries_scanned: self.cells.len(),
            defined_cells_returned: values.defined_cells.len(),
        })
    }

    fn cell_value(&self, address: &ExcelGridCellAddress) -> CalcValue {
        self.cells
            .get(address)
            .cloned()
            .unwrap_or_else(CalcValue::empty)
    }

    fn defined_name_rect(&self, name: &str) -> Option<GridRect> {
        let key = excel_grid_defined_name_key(name, self.bounds)?;
        self.defined_names
            .get(&key)
            .cloned()
            .or_else(|| self.caller_local_table_column_rect(name))
    }

    fn structured_reference_rect(&self, text: &str) -> Option<GridRect> {
        let rects = self.structured_reference_rects(text)?;
        match rects.as_slice() {
            [rect] => Some(rect.clone()),
            _ => None,
        }
    }

    fn structured_reference_rects(&self, text: &str) -> Option<Vec<GridRect>> {
        self.structured_references
            .get(&excel_grid_structured_reference_key(text))
            .cloned()
            .map(|rect| vec![rect])
            .or_else(|| {
                resolve_structured_reference_rects_from_tables(text, &self.structured_tables)
            })
    }

    fn structured_reference_like(&self, text: &str) -> Option<ReferenceLike> {
        let rects = self.structured_reference_rects(text)?;
        reference_like_for_rects(&rects)
    }

    fn caller_local_table_column_rect(&self, name: &str) -> Option<GridRect> {
        let caller = ExcelGridCellAddress::new(
            self.workbook_id.clone(),
            self.sheet_id.clone(),
            self.caller_row,
            self.caller_col,
        );
        for table in self.structured_tables.values() {
            if !table.table_range.contains(&caller) {
                continue;
            }
            let column_index = table_column_index(table, name)?;
            return Some(table.columns[column_index].data_rect.clone());
        }
        None
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

    fn bind_name(&self, request: &ReferenceNameBindRequest) -> ReferenceAtomBindResult {
        let name = atom_text_without_qualifier(&request.source_text).trim();
        if excel_grid_defined_name_key(name, self.bounds).is_none() {
            return ReferenceAtomBindResult::LegacyCompatibility;
        }

        ReferenceAtomBindResult::Bound(profile_record_for_name_reference(
            self.profile_id(),
            request,
            ExcelGridReference::Name {
                workbook_id: request.workbook_id.clone(),
                sheet_id: request.sheet_id.clone(),
                name: name.to_string(),
                source_text: request.source_text.clone(),
                parsed_qualifier: request.parsed_qualifier.clone(),
            },
            ReferenceValidity::DynamicOrHostSensitive,
        ))
    }

    fn bind_structured_reference(
        &self,
        request: &ReferenceStructuredBindRequest,
    ) -> ReferenceAtomBindResult {
        ReferenceAtomBindResult::Bound(profile_record_for_structured_reference(
            self.profile_id(),
            request,
            ExcelGridReference::StructuredReference {
                workbook_id: request.workbook_id.clone(),
                sheet_id: request.sheet_id.clone(),
                source_text: request.source_text.clone(),
                parsed_qualifier: None,
            },
            ReferenceValidity::DynamicOrHostSensitive,
        ))
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

    fn transform_reference(&self, request: &ReferenceTransformRequest) -> ReferenceTransformResult {
        if request.reference.profile_id != self.profile_id() {
            return ReferenceTransformResult {
                outcome: ReferenceTransformOutcome::Unsupported,
                reference: Some(request.reference.clone()),
                diagnostics: vec![format!(
                    "strict grid profile cannot transform reference for profile '{}'",
                    request.reference.profile_id
                )],
            };
        }

        if request.transform_kind != ReferenceTransformKind::StructuralEdit {
            return ReferenceTransformResult {
                outcome: ReferenceTransformOutcome::Unsupported,
                reference: Some(request.reference.clone()),
                diagnostics: vec![format!(
                    "strict grid profile only supports structural edit transforms, got {:?}",
                    request.transform_kind
                )],
            };
        }

        let Some(payload) = &request.payload else {
            return ReferenceTransformResult {
                outcome: ReferenceTransformOutcome::Unsupported,
                reference: Some(request.reference.clone()),
                diagnostics: vec![
                    "strict grid structural transform requires an excel-grid-structural-edit.v1 payload"
                        .to_string(),
                ],
            };
        };
        let Some(payload) = decode_excel_grid_transform_payload(payload) else {
            return ReferenceTransformResult {
                outcome: ReferenceTransformOutcome::Unsupported,
                reference: Some(request.reference.clone()),
                diagnostics: vec![
                    "strict grid structural transform payload is missing or malformed".to_string(),
                ],
            };
        };
        if let Err(diagnostic) = validate_structural_edit(&payload.edit, self.bounds) {
            return ReferenceTransformResult {
                outcome: ReferenceTransformOutcome::Unsupported,
                reference: Some(request.reference.clone()),
                diagnostics: vec![diagnostic],
            };
        }

        let Some(reference) =
            decode_excel_grid_reference_payload(&request.reference.profile_payload)
        else {
            return ReferenceTransformResult {
                outcome: ReferenceTransformOutcome::Unsupported,
                reference: Some(request.reference.clone()),
                diagnostics: vec![
                    "strict grid reference payload is missing or malformed".to_string(),
                ],
            };
        };

        match transform_excel_grid_reference(&reference, &request.reference, &payload, self.bounds)
        {
            Ok(result) => result,
            Err(diagnostic) => ReferenceTransformResult {
                outcome: ReferenceTransformOutcome::Unsupported,
                reference: Some(request.reference.clone()),
                diagnostics: vec![diagnostic],
            },
        }
    }
}

impl<'a> ReferenceSystemProvider for ExcelGridReferenceSystemProvider<'a> {
    fn transform_reference(
        &self,
        request: &EvalTransformRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        match &request.transform {
            EvalTransformKind::Offset {
                row_offset,
                col_offset,
                height,
                width,
            } => self.offset_reference(
                &request.reference,
                *row_offset,
                *col_offset,
                *height,
                *width,
            ),
            _ => Err(ReferenceSystemError::Unsupported {
                operation: ReferenceSystemOperation::Transform,
            }),
        }
    }

    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        if let Some(values) = self.resolved_values_for_reference_shape(&request.reference)? {
            if values.declared_extent.declared_cell_count() > MAX_MATERIALIZED_GRID_CELLS {
                return Err(ReferenceResolutionError::ProviderFailure {
                    detail: "excel_grid_reference_requires_sparse_enumeration".to_string(),
                });
            }
            return materialize_resolved_reference_values(&values).map(CalcValue::array);
        }

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
        if let Some(values) = self.resolved_values_for_reference_shape(&request.reference)? {
            return Ok(Some(values));
        }
        let rect = self.reference_rect(&request.reference)?;
        self.resolved_values_for_rect(&rect).map(Some)
    }

    fn resolve_text(
        &self,
        request: &ReferenceTextResolveRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        match request.mode {
            ReferenceTextResolutionMode::Indirect => {}
        }

        let text = request.text.trim();
        if let Some(rect) = self.defined_name_rect(text) {
            return Ok(reference_like_for_rect(&rect));
        }
        if let Some(reference) = self.structured_reference_like(text) {
            return Ok(reference);
        }

        if request.a1_style != Some(false) {
            let kind = if text.contains(':') {
                ReferenceKind::Area
            } else {
                ReferenceKind::A1
            };
            let candidate = ReferenceLike::textual(
                ReferenceSystemId(EXCEL_GRID_PROFILE_ID.to_string()),
                kind,
                ExcelText::from_interop_assignment(text),
                Some(ReferenceDisplay {
                    text: ExcelText::from_interop_assignment(text),
                }),
            );
            if let Some(rect) = parse_excel_grid_textual_reference(&candidate, self) {
                return Ok(reference_like_for_rect(&rect));
            }
        }

        Err(ReferenceSystemError::InvalidReferenceText {
            text: request.text.clone(),
        })
    }

    fn facts(
        &self,
        request: &ReferenceFactsRequest,
    ) -> Result<ReferenceFacts, ReferenceSystemError> {
        Ok(reference_facts(&request.reference))
    }

    fn compose_references(
        &self,
        request: &ReferenceComposeRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        match request.operation {
            ReferenceComposeOperation::Range => {
                let lhs = self
                    .reference_rect(&request.lhs)
                    .map_err(reference_resolution_as_system_error)?;
                let rhs = self
                    .reference_rect(&request.rhs)
                    .map_err(reference_resolution_as_system_error)?;
                if lhs.workbook_id != rhs.workbook_id || lhs.sheet_id != rhs.sheet_id {
                    return Err(ReferenceSystemError::ProviderFailure {
                        detail: "excel_grid_range_requires_same_sheet".to_string(),
                    });
                }
                Ok(reference_like_for_rect(&GridRect {
                    workbook_id: lhs.workbook_id,
                    sheet_id: lhs.sheet_id,
                    top_row: lhs.top_row.min(rhs.top_row),
                    left_col: lhs.left_col.min(rhs.left_col),
                    bottom_row: lhs.bottom_row.max(rhs.bottom_row),
                    right_col: lhs.right_col.max(rhs.right_col),
                }))
            }
            ReferenceComposeOperation::Union => {
                let parts =
                    multi_area_parts_for_union(&request.lhs, &request.rhs).ok_or_else(|| {
                        ReferenceSystemError::ProviderFailure {
                            detail: "excel_grid_union_requires_reference_targets".to_string(),
                        }
                    })?;
                ReferenceLike::multi_area(parts).ok_or_else(|| {
                    ReferenceSystemError::ProviderFailure {
                        detail: "excel_grid_union_requires_at_least_two_references".to_string(),
                    }
                })
            }
            ReferenceComposeOperation::Intersection => Err(ReferenceSystemError::Unsupported {
                operation: ReferenceSystemOperation::Compose,
            }),
        }
    }

    fn caller_context(&self) -> Option<CallerContext> {
        Some(CallerContext {
            prefix: Some(format!("{}!{}", self.workbook_id, self.sheet_id)),
            row: usize::try_from(self.caller_row).unwrap_or(usize::MAX),
            col: usize::try_from(self.caller_col).unwrap_or(usize::MAX),
        })
    }
}

fn reference_resolution_as_system_error(error: ReferenceResolutionError) -> ReferenceSystemError {
    ReferenceSystemError::ProviderFailure {
        detail: format!("excel_grid_reference_resolution_failed:{error:?}"),
    }
}

const MAX_MATERIALIZED_GRID_CELLS: usize = 100_000;

impl<'a> ExcelGridReferenceSystemProvider<'a> {
    pub fn resolved_rect_for_reference(
        &self,
        reference: &ReferenceLike,
    ) -> Result<ExcelGridResolvedRect, ReferenceResolutionError> {
        let rects = self.resolved_rects_for_reference(reference)?;
        match rects.as_slice() {
            [rect] => Ok(rect.clone()),
            _ => Err(ReferenceResolutionError::ProviderFailure {
                detail: "excel_grid_reference_requires_single_rect".to_string(),
            }),
        }
    }

    /// Apply an Excel `OFFSET` to a single-rect reference, returning a new grid
    /// reference at the offset (optionally resized) position. Off-grid results
    /// surface as a provider failure, which `OFFSET` maps to `#REF!`.
    pub fn offset_reference(
        &self,
        reference: &ReferenceLike,
        row_offset: i64,
        col_offset: i64,
        height: Option<usize>,
        width: Option<usize>,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        let rects = self.resolved_rects_for_reference(reference).map_err(|_| {
            ReferenceSystemError::Unsupported {
                operation: ReferenceSystemOperation::Transform,
            }
        })?;
        let [rect] = rects.as_slice() else {
            // OFFSET operates on a single contiguous rectangle, not a multi-area.
            return Err(ReferenceSystemError::Unsupported {
                operation: ReferenceSystemOperation::Transform,
            });
        };
        let new_top = i64::from(rect.top_row) + row_offset;
        let new_left = i64::from(rect.left_col) + col_offset;
        let new_height = height
            .and_then(|value| i64::try_from(value).ok())
            .unwrap_or_else(|| i64::from(rect.row_count()));
        let new_width = width
            .and_then(|value| i64::try_from(value).ok())
            .unwrap_or_else(|| i64::from(rect.col_count()));
        let new_bottom = new_top + new_height - 1;
        let new_right = new_left + new_width - 1;
        if new_top < 1
            || new_left < 1
            || new_height < 1
            || new_width < 1
            || new_bottom > i64::from(self.bounds.max_rows)
            || new_right > i64::from(self.bounds.max_cols)
        {
            return Err(ReferenceSystemError::ProviderFailure {
                detail: "excel_grid_offset_out_of_bounds".to_string(),
            });
        }
        let offset_rect = GridRect {
            workbook_id: rect.workbook_id.clone(),
            sheet_id: rect.sheet_id.clone(),
            top_row: new_top as u32,
            left_col: new_left as u32,
            bottom_row: new_bottom as u32,
            right_col: new_right as u32,
        };
        Ok(reference_like_for_rect(&offset_rect))
    }

    pub fn resolved_rects_for_reference(
        &self,
        reference: &ReferenceLike,
    ) -> Result<Vec<ExcelGridResolvedRect>, ReferenceResolutionError> {
        if reference.system.0 != EXCEL_GRID_PROFILE_ID {
            return Err(ReferenceResolutionError::UnresolvedReference {
                target: reference.target().to_string(),
            });
        }

        let rects = if reference.kind() == ReferenceKind::MultiArea {
            self.multi_area_rects(reference)?
        } else if reference.kind() == ReferenceKind::Structured {
            match self.structured_reference_rects(reference.target()) {
                Some(rects) => rects,
                None => vec![self.reference_rect(reference)?],
            }
        } else {
            vec![self.reference_rect(reference)?]
        };

        Ok(rects
            .into_iter()
            .map(|rect| ExcelGridResolvedRect {
                workbook_id: rect.workbook_id,
                sheet_id: rect.sheet_id,
                top_row: rect.top_row,
                left_col: rect.left_col,
                bottom_row: rect.bottom_row,
                right_col: rect.right_col,
            })
            .collect())
    }

    fn reference_rect(
        &self,
        reference: &ReferenceLike,
    ) -> Result<GridRect, ReferenceResolutionError> {
        if reference.system.0 != EXCEL_GRID_PROFILE_ID {
            return Err(ReferenceResolutionError::UnresolvedReference {
                target: reference.target().to_string(),
            });
        }
        if reference.kind() == ReferenceKind::SpillAnchor {
            return self.spill_rect(reference);
        }
        if let Some(rect) = self.defined_name_rect(reference.target()) {
            return Ok(rect);
        }
        if reference.kind() == ReferenceKind::Structured
            && let Some(rect) = self.structured_reference_rect(reference.target())
        {
            return Ok(rect);
        }
        if let Some(key) = opaque_reference_key(reference)
            && let Some(rect) = parse_excel_grid_reference_key(&key, self)
        {
            return Ok(rect);
        }
        parse_excel_grid_textual_reference(reference, self).ok_or_else(|| {
            ReferenceResolutionError::UnresolvedReference {
                target: reference.target().to_string(),
            }
        })
    }

    fn spill_rect(&self, reference: &ReferenceLike) -> Result<GridRect, ReferenceResolutionError> {
        let anchor = self.spill_anchor_address(reference)?;
        self.spill_extents.get(&anchor).cloned().ok_or_else(|| {
            ReferenceResolutionError::UnresolvedReference {
                target: reference.target().to_string(),
            }
        })
    }

    fn spill_anchor_address(
        &self,
        reference: &ReferenceLike,
    ) -> Result<ExcelGridCellAddress, ReferenceResolutionError> {
        let Some(anchor_target) = reference.target().trim().strip_suffix('#') else {
            return Err(ReferenceResolutionError::UnresolvedReference {
                target: reference.target().to_string(),
            });
        };
        let anchor_reference = ReferenceLike::new(ReferenceKind::A1, anchor_target);
        let anchor_rect =
            parse_excel_grid_textual_reference(&anchor_reference, self).ok_or_else(|| {
                ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                }
            })?;
        if anchor_rect.row_count() != 1 || anchor_rect.col_count() != 1 {
            return Err(ReferenceResolutionError::UnresolvedReference {
                target: reference.target().to_string(),
            });
        }
        Ok(ExcelGridCellAddress::new(
            anchor_rect.workbook_id,
            anchor_rect.sheet_id,
            anchor_rect.top_row,
            anchor_rect.left_col,
        ))
    }

    fn resolved_values_for_rect(
        &self,
        rect: &GridRect,
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

    fn resolved_values_for_reference_shape(
        &self,
        reference: &ReferenceLike,
    ) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
        if reference.system.0 != EXCEL_GRID_PROFILE_ID {
            return Ok(None);
        }

        if reference.kind() == ReferenceKind::MultiArea {
            return self
                .multi_area_rects(reference)
                .and_then(|rects| self.resolved_values_for_rects(&rects))
                .map(Some);
        }

        if reference.kind() == ReferenceKind::Structured
            && let Some(rects) = self.structured_reference_rects(reference.target())
            && rects.len() > 1
        {
            return self.resolved_values_for_rects(&rects).map(Some);
        }

        Ok(None)
    }

    fn multi_area_rects(
        &self,
        reference: &ReferenceLike,
    ) -> Result<Vec<GridRect>, ReferenceResolutionError> {
        let targets = reference.multi_area_targets().ok_or_else(|| {
            ReferenceResolutionError::ProviderFailure {
                detail: "excel_grid_multi_area_targets_invalid".to_string(),
            }
        })?;
        targets
            .into_iter()
            .map(|target| {
                self.reference_rect(&ReferenceLike::textual(
                    ReferenceSystemId(EXCEL_GRID_PROFILE_ID.to_string()),
                    ReferenceKind::Area,
                    ExcelText::from_interop_assignment(&target),
                    Some(ReferenceDisplay {
                        text: ExcelText::from_interop_assignment(&target),
                    }),
                ))
            })
            .collect()
    }

    fn resolved_values_for_rects(
        &self,
        rects: &[GridRect],
    ) -> Result<ResolvedReferenceValues, ReferenceResolutionError> {
        match rects {
            [] => Err(ReferenceResolutionError::UnresolvedReference {
                target: "empty grid reference area set".to_string(),
            }),
            [rect] => self.resolved_values_for_rect(rect),
            _ => {
                let mut cells = Vec::new();
                let mut col_offset = 0usize;
                let mut identities = Vec::with_capacity(rects.len());
                for rect in rects {
                    let values = self.resolved_values_for_rect(rect)?;
                    let area_cols = values.declared_extent.cols;
                    for cell in values.defined_cells {
                        let flattened_col = (cell.row - 1)
                            .checked_mul(area_cols)
                            .and_then(|base| base.checked_add(cell.col))
                            .and_then(|col| col_offset.checked_add(col))
                            .ok_or_else(|| ReferenceResolutionError::ProviderFailure {
                                detail: "excel_grid_multi_area_extent_overflow".to_string(),
                            })?;
                        cells.push(ResolvedReferenceCell::new(1, flattened_col, cell.value));
                    }
                    col_offset = col_offset
                        .checked_add(values.declared_extent.declared_cell_count())
                        .ok_or_else(|| ReferenceResolutionError::ProviderFailure {
                            detail: "excel_grid_multi_area_extent_overflow".to_string(),
                        })?;
                    if let Some(identity) = values.reader_identity {
                        identities.push(identity);
                    }
                }
                cells.sort_by_key(|cell| (cell.row, cell.col));
                Ok(ResolvedReferenceValues::new(
                    ResolvedReferenceExtent::new(1, col_offset),
                    cells,
                    Some(format!("excel-grid:v1:multi-area:{}", identities.join("|"))),
                ))
            }
        }
    }
}

fn opaque_reference_key(reference: &ReferenceLike) -> Option<String> {
    match &reference.identity {
        ReferenceIdentity::Opaque(handle) => String::from_utf8(handle.id.bytes.clone()).ok(),
        ReferenceIdentity::Textual(textual)
            if reference.system.0 == EXCEL_GRID_PROFILE_ID
                && textual.kind == ReferenceKind::Area =>
        {
            Some(textual.text.to_string_lossy())
        }
        _ => None,
    }
}

fn reference_like_for_rect(rect: &GridRect) -> ReferenceLike {
    let key = reference_key_for_rect(rect);
    ReferenceLike::textual(
        ReferenceSystemId(EXCEL_GRID_PROFILE_ID.to_string()),
        ReferenceKind::Area,
        ExcelText::from_interop_assignment(&key),
        Some(ReferenceDisplay {
            text: ExcelText::from_interop_assignment(&format!(
                "R{}C{}:R{}C{}",
                rect.top_row, rect.left_col, rect.bottom_row, rect.right_col
            )),
        }),
    )
}

fn reference_like_for_rects(rects: &[GridRect]) -> Option<ReferenceLike> {
    match rects {
        [] => None,
        [rect] => Some(reference_like_for_rect(rect)),
        _ => ReferenceLike::multi_area(rects.iter().map(reference_key_for_rect).collect()),
    }
}

fn reference_key_for_rect(rect: &GridRect) -> String {
    let key = format!(
        "{}:area:{}:{}:R{}C{}:R{}C{}",
        EXCEL_GRID_PROFILE_ID,
        key_component(&rect.workbook_id),
        key_component(&rect.sheet_id),
        rect.top_row,
        rect.left_col,
        rect.bottom_row,
        rect.right_col
    );
    key
}

fn multi_area_parts_for_union(lhs: &ReferenceLike, rhs: &ReferenceLike) -> Option<Vec<String>> {
    let mut parts = Vec::new();
    append_multi_area_parts(&mut parts, lhs)?;
    append_multi_area_parts(&mut parts, rhs)?;
    Some(parts)
}

fn append_multi_area_parts(parts: &mut Vec<String>, reference: &ReferenceLike) -> Option<()> {
    if reference.kind() == ReferenceKind::MultiArea {
        parts.extend(reference.multi_area_targets()?);
        return Some(());
    }
    parts.push(
        opaque_reference_key(reference).unwrap_or_else(|| reference.target().trim().to_string()),
    );
    Some(())
}

fn parse_excel_grid_reference_key(
    key: &str,
    provider: &ExcelGridReferenceSystemProvider<'_>,
) -> Option<GridRect> {
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
            Some(GridRect {
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
            Some(GridRect {
                workbook_id: unkey_component(workbook_id)?,
                sheet_id: unkey_component(sheet_id)?,
                top_row: start_row.min(end_row),
                left_col: start_col.min(end_col),
                bottom_row: start_row.max(end_row),
                right_col: start_col.max(end_col),
            })
        }
        [_, "name", workbook_id, sheet_id, name] => {
            if unkey_component(workbook_id)? != provider.workbook_id
                || unkey_component(sheet_id)? != provider.sheet_id
            {
                return None;
            }
            provider.defined_name_rect(&unkey_component(name)?)
        }
        [_, "structured", workbook_id, sheet_id, source_text] => {
            if unkey_component(workbook_id)? != provider.workbook_id
                || unkey_component(sheet_id)? != provider.sheet_id
            {
                return None;
            }
            provider.structured_reference_rect(&unkey_component(source_text)?)
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
            Some(GridRect {
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
            Some(GridRect {
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

fn parse_excel_grid_textual_reference(
    reference: &ReferenceLike,
    provider: &ExcelGridReferenceSystemProvider<'_>,
) -> Option<GridRect> {
    let target = textual_grid_target_on_provider_sheet(reference.target(), provider)?;
    match reference.kind() {
        ReferenceKind::A1 => {
            let (row, col) = parse_textual_a1_point(target, provider.bounds)?;
            Some(GridRect {
                workbook_id: provider.workbook_id.clone(),
                sheet_id: provider.sheet_id.clone(),
                top_row: row,
                left_col: col,
                bottom_row: row,
                right_col: col,
            })
        }
        ReferenceKind::Area => {
            let (start, end) = target
                .split_once(':')
                .map_or((target, target), |(start, end)| (start, end));
            let (start_row, start_col) = parse_textual_a1_point(start, provider.bounds)?;
            let (end_row, end_col) = parse_textual_a1_point(end, provider.bounds)?;
            Some(GridRect {
                workbook_id: provider.workbook_id.clone(),
                sheet_id: provider.sheet_id.clone(),
                top_row: start_row.min(end_row),
                left_col: start_col.min(end_col),
                bottom_row: start_row.max(end_row),
                right_col: start_col.max(end_col),
            })
        }
        ReferenceKind::Structured => provider
            .structured_reference_rect(target)
            .or_else(|| provider.defined_name_rect(target)),
        _ => None,
    }
}

fn excel_grid_structured_reference_key(text: &str) -> String {
    text.trim().to_ascii_uppercase()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExcelGridStructuredSection {
    All,
    Data,
    Headers,
    Totals,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedExcelGridStructuredReference {
    table_name: String,
    sections: Vec<ExcelGridStructuredSection>,
    column_start: Option<String>,
    column_end: Option<String>,
}

fn resolve_structured_reference_rects_from_tables(
    text: &str,
    tables: &BTreeMap<String, ExcelGridStructuredTable>,
) -> Option<Vec<GridRect>> {
    let parsed = parse_provider_structured_reference_text(text)?;
    let table = tables.get(&excel_grid_structured_reference_key(&parsed.table_name))?;
    resolve_provider_structured_reference_rects(table, &parsed)
}

fn parse_provider_structured_reference_text(
    text: &str,
) -> Option<ParsedExcelGridStructuredReference> {
    let text = text.trim();
    if text.starts_with('[') {
        return None;
    }
    let local = text
        .rsplit_once('!')
        .map_or(text, |(_, local_target)| local_target.trim());
    let bracket_index = local.find('[')?;
    let table_name = local[..bracket_index].trim();
    if table_name.is_empty() {
        return None;
    }
    let selector = local[bracket_index..].trim();
    if !selector.starts_with('[')
        || !selector.ends_with(']')
        || matching_structured_outer_bracket_end(selector)? != selector.len() - 1
    {
        return None;
    }
    let inner = &selector[1..selector.len() - 1];
    if inner.is_empty() {
        return None;
    }

    let mut sections = Vec::new();
    let mut column_start = None;
    let mut column_end = None;

    if inner.starts_with('[') {
        for segment in split_structured_top_level(inner, ',') {
            if let Some(raw) = strip_structured_brackets(segment.trim()) {
                if !raw.contains('\'') {
                    if let Some(parsed_section) = parse_provider_structured_section(raw) {
                        sections.push(parsed_section);
                        continue;
                    }
                }
            }
            if column_start.is_some() {
                return None;
            }
            let (start, end) = parse_provider_structured_column_segment(segment.trim())?;
            column_start = Some(start);
            column_end = end;
        }
    } else if let Some(raw_section) = parse_provider_structured_section(inner) {
        sections.push(raw_section);
    } else if inner.starts_with('@') {
        return None;
    } else {
        let column_name = unescape_provider_structured_text(inner);
        column_start = Some(column_name);
        column_end = None;
    }

    if sections.is_empty() {
        sections.push(ExcelGridStructuredSection::Data);
    }

    Some(ParsedExcelGridStructuredReference {
        table_name: table_name.to_string(),
        sections,
        column_start,
        column_end,
    })
}

fn resolve_provider_structured_reference_rects(
    table: &ExcelGridStructuredTable,
    parsed: &ParsedExcelGridStructuredReference,
) -> Option<Vec<GridRect>> {
    parsed
        .sections
        .iter()
        .map(|section| resolve_provider_structured_section_reference(table, *section, parsed))
        .collect()
}

fn resolve_provider_structured_section_reference(
    table: &ExcelGridStructuredTable,
    section: ExcelGridStructuredSection,
    parsed: &ParsedExcelGridStructuredReference,
) -> Option<GridRect> {
    match &parsed.column_start {
        Some(start_name) => {
            let start_index = table_column_index(table, start_name)?;
            let end_index = parsed
                .column_end
                .as_ref()
                .map_or(Some(start_index), |end_name| {
                    table_column_index(table, end_name)
                })?;
            let (first, last) = if start_index <= end_index {
                (start_index, end_index)
            } else {
                (end_index, start_index)
            };
            let columns = &table.columns[first..=last];
            structured_rect_for_columns(table, section, columns)
        }
        None => structured_rect_for_table_section(table, section),
    }
}

fn structured_rect_for_table_section(
    table: &ExcelGridStructuredTable,
    section: ExcelGridStructuredSection,
) -> Option<GridRect> {
    match section {
        ExcelGridStructuredSection::All => Some(table.table_range.clone()),
        ExcelGridStructuredSection::Data => structured_data_rect_for_columns(&table.columns),
        ExcelGridStructuredSection::Headers => table.header_rect.clone(),
        ExcelGridStructuredSection::Totals => table.totals_rect.clone(),
    }
}

fn structured_rect_for_columns(
    table: &ExcelGridStructuredTable,
    section: ExcelGridStructuredSection,
    columns: &[ExcelGridStructuredTableColumn],
) -> Option<GridRect> {
    if columns.is_empty() {
        return None;
    }
    let data_rect = structured_data_rect_for_columns(columns)?;
    match section {
        ExcelGridStructuredSection::Data => Some(data_rect),
        ExcelGridStructuredSection::All => Some(GridRect {
            workbook_id: table.table_range.workbook_id.clone(),
            sheet_id: table.table_range.sheet_id.clone(),
            top_row: table.table_range.top_row,
            left_col: data_rect.left_col,
            bottom_row: table.table_range.bottom_row,
            right_col: data_rect.right_col,
        }),
        ExcelGridStructuredSection::Headers => table.header_rect.as_ref().map(|header| {
            section_rect_for_column_span(header, data_rect.left_col, data_rect.right_col)
        }),
        ExcelGridStructuredSection::Totals => table.totals_rect.as_ref().map(|totals| {
            section_rect_for_column_span(totals, data_rect.left_col, data_rect.right_col)
        }),
    }
}

fn structured_data_rect_for_columns(
    columns: &[ExcelGridStructuredTableColumn],
) -> Option<GridRect> {
    let first = columns.first()?;
    let mut rect = first.data_rect.clone();
    for column in &columns[1..] {
        let column_rect = column.data_rect.clone();
        rect.top_row = rect.top_row.min(column_rect.top_row);
        rect.left_col = rect.left_col.min(column_rect.left_col);
        rect.bottom_row = rect.bottom_row.max(column_rect.bottom_row);
        rect.right_col = rect.right_col.max(column_rect.right_col);
    }
    Some(rect)
}

fn section_rect_for_column_span(
    section: &ExcelGridResolvedRect,
    left_col: u32,
    right_col: u32,
) -> GridRect {
    GridRect {
        workbook_id: section.workbook_id.clone(),
        sheet_id: section.sheet_id.clone(),
        top_row: section.top_row,
        left_col,
        bottom_row: section.bottom_row,
        right_col,
    }
}

fn table_column_index(table: &ExcelGridStructuredTable, column_name: &str) -> Option<usize> {
    table
        .columns
        .iter()
        .position(|column| column.column_name.eq_ignore_ascii_case(column_name))
}

fn parse_provider_structured_section(text: &str) -> Option<ExcelGridStructuredSection> {
    match text.trim().to_ascii_uppercase().as_str() {
        "#ALL" => Some(ExcelGridStructuredSection::All),
        "#DATA" => Some(ExcelGridStructuredSection::Data),
        "#HEADERS" => Some(ExcelGridStructuredSection::Headers),
        "#TOTALS" => Some(ExcelGridStructuredSection::Totals),
        _ => None,
    }
}

fn parse_provider_structured_column_segment(text: &str) -> Option<(String, Option<String>)> {
    if let Some((start, end)) = split_structured_top_level_once(text, ':') {
        let start = unescape_provider_structured_text(strip_structured_brackets(start.trim())?);
        let end = unescape_provider_structured_text(strip_structured_brackets(end.trim())?);
        return Some((start, Some(end)));
    }
    Some((
        unescape_provider_structured_text(strip_structured_brackets(text.trim())?),
        None,
    ))
}

fn strip_structured_brackets(text: &str) -> Option<&str> {
    let text = text.trim();
    if text.starts_with('[') {
        if !text.ends_with(']') || matching_structured_outer_bracket_end(text)? != text.len() - 1 {
            return None;
        }
        Some(&text[1..text.len() - 1])
    } else {
        Some(text)
    }
}

fn matching_structured_outer_bracket_end(text: &str) -> Option<usize> {
    let mut depth = 0u32;
    let mut escaped_next = false;
    for (index, ch) in text.char_indices() {
        if escaped_next {
            escaped_next = false;
            continue;
        }
        if ch == '\'' {
            escaped_next = true;
            continue;
        }
        match ch {
            '[' => depth = depth.saturating_add(1),
            ']' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn split_structured_top_level(text: &str, separator: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0u32;
    let mut start = 0usize;
    let mut escaped_next = false;
    for (index, ch) in text.char_indices() {
        if escaped_next {
            escaped_next = false;
            continue;
        }
        if ch == '\'' {
            escaped_next = true;
            continue;
        }
        match ch {
            '[' => depth = depth.saturating_add(1),
            ']' => depth = depth.saturating_sub(1),
            _ if ch == separator && depth == 0 => {
                parts.push(text[start..index].trim());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(text[start..].trim());
    parts
}

fn split_structured_top_level_once(text: &str, separator: char) -> Option<(&str, &str)> {
    let mut depth = 0u32;
    let mut escaped_next = false;
    for (index, ch) in text.char_indices() {
        if escaped_next {
            escaped_next = false;
            continue;
        }
        if ch == '\'' {
            escaped_next = true;
            continue;
        }
        match ch {
            '[' => depth = depth.saturating_add(1),
            ']' => depth = depth.saturating_sub(1),
            _ if ch == separator && depth == 0 => {
                let after = index + ch.len_utf8();
                return Some((&text[..index], &text[after..]));
            }
            _ => {}
        }
    }
    None
}

fn unescape_provider_structured_text(text: &str) -> String {
    let mut unescaped = String::new();
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch == '\'' {
            if let Some(next) = chars.next() {
                unescaped.push(next);
            }
        } else {
            unescaped.push(ch);
        }
    }
    unescaped
}

fn textual_grid_target_on_provider_sheet<'a>(
    target: &'a str,
    provider: &ExcelGridReferenceSystemProvider<'_>,
) -> Option<&'a str> {
    let target = target.trim();
    if let Some((sheet, local_target)) = target.rsplit_once('!') {
        if sheet.trim_matches('\'') == provider.sheet_id {
            return Some(local_target.trim());
        }
        return None;
    }
    Some(target)
}

fn parse_textual_a1_point(target: &str, bounds: ExcelGridBounds) -> Option<(u32, u32)> {
    let mut rest = target.trim();
    if let Some(after_dollar) = rest.strip_prefix('$') {
        rest = after_dollar;
    }
    let col_len = rest
        .chars()
        .take_while(|ch| ch.is_ascii_alphabetic())
        .map(char::len_utf8)
        .sum::<usize>();
    if col_len == 0 {
        return None;
    }
    let col = column_to_index(&rest[..col_len])?;
    rest = &rest[col_len..];
    if let Some(after_dollar) = rest.strip_prefix('$') {
        rest = after_dollar;
    }
    let row_len = rest
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .map(char::len_utf8)
        .sum::<usize>();
    if row_len == 0 || row_len != rest.len() {
        return None;
    }
    let row = rest.parse::<u32>().ok()?;
    (bounds.contains_row(row) && bounds.contains_col(col)).then_some((row, col))
}

fn instantiate_cell_axes(
    axes: &str,
    provider: &ExcelGridReferenceSystemProvider<'_>,
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

fn profile_record_for_name_reference(
    profile_id: &str,
    request: &ReferenceNameBindRequest,
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

fn profile_record_for_structured_reference(
    profile_id: &str,
    request: &ReferenceStructuredBindRequest,
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
            parsed_qualifier: None,
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

fn profile_record_for_transformed_reference(
    original: &ProfileReferenceRecord,
    reference: ExcelGridReference,
    validity: ReferenceValidity,
    anchor_after: Option<&ExcelGridFormulaAnchor>,
) -> ProfileReferenceRecord {
    let normal_form_key = normal_form_key_for_reference(&original.profile_id, &reference);
    let render_hint = render_reference_for_channel(
        &reference,
        original.source_info.source_channel,
        anchor_after,
    )
    .unwrap_or_else(|| normal_form_key.0.clone());
    let payload_data =
        serde_json::to_string(&reference).expect("excel grid reference payload serializes");
    let mut source_info = original.source_info.clone();
    source_info.source_text = render_hint.clone();
    source_info.address_fidelity = Some(render_hint.clone());
    source_info.parsed_qualifier = transformed_parsed_qualifier(&reference);
    ProfileReferenceRecord {
        profile_id: original.profile_id.clone(),
        profile_version: original.profile_version.clone(),
        source_info,
        profile_payload: ProfilePayload {
            payload_kind: "excel-grid-reference".to_string(),
            encoding: "json".to_string(),
            data: payload_data,
        },
        normal_form_key,
        render_hint: Some(render_hint),
        validity,
    }
}

fn transformed_parsed_qualifier(reference: &ExcelGridReference) -> Option<String> {
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

fn decode_excel_grid_transform_payload(
    payload: &ProfilePayload,
) -> Option<ExcelGridReferenceTransformPayload> {
    if payload.payload_kind != EXCEL_GRID_STRUCTURAL_EDIT_PAYLOAD_KIND || payload.encoding != "json"
    {
        return None;
    }
    serde_json::from_str(&payload.data).ok()
}

fn transform_excel_grid_reference(
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

fn validate_structural_edit(
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

fn render_reference_for_channel(
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
        | ExcelGridReference::Name { source_text, .. } => Some(source_text.clone()),
    }
}

fn render_cell_reference(
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

fn render_row_axis(
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

fn render_col_axis(
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

fn with_optional_qualifier(qualifier: Option<&str>, local: &str) -> String {
    qualifier.map_or_else(
        || local.to_string(),
        |qualifier| format!("{qualifier}!{local}"),
    )
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

fn looks_like_a1_reference_name(name: &str) -> bool {
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

fn looks_like_r1c1_reference_name(name: &str) -> bool {
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

fn is_r1c1_axis_fragment(fragment: &str) -> bool {
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

fn index_to_column(mut index: u32) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use oxfml_core::binding::{
        BindContext, BindRequest, BoundExpr, BoundFormula, NormalizedReference,
        ReferenceBindProfile, ReferenceExpr, ReferenceTransformKind, ReferenceTransformOutcome,
        ReferenceTransformRequest,
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
    fn strict_profile_structural_insert_expands_area_reference() {
        let profile = StrictExcelGridReferenceProfile::new();
        let record = test_profile_record(ExcelGridReference::Area {
            workbook_id: "book:default".to_string(),
            sheet_id: "sheet:default".to_string(),
            start_row: ExcelGridAxisRef::Relative(-1),
            start_col: ExcelGridAxisRef::Relative(-2),
            end_row: ExcelGridAxisRef::Relative(1),
            end_col: ExcelGridAxisRef::Relative(-1),
            source_style: ExcelGridReferenceStyle::A1,
            source_text: "A1:B3".to_string(),
            parsed_qualifier: None,
        });

        let result = profile.transform_reference(&ReferenceTransformRequest {
            reference: record,
            transform_kind: ReferenceTransformKind::StructuralEdit,
            payload: Some(structural_payload(
                ExcelGridStructuralEdit::insert_rows("book:default", "sheet:default", 2, 1),
                2,
                3,
            )),
        });

        assert_eq!(result.outcome, ReferenceTransformOutcome::Expanded);
        let transformed = result.reference.as_ref().expect("transformed reference");
        assert_eq!(transformed.render_hint.as_deref(), Some("A1:B4"));
        match decode_excel_grid_reference_payload(&transformed.profile_payload)
            .expect("transformed grid payload")
        {
            ExcelGridReference::Area {
                start_row,
                start_col,
                end_row,
                end_col,
                source_text,
                ..
            } => {
                assert_eq!(start_row, ExcelGridAxisRef::Relative(-2));
                assert_eq!(start_col, ExcelGridAxisRef::Relative(-2));
                assert_eq!(end_row, ExcelGridAxisRef::Relative(1));
                assert_eq!(end_col, ExcelGridAxisRef::Relative(-1));
                assert_eq!(source_text, "A1:B4");
            }
            other => panic!("expected transformed area, got {other:?}"),
        }
    }

    #[test]
    fn strict_profile_structural_delete_shrinks_area_reference() {
        let profile = StrictExcelGridReferenceProfile::new();
        let record = test_profile_record(ExcelGridReference::Area {
            workbook_id: "book:default".to_string(),
            sheet_id: "sheet:default".to_string(),
            start_row: ExcelGridAxisRef::Relative(0),
            start_col: ExcelGridAxisRef::Relative(-4),
            end_row: ExcelGridAxisRef::Relative(0),
            end_col: ExcelGridAxisRef::Relative(-2),
            source_style: ExcelGridReferenceStyle::A1,
            source_text: "A1:C1".to_string(),
            parsed_qualifier: None,
        });

        let result = profile.transform_reference(&ReferenceTransformRequest {
            reference: record,
            transform_kind: ReferenceTransformKind::StructuralEdit,
            payload: Some(structural_payload(
                ExcelGridStructuralEdit::delete_columns("book:default", "sheet:default", 2, 1),
                1,
                5,
            )),
        });

        assert_eq!(result.outcome, ReferenceTransformOutcome::Shrunk);
        let transformed = result.reference.as_ref().expect("transformed reference");
        assert_eq!(transformed.render_hint.as_deref(), Some("A1:B1"));
        match decode_excel_grid_reference_payload(&transformed.profile_payload)
            .expect("transformed grid payload")
        {
            ExcelGridReference::Area {
                start_col,
                end_col,
                source_text,
                ..
            } => {
                assert_eq!(start_col, ExcelGridAxisRef::Relative(-3));
                assert_eq!(end_col, ExcelGridAxisRef::Relative(-2));
                assert_eq!(source_text, "A1:B1");
            }
            other => panic!("expected transformed area, got {other:?}"),
        }
    }

    #[test]
    fn strict_profile_structural_delete_turns_deleted_point_into_ref_error() {
        let profile = StrictExcelGridReferenceProfile::new();
        let bound = bind_for(
            "strict-transform-deleted-point",
            "=A2",
            FormulaChannelKind::WorksheetA1,
            5,
            4,
            &profile,
        );
        let record = profile_record(&bound.normalized_references[0]).clone();

        let result = profile.transform_reference(&ReferenceTransformRequest {
            reference: record,
            transform_kind: ReferenceTransformKind::StructuralEdit,
            payload: Some(structural_payload(
                ExcelGridStructuralEdit::delete_rows("book:default", "sheet:default", 2, 1),
                5,
                4,
            )),
        });

        assert_eq!(result.outcome, ReferenceTransformOutcome::FullyInvalid);
        let transformed = result.reference.as_ref().expect("transformed reference");
        assert_eq!(transformed.validity, ReferenceValidity::InvalidStatic);
        assert_eq!(transformed.render_hint.as_deref(), Some("#REF!"));
        assert!(matches!(
            decode_excel_grid_reference_payload(&transformed.profile_payload)
                .expect("transformed grid payload"),
            ExcelGridReference::RefError { .. }
        ));
    }

    #[test]
    fn strict_profile_structural_insert_expands_whole_row_reference() {
        let profile = StrictExcelGridReferenceProfile::new();
        let bound = bind_for(
            "strict-transform-whole-row",
            "=1:3",
            FormulaChannelKind::WorksheetA1,
            5,
            5,
            &profile,
        );
        let record = profile_record(&bound.normalized_references[0]).clone();

        let result = profile.transform_reference(&ReferenceTransformRequest {
            reference: record,
            transform_kind: ReferenceTransformKind::StructuralEdit,
            payload: Some(structural_payload(
                ExcelGridStructuralEdit::insert_rows("book:default", "sheet:default", 2, 1),
                5,
                5,
            )),
        });

        assert_eq!(result.outcome, ReferenceTransformOutcome::Expanded);
        let transformed = result.reference.as_ref().expect("transformed reference");
        assert_eq!(transformed.render_hint.as_deref(), Some("1:4"));
        match decode_excel_grid_reference_payload(&transformed.profile_payload)
            .expect("transformed grid payload")
        {
            ExcelGridReference::WholeRow {
                start_row,
                end_row,
                source_text,
                ..
            } => {
                assert_eq!(start_row, ExcelGridAxisRef::Relative(-5));
                assert_eq!(end_row, ExcelGridAxisRef::Relative(-2));
                assert_eq!(source_text, "1:4");
            }
            other => panic!("expected transformed whole-row reference, got {other:?}"),
        }
    }

    #[test]
    fn strict_profile_structural_insert_expands_whole_column_reference() {
        let profile = StrictExcelGridReferenceProfile::new();
        let bound = bind_for(
            "strict-transform-whole-column",
            "=A:C",
            FormulaChannelKind::WorksheetA1,
            5,
            5,
            &profile,
        );
        let record = profile_record(&bound.normalized_references[0]).clone();

        let result = profile.transform_reference(&ReferenceTransformRequest {
            reference: record,
            transform_kind: ReferenceTransformKind::StructuralEdit,
            payload: Some(structural_payload(
                ExcelGridStructuralEdit::insert_columns("book:default", "sheet:default", 2, 1),
                5,
                5,
            )),
        });

        assert_eq!(result.outcome, ReferenceTransformOutcome::Expanded);
        let transformed = result.reference.as_ref().expect("transformed reference");
        assert_eq!(transformed.render_hint.as_deref(), Some("A:D"));
        match decode_excel_grid_reference_payload(&transformed.profile_payload)
            .expect("transformed grid payload")
        {
            ExcelGridReference::WholeColumn {
                start_col,
                end_col,
                source_text,
                ..
            } => {
                assert_eq!(start_col, ExcelGridAxisRef::Relative(-5));
                assert_eq!(end_col, ExcelGridAxisRef::Relative(-2));
                assert_eq!(source_text, "A:D");
            }
            other => panic!("expected transformed whole-column reference, got {other:?}"),
        }
    }

    #[test]
    fn strict_profile_structural_insert_preserves_r1c1_relative_shape() {
        let profile = StrictExcelGridReferenceProfile::new();
        let bound = bind_for(
            "strict-transform-r1c1-point",
            "=R[-1]C",
            FormulaChannelKind::WorksheetR1C1,
            5,
            3,
            &profile,
        );
        let record = profile_record(&bound.normalized_references[0]).clone();

        let result = profile.transform_reference(&ReferenceTransformRequest {
            reference: record,
            transform_kind: ReferenceTransformKind::StructuralEdit,
            payload: Some(structural_payload(
                ExcelGridStructuralEdit::insert_rows("book:default", "sheet:default", 4, 1),
                5,
                3,
            )),
        });

        assert_eq!(result.outcome, ReferenceTransformOutcome::Shifted);
        let transformed = result.reference.as_ref().expect("transformed reference");
        assert_eq!(transformed.render_hint.as_deref(), Some("R[-1]C"));
        match decode_excel_grid_reference_payload(&transformed.profile_payload)
            .expect("transformed grid payload")
        {
            ExcelGridReference::Cell {
                row,
                col,
                source_text,
                ..
            } => {
                assert_eq!(row, ExcelGridAxisRef::Relative(-1));
                assert_eq!(col, ExcelGridAxisRef::Relative(0));
                assert_eq!(source_text, "R[-1]C");
            }
            other => panic!("expected transformed R1C1 cell reference, got {other:?}"),
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
    fn strict_grid_provider_composes_cell_range_to_sparse_area() {
        let profile = StrictExcelGridReferenceProfile::new();
        let bound = bind_for(
            "strict-provider-compose-range",
            "=A1+B1",
            FormulaChannelKind::WorksheetA1,
            1,
            3,
            &profile,
        );
        let lhs = excel_grid_reference_like_from_profile_record(profile_record(
            &bound.normalized_references[0],
        ))
        .expect("left endpoint should lower to ReferenceLike");
        let rhs = excel_grid_reference_like_from_profile_record(profile_record(
            &bound.normalized_references[1],
        ))
        .expect("right endpoint should lower to ReferenceLike");
        let provider = ExcelGridReferenceSystemProvider::new("book:default", "sheet:default", 1, 3)
            .with_cell_value(
                "book:default",
                "sheet:default",
                1,
                1,
                CalcValue::number(2.0),
            )
            .with_cell_value(
                "book:default",
                "sheet:default",
                1,
                2,
                CalcValue::number(3.0),
            );

        let area = provider
            .compose_references(&ReferenceComposeRequest {
                lhs,
                rhs,
                operation: ReferenceComposeOperation::Range,
            })
            .expect("grid provider should compose same-sheet cell endpoints into an area");
        let values = provider
            .enumerate_values(&ReferenceEnumerationRequest { reference: area })
            .expect("composed area should enumerate")
            .expect("grid area should return sparse values");

        assert_eq!(values.declared_extent, ResolvedReferenceExtent::new(1, 2));
        assert_eq!(
            values.defined_cells,
            vec![
                ResolvedReferenceCell::new(1, 1, CalcValue::number(2.0)),
                ResolvedReferenceCell::new(1, 2, CalcValue::number(3.0)),
            ]
        );
    }

    #[test]
    fn strict_grid_provider_composes_textual_a1_range_to_sparse_area() {
        let provider = ExcelGridReferenceSystemProvider::new("book:default", "sheet:default", 1, 3)
            .with_cell_value(
                "book:default",
                "sheet:default",
                1,
                1,
                CalcValue::number(2.0),
            )
            .with_cell_value(
                "book:default",
                "sheet:default",
                1,
                2,
                CalcValue::number(3.0),
            );

        let area = provider
            .compose_references(&ReferenceComposeRequest {
                lhs: ReferenceLike::new(ReferenceKind::A1, "A1"),
                rhs: ReferenceLike::new(ReferenceKind::A1, "B1"),
                operation: ReferenceComposeOperation::Range,
            })
            .expect("grid provider should compose textual same-sheet A1 endpoints into an area");
        let values = provider
            .enumerate_values(&ReferenceEnumerationRequest { reference: area })
            .expect("textual composed area should enumerate")
            .expect("grid area should return sparse values");

        assert_eq!(values.declared_extent, ResolvedReferenceExtent::new(1, 2));
        assert_eq!(
            values.defined_cells,
            vec![
                ResolvedReferenceCell::new(1, 1, CalcValue::number(2.0)),
                ResolvedReferenceCell::new(1, 2, CalcValue::number(3.0)),
            ]
        );
    }

    #[test]
    fn strict_grid_provider_composed_area_feeds_oxfunc_sum() {
        let profile = StrictExcelGridReferenceProfile::new();
        let bound = bind_for(
            "strict-provider-compose-range-sum",
            "=A1+B1",
            FormulaChannelKind::WorksheetA1,
            1,
            3,
            &profile,
        );
        let lhs = excel_grid_reference_like_from_profile_record(profile_record(
            &bound.normalized_references[0],
        ))
        .expect("left endpoint should lower to ReferenceLike");
        let rhs = excel_grid_reference_like_from_profile_record(profile_record(
            &bound.normalized_references[1],
        ))
        .expect("right endpoint should lower to ReferenceLike");
        let provider = ExcelGridReferenceSystemProvider::new("book:default", "sheet:default", 1, 3)
            .with_cell_value(
                "book:default",
                "sheet:default",
                1,
                1,
                CalcValue::number(2.0),
            )
            .with_cell_value(
                "book:default",
                "sheet:default",
                1,
                2,
                CalcValue::number(3.0),
            );
        let area = provider
            .compose_references(&ReferenceComposeRequest {
                lhs,
                rhs,
                operation: ReferenceComposeOperation::Range,
            })
            .expect("grid provider should compose same-sheet cell endpoints into an area");

        let sum =
            oxfunc_core::functions::sum::eval_sum_surface(&[CalcValue::reference(area)], &provider)
                .expect("SUM should expand the composed grid area through sparse enumeration");

        assert_eq!(sum, CalcValue::number(5.0));
    }

    #[test]
    fn strict_grid_provider_composes_union_to_multi_area_sparse_values() {
        let provider = ExcelGridReferenceSystemProvider::new("book:default", "sheet:default", 1, 4)
            .with_cell_value(
                "book:default",
                "sheet:default",
                1,
                1,
                CalcValue::number(2.0),
            )
            .with_cell_value(
                "book:default",
                "sheet:default",
                1,
                3,
                CalcValue::number(5.0),
            );

        let union = provider
            .compose_references(&ReferenceComposeRequest {
                lhs: ReferenceLike::new(ReferenceKind::A1, "A1"),
                rhs: ReferenceLike::new(ReferenceKind::A1, "C1"),
                operation: ReferenceComposeOperation::Union,
            })
            .expect("grid provider should compose same-sheet references into a multi-area union");
        assert_eq!(union.kind(), ReferenceKind::MultiArea);

        let values = provider
            .enumerate_values(&ReferenceEnumerationRequest {
                reference: union.clone(),
            })
            .expect("composed union should enumerate")
            .expect("grid union should return sparse values");
        assert_eq!(values.declared_extent, ResolvedReferenceExtent::new(1, 2));
        assert_eq!(
            values.defined_cells,
            vec![
                ResolvedReferenceCell::new(1, 1, CalcValue::number(2.0)),
                ResolvedReferenceCell::new(1, 2, CalcValue::number(5.0)),
            ]
        );

        let sum = oxfunc_core::functions::sum::eval_sum_surface(
            &[CalcValue::reference(union)],
            &provider,
        )
        .expect("SUM should consume grid union sparse values");
        assert_eq!(sum, CalcValue::number(7.0));
    }

    #[test]
    fn strict_profile_binds_defined_name_symbolically() {
        let profile = StrictExcelGridReferenceProfile::new();
        let bound = bind_for(
            "strict-defined-name-bind",
            "=InputRange",
            FormulaChannelKind::WorksheetA1,
            5,
            3,
            &profile,
        );

        assert_eq!(bound.normalized_references.len(), 1);
        let record = profile_record(&bound.normalized_references[0]);
        assert_eq!(record.validity, ReferenceValidity::DynamicOrHostSensitive);
        assert_eq!(
            record.normal_form_key.0,
            "excel.grid.v1:name:book%3Adefault:sheet%3Adefault:InputRange"
        );
        match decode_excel_grid_reference_payload(&record.profile_payload)
            .expect("defined name payload")
        {
            ExcelGridReference::Name {
                name, source_text, ..
            } => {
                assert_eq!(name, "InputRange");
                assert_eq!(source_text, "InputRange");
            }
            other => panic!("expected name reference payload, got {other:?}"),
        }
    }

    #[test]
    fn strict_grid_provider_resolves_defined_name_to_sparse_area() {
        let profile = StrictExcelGridReferenceProfile::new();
        let bound = bind_for(
            "strict-defined-name-provider",
            "=InputRange",
            FormulaChannelKind::WorksheetA1,
            5,
            3,
            &profile,
        );
        let reference = excel_grid_reference_like_from_profile_record(profile_record(
            &bound.normalized_references[0],
        ))
        .expect("defined name should lower to ReferenceLike");
        let provider = ExcelGridReferenceSystemProvider::new("book:default", "sheet:default", 5, 3)
            .with_cell_value(
                "book:default",
                "sheet:default",
                1,
                1,
                CalcValue::number(2.0),
            )
            .with_cell_value(
                "book:default",
                "sheet:default",
                2,
                1,
                CalcValue::number(4.0),
            )
            .with_defined_name(
                "InputRange",
                ExcelGridResolvedRect {
                    workbook_id: "book:default".to_string(),
                    sheet_id: "sheet:default".to_string(),
                    top_row: 1,
                    left_col: 1,
                    bottom_row: 3,
                    right_col: 1,
                },
            );

        let values = provider
            .enumerate_values(&ReferenceEnumerationRequest { reference })
            .expect("defined name reference should enumerate")
            .expect("defined name should resolve to sparse values");

        assert_eq!(values.declared_extent, ResolvedReferenceExtent::new(3, 1));
        assert_eq!(
            values.defined_cells,
            vec![
                ResolvedReferenceCell::new(1, 1, CalcValue::number(2.0)),
                ResolvedReferenceCell::new(2, 1, CalcValue::number(4.0)),
            ]
        );
    }

    #[test]
    fn strict_grid_provider_resolves_structured_sections_and_escaped_columns() {
        let profile = StrictExcelGridReferenceProfile::new();
        let bound = bind_for(
            "strict-structured-section-provider",
            "=Table1[[#Data],[Amount]:[Tax]]",
            FormulaChannelKind::WorksheetA1,
            5,
            4,
            &profile,
        );
        let reference = excel_grid_reference_like_from_profile_record(profile_record(
            &bound.normalized_references[0],
        ))
        .expect("structured reference should lower to ReferenceLike");
        let provider = ExcelGridReferenceSystemProvider::new("book:default", "sheet:default", 5, 4)
            .with_cell_value(
                "book:default",
                "sheet:default",
                2,
                2,
                CalcValue::number(2.0),
            )
            .with_cell_value(
                "book:default",
                "sheet:default",
                4,
                3,
                CalcValue::number(3.0),
            )
            .with_cell_value(
                "book:default",
                "sheet:default",
                5,
                2,
                CalcValue::number(12.0),
            )
            .with_cell_value(
                "book:default",
                "sheet:default",
                5,
                3,
                CalcValue::number(6.0),
            )
            .with_structured_table(
                ExcelGridStructuredTable::new(
                    "Table1",
                    ExcelGridResolvedRect {
                        workbook_id: "book:default".to_string(),
                        sheet_id: "sheet:default".to_string(),
                        top_row: 1,
                        left_col: 1,
                        bottom_row: 5,
                        right_col: 3,
                    },
                    vec![
                        ExcelGridStructuredTableColumn::new(
                            "Label",
                            1,
                            ExcelGridResolvedRect {
                                workbook_id: "book:default".to_string(),
                                sheet_id: "sheet:default".to_string(),
                                top_row: 2,
                                left_col: 1,
                                bottom_row: 4,
                                right_col: 1,
                            },
                        ),
                        ExcelGridStructuredTableColumn::new(
                            "Amount",
                            2,
                            ExcelGridResolvedRect {
                                workbook_id: "book:default".to_string(),
                                sheet_id: "sheet:default".to_string(),
                                top_row: 2,
                                left_col: 2,
                                bottom_row: 4,
                                right_col: 2,
                            },
                        ),
                        ExcelGridStructuredTableColumn::new(
                            "Tax",
                            3,
                            ExcelGridResolvedRect {
                                workbook_id: "book:default".to_string(),
                                sheet_id: "sheet:default".to_string(),
                                top_row: 2,
                                left_col: 3,
                                bottom_row: 4,
                                right_col: 3,
                            },
                        ),
                    ],
                )
                .with_header_rect(ExcelGridResolvedRect {
                    workbook_id: "book:default".to_string(),
                    sheet_id: "sheet:default".to_string(),
                    top_row: 1,
                    left_col: 1,
                    bottom_row: 1,
                    right_col: 3,
                })
                .with_totals_rect(ExcelGridResolvedRect {
                    workbook_id: "book:default".to_string(),
                    sheet_id: "sheet:default".to_string(),
                    top_row: 5,
                    left_col: 1,
                    bottom_row: 5,
                    right_col: 3,
                }),
            )
            .with_structured_table(ExcelGridStructuredTable::new(
                "TableEsc",
                ExcelGridResolvedRect {
                    workbook_id: "book:default".to_string(),
                    sheet_id: "sheet:default".to_string(),
                    top_row: 6,
                    left_col: 1,
                    bottom_row: 9,
                    right_col: 3,
                },
                vec![
                    ExcelGridStructuredTableColumn::new(
                        "Label",
                        1,
                        ExcelGridResolvedRect {
                            workbook_id: "book:default".to_string(),
                            sheet_id: "sheet:default".to_string(),
                            top_row: 7,
                            left_col: 1,
                            bottom_row: 9,
                            right_col: 1,
                        },
                    ),
                    ExcelGridStructuredTableColumn::new(
                        "#Data",
                        2,
                        ExcelGridResolvedRect {
                            workbook_id: "book:default".to_string(),
                            sheet_id: "sheet:default".to_string(),
                            top_row: 7,
                            left_col: 2,
                            bottom_row: 9,
                            right_col: 2,
                        },
                    ),
                    ExcelGridStructuredTableColumn::new(
                        "Gross]Margin",
                        3,
                        ExcelGridResolvedRect {
                            workbook_id: "book:default".to_string(),
                            sheet_id: "sheet:default".to_string(),
                            top_row: 7,
                            left_col: 3,
                            bottom_row: 9,
                            right_col: 3,
                        },
                    ),
                ],
            ));

        let values = provider
            .enumerate_values(&ReferenceEnumerationRequest { reference })
            .expect("structured reference should enumerate")
            .expect("structured reference should resolve to sparse values");

        assert_eq!(values.declared_extent, ResolvedReferenceExtent::new(3, 2));
        assert_eq!(
            values.defined_cells,
            vec![
                ResolvedReferenceCell::new(1, 1, CalcValue::number(2.0)),
                ResolvedReferenceCell::new(3, 2, CalcValue::number(3.0)),
            ]
        );

        let escaped = provider
            .resolve_text(&ReferenceTextResolveRequest {
                text: "TableEsc[[#Data],['#Data]:[Gross']Margin]]".to_string(),
                mode: ReferenceTextResolutionMode::Indirect,
                a1_style: Some(true),
                caller_context: provider.caller_context(),
            })
            .expect("escaped structured text should resolve");
        let escaped_rect = provider
            .resolved_rect_for_reference(&escaped)
            .expect("escaped structured text should round-trip to a provider rect");
        assert_eq!(escaped_rect.top_row, 7);
        assert_eq!(escaped_rect.left_col, 2);
        assert_eq!(escaped_rect.bottom_row, 9);
        assert_eq!(escaped_rect.right_col, 3);

        let multi_bound = bind_for(
            "strict-structured-multi-section-provider",
            "=Table1[[#Headers],[#Totals],[Amount]:[Tax]]",
            FormulaChannelKind::WorksheetA1,
            5,
            5,
            &profile,
        );
        let multi_reference = excel_grid_reference_like_from_profile_record(profile_record(
            &multi_bound.normalized_references[0],
        ))
        .expect("multi-section structured reference should lower to ReferenceLike");
        let multi_values = provider
            .enumerate_values(&ReferenceEnumerationRequest {
                reference: multi_reference.clone(),
            })
            .expect("multi-section structured reference should enumerate")
            .expect("multi-section structured reference should resolve to sparse values");

        assert_eq!(
            multi_values.declared_extent,
            ResolvedReferenceExtent::new(1, 4)
        );
        assert_eq!(
            multi_values.defined_cells,
            vec![
                ResolvedReferenceCell::new(1, 3, CalcValue::number(12.0)),
                ResolvedReferenceCell::new(1, 4, CalcValue::number(6.0)),
            ]
        );
        let sum = oxfunc_core::functions::sum::eval_sum_surface(
            &[CalcValue::reference(multi_reference)],
            &provider,
        )
        .expect("SUM should consume sparse multi-section structured values");
        assert_eq!(sum, CalcValue::number(18.0));

        let indirect_multi = provider
            .resolve_text(&ReferenceTextResolveRequest {
                text: "Table1[[#Headers],[#Totals],[Amount]:[Tax]]".to_string(),
                mode: ReferenceTextResolutionMode::Indirect,
                a1_style: Some(true),
                caller_context: provider.caller_context(),
            })
            .expect("multi-section structured text should resolve");
        assert_eq!(indirect_multi.kind(), ReferenceKind::MultiArea);
        let indirect_sum = oxfunc_core::functions::sum::eval_sum_surface(
            &[CalcValue::reference(indirect_multi)],
            &provider,
        )
        .expect("INDIRECT multi-section structured values should feed SUM");
        assert_eq!(indirect_sum, CalcValue::number(18.0));
    }

    #[test]
    fn strict_grid_provider_resolves_caller_local_table_column_name() {
        let profile = StrictExcelGridReferenceProfile::new();
        let bound = bind_for(
            "strict-caller-local-table-column",
            "=[Amount]",
            FormulaChannelKind::WorksheetA1,
            3,
            3,
            &profile,
        );
        let reference = excel_grid_reference_like_from_profile_record(profile_record(
            &bound.normalized_references[0],
        ))
        .expect("bracketed table column should lower through the strict profile");
        let provider = ExcelGridReferenceSystemProvider::new("book:default", "sheet:default", 3, 3)
            .with_cell_value(
                "book:default",
                "sheet:default",
                2,
                2,
                CalcValue::number(2.0),
            )
            .with_cell_value(
                "book:default",
                "sheet:default",
                3,
                2,
                CalcValue::number(4.0),
            )
            .with_cell_value(
                "book:default",
                "sheet:default",
                4,
                2,
                CalcValue::number(6.0),
            )
            .with_structured_table(ExcelGridStructuredTable::new(
                "Table1",
                ExcelGridResolvedRect {
                    workbook_id: "book:default".to_string(),
                    sheet_id: "sheet:default".to_string(),
                    top_row: 1,
                    left_col: 1,
                    bottom_row: 4,
                    right_col: 3,
                },
                vec![
                    ExcelGridStructuredTableColumn::new(
                        "Label",
                        1,
                        ExcelGridResolvedRect {
                            workbook_id: "book:default".to_string(),
                            sheet_id: "sheet:default".to_string(),
                            top_row: 2,
                            left_col: 1,
                            bottom_row: 4,
                            right_col: 1,
                        },
                    ),
                    ExcelGridStructuredTableColumn::new(
                        "Amount",
                        2,
                        ExcelGridResolvedRect {
                            workbook_id: "book:default".to_string(),
                            sheet_id: "sheet:default".to_string(),
                            top_row: 2,
                            left_col: 2,
                            bottom_row: 4,
                            right_col: 2,
                        },
                    ),
                ],
            ));

        let values = provider
            .enumerate_values(&ReferenceEnumerationRequest { reference })
            .expect("caller-local table column should enumerate")
            .expect("caller-local table column should resolve to sparse values");

        assert_eq!(values.declared_extent, ResolvedReferenceExtent::new(3, 1));
        assert_eq!(
            values.defined_cells,
            vec![
                ResolvedReferenceCell::new(1, 1, CalcValue::number(2.0)),
                ResolvedReferenceCell::new(2, 1, CalcValue::number(4.0)),
                ResolvedReferenceCell::new(3, 1, CalcValue::number(6.0)),
            ]
        );
    }

    #[test]
    fn strict_grid_provider_resolves_indirect_defined_name_text() {
        let provider = ExcelGridReferenceSystemProvider::new("book:default", "sheet:default", 5, 3)
            .with_cell_value(
                "book:default",
                "sheet:default",
                1,
                1,
                CalcValue::number(3.0),
            )
            .with_cell_value(
                "book:default",
                "sheet:default",
                2,
                1,
                CalcValue::number(5.0),
            )
            .with_defined_name(
                "InputRange",
                ExcelGridResolvedRect {
                    workbook_id: "book:default".to_string(),
                    sheet_id: "sheet:default".to_string(),
                    top_row: 1,
                    left_col: 1,
                    bottom_row: 2,
                    right_col: 1,
                },
            );

        let reference = provider
            .resolve_text(&ReferenceTextResolveRequest {
                text: "InputRange".to_string(),
                mode: ReferenceTextResolutionMode::Indirect,
                a1_style: Some(true),
                caller_context: provider.caller_context(),
            })
            .expect("defined name text should resolve");
        let values = provider
            .enumerate_values(&ReferenceEnumerationRequest { reference })
            .expect("resolved defined name should enumerate")
            .expect("defined name should produce sparse values");

        assert_eq!(values.declared_extent, ResolvedReferenceExtent::new(2, 1));
        assert_eq!(
            values.defined_cells,
            vec![
                ResolvedReferenceCell::new(1, 1, CalcValue::number(3.0)),
                ResolvedReferenceCell::new(2, 1, CalcValue::number(5.0)),
            ]
        );
    }

    #[test]
    fn strict_grid_provider_enumerates_spill_anchor_extent() {
        let provider = ExcelGridReferenceSystemProvider::new("book:default", "sheet:default", 1, 3)
            .with_cell_value(
                "book:default",
                "sheet:default",
                1,
                1,
                CalcValue::number(10.0),
            )
            .with_cell_value(
                "book:default",
                "sheet:default",
                2,
                1,
                CalcValue::number(20.0),
            )
            .with_spill_extent(
                "book:default",
                "sheet:default",
                1,
                1,
                ExcelGridResolvedRect {
                    workbook_id: "book:default".to_string(),
                    sheet_id: "sheet:default".to_string(),
                    top_row: 1,
                    left_col: 1,
                    bottom_row: 2,
                    right_col: 1,
                },
            );

        let values = provider
            .enumerate_values(&ReferenceEnumerationRequest {
                reference: ReferenceLike::new(ReferenceKind::SpillAnchor, "A1#"),
            })
            .expect("spill anchor should enumerate")
            .expect("spill anchor should resolve to recorded sparse extent");

        assert_eq!(values.declared_extent, ResolvedReferenceExtent::new(2, 1));
        assert_eq!(
            values.defined_cells,
            vec![
                ResolvedReferenceCell::new(1, 1, CalcValue::number(10.0)),
                ResolvedReferenceCell::new(2, 1, CalcValue::number(20.0)),
            ]
        );
    }

    #[test]
    fn strict_grid_provider_reports_spill_anchor_ledger_probe_floor() {
        let provider = ExcelGridReferenceSystemProvider::new("book:default", "sheet:default", 1, 3)
            .with_cell_value(
                "book:default",
                "sheet:default",
                1,
                1,
                CalcValue::number(10.0),
            )
            .with_cell_value(
                "book:default",
                "sheet:default",
                1_000_000,
                1,
                CalcValue::number(20.0),
            )
            .with_spill_extent(
                "book:default",
                "sheet:default",
                1,
                1,
                ExcelGridResolvedRect {
                    workbook_id: "book:default".to_string(),
                    sheet_id: "sheet:default".to_string(),
                    top_row: 1,
                    left_col: 1,
                    bottom_row: 1_000_000,
                    right_col: 1,
                },
            );

        let report = provider
            .spill_anchor_dereference_report(&ReferenceLike::new(ReferenceKind::SpillAnchor, "A1#"))
            .expect("spill anchor should report provider ledger probes");

        assert_eq!(report.declared_cell_count, 1_000_000);
        assert_eq!(report.ledger_probe_count, 1);
        assert_eq!(report.extent_cells_scanned_for_ledger, 0);
        assert_eq!(report.value_entries_scanned, 2);
        assert_eq!(report.defined_cells_returned, 2);
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

    fn test_profile_record(reference: ExcelGridReference) -> ProfileReferenceRecord {
        let source_text = match &reference {
            ExcelGridReference::Cell { source_text, .. }
            | ExcelGridReference::Area { source_text, .. }
            | ExcelGridReference::WholeRow { source_text, .. }
            | ExcelGridReference::WholeColumn { source_text, .. }
            | ExcelGridReference::SpillAnchor { source_text, .. }
            | ExcelGridReference::StructuredReference { source_text, .. }
            | ExcelGridReference::Name { source_text, .. }
            | ExcelGridReference::RefError { source_text, .. } => source_text.clone(),
        };
        ProfileReferenceRecord {
            profile_id: EXCEL_GRID_PROFILE_ID.to_string(),
            profile_version: ProfileVersion::v1(),
            source_info: ReferenceSourceInfo {
                source_channel: FormulaChannelKind::WorksheetA1,
                source_span: oxfml_core::syntax::token::TextSpan::new(1, source_text.len()),
                source_text: source_text.clone(),
                parsed_qualifier: transformed_parsed_qualifier(&reference),
                address_fidelity: Some(source_text.clone()),
            },
            profile_payload: ProfilePayload {
                payload_kind: "excel-grid-reference".to_string(),
                encoding: "json".to_string(),
                data: serde_json::to_string(&reference).unwrap(),
            },
            normal_form_key: normal_form_key_for_reference(EXCEL_GRID_PROFILE_ID, &reference),
            render_hint: Some(source_text),
            validity: ReferenceValidity::ValidAfterInstantiation,
        }
    }

    fn structural_payload(edit: ExcelGridStructuralEdit, row: u32, col: u32) -> ProfilePayload {
        ExcelGridReferenceTransformPayload::new(
            edit,
            Some(ExcelGridFormulaAnchor::new(
                "book:default",
                "sheet:default",
                row,
                col,
            )),
        )
        .into_profile_payload()
    }
}
