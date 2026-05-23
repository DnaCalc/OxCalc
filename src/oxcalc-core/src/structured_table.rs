#![forbid(unsafe_code)]

//! OxCalc-owned structured table dependency lowering.
//!
//! This module consumes public OxFml table-context packets. It does not parse
//! structured-reference formula text and does not mirror OxFml grammar.

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

use oxfml_core::binding::{AreaRef, CellRef, StructuredResolvedRef};
use oxfml_core::consumer::runtime::{
    RuntimeEnvironment, RuntimeFormulaRequest, RuntimeHostFormulaContext,
    RuntimeSparseReferenceCell, RuntimeSparseReferenceValuesBinding,
};
pub use oxfml_core::interface::{
    TableCallerRegion, TableColumnDescriptor, TableDescriptor, TableRef, TableRegionKind,
};
use oxfml_core::{
    EvaluationBackend, StructuredReferenceBindDiagnosticLink, StructuredReferenceBindRecord,
    StructuredReferenceSelectedRegion, StructuredReferenceSourceTokenKind, StructuredSectionKind,
    seam::Locus,
    source::{FormulaSourceRecord, StructureContextVersion},
    syntax::token::TextSpan,
};
use oxfunc_core::registry::builtin_registry;
use oxfunc_core::value::{
    ArrayCellValue, EvalValue, ExcelText, ReferenceKind, ReferenceLike, WorksheetErrorCode,
};

use crate::dependency::{
    DependencyDescriptor, DependencyDescriptorKind, InvalidationReasonKind, InvalidationSeed,
};
use crate::sparse_reader::{
    SparseCellCoord, SparseCellRead, SparseDefinedCell, SparseRangeExtent, SparseRangeReader,
    SparseReaderAccessSummary, SparseReaderIdentity,
};
use crate::structural::TreeNodeId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableVirtualAnchor {
    pub workbook_scope_ref: String,
    pub sheet_scope_ref: String,
    pub start_row: u32,
    pub start_col: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TreeCalcTableRowId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableFormulaMetadata {
    pub formula_artifact_id: String,
    pub bind_artifact_id: Option<String>,
    pub formula_text_version: String,
}

impl TreeCalcTableFormulaMetadata {
    #[must_use]
    pub fn identity_fragment(&self) -> String {
        identity_record(
            "treecalc.table_formula_metadata.v1",
            [
                ("formula_artifact_id", self.formula_artifact_id.clone()),
                (
                    "bind_artifact_id",
                    self.bind_artifact_id
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
                ),
                ("formula_text_version", self.formula_text_version.clone()),
            ],
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeCalcTableColumnBodyMetadata {
    ConstantCells,
    Formula(TreeCalcTableFormulaMetadata),
}

impl TreeCalcTableColumnBodyMetadata {
    #[must_use]
    pub fn identity_fragment(&self) -> String {
        match self {
            Self::ConstantCells => "treecalc.table_body.constant_cells.v1".to_string(),
            Self::Formula(metadata) => identity_record(
                "treecalc.table_body.formula.v1",
                [("metadata", metadata.identity_fragment())],
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableColumnSnapshot {
    pub column_id: String,
    pub column_name: String,
    pub ordinal: u32,
    pub body_metadata: TreeCalcTableColumnBodyMetadata,
    pub totals_metadata: Option<TreeCalcTableFormulaMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableNodeSnapshot {
    pub table_node_id: TreeNodeId,
    pub table_id: String,
    pub table_name: String,
    pub display_path: String,
    pub canonical_path: String,
    pub virtual_anchor: TreeCalcTableVirtualAnchor,
    pub rows: Vec<TreeCalcTableRowId>,
    pub columns: Vec<TreeCalcTableColumnSnapshot>,
    pub header_row_present: bool,
    pub totals_row_present: bool,
    pub table_namespace_version: String,
    pub row_membership_version: String,
    pub row_order_version: String,
    pub column_identity_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableNodeProjection {
    pub table_node_id: TreeNodeId,
    pub table_id: String,
    pub display_path: String,
    pub canonical_path: String,
    pub table_descriptor: TableDescriptor,
    pub context_packet: StructuredTableContextPacket,
    pub table_context_identity: String,
    pub table_invalidation_identity: String,
    pub table_namespace_identity: String,
    pub table_namespace_token: String,
    pub row_membership_identity: String,
    pub row_order_identity: String,
    pub oxcalc_row_membership_identity: String,
    pub oxcalc_row_order_identity: String,
    pub column_identity: String,
    pub oxcalc_column_identity: String,
    pub virtual_anchor_identity: String,
    pub virtual_anchor_token: String,
    pub body_metadata_identity: String,
    pub body_metadata_token: String,
    pub totals_metadata_identity: String,
    pub totals_metadata_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeCalcTableProjectionError {
    EmptyTableId,
    EmptyTableName,
    EmptyDisplayPath,
    EmptyCanonicalPath,
    EmptyColumns,
    EmptyTableHasNoRepresentableRows,
    InvalidVirtualAnchor { start_row: u32, start_col: u32 },
    DuplicateColumnId { column_id: String },
    DuplicateColumnName { column_name: String },
    DuplicateColumnOrdinal { ordinal: u32 },
    DuplicateRowId { row_id: String },
    ColumnOrdinalMustStartAtOne { column_id: String, ordinal: u32 },
    RangeOverflow,
}

pub fn project_treecalc_table_node_snapshot(
    snapshot: &TreeCalcTableNodeSnapshot,
) -> Result<TreeCalcTableNodeProjection, TreeCalcTableProjectionError> {
    validate_treecalc_table_node_snapshot(snapshot)?;

    let sorted_columns = sorted_treecalc_table_columns(snapshot)?;
    let row_count = u32::try_from(snapshot.rows.len())
        .map_err(|_| TreeCalcTableProjectionError::RangeOverflow)?;
    let column_count = u32::try_from(sorted_columns.len())
        .map_err(|_| TreeCalcTableProjectionError::RangeOverflow)?;
    let header_rows = u32::from(snapshot.header_row_present);
    let totals_rows = u32::from(snapshot.totals_row_present);
    let total_rows = header_rows
        .checked_add(row_count)
        .and_then(|value| value.checked_add(totals_rows))
        .ok_or(TreeCalcTableProjectionError::RangeOverflow)?;
    if total_rows == 0 {
        return Err(TreeCalcTableProjectionError::EmptyTableHasNoRepresentableRows);
    }
    let table_end_row = snapshot
        .virtual_anchor
        .start_row
        .checked_add(total_rows.saturating_sub(1))
        .ok_or(TreeCalcTableProjectionError::RangeOverflow)?;
    let table_end_col = snapshot
        .virtual_anchor
        .start_col
        .checked_add(column_count - 1)
        .ok_or(TreeCalcTableProjectionError::RangeOverflow)?;
    let data_start_row = snapshot
        .virtual_anchor
        .start_row
        .checked_add(header_rows)
        .ok_or(TreeCalcTableProjectionError::RangeOverflow)?;
    let data_end_row = (row_count > 0)
        .then(|| {
            data_start_row
                .checked_add(row_count - 1)
                .ok_or(TreeCalcTableProjectionError::RangeOverflow)
        })
        .transpose()?;

    let oxcalc_row_membership_identity = treecalc_table_row_membership_fact_identity(snapshot);
    let oxcalc_row_order_identity = treecalc_table_row_order_fact_identity(snapshot);
    let oxcalc_column_identity = treecalc_table_column_fact_identity(snapshot, &sorted_columns);
    let row_membership_identity = treecalc_table_row_membership_descriptor_identity(snapshot);
    let row_order_identity = treecalc_table_row_order_descriptor_identity(snapshot);
    let column_identity = treecalc_table_column_descriptor_identity(snapshot, &sorted_columns);
    let virtual_anchor_identity = treecalc_table_virtual_anchor_identity(snapshot);
    let table_namespace_identity = treecalc_table_namespace_identity(snapshot);
    let body_metadata_identity = treecalc_table_body_metadata_identity(snapshot, &sorted_columns);
    let totals_metadata_identity =
        treecalc_table_totals_metadata_identity(snapshot, &sorted_columns);
    let table_namespace_token = opaque_identity_token(
        "treecalc.table_namespace.token.v1",
        &table_namespace_identity,
    );
    let virtual_anchor_token =
        opaque_identity_token("treecalc.table_anchor.token.v1", &virtual_anchor_identity);
    let body_metadata_token =
        opaque_identity_token("treecalc.table_body.token.v1", &body_metadata_identity);
    let totals_metadata_token =
        opaque_identity_token("treecalc.table_totals.token.v1", &totals_metadata_identity);

    let columns = sorted_columns
        .iter()
        .enumerate()
        .map(|(index, column)| {
            let offset =
                u32::try_from(index).map_err(|_| TreeCalcTableProjectionError::RangeOverflow)?;
            let col = snapshot
                .virtual_anchor
                .start_col
                .checked_add(offset)
                .ok_or(TreeCalcTableProjectionError::RangeOverflow)?;
            Ok(TableColumnDescriptor {
                column_id: column.column_id.clone(),
                column_name: column.column_name.clone(),
                ordinal: column.ordinal,
                column_range_ref: data_end_row
                    .map(|end_row| a1_range_ref(data_start_row, col, end_row, col))
                    .transpose()?
                    .unwrap_or_default(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let table_descriptor = TableDescriptor {
        table_id: snapshot.table_id.clone(),
        table_name: snapshot.table_name.clone(),
        workbook_scope_ref: snapshot.virtual_anchor.workbook_scope_ref.clone(),
        sheet_scope_ref: snapshot.virtual_anchor.sheet_scope_ref.clone(),
        table_range_ref: a1_range_ref(
            snapshot.virtual_anchor.start_row,
            snapshot.virtual_anchor.start_col,
            table_end_row,
            table_end_col,
        )?,
        row_membership_identity: Some(row_membership_identity.clone()),
        row_order_identity: Some(row_order_identity.clone()),
        header_region_ref: snapshot
            .header_row_present
            .then(|| {
                a1_range_ref(
                    snapshot.virtual_anchor.start_row,
                    snapshot.virtual_anchor.start_col,
                    snapshot.virtual_anchor.start_row,
                    table_end_col,
                )
            })
            .transpose()?,
        totals_region_ref: snapshot
            .totals_row_present
            .then(|| {
                a1_range_ref(
                    table_end_row,
                    snapshot.virtual_anchor.start_col,
                    table_end_row,
                    table_end_col,
                )
            })
            .transpose()?,
        header_row_present: snapshot.header_row_present,
        totals_row_present: snapshot.totals_row_present,
        columns,
    };
    let context_packet = StructuredTableContextPacket::from_oxfml_table_packet(
        vec![table_descriptor.clone()],
        Some(TableRef {
            table_id: snapshot.table_id.clone(),
        }),
        None,
    );
    let table_invalidation_identity = identity_record(
        "treecalc.table_invalidation.v1",
        [
            ("table_node_id", snapshot.table_node_id.to_string()),
            ("table_id", snapshot.table_id.clone()),
            ("namespace", table_namespace_identity.clone()),
            ("anchor", virtual_anchor_identity.clone()),
            ("row_membership", oxcalc_row_membership_identity.clone()),
            ("row_order", oxcalc_row_order_identity.clone()),
            ("columns", oxcalc_column_identity.clone()),
            ("body", body_metadata_identity.clone()),
            ("totals", totals_metadata_identity.clone()),
        ],
    );
    let table_context_identity = identity_record(
        "treecalc.table_context.v1",
        [
            ("table_id", snapshot.table_id.clone()),
            ("namespace_token", table_namespace_token.clone()),
            ("anchor_token", virtual_anchor_token.clone()),
            ("row_membership", row_membership_identity.clone()),
            ("row_order", row_order_identity.clone()),
            ("columns", column_identity.clone()),
            ("body_token", body_metadata_token.clone()),
            ("totals_token", totals_metadata_token.clone()),
            (
                "generic_packet",
                context_packet.table_context_identity.clone(),
            ),
        ],
    );

    Ok(TreeCalcTableNodeProjection {
        table_node_id: snapshot.table_node_id,
        table_id: snapshot.table_id.clone(),
        display_path: snapshot.display_path.clone(),
        canonical_path: snapshot.canonical_path.clone(),
        table_descriptor,
        context_packet,
        table_context_identity,
        table_invalidation_identity,
        table_namespace_identity,
        table_namespace_token,
        row_membership_identity,
        row_order_identity,
        oxcalc_row_membership_identity,
        oxcalc_row_order_identity,
        column_identity,
        oxcalc_column_identity,
        virtual_anchor_identity,
        virtual_anchor_token,
        body_metadata_identity,
        body_metadata_token,
        totals_metadata_identity,
        totals_metadata_token,
    })
}

fn validate_treecalc_table_node_snapshot(
    snapshot: &TreeCalcTableNodeSnapshot,
) -> Result<(), TreeCalcTableProjectionError> {
    if snapshot.table_id.is_empty() {
        return Err(TreeCalcTableProjectionError::EmptyTableId);
    }
    if snapshot.table_name.is_empty() {
        return Err(TreeCalcTableProjectionError::EmptyTableName);
    }
    if snapshot.display_path.is_empty() {
        return Err(TreeCalcTableProjectionError::EmptyDisplayPath);
    }
    if snapshot.canonical_path.is_empty() {
        return Err(TreeCalcTableProjectionError::EmptyCanonicalPath);
    }
    if snapshot.columns.is_empty() {
        return Err(TreeCalcTableProjectionError::EmptyColumns);
    }
    if snapshot.virtual_anchor.start_row == 0 || snapshot.virtual_anchor.start_col == 0 {
        return Err(TreeCalcTableProjectionError::InvalidVirtualAnchor {
            start_row: snapshot.virtual_anchor.start_row,
            start_col: snapshot.virtual_anchor.start_col,
        });
    }

    let mut row_ids = BTreeSet::new();
    for row in &snapshot.rows {
        if !row_ids.insert(row.0.clone()) {
            return Err(TreeCalcTableProjectionError::DuplicateRowId {
                row_id: row.0.clone(),
            });
        }
    }

    let mut column_ids = BTreeSet::new();
    let mut column_names = BTreeSet::new();
    let mut column_ordinals = BTreeSet::new();
    for column in &snapshot.columns {
        if !column_ids.insert(column.column_id.clone()) {
            return Err(TreeCalcTableProjectionError::DuplicateColumnId {
                column_id: column.column_id.clone(),
            });
        }
        let normalized_name = column.column_name.to_ascii_uppercase();
        if !column_names.insert(normalized_name) {
            return Err(TreeCalcTableProjectionError::DuplicateColumnName {
                column_name: column.column_name.clone(),
            });
        }
        if !column_ordinals.insert(column.ordinal) {
            return Err(TreeCalcTableProjectionError::DuplicateColumnOrdinal {
                ordinal: column.ordinal,
            });
        }
        if column.ordinal == 0 {
            return Err(TreeCalcTableProjectionError::ColumnOrdinalMustStartAtOne {
                column_id: column.column_id.clone(),
                ordinal: column.ordinal,
            });
        }
    }

    Ok(())
}

fn sorted_treecalc_table_columns(
    snapshot: &TreeCalcTableNodeSnapshot,
) -> Result<Vec<TreeCalcTableColumnSnapshot>, TreeCalcTableProjectionError> {
    let mut columns = snapshot.columns.clone();
    columns.sort_by(|left, right| {
        left.ordinal
            .cmp(&right.ordinal)
            .then_with(|| left.column_id.cmp(&right.column_id))
    });
    for (index, column) in columns.iter().enumerate() {
        let expected =
            u32::try_from(index + 1).map_err(|_| TreeCalcTableProjectionError::RangeOverflow)?;
        if column.ordinal != expected {
            return Err(TreeCalcTableProjectionError::ColumnOrdinalMustStartAtOne {
                column_id: column.column_id.clone(),
                ordinal: column.ordinal,
            });
        }
    }
    Ok(columns)
}

fn treecalc_table_namespace_identity(snapshot: &TreeCalcTableNodeSnapshot) -> String {
    identity_record(
        "treecalc.table_namespace.v1",
        [
            ("version", snapshot.table_namespace_version.clone()),
            ("display_path", snapshot.display_path.clone()),
            ("canonical_path", snapshot.canonical_path.clone()),
            ("name", snapshot.table_name.clone()),
        ],
    )
}

fn treecalc_table_virtual_anchor_identity(snapshot: &TreeCalcTableNodeSnapshot) -> String {
    identity_record(
        "treecalc.table_anchor.v1",
        [
            (
                "workbook",
                snapshot.virtual_anchor.workbook_scope_ref.clone(),
            ),
            ("sheet", snapshot.virtual_anchor.sheet_scope_ref.clone()),
            ("row", snapshot.virtual_anchor.start_row.to_string()),
            ("col", snapshot.virtual_anchor.start_col.to_string()),
        ],
    )
}

fn treecalc_table_row_membership_fact_identity(snapshot: &TreeCalcTableNodeSnapshot) -> String {
    let mut rows = snapshot
        .rows
        .iter()
        .map(|row| row.0.clone())
        .collect::<Vec<_>>();
    rows.sort();
    identity_record(
        "treecalc.table_rows.membership.v1",
        [
            ("table", snapshot.table_id.clone()),
            ("version", snapshot.row_membership_version.clone()),
            ("rows", identity_list(rows)),
        ],
    )
}

fn treecalc_table_row_order_fact_identity(snapshot: &TreeCalcTableNodeSnapshot) -> String {
    identity_record(
        "treecalc.table_rows.order.v1",
        [
            ("table", snapshot.table_id.clone()),
            ("version", snapshot.row_order_version.clone()),
            (
                "rows",
                identity_list(snapshot.rows.iter().map(|row| row.0.clone())),
            ),
        ],
    )
}

fn treecalc_table_row_membership_descriptor_identity(
    snapshot: &TreeCalcTableNodeSnapshot,
) -> String {
    identity_record(
        "treecalc.table_rows.membership_token.v1",
        [
            ("table", snapshot.table_id.clone()),
            ("version", snapshot.row_membership_version.clone()),
            ("row_count", snapshot.rows.len().to_string()),
        ],
    )
}

fn treecalc_table_row_order_descriptor_identity(snapshot: &TreeCalcTableNodeSnapshot) -> String {
    identity_record(
        "treecalc.table_rows.order_token.v1",
        [
            ("table", snapshot.table_id.clone()),
            ("version", snapshot.row_order_version.clone()),
            ("row_count", snapshot.rows.len().to_string()),
        ],
    )
}

fn treecalc_table_column_fact_identity(
    snapshot: &TreeCalcTableNodeSnapshot,
    sorted_columns: &[TreeCalcTableColumnSnapshot],
) -> String {
    identity_record(
        "treecalc.table_columns.identity.v1",
        [
            ("table", snapshot.table_id.clone()),
            ("version", snapshot.column_identity_version.clone()),
            (
                "columns",
                identity_list(sorted_columns.iter().map(|column| {
                    identity_record(
                        "column",
                        [
                            ("ordinal", column.ordinal.to_string()),
                            ("id", column.column_id.clone()),
                            ("name", column.column_name.clone()),
                        ],
                    )
                })),
            ),
        ],
    )
}

fn treecalc_table_column_descriptor_identity(
    snapshot: &TreeCalcTableNodeSnapshot,
    sorted_columns: &[TreeCalcTableColumnSnapshot],
) -> String {
    identity_record(
        "treecalc.table_columns.identity_token.v1",
        [
            ("table", snapshot.table_id.clone()),
            ("version", snapshot.column_identity_version.clone()),
            ("column_count", sorted_columns.len().to_string()),
        ],
    )
}

fn treecalc_table_body_metadata_identity(
    snapshot: &TreeCalcTableNodeSnapshot,
    sorted_columns: &[TreeCalcTableColumnSnapshot],
) -> String {
    identity_record(
        "treecalc.table_body.v1",
        [
            ("table", snapshot.table_id.clone()),
            (
                "columns",
                identity_list(sorted_columns.iter().map(|column| {
                    identity_record(
                        "column_body",
                        [
                            ("column_id", column.column_id.clone()),
                            ("body", column.body_metadata.identity_fragment()),
                        ],
                    )
                })),
            ),
        ],
    )
}

fn treecalc_table_totals_metadata_identity(
    snapshot: &TreeCalcTableNodeSnapshot,
    sorted_columns: &[TreeCalcTableColumnSnapshot],
) -> String {
    identity_record(
        "treecalc.table_totals.v1",
        [
            ("table", snapshot.table_id.clone()),
            ("present", snapshot.totals_row_present.to_string()),
            (
                "columns",
                identity_list(sorted_columns.iter().map(|column| {
                    identity_record(
                        "column_totals",
                        [
                            ("column_id", column.column_id.clone()),
                            (
                                "totals",
                                column.totals_metadata.as_ref().map_or(
                                    "none".to_string(),
                                    TreeCalcTableFormulaMetadata::identity_fragment,
                                ),
                            ),
                        ],
                    )
                })),
            ),
        ],
    )
}

fn identity_record<const N: usize>(kind: &str, fields: [(&str, String); N]) -> String {
    let mut result = String::from(kind);
    for (name, value) in fields {
        result.push(';');
        result.push_str(name);
        result.push('=');
        result.push_str(&identity_atom(&value));
    }
    result
}

fn identity_list(values: impl IntoIterator<Item = String>) -> String {
    let values = values.into_iter().collect::<Vec<_>>();
    let mut result = format!("count={};", values.len());
    for value in values {
        result.push_str(&identity_atom(&value));
    }
    result
}

fn identity_atom(value: &str) -> String {
    format!("{}:{value}", value.len())
}

fn opaque_identity_token(kind: &str, identity: &str) -> String {
    format!("{kind};fnv1a64={:016x}", fnv1a64(identity.as_bytes()))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn a1_range_ref(
    top_row: u32,
    left_col: u32,
    bottom_row: u32,
    right_col: u32,
) -> Result<String, TreeCalcTableProjectionError> {
    if top_row == 0 || left_col == 0 || bottom_row == 0 || right_col == 0 {
        return Err(TreeCalcTableProjectionError::RangeOverflow);
    }
    Ok(format!(
        "{}{}:{}{}",
        excel_column_name(left_col)?,
        top_row,
        excel_column_name(right_col)?,
        bottom_row
    ))
}

fn excel_column_name(mut col: u32) -> Result<String, TreeCalcTableProjectionError> {
    if col == 0 {
        return Err(TreeCalcTableProjectionError::RangeOverflow);
    }
    let mut chars = Vec::new();
    while col > 0 {
        let remainder = (col - 1) % 26;
        let ch = char::from(b'A' + u8::try_from(remainder).expect("remainder is less than 26"));
        chars.push(ch);
        col = (col - 1) / 26;
    }
    chars.reverse();
    Ok(chars.into_iter().collect())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredTableContextPacket {
    pub table_catalog: Vec<TableDescriptor>,
    pub enclosing_table_ref: Option<TableRef>,
    pub caller_table_region: Option<TableCallerRegion>,
    pub table_context_identity: String,
}

impl StructuredTableContextPacket {
    #[must_use]
    pub fn from_oxfml_table_packet(
        table_catalog: Vec<TableDescriptor>,
        enclosing_table_ref: Option<TableRef>,
        caller_table_region: Option<TableCallerRegion>,
    ) -> Self {
        let table_context_identity =
            table_context_identity(&table_catalog, &enclosing_table_ref, &caller_table_region);
        Self {
            table_catalog,
            enclosing_table_ref,
            caller_table_region,
            table_context_identity,
        }
    }

    fn table_by_id(&self) -> BTreeMap<&str, &TableDescriptor> {
        self.table_catalog
            .iter()
            .map(|table| (table.table_id.as_str(), table))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableStructuredReferencePrebind {
    pub source_span_utf8: TextSpan,
    pub source_token_text: String,
    pub source_token_kind: StructuredReferenceSourceTokenKind,
    pub path_span_utf8: Option<TextSpan>,
    pub path_token_text: Option<String>,
    pub structured_tail_span_utf8: TextSpan,
    pub structured_tail_token_text: String,
    pub host_ref_handle: String,
    pub resolved_table_node_id: Option<TreeNodeId>,
    pub resolved_table_id: Option<String>,
    pub selector_payload: TreeCalcTableStructuredSelectorPayload,
    pub caller_context_dependency: bool,
    pub replay_identity: String,
    pub bind_record: StructuredReferenceBindRecord,
    pub diagnostics: Vec<TreeCalcTableStructuredReferenceDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableStructuredSelectorPayload {
    pub table_path_token: Option<String>,
    pub selected_column_ids: Vec<String>,
    pub selected_sections: Vec<StructuredSectionKind>,
    pub uses_this_row: bool,
    pub omitted_table_name: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableStructuredReferenceDiagnostic {
    pub diagnostic_code: String,
    pub message: String,
    pub source_span_utf8: TextSpan,
}

pub fn prebind_treecalc_table_structured_references(
    source_text: &str,
    table_projections: &[TreeCalcTableNodeProjection],
    enclosing_table_ref: Option<TableRef>,
    caller_table_region: Option<TableCallerRegion>,
) -> Vec<TreeCalcTableStructuredReferencePrebind> {
    let table_lookup = treecalc_table_path_lookup(table_projections);
    let mut results = Vec::new();
    let mut cursor = 0;

    while let Some(relative_open) = source_text[cursor..].find('[') {
        let open = cursor + relative_open;
        if is_nested_structured_tail_open(source_text, open) {
            cursor = open + 1;
            continue;
        }
        let Some(close) = matching_structured_tail_close(source_text, open) else {
            break;
        };
        let path_start = treecalc_table_path_token_start(source_text, open);
        let path_token = source_text[path_start..open].trim();
        let source_start = if path_token.is_empty() {
            open
        } else {
            path_start
        };
        let source_span = TextSpan::new(source_start, close + 1 - source_start);
        let tail_span = TextSpan::new(open, close + 1 - open);
        let source_token_text = source_text[source_span.start..source_span.end()].to_string();
        let tail_token_text = source_text[tail_span.start..tail_span.end()].to_string();

        if path_token.is_empty() && !tail_supports_omitted_table_name(&tail_token_text) {
            cursor = close + 1;
            continue;
        }

        let projection = (!path_token.is_empty())
            .then(|| table_lookup.get(path_token))
            .flatten()
            .copied();
        let mut diagnostics = Vec::new();
        let (table_id, table_name, table_node_id, descriptor) = if let Some(projection) = projection
        {
            (
                Some(projection.table_id.clone()),
                Some(projection.table_descriptor.table_name.clone()),
                Some(projection.table_node_id),
                Some(&projection.table_descriptor),
            )
        } else if path_token.is_empty() {
            let descriptor = enclosing_table_ref
                .as_ref()
                .and_then(|table_ref| {
                    table_projections
                        .iter()
                        .find(|projection| projection.table_id == table_ref.table_id)
                })
                .map(|projection| &projection.table_descriptor);
            if descriptor.is_none() {
                diagnostics.push(treecalc_table_structured_diagnostic(
                    "treecalc.table.omitted_without_caller_context",
                    "omitted table structured reference has no enclosing table context",
                    source_span,
                ));
            }
            (
                enclosing_table_ref
                    .as_ref()
                    .map(|table_ref| table_ref.table_id.clone()),
                descriptor.map(|descriptor| descriptor.table_name.clone()),
                table_projections
                    .iter()
                    .find(|projection| {
                        enclosing_table_ref
                            .as_ref()
                            .is_some_and(|table_ref| projection.table_id == table_ref.table_id)
                    })
                    .map(|projection| projection.table_node_id),
                descriptor,
            )
        } else {
            diagnostics.push(treecalc_table_structured_diagnostic(
                "treecalc.table.path_not_table",
                format!("TreeCalc path token '{path_token}' does not resolve to a table node"),
                TextSpan::new(path_start, open - path_start),
            ));
            (None, None, None, None)
        };

        let parsed_selector =
            parse_treecalc_structured_tail(&tail_token_text, descriptor, tail_span);
        diagnostics.extend(parsed_selector.diagnostics);

        let caller_context_dependency =
            parsed_selector.uses_this_row || path_token.is_empty() || caller_table_region.is_some();
        let host_ref_handle = treecalc_table_structured_host_ref_handle(source_span);
        let replay_identity = treecalc_table_structured_replay_identity(
            &host_ref_handle,
            &source_token_text,
            table_id.as_deref(),
            &parsed_selector.selected_column_ids,
            &parsed_selector.selected_sections,
            parsed_selector.uses_this_row,
        );
        let selector_payload = TreeCalcTableStructuredSelectorPayload {
            table_path_token: (!path_token.is_empty()).then(|| path_token.to_string()),
            selected_column_ids: parsed_selector.selected_column_ids.clone(),
            selected_sections: parsed_selector.selected_sections.clone(),
            uses_this_row: parsed_selector.uses_this_row,
            omitted_table_name: path_token.is_empty(),
        };

        let bind_diagnostics = diagnostics
            .iter()
            .map(|diagnostic| StructuredReferenceBindDiagnosticLink {
                diagnostic_code: diagnostic.diagnostic_code.clone(),
                message: diagnostic.message.clone(),
                source_span_utf8: diagnostic.source_span_utf8,
            })
            .collect::<Vec<_>>();
        let bind_record = StructuredReferenceBindRecord {
            bind_record_handle: host_ref_handle.clone(),
            source_span_utf8: source_span,
            source_token_text: source_token_text.clone(),
            source_token_kind: StructuredReferenceSourceTokenKind::StructuredReference,
            explicit_table_name: (!path_token.is_empty()).then(|| path_token.to_string()),
            omitted_table_name: path_token.is_empty(),
            effective_table_id: table_id.clone(),
            effective_table_name: table_name,
            selected_column_ids: parsed_selector.selected_column_ids.clone(),
            selected_sections: parsed_selector.selected_sections.clone(),
            selected_regions: parsed_selector.selected_regions,
            uses_this_row: parsed_selector.uses_this_row,
            caller_context_dependent: caller_context_dependency,
            resolved_reference: None,
            diagnostics: bind_diagnostics,
        };

        results.push(TreeCalcTableStructuredReferencePrebind {
            source_span_utf8: source_span,
            source_token_text,
            source_token_kind: StructuredReferenceSourceTokenKind::StructuredReference,
            path_span_utf8: (!path_token.is_empty())
                .then(|| TextSpan::new(path_start, open - path_start)),
            path_token_text: (!path_token.is_empty()).then(|| path_token.to_string()),
            structured_tail_span_utf8: tail_span,
            structured_tail_token_text: tail_token_text,
            host_ref_handle,
            resolved_table_node_id: table_node_id,
            resolved_table_id: table_id,
            selector_payload,
            caller_context_dependency,
            replay_identity,
            bind_record,
            diagnostics,
        });

        cursor = close + 1;
    }

    results
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedTreeCalcStructuredTail {
    selected_column_ids: Vec<String>,
    selected_sections: Vec<StructuredSectionKind>,
    selected_regions: Vec<StructuredReferenceSelectedRegion>,
    uses_this_row: bool,
    diagnostics: Vec<TreeCalcTableStructuredReferenceDiagnostic>,
}

fn treecalc_table_path_lookup(
    table_projections: &[TreeCalcTableNodeProjection],
) -> BTreeMap<String, &TreeCalcTableNodeProjection> {
    let mut lookup = BTreeMap::new();
    for projection in table_projections {
        for token in treecalc_table_path_tokens(projection) {
            lookup.entry(token).or_insert(projection);
        }
    }
    lookup
}

fn treecalc_table_path_tokens(projection: &TreeCalcTableNodeProjection) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    tokens.insert(projection.display_path.clone());
    tokens.insert(projection.canonical_path.clone());
    tokens.insert(projection.table_descriptor.table_name.clone());
    tokens.insert(bracket_escape_treecalc_path(
        &projection.table_descriptor.table_name,
    ));
    tokens.insert(bracket_escape_treecalc_path(&projection.canonical_path));
    tokens.insert(bracket_escape_treecalc_path(&projection.display_path));
    tokens
}

fn bracket_escape_treecalc_path(path: &str) -> String {
    path.split('.')
        .map(|segment| {
            if segment
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
            {
                segment.to_string()
            } else {
                format!("[{}]", segment.replace(']', "]]"))
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}

fn is_nested_structured_tail_open(source_text: &str, open: usize) -> bool {
    open > 0
        && source_text
            .as_bytes()
            .get(open - 1)
            .is_some_and(|byte| matches!(byte, b'[' | b','))
}

fn matching_structured_tail_close(source_text: &str, open: usize) -> Option<usize> {
    let bytes = source_text.as_bytes();
    let mut depth = 0usize;
    let mut index = open;
    while index < bytes.len() {
        match bytes[index] {
            b'[' => {
                depth += 1;
                index += 1;
            }
            b']' => {
                let run_start = index;
                while index < bytes.len() && bytes[index] == b']' {
                    index += 1;
                }
                let run_len = index - run_start;
                let after_run = bytes.get(index).copied();
                let structural_close = run_len == 1
                    || after_run.is_none()
                    || after_run.is_some_and(|byte| !byte.is_ascii_alphanumeric() && byte != b'_');
                if !structural_close {
                    continue;
                }
                let closers = run_len.min(depth);
                depth = depth.saturating_sub(closers);
                if depth == 0 {
                    return Some(run_start + run_len - 1);
                }
            }
            _ => index += 1,
        }
    }
    None
}

fn treecalc_table_path_token_start(source_text: &str, open: usize) -> usize {
    let bytes = source_text.as_bytes();
    let mut start = open;
    while start > 0 {
        let byte = bytes[start - 1];
        if matches!(
            byte,
            b'=' | b'+'
                | b'-'
                | b'*'
                | b'/'
                | b'^'
                | b'&'
                | b'('
                | b')'
                | b','
                | b'{'
                | b'}'
                | b' '
                | b'\t'
                | b'\r'
                | b'\n'
        ) {
            break;
        }
        start -= 1;
    }
    start
}

fn tail_supports_omitted_table_name(tail_token_text: &str) -> bool {
    let content = tail_token_text
        .strip_prefix('[')
        .and_then(|text| text.strip_suffix(']'))
        .unwrap_or(tail_token_text)
        .trim();
    content.starts_with('@') || content.starts_with("[@")
}

fn parse_treecalc_structured_tail(
    tail_token_text: &str,
    table: Option<&TableDescriptor>,
    tail_span: TextSpan,
) -> ParsedTreeCalcStructuredTail {
    let content = tail_token_text
        .strip_prefix('[')
        .and_then(|text| text.strip_suffix(']'))
        .unwrap_or(tail_token_text);
    let mut selected_column_ids = Vec::new();
    let mut selected_sections = Vec::new();
    let mut uses_this_row = false;
    let mut diagnostics = Vec::new();

    for part in split_structured_tail_parts(content) {
        let part = unwrap_structured_tail_part(part.trim());
        if part.is_empty() {
            continue;
        }
        if let Some(section) = parse_structured_section(part) {
            selected_sections.push(section);
            if section == StructuredSectionKind::ThisRow {
                uses_this_row = true;
            }
            continue;
        }
        if let Some(column_name) = part.strip_prefix('@') {
            uses_this_row = true;
            push_unique_section(&mut selected_sections, StructuredSectionKind::ThisRow);
            resolve_treecalc_structured_column_or_range(
                column_name,
                table,
                tail_span,
                &mut selected_column_ids,
                &mut diagnostics,
            );
            continue;
        }
        resolve_treecalc_structured_column_or_range(
            part,
            table,
            tail_span,
            &mut selected_column_ids,
            &mut diagnostics,
        );
    }

    if selected_sections.is_empty() {
        selected_sections.push(if uses_this_row {
            StructuredSectionKind::ThisRow
        } else {
            StructuredSectionKind::Data
        });
    }
    selected_sections.sort_by_key(|section| structured_section_sort_key(*section));
    selected_sections.dedup();
    selected_column_ids.sort();
    selected_column_ids.dedup();
    let selected_regions = selected_sections
        .iter()
        .map(|section_kind| StructuredReferenceSelectedRegion {
            section_kind: *section_kind,
            region_ref: structured_region_ref(table, *section_kind),
            column_range_refs: selected_column_ids
                .iter()
                .filter_map(|column_id| {
                    table.and_then(|descriptor| {
                        descriptor
                            .columns
                            .iter()
                            .find(|column| &column.column_id == column_id)
                            .and_then(|column| {
                                (!column.column_range_ref.trim().is_empty())
                                    .then(|| column.column_range_ref.clone())
                            })
                    })
                })
                .collect(),
            is_empty: matches!(
                *section_kind,
                StructuredSectionKind::Data | StructuredSectionKind::ThisRow
            ) && table.is_some_and(|descriptor| {
                !selected_column_ids.is_empty()
                    && selected_column_ids.iter().all(|column_id| {
                        descriptor
                            .columns
                            .iter()
                            .find(|column| &column.column_id == column_id)
                            .is_some_and(|column| column.column_range_ref.trim().is_empty())
                    })
            }),
        })
        .collect();

    ParsedTreeCalcStructuredTail {
        selected_column_ids,
        selected_sections,
        selected_regions,
        uses_this_row,
        diagnostics,
    }
}

fn split_structured_tail_parts(content: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    let bytes = content.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'[' => {
                depth += 1;
                index += 1;
            }
            b']' => {
                let run_start = index;
                while index < bytes.len() && bytes[index] == b']' {
                    index += 1;
                }
                let run_len = index - run_start;
                let after_run = bytes.get(index).copied();
                let structural_close = run_len == 1
                    || after_run.is_none()
                    || after_run.is_some_and(|byte| !byte.is_ascii_alphanumeric() && byte != b'_');
                if structural_close {
                    depth = depth.saturating_sub(run_len.min(depth));
                }
            }
            b',' if depth == 0 => {
                parts.push(&content[start..index]);
                start = index + 1;
                index += 1;
            }
            _ => index += 1,
        }
    }
    parts.push(&content[start..]);
    parts
}

fn unwrap_structured_tail_part(part: &str) -> &str {
    if part.starts_with('[')
        && part.ends_with(']')
        && matching_structured_tail_close(part, 0) == Some(part.len() - 1)
    {
        &part[1..part.len() - 1]
    } else {
        part
    }
}

fn parse_structured_section(part: &str) -> Option<StructuredSectionKind> {
    match part.to_ascii_lowercase().as_str() {
        "#all" => Some(StructuredSectionKind::All),
        "#data" => Some(StructuredSectionKind::Data),
        "#headers" => Some(StructuredSectionKind::Headers),
        "#totals" => Some(StructuredSectionKind::Totals),
        "#this row" | "@" => Some(StructuredSectionKind::ThisRow),
        _ => None,
    }
}

fn push_unique_section(sections: &mut Vec<StructuredSectionKind>, section: StructuredSectionKind) {
    if !sections.contains(&section) {
        sections.push(section);
    }
}

fn resolve_treecalc_structured_column_or_range(
    column_name: &str,
    table: Option<&TableDescriptor>,
    tail_span: TextSpan,
    selected_column_ids: &mut Vec<String>,
    diagnostics: &mut Vec<TreeCalcTableStructuredReferenceDiagnostic>,
) {
    if let Some((start, end)) = split_structured_column_range(column_name) {
        resolve_treecalc_structured_column_range(
            unwrap_structured_tail_part(start.trim()),
            unwrap_structured_tail_part(end.trim()),
            table,
            tail_span,
            selected_column_ids,
            diagnostics,
        );
        return;
    }
    resolve_treecalc_structured_column(
        column_name,
        table,
        tail_span,
        selected_column_ids,
        diagnostics,
    );
}

fn split_structured_column_range(part: &str) -> Option<(&str, &str)> {
    let bytes = part.as_bytes();
    let mut depth = 0usize;
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'[' => {
                depth += 1;
                index += 1;
            }
            b']' => {
                let run_start = index;
                while index < bytes.len() && bytes[index] == b']' {
                    index += 1;
                }
                let run_len = index - run_start;
                let after_run = bytes.get(index).copied();
                let structural_close = run_len == 1
                    || after_run.is_none()
                    || after_run.is_some_and(|byte| !byte.is_ascii_alphanumeric() && byte != b'_');
                if structural_close {
                    depth = depth.saturating_sub(run_len.min(depth));
                }
            }
            b':' if depth == 0 => return Some((&part[..index], &part[index + 1..])),
            _ => index += 1,
        }
    }
    None
}

fn resolve_treecalc_structured_column_range(
    start_column_name: &str,
    end_column_name: &str,
    table: Option<&TableDescriptor>,
    tail_span: TextSpan,
    selected_column_ids: &mut Vec<String>,
    diagnostics: &mut Vec<TreeCalcTableStructuredReferenceDiagnostic>,
) {
    let Some(table) = table else {
        return;
    };
    let Some(start_column_id) =
        resolve_treecalc_structured_column_id(start_column_name, table, tail_span, diagnostics)
    else {
        return;
    };
    let Some(end_column_id) =
        resolve_treecalc_structured_column_id(end_column_name, table, tail_span, diagnostics)
    else {
        return;
    };
    let Some(start_column) = table
        .columns
        .iter()
        .find(|column| column.column_id == start_column_id)
    else {
        return;
    };
    let Some(end_column) = table
        .columns
        .iter()
        .find(|column| column.column_id == end_column_id)
    else {
        return;
    };
    let start = start_column.ordinal.min(end_column.ordinal);
    let end = start_column.ordinal.max(end_column.ordinal);
    selected_column_ids.extend(
        table
            .columns
            .iter()
            .filter(|column| column.ordinal >= start && column.ordinal <= end)
            .map(|column| column.column_id.clone()),
    );
}

fn resolve_treecalc_structured_column(
    column_name: &str,
    table: Option<&TableDescriptor>,
    tail_span: TextSpan,
    selected_column_ids: &mut Vec<String>,
    diagnostics: &mut Vec<TreeCalcTableStructuredReferenceDiagnostic>,
) {
    let Some(table) = table else {
        return;
    };
    if let Some(column_id) =
        resolve_treecalc_structured_column_id(column_name, table, tail_span, diagnostics)
    {
        selected_column_ids.push(column_id);
    }
}

fn resolve_treecalc_structured_column_id(
    column_name: &str,
    table: &TableDescriptor,
    tail_span: TextSpan,
    diagnostics: &mut Vec<TreeCalcTableStructuredReferenceDiagnostic>,
) -> Option<String> {
    let normalized = column_name.replace("]]", "]").trim().to_ascii_uppercase();
    if let Some(column) = table.columns.iter().find(|column| {
        column.column_id.eq_ignore_ascii_case(&normalized)
            || column.column_name.to_ascii_uppercase() == normalized
    }) {
        Some(column.column_id.clone())
    } else {
        diagnostics.push(treecalc_table_structured_diagnostic(
            "treecalc.table.unknown_column",
            format!(
                "structured reference column '{}' is not present in table '{}'",
                column_name, table.table_name
            ),
            tail_span,
        ));
        None
    }
}

fn structured_region_ref(
    table: Option<&TableDescriptor>,
    section_kind: StructuredSectionKind,
) -> Option<String> {
    let table = table?;
    match section_kind {
        StructuredSectionKind::All => Some(table.table_range_ref.clone()),
        StructuredSectionKind::Data | StructuredSectionKind::ThisRow => None,
        StructuredSectionKind::Headers => table.header_region_ref.clone(),
        StructuredSectionKind::Totals => table.totals_region_ref.clone(),
    }
}

fn structured_section_sort_key(section: StructuredSectionKind) -> u8 {
    match section {
        StructuredSectionKind::All => 0,
        StructuredSectionKind::Headers => 1,
        StructuredSectionKind::Data => 2,
        StructuredSectionKind::Totals => 3,
        StructuredSectionKind::ThisRow => 4,
    }
}

fn treecalc_table_structured_diagnostic(
    diagnostic_code: impl Into<String>,
    message: impl Into<String>,
    source_span_utf8: TextSpan,
) -> TreeCalcTableStructuredReferenceDiagnostic {
    TreeCalcTableStructuredReferenceDiagnostic {
        diagnostic_code: diagnostic_code.into(),
        message: message.into(),
        source_span_utf8,
    }
}

fn treecalc_table_structured_host_ref_handle(source_span: TextSpan) -> String {
    identity_record(
        "treecalc.structured_table_ref.handle.v1",
        [
            ("start", source_span.start.to_string()),
            ("len", source_span.len.to_string()),
        ],
    )
}

fn treecalc_table_structured_replay_identity(
    host_ref_handle: &str,
    source_token_text: &str,
    table_id: Option<&str>,
    selected_column_ids: &[String],
    selected_sections: &[StructuredSectionKind],
    uses_this_row: bool,
) -> String {
    identity_record(
        "treecalc.structured_table_ref.replay.v1",
        [
            ("handle", host_ref_handle.to_string()),
            ("source", source_token_text.to_string()),
            ("table_id", table_id.unwrap_or("none").to_string()),
            (
                "columns",
                identity_list(selected_column_ids.iter().cloned()),
            ),
            (
                "sections",
                identity_list(
                    selected_sections
                        .iter()
                        .map(|section| format!("{section:?}")),
                ),
            ),
            ("uses_this_row", uses_this_row.to_string()),
        ],
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeCalcTableSparseSection {
    Headers,
    Data,
    Totals,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TreeCalcTableSparseValue {
    pub section: TreeCalcTableSparseSection,
    pub row_id: Option<TreeCalcTableRowId>,
    pub column_id: String,
    pub value: EvalValue,
}

impl TreeCalcTableSparseValue {
    #[must_use]
    pub fn data(row_id: impl Into<String>, column_id: impl Into<String>, value: EvalValue) -> Self {
        Self {
            section: TreeCalcTableSparseSection::Data,
            row_id: Some(TreeCalcTableRowId(row_id.into())),
            column_id: column_id.into(),
            value,
        }
    }

    #[must_use]
    pub fn header(column_id: impl Into<String>, value: EvalValue) -> Self {
        Self {
            section: TreeCalcTableSparseSection::Headers,
            row_id: None,
            column_id: column_id.into(),
            value,
        }
    }

    #[must_use]
    pub fn totals(column_id: impl Into<String>, value: EvalValue) -> Self {
        Self {
            section: TreeCalcTableSparseSection::Totals,
            row_id: None,
            column_id: column_id.into(),
            value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeCalcTableSparseReaderError {
    BindRecordIntake {
        detail: String,
    },
    ProjectionSnapshotMismatch {
        projection_table_id: String,
        snapshot_table_id: String,
    },
    ReferencedTableMismatch {
        referenced_table_id: String,
        projection_table_id: String,
    },
    MissingSelectedColumn {
        column_id: String,
    },
    NonContiguousColumnSelection {
        column_ids: Vec<String>,
    },
    HeaderRowAbsent,
    TotalsRowAbsent,
    MissingHeaderRegion,
    MissingTotalsRegion,
    MissingCallerTableRegion,
    CallerTableMismatch {
        caller_table_id: String,
        referenced_table_id: String,
    },
    CallerRegionNotData {
        region_kind: TableRegionKind,
    },
    CallerDataRowOffsetMissing,
    CallerRowOutOfRange {
        row_offset: u32,
        row_count: usize,
    },
    MissingColumnRange {
        column_id: String,
        range_ref: String,
    },
    InvalidRegionRange {
        region: TreeCalcTableSparseSection,
        range_ref: String,
    },
    EmptySelection,
    RangeOverflow,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TreeCalcStructuredTableRuntimeBinding {
    pub reference: ReferenceLike,
    pub sparse_reference_values: RuntimeSparseReferenceValuesBinding,
    pub scalar_cell_values: BTreeMap<String, EvalValue>,
    pub reader_identity: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeCalcStructuredTableFunctionAdmissionStatus {
    CurrentReferencePreservingEvidence,
    AdmittedPendingOxFuncEvidence,
    RequiresTypedHostContext,
    TypedExclusion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeCalcStructuredTableFunctionCarrierMode {
    SparseReferenceLike,
    ReferenceShapeOnly,
    ResolverIndexedReference,
    MultiReferenceResolver,
    DynamicArrayReferenceTransform,
    HostContextReference,
    ExternalNativeInvocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeCalcStructuredTableFunctionContextRequirement {
    SparseReaderCells,
    ReferenceExtent,
    ReferenceCoordinateAccess,
    MultiRangeAlignment,
    DynamicArraySpillPolicy,
    RowVisibilityAndFilterState,
    MetadataDisclosurePolicy,
    VolatileReferenceRebind,
    ExternalNativeInvocationPolicy,
    ImplicitIntersectionCallerContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeCalcStructuredTableFunctionAdmission {
    pub group_id: &'static str,
    pub function_names: &'static [&'static str],
    pub status: TreeCalcStructuredTableFunctionAdmissionStatus,
    pub carrier_mode: TreeCalcStructuredTableFunctionCarrierMode,
    pub required_context: &'static [TreeCalcStructuredTableFunctionContextRequirement],
    pub oxfunc_counterpart_bead: &'static str,
    pub evidence_note: &'static str,
    pub treecalc_selector_visible_to_oxfunc: bool,
    pub eager_materialization_closure_allowed: bool,
}

pub const TREECALC_STRUCTURED_TABLE_FUNCTION_ADMISSION_INVENTORY:
    &[TreeCalcStructuredTableFunctionAdmission] = &[
    TreeCalcStructuredTableFunctionAdmission {
        group_id: "first_aggregate_group",
        function_names: &["SUM", "COUNT", "COUNTA", "COUNTBLANK"],
        status: TreeCalcStructuredTableFunctionAdmissionStatus::CurrentReferencePreservingEvidence,
        carrier_mode: TreeCalcStructuredTableFunctionCarrierMode::SparseReferenceLike,
        required_context: &[
            TreeCalcStructuredTableFunctionContextRequirement::SparseReaderCells,
            TreeCalcStructuredTableFunctionContextRequirement::ReferenceExtent,
        ],
        oxfunc_counterpart_bead: "oxf-ypq2.15",
        evidence_note: "current OxCalc/OxFml/OxFunc sparse ReferenceLike runtime evidence; widen under oxf-ypq2.15",
        treecalc_selector_visible_to_oxfunc: false,
        eager_materialization_closure_allowed: false,
    },
    TreeCalcStructuredTableFunctionAdmission {
        group_id: "shape_functions",
        function_names: &["ROWS", "COLUMNS"],
        status: TreeCalcStructuredTableFunctionAdmissionStatus::AdmittedPendingOxFuncEvidence,
        carrier_mode: TreeCalcStructuredTableFunctionCarrierMode::ReferenceShapeOnly,
        required_context: &[TreeCalcStructuredTableFunctionContextRequirement::ReferenceExtent],
        oxfunc_counterpart_bead: "oxf-ypq2.16",
        evidence_note: "must consume declared extent from generic reference resolver without reading cells",
        treecalc_selector_visible_to_oxfunc: false,
        eager_materialization_closure_allowed: false,
    },
    TreeCalcStructuredTableFunctionAdmission {
        group_id: "indexed_reference_functions",
        function_names: &["INDEX"],
        status: TreeCalcStructuredTableFunctionAdmissionStatus::AdmittedPendingOxFuncEvidence,
        carrier_mode: TreeCalcStructuredTableFunctionCarrierMode::ResolverIndexedReference,
        required_context: &[
            TreeCalcStructuredTableFunctionContextRequirement::ReferenceExtent,
            TreeCalcStructuredTableFunctionContextRequirement::ReferenceCoordinateAccess,
        ],
        oxfunc_counterpart_bead: "oxf-ypq2.16",
        evidence_note: "must read coordinates through generic resolver APIs and preserve reference-returning forms",
        treecalc_selector_visible_to_oxfunc: false,
        eager_materialization_closure_allowed: false,
    },
    TreeCalcStructuredTableFunctionAdmission {
        group_id: "lookup_match_functions",
        function_names: &["MATCH", "XMATCH", "XLOOKUP", "VLOOKUP", "HLOOKUP", "LOOKUP"],
        status: TreeCalcStructuredTableFunctionAdmissionStatus::AdmittedPendingOxFuncEvidence,
        carrier_mode: TreeCalcStructuredTableFunctionCarrierMode::MultiReferenceResolver,
        required_context: &[
            TreeCalcStructuredTableFunctionContextRequirement::SparseReaderCells,
            TreeCalcStructuredTableFunctionContextRequirement::ReferenceExtent,
            TreeCalcStructuredTableFunctionContextRequirement::MultiRangeAlignment,
        ],
        oxfunc_counterpart_bead: "oxf-ypq2.16",
        evidence_note: "must compare through opaque sparse/reference readers and keep lookup array/result array alignment generic",
        treecalc_selector_visible_to_oxfunc: false,
        eager_materialization_closure_allowed: false,
    },
    TreeCalcStructuredTableFunctionAdmission {
        group_id: "ordinary_range_scan_functions",
        function_names: &[
            "AVERAGE",
            "AVERAGEA",
            "MIN",
            "MINA",
            "MAX",
            "MAXA",
            "PRODUCT",
            "MEDIAN",
            "MODE.SNGL",
            "MODE.MULT",
            "STDEV.S",
            "STDEV.P",
            "VAR.S",
            "VAR.P",
            "LARGE",
            "SMALL",
            "AVEDEV",
            "DEVSQ",
            "GEOMEAN",
            "HARMEAN",
            "TRIMMEAN",
            "AND",
            "OR",
            "TEXTJOIN",
            "CONCAT",
            "CONCATENATE",
        ],
        status: TreeCalcStructuredTableFunctionAdmissionStatus::AdmittedPendingOxFuncEvidence,
        carrier_mode: TreeCalcStructuredTableFunctionCarrierMode::SparseReferenceLike,
        required_context: &[
            TreeCalcStructuredTableFunctionContextRequirement::SparseReaderCells,
            TreeCalcStructuredTableFunctionContextRequirement::ReferenceExtent,
        ],
        oxfunc_counterpart_bead: "oxf-ypq2.15",
        evidence_note: "ordinary aggregate/statistical/logical/text range scans must consume opaque sparse references through OxFunc resolver APIs",
        treecalc_selector_visible_to_oxfunc: false,
        eager_materialization_closure_allowed: false,
    },
    TreeCalcStructuredTableFunctionAdmission {
        group_id: "conditional_aggregate_functions",
        function_names: &[
            "SUMIF",
            "SUMIFS",
            "COUNTIF",
            "COUNTIFS",
            "AVERAGEIF",
            "AVERAGEIFS",
            "MAXIFS",
            "MINIFS",
        ],
        status: TreeCalcStructuredTableFunctionAdmissionStatus::AdmittedPendingOxFuncEvidence,
        carrier_mode: TreeCalcStructuredTableFunctionCarrierMode::MultiReferenceResolver,
        required_context: &[
            TreeCalcStructuredTableFunctionContextRequirement::SparseReaderCells,
            TreeCalcStructuredTableFunctionContextRequirement::ReferenceExtent,
            TreeCalcStructuredTableFunctionContextRequirement::MultiRangeAlignment,
        ],
        oxfunc_counterpart_bead: "oxf-ypq2.15",
        evidence_note: "must consume criteria and value ranges through generic aligned sparse-reference readers",
        treecalc_selector_visible_to_oxfunc: false,
        eager_materialization_closure_allowed: false,
    },
    TreeCalcStructuredTableFunctionAdmission {
        group_id: "dynamic_array_table_transforms",
        function_names: &[
            "FILTER",
            "SORT",
            "SORTBY",
            "UNIQUE",
            "TAKE",
            "DROP",
            "CHOOSECOLS",
            "CHOOSEROWS",
            "TOCOL",
            "TOROW",
            "WRAPROWS",
            "WRAPCOLS",
        ],
        status: TreeCalcStructuredTableFunctionAdmissionStatus::RequiresTypedHostContext,
        carrier_mode: TreeCalcStructuredTableFunctionCarrierMode::DynamicArrayReferenceTransform,
        required_context: &[
            TreeCalcStructuredTableFunctionContextRequirement::SparseReaderCells,
            TreeCalcStructuredTableFunctionContextRequirement::ReferenceExtent,
            TreeCalcStructuredTableFunctionContextRequirement::DynamicArraySpillPolicy,
        ],
        oxfunc_counterpart_bead: "oxf-ypq2.16",
        evidence_note: "requires generic spill/dynamic-array policy before product admission over table references",
        treecalc_selector_visible_to_oxfunc: false,
        eager_materialization_closure_allowed: false,
    },
    TreeCalcStructuredTableFunctionAdmission {
        group_id: "subtotal_aggregate_context_sensitive",
        function_names: &["SUBTOTAL", "AGGREGATE"],
        status: TreeCalcStructuredTableFunctionAdmissionStatus::RequiresTypedHostContext,
        carrier_mode: TreeCalcStructuredTableFunctionCarrierMode::HostContextReference,
        required_context: &[
            TreeCalcStructuredTableFunctionContextRequirement::SparseReaderCells,
            TreeCalcStructuredTableFunctionContextRequirement::ReferenceExtent,
            TreeCalcStructuredTableFunctionContextRequirement::RowVisibilityAndFilterState,
        ],
        oxfunc_counterpart_bead: "oxf-ypq2.16",
        evidence_note: "requires typed row-hidden/filter/subtotal context; no TreeCalc table branch may stand in for it",
        treecalc_selector_visible_to_oxfunc: false,
        eager_materialization_closure_allowed: false,
    },
    TreeCalcStructuredTableFunctionAdmission {
        group_id: "reference_metadata_functions",
        function_names: &["AREAS", "FORMULATEXT", "CELL", "ROW", "COLUMN", "ADDRESS"],
        status: TreeCalcStructuredTableFunctionAdmissionStatus::RequiresTypedHostContext,
        carrier_mode: TreeCalcStructuredTableFunctionCarrierMode::HostContextReference,
        required_context: &[
            TreeCalcStructuredTableFunctionContextRequirement::ReferenceExtent,
            TreeCalcStructuredTableFunctionContextRequirement::MetadataDisclosurePolicy,
            TreeCalcStructuredTableFunctionContextRequirement::ImplicitIntersectionCallerContext,
        ],
        oxfunc_counterpart_bead: "oxf-ypq2.16",
        evidence_note: "requires generic metadata disclosure and caller-context rules before table references are product admitted",
        treecalc_selector_visible_to_oxfunc: false,
        eager_materialization_closure_allowed: false,
    },
    TreeCalcStructuredTableFunctionAdmission {
        group_id: "volatile_reference_constructors",
        function_names: &["OFFSET", "INDIRECT"],
        status: TreeCalcStructuredTableFunctionAdmissionStatus::RequiresTypedHostContext,
        carrier_mode: TreeCalcStructuredTableFunctionCarrierMode::ResolverIndexedReference,
        required_context: &[
            TreeCalcStructuredTableFunctionContextRequirement::ReferenceExtent,
            TreeCalcStructuredTableFunctionContextRequirement::ReferenceCoordinateAccess,
            TreeCalcStructuredTableFunctionContextRequirement::VolatileReferenceRebind,
        ],
        oxfunc_counterpart_bead: "oxf-ypq2.16",
        evidence_note: "requires generic volatile/dynamic rebind context rather than host-side eager table materialization",
        treecalc_selector_visible_to_oxfunc: false,
        eager_materialization_closure_allowed: false,
    },
    TreeCalcStructuredTableFunctionAdmission {
        group_id: "reference_operator_forms",
        function_names: &["OP_IMPLICIT_INTERSECTION", "OP_SPILL_REF"],
        status: TreeCalcStructuredTableFunctionAdmissionStatus::RequiresTypedHostContext,
        carrier_mode: TreeCalcStructuredTableFunctionCarrierMode::HostContextReference,
        required_context: &[
            TreeCalcStructuredTableFunctionContextRequirement::ImplicitIntersectionCallerContext,
            TreeCalcStructuredTableFunctionContextRequirement::DynamicArraySpillPolicy,
        ],
        oxfunc_counterpart_bead: "oxf-ypq2.16",
        evidence_note: "operators need generic caller/spill context and remain outside OxCalc table semantics",
        treecalc_selector_visible_to_oxfunc: false,
        eager_materialization_closure_allowed: false,
    },
    TreeCalcStructuredTableFunctionAdmission {
        group_id: "native_external_invocation",
        function_names: &["CALL"],
        status: TreeCalcStructuredTableFunctionAdmissionStatus::TypedExclusion,
        carrier_mode: TreeCalcStructuredTableFunctionCarrierMode::ExternalNativeInvocation,
        required_context: &[
            TreeCalcStructuredTableFunctionContextRequirement::ExternalNativeInvocationPolicy,
        ],
        oxfunc_counterpart_bead: "oxf-ypq2.16",
        evidence_note: "native invocation policy is outside table-reference semantics and must not inspect TreeCalc selectors",
        treecalc_selector_visible_to_oxfunc: false,
        eager_materialization_closure_allowed: false,
    },
];

#[must_use]
pub fn treecalc_structured_table_function_admission(
    function_name: &str,
) -> Option<&'static TreeCalcStructuredTableFunctionAdmission> {
    TREECALC_STRUCTURED_TABLE_FUNCTION_ADMISSION_INVENTORY
        .iter()
        .find(|admission| {
            admission
                .function_names
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(function_name))
        })
}

#[derive(Debug, Clone, PartialEq)]
struct TreeCalcTableSparseSlot {
    coord: SparseCellCoord,
    absolute_row: u32,
    absolute_col: u32,
    section: TreeCalcTableSparseSection,
    row_id: Option<TreeCalcTableRowId>,
    column_id: String,
    value: Option<EvalValue>,
}

#[derive(Debug, Default)]
struct TreeCalcTableSparseReaderTelemetry {
    contains_calls: Cell<usize>,
    read_at_calls: Cell<usize>,
    defined_iter_calls: Cell<usize>,
    defined_iter_yield_count: Cell<usize>,
}

impl TreeCalcTableSparseReaderTelemetry {
    fn record_contains(&self) {
        self.contains_calls.set(self.contains_calls.get() + 1);
    }

    fn record_read_at(&self) {
        self.read_at_calls.set(self.read_at_calls.get() + 1);
    }

    fn record_defined_iter(&self) {
        self.defined_iter_calls
            .set(self.defined_iter_calls.get() + 1);
    }

    fn record_defined_yield(&self) {
        self.defined_iter_yield_count
            .set(self.defined_iter_yield_count.get() + 1);
    }

    fn summary(&self) -> SparseReaderAccessSummary {
        SparseReaderAccessSummary {
            contains_calls: self.contains_calls.get(),
            read_at_calls: self.read_at_calls.get(),
            defined_iter_calls: self.defined_iter_calls.get(),
            defined_iter_yield_count: self.defined_iter_yield_count.get(),
        }
    }
}

#[derive(Debug)]
pub struct TreeCalcTableSparseReader {
    identity: SparseReaderIdentity,
    reference: ReferenceLike,
    sheet_scope_ref: String,
    extent: SparseRangeExtent,
    slots: Vec<TreeCalcTableSparseSlot>,
    defined_cells: BTreeMap<SparseCellCoord, EvalValue>,
    current_row_sensitive: bool,
    telemetry: TreeCalcTableSparseReaderTelemetry,
}

impl TreeCalcTableSparseReader {
    pub fn from_oxfml_bind_record(
        snapshot: &TreeCalcTableNodeSnapshot,
        projection: &TreeCalcTableNodeProjection,
        record: &StructuredReferenceBindRecord,
        caller_table_region: Option<&TableCallerRegion>,
        values: impl IntoIterator<Item = TreeCalcTableSparseValue>,
    ) -> Result<Self, TreeCalcTableSparseReaderError> {
        let reference =
            StructuredTableReferenceIntake::from_oxfml_bind_record(record).map_err(|error| {
                TreeCalcTableSparseReaderError::BindRecordIntake {
                    detail: format!("{error:?}"),
                }
            })?;
        let resolved_reference = record
            .resolved_reference
            .as_ref()
            .map(reference_like_from_structured_resolved_ref);
        Self::from_reference_intake(
            snapshot,
            projection,
            &reference,
            caller_table_region,
            values,
            record.source_token_text.clone(),
            record.bind_record_handle.clone(),
            resolved_reference,
        )
    }

    pub fn from_reference_intake(
        snapshot: &TreeCalcTableNodeSnapshot,
        projection: &TreeCalcTableNodeProjection,
        reference: &StructuredTableReferenceIntake,
        caller_table_region: Option<&TableCallerRegion>,
        values: impl IntoIterator<Item = TreeCalcTableSparseValue>,
        source_token_text: impl Into<String>,
        reference_handle: impl Into<String>,
        resolved_reference: Option<ReferenceLike>,
    ) -> Result<Self, TreeCalcTableSparseReaderError> {
        if projection.table_id != snapshot.table_id {
            return Err(TreeCalcTableSparseReaderError::ProjectionSnapshotMismatch {
                projection_table_id: projection.table_id.clone(),
                snapshot_table_id: snapshot.table_id.clone(),
            });
        }

        let referenced_table_id =
            resolved_table_id_for_sparse_reference(reference).ok_or_else(|| {
                TreeCalcTableSparseReaderError::BindRecordIntake {
                    detail: "structured table sparse reader requires an effective table id"
                        .to_string(),
                }
            })?;
        if referenced_table_id != projection.table_id {
            return Err(TreeCalcTableSparseReaderError::ReferencedTableMismatch {
                referenced_table_id,
                projection_table_id: projection.table_id.clone(),
            });
        }

        let selected_columns =
            selected_columns_for_sparse_reader(&projection.table_descriptor, reference)?;
        ensure_contiguous_columns(&selected_columns)?;
        let rows = selected_rows_for_sparse_reader(
            snapshot,
            &projection.table_descriptor,
            reference,
            caller_table_region,
        )?;
        if selected_columns.is_empty() {
            return Err(TreeCalcTableSparseReaderError::EmptySelection);
        }
        let empty_data_body_selection = rows.is_empty()
            && snapshot.rows.is_empty()
            && selected_columns
                .iter()
                .all(|column| column.column_range_ref.trim().is_empty());
        if rows.is_empty() && !empty_data_body_selection {
            return Err(TreeCalcTableSparseReaderError::EmptySelection);
        }

        let current_row_sensitive = reference.uses_this_row;
        let values_by_key = values
            .into_iter()
            .map(|value| {
                (
                    (value.section, value.row_id.clone(), value.column_id.clone()),
                    value.value,
                )
            })
            .collect::<BTreeMap<_, _>>();
        let column_positions = selected_columns
            .iter()
            .map(|column| {
                let col = table_column_absolute_col(&projection.table_descriptor, column)?;
                Ok((column.column_id.clone(), col))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let mut slots = Vec::new();

        for (row_index, row) in rows.iter().enumerate() {
            for (column_index, column) in selected_columns.iter().enumerate() {
                let absolute_col = column_positions[&column.column_id];
                let (absolute_row, value) = match row {
                    TreeCalcTableSparseRow::Headers => {
                        let header_row = table_header_row(&projection.table_descriptor)?
                            .ok_or(TreeCalcTableSparseReaderError::HeaderRowAbsent)?;
                        let explicit = values_by_key
                            .get(&(
                                TreeCalcTableSparseSection::Headers,
                                None,
                                column.column_id.clone(),
                            ))
                            .cloned();
                        (
                            header_row,
                            Some(explicit.unwrap_or_else(|| {
                                EvalValue::Text(ExcelText::from_interop_assignment(
                                    &column.column_name,
                                ))
                            })),
                        )
                    }
                    TreeCalcTableSparseRow::Data { row_id, row_offset } => {
                        let data_range =
                            parse_non_empty_table_column_range(column).ok_or_else(|| {
                                TreeCalcTableSparseReaderError::MissingColumnRange {
                                    column_id: column.column_id.clone(),
                                    range_ref: column.column_range_ref.clone(),
                                }
                            })?;
                        let absolute_row = data_range
                            .top_row
                            .checked_add(*row_offset)
                            .ok_or(TreeCalcTableSparseReaderError::RangeOverflow)?;
                        let value = values_by_key
                            .get(&(
                                TreeCalcTableSparseSection::Data,
                                Some(row_id.clone()),
                                column.column_id.clone(),
                            ))
                            .cloned();
                        (absolute_row, value)
                    }
                    TreeCalcTableSparseRow::Totals => {
                        let totals_row = table_totals_row(&projection.table_descriptor)?
                            .ok_or(TreeCalcTableSparseReaderError::TotalsRowAbsent)?;
                        let value = values_by_key
                            .get(&(
                                TreeCalcTableSparseSection::Totals,
                                None,
                                column.column_id.clone(),
                            ))
                            .cloned();
                        (totals_row, value)
                    }
                };
                slots.push(TreeCalcTableSparseSlot {
                    coord: SparseCellCoord::new(
                        u32::try_from(row_index + 1)
                            .map_err(|_| TreeCalcTableSparseReaderError::RangeOverflow)?,
                        u32::try_from(column_index + 1)
                            .map_err(|_| TreeCalcTableSparseReaderError::RangeOverflow)?,
                    ),
                    absolute_row,
                    absolute_col,
                    section: row.section(),
                    row_id: row.row_id().cloned(),
                    column_id: column.column_id.clone(),
                    value,
                });
            }
        }

        let reference = resolved_reference.unwrap_or_else(|| {
            reference_like_for_sparse_slots(&projection.table_descriptor.sheet_scope_ref, &slots)
                .unwrap_or_else(|| {
                    reference_like_for_empty_table_selection(
                        &projection.table_descriptor,
                        reference,
                        &selected_columns,
                    )
                })
        });
        let extent = SparseRangeExtent::new(
            SparseCellCoord::new(1, 1),
            u32::try_from(rows.len()).map_err(|_| TreeCalcTableSparseReaderError::RangeOverflow)?,
            u32::try_from(selected_columns.len())
                .map_err(|_| TreeCalcTableSparseReaderError::RangeOverflow)?,
        );
        let defined_cells = slots
            .iter()
            .filter_map(|slot| slot.value.clone().map(|value| (slot.coord, value)))
            .collect::<BTreeMap<_, _>>();
        let reference_handle = reference_handle.into();
        let source_token_text = source_token_text.into();
        let identity = SparseReaderIdentity::new(
            format!("treecalc-table-reader:v1:{reference_handle}"),
            identity_record(
                "treecalc.table_sparse_selection.v1",
                [
                    ("table_id", projection.table_id.clone()),
                    ("source", source_token_text),
                    (
                        "sections",
                        identity_list(rows.iter().map(|row| format!("{:?}", row.section()))),
                    ),
                    (
                        "columns",
                        identity_list(
                            selected_columns
                                .iter()
                                .map(|column| format!("{}:{}", column.ordinal, column.column_id)),
                        ),
                    ),
                    ("reference", reference.target.clone()),
                ],
            ),
            identity_record(
                "treecalc.table_sparse_snapshot.v1",
                [
                    ("context", projection.table_context_identity.clone()),
                    (
                        "membership",
                        projection.oxcalc_row_membership_identity.clone(),
                    ),
                    ("order", projection.oxcalc_row_order_identity.clone()),
                    ("columns", projection.oxcalc_column_identity.clone()),
                    ("body", projection.body_metadata_identity.clone()),
                    ("totals", projection.totals_metadata_identity.clone()),
                ],
            ),
        );

        Ok(Self {
            identity,
            reference,
            sheet_scope_ref: projection.table_descriptor.sheet_scope_ref.clone(),
            extent,
            slots,
            defined_cells,
            current_row_sensitive,
            telemetry: TreeCalcTableSparseReaderTelemetry::default(),
        })
    }

    #[must_use]
    pub fn reference(&self) -> &ReferenceLike {
        &self.reference
    }

    #[must_use]
    pub fn access_summary(&self) -> SparseReaderAccessSummary {
        self.telemetry.summary()
    }

    #[must_use]
    pub fn runtime_binding(&self) -> TreeCalcStructuredTableRuntimeBinding {
        let sparse_reference_values = RuntimeSparseReferenceValuesBinding {
            reference: self.reference.clone(),
            declared_rows: usize::try_from(self.extent.row_count).unwrap_or(usize::MAX),
            declared_cols: usize::try_from(self.extent.column_count).unwrap_or(usize::MAX),
            defined_cells: self
                .defined_iter()
                .map(|cell| {
                    RuntimeSparseReferenceCell::new(
                        usize::try_from(cell.coord.row).unwrap_or(usize::MAX),
                        usize::try_from(cell.coord.column).unwrap_or(usize::MAX),
                        table_eval_value_to_array_cell(cell.value),
                    )
                })
                .collect(),
            reader_identity: Some(format!(
                "reader_id={};source={};snapshot={}",
                self.identity.reader_id,
                self.identity.source_identity,
                self.identity.snapshot_identity
            )),
        };
        TreeCalcStructuredTableRuntimeBinding {
            reference: self.reference.clone(),
            sparse_reference_values,
            scalar_cell_values: self.current_row_cell_values(),
            reader_identity: format!(
                "reader_id={};source={};snapshot={}",
                self.identity.reader_id,
                self.identity.source_identity,
                self.identity.snapshot_identity
            ),
        }
    }

    #[must_use]
    pub fn current_row_cell_values(&self) -> BTreeMap<String, EvalValue> {
        if !self.current_row_sensitive {
            return BTreeMap::new();
        }
        self.slots
            .iter()
            .filter_map(|slot| {
                slot.value.clone().map(|value| {
                    (
                        qualified_local_reference_target(
                            &self.sheet_scope_ref,
                            &a1_cell_ref(slot.absolute_row, slot.absolute_col)
                                .unwrap_or_else(|| "#REF!".to_string()),
                        ),
                        value,
                    )
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableFormulaRuntimeContext {
    pub dialect_id: String,
    pub capability_profile_id: String,
    pub resolution_rule_version: String,
    pub host_namespace_version: Option<String>,
    pub structure_context_version: String,
    pub registry_snapshot_identity: Option<String>,
}

impl Default for TreeCalcTableFormulaRuntimeContext {
    fn default() -> Self {
        Self {
            dialect_id: "oxcalc.treecalc-v1".to_string(),
            capability_profile_id: "host-capabilities:treecalc-v1".to_string(),
            resolution_rule_version: "treecalc-host-resolution:v1".to_string(),
            host_namespace_version: Some("treecalc-host-namespace:v1".to_string()),
            structure_context_version: "treecalc-structure:v1".to_string(),
            registry_snapshot_identity: Some(builtin_registry().snapshot_identity().stable_key()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TreeCalcTableColumnFormulaRuntimeRequest {
    pub target_column_id: String,
    pub formula_stable_id: String,
    pub formula_text_version: u64,
    pub formula_text: String,
    pub values: Vec<TreeCalcTableSparseValue>,
    pub runtime_context: TreeCalcTableFormulaRuntimeContext,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TreeCalcTableFormulaRuntimeReport {
    pub table_id: String,
    pub target_column_id: String,
    pub formula_stable_id: String,
    pub formula_text_version: u64,
    pub formula_text: String,
    pub table_context_identity: String,
    pub cell_results: Vec<TreeCalcTableFormulaRuntimeCellResult>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TreeCalcTableFormulaRuntimeCellResult {
    pub row_id: Option<TreeCalcTableRowId>,
    pub row_offset: Option<u32>,
    pub region_kind: TableRegionKind,
    pub caller_context_id: String,
    pub primary_locus: Locus,
    pub value: EvalValue,
    pub prepared_formula_key: String,
    pub dispatch_skeleton_key: String,
    pub plan_template_key: String,
    pub registry_snapshot_identity: Option<String>,
    pub host_formula_context: RuntimeHostFormulaContext,
    pub structured_reference_handles: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TreeCalcTableFormulaRuntimeError {
    ProjectionSnapshotMismatch {
        projection_table_id: String,
        snapshot_table_id: String,
    },
    MissingTargetColumn {
        column_id: String,
    },
    TargetColumnBodyNotFormula {
        column_id: String,
    },
    TargetColumnTotalsNotFormula {
        column_id: String,
    },
    MissingTargetColumnRange {
        column_id: String,
    },
    InvalidTargetColumnRange {
        column_id: String,
        range_ref: String,
    },
    TotalsRowAbsent,
    PrebindDiagnostics {
        row_id: Option<TreeCalcTableRowId>,
        diagnostics: Vec<TreeCalcTableStructuredReferenceDiagnostic>,
    },
    SparseReader {
        row_id: Option<TreeCalcTableRowId>,
        error: TreeCalcTableSparseReaderError,
    },
    RuntimeExecution {
        row_id: Option<TreeCalcTableRowId>,
        detail: String,
    },
    RuntimeDiagnostics {
        row_id: Option<TreeCalcTableRowId>,
        syntax_count: usize,
        bind_count: usize,
    },
}

pub fn evaluate_treecalc_table_column_formula_rows(
    snapshot: &TreeCalcTableNodeSnapshot,
    projection: &TreeCalcTableNodeProjection,
    request: &TreeCalcTableColumnFormulaRuntimeRequest,
) -> Result<TreeCalcTableFormulaRuntimeReport, TreeCalcTableFormulaRuntimeError> {
    ensure_formula_projection_matches_snapshot(snapshot, projection)?;
    ensure_body_formula_column(snapshot, &request.target_column_id)?;

    let cell_results = snapshot
        .rows
        .iter()
        .enumerate()
        .map(|(row_index, row_id)| {
            let row_offset = u32::try_from(row_index).map_err(|_| {
                TreeCalcTableFormulaRuntimeError::InvalidTargetColumnRange {
                    column_id: request.target_column_id.clone(),
                    range_ref: "row offset overflow".to_string(),
                }
            })?;
            evaluate_treecalc_table_formula_at_region(
                snapshot,
                projection,
                request,
                TreeCalcTableFormulaRuntimeRegion::Data {
                    row_id: row_id.clone(),
                    row_offset,
                },
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(TreeCalcTableFormulaRuntimeReport {
        table_id: projection.table_id.clone(),
        target_column_id: request.target_column_id.clone(),
        formula_stable_id: request.formula_stable_id.clone(),
        formula_text_version: request.formula_text_version,
        formula_text: request.formula_text.clone(),
        table_context_identity: projection.table_context_identity.clone(),
        cell_results,
    })
}

pub fn evaluate_treecalc_table_totals_formula(
    snapshot: &TreeCalcTableNodeSnapshot,
    projection: &TreeCalcTableNodeProjection,
    request: &TreeCalcTableColumnFormulaRuntimeRequest,
) -> Result<TreeCalcTableFormulaRuntimeCellResult, TreeCalcTableFormulaRuntimeError> {
    ensure_formula_projection_matches_snapshot(snapshot, projection)?;
    ensure_totals_formula_column(snapshot, &request.target_column_id)?;
    if !snapshot.totals_row_present {
        return Err(TreeCalcTableFormulaRuntimeError::TotalsRowAbsent);
    }
    evaluate_treecalc_table_formula_at_region(
        snapshot,
        projection,
        request,
        TreeCalcTableFormulaRuntimeRegion::Totals,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TreeCalcTableFormulaRuntimeRegion {
    Data {
        row_id: TreeCalcTableRowId,
        row_offset: u32,
    },
    Totals,
}

impl TreeCalcTableFormulaRuntimeRegion {
    fn row_id(&self) -> Option<TreeCalcTableRowId> {
        match self {
            Self::Data { row_id, .. } => Some(row_id.clone()),
            Self::Totals => None,
        }
    }

    fn row_offset(&self) -> Option<u32> {
        match self {
            Self::Data { row_offset, .. } => Some(*row_offset),
            Self::Totals => None,
        }
    }

    fn region_kind(&self) -> TableRegionKind {
        match self {
            Self::Data { .. } => TableRegionKind::Data,
            Self::Totals => TableRegionKind::Totals,
        }
    }

    fn caller_region(&self, table_id: &str) -> TableCallerRegion {
        TableCallerRegion {
            table_id: table_id.to_string(),
            region_kind: self.region_kind(),
            data_row_offset: self.row_offset(),
        }
    }
}

fn ensure_formula_projection_matches_snapshot(
    snapshot: &TreeCalcTableNodeSnapshot,
    projection: &TreeCalcTableNodeProjection,
) -> Result<(), TreeCalcTableFormulaRuntimeError> {
    if projection.table_id == snapshot.table_id {
        Ok(())
    } else {
        Err(
            TreeCalcTableFormulaRuntimeError::ProjectionSnapshotMismatch {
                projection_table_id: projection.table_id.clone(),
                snapshot_table_id: snapshot.table_id.clone(),
            },
        )
    }
}

fn ensure_body_formula_column(
    snapshot: &TreeCalcTableNodeSnapshot,
    column_id: &str,
) -> Result<(), TreeCalcTableFormulaRuntimeError> {
    let column = snapshot
        .columns
        .iter()
        .find(|column| column.column_id == column_id)
        .ok_or_else(|| TreeCalcTableFormulaRuntimeError::MissingTargetColumn {
            column_id: column_id.to_string(),
        })?;
    match column.body_metadata {
        TreeCalcTableColumnBodyMetadata::Formula(_) => Ok(()),
        TreeCalcTableColumnBodyMetadata::ConstantCells => Err(
            TreeCalcTableFormulaRuntimeError::TargetColumnBodyNotFormula {
                column_id: column_id.to_string(),
            },
        ),
    }
}

fn ensure_totals_formula_column(
    snapshot: &TreeCalcTableNodeSnapshot,
    column_id: &str,
) -> Result<(), TreeCalcTableFormulaRuntimeError> {
    let column = snapshot
        .columns
        .iter()
        .find(|column| column.column_id == column_id)
        .ok_or_else(|| TreeCalcTableFormulaRuntimeError::MissingTargetColumn {
            column_id: column_id.to_string(),
        })?;
    if column.totals_metadata.is_some() {
        Ok(())
    } else {
        Err(
            TreeCalcTableFormulaRuntimeError::TargetColumnTotalsNotFormula {
                column_id: column_id.to_string(),
            },
        )
    }
}

fn evaluate_treecalc_table_formula_at_region(
    snapshot: &TreeCalcTableNodeSnapshot,
    projection: &TreeCalcTableNodeProjection,
    request: &TreeCalcTableColumnFormulaRuntimeRequest,
    region: TreeCalcTableFormulaRuntimeRegion,
) -> Result<TreeCalcTableFormulaRuntimeCellResult, TreeCalcTableFormulaRuntimeError> {
    let caller_region = region.caller_region(&projection.table_id);
    let row_id = region.row_id();
    let primary_locus =
        treecalc_table_formula_primary_locus(projection, &request.target_column_id, &region)?;
    let prebound = prebind_treecalc_table_structured_references(
        &request.formula_text,
        std::slice::from_ref(projection),
        Some(TableRef {
            table_id: projection.table_id.clone(),
        }),
        Some(caller_region.clone()),
    );
    let diagnostics = prebound
        .iter()
        .flat_map(|prebind| prebind.diagnostics.clone())
        .collect::<Vec<_>>();
    if !diagnostics.is_empty() {
        return Err(TreeCalcTableFormulaRuntimeError::PrebindDiagnostics {
            row_id,
            diagnostics,
        });
    }

    let mut scalar_cell_values = BTreeMap::new();
    let mut sparse_reference_value_bindings = Vec::new();
    for prebind in &prebound {
        let reader = TreeCalcTableSparseReader::from_oxfml_bind_record(
            snapshot,
            projection,
            &prebind.bind_record,
            Some(&caller_region),
            request.values.clone(),
        )
        .map_err(|error| TreeCalcTableFormulaRuntimeError::SparseReader {
            row_id: row_id.clone(),
            error,
        })?;
        let runtime_binding = reader.runtime_binding();
        scalar_cell_values.extend(runtime_binding.scalar_cell_values);
        sparse_reference_value_bindings.push(runtime_binding.sparse_reference_values);
    }

    let caller_context_id =
        treecalc_table_formula_caller_context_id(projection, &request.target_column_id, &region);
    let host_formula_context = RuntimeHostFormulaContext {
        dialect_id: request.runtime_context.dialect_id.clone(),
        capability_profile_id: request.runtime_context.capability_profile_id.clone(),
        resolution_rule_version: request.runtime_context.resolution_rule_version.clone(),
        host_namespace_version: request.runtime_context.host_namespace_version.clone(),
        registry_snapshot_identity: request.runtime_context.registry_snapshot_identity.clone(),
        structure_context_version: Some(request.runtime_context.structure_context_version.clone()),
        caller_context_identity: Some(caller_context_id.clone()),
        table_context_identity: Some(projection.table_context_identity.clone()),
    };
    let result = RuntimeEnvironment::new()
        .with_structure_context_version(StructureContextVersion(
            request.runtime_context.structure_context_version.clone(),
        ))
        .with_primary_locus(primary_locus.clone())
        .with_caller_position(primary_locus.row, primary_locus.col)
        .with_table_context(
            vec![projection.table_descriptor.clone()],
            Some(TableRef {
                table_id: projection.table_id.clone(),
            }),
            Some(caller_region),
        )
        .with_host_formula_context(host_formula_context.clone())
        .with_cell_values(scalar_cell_values)
        .with_sparse_reference_value_bindings(sparse_reference_value_bindings)
        .with_function_registry(builtin_registry())
        .execute(
            RuntimeFormulaRequest::new(
                FormulaSourceRecord::new(
                    request.formula_stable_id.clone(),
                    request.formula_text_version,
                    request.formula_text.clone(),
                ),
                oxfml_core::interface::TypedContextQueryBundle::default(),
            )
            .with_backend(EvaluationBackend::OxFuncBacked),
        )
        .map_err(
            |detail| TreeCalcTableFormulaRuntimeError::RuntimeExecution {
                row_id: row_id.clone(),
                detail,
            },
        )?;
    if !result.syntax_diagnostics.is_empty() || !result.bind_diagnostics.is_empty() {
        return Err(TreeCalcTableFormulaRuntimeError::RuntimeDiagnostics {
            row_id,
            syntax_count: result.syntax_diagnostics.len(),
            bind_count: result.bind_diagnostics.len(),
        });
    }

    Ok(TreeCalcTableFormulaRuntimeCellResult {
        row_id,
        row_offset: region.row_offset(),
        region_kind: region.region_kind(),
        caller_context_id,
        primary_locus,
        value: result.evaluation.oxfunc_value,
        prepared_formula_key: result.prepared_formula_identity.prepared_formula_key,
        dispatch_skeleton_key: result
            .prepared_formula_identity
            .plan_template
            .dispatch_skeleton_key,
        plan_template_key: result
            .prepared_formula_identity
            .plan_template
            .plan_template_key,
        registry_snapshot_identity: result.prepared_formula_identity.registry_snapshot_identity,
        host_formula_context,
        structured_reference_handles: result
            .structured_reference_bind_records
            .into_iter()
            .map(|record| record.bind_record_handle)
            .collect(),
    })
}

fn treecalc_table_formula_primary_locus(
    projection: &TreeCalcTableNodeProjection,
    target_column_id: &str,
    region: &TreeCalcTableFormulaRuntimeRegion,
) -> Result<Locus, TreeCalcTableFormulaRuntimeError> {
    let column = projection
        .table_descriptor
        .columns
        .iter()
        .find(|column| column.column_id == target_column_id)
        .ok_or_else(|| TreeCalcTableFormulaRuntimeError::MissingTargetColumn {
            column_id: target_column_id.to_string(),
        })?;
    let column_range = parse_local_a1_range(&column.column_range_ref).ok_or_else(|| {
        TreeCalcTableFormulaRuntimeError::InvalidTargetColumnRange {
            column_id: target_column_id.to_string(),
            range_ref: column.column_range_ref.clone(),
        }
    })?;
    let row = match region {
        TreeCalcTableFormulaRuntimeRegion::Data { row_offset, .. } => column_range
            .top_row
            .checked_add(*row_offset)
            .ok_or_else(
                || TreeCalcTableFormulaRuntimeError::InvalidTargetColumnRange {
                    column_id: target_column_id.to_string(),
                    range_ref: column.column_range_ref.clone(),
                },
            )?,
        TreeCalcTableFormulaRuntimeRegion::Totals => table_totals_row(&projection.table_descriptor)
            .map_err(
                |_| TreeCalcTableFormulaRuntimeError::InvalidTargetColumnRange {
                    column_id: target_column_id.to_string(),
                    range_ref: projection
                        .table_descriptor
                        .totals_region_ref
                        .clone()
                        .unwrap_or_default(),
                },
            )?
            .ok_or(TreeCalcTableFormulaRuntimeError::TotalsRowAbsent)?,
    };
    Ok(Locus {
        sheet_id: projection.table_descriptor.sheet_scope_ref.clone(),
        row,
        col: column_range.left_col,
    })
}

fn treecalc_table_formula_caller_context_id(
    projection: &TreeCalcTableNodeProjection,
    target_column_id: &str,
    region: &TreeCalcTableFormulaRuntimeRegion,
) -> String {
    match region {
        TreeCalcTableFormulaRuntimeRegion::Data { row_id, row_offset } => identity_record(
            "treecalc.table_formula_caller.v1",
            [
                ("table_id", projection.table_id.clone()),
                ("target_column_id", target_column_id.to_string()),
                ("row_order", projection.row_order_identity.clone()),
                ("table_context", projection.table_context_identity.clone()),
                ("region", "data".to_string()),
                ("row_id", row_id.0.clone()),
                ("row_offset", row_offset.to_string()),
            ],
        ),
        TreeCalcTableFormulaRuntimeRegion::Totals => identity_record(
            "treecalc.table_formula_caller.v1",
            [
                ("table_id", projection.table_id.clone()),
                ("target_column_id", target_column_id.to_string()),
                ("row_order", projection.row_order_identity.clone()),
                ("table_context", projection.table_context_identity.clone()),
                ("region", "totals".to_string()),
            ],
        ),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeCalcTableUpdateScenarioKind {
    BodyCellEdit,
    BodyFormulaEdit,
    RowInsert,
    RowDelete,
    RowReorder,
    ColumnInsert,
    ColumnDelete,
    ColumnReorder,
    ColumnRename,
    HeaderTextEdit,
    TotalsRowToggle,
    TotalsFormulaEdit,
    TableRename,
    TableMove,
    TableDelete,
    SaveReopen,
    StructuralRebind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeCalcTablePreparedIdentityInput {
    HostNamespaceVersion,
    StructureContextVersion,
    TableContextIdentity,
    CallerContextIdentity,
    RegistrySnapshotIdentity,
    ResolutionRuleVersion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableUpdateImpact {
    pub scenario: TreeCalcTableUpdateScenarioKind,
    pub before_table_context_identity: Option<String>,
    pub after_table_context_identity: Option<String>,
    pub source_reference_handles: Vec<String>,
    pub changed_dependency_kinds: BTreeSet<DependencyDescriptorKind>,
    pub invalidation_reasons: BTreeSet<InvalidationReasonKind>,
    pub prepared_identity_inputs: BTreeSet<TreeCalcTablePreparedIdentityInput>,
    pub invalidation_seeds: Vec<InvalidationSeed>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeCalcTableUpdateDiagnostic {
    DeletedTable {
        table_id: String,
        reference_handle: String,
    },
    MissingColumn {
        table_id: String,
        reference_handle: String,
        column_id: String,
    },
    MissingCallerTableRegion {
        table_id: String,
        reference_handle: String,
    },
    HeaderRowAbsent {
        table_id: String,
        reference_handle: String,
    },
    TotalsRowAbsent {
        table_id: String,
        reference_handle: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeCalcTableLifecycleEventKind {
    TableCreate,
    BodyCellEdit,
    BodyFormulaEdit,
    RowInsert,
    RowDelete,
    RowReorder,
    ColumnInsert,
    ColumnDelete,
    ColumnReorder,
    ColumnRename,
    HeaderTextEdit,
    TotalsRowToggle,
    TotalsFormulaEdit,
    TableRename,
    TableMove,
    TableDelete,
    SaveReopen,
    StructuralRebind,
}

impl TreeCalcTableLifecycleEventKind {
    #[must_use]
    pub fn update_scenario(self) -> Option<TreeCalcTableUpdateScenarioKind> {
        Some(match self {
            Self::TableCreate => return None,
            Self::BodyCellEdit => TreeCalcTableUpdateScenarioKind::BodyCellEdit,
            Self::BodyFormulaEdit => TreeCalcTableUpdateScenarioKind::BodyFormulaEdit,
            Self::RowInsert => TreeCalcTableUpdateScenarioKind::RowInsert,
            Self::RowDelete => TreeCalcTableUpdateScenarioKind::RowDelete,
            Self::RowReorder => TreeCalcTableUpdateScenarioKind::RowReorder,
            Self::ColumnInsert => TreeCalcTableUpdateScenarioKind::ColumnInsert,
            Self::ColumnDelete => TreeCalcTableUpdateScenarioKind::ColumnDelete,
            Self::ColumnReorder => TreeCalcTableUpdateScenarioKind::ColumnReorder,
            Self::ColumnRename => TreeCalcTableUpdateScenarioKind::ColumnRename,
            Self::HeaderTextEdit => TreeCalcTableUpdateScenarioKind::HeaderTextEdit,
            Self::TotalsRowToggle => TreeCalcTableUpdateScenarioKind::TotalsRowToggle,
            Self::TotalsFormulaEdit => TreeCalcTableUpdateScenarioKind::TotalsFormulaEdit,
            Self::TableRename => TreeCalcTableUpdateScenarioKind::TableRename,
            Self::TableMove => TreeCalcTableUpdateScenarioKind::TableMove,
            Self::TableDelete => TreeCalcTableUpdateScenarioKind::TableDelete,
            Self::SaveReopen => TreeCalcTableUpdateScenarioKind::SaveReopen,
            Self::StructuralRebind => TreeCalcTableUpdateScenarioKind::StructuralRebind,
        })
    }

    #[must_use]
    pub fn stable_id(self) -> &'static str {
        match self {
            Self::TableCreate => "table_create",
            Self::BodyCellEdit => "body_cell_edit",
            Self::BodyFormulaEdit => "body_formula_edit",
            Self::RowInsert => "row_insert",
            Self::RowDelete => "row_delete",
            Self::RowReorder => "row_reorder",
            Self::ColumnInsert => "column_insert",
            Self::ColumnDelete => "column_delete",
            Self::ColumnReorder => "column_reorder",
            Self::ColumnRename => "column_rename",
            Self::HeaderTextEdit => "header_text_edit",
            Self::TotalsRowToggle => "totals_row_toggle",
            Self::TotalsFormulaEdit => "totals_formula_edit",
            Self::TableRename => "table_rename",
            Self::TableMove => "table_move",
            Self::TableDelete => "table_delete",
            Self::SaveReopen => "save_reopen",
            Self::StructuralRebind => "structural_rebind",
        }
    }
}

impl From<TreeCalcTableUpdateScenarioKind> for TreeCalcTableLifecycleEventKind {
    fn from(value: TreeCalcTableUpdateScenarioKind) -> Self {
        match value {
            TreeCalcTableUpdateScenarioKind::BodyCellEdit => Self::BodyCellEdit,
            TreeCalcTableUpdateScenarioKind::BodyFormulaEdit => Self::BodyFormulaEdit,
            TreeCalcTableUpdateScenarioKind::RowInsert => Self::RowInsert,
            TreeCalcTableUpdateScenarioKind::RowDelete => Self::RowDelete,
            TreeCalcTableUpdateScenarioKind::RowReorder => Self::RowReorder,
            TreeCalcTableUpdateScenarioKind::ColumnInsert => Self::ColumnInsert,
            TreeCalcTableUpdateScenarioKind::ColumnDelete => Self::ColumnDelete,
            TreeCalcTableUpdateScenarioKind::ColumnReorder => Self::ColumnReorder,
            TreeCalcTableUpdateScenarioKind::ColumnRename => Self::ColumnRename,
            TreeCalcTableUpdateScenarioKind::HeaderTextEdit => Self::HeaderTextEdit,
            TreeCalcTableUpdateScenarioKind::TotalsRowToggle => Self::TotalsRowToggle,
            TreeCalcTableUpdateScenarioKind::TotalsFormulaEdit => Self::TotalsFormulaEdit,
            TreeCalcTableUpdateScenarioKind::TableRename => Self::TableRename,
            TreeCalcTableUpdateScenarioKind::TableMove => Self::TableMove,
            TreeCalcTableUpdateScenarioKind::TableDelete => Self::TableDelete,
            TreeCalcTableUpdateScenarioKind::SaveReopen => Self::SaveReopen,
            TreeCalcTableUpdateScenarioKind::StructuralRebind => Self::StructuralRebind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableLifecycleContextVersions {
    pub host_namespace_version: Option<String>,
    pub structure_context_version: String,
    pub registry_snapshot_identity: String,
    pub resolution_rule_version: String,
}

impl Default for TreeCalcTableLifecycleContextVersions {
    fn default() -> Self {
        Self {
            host_namespace_version: Some("treecalc-host-namespace:v1".to_string()),
            structure_context_version: "treecalc-structure:v1".to_string(),
            registry_snapshot_identity: "oxfunc-registry:default".to_string(),
            resolution_rule_version: "treecalc-host-resolution:v1".to_string(),
        }
    }
}

impl TreeCalcTableLifecycleContextVersions {
    #[must_use]
    pub fn identity_fragment(&self) -> String {
        identity_record(
            "treecalc.table_lifecycle.context_versions.v1",
            [
                (
                    "host_namespace_version",
                    self.host_namespace_version
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
                ),
                (
                    "structure_context_version",
                    self.structure_context_version.clone(),
                ),
                (
                    "registry_snapshot_identity",
                    self.registry_snapshot_identity.clone(),
                ),
                (
                    "resolution_rule_version",
                    self.resolution_rule_version.clone(),
                ),
            ],
        )
    }

    #[must_use]
    pub fn prepared_identity_inputs(&self) -> BTreeSet<TreeCalcTablePreparedIdentityInput> {
        let mut inputs = BTreeSet::from([
            TreeCalcTablePreparedIdentityInput::StructureContextVersion,
            TreeCalcTablePreparedIdentityInput::RegistrySnapshotIdentity,
            TreeCalcTablePreparedIdentityInput::ResolutionRuleVersion,
        ]);
        if self.host_namespace_version.is_some() {
            inputs.insert(TreeCalcTablePreparedIdentityInput::HostNamespaceVersion);
        }
        inputs
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableLifecycleVersionState {
    pub table_node_id: TreeNodeId,
    pub table_id: String,
    pub table_name: String,
    pub display_path: String,
    pub canonical_path: String,
    pub workbook_scope_ref: String,
    pub sheet_scope_ref: String,
    pub table_range_ref: String,
    pub header_region_ref: Option<String>,
    pub totals_region_ref: Option<String>,
    pub column_range_refs: Vec<String>,
    pub virtual_anchor_identity: String,
    pub virtual_anchor_token: String,
    pub table_context_identity: String,
    pub table_invalidation_identity: String,
    pub table_namespace_identity: String,
    pub table_namespace_version: String,
    pub row_membership_identity: String,
    pub row_membership_version: String,
    pub row_order_identity: String,
    pub row_order_version: String,
    pub column_identity: String,
    pub column_identity_version: String,
    pub row_ids: Vec<TreeCalcTableRowId>,
    pub column_ids: Vec<String>,
}

impl TreeCalcTableLifecycleVersionState {
    #[must_use]
    pub fn from_snapshot_projection(
        snapshot: &TreeCalcTableNodeSnapshot,
        projection: &TreeCalcTableNodeProjection,
    ) -> Self {
        Self {
            table_node_id: snapshot.table_node_id,
            table_id: snapshot.table_id.clone(),
            table_name: snapshot.table_name.clone(),
            display_path: snapshot.display_path.clone(),
            canonical_path: snapshot.canonical_path.clone(),
            workbook_scope_ref: snapshot.virtual_anchor.workbook_scope_ref.clone(),
            sheet_scope_ref: snapshot.virtual_anchor.sheet_scope_ref.clone(),
            table_range_ref: projection.table_descriptor.table_range_ref.clone(),
            header_region_ref: projection.table_descriptor.header_region_ref.clone(),
            totals_region_ref: projection.table_descriptor.totals_region_ref.clone(),
            column_range_refs: projection
                .table_descriptor
                .columns
                .iter()
                .map(|column| format!("{}={}", column.column_id, column.column_range_ref))
                .collect(),
            virtual_anchor_identity: projection.virtual_anchor_identity.clone(),
            virtual_anchor_token: projection.virtual_anchor_token.clone(),
            table_context_identity: projection.table_context_identity.clone(),
            table_invalidation_identity: projection.table_invalidation_identity.clone(),
            table_namespace_identity: projection.table_namespace_identity.clone(),
            table_namespace_version: snapshot.table_namespace_version.clone(),
            row_membership_identity: projection.oxcalc_row_membership_identity.clone(),
            row_membership_version: snapshot.row_membership_version.clone(),
            row_order_identity: projection.oxcalc_row_order_identity.clone(),
            row_order_version: snapshot.row_order_version.clone(),
            column_identity: projection.oxcalc_column_identity.clone(),
            column_identity_version: snapshot.column_identity_version.clone(),
            row_ids: snapshot.rows.clone(),
            column_ids: sorted_treecalc_table_columns(snapshot)
                .map(|columns| columns.into_iter().map(|column| column.column_id).collect())
                .unwrap_or_else(|_| {
                    snapshot
                        .columns
                        .iter()
                        .map(|column| column.column_id.clone())
                        .collect()
                }),
        }
    }

    #[must_use]
    pub fn identity_fragment(&self) -> String {
        identity_record(
            "treecalc.table_lifecycle.version_state.v1",
            [
                ("table_node_id", self.table_node_id.to_string()),
                ("table_id", self.table_id.clone()),
                ("table_name", self.table_name.clone()),
                ("display_path", self.display_path.clone()),
                ("canonical_path", self.canonical_path.clone()),
                ("workbook_scope_ref", self.workbook_scope_ref.clone()),
                ("sheet_scope_ref", self.sheet_scope_ref.clone()),
                ("table_range_ref", self.table_range_ref.clone()),
                (
                    "header_region_ref",
                    self.header_region_ref
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
                ),
                (
                    "totals_region_ref",
                    self.totals_region_ref
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
                ),
                (
                    "column_range_refs",
                    identity_list(self.column_range_refs.iter().cloned()),
                ),
                (
                    "table_context_identity",
                    self.table_context_identity.clone(),
                ),
                (
                    "table_invalidation_identity",
                    self.table_invalidation_identity.clone(),
                ),
                (
                    "table_namespace_identity",
                    self.table_namespace_identity.clone(),
                ),
                (
                    "table_namespace_version",
                    self.table_namespace_version.clone(),
                ),
                (
                    "row_membership_identity",
                    self.row_membership_identity.clone(),
                ),
                (
                    "row_membership_version",
                    self.row_membership_version.clone(),
                ),
                ("row_order_identity", self.row_order_identity.clone()),
                ("row_order_version", self.row_order_version.clone()),
                ("column_identity", self.column_identity.clone()),
                (
                    "column_identity_version",
                    self.column_identity_version.clone(),
                ),
                (
                    "virtual_anchor_identity",
                    self.virtual_anchor_identity.clone(),
                ),
                ("virtual_anchor_token", self.virtual_anchor_token.clone()),
                (
                    "rows",
                    identity_list(self.row_ids.iter().map(|row| row.0.clone())),
                ),
                ("columns", identity_list(self.column_ids.iter().cloned())),
            ],
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableLifecycleCallbackPacket {
    pub event_kind: TreeCalcTableLifecycleEventKind,
    pub before_state: Option<TreeCalcTableLifecycleVersionState>,
    pub after_state: Option<TreeCalcTableLifecycleVersionState>,
    pub context_versions: TreeCalcTableLifecycleContextVersions,
    pub owner_node_ids: Vec<TreeNodeId>,
    pub source_reference_handles: Vec<String>,
    pub changed_row_ids: Vec<TreeCalcTableRowId>,
    pub changed_column_ids: Vec<String>,
}

impl TreeCalcTableLifecycleCallbackPacket {
    #[must_use]
    pub fn new(event_kind: TreeCalcTableLifecycleEventKind) -> Self {
        Self {
            event_kind,
            before_state: None,
            after_state: None,
            context_versions: TreeCalcTableLifecycleContextVersions::default(),
            owner_node_ids: Vec::new(),
            source_reference_handles: Vec::new(),
            changed_row_ids: Vec::new(),
            changed_column_ids: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_before(mut self, state: TreeCalcTableLifecycleVersionState) -> Self {
        self.before_state = Some(state);
        self
    }

    #[must_use]
    pub fn with_after(mut self, state: TreeCalcTableLifecycleVersionState) -> Self {
        self.after_state = Some(state);
        self
    }

    #[must_use]
    pub fn with_owner_nodes(
        mut self,
        owner_node_ids: impl IntoIterator<Item = TreeNodeId>,
    ) -> Self {
        self.owner_node_ids = owner_node_ids.into_iter().collect();
        self
    }

    #[must_use]
    pub fn with_source_reference_handles(
        mut self,
        source_reference_handles: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.source_reference_handles = source_reference_handles
            .into_iter()
            .map(Into::into)
            .collect();
        self
    }

    #[must_use]
    pub fn with_changed_rows(
        mut self,
        changed_row_ids: impl IntoIterator<Item = TreeCalcTableRowId>,
    ) -> Self {
        self.changed_row_ids = changed_row_ids.into_iter().collect();
        self
    }

    #[must_use]
    pub fn with_changed_columns(
        mut self,
        changed_column_ids: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.changed_column_ids = changed_column_ids.into_iter().map(Into::into).collect();
        self
    }

    #[must_use]
    pub fn callback_identity(&self) -> String {
        identity_record(
            "treecalc.table_lifecycle.callback.v1",
            [
                ("event", self.event_kind.stable_id().to_string()),
                (
                    "before",
                    self.before_state.as_ref().map_or_else(
                        || "none".to_string(),
                        TreeCalcTableLifecycleVersionState::identity_fragment,
                    ),
                ),
                (
                    "after",
                    self.after_state.as_ref().map_or_else(
                        || "none".to_string(),
                        TreeCalcTableLifecycleVersionState::identity_fragment,
                    ),
                ),
                ("context", self.context_versions.identity_fragment()),
                (
                    "owners",
                    identity_list(self.owner_node_ids.iter().map(ToString::to_string)),
                ),
                (
                    "source_reference_handles",
                    identity_list(self.source_reference_handles.iter().cloned()),
                ),
                (
                    "changed_rows",
                    identity_list(self.changed_row_ids.iter().map(|row| row.0.clone())),
                ),
                (
                    "changed_columns",
                    identity_list(self.changed_column_ids.iter().cloned()),
                ),
            ],
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeCalcTableLifecycleContractDiagnostic {
    MissingBeforeState {
        event_kind: TreeCalcTableLifecycleEventKind,
    },
    MissingAfterState {
        event_kind: TreeCalcTableLifecycleEventKind,
    },
    UnexpectedBeforeState {
        event_kind: TreeCalcTableLifecycleEventKind,
    },
    UnexpectedAfterState {
        event_kind: TreeCalcTableLifecycleEventKind,
    },
    TableNodeChangedAcrossLifecycle {
        before: TreeNodeId,
        after: TreeNodeId,
    },
    TableIdChangedAcrossLifecycle {
        before: String,
        after: String,
    },
    MissingOwnerNode {
        event_kind: TreeCalcTableLifecycleEventKind,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableLifecycleContractReport {
    pub callback_identity: String,
    pub event_kind: TreeCalcTableLifecycleEventKind,
    pub before_state: Option<TreeCalcTableLifecycleVersionState>,
    pub after_state: Option<TreeCalcTableLifecycleVersionState>,
    pub context_versions: TreeCalcTableLifecycleContextVersions,
    pub changed_dependency_kinds: BTreeSet<DependencyDescriptorKind>,
    pub invalidation_reasons: BTreeSet<InvalidationReasonKind>,
    pub prepared_identity_inputs: BTreeSet<TreeCalcTablePreparedIdentityInput>,
    pub invalidation_seeds: Vec<InvalidationSeed>,
    pub source_reference_handles: Vec<String>,
    pub changed_row_ids: Vec<TreeCalcTableRowId>,
    pub changed_column_ids: Vec<String>,
    pub diagnostics: Vec<TreeCalcTableLifecycleContractDiagnostic>,
}

#[must_use]
pub fn classify_treecalc_table_lifecycle_callback(
    packet: &TreeCalcTableLifecycleCallbackPacket,
) -> TreeCalcTableLifecycleContractReport {
    let diagnostics = validate_lifecycle_callback_packet(packet);
    let (
        changed_dependency_kinds,
        invalidation_reasons,
        mut prepared_identity_inputs,
        invalidation_seeds,
    ) = if let Some(scenario) = packet.event_kind.update_scenario() {
        classify_treecalc_table_update_from_lifecycle_states(
            scenario,
            packet.before_state.as_ref(),
            packet.after_state.as_ref(),
            packet.owner_node_ids.clone(),
        )
    } else {
        classify_table_create_lifecycle_impact(packet)
    };
    prepared_identity_inputs.extend(packet.context_versions.prepared_identity_inputs());

    TreeCalcTableLifecycleContractReport {
        callback_identity: packet.callback_identity(),
        event_kind: packet.event_kind,
        before_state: packet.before_state.clone(),
        after_state: packet.after_state.clone(),
        context_versions: packet.context_versions.clone(),
        changed_dependency_kinds,
        invalidation_reasons,
        prepared_identity_inputs,
        invalidation_seeds,
        source_reference_handles: packet.source_reference_handles.clone(),
        changed_row_ids: packet.changed_row_ids.clone(),
        changed_column_ids: packet.changed_column_ids.clone(),
        diagnostics,
    }
}

#[must_use]
pub fn classify_treecalc_table_update(
    scenario: TreeCalcTableUpdateScenarioKind,
    before: Option<&TreeCalcTableNodeProjection>,
    after: Option<&TreeCalcTableNodeProjection>,
    owner_node_ids: impl IntoIterator<Item = TreeNodeId>,
    source_reference_handles: impl IntoIterator<Item = String>,
) -> TreeCalcTableUpdateImpact {
    let mut changed_dependency_kinds = scenario_changed_dependency_kinds(scenario);
    let mut invalidation_reasons = scenario_invalidation_reasons(scenario);
    let mut prepared_identity_inputs = scenario_prepared_identity_inputs(scenario);

    if let (Some(before), Some(after)) = (before, after) {
        add_observed_table_identity_changes(
            before,
            after,
            &mut changed_dependency_kinds,
            &mut invalidation_reasons,
            &mut prepared_identity_inputs,
        );
    }

    if matches!(scenario, TreeCalcTableUpdateScenarioKind::SaveReopen)
        && before
            .zip(after)
            .is_some_and(|(before, after)| table_projection_identities_equal(before, after))
    {
        changed_dependency_kinds.clear();
        invalidation_reasons.clear();
        prepared_identity_inputs.clear();
    }

    let invalidation_seeds = owner_node_ids
        .into_iter()
        .flat_map(|node_id| {
            invalidation_reasons
                .iter()
                .copied()
                .map(move |reason| InvalidationSeed { node_id, reason })
        })
        .collect::<Vec<_>>();

    TreeCalcTableUpdateImpact {
        scenario,
        before_table_context_identity: before
            .map(|projection| projection.table_context_identity.clone()),
        after_table_context_identity: after
            .map(|projection| projection.table_context_identity.clone()),
        source_reference_handles: source_reference_handles.into_iter().collect(),
        changed_dependency_kinds,
        invalidation_reasons,
        prepared_identity_inputs,
        invalidation_seeds,
    }
}

fn classify_treecalc_table_update_from_lifecycle_states(
    scenario: TreeCalcTableUpdateScenarioKind,
    before: Option<&TreeCalcTableLifecycleVersionState>,
    after: Option<&TreeCalcTableLifecycleVersionState>,
    owner_node_ids: impl IntoIterator<Item = TreeNodeId>,
) -> (
    BTreeSet<DependencyDescriptorKind>,
    BTreeSet<InvalidationReasonKind>,
    BTreeSet<TreeCalcTablePreparedIdentityInput>,
    Vec<InvalidationSeed>,
) {
    let mut changed_dependency_kinds = scenario_changed_dependency_kinds(scenario);
    let mut invalidation_reasons = scenario_invalidation_reasons(scenario);
    let mut prepared_identity_inputs = scenario_prepared_identity_inputs(scenario);

    if let (Some(before), Some(after)) = (before, after) {
        add_observed_lifecycle_state_changes(
            before,
            after,
            &mut changed_dependency_kinds,
            &mut invalidation_reasons,
            &mut prepared_identity_inputs,
        );
    }

    if matches!(scenario, TreeCalcTableUpdateScenarioKind::SaveReopen)
        && before
            .zip(after)
            .is_some_and(|(before, after)| lifecycle_version_states_equal(before, after))
    {
        changed_dependency_kinds.clear();
        invalidation_reasons.clear();
        prepared_identity_inputs.clear();
    }

    let invalidation_seeds = owner_node_ids
        .into_iter()
        .flat_map(|node_id| {
            invalidation_reasons
                .iter()
                .copied()
                .map(move |reason| InvalidationSeed { node_id, reason })
        })
        .collect::<Vec<_>>();

    (
        changed_dependency_kinds,
        invalidation_reasons,
        prepared_identity_inputs,
        invalidation_seeds,
    )
}

fn classify_table_create_lifecycle_impact(
    packet: &TreeCalcTableLifecycleCallbackPacket,
) -> (
    BTreeSet<DependencyDescriptorKind>,
    BTreeSet<InvalidationReasonKind>,
    BTreeSet<TreeCalcTablePreparedIdentityInput>,
    Vec<InvalidationSeed>,
) {
    let changed_dependency_kinds = BTreeSet::from([
        DependencyDescriptorKind::StructuredTableIdentity,
        DependencyDescriptorKind::StructuredTableRowMembership,
        DependencyDescriptorKind::StructuredTableRowOrder,
        DependencyDescriptorKind::StructuredTableColumnIdentity,
        DependencyDescriptorKind::StructuredTableHeaderText,
        DependencyDescriptorKind::StructuredTableHeaderRegion,
        DependencyDescriptorKind::StructuredTableDataRegion,
        DependencyDescriptorKind::StructuredTableTotalsRegion,
        DependencyDescriptorKind::StructuredTableCallerContext,
        DependencyDescriptorKind::StructuredTableEnclosingTable,
    ]);
    let invalidation_reasons = BTreeSet::from([
        InvalidationReasonKind::DependencyAdded,
        InvalidationReasonKind::StructuredTableContextChanged,
        InvalidationReasonKind::StructuredTableRowMembershipChanged,
        InvalidationReasonKind::StructuredTableRowOrderChanged,
        InvalidationReasonKind::StructuredTableColumnChanged,
        InvalidationReasonKind::StructuredTableRegionChanged,
        InvalidationReasonKind::StructuredTableCallerContextChanged,
    ]);
    let prepared_identity_inputs = BTreeSet::from([
        TreeCalcTablePreparedIdentityInput::HostNamespaceVersion,
        TreeCalcTablePreparedIdentityInput::StructureContextVersion,
        TreeCalcTablePreparedIdentityInput::TableContextIdentity,
        TreeCalcTablePreparedIdentityInput::CallerContextIdentity,
        TreeCalcTablePreparedIdentityInput::ResolutionRuleVersion,
    ]);
    let invalidation_seeds = packet
        .owner_node_ids
        .iter()
        .copied()
        .flat_map(|node_id| {
            invalidation_reasons
                .iter()
                .copied()
                .map(move |reason| InvalidationSeed { node_id, reason })
        })
        .collect::<Vec<_>>();

    (
        changed_dependency_kinds,
        invalidation_reasons,
        prepared_identity_inputs,
        invalidation_seeds,
    )
}

fn validate_lifecycle_callback_packet(
    packet: &TreeCalcTableLifecycleCallbackPacket,
) -> Vec<TreeCalcTableLifecycleContractDiagnostic> {
    let mut diagnostics = Vec::new();
    match packet.event_kind {
        TreeCalcTableLifecycleEventKind::TableCreate => {
            if packet.before_state.is_some() {
                diagnostics.push(
                    TreeCalcTableLifecycleContractDiagnostic::UnexpectedBeforeState {
                        event_kind: packet.event_kind,
                    },
                );
            }
            if packet.after_state.is_none() {
                diagnostics.push(
                    TreeCalcTableLifecycleContractDiagnostic::MissingAfterState {
                        event_kind: packet.event_kind,
                    },
                );
            }
        }
        TreeCalcTableLifecycleEventKind::TableDelete => {
            if packet.before_state.is_none() {
                diagnostics.push(
                    TreeCalcTableLifecycleContractDiagnostic::MissingBeforeState {
                        event_kind: packet.event_kind,
                    },
                );
            }
            if packet.after_state.is_some() {
                diagnostics.push(
                    TreeCalcTableLifecycleContractDiagnostic::UnexpectedAfterState {
                        event_kind: packet.event_kind,
                    },
                );
            }
        }
        _ => {
            if packet.before_state.is_none() {
                diagnostics.push(
                    TreeCalcTableLifecycleContractDiagnostic::MissingBeforeState {
                        event_kind: packet.event_kind,
                    },
                );
            }
            if packet.after_state.is_none() {
                diagnostics.push(
                    TreeCalcTableLifecycleContractDiagnostic::MissingAfterState {
                        event_kind: packet.event_kind,
                    },
                );
            }
        }
    }

    if let (Some(before), Some(after)) = (&packet.before_state, &packet.after_state) {
        if before.table_node_id != after.table_node_id {
            diagnostics.push(
                TreeCalcTableLifecycleContractDiagnostic::TableNodeChangedAcrossLifecycle {
                    before: before.table_node_id,
                    after: after.table_node_id,
                },
            );
        }
        if before.table_id != after.table_id {
            diagnostics.push(
                TreeCalcTableLifecycleContractDiagnostic::TableIdChangedAcrossLifecycle {
                    before: before.table_id.clone(),
                    after: after.table_id.clone(),
                },
            );
        }
    }

    if packet.owner_node_ids.is_empty() {
        diagnostics.push(TreeCalcTableLifecycleContractDiagnostic::MissingOwnerNode {
            event_kind: packet.event_kind,
        });
    }

    diagnostics
}

#[must_use]
pub fn validate_treecalc_table_reference_after_update(
    table_id: impl Into<String>,
    projection: Option<&TreeCalcTableNodeProjection>,
    reference: &StructuredTableReferenceIntake,
    caller_table_region: Option<&TableCallerRegion>,
) -> Vec<TreeCalcTableUpdateDiagnostic> {
    let table_id = table_id.into();
    let Some(projection) = projection else {
        return vec![TreeCalcTableUpdateDiagnostic::DeletedTable {
            table_id,
            reference_handle: reference.reference_handle.clone(),
        }];
    };
    let mut diagnostics = Vec::new();
    let available_columns = projection
        .table_descriptor
        .columns
        .iter()
        .map(|column| column.column_id.as_str())
        .collect::<BTreeSet<_>>();
    for column_id in &reference.selected_column_ids {
        if !available_columns.contains(column_id.as_str()) {
            diagnostics.push(TreeCalcTableUpdateDiagnostic::MissingColumn {
                table_id: projection.table_id.clone(),
                reference_handle: reference.reference_handle.clone(),
                column_id: column_id.clone(),
            });
        }
    }
    if reference.uses_this_row && caller_table_region.is_none() {
        diagnostics.push(TreeCalcTableUpdateDiagnostic::MissingCallerTableRegion {
            table_id: projection.table_id.clone(),
            reference_handle: reference.reference_handle.clone(),
        });
    }
    if reference
        .selected_regions
        .contains(&StructuredTableRegionSelection::Headers)
        && !projection.table_descriptor.header_row_present
    {
        diagnostics.push(TreeCalcTableUpdateDiagnostic::HeaderRowAbsent {
            table_id: projection.table_id.clone(),
            reference_handle: reference.reference_handle.clone(),
        });
    }
    if reference
        .selected_regions
        .contains(&StructuredTableRegionSelection::Totals)
        && !projection.table_descriptor.totals_row_present
    {
        diagnostics.push(TreeCalcTableUpdateDiagnostic::TotalsRowAbsent {
            table_id: projection.table_id.clone(),
            reference_handle: reference.reference_handle.clone(),
        });
    }
    diagnostics
}

fn scenario_changed_dependency_kinds(
    scenario: TreeCalcTableUpdateScenarioKind,
) -> BTreeSet<DependencyDescriptorKind> {
    use DependencyDescriptorKind as Kind;
    let kinds: &[Kind] = match scenario {
        TreeCalcTableUpdateScenarioKind::BodyCellEdit => &[Kind::StructuredTableDataRegion],
        TreeCalcTableUpdateScenarioKind::BodyFormulaEdit => &[
            Kind::StructuredTableDataRegion,
            Kind::StructuredTableColumnIdentity,
            Kind::StructuredTableIdentity,
        ],
        TreeCalcTableUpdateScenarioKind::RowInsert | TreeCalcTableUpdateScenarioKind::RowDelete => {
            &[
                Kind::StructuredTableRowMembership,
                Kind::StructuredTableRowOrder,
                Kind::StructuredTableDataRegion,
                Kind::StructuredTableTotalsRegion,
                Kind::StructuredTableCallerContext,
                Kind::StructuredTableIdentity,
            ]
        }
        TreeCalcTableUpdateScenarioKind::RowReorder => &[
            Kind::StructuredTableRowOrder,
            Kind::StructuredTableCallerContext,
            Kind::StructuredTableIdentity,
        ],
        TreeCalcTableUpdateScenarioKind::ColumnInsert
        | TreeCalcTableUpdateScenarioKind::ColumnDelete
        | TreeCalcTableUpdateScenarioKind::ColumnReorder
        | TreeCalcTableUpdateScenarioKind::ColumnRename
        | TreeCalcTableUpdateScenarioKind::HeaderTextEdit => &[
            Kind::StructuredTableColumnIdentity,
            Kind::StructuredTableHeaderText,
            Kind::StructuredTableHeaderRegion,
            Kind::StructuredTableDataRegion,
            Kind::StructuredTableTotalsRegion,
            Kind::StructuredTableIdentity,
        ],
        TreeCalcTableUpdateScenarioKind::TotalsRowToggle
        | TreeCalcTableUpdateScenarioKind::TotalsFormulaEdit => &[
            Kind::StructuredTableTotalsRegion,
            Kind::StructuredTableIdentity,
        ],
        TreeCalcTableUpdateScenarioKind::TableRename
        | TreeCalcTableUpdateScenarioKind::StructuralRebind => &[
            Kind::StructuredTableIdentity,
            Kind::StructuredTableEnclosingTable,
        ],
        TreeCalcTableUpdateScenarioKind::TableMove => &[
            Kind::StructuredTableIdentity,
            Kind::StructuredTableHeaderRegion,
            Kind::StructuredTableDataRegion,
            Kind::StructuredTableTotalsRegion,
        ],
        TreeCalcTableUpdateScenarioKind::TableDelete => &[
            Kind::StructuredTableIdentity,
            Kind::StructuredTableRowMembership,
            Kind::StructuredTableRowOrder,
            Kind::StructuredTableColumnIdentity,
            Kind::StructuredTableHeaderText,
            Kind::StructuredTableHeaderRegion,
            Kind::StructuredTableDataRegion,
            Kind::StructuredTableTotalsRegion,
            Kind::StructuredTableCallerContext,
            Kind::StructuredTableEnclosingTable,
        ],
        TreeCalcTableUpdateScenarioKind::SaveReopen => &[],
    };
    kinds.iter().copied().collect()
}

fn scenario_invalidation_reasons(
    scenario: TreeCalcTableUpdateScenarioKind,
) -> BTreeSet<InvalidationReasonKind> {
    use InvalidationReasonKind as Reason;
    let reasons: &[Reason] = match scenario {
        TreeCalcTableUpdateScenarioKind::BodyCellEdit => &[Reason::StructuredTableRegionChanged],
        TreeCalcTableUpdateScenarioKind::BodyFormulaEdit => &[
            Reason::StructuredTableRegionChanged,
            Reason::StructuredTableContextChanged,
        ],
        TreeCalcTableUpdateScenarioKind::RowInsert | TreeCalcTableUpdateScenarioKind::RowDelete => {
            &[
                Reason::StructuredTableRowMembershipChanged,
                Reason::StructuredTableRowOrderChanged,
                Reason::StructuredTableRegionChanged,
                Reason::StructuredTableCallerContextChanged,
                Reason::StructuredTableContextChanged,
            ]
        }
        TreeCalcTableUpdateScenarioKind::RowReorder => &[
            Reason::StructuredTableRowOrderChanged,
            Reason::StructuredTableCallerContextChanged,
            Reason::StructuredTableContextChanged,
        ],
        TreeCalcTableUpdateScenarioKind::ColumnInsert
        | TreeCalcTableUpdateScenarioKind::ColumnDelete
        | TreeCalcTableUpdateScenarioKind::ColumnReorder
        | TreeCalcTableUpdateScenarioKind::ColumnRename
        | TreeCalcTableUpdateScenarioKind::HeaderTextEdit => &[
            Reason::StructuredTableColumnChanged,
            Reason::StructuredTableRegionChanged,
            Reason::StructuredTableContextChanged,
        ],
        TreeCalcTableUpdateScenarioKind::TotalsRowToggle
        | TreeCalcTableUpdateScenarioKind::TotalsFormulaEdit => &[
            Reason::StructuredTableRegionChanged,
            Reason::StructuredTableContextChanged,
        ],
        TreeCalcTableUpdateScenarioKind::TableRename
        | TreeCalcTableUpdateScenarioKind::TableMove
        | TreeCalcTableUpdateScenarioKind::StructuralRebind => &[
            Reason::StructuredTableContextChanged,
            Reason::StructuralRebindRequired,
        ],
        TreeCalcTableUpdateScenarioKind::TableDelete => &[Reason::StructuralRebindRequired],
        TreeCalcTableUpdateScenarioKind::SaveReopen => &[],
    };
    reasons.iter().copied().collect()
}

fn scenario_prepared_identity_inputs(
    scenario: TreeCalcTableUpdateScenarioKind,
) -> BTreeSet<TreeCalcTablePreparedIdentityInput> {
    use TreeCalcTablePreparedIdentityInput as Input;
    let inputs: &[Input] = match scenario {
        TreeCalcTableUpdateScenarioKind::BodyCellEdit => &[],
        TreeCalcTableUpdateScenarioKind::BodyFormulaEdit => {
            &[Input::StructureContextVersion, Input::TableContextIdentity]
        }
        TreeCalcTableUpdateScenarioKind::RowInsert
        | TreeCalcTableUpdateScenarioKind::RowDelete
        | TreeCalcTableUpdateScenarioKind::RowReorder => {
            &[Input::TableContextIdentity, Input::CallerContextIdentity]
        }
        TreeCalcTableUpdateScenarioKind::ColumnInsert
        | TreeCalcTableUpdateScenarioKind::ColumnDelete
        | TreeCalcTableUpdateScenarioKind::ColumnReorder
        | TreeCalcTableUpdateScenarioKind::ColumnRename
        | TreeCalcTableUpdateScenarioKind::HeaderTextEdit
        | TreeCalcTableUpdateScenarioKind::TotalsRowToggle
        | TreeCalcTableUpdateScenarioKind::TotalsFormulaEdit
        | TreeCalcTableUpdateScenarioKind::TableMove => &[Input::TableContextIdentity],
        TreeCalcTableUpdateScenarioKind::TableRename
        | TreeCalcTableUpdateScenarioKind::StructuralRebind
        | TreeCalcTableUpdateScenarioKind::TableDelete => &[
            Input::HostNamespaceVersion,
            Input::TableContextIdentity,
            Input::ResolutionRuleVersion,
        ],
        TreeCalcTableUpdateScenarioKind::SaveReopen => &[],
    };
    inputs.iter().copied().collect()
}

fn add_observed_table_identity_changes(
    before: &TreeCalcTableNodeProjection,
    after: &TreeCalcTableNodeProjection,
    changed_dependency_kinds: &mut BTreeSet<DependencyDescriptorKind>,
    invalidation_reasons: &mut BTreeSet<InvalidationReasonKind>,
    prepared_identity_inputs: &mut BTreeSet<TreeCalcTablePreparedIdentityInput>,
) {
    if before.table_context_identity != after.table_context_identity {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableIdentity);
        invalidation_reasons.insert(InvalidationReasonKind::StructuredTableContextChanged);
        prepared_identity_inputs.insert(TreeCalcTablePreparedIdentityInput::TableContextIdentity);
    }
    if before.row_membership_identity != after.row_membership_identity {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableRowMembership);
        invalidation_reasons.insert(InvalidationReasonKind::StructuredTableRowMembershipChanged);
    }
    if before.row_order_identity != after.row_order_identity {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableRowOrder);
        invalidation_reasons.insert(InvalidationReasonKind::StructuredTableRowOrderChanged);
        prepared_identity_inputs.insert(TreeCalcTablePreparedIdentityInput::CallerContextIdentity);
    }
    if before.column_identity != after.column_identity {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableColumnIdentity);
        invalidation_reasons.insert(InvalidationReasonKind::StructuredTableColumnChanged);
    }
    if before.virtual_anchor_identity != after.virtual_anchor_identity
        || before.table_descriptor.table_range_ref != after.table_descriptor.table_range_ref
    {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableDataRegion);
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableHeaderRegion);
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableTotalsRegion);
        invalidation_reasons.insert(InvalidationReasonKind::StructuredTableRegionChanged);
    }
    if before.table_namespace_identity != after.table_namespace_identity {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableEnclosingTable);
        invalidation_reasons.insert(InvalidationReasonKind::StructuredTableContextChanged);
        prepared_identity_inputs.insert(TreeCalcTablePreparedIdentityInput::HostNamespaceVersion);
    }
    if before.body_metadata_identity != after.body_metadata_identity {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableDataRegion);
        prepared_identity_inputs
            .insert(TreeCalcTablePreparedIdentityInput::StructureContextVersion);
    }
    if before.totals_metadata_identity != after.totals_metadata_identity {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableTotalsRegion);
        prepared_identity_inputs
            .insert(TreeCalcTablePreparedIdentityInput::StructureContextVersion);
    }
}

fn add_observed_lifecycle_state_changes(
    before: &TreeCalcTableLifecycleVersionState,
    after: &TreeCalcTableLifecycleVersionState,
    changed_dependency_kinds: &mut BTreeSet<DependencyDescriptorKind>,
    invalidation_reasons: &mut BTreeSet<InvalidationReasonKind>,
    prepared_identity_inputs: &mut BTreeSet<TreeCalcTablePreparedIdentityInput>,
) {
    if before.table_context_identity != after.table_context_identity {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableIdentity);
        invalidation_reasons.insert(InvalidationReasonKind::StructuredTableContextChanged);
        prepared_identity_inputs.insert(TreeCalcTablePreparedIdentityInput::TableContextIdentity);
    }
    if before.row_membership_identity != after.row_membership_identity
        || before.row_membership_version != after.row_membership_version
    {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableRowMembership);
        invalidation_reasons.insert(InvalidationReasonKind::StructuredTableRowMembershipChanged);
    }
    if before.row_order_identity != after.row_order_identity
        || before.row_order_version != after.row_order_version
    {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableRowOrder);
        invalidation_reasons.insert(InvalidationReasonKind::StructuredTableRowOrderChanged);
        prepared_identity_inputs.insert(TreeCalcTablePreparedIdentityInput::CallerContextIdentity);
    }
    if before.column_identity != after.column_identity
        || before.column_identity_version != after.column_identity_version
    {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableColumnIdentity);
        invalidation_reasons.insert(InvalidationReasonKind::StructuredTableColumnChanged);
    }
    if before.virtual_anchor_identity != after.virtual_anchor_identity {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableDataRegion);
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableHeaderRegion);
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableTotalsRegion);
        invalidation_reasons.insert(InvalidationReasonKind::StructuredTableRegionChanged);
    }
    if before.table_range_ref != after.table_range_ref {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableDataRegion);
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableHeaderRegion);
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableTotalsRegion);
        invalidation_reasons.insert(InvalidationReasonKind::StructuredTableRegionChanged);
    }
    if before.header_region_ref != after.header_region_ref {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableHeaderRegion);
        invalidation_reasons.insert(InvalidationReasonKind::StructuredTableRegionChanged);
    }
    if before.totals_region_ref != after.totals_region_ref {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableTotalsRegion);
        invalidation_reasons.insert(InvalidationReasonKind::StructuredTableRegionChanged);
    }
    if before.column_range_refs != after.column_range_refs {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableDataRegion);
        invalidation_reasons.insert(InvalidationReasonKind::StructuredTableRegionChanged);
    }
    if before.table_namespace_identity != after.table_namespace_identity
        || before.table_namespace_version != after.table_namespace_version
    {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableEnclosingTable);
        invalidation_reasons.insert(InvalidationReasonKind::StructuredTableContextChanged);
        prepared_identity_inputs.insert(TreeCalcTablePreparedIdentityInput::HostNamespaceVersion);
    }
    if before.table_invalidation_identity != after.table_invalidation_identity {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableDataRegion);
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableTotalsRegion);
        prepared_identity_inputs
            .insert(TreeCalcTablePreparedIdentityInput::StructureContextVersion);
    }
}

fn table_projection_identities_equal(
    before: &TreeCalcTableNodeProjection,
    after: &TreeCalcTableNodeProjection,
) -> bool {
    before.table_context_identity == after.table_context_identity
        && before.table_invalidation_identity == after.table_invalidation_identity
        && before.context_packet.table_context_identity
            == after.context_packet.table_context_identity
}

fn lifecycle_version_states_equal(
    before: &TreeCalcTableLifecycleVersionState,
    after: &TreeCalcTableLifecycleVersionState,
) -> bool {
    before.identity_fragment() == after.identity_fragment()
}

impl SparseRangeReader for TreeCalcTableSparseReader {
    fn reader_identity(&self) -> &SparseReaderIdentity {
        &self.identity
    }

    fn declared_extent(&self) -> SparseRangeExtent {
        self.extent
    }

    fn defined_cardinality(&self) -> usize {
        self.defined_cells.len()
    }

    fn defined_iter(&self) -> Box<dyn Iterator<Item = SparseDefinedCell> + '_> {
        self.telemetry.record_defined_iter();
        Box::new(self.defined_cells.iter().map(|(coord, value)| {
            self.telemetry.record_defined_yield();
            SparseDefinedCell {
                coord: *coord,
                value: value.clone(),
            }
        }))
    }

    fn read_at(&self, coord: SparseCellCoord) -> SparseCellRead {
        self.telemetry.record_read_at();
        if !self.extent.contains(coord) {
            return SparseCellRead::Blank;
        }
        self.defined_cells
            .get(&coord)
            .cloned()
            .map_or(SparseCellRead::Blank, SparseCellRead::Defined)
    }

    fn contains(&self, coord: SparseCellCoord) -> bool {
        self.telemetry.record_contains();
        self.extent.contains(coord)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TreeCalcTableSparseRow {
    Headers,
    Data {
        row_id: TreeCalcTableRowId,
        row_offset: u32,
    },
    Totals,
}

impl TreeCalcTableSparseRow {
    fn section(&self) -> TreeCalcTableSparseSection {
        match self {
            Self::Headers => TreeCalcTableSparseSection::Headers,
            Self::Data { .. } => TreeCalcTableSparseSection::Data,
            Self::Totals => TreeCalcTableSparseSection::Totals,
        }
    }

    fn row_id(&self) -> Option<&TreeCalcTableRowId> {
        match self {
            Self::Data { row_id, .. } => Some(row_id),
            Self::Headers | Self::Totals => None,
        }
    }
}

fn resolved_table_id_for_sparse_reference(
    reference: &StructuredTableReferenceIntake,
) -> Option<String> {
    reference
        .effective_table_ref
        .as_ref()
        .map(|table_ref| table_ref.table_id.clone())
        .or_else(|| {
            reference
                .explicit_table_ref
                .as_ref()
                .map(|table_ref| table_ref.table_id.clone())
        })
}

fn selected_columns_for_sparse_reader<'a>(
    table: &'a TableDescriptor,
    reference: &StructuredTableReferenceIntake,
) -> Result<Vec<&'a TableColumnDescriptor>, TreeCalcTableSparseReaderError> {
    let selected = if reference.selected_column_ids.is_empty() {
        table.columns.iter().collect::<Vec<_>>()
    } else {
        let by_id = table
            .columns
            .iter()
            .map(|column| (column.column_id.as_str(), column))
            .collect::<BTreeMap<_, _>>();
        reference
            .selected_column_ids
            .iter()
            .map(|column_id| {
                by_id.get(column_id.as_str()).copied().ok_or_else(|| {
                    TreeCalcTableSparseReaderError::MissingSelectedColumn {
                        column_id: column_id.clone(),
                    }
                })
            })
            .collect::<Result<Vec<_>, _>>()?
    };
    let mut selected = selected;
    selected.sort_by_key(|column| column.ordinal);
    selected.dedup_by(|left, right| left.column_id == right.column_id);
    Ok(selected)
}

fn ensure_contiguous_columns(
    columns: &[&TableColumnDescriptor],
) -> Result<(), TreeCalcTableSparseReaderError> {
    let Some(first) = columns.first() else {
        return Ok(());
    };
    for (index, column) in columns.iter().enumerate() {
        let expected = first
            .ordinal
            .checked_add(
                u32::try_from(index).map_err(|_| TreeCalcTableSparseReaderError::RangeOverflow)?,
            )
            .ok_or(TreeCalcTableSparseReaderError::RangeOverflow)?;
        if column.ordinal != expected {
            return Err(
                TreeCalcTableSparseReaderError::NonContiguousColumnSelection {
                    column_ids: columns
                        .iter()
                        .map(|column| column.column_id.clone())
                        .collect(),
                },
            );
        }
    }
    Ok(())
}

fn selected_rows_for_sparse_reader(
    snapshot: &TreeCalcTableNodeSnapshot,
    table: &TableDescriptor,
    reference: &StructuredTableReferenceIntake,
    caller_table_region: Option<&TableCallerRegion>,
) -> Result<Vec<TreeCalcTableSparseRow>, TreeCalcTableSparseReaderError> {
    if reference.uses_this_row {
        let caller =
            caller_table_region.ok_or(TreeCalcTableSparseReaderError::MissingCallerTableRegion)?;
        if caller.table_id != table.table_id {
            return Err(TreeCalcTableSparseReaderError::CallerTableMismatch {
                caller_table_id: caller.table_id.clone(),
                referenced_table_id: table.table_id.clone(),
            });
        }
        if caller.region_kind != TableRegionKind::Data {
            return Err(TreeCalcTableSparseReaderError::CallerRegionNotData {
                region_kind: caller.region_kind,
            });
        }
        let row_offset = caller
            .data_row_offset
            .ok_or(TreeCalcTableSparseReaderError::CallerDataRowOffsetMissing)?;
        let index = usize::try_from(row_offset)
            .map_err(|_| TreeCalcTableSparseReaderError::RangeOverflow)?;
        let row_id = snapshot.rows.get(index).cloned().ok_or(
            TreeCalcTableSparseReaderError::CallerRowOutOfRange {
                row_offset,
                row_count: snapshot.rows.len(),
            },
        )?;
        return Ok(vec![TreeCalcTableSparseRow::Data { row_id, row_offset }]);
    }

    let mut regions = reference.selected_regions.clone();
    if regions.is_empty() {
        regions.insert(StructuredTableRegionSelection::Data);
    }
    let selects_all = regions.contains(&StructuredTableRegionSelection::All);
    let mut rows = Vec::new();
    if regions.contains(&StructuredTableRegionSelection::Headers)
        || (selects_all && table.header_row_present)
    {
        if !table.header_row_present {
            return Err(TreeCalcTableSparseReaderError::HeaderRowAbsent);
        }
        table_header_row(table)?.ok_or(TreeCalcTableSparseReaderError::MissingHeaderRegion)?;
        rows.push(TreeCalcTableSparseRow::Headers);
    }
    if regions.contains(&StructuredTableRegionSelection::Data) || selects_all {
        for (index, row_id) in snapshot.rows.iter().enumerate() {
            rows.push(TreeCalcTableSparseRow::Data {
                row_id: row_id.clone(),
                row_offset: u32::try_from(index)
                    .map_err(|_| TreeCalcTableSparseReaderError::RangeOverflow)?,
            });
        }
    }
    if regions.contains(&StructuredTableRegionSelection::Totals)
        || (selects_all && table.totals_row_present)
    {
        if !table.totals_row_present {
            return Err(TreeCalcTableSparseReaderError::TotalsRowAbsent);
        }
        table_totals_row(table)?.ok_or(TreeCalcTableSparseReaderError::MissingTotalsRegion)?;
        rows.push(TreeCalcTableSparseRow::Totals);
    }
    Ok(rows)
}

fn parse_non_empty_table_column_range(column: &TableColumnDescriptor) -> Option<LocalA1Range> {
    let range_ref = column.column_range_ref.trim();
    (!range_ref.is_empty())
        .then(|| parse_local_a1_range(range_ref))
        .flatten()
}

fn table_column_absolute_col(
    table: &TableDescriptor,
    column: &TableColumnDescriptor,
) -> Result<u32, TreeCalcTableSparseReaderError> {
    if let Some(range) = parse_non_empty_table_column_range(column) {
        return Ok(range.left_col);
    }

    let table_range = parse_local_a1_range(&table.table_range_ref).ok_or_else(|| {
        TreeCalcTableSparseReaderError::InvalidRegionRange {
            region: TreeCalcTableSparseSection::Data,
            range_ref: table.table_range_ref.clone(),
        }
    })?;
    table_range
        .left_col
        .checked_add(column.ordinal.saturating_sub(1))
        .ok_or(TreeCalcTableSparseReaderError::RangeOverflow)
}

fn table_header_row(
    table: &TableDescriptor,
) -> Result<Option<u32>, TreeCalcTableSparseReaderError> {
    table
        .header_region_ref
        .as_ref()
        .map(|range_ref| {
            parse_local_a1_range(range_ref)
                .map(|range| range.top_row)
                .ok_or_else(|| TreeCalcTableSparseReaderError::InvalidRegionRange {
                    region: TreeCalcTableSparseSection::Headers,
                    range_ref: range_ref.clone(),
                })
        })
        .transpose()
}

fn table_totals_row(
    table: &TableDescriptor,
) -> Result<Option<u32>, TreeCalcTableSparseReaderError> {
    table
        .totals_region_ref
        .as_ref()
        .map(|range_ref| {
            parse_local_a1_range(range_ref)
                .map(|range| range.top_row)
                .ok_or_else(|| TreeCalcTableSparseReaderError::InvalidRegionRange {
                    region: TreeCalcTableSparseSection::Totals,
                    range_ref: range_ref.clone(),
                })
        })
        .transpose()
}

fn reference_like_for_sparse_slots(
    sheet_scope_ref: &str,
    slots: &[TreeCalcTableSparseSlot],
) -> Option<ReferenceLike> {
    let min_row = slots.iter().map(|slot| slot.absolute_row).min()?;
    let max_row = slots.iter().map(|slot| slot.absolute_row).max()?;
    let min_col = slots.iter().map(|slot| slot.absolute_col).min()?;
    let max_col = slots.iter().map(|slot| slot.absolute_col).max()?;
    let local_ref = if min_row == max_row && min_col == max_col {
        a1_cell_ref(min_row, min_col)?
    } else {
        a1_range_ref(min_row, min_col, max_row, max_col).ok()?
    };
    Some(ReferenceLike::new(
        if min_row == max_row && min_col == max_col {
            ReferenceKind::A1
        } else {
            ReferenceKind::Area
        },
        qualified_local_reference_target(sheet_scope_ref, &local_ref),
    ))
}

fn reference_like_for_empty_table_selection(
    table: &TableDescriptor,
    reference: &StructuredTableReferenceIntake,
    selected_columns: &[&TableColumnDescriptor],
) -> ReferenceLike {
    let section = if reference.uses_this_row {
        "ThisRow"
    } else if reference
        .selected_regions
        .contains(&StructuredTableRegionSelection::All)
    {
        "All"
    } else if reference
        .selected_regions
        .contains(&StructuredTableRegionSelection::Headers)
    {
        "Headers"
    } else if reference
        .selected_regions
        .contains(&StructuredTableRegionSelection::Totals)
    {
        "Totals"
    } else {
        "Data"
    };
    ReferenceLike::new(
        ReferenceKind::Structured,
        format!(
            "empty-structured:{}:{}:{}:{}",
            table.sheet_scope_ref,
            section,
            selected_columns
                .iter()
                .map(|column| column.column_id.as_str())
                .collect::<Vec<_>>()
                .join("|"),
            selected_columns.len()
        ),
    )
}

pub fn reference_like_from_structured_resolved_ref(
    resolved: &StructuredResolvedRef,
) -> ReferenceLike {
    match resolved {
        StructuredResolvedRef::Cell(cell) => {
            ReferenceLike::new(ReferenceKind::A1, reference_target_for_cell_ref(cell))
        }
        StructuredResolvedRef::Area(area) => {
            ReferenceLike::new(ReferenceKind::Area, reference_target_for_area_ref(area))
        }
        StructuredResolvedRef::EmptyArea(empty) => ReferenceLike::new(
            ReferenceKind::Structured,
            format!(
                "empty-structured:{}:{}:{}:{}",
                empty.sheet_id,
                match empty.section_kind {
                    StructuredSectionKind::All => "All",
                    StructuredSectionKind::Data => "Data",
                    StructuredSectionKind::Headers => "Headers",
                    StructuredSectionKind::Totals => "Totals",
                    StructuredSectionKind::ThisRow => "ThisRow",
                },
                empty.selected_column_ids.join("|"),
                empty.column_count
            ),
        ),
    }
}

fn reference_target_for_cell_ref(cell: &CellRef) -> String {
    let local_ref = a1_cell_ref(cell.coord.row, cell.coord.col).unwrap_or_else(|| "#REF!".into());
    qualified_local_reference_target(&cell.sheet_id, &local_ref)
}

fn reference_target_for_area_ref(area: &AreaRef) -> String {
    let bottom_row = area
        .top_left
        .row
        .saturating_add(area.height.saturating_sub(1));
    let right_col = area
        .top_left
        .col
        .saturating_add(area.width.saturating_sub(1));
    let local_ref = a1_range_ref(area.top_left.row, area.top_left.col, bottom_row, right_col)
        .unwrap_or_else(|_| "#REF!".into());
    qualified_local_reference_target(&area.sheet_id, &local_ref)
}

fn qualified_local_reference_target(sheet_scope_ref: &str, local_ref: &str) -> String {
    if sheet_scope_ref.is_empty() || sheet_scope_ref.starts_with("sheet:") {
        local_ref.to_string()
    } else {
        format!("{sheet_scope_ref}!{local_ref}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LocalA1Range {
    top_row: u32,
    left_col: u32,
    bottom_row: u32,
    right_col: u32,
}

fn parse_local_a1_range(range_ref: &str) -> Option<LocalA1Range> {
    let local = range_ref
        .rsplit_once('!')
        .map_or(range_ref, |(_, local)| local);
    let (start, end) = local.split_once(':').unwrap_or((local, local));
    let start = parse_local_a1_cell(start)?;
    let end = parse_local_a1_cell(end)?;
    Some(LocalA1Range {
        top_row: start.0.min(end.0),
        left_col: start.1.min(end.1),
        bottom_row: start.0.max(end.0),
        right_col: start.1.max(end.1),
    })
}

fn parse_local_a1_cell(cell_ref: &str) -> Option<(u32, u32)> {
    let col_len = cell_ref
        .chars()
        .take_while(|ch| ch.is_ascii_alphabetic())
        .count();
    if col_len == 0 || col_len == cell_ref.len() {
        return None;
    }
    let (col_text, row_text) = cell_ref.split_at(col_len);
    let row = row_text.parse::<u32>().ok()?;
    let col = excel_column_number(col_text)?;
    Some((row, col))
}

fn excel_column_number(text: &str) -> Option<u32> {
    let mut value = 0u32;
    for ch in text.chars() {
        if !ch.is_ascii_alphabetic() {
            return None;
        }
        let upper = ch.to_ascii_uppercase() as u32;
        value = value.checked_mul(26)?;
        value = value.checked_add(upper.checked_sub(u32::from(b'A'))? + 1)?;
    }
    Some(value)
}

fn a1_cell_ref(row: u32, col: u32) -> Option<String> {
    Some(format!("{}{}", excel_column_name(col).ok()?, row))
}

fn table_eval_value_to_array_cell(value: EvalValue) -> ArrayCellValue {
    match value {
        EvalValue::Number(value) => ArrayCellValue::Number(value),
        EvalValue::Text(value) => ArrayCellValue::Text(value),
        EvalValue::Logical(value) => ArrayCellValue::Logical(value),
        EvalValue::Error(value) => ArrayCellValue::Error(value),
        EvalValue::Array(_) | EvalValue::Reference(_) | EvalValue::Lambda(_) => {
            ArrayCellValue::Error(WorksheetErrorCode::Value)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StructuredTableRegionSelection {
    All,
    Headers,
    Data,
    Totals,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredTableReferenceIntake {
    pub reference_handle: String,
    pub source_token_kind: StructuredReferenceSourceTokenKind,
    pub effective_table_ref: Option<TableRef>,
    pub explicit_table_ref: Option<TableRef>,
    pub uses_omitted_table_name: bool,
    pub selected_column_ids: Vec<String>,
    pub selected_regions: BTreeSet<StructuredTableRegionSelection>,
    pub uses_this_row: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructuredTableBindRecordIntakeError {
    UnresolvedStructuredReference {
        bind_record_handle: String,
        source_token_text: String,
        diagnostic_codes: Vec<String>,
    },
    MissingEffectiveTableId {
        bind_record_handle: String,
        source_token_text: String,
    },
}

impl StructuredTableReferenceIntake {
    #[must_use]
    pub fn explicit_table(
        reference_handle: impl Into<String>,
        table_id: impl Into<String>,
    ) -> Self {
        let table_id = table_id.into();
        Self {
            reference_handle: reference_handle.into(),
            source_token_kind: StructuredReferenceSourceTokenKind::StructuredReference,
            explicit_table_ref: Some(TableRef {
                table_id: table_id.clone(),
            }),
            effective_table_ref: Some(TableRef { table_id }),
            uses_omitted_table_name: false,
            selected_column_ids: Vec::new(),
            selected_regions: BTreeSet::new(),
            uses_this_row: false,
        }
    }

    #[must_use]
    pub fn omitted_table_name(reference_handle: impl Into<String>) -> Self {
        Self {
            reference_handle: reference_handle.into(),
            source_token_kind: StructuredReferenceSourceTokenKind::StructuredReference,
            effective_table_ref: None,
            explicit_table_ref: None,
            uses_omitted_table_name: true,
            selected_column_ids: Vec::new(),
            selected_regions: BTreeSet::new(),
            uses_this_row: false,
        }
    }

    #[must_use]
    pub fn with_selected_columns(mut self, column_ids: impl IntoIterator<Item = String>) -> Self {
        self.selected_column_ids = column_ids.into_iter().collect();
        self
    }

    #[must_use]
    pub fn with_selected_regions(
        mut self,
        regions: impl IntoIterator<Item = StructuredTableRegionSelection>,
    ) -> Self {
        self.selected_regions = regions.into_iter().collect();
        self
    }

    #[must_use]
    pub fn with_this_row(mut self) -> Self {
        self.uses_this_row = true;
        self
    }

    pub fn from_oxfml_bind_record(
        record: &StructuredReferenceBindRecord,
    ) -> Result<Self, StructuredTableBindRecordIntakeError> {
        if !record.diagnostics.is_empty() {
            return Err(
                StructuredTableBindRecordIntakeError::UnresolvedStructuredReference {
                    bind_record_handle: record.bind_record_handle.clone(),
                    source_token_text: record.source_token_text.clone(),
                    diagnostic_codes: record
                        .diagnostics
                        .iter()
                        .map(|diagnostic| diagnostic.diagnostic_code.clone())
                        .collect(),
                },
            );
        }

        let Some(effective_table_id) = record.effective_table_id.clone() else {
            return Err(
                StructuredTableBindRecordIntakeError::MissingEffectiveTableId {
                    bind_record_handle: record.bind_record_handle.clone(),
                    source_token_text: record.source_token_text.clone(),
                },
            );
        };

        let selected_regions = structured_table_regions_from_oxfml_record(record);

        Ok(Self {
            reference_handle: record.bind_record_handle.clone(),
            source_token_kind: record.source_token_kind,
            effective_table_ref: Some(TableRef {
                table_id: effective_table_id.clone(),
            }),
            explicit_table_ref: (!record.omitted_table_name).then_some(TableRef {
                table_id: effective_table_id,
            }),
            uses_omitted_table_name: record.omitted_table_name,
            selected_column_ids: record.selected_column_ids.clone(),
            selected_regions,
            uses_this_row: record.uses_this_row
                || record
                    .selected_sections
                    .contains(&StructuredSectionKind::ThisRow),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredTableDependencyLoweringRequest {
    pub owner_node_id: TreeNodeId,
    pub source_reference_handle: Option<String>,
    pub context_packet: StructuredTableContextPacket,
    pub reference: StructuredTableReferenceIntake,
}

impl StructuredTableDependencyLoweringRequest {
    pub fn from_oxfml_bind_record(
        owner_node_id: TreeNodeId,
        context_packet: StructuredTableContextPacket,
        record: &StructuredReferenceBindRecord,
    ) -> Result<Self, StructuredTableBindRecordIntakeError> {
        let reference = StructuredTableReferenceIntake::from_oxfml_bind_record(record)?;
        Ok(Self {
            owner_node_id,
            source_reference_handle: Some(record.bind_record_handle.clone()),
            context_packet,
            reference,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StructuredTableDependencyFactKind {
    TableIdentity,
    RowMembership,
    RowOrder,
    ColumnIdentity,
    HeaderText,
    HeaderRegion,
    DataRegion,
    TotalsRegion,
    CallerRowContext,
    OmittedTableNameEnclosingTable,
}

impl StructuredTableDependencyFactKind {
    #[must_use]
    pub fn descriptor_kind(self) -> DependencyDescriptorKind {
        match self {
            Self::TableIdentity => DependencyDescriptorKind::StructuredTableIdentity,
            Self::RowMembership => DependencyDescriptorKind::StructuredTableRowMembership,
            Self::RowOrder => DependencyDescriptorKind::StructuredTableRowOrder,
            Self::ColumnIdentity => DependencyDescriptorKind::StructuredTableColumnIdentity,
            Self::HeaderText => DependencyDescriptorKind::StructuredTableHeaderText,
            Self::HeaderRegion => DependencyDescriptorKind::StructuredTableHeaderRegion,
            Self::DataRegion => DependencyDescriptorKind::StructuredTableDataRegion,
            Self::TotalsRegion => DependencyDescriptorKind::StructuredTableTotalsRegion,
            Self::CallerRowContext => DependencyDescriptorKind::StructuredTableCallerContext,
            Self::OmittedTableNameEnclosingTable => {
                DependencyDescriptorKind::StructuredTableEnclosingTable
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredTableDependencyFactStatus {
    Lowered,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StructuredTableLoweringBlocker {
    MissingTableCatalogEntry,
    MissingEnclosingTableContext,
    MissingStableRowMembershipAndOrderPacket,
    MissingSelectedColumn,
    MissingHeaderRegionRange,
    MissingTotalsRegionRange,
    HeaderRowAbsent,
    TotalsRowAbsent,
    MissingCallerTableRegion,
    CallerTableMismatch,
    CallerRegionNotData,
    CallerDataRowOffsetMissing,
    OmittedTableEnclosingMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredTableDependencyFact {
    pub fact_id: String,
    pub kind: StructuredTableDependencyFactKind,
    pub status: StructuredTableDependencyFactStatus,
    pub table_id: Option<String>,
    pub column_id: Option<String>,
    pub identity: Option<String>,
    pub blocker: Option<StructuredTableLoweringBlocker>,
    pub detail: String,
}

impl StructuredTableDependencyFact {
    fn lowered(
        fact_id: String,
        kind: StructuredTableDependencyFactKind,
        table_id: impl Into<String>,
        column_id: Option<String>,
        identity: String,
        detail: String,
    ) -> Self {
        Self {
            fact_id,
            kind,
            status: StructuredTableDependencyFactStatus::Lowered,
            table_id: Some(table_id.into()),
            column_id,
            identity: Some(identity),
            blocker: None,
            detail,
        }
    }

    fn blocked(
        fact_id: String,
        kind: StructuredTableDependencyFactKind,
        table_id: Option<String>,
        column_id: Option<String>,
        blocker: StructuredTableLoweringBlocker,
        detail: String,
    ) -> Self {
        Self {
            fact_id,
            kind,
            status: StructuredTableDependencyFactStatus::Blocked,
            table_id,
            column_id,
            identity: None,
            blocker: Some(blocker),
            detail,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredTableDependencyLowering {
    pub table_context_identity: String,
    pub facts: Vec<StructuredTableDependencyFact>,
    pub descriptors: Vec<DependencyDescriptor>,
}

impl StructuredTableDependencyLowering {
    #[must_use]
    pub fn blocked_facts(&self) -> Vec<&StructuredTableDependencyFact> {
        self.facts
            .iter()
            .filter(|fact| fact.status == StructuredTableDependencyFactStatus::Blocked)
            .collect()
    }
}

#[must_use]
pub fn lower_structured_table_dependencies(
    request: &StructuredTableDependencyLoweringRequest,
) -> StructuredTableDependencyLowering {
    let mut facts = Vec::new();
    let tables = request.context_packet.table_by_id();
    let Some(table_id) = resolved_table_id(request) else {
        facts.push(StructuredTableDependencyFact::blocked(
            fact_id(request, "table", "unresolved"),
            StructuredTableDependencyFactKind::TableIdentity,
            None,
            None,
            StructuredTableLoweringBlocker::MissingEnclosingTableContext,
            "omitted table name requires enclosing_table_ref in the OxFml table packet".to_string(),
        ));
        return lowering_from_facts(request, facts);
    };

    let Some(table) = tables.get(table_id.as_str()).copied() else {
        facts.push(StructuredTableDependencyFact::blocked(
            fact_id(request, "table", &table_id),
            StructuredTableDependencyFactKind::TableIdentity,
            Some(table_id),
            None,
            StructuredTableLoweringBlocker::MissingTableCatalogEntry,
            "referenced table_id is absent from table_catalog".to_string(),
        ));
        return lowering_from_facts(request, facts);
    };

    push_table_identity(request, table, &mut facts);
    push_row_membership_and_order_facts(request, table, &mut facts);
    push_column_facts(request, table, &mut facts);
    push_region_facts(request, table, &mut facts);
    push_caller_context_fact(request, table, &mut facts);
    push_enclosing_table_fact(request, table, &mut facts);

    lowering_from_facts(request, facts)
}

fn resolved_table_id(request: &StructuredTableDependencyLoweringRequest) -> Option<String> {
    request
        .reference
        .effective_table_ref
        .as_ref()
        .map(|table_ref| table_ref.table_id.clone())
        .or_else(|| {
            request
                .reference
                .explicit_table_ref
                .as_ref()
                .map(|table_ref| table_ref.table_id.clone())
        })
        .or_else(|| {
            request
                .reference
                .uses_omitted_table_name
                .then(|| {
                    request
                        .context_packet
                        .enclosing_table_ref
                        .as_ref()
                        .map(|table_ref| table_ref.table_id.clone())
                })
                .flatten()
        })
}

fn push_table_identity(
    request: &StructuredTableDependencyLoweringRequest,
    table: &TableDescriptor,
    facts: &mut Vec<StructuredTableDependencyFact>,
) {
    facts.push(StructuredTableDependencyFact::lowered(
        fact_id(request, "table_identity", &table.table_id),
        StructuredTableDependencyFactKind::TableIdentity,
        table.table_id.clone(),
        None,
        format!(
            "table_identity:v1:id={};name={};workbook={};sheet={};range={}",
            table.table_id,
            table.table_name,
            table.workbook_scope_ref,
            table.sheet_scope_ref,
            table.table_range_ref
        ),
        "table identity, scope, and range are supplied by table_catalog".to_string(),
    ));
}

fn push_row_membership_and_order_facts(
    request: &StructuredTableDependencyLoweringRequest,
    table: &TableDescriptor,
    facts: &mut Vec<StructuredTableDependencyFact>,
) {
    for (suffix, kind, identity) in [
        (
            "row_membership",
            StructuredTableDependencyFactKind::RowMembership,
            table.row_membership_identity.as_ref(),
        ),
        (
            "row_order",
            StructuredTableDependencyFactKind::RowOrder,
            table.row_order_identity.as_ref(),
        ),
    ] {
        if let Some(identity) = identity {
            facts.push(StructuredTableDependencyFact::lowered(
                fact_id(request, suffix, identity),
                kind,
                table.table_id.clone(),
                None,
                format!(
                    "table_{suffix}:v1:table={};identity={identity}",
                    table.table_id
                ),
                format!("stable {suffix} identity is supplied by the OxFml TableDescriptor"),
            ));
        } else {
            facts.push(StructuredTableDependencyFact::blocked(
                fact_id(request, suffix, &table.table_id),
                kind,
                Some(table.table_id.clone()),
                None,
                StructuredTableLoweringBlocker::MissingStableRowMembershipAndOrderPacket,
                format!(
                    "current OxFml TableDescriptor supplies table_range_ref={} but no stable {suffix} identity",
                    table.table_range_ref
                ),
            ));
        }
    }
}

fn push_column_facts(
    request: &StructuredTableDependencyLoweringRequest,
    table: &TableDescriptor,
    facts: &mut Vec<StructuredTableDependencyFact>,
) {
    let columns_by_id = table
        .columns
        .iter()
        .map(|column| (column.column_id.as_str(), column))
        .collect::<BTreeMap<_, _>>();

    for column_id in &request.reference.selected_column_ids {
        let Some(column) = columns_by_id.get(column_id.as_str()).copied() else {
            facts.push(StructuredTableDependencyFact::blocked(
                fact_id(request, "column", column_id),
                StructuredTableDependencyFactKind::ColumnIdentity,
                Some(table.table_id.clone()),
                Some(column_id.clone()),
                StructuredTableLoweringBlocker::MissingSelectedColumn,
                "selected column_id is absent from the table catalog entry".to_string(),
            ));
            continue;
        };
        facts.push(StructuredTableDependencyFact::lowered(
            fact_id(request, "column", &column.column_id),
            StructuredTableDependencyFactKind::ColumnIdentity,
            table.table_id.clone(),
            Some(column.column_id.clone()),
            format!(
                "table_column_identity:v1:table={};column={};ordinal={};name={};range={}",
                table.table_id,
                column.column_id,
                column.ordinal,
                column.column_name,
                column.column_range_ref
            ),
            "column id, text, ordinal, and data range are supplied by table_catalog".to_string(),
        ));
    }
}

fn push_region_facts(
    request: &StructuredTableDependencyLoweringRequest,
    table: &TableDescriptor,
    facts: &mut Vec<StructuredTableDependencyFact>,
) {
    let selects_all = request
        .reference
        .selected_regions
        .contains(&StructuredTableRegionSelection::All);
    let selects_headers = request
        .reference
        .selected_regions
        .contains(&StructuredTableRegionSelection::Headers);
    let selects_data = request
        .reference
        .selected_regions
        .contains(&StructuredTableRegionSelection::Data);
    let selects_totals = request
        .reference
        .selected_regions
        .contains(&StructuredTableRegionSelection::Totals);

    if selects_headers || (selects_all && table.header_row_present) {
        if table.header_row_present {
            for column in selected_columns_or_all(request, table) {
                facts.push(StructuredTableDependencyFact::lowered(
                    fact_id(request, "header_text", &column.column_id),
                    StructuredTableDependencyFactKind::HeaderText,
                    table.table_id.clone(),
                    Some(column.column_id.clone()),
                    format!(
                        "table_header_text:v1:table={};column={};text={}",
                        table.table_id, column.column_id, column.column_name
                    ),
                    "header text is supplied as TableColumnDescriptor.column_name".to_string(),
                ));
            }
            if let Some(header_region_ref) = table.header_region_ref.as_ref() {
                facts.push(StructuredTableDependencyFact::lowered(
                    fact_id(request, "header_region", header_region_ref),
                    StructuredTableDependencyFactKind::HeaderRegion,
                    table.table_id.clone(),
                    None,
                    format!(
                        "table_header_region:v1:table={};region={header_region_ref}",
                        table.table_id
                    ),
                    "exact header row region identity is supplied by the OxFml TableDescriptor"
                        .to_string(),
                ));
            } else {
                facts.push(StructuredTableDependencyFact::blocked(
                    fact_id(request, "header_region", &table.table_id),
                    StructuredTableDependencyFactKind::HeaderRegion,
                    Some(table.table_id.clone()),
                    None,
                    StructuredTableLoweringBlocker::MissingHeaderRegionRange,
                    "current table packet has header presence/text but no header row region identity"
                        .to_string(),
                ));
            }
        } else {
            facts.push(StructuredTableDependencyFact::blocked(
                fact_id(request, "header_region", &table.table_id),
                StructuredTableDependencyFactKind::HeaderRegion,
                Some(table.table_id.clone()),
                None,
                StructuredTableLoweringBlocker::HeaderRowAbsent,
                "structured reference selected headers but table declares no header row"
                    .to_string(),
            ));
        }
    }

    if selects_data || selects_all {
        let ranges = selected_columns_or_all(request, table)
            .into_iter()
            .map(|column| format!("{}={}", column.column_id, column.column_range_ref))
            .collect::<Vec<_>>()
            .join(",");
        facts.push(StructuredTableDependencyFact::lowered(
            fact_id(request, "data_region", &table.table_id),
            StructuredTableDependencyFactKind::DataRegion,
            table.table_id.clone(),
            None,
            format!(
                "table_data_region:v1:table={};columns={ranges}",
                table.table_id
            ),
            "data region identity is preserved as supplied column_range_ref values".to_string(),
        ));
    }

    if selects_totals || (selects_all && table.totals_row_present) {
        if !table.totals_row_present {
            facts.push(StructuredTableDependencyFact::blocked(
                fact_id(request, "totals_region", &table.table_id),
                StructuredTableDependencyFactKind::TotalsRegion,
                Some(table.table_id.clone()),
                None,
                StructuredTableLoweringBlocker::TotalsRowAbsent,
                "structured reference selected totals but table declares no totals row".to_string(),
            ));
        } else if let Some(totals_region_ref) = table.totals_region_ref.as_ref() {
            facts.push(StructuredTableDependencyFact::lowered(
                fact_id(request, "totals_region", totals_region_ref),
                StructuredTableDependencyFactKind::TotalsRegion,
                table.table_id.clone(),
                None,
                format!(
                    "table_totals_region:v1:table={};region={totals_region_ref}",
                    table.table_id
                ),
                "exact totals row region identity is supplied by the OxFml TableDescriptor"
                    .to_string(),
            ));
        } else {
            facts.push(StructuredTableDependencyFact::blocked(
                fact_id(request, "totals_region", &table.table_id),
                StructuredTableDependencyFactKind::TotalsRegion,
                Some(table.table_id.clone()),
                None,
                StructuredTableLoweringBlocker::MissingTotalsRegionRange,
                "current table packet has totals presence but no totals row region identity"
                    .to_string(),
            ));
        }
    }
}

fn push_caller_context_fact(
    request: &StructuredTableDependencyLoweringRequest,
    table: &TableDescriptor,
    facts: &mut Vec<StructuredTableDependencyFact>,
) {
    if !request.reference.uses_this_row {
        return;
    }

    let Some(caller_region) = &request.context_packet.caller_table_region else {
        facts.push(StructuredTableDependencyFact::blocked(
            fact_id(request, "caller_context", &table.table_id),
            StructuredTableDependencyFactKind::CallerRowContext,
            Some(table.table_id.clone()),
            None,
            StructuredTableLoweringBlocker::MissingCallerTableRegion,
            "#This Row requires caller_table_region in the OxFml table packet".to_string(),
        ));
        return;
    };

    if caller_region.table_id != table.table_id {
        facts.push(StructuredTableDependencyFact::blocked(
            fact_id(request, "caller_context", &table.table_id),
            StructuredTableDependencyFactKind::CallerRowContext,
            Some(table.table_id.clone()),
            None,
            StructuredTableLoweringBlocker::CallerTableMismatch,
            format!(
                "caller_table_region table_id={} does not match referenced table_id={}",
                caller_region.table_id, table.table_id
            ),
        ));
        return;
    }

    if caller_region.region_kind != TableRegionKind::Data {
        facts.push(StructuredTableDependencyFact::blocked(
            fact_id(request, "caller_context", &table.table_id),
            StructuredTableDependencyFactKind::CallerRowContext,
            Some(table.table_id.clone()),
            None,
            StructuredTableLoweringBlocker::CallerRegionNotData,
            format!(
                "#This Row requires data caller region, got {:?}",
                caller_region.region_kind
            ),
        ));
        return;
    }

    let Some(row_offset) = caller_region.data_row_offset else {
        facts.push(StructuredTableDependencyFact::blocked(
            fact_id(request, "caller_context", &table.table_id),
            StructuredTableDependencyFactKind::CallerRowContext,
            Some(table.table_id.clone()),
            None,
            StructuredTableLoweringBlocker::CallerDataRowOffsetMissing,
            "#This Row requires caller data_row_offset".to_string(),
        ));
        return;
    };

    facts.push(StructuredTableDependencyFact::lowered(
        fact_id(request, "caller_context", &table.table_id),
        StructuredTableDependencyFactKind::CallerRowContext,
        table.table_id.clone(),
        None,
        format!(
            "table_caller_context:v1:table={};region=data;row_offset={row_offset}",
            table.table_id
        ),
        "caller row context is supplied by caller_table_region".to_string(),
    ));
}

fn push_enclosing_table_fact(
    request: &StructuredTableDependencyLoweringRequest,
    table: &TableDescriptor,
    facts: &mut Vec<StructuredTableDependencyFact>,
) {
    if !request.reference.uses_omitted_table_name {
        return;
    }

    let Some(enclosing_table_ref) = &request.context_packet.enclosing_table_ref else {
        facts.push(StructuredTableDependencyFact::blocked(
            fact_id(request, "enclosing_table", &table.table_id),
            StructuredTableDependencyFactKind::OmittedTableNameEnclosingTable,
            Some(table.table_id.clone()),
            None,
            StructuredTableLoweringBlocker::MissingEnclosingTableContext,
            "omitted table name requires enclosing_table_ref in the OxFml table packet".to_string(),
        ));
        return;
    };

    if let Some(effective_table_ref) = request.reference.effective_table_ref.as_ref()
        && enclosing_table_ref.table_id != effective_table_ref.table_id
    {
        facts.push(StructuredTableDependencyFact::blocked(
            fact_id(request, "enclosing_table", &table.table_id),
            StructuredTableDependencyFactKind::OmittedTableNameEnclosingTable,
            Some(table.table_id.clone()),
            None,
            StructuredTableLoweringBlocker::OmittedTableEnclosingMismatch,
            format!(
                "omitted table-name bind record resolved table_id={} but enclosing_table_ref table_id={}",
                effective_table_ref.table_id, enclosing_table_ref.table_id
            ),
        ));
        return;
    }

    facts.push(StructuredTableDependencyFact::lowered(
        fact_id(request, "enclosing_table", &table.table_id),
        StructuredTableDependencyFactKind::OmittedTableNameEnclosingTable,
        table.table_id.clone(),
        None,
        format!(
            "table_enclosing_context:v1:table={};enclosing={}",
            table.table_id, enclosing_table_ref.table_id
        ),
        "omitted table-name dependency is supplied by enclosing_table_ref".to_string(),
    ));
}

fn selected_columns_or_all<'a>(
    request: &StructuredTableDependencyLoweringRequest,
    table: &'a TableDescriptor,
) -> Vec<&'a oxfml_core::interface::TableColumnDescriptor> {
    if request.reference.selected_column_ids.is_empty() {
        return table.columns.iter().collect();
    }

    let selected = request
        .reference
        .selected_column_ids
        .iter()
        .collect::<BTreeSet<_>>();
    table
        .columns
        .iter()
        .filter(|column| selected.contains(&column.column_id))
        .collect()
}

fn structured_table_regions_from_oxfml_sections(
    sections: &[StructuredSectionKind],
    caller_row_sensitive: bool,
) -> BTreeSet<StructuredTableRegionSelection> {
    let mut regions = BTreeSet::new();

    if sections.is_empty() {
        regions.insert(StructuredTableRegionSelection::Data);
    }

    for section in sections {
        match section {
            StructuredSectionKind::All => {
                regions.insert(StructuredTableRegionSelection::All);
            }
            StructuredSectionKind::Data => {
                regions.insert(StructuredTableRegionSelection::Data);
            }
            StructuredSectionKind::Headers => {
                regions.insert(StructuredTableRegionSelection::Headers);
            }
            StructuredSectionKind::Totals => {
                regions.insert(StructuredTableRegionSelection::Totals);
            }
            StructuredSectionKind::ThisRow => {
                regions.insert(StructuredTableRegionSelection::Data);
            }
        }
    }

    if caller_row_sensitive {
        regions.insert(StructuredTableRegionSelection::Data);
    }

    regions
}

fn structured_table_regions_from_oxfml_record(
    record: &StructuredReferenceBindRecord,
) -> BTreeSet<StructuredTableRegionSelection> {
    let mut sections = record.selected_sections.clone();
    for selected_region in &record.selected_regions {
        if !sections.contains(&selected_region.section_kind) {
            sections.push(selected_region.section_kind);
        }
    }

    structured_table_regions_from_oxfml_sections(
        &sections,
        record.uses_this_row || record.caller_context_dependent,
    )
}

fn lowering_from_facts(
    request: &StructuredTableDependencyLoweringRequest,
    facts: Vec<StructuredTableDependencyFact>,
) -> StructuredTableDependencyLowering {
    let mut descriptors = facts
        .iter()
        .filter(|fact| fact.status == StructuredTableDependencyFactStatus::Lowered)
        .map(|fact| DependencyDescriptor {
            descriptor_id: format!("{}:descriptor", fact.fact_id),
            source_reference_handle: request
                .source_reference_handle
                .clone()
                .or_else(|| Some(request.reference.reference_handle.clone())),
            owner_node_id: request.owner_node_id,
            target_node_id: None,
            workspace_target: None,
            kind: fact.kind.descriptor_kind(),
            carrier_detail: fact.identity.clone().unwrap_or_else(|| fact.detail.clone()),
            tree_reference_collection: None,
            requires_rebind_on_structural_change: true,
        })
        .collect::<Vec<_>>();
    descriptors.sort_by(|left, right| left.descriptor_id.cmp(&right.descriptor_id));

    StructuredTableDependencyLowering {
        table_context_identity: request.context_packet.table_context_identity.clone(),
        facts,
        descriptors,
    }
}

fn fact_id(
    request: &StructuredTableDependencyLoweringRequest,
    suffix: &str,
    identity: &str,
) -> String {
    format!(
        "bind:node:{}:table_ref:{}:{suffix}:{}",
        request.owner_node_id.0,
        sanitize_identifier(&request.reference.reference_handle),
        sanitize_identifier(identity)
    )
}

fn sanitize_identifier(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn table_context_identity(
    table_catalog: &[TableDescriptor],
    enclosing_table_ref: &Option<TableRef>,
    caller_table_region: &Option<TableCallerRegion>,
) -> String {
    let table_parts = identity_list(table_catalog.iter().map(|table| {
        let columns = identity_list(
            table
                .columns
                .iter()
                .map(|column| {
                    identity_record(
                        "table_column",
                        [
                            ("column_id", column.column_id.clone()),
                            ("ordinal", column.ordinal.to_string()),
                            ("column_name", column.column_name.clone()),
                            ("column_range_ref", column.column_range_ref.clone()),
                        ],
                    )
                })
                .collect::<Vec<_>>(),
        );
        identity_record(
            "table_descriptor",
            [
                ("table_id", table.table_id.clone()),
                ("table_name", table.table_name.clone()),
                ("workbook_scope_ref", table.workbook_scope_ref.clone()),
                ("sheet_scope_ref", table.sheet_scope_ref.clone()),
                ("table_range_ref", table.table_range_ref.clone()),
                (
                    "row_membership_identity",
                    table
                        .row_membership_identity
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
                ),
                (
                    "row_order_identity",
                    table
                        .row_order_identity
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
                ),
                (
                    "header_region_ref",
                    table
                        .header_region_ref
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
                ),
                (
                    "totals_region_ref",
                    table
                        .totals_region_ref
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
                ),
                ("header_row_present", table.header_row_present.to_string()),
                ("totals_row_present", table.totals_row_present.to_string()),
                ("columns", columns),
            ],
        )
    }));
    let enclosing = enclosing_table_ref
        .as_ref()
        .map_or("none".to_string(), |table_ref| table_ref.table_id.clone());
    let caller = caller_table_region
        .as_ref()
        .map_or("none".to_string(), |region| {
            identity_record(
                "caller_table_region",
                [
                    ("table_id", region.table_id.clone()),
                    (
                        "region_kind",
                        match region.region_kind {
                            TableRegionKind::Headers => "headers",
                            TableRegionKind::Data => "data",
                            TableRegionKind::Totals => "totals",
                        }
                        .to_string(),
                    ),
                    (
                        "data_row_offset",
                        region
                            .data_row_offset
                            .map_or("none".to_string(), |offset| offset.to_string()),
                    ),
                ],
            )
        });
    identity_record(
        "oxcalc.table_context.v1",
        [
            ("tables", table_parts),
            ("enclosing", enclosing),
            ("caller", caller),
        ],
    )
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use oxfml_core::{
        EvaluationBackend, StructuredReferenceBindDiagnosticLink,
        StructuredReferenceSelectedRegion,
        consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest},
        interface::{TableColumnDescriptor, TableRegionKind, TypedContextQueryBundle},
        seam::Locus,
        source::FormulaSourceRecord,
        syntax::token::TextSpan,
    };

    use super::*;
    use crate::dependency::DependencyGraph;
    use crate::structural::{
        StructuralNode, StructuralNodeKind, StructuralSnapshot, StructuralSnapshotId,
    };

    fn treecalc_table_snapshot() -> TreeCalcTableNodeSnapshot {
        TreeCalcTableNodeSnapshot {
            table_node_id: TreeNodeId(20),
            table_id: "tree-table:sales".to_string(),
            table_name: "SalesTable".to_string(),
            display_path: "Sales Table".to_string(),
            canonical_path: "Root/SalesTable".to_string(),
            virtual_anchor: TreeCalcTableVirtualAnchor {
                workbook_scope_ref: "treecalc-workbook:main".to_string(),
                sheet_scope_ref: "treecalc-virtual-sheet:tables".to_string(),
                start_row: 3,
                start_col: 2,
            },
            rows: vec![
                TreeCalcTableRowId("row:west".to_string()),
                TreeCalcTableRowId("row:east".to_string()),
                TreeCalcTableRowId("row:north".to_string()),
            ],
            columns: vec![
                TreeCalcTableColumnSnapshot {
                    column_id: "col:amount".to_string(),
                    column_name: "Amount".to_string(),
                    ordinal: 2,
                    body_metadata: TreeCalcTableColumnBodyMetadata::ConstantCells,
                    totals_metadata: Some(TreeCalcTableFormulaMetadata {
                        formula_artifact_id: "formula:totals:amount".to_string(),
                        bind_artifact_id: Some("bind:totals:amount".to_string()),
                        formula_text_version: "v1".to_string(),
                    }),
                },
                TreeCalcTableColumnSnapshot {
                    column_id: "col:region".to_string(),
                    column_name: "Region".to_string(),
                    ordinal: 1,
                    body_metadata: TreeCalcTableColumnBodyMetadata::ConstantCells,
                    totals_metadata: None,
                },
                TreeCalcTableColumnSnapshot {
                    column_id: "col:tax".to_string(),
                    column_name: "Tax".to_string(),
                    ordinal: 3,
                    body_metadata: TreeCalcTableColumnBodyMetadata::Formula(
                        TreeCalcTableFormulaMetadata {
                            formula_artifact_id: "formula:body:tax".to_string(),
                            bind_artifact_id: Some("bind:body:tax".to_string()),
                            formula_text_version: "v1".to_string(),
                        },
                    ),
                    totals_metadata: Some(TreeCalcTableFormulaMetadata {
                        formula_artifact_id: "formula:totals:tax".to_string(),
                        bind_artifact_id: None,
                        formula_text_version: "v1".to_string(),
                    }),
                },
            ],
            header_row_present: true,
            totals_row_present: true,
            table_namespace_version: "namespace:v1".to_string(),
            row_membership_version: "row-membership:v1".to_string(),
            row_order_version: "row-order:v1".to_string(),
            column_identity_version: "columns:v1".to_string(),
        }
    }

    #[test]
    fn projects_treecalc_table_node_snapshot_to_virtual_excel_table_descriptor() {
        let snapshot = treecalc_table_snapshot();

        let projection = project_treecalc_table_node_snapshot(&snapshot)
            .expect("table-node snapshot projects to generic TableDescriptor");
        let projected_again =
            project_treecalc_table_node_snapshot(&snapshot).expect("projection is repeatable");

        assert_eq!(projection, projected_again);
        assert_eq!(projection.table_node_id, TreeNodeId(20));
        assert_eq!(projection.display_path, "Sales Table");
        assert_eq!(projection.canonical_path, "Root/SalesTable");
        assert_eq!(projection.table_descriptor.table_id, "tree-table:sales");
        assert_eq!(projection.table_descriptor.table_name, "SalesTable");
        assert_eq!(
            projection.table_descriptor.workbook_scope_ref,
            "treecalc-workbook:main"
        );
        assert_eq!(
            projection.table_descriptor.sheet_scope_ref,
            "treecalc-virtual-sheet:tables"
        );
        assert_eq!(projection.table_descriptor.table_range_ref, "B3:D7");
        assert_eq!(
            projection.table_descriptor.header_region_ref.as_deref(),
            Some("B3:D3")
        );
        assert_eq!(
            projection.table_descriptor.totals_region_ref.as_deref(),
            Some("B7:D7")
        );
        assert_eq!(
            projection
                .table_descriptor
                .columns
                .iter()
                .map(|column| (
                    column.column_id.as_str(),
                    column.column_name.as_str(),
                    column.ordinal,
                    column.column_range_ref.as_str()
                ))
                .collect::<Vec<_>>(),
            vec![
                ("col:region", "Region", 1, "B4:B6"),
                ("col:amount", "Amount", 2, "C4:C6"),
                ("col:tax", "Tax", 3, "D4:D6"),
            ]
        );
        assert_eq!(
            projection
                .table_descriptor
                .row_membership_identity
                .as_deref(),
            Some(
                "treecalc.table_rows.membership_token.v1;table=16:tree-table:sales;version=17:row-membership:v1;row_count=1:3"
            )
        );
        assert_eq!(
            projection.table_descriptor.row_order_identity.as_deref(),
            Some(
                "treecalc.table_rows.order_token.v1;table=16:tree-table:sales;version=12:row-order:v1;row_count=1:3"
            )
        );
        assert!(
            !projection
                .table_descriptor
                .row_membership_identity
                .as_deref()
                .unwrap()
                .contains("row:east")
        );
        assert!(
            projection
                .oxcalc_row_membership_identity
                .contains("row:east")
        );
        assert_eq!(projection.context_packet.table_catalog.len(), 1);
        assert_eq!(
            projection.context_packet.table_catalog[0],
            projection.table_descriptor
        );
        assert!(
            projection
                .table_context_identity
                .contains("treecalc.table_namespace.token.v1")
        );
        assert!(
            projection
                .table_context_identity
                .contains("treecalc.table_anchor.token.v1")
        );
        assert!(!projection.table_context_identity.contains("row:east"));
        assert!(
            !projection
                .table_context_identity
                .contains("formula:body:tax")
        );
        assert!(
            projection
                .body_metadata_identity
                .contains("treecalc.table_body.formula.v1")
        );
        assert!(
            projection
                .totals_metadata_identity
                .contains("21:formula:totals:amount")
        );
    }

    #[test]
    fn table_context_identity_changes_on_table_shape_and_version_mutations() {
        let baseline = project_treecalc_table_node_snapshot(&treecalc_table_snapshot())
            .expect("baseline projects");
        let baseline_identity = baseline.table_context_identity;

        let mut renamed = treecalc_table_snapshot();
        renamed.table_namespace_version = "namespace:v2".to_string();
        renamed.table_name = "SalesRenamed".to_string();
        assert_ne!(
            project_treecalc_table_node_snapshot(&renamed)
                .unwrap()
                .table_context_identity,
            baseline_identity
        );

        let mut reordered = treecalc_table_snapshot();
        reordered.rows.reverse();
        reordered.row_order_version = "row-order:v2".to_string();
        assert_ne!(
            project_treecalc_table_node_snapshot(&reordered)
                .unwrap()
                .table_context_identity,
            baseline_identity
        );

        let mut row_added = treecalc_table_snapshot();
        row_added
            .rows
            .push(TreeCalcTableRowId("row:south".to_string()));
        row_added.row_membership_version = "row-membership:v2".to_string();
        let row_added_projection = project_treecalc_table_node_snapshot(&row_added).unwrap();
        assert_eq!(
            row_added_projection.table_descriptor.table_range_ref,
            "B3:D8"
        );
        assert_ne!(
            row_added_projection.table_context_identity,
            baseline_identity
        );

        let mut row_replaced = treecalc_table_snapshot();
        row_replaced.rows[0] = TreeCalcTableRowId("row:central".to_string());
        row_replaced.row_membership_version = "row-membership:v3".to_string();
        assert_ne!(
            project_treecalc_table_node_snapshot(&row_replaced)
                .unwrap()
                .table_context_identity,
            baseline_identity
        );

        let mut column_changed = treecalc_table_snapshot();
        column_changed.columns[0].column_name = "GrossAmount".to_string();
        column_changed.column_identity_version = "columns:v2".to_string();
        assert_ne!(
            project_treecalc_table_node_snapshot(&column_changed)
                .unwrap()
                .table_context_identity,
            baseline_identity
        );

        let mut column_added = treecalc_table_snapshot();
        column_added.columns.push(TreeCalcTableColumnSnapshot {
            column_id: "col:discount".to_string(),
            column_name: "Discount".to_string(),
            ordinal: 4,
            body_metadata: TreeCalcTableColumnBodyMetadata::ConstantCells,
            totals_metadata: None,
        });
        column_added.column_identity_version = "columns:v3".to_string();
        let column_added_projection = project_treecalc_table_node_snapshot(&column_added).unwrap();
        assert_eq!(
            column_added_projection.table_descriptor.table_range_ref,
            "B3:E7"
        );
        assert_ne!(
            column_added_projection.table_context_identity,
            baseline_identity
        );

        let mut column_reordered = treecalc_table_snapshot();
        column_reordered.columns[0].ordinal = 3;
        column_reordered.columns[2].ordinal = 2;
        column_reordered.column_identity_version = "columns:v4".to_string();
        assert_eq!(
            project_treecalc_table_node_snapshot(&column_reordered)
                .unwrap()
                .table_descriptor
                .columns
                .iter()
                .map(|column| column.column_id.as_str())
                .collect::<Vec<_>>(),
            vec!["col:region", "col:tax", "col:amount"]
        );
        assert_ne!(
            project_treecalc_table_node_snapshot(&column_reordered)
                .unwrap()
                .table_context_identity,
            baseline_identity
        );

        let mut header_removed = treecalc_table_snapshot();
        header_removed.header_row_present = false;
        assert_ne!(
            project_treecalc_table_node_snapshot(&header_removed)
                .unwrap()
                .table_context_identity,
            baseline_identity
        );

        let mut totals_removed = treecalc_table_snapshot();
        totals_removed.totals_row_present = false;
        assert_ne!(
            project_treecalc_table_node_snapshot(&totals_removed)
                .unwrap()
                .table_context_identity,
            baseline_identity
        );

        let mut moved_anchor = treecalc_table_snapshot();
        moved_anchor.virtual_anchor.start_col = 5;
        let moved_projection = project_treecalc_table_node_snapshot(&moved_anchor).unwrap();
        assert_eq!(moved_projection.table_descriptor.table_range_ref, "E3:G7");
        assert_ne!(moved_projection.table_context_identity, baseline_identity);
    }

    #[test]
    fn virtual_anchor_identity_contract_separates_table_namespace_anchor_and_membership_changes() {
        let baseline_snapshot = treecalc_table_snapshot();
        let baseline =
            project_treecalc_table_node_snapshot(&baseline_snapshot).expect("baseline projects");
        let reopened = project_treecalc_table_node_snapshot(&baseline_snapshot)
            .expect("save reopen of unchanged table preserves identities");

        assert_eq!(reopened.table_id, baseline.table_id);
        assert_eq!(reopened.table_node_id, baseline.table_node_id);
        assert_eq!(
            reopened.virtual_anchor_identity,
            baseline.virtual_anchor_identity
        );
        assert_eq!(
            reopened.table_context_identity,
            baseline.table_context_identity
        );
        assert_eq!(
            reopened.table_invalidation_identity,
            baseline.table_invalidation_identity
        );

        let mut renamed_path = baseline_snapshot.clone();
        renamed_path.table_name = "SalesByRegion".to_string();
        renamed_path.display_path = "Sales By Region".to_string();
        renamed_path.canonical_path = "Root/Reports/SalesByRegion".to_string();
        renamed_path.table_namespace_version = "namespace:v2".to_string();
        let renamed_path_projection =
            project_treecalc_table_node_snapshot(&renamed_path).expect("renamed table projects");
        assert_eq!(renamed_path_projection.table_id, baseline.table_id);
        assert_ne!(
            renamed_path_projection.table_namespace_identity,
            baseline.table_namespace_identity
        );
        assert_ne!(
            renamed_path_projection.table_context_identity,
            baseline.table_context_identity
        );
        assert_eq!(
            renamed_path_projection.virtual_anchor_identity,
            baseline.virtual_anchor_identity
        );
        assert_eq!(
            renamed_path_projection.oxcalc_row_membership_identity,
            baseline.oxcalc_row_membership_identity
        );
        assert_eq!(
            renamed_path_projection.oxcalc_column_identity,
            baseline.oxcalc_column_identity
        );

        let mut workspace_alias_changed = baseline_snapshot.clone();
        workspace_alias_changed.virtual_anchor.workbook_scope_ref =
            "treecalc-workbook:alias".to_string();
        let workspace_alias_projection =
            project_treecalc_table_node_snapshot(&workspace_alias_changed)
                .expect("workspace alias projects");
        assert_eq!(
            workspace_alias_projection.table_descriptor.table_range_ref,
            baseline.table_descriptor.table_range_ref
        );
        assert_ne!(
            workspace_alias_projection.virtual_anchor_identity,
            baseline.virtual_anchor_identity
        );
        assert_ne!(
            workspace_alias_projection.table_context_identity,
            baseline.table_context_identity
        );
        assert_eq!(
            workspace_alias_projection.oxcalc_row_order_identity,
            baseline.oxcalc_row_order_identity
        );

        let mut anchor_moved = baseline_snapshot.clone();
        anchor_moved.virtual_anchor.start_row = 6;
        anchor_moved.virtual_anchor.start_col = 4;
        let anchor_moved_projection =
            project_treecalc_table_node_snapshot(&anchor_moved).expect("moved table projects");
        assert_eq!(
            anchor_moved_projection.table_descriptor.table_range_ref,
            "D6:F10"
        );
        assert_ne!(
            anchor_moved_projection.virtual_anchor_identity,
            baseline.virtual_anchor_identity
        );
        assert_eq!(
            anchor_moved_projection.oxcalc_row_membership_identity,
            baseline.oxcalc_row_membership_identity
        );
        assert_eq!(
            anchor_moved_projection.oxcalc_column_identity,
            baseline.oxcalc_column_identity
        );

        let mut reordered = baseline_snapshot.clone();
        reordered.rows.reverse();
        reordered.row_order_version = "row-order:v2".to_string();
        let reordered_projection =
            project_treecalc_table_node_snapshot(&reordered).expect("reordered table projects");
        assert_eq!(
            reordered_projection.oxcalc_row_membership_identity,
            baseline.oxcalc_row_membership_identity
        );
        assert_ne!(
            reordered_projection.oxcalc_row_order_identity,
            baseline.oxcalc_row_order_identity
        );
        assert_ne!(
            reordered_projection.table_context_identity,
            baseline.table_context_identity
        );

        let mut row_added = baseline_snapshot;
        row_added
            .rows
            .push(TreeCalcTableRowId("row:south".to_string()));
        row_added.row_membership_version = "row-membership:v2".to_string();
        row_added.row_order_version = "row-order:v2".to_string();
        let row_added_projection =
            project_treecalc_table_node_snapshot(&row_added).expect("row-added table projects");
        assert_ne!(
            row_added_projection.oxcalc_row_membership_identity,
            baseline.oxcalc_row_membership_identity
        );
        assert_ne!(
            row_added_projection.oxcalc_row_order_identity,
            baseline.oxcalc_row_order_identity
        );
        assert_eq!(
            row_added_projection.table_descriptor.table_range_ref,
            "B3:D8"
        );
    }

    #[test]
    fn oxcalc_invalidation_identities_frame_domain_strings_without_separator_collisions() {
        let mut left = treecalc_table_snapshot();
        left.table_id = "table;a=1".to_string();
        left.rows = vec![
            TreeCalcTableRowId("row:a|b".to_string()),
            TreeCalcTableRowId("row:c".to_string()),
        ];
        left.columns[0].column_name = "Gross;Amount".to_string();
        left.columns[2].body_metadata =
            TreeCalcTableColumnBodyMetadata::Formula(TreeCalcTableFormulaMetadata {
                formula_artifact_id: "formula;a|b".to_string(),
                bind_artifact_id: Some("bind;c".to_string()),
                formula_text_version: "version:d".to_string(),
            });

        let mut right = left.clone();
        right.rows = vec![
            TreeCalcTableRowId("row:a".to_string()),
            TreeCalcTableRowId("row:b|c".to_string()),
        ];
        right.columns[0].column_name = "Gross".to_string();
        right.columns[1].column_name = "Amount;a=1".to_string();
        right.columns[2].body_metadata =
            TreeCalcTableColumnBodyMetadata::Formula(TreeCalcTableFormulaMetadata {
                formula_artifact_id: "formula;a".to_string(),
                bind_artifact_id: Some("b|bind;c".to_string()),
                formula_text_version: "version:d".to_string(),
            });

        let left_projection = project_treecalc_table_node_snapshot(&left).unwrap();
        let right_projection = project_treecalc_table_node_snapshot(&right).unwrap();

        assert_ne!(
            left_projection.oxcalc_row_membership_identity,
            right_projection.oxcalc_row_membership_identity
        );
        assert_ne!(
            left_projection.oxcalc_column_identity,
            right_projection.oxcalc_column_identity
        );
        assert_ne!(
            left_projection.table_invalidation_identity,
            right_projection.table_invalidation_identity
        );
        assert_ne!(
            left_projection.body_metadata_identity,
            right_projection.body_metadata_identity
        );
        assert_ne!(
            left_projection.context_packet.table_context_identity,
            right_projection.context_packet.table_context_identity
        );
        assert!(
            left_projection
                .oxcalc_row_membership_identity
                .contains("7:row:a|b")
        );
        assert!(
            left_projection
                .body_metadata_identity
                .contains("11:formula;a|b")
        );
        assert!(
            left_projection
                .context_packet
                .table_context_identity
                .contains("12:Gross;Amount")
        );
        assert!(!left_projection.table_context_identity.contains("row:a|b"));
    }

    #[test]
    fn projects_empty_body_table_snapshot_with_empty_data_ranges() {
        let mut empty_data_body = treecalc_table_snapshot();
        empty_data_body.rows.clear();
        empty_data_body.row_membership_version = "row-membership:empty".to_string();
        empty_data_body.row_order_version = "row-order:empty".to_string();

        let projection = project_treecalc_table_node_snapshot(&empty_data_body)
            .expect("zero-row table projects through generic empty data-body packet shape");

        assert_eq!(projection.table_descriptor.table_range_ref, "B3:D4");
        assert_eq!(
            projection.table_descriptor.header_region_ref.as_deref(),
            Some("B3:D3")
        );
        assert_eq!(
            projection.table_descriptor.totals_region_ref.as_deref(),
            Some("B4:D4")
        );
        assert_eq!(
            projection
                .table_descriptor
                .columns
                .iter()
                .map(|column| (
                    column.column_id.as_str(),
                    column.ordinal,
                    column.column_range_ref.as_str()
                ))
                .collect::<Vec<_>>(),
            vec![
                ("col:region", 1, ""),
                ("col:amount", 2, ""),
                ("col:tax", 3, "")
            ]
        );
        assert!(
            projection
                .table_descriptor
                .row_membership_identity
                .as_deref()
                .unwrap()
                .contains("row_count=1:0")
        );
        assert!(
            projection
                .table_context_identity
                .contains("row-membership:empty")
        );
    }

    #[test]
    fn table_projection_rejects_snapshot_shapes_that_would_create_private_semantics() {
        let mut invisible_empty_table = treecalc_table_snapshot();
        invisible_empty_table.rows.clear();
        invisible_empty_table.header_row_present = false;
        invisible_empty_table.totals_row_present = false;
        assert_eq!(
            project_treecalc_table_node_snapshot(&invisible_empty_table).unwrap_err(),
            TreeCalcTableProjectionError::EmptyTableHasNoRepresentableRows
        );

        let mut duplicate_column = treecalc_table_snapshot();
        duplicate_column.columns[1].column_id = duplicate_column.columns[0].column_id.clone();
        assert_eq!(
            project_treecalc_table_node_snapshot(&duplicate_column).unwrap_err(),
            TreeCalcTableProjectionError::DuplicateColumnId {
                column_id: "col:amount".to_string()
            }
        );

        let mut duplicate_row = treecalc_table_snapshot();
        duplicate_row.rows[1] = duplicate_row.rows[0].clone();
        assert_eq!(
            project_treecalc_table_node_snapshot(&duplicate_row).unwrap_err(),
            TreeCalcTableProjectionError::DuplicateRowId {
                row_id: "row:west".to_string()
            }
        );

        let mut duplicate_column_name = treecalc_table_snapshot();
        duplicate_column_name.columns[1].column_name =
            duplicate_column_name.columns[0].column_name.clone();
        assert_eq!(
            project_treecalc_table_node_snapshot(&duplicate_column_name).unwrap_err(),
            TreeCalcTableProjectionError::DuplicateColumnName {
                column_name: "Amount".to_string()
            }
        );

        let mut duplicate_column_ordinal = treecalc_table_snapshot();
        duplicate_column_ordinal.columns[1].ordinal = duplicate_column_ordinal.columns[0].ordinal;
        assert_eq!(
            project_treecalc_table_node_snapshot(&duplicate_column_ordinal).unwrap_err(),
            TreeCalcTableProjectionError::DuplicateColumnOrdinal { ordinal: 2 }
        );

        let mut ordinal_gap = treecalc_table_snapshot();
        ordinal_gap.columns[2].ordinal = 4;
        assert_eq!(
            project_treecalc_table_node_snapshot(&ordinal_gap).unwrap_err(),
            TreeCalcTableProjectionError::ColumnOrdinalMustStartAtOne {
                column_id: "col:tax".to_string(),
                ordinal: 4
            }
        );

        let mut invalid_anchor = treecalc_table_snapshot();
        invalid_anchor.virtual_anchor.start_row = 0;
        assert_eq!(
            project_treecalc_table_node_snapshot(&invalid_anchor).unwrap_err(),
            TreeCalcTableProjectionError::InvalidVirtualAnchor {
                start_row: 0,
                start_col: 2
            }
        );
    }

    #[test]
    fn prebinds_treecalc_table_path_structured_references_with_source_preservation() {
        let projection = project_treecalc_table_node_snapshot(&treecalc_table_snapshot()).unwrap();
        let source = "=SUM(SalesTable[Amount])+SalesTable[[#Headers],[Tax]]+SalesTable[@Tax]";
        let prebound =
            prebind_treecalc_table_structured_references(source, &[projection], None, None);

        assert_eq!(prebound.len(), 3);
        assert_eq!(prebound[0].source_token_text, "SalesTable[Amount]");
        assert_eq!(prebound[0].source_span_utf8, TextSpan::new(5, 18));
        assert_eq!(
            prebound[0].bind_record.source_span_utf8,
            prebound[0].source_span_utf8
        );
        assert_eq!(
            prebound[0].bind_record.source_token_text,
            prebound[0].source_token_text
        );
        assert_eq!(
            prebound[0].source_token_kind,
            StructuredReferenceSourceTokenKind::StructuredReference
        );
        assert_eq!(
            prebound[0].bind_record.source_token_kind,
            prebound[0].source_token_kind
        );
        assert_eq!(
            prebound[0].bind_record.bind_record_handle,
            prebound[0].host_ref_handle
        );
        assert_eq!(prebound[0].path_span_utf8, Some(TextSpan::new(5, 10)));
        assert_eq!(prebound[0].structured_tail_span_utf8, TextSpan::new(15, 8));
        assert_eq!(prebound[0].path_token_text.as_deref(), Some("SalesTable"));
        assert_eq!(
            prebound[0].structured_tail_token_text,
            "[Amount]".to_string()
        );
        assert_eq!(prebound[0].resolved_table_node_id, Some(TreeNodeId(20)));
        assert_eq!(
            prebound[0].resolved_table_id.as_deref(),
            Some("tree-table:sales")
        );
        assert_eq!(
            prebound[0].bind_record.selected_column_ids,
            vec!["col:amount"]
        );
        assert_eq!(
            prebound[0].bind_record.selected_sections,
            vec![StructuredSectionKind::Data]
        );
        assert!(!prebound[0].caller_context_dependency);
        assert!(prebound[0].diagnostics.is_empty());

        assert_eq!(
            prebound[1].source_token_text,
            "SalesTable[[#Headers],[Tax]]"
        );
        assert_eq!(prebound[1].bind_record.selected_column_ids, vec!["col:tax"]);
        assert_eq!(
            prebound[1].bind_record.selected_sections,
            vec![StructuredSectionKind::Headers]
        );
        assert_eq!(
            prebound[1].bind_record.selected_regions[0]
                .region_ref
                .as_deref(),
            Some("B3:D3")
        );

        assert_eq!(prebound[2].source_token_text, "SalesTable[@Tax]");
        assert_eq!(
            prebound[2].bind_record.source_token_kind,
            StructuredReferenceSourceTokenKind::StructuredReference
        );
        assert_eq!(
            prebound[2].bind_record.selected_sections,
            vec![StructuredSectionKind::ThisRow]
        );
        assert_eq!(prebound[2].bind_record.selected_column_ids, vec!["col:tax"]);
        assert!(prebound[2].bind_record.uses_this_row);
        assert!(prebound[2].caller_context_dependency);
        assert!(prebound[2].replay_identity.contains("SalesTable[@Tax]"));
    }

    #[test]
    fn prebinds_bracket_escaped_table_and_column_names_with_closing_brackets() {
        let mut snapshot = treecalc_table_snapshot();
        snapshot.display_path = "Sales]Table".to_string();
        snapshot.canonical_path = "Sales]Table".to_string();
        snapshot.table_name = "Sales]Table".to_string();
        snapshot.columns[2].column_name = "Tax]Rate".to_string();
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let source = "=SUM([Sales]]Table][[Tax]]Rate]])";
        let prebound =
            prebind_treecalc_table_structured_references(source, &[projection], None, None);

        assert_eq!(prebound.len(), 1);
        assert_eq!(prebound[0].source_token_text, "[Sales]]Table][[Tax]]Rate]]");
        assert_eq!(
            prebound[0].path_token_text.as_deref(),
            Some("[Sales]]Table]")
        );
        assert_eq!(prebound[0].structured_tail_token_text, "[[Tax]]Rate]]");
        assert_eq!(prebound[0].bind_record.selected_column_ids, vec!["col:tax"]);
        assert!(prebound[0].diagnostics.is_empty());
        assert_eq!(
            prebound[0].bind_record.source_span_utf8,
            prebound[0].source_span_utf8
        );
        assert_eq!(
            prebound[0].bind_record.bind_record_handle,
            prebound[0].host_ref_handle
        );
    }

    #[test]
    fn prebinds_omitted_table_refs_and_reports_table_path_diagnostics() {
        let projection = project_treecalc_table_node_snapshot(&treecalc_table_snapshot()).unwrap();
        let omitted = prebind_treecalc_table_structured_references(
            "=[@Tax]",
            std::slice::from_ref(&projection),
            Some(TableRef {
                table_id: "tree-table:sales".to_string(),
            }),
            Some(TableCallerRegion {
                table_id: "tree-table:sales".to_string(),
                region_kind: TableRegionKind::Data,
                data_row_offset: Some(1),
            }),
        );
        assert_eq!(omitted.len(), 1);
        assert_eq!(omitted[0].source_token_text, "[@Tax]");
        assert_eq!(
            omitted[0].source_token_kind,
            StructuredReferenceSourceTokenKind::StructuredReference
        );
        assert_eq!(
            omitted[0].resolved_table_id.as_deref(),
            Some("tree-table:sales")
        );
        assert!(omitted[0].selector_payload.omitted_table_name);
        assert!(omitted[0].caller_context_dependency);
        assert_eq!(omitted[0].bind_record.selected_column_ids, vec!["col:tax"]);

        let diagnostic = prebind_treecalc_table_structured_references(
            "=NotTable[Amount]+SalesTable[Missing]",
            &[projection],
            None,
            None,
        );
        assert_eq!(diagnostic.len(), 2);
        assert_eq!(
            diagnostic[0].diagnostics[0].diagnostic_code,
            "treecalc.table.path_not_table"
        );
        assert_eq!(
            diagnostic[1].diagnostics[0].diagnostic_code,
            "treecalc.table.unknown_column"
        );
        assert_eq!(
            diagnostic[1].bind_record.diagnostics[0].diagnostic_code,
            "treecalc.table.unknown_column"
        );
    }

    #[test]
    fn prebinds_treecalc_table_multi_column_ranges_for_reader_inputs() {
        let projection = project_treecalc_table_node_snapshot(&treecalc_table_snapshot()).unwrap();
        let source = "=SUM(SalesTable[[#Data],[Amount]:[Tax]])";
        let prebound =
            prebind_treecalc_table_structured_references(source, &[projection], None, None);

        assert_eq!(prebound.len(), 1);
        assert_eq!(
            prebound[0].bind_record.selected_column_ids,
            vec!["col:amount", "col:tax"]
        );
        assert_eq!(
            prebound[0].bind_record.selected_sections,
            vec![StructuredSectionKind::Data]
        );
        assert!(prebound[0].diagnostics.is_empty());
    }

    #[test]
    fn table_sparse_reader_projects_data_column_without_dense_blanks() {
        let snapshot = treecalc_table_snapshot();
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let prebound = prebind_treecalc_table_structured_references(
            "=SUM(SalesTable[Amount])",
            std::slice::from_ref(&projection),
            None,
            None,
        );
        let reader = TreeCalcTableSparseReader::from_oxfml_bind_record(
            &snapshot,
            &projection,
            &prebound[0].bind_record,
            None,
            amount_column_values(),
        )
        .expect("table column reader should build");

        assert_eq!(reader.declared_extent().row_count, 3);
        assert_eq!(reader.declared_extent().column_count, 1);
        assert_eq!(reader.defined_cardinality(), 2);
        assert_eq!(
            reader.reference(),
            &ReferenceLike::new(ReferenceKind::Area, "treecalc-virtual-sheet:tables!C4:C6")
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(1, 1)),
            SparseCellRead::Defined(EvalValue::Number(3.0))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(2, 1)),
            SparseCellRead::Defined(EvalValue::Text(ExcelText::from_interop_assignment("")))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(3, 1)),
            SparseCellRead::Blank
        );

        let cells = reader.defined_iter().collect::<Vec<_>>();
        assert_eq!(cells.len(), 2);
        assert_eq!(reader.access_summary().read_at_calls, 3);
        assert_eq!(reader.access_summary().defined_iter_calls, 1);
        assert_eq!(reader.access_summary().defined_iter_yield_count, 2);
    }

    #[test]
    fn table_sparse_reader_preserves_empty_data_body_as_zero_row_reference() {
        let mut snapshot = treecalc_table_snapshot();
        snapshot.rows.clear();
        snapshot.row_membership_version = "row-membership:empty".to_string();
        snapshot.row_order_version = "row-order:empty".to_string();
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let data = StructuredTableReferenceIntake::explicit_table(
            "structured-ref:empty-amount",
            "tree-table:sales",
        )
        .with_selected_columns(["col:amount".to_string()])
        .with_selected_regions([StructuredTableRegionSelection::Data]);

        let reader = TreeCalcTableSparseReader::from_reference_intake(
            &snapshot,
            &projection,
            &data,
            None,
            Vec::<TreeCalcTableSparseValue>::new(),
            "SalesTable[Amount]",
            "structured-ref:empty-amount",
            None,
        )
        .expect("empty data-body table column should produce a zero-row reader");

        assert_eq!(reader.declared_extent().row_count, 0);
        assert_eq!(reader.declared_extent().column_count, 1);
        assert_eq!(reader.defined_cardinality(), 0);
        assert_eq!(
            reader.reference(),
            &ReferenceLike::new(
                ReferenceKind::Structured,
                "empty-structured:treecalc-virtual-sheet:tables:Data:col:amount:1"
            )
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(1, 1)),
            SparseCellRead::Blank
        );
        assert_eq!(reader.access_summary().read_at_calls, 1);
        assert!(reader.defined_iter().next().is_none());

        let runtime_binding = reader.runtime_binding();
        assert_eq!(runtime_binding.sparse_reference_values.declared_rows, 0);
        assert_eq!(runtime_binding.sparse_reference_values.declared_cols, 1);
        assert!(
            runtime_binding
                .sparse_reference_values
                .defined_cells
                .is_empty()
        );
        assert!(runtime_binding.scalar_cell_values.is_empty());
    }

    #[test]
    fn empty_body_table_reader_keeps_headers_totals_and_current_row_diagnostics_typed() {
        let mut snapshot = treecalc_table_snapshot();
        snapshot.rows.clear();
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let all = StructuredTableReferenceIntake::explicit_table(
            "structured-ref:empty-all",
            "tree-table:sales",
        )
        .with_selected_columns(["col:amount".to_string(), "col:tax".to_string()])
        .with_selected_regions([StructuredTableRegionSelection::All]);
        let reader = TreeCalcTableSparseReader::from_reference_intake(
            &snapshot,
            &projection,
            &all,
            None,
            [TreeCalcTableSparseValue::totals(
                "col:amount",
                EvalValue::Number(0.0),
            )],
            "SalesTable[[#All],[Amount]:[Tax]]",
            "structured-ref:empty-all",
            None,
        )
        .expect("empty table #All should retain header and totals rows");

        assert_eq!(reader.declared_extent().row_count, 2);
        assert_eq!(reader.declared_extent().column_count, 2);
        assert_eq!(
            reader.reference(),
            &ReferenceLike::new(ReferenceKind::Area, "treecalc-virtual-sheet:tables!C3:D4")
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(1, 1)),
            SparseCellRead::Defined(EvalValue::Text(ExcelText::from_interop_assignment(
                "Amount"
            )))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(2, 1)),
            SparseCellRead::Defined(EvalValue::Number(0.0))
        );

        let current_row = StructuredTableReferenceIntake::explicit_table(
            "structured-ref:empty-current-row",
            "tree-table:sales",
        )
        .with_selected_columns(["col:amount".to_string()])
        .with_this_row();
        let current_row_error = TreeCalcTableSparseReader::from_reference_intake(
            &snapshot,
            &projection,
            &current_row,
            Some(&TableCallerRegion {
                table_id: "tree-table:sales".to_string(),
                region_kind: TableRegionKind::Data,
                data_row_offset: Some(0),
            }),
            Vec::<TreeCalcTableSparseValue>::new(),
            "SalesTable[@Amount]",
            "structured-ref:empty-current-row",
            None,
        )
        .unwrap_err();
        assert_eq!(
            current_row_error,
            TreeCalcTableSparseReaderError::CallerRowOutOfRange {
                row_offset: 0,
                row_count: 0
            }
        );
    }

    #[test]
    fn table_sparse_runtime_bindings_feed_first_aggregate_group() {
        let cases = [
            ("=SUM(SalesTable[Amount])", EvalValue::Number(3.0)),
            ("=COUNT(SalesTable[Amount])", EvalValue::Number(1.0)),
            ("=COUNTA(SalesTable[Amount])", EvalValue::Number(2.0)),
            ("=COUNTBLANK(SalesTable[Amount])", EvalValue::Number(2.0)),
        ];

        for (formula, expected) in cases {
            let snapshot = runtime_treecalc_table_snapshot();
            let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
            let prebound = prebind_treecalc_table_structured_references(
                formula,
                std::slice::from_ref(&projection),
                None,
                None,
            );
            let reader = TreeCalcTableSparseReader::from_oxfml_bind_record(
                &snapshot,
                &projection,
                &prebound[0].bind_record,
                None,
                amount_column_values(),
            )
            .expect("table column reader should build");
            let runtime_binding = reader.runtime_binding();
            let sparse_reference = runtime_binding.reference.clone();
            let table_descriptor = projection.table_descriptor.clone();
            assert!(runtime_binding.scalar_cell_values.is_empty(), "{formula}");
            assert_eq!(
                runtime_binding.sparse_reference_values.declared_rows, 3,
                "{formula}"
            );
            assert_eq!(
                runtime_binding.sparse_reference_values.defined_cells.len(),
                reader.defined_cardinality(),
                "{formula}"
            );
            assert_eq!(reader.access_summary().read_at_calls, 0, "{formula}");

            let result = RuntimeEnvironment::new()
                .with_primary_locus(table_primary_locus(&table_descriptor))
                .with_table_context(vec![table_descriptor], None, None)
                .with_sparse_reference_value_bindings(vec![runtime_binding.sparse_reference_values])
                .execute(
                    RuntimeFormulaRequest::new(
                        FormulaSourceRecord::new("runtime:w056-tree-table-aggregate", 1, formula),
                        TypedContextQueryBundle::default(),
                    )
                    .with_backend(EvaluationBackend::OxFuncBacked),
                )
                .expect("table sparse aggregate should execute through OxFml/OxFunc");

            assert_eq!(result.evaluation.oxfunc_value, expected, "{formula}");
            assert_eq!(sparse_reference.target, "C4:C6");
        }
    }

    #[test]
    fn structured_table_function_breadth_inventory_keeps_table_refs_opaque() {
        let mut seen = BTreeSet::new();
        for admission in TREECALC_STRUCTURED_TABLE_FUNCTION_ADMISSION_INVENTORY {
            assert!(
                !admission.treecalc_selector_visible_to_oxfunc,
                "{} exposes TreeCalc selector semantics to OxFunc",
                admission.group_id
            );
            assert!(
                !admission.eager_materialization_closure_allowed,
                "{} allows eager materialization as closure evidence",
                admission.group_id
            );
            assert!(
                !admission.required_context.is_empty(),
                "{} should name its generic reference/context requirement",
                admission.group_id
            );
            assert!(
                matches!(
                    admission.oxfunc_counterpart_bead,
                    "oxf-ypq2.15" | "oxf-ypq2.16"
                ),
                "{} should link to the structured-table OxFunc counterpart bead",
                admission.group_id
            );
            for function_name in admission.function_names {
                assert!(
                    seen.insert(*function_name),
                    "{function_name} is assigned to multiple structured-table lanes"
                );
            }
        }

        for function_name in ["SUM", "COUNT", "COUNTA", "COUNTBLANK"] {
            let admission = treecalc_structured_table_function_admission(function_name)
                .expect("first aggregate function should be admitted");
            assert_eq!(
                admission.status,
                TreeCalcStructuredTableFunctionAdmissionStatus::CurrentReferencePreservingEvidence
            );
            assert_eq!(
                admission.carrier_mode,
                TreeCalcStructuredTableFunctionCarrierMode::SparseReferenceLike
            );
            assert!(
                admission.required_context.contains(
                    &TreeCalcStructuredTableFunctionContextRequirement::SparseReaderCells
                )
            );
        }

        assert_eq!(
            treecalc_structured_table_function_admission("sum")
                .expect("lookup is case-insensitive")
                .group_id,
            "first_aggregate_group"
        );
    }

    #[test]
    fn structured_table_function_breadth_inventory_types_context_sensitive_lanes() {
        let rows = treecalc_structured_table_function_admission("ROWS").unwrap();
        assert_eq!(
            rows.status,
            TreeCalcStructuredTableFunctionAdmissionStatus::AdmittedPendingOxFuncEvidence
        );
        assert_eq!(
            rows.carrier_mode,
            TreeCalcStructuredTableFunctionCarrierMode::ReferenceShapeOnly
        );
        assert!(
            rows.required_context
                .contains(&TreeCalcStructuredTableFunctionContextRequirement::ReferenceExtent)
        );

        let index = treecalc_structured_table_function_admission("INDEX").unwrap();
        assert!(index.required_context.contains(
            &TreeCalcStructuredTableFunctionContextRequirement::ReferenceCoordinateAccess
        ));

        let xlookup = treecalc_structured_table_function_admission("XLOOKUP").unwrap();
        assert_eq!(
            xlookup.carrier_mode,
            TreeCalcStructuredTableFunctionCarrierMode::MultiReferenceResolver
        );
        assert!(
            xlookup
                .required_context
                .contains(&TreeCalcStructuredTableFunctionContextRequirement::MultiRangeAlignment)
        );

        let average = treecalc_structured_table_function_admission("AVERAGE").unwrap();
        assert_eq!(average.group_id, "ordinary_range_scan_functions");
        assert_eq!(
            average.carrier_mode,
            TreeCalcStructuredTableFunctionCarrierMode::SparseReferenceLike
        );
        assert_eq!(
            average.status,
            TreeCalcStructuredTableFunctionAdmissionStatus::AdmittedPendingOxFuncEvidence
        );

        let textjoin = treecalc_structured_table_function_admission("TEXTJOIN").unwrap();
        assert_eq!(textjoin.group_id, "ordinary_range_scan_functions");
        assert!(
            textjoin
                .required_context
                .contains(&TreeCalcStructuredTableFunctionContextRequirement::SparseReaderCells)
        );

        let subtotal = treecalc_structured_table_function_admission("SUBTOTAL").unwrap();
        assert_eq!(
            subtotal.status,
            TreeCalcStructuredTableFunctionAdmissionStatus::RequiresTypedHostContext
        );
        assert!(subtotal.required_context.contains(
            &TreeCalcStructuredTableFunctionContextRequirement::RowVisibilityAndFilterState
        ));

        let filter = treecalc_structured_table_function_admission("FILTER").unwrap();
        assert_eq!(
            filter.carrier_mode,
            TreeCalcStructuredTableFunctionCarrierMode::DynamicArrayReferenceTransform
        );
        assert!(
            filter.required_context.contains(
                &TreeCalcStructuredTableFunctionContextRequirement::DynamicArraySpillPolicy
            )
        );

        let call = treecalc_structured_table_function_admission("CALL").unwrap();
        assert_eq!(
            call.status,
            TreeCalcStructuredTableFunctionAdmissionStatus::TypedExclusion
        );
        assert!(call.required_context.contains(
            &TreeCalcStructuredTableFunctionContextRequirement::ExternalNativeInvocationPolicy
        ));
    }

    #[test]
    fn table_sparse_runtime_binding_supports_current_row_scalar_formula() {
        let snapshot = runtime_treecalc_table_snapshot();
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let enclosing = Some(TableRef {
            table_id: "tree-table:sales".to_string(),
        });
        let caller_region = Some(TableCallerRegion {
            table_id: "tree-table:sales".to_string(),
            region_kind: TableRegionKind::Data,
            data_row_offset: Some(1),
        });
        let prebound = prebind_treecalc_table_structured_references(
            "=[@Amount]+2",
            std::slice::from_ref(&projection),
            enclosing.clone(),
            caller_region.clone(),
        );
        let reader = TreeCalcTableSparseReader::from_oxfml_bind_record(
            &snapshot,
            &projection,
            &prebound[0].bind_record,
            caller_region.as_ref(),
            [TreeCalcTableSparseValue::data(
                "row:east",
                "col:amount",
                EvalValue::Number(4.0),
            )],
        )
        .expect("current-row table reader should build");
        let runtime_binding = reader.runtime_binding();
        let table_descriptor = projection.table_descriptor.clone();

        assert_eq!(
            runtime_binding.scalar_cell_values,
            BTreeMap::from([("C5".to_string(), EvalValue::Number(4.0))])
        );

        let result = RuntimeEnvironment::new()
            .with_primary_locus(table_primary_locus(&table_descriptor))
            .with_table_context(vec![table_descriptor], enclosing, caller_region)
            .with_cell_values(runtime_binding.scalar_cell_values)
            .with_sparse_reference_value_bindings(vec![runtime_binding.sparse_reference_values])
            .execute(
                RuntimeFormulaRequest::new(
                    FormulaSourceRecord::new(
                        "runtime:w056-tree-table-current-row",
                        1,
                        "=[@Amount]+2",
                    ),
                    TypedContextQueryBundle::default(),
                )
                .with_backend(EvaluationBackend::OxFuncBacked),
            )
            .expect("current-row scalar structured reference should execute");

        assert_eq!(result.evaluation.oxfunc_value, EvalValue::Number(6.0));
    }

    #[test]
    fn table_sparse_reader_projects_headers_totals_and_all_regions() {
        let snapshot = treecalc_table_snapshot();
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let all = StructuredTableReferenceIntake::explicit_table(
            "structured-ref:all-amount-tax",
            "tree-table:sales",
        )
        .with_selected_columns(["col:amount".to_string(), "col:tax".to_string()])
        .with_selected_regions([StructuredTableRegionSelection::All]);
        let reader = TreeCalcTableSparseReader::from_reference_intake(
            &snapshot,
            &projection,
            &all,
            None,
            [
                TreeCalcTableSparseValue::data("row:west", "col:amount", EvalValue::Number(3.0)),
                TreeCalcTableSparseValue::data("row:east", "col:amount", EvalValue::Number(4.0)),
                TreeCalcTableSparseValue::data("row:north", "col:tax", EvalValue::Number(1.5)),
                TreeCalcTableSparseValue::totals("col:amount", EvalValue::Number(7.0)),
            ],
            "SalesTable[[#All],[Amount]:[Tax]]",
            "structured-ref:all-amount-tax",
            None,
        )
        .expect("#All table reader should build");

        assert_eq!(reader.declared_extent().row_count, 5);
        assert_eq!(reader.declared_extent().column_count, 2);
        assert_eq!(
            reader.reference(),
            &ReferenceLike::new(ReferenceKind::Area, "treecalc-virtual-sheet:tables!C3:D7")
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(1, 1)),
            SparseCellRead::Defined(EvalValue::Text(ExcelText::from_interop_assignment(
                "Amount"
            )))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(1, 2)),
            SparseCellRead::Defined(EvalValue::Text(ExcelText::from_interop_assignment("Tax")))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(5, 1)),
            SparseCellRead::Defined(EvalValue::Number(7.0))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(5, 2)),
            SparseCellRead::Blank
        );
    }

    #[test]
    fn table_column_formula_runtime_evaluates_same_text_per_data_row() {
        let snapshot = runtime_treecalc_table_snapshot();
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let request = TreeCalcTableColumnFormulaRuntimeRequest {
            target_column_id: "col:tax".to_string(),
            formula_stable_id: "formula:body:tax".to_string(),
            formula_text_version: 1,
            formula_text: "=[@Amount]/10".to_string(),
            values: table_formula_amount_values(),
            runtime_context: TreeCalcTableFormulaRuntimeContext::default(),
        };

        let report = evaluate_treecalc_table_column_formula_rows(&snapshot, &projection, &request)
            .expect("table body formula rows should evaluate");

        assert_eq!(report.cell_results.len(), 3);
        assert_eq!(
            report
                .cell_results
                .iter()
                .map(|cell| cell.value.clone())
                .collect::<Vec<_>>(),
            vec![
                EvalValue::Number(1.0),
                EvalValue::Number(2.0),
                EvalValue::Number(3.0)
            ]
        );
        assert_eq!(report.formula_stable_id, "formula:body:tax");
        assert_eq!(report.formula_text, "=[@Amount]/10");
        assert_eq!(
            report.table_context_identity,
            projection.table_context_identity
        );
        assert_eq!(
            report
                .cell_results
                .iter()
                .map(|cell| cell.primary_locus.clone())
                .collect::<Vec<_>>(),
            vec![
                Locus {
                    sheet_id: "sheet:default".to_string(),
                    row: 4,
                    col: 4,
                },
                Locus {
                    sheet_id: "sheet:default".to_string(),
                    row: 5,
                    col: 4,
                },
                Locus {
                    sheet_id: "sheet:default".to_string(),
                    row: 6,
                    col: 4,
                },
            ]
        );

        let prepared_keys = report
            .cell_results
            .iter()
            .map(|cell| cell.prepared_formula_key.as_str())
            .collect::<BTreeSet<_>>();
        let dispatch_keys = report
            .cell_results
            .iter()
            .map(|cell| cell.dispatch_skeleton_key.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            prepared_keys.len(),
            3,
            "caller row context must invalidate per-row prepared identity"
        );
        assert_eq!(
            dispatch_keys.len(),
            1,
            "the same formula text should keep a reusable dispatch skeleton"
        );
        for cell in &report.cell_results {
            assert_eq!(cell.host_formula_context.dialect_id, "oxcalc.treecalc-v1");
            assert_eq!(
                cell.host_formula_context.capability_profile_id,
                "host-capabilities:treecalc-v1"
            );
            assert_eq!(
                cell.host_formula_context.resolution_rule_version,
                "treecalc-host-resolution:v1"
            );
            assert_eq!(
                cell.host_formula_context.table_context_identity.as_deref(),
                Some(report.table_context_identity.as_str())
            );
            assert_eq!(
                cell.host_formula_context.caller_context_identity.as_deref(),
                Some(cell.caller_context_id.as_str())
            );
            assert!(cell.registry_snapshot_identity.is_some());
            assert_eq!(cell.structured_reference_handles.len(), 1);
        }
    }

    #[test]
    fn table_totals_formula_runtime_uses_sparse_column_reference() {
        let snapshot = runtime_treecalc_table_snapshot();
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let request = TreeCalcTableColumnFormulaRuntimeRequest {
            target_column_id: "col:amount".to_string(),
            formula_stable_id: "formula:totals:amount".to_string(),
            formula_text_version: 1,
            formula_text: "=SUM(SalesTable[Amount])".to_string(),
            values: table_formula_amount_values(),
            runtime_context: TreeCalcTableFormulaRuntimeContext::default(),
        };

        let result = evaluate_treecalc_table_totals_formula(&snapshot, &projection, &request)
            .expect("totals formula should evaluate");

        assert_eq!(result.value, EvalValue::Number(60.0));
        assert_eq!(result.region_kind, TableRegionKind::Totals);
        assert_eq!(result.row_id, None);
        assert_eq!(result.row_offset, None);
        assert_eq!(
            result.primary_locus,
            Locus {
                sheet_id: "sheet:default".to_string(),
                row: 7,
                col: 3,
            }
        );
        assert!(
            result
                .host_formula_context
                .caller_context_identity
                .as_deref()
                .is_some_and(|identity| identity.contains("region=6:totals"))
        );
        assert_eq!(result.structured_reference_handles.len(), 1);
    }

    #[test]
    fn table_totals_formula_runtime_rejects_this_row_outside_data_region() {
        let snapshot = runtime_treecalc_table_snapshot();
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let request = TreeCalcTableColumnFormulaRuntimeRequest {
            target_column_id: "col:amount".to_string(),
            formula_stable_id: "formula:totals:amount".to_string(),
            formula_text_version: 1,
            formula_text: "=[@Amount]".to_string(),
            values: table_formula_amount_values(),
            runtime_context: TreeCalcTableFormulaRuntimeContext::default(),
        };

        let error = evaluate_treecalc_table_totals_formula(&snapshot, &projection, &request)
            .expect_err("current-row reference outside data region should be rejected");

        assert_eq!(
            error,
            TreeCalcTableFormulaRuntimeError::SparseReader {
                row_id: None,
                error: TreeCalcTableSparseReaderError::CallerRegionNotData {
                    region_kind: TableRegionKind::Totals
                },
            }
        );
    }

    #[test]
    fn table_current_row_reader_requires_caller_context() {
        let snapshot = runtime_treecalc_table_snapshot();
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let reference = StructuredTableReferenceIntake::explicit_table(
            "structured-ref:this-row",
            "tree-table:sales",
        )
        .with_selected_columns(["col:amount".to_string()])
        .with_this_row();

        let error = TreeCalcTableSparseReader::from_reference_intake(
            &snapshot,
            &projection,
            &reference,
            None,
            table_formula_amount_values(),
            "SalesTable[@Amount]",
            "structured-ref:this-row",
            None,
        )
        .expect_err("current-row sparse reader needs caller context");

        assert_eq!(
            error,
            TreeCalcTableSparseReaderError::MissingCallerTableRegion
        );
    }

    #[test]
    fn table_update_impact_covers_full_w056_update_set() {
        let owner = TreeNodeId(30);
        let baseline_snapshot = runtime_treecalc_table_snapshot();
        let baseline = project_treecalc_table_node_snapshot(&baseline_snapshot).unwrap();
        let source_handles = || vec!["structured-ref:amount".to_string()];

        let body_cell = classify_treecalc_table_update(
            TreeCalcTableUpdateScenarioKind::BodyCellEdit,
            Some(&baseline),
            Some(&baseline),
            [owner],
            source_handles(),
        );
        assert_eq!(
            body_cell.invalidation_reasons,
            BTreeSet::from([InvalidationReasonKind::StructuredTableRegionChanged])
        );
        assert!(body_cell.prepared_identity_inputs.is_empty());

        let mut body_formula = baseline_snapshot.clone();
        body_formula.columns[2].body_metadata =
            TreeCalcTableColumnBodyMetadata::Formula(TreeCalcTableFormulaMetadata {
                formula_artifact_id: "formula:body:tax".to_string(),
                bind_artifact_id: Some("bind:body:tax:v2".to_string()),
                formula_text_version: "v2".to_string(),
            });
        let body_formula_projection = project_treecalc_table_node_snapshot(&body_formula).unwrap();
        assert_update_has(
            TreeCalcTableUpdateScenarioKind::BodyFormulaEdit,
            &baseline,
            Some(&body_formula_projection),
            [
                InvalidationReasonKind::StructuredTableRegionChanged,
                InvalidationReasonKind::StructuredTableContextChanged,
            ],
            [
                TreeCalcTablePreparedIdentityInput::StructureContextVersion,
                TreeCalcTablePreparedIdentityInput::TableContextIdentity,
            ],
        );

        let mut row_insert = baseline_snapshot.clone();
        row_insert
            .rows
            .push(TreeCalcTableRowId("row:south".to_string()));
        row_insert.row_membership_version = "row-membership:v2".to_string();
        row_insert.row_order_version = "row-order:v2".to_string();
        let row_insert_projection = project_treecalc_table_node_snapshot(&row_insert).unwrap();
        assert_update_has(
            TreeCalcTableUpdateScenarioKind::RowInsert,
            &baseline,
            Some(&row_insert_projection),
            [
                InvalidationReasonKind::StructuredTableRowMembershipChanged,
                InvalidationReasonKind::StructuredTableRowOrderChanged,
                InvalidationReasonKind::StructuredTableRegionChanged,
                InvalidationReasonKind::StructuredTableCallerContextChanged,
                InvalidationReasonKind::StructuredTableContextChanged,
            ],
            [
                TreeCalcTablePreparedIdentityInput::TableContextIdentity,
                TreeCalcTablePreparedIdentityInput::CallerContextIdentity,
            ],
        );

        let mut row_delete = baseline_snapshot.clone();
        row_delete.rows.pop();
        row_delete.row_membership_version = "row-membership:v3".to_string();
        row_delete.row_order_version = "row-order:v3".to_string();
        let row_delete_projection = project_treecalc_table_node_snapshot(&row_delete).unwrap();
        assert_update_has(
            TreeCalcTableUpdateScenarioKind::RowDelete,
            &baseline,
            Some(&row_delete_projection),
            [
                InvalidationReasonKind::StructuredTableRowMembershipChanged,
                InvalidationReasonKind::StructuredTableRowOrderChanged,
                InvalidationReasonKind::StructuredTableRegionChanged,
                InvalidationReasonKind::StructuredTableCallerContextChanged,
                InvalidationReasonKind::StructuredTableContextChanged,
            ],
            [
                TreeCalcTablePreparedIdentityInput::TableContextIdentity,
                TreeCalcTablePreparedIdentityInput::CallerContextIdentity,
            ],
        );

        let mut row_reorder = baseline_snapshot.clone();
        row_reorder.rows.reverse();
        row_reorder.row_order_version = "row-order:v4".to_string();
        let row_reorder_projection = project_treecalc_table_node_snapshot(&row_reorder).unwrap();
        assert_update_has(
            TreeCalcTableUpdateScenarioKind::RowReorder,
            &baseline,
            Some(&row_reorder_projection),
            [
                InvalidationReasonKind::StructuredTableRowOrderChanged,
                InvalidationReasonKind::StructuredTableCallerContextChanged,
                InvalidationReasonKind::StructuredTableContextChanged,
            ],
            [
                TreeCalcTablePreparedIdentityInput::TableContextIdentity,
                TreeCalcTablePreparedIdentityInput::CallerContextIdentity,
            ],
        );

        let mut column_insert = baseline_snapshot.clone();
        column_insert.columns.push(TreeCalcTableColumnSnapshot {
            column_id: "col:discount".to_string(),
            column_name: "Discount".to_string(),
            ordinal: 4,
            body_metadata: TreeCalcTableColumnBodyMetadata::ConstantCells,
            totals_metadata: None,
        });
        column_insert.column_identity_version = "columns:v2".to_string();
        let column_insert_projection =
            project_treecalc_table_node_snapshot(&column_insert).unwrap();
        assert_column_update(
            TreeCalcTableUpdateScenarioKind::ColumnInsert,
            &baseline,
            &column_insert_projection,
        );

        let mut column_delete = baseline_snapshot.clone();
        column_delete
            .columns
            .retain(|column| column.column_id != "col:tax");
        column_delete.column_identity_version = "columns:v3".to_string();
        let column_delete_projection =
            project_treecalc_table_node_snapshot(&column_delete).unwrap();
        assert_column_update(
            TreeCalcTableUpdateScenarioKind::ColumnDelete,
            &baseline,
            &column_delete_projection,
        );

        let mut column_reorder = baseline_snapshot.clone();
        column_reorder.columns[0].ordinal = 3;
        column_reorder.columns[2].ordinal = 2;
        column_reorder.column_identity_version = "columns:v4".to_string();
        let column_reorder_projection =
            project_treecalc_table_node_snapshot(&column_reorder).unwrap();
        assert_column_update(
            TreeCalcTableUpdateScenarioKind::ColumnReorder,
            &baseline,
            &column_reorder_projection,
        );

        let mut column_rename = baseline_snapshot.clone();
        column_rename.columns[0].column_name = "GrossAmount".to_string();
        column_rename.column_identity_version = "columns:v5".to_string();
        let column_rename_projection =
            project_treecalc_table_node_snapshot(&column_rename).unwrap();
        assert_column_update(
            TreeCalcTableUpdateScenarioKind::ColumnRename,
            &baseline,
            &column_rename_projection,
        );
        assert_column_update(
            TreeCalcTableUpdateScenarioKind::HeaderTextEdit,
            &baseline,
            &column_rename_projection,
        );

        let mut totals_toggle = baseline_snapshot.clone();
        totals_toggle.totals_row_present = false;
        let totals_toggle_projection =
            project_treecalc_table_node_snapshot(&totals_toggle).unwrap();
        assert_update_has(
            TreeCalcTableUpdateScenarioKind::TotalsRowToggle,
            &baseline,
            Some(&totals_toggle_projection),
            [
                InvalidationReasonKind::StructuredTableRegionChanged,
                InvalidationReasonKind::StructuredTableContextChanged,
            ],
            [TreeCalcTablePreparedIdentityInput::TableContextIdentity],
        );

        let mut totals_formula = baseline_snapshot.clone();
        totals_formula.columns[0].totals_metadata = Some(TreeCalcTableFormulaMetadata {
            formula_artifact_id: "formula:totals:amount".to_string(),
            bind_artifact_id: Some("bind:totals:amount:v2".to_string()),
            formula_text_version: "v2".to_string(),
        });
        let totals_formula_projection =
            project_treecalc_table_node_snapshot(&totals_formula).unwrap();
        assert_update_has(
            TreeCalcTableUpdateScenarioKind::TotalsFormulaEdit,
            &baseline,
            Some(&totals_formula_projection),
            [
                InvalidationReasonKind::StructuredTableRegionChanged,
                InvalidationReasonKind::StructuredTableContextChanged,
            ],
            [
                TreeCalcTablePreparedIdentityInput::TableContextIdentity,
                TreeCalcTablePreparedIdentityInput::StructureContextVersion,
            ],
        );

        let mut table_rename = baseline_snapshot.clone();
        table_rename.table_name = "SalesRenamed".to_string();
        table_rename.table_namespace_version = "namespace:v2".to_string();
        let table_rename_projection = project_treecalc_table_node_snapshot(&table_rename).unwrap();
        assert_update_has(
            TreeCalcTableUpdateScenarioKind::TableRename,
            &baseline,
            Some(&table_rename_projection),
            [
                InvalidationReasonKind::StructuredTableContextChanged,
                InvalidationReasonKind::StructuralRebindRequired,
            ],
            [
                TreeCalcTablePreparedIdentityInput::HostNamespaceVersion,
                TreeCalcTablePreparedIdentityInput::TableContextIdentity,
                TreeCalcTablePreparedIdentityInput::ResolutionRuleVersion,
            ],
        );

        let mut table_move = baseline_snapshot.clone();
        table_move.virtual_anchor.start_col = 5;
        let table_move_projection = project_treecalc_table_node_snapshot(&table_move).unwrap();
        assert_update_has(
            TreeCalcTableUpdateScenarioKind::TableMove,
            &baseline,
            Some(&table_move_projection),
            [
                InvalidationReasonKind::StructuredTableRegionChanged,
                InvalidationReasonKind::StructuredTableContextChanged,
                InvalidationReasonKind::StructuralRebindRequired,
            ],
            [TreeCalcTablePreparedIdentityInput::TableContextIdentity],
        );

        let delete = classify_treecalc_table_update(
            TreeCalcTableUpdateScenarioKind::TableDelete,
            Some(&baseline),
            None,
            [owner],
            source_handles(),
        );
        assert!(
            delete
                .invalidation_reasons
                .contains(&InvalidationReasonKind::StructuralRebindRequired)
        );
        assert!(
            delete
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::StructuredTableDataRegion)
        );

        let save_reopen = classify_treecalc_table_update(
            TreeCalcTableUpdateScenarioKind::SaveReopen,
            Some(&baseline),
            Some(&baseline),
            [owner],
            source_handles(),
        );
        assert!(save_reopen.changed_dependency_kinds.is_empty());
        assert!(save_reopen.invalidation_reasons.is_empty());
        assert!(save_reopen.invalidation_seeds.is_empty());

        let mut structural_rebind = baseline_snapshot;
        structural_rebind.canonical_path = "Root/Archive/SalesTable".to_string();
        structural_rebind.table_namespace_version = "namespace:v3".to_string();
        let structural_rebind_projection =
            project_treecalc_table_node_snapshot(&structural_rebind).unwrap();
        assert_update_has(
            TreeCalcTableUpdateScenarioKind::StructuralRebind,
            &baseline,
            Some(&structural_rebind_projection),
            [
                InvalidationReasonKind::StructuredTableContextChanged,
                InvalidationReasonKind::StructuralRebindRequired,
            ],
            [
                TreeCalcTablePreparedIdentityInput::HostNamespaceVersion,
                TreeCalcTablePreparedIdentityInput::TableContextIdentity,
                TreeCalcTablePreparedIdentityInput::ResolutionRuleVersion,
            ],
        );
    }

    #[test]
    fn table_lifecycle_callback_matrix_covers_full_w056_update_set() {
        let owner = TreeNodeId(30);
        let baseline_snapshot = runtime_treecalc_table_snapshot();
        let baseline_projection = project_treecalc_table_node_snapshot(&baseline_snapshot).unwrap();
        let baseline_state = TreeCalcTableLifecycleVersionState::from_snapshot_projection(
            &baseline_snapshot,
            &baseline_projection,
        );
        let context_inputs =
            TreeCalcTableLifecycleContextVersions::default().prepared_identity_inputs();

        let exercise = |scenario: TreeCalcTableUpdateScenarioKind,
                        after_snapshot: Option<TreeCalcTableNodeSnapshot>| {
            let after_projection = after_snapshot
                .as_ref()
                .map(project_treecalc_table_node_snapshot)
                .transpose()
                .unwrap();
            let after_state = after_snapshot.as_ref().zip(after_projection.as_ref()).map(
                |(snapshot, projection)| {
                    TreeCalcTableLifecycleVersionState::from_snapshot_projection(
                        snapshot, projection,
                    )
                },
            );
            let expected = classify_treecalc_table_update(
                scenario,
                Some(&baseline_projection),
                after_projection.as_ref(),
                [owner],
                ["structured-ref:amount".to_string()],
            );
            let mut packet = TreeCalcTableLifecycleCallbackPacket::new(scenario.into())
                .with_before(baseline_state.clone())
                .with_owner_nodes([owner])
                .with_source_reference_handles(["structured-ref:amount"]);
            if let Some(after_state) = after_state {
                packet = packet.with_after(after_state);
            }
            let report = classify_treecalc_table_lifecycle_callback(&packet);
            let mut expected_inputs = expected.prepared_identity_inputs.clone();
            expected_inputs.extend(context_inputs.iter().copied());

            assert!(
                report.diagnostics.is_empty(),
                "{scenario:?}: unexpected diagnostics {:?}",
                report.diagnostics
            );
            assert_eq!(
                report.source_reference_handles,
                expected.source_reference_handles
            );
            assert!(
                expected
                    .changed_dependency_kinds
                    .is_subset(&report.changed_dependency_kinds),
                "{scenario:?}: expected dependency kinds {:?}, got {:?}",
                expected.changed_dependency_kinds,
                report.changed_dependency_kinds
            );
            assert!(
                expected
                    .invalidation_reasons
                    .is_subset(&report.invalidation_reasons),
                "{scenario:?}: expected invalidation reasons {:?}, got {:?}",
                expected.invalidation_reasons,
                report.invalidation_reasons
            );
            assert!(
                expected_inputs.is_subset(&report.prepared_identity_inputs),
                "{scenario:?}: expected prepared inputs {expected_inputs:?}, got {:?}",
                report.prepared_identity_inputs
            );
            assert!(
                report
                    .invalidation_seeds
                    .iter()
                    .all(|seed| seed.node_id == owner),
                "{scenario:?}: invalidation seeds used the wrong owner"
            );
        };

        exercise(
            TreeCalcTableUpdateScenarioKind::BodyCellEdit,
            Some(baseline_snapshot.clone()),
        );

        let mut body_formula = baseline_snapshot.clone();
        body_formula.columns[2].body_metadata =
            TreeCalcTableColumnBodyMetadata::Formula(TreeCalcTableFormulaMetadata {
                formula_artifact_id: "formula:body:tax".to_string(),
                bind_artifact_id: Some("bind:body:tax:v2".to_string()),
                formula_text_version: "v2".to_string(),
            });
        exercise(
            TreeCalcTableUpdateScenarioKind::BodyFormulaEdit,
            Some(body_formula),
        );

        let mut row_insert = baseline_snapshot.clone();
        row_insert
            .rows
            .push(TreeCalcTableRowId("row:south".to_string()));
        row_insert.row_membership_version = "row-membership:v2".to_string();
        row_insert.row_order_version = "row-order:v2".to_string();
        exercise(TreeCalcTableUpdateScenarioKind::RowInsert, Some(row_insert));

        let mut row_delete = baseline_snapshot.clone();
        row_delete.rows.pop();
        row_delete.row_membership_version = "row-membership:v3".to_string();
        row_delete.row_order_version = "row-order:v3".to_string();
        exercise(TreeCalcTableUpdateScenarioKind::RowDelete, Some(row_delete));

        let mut row_reorder = baseline_snapshot.clone();
        row_reorder.rows.reverse();
        row_reorder.row_order_version = "row-order:v4".to_string();
        exercise(
            TreeCalcTableUpdateScenarioKind::RowReorder,
            Some(row_reorder),
        );

        let mut column_insert = baseline_snapshot.clone();
        column_insert.columns.push(TreeCalcTableColumnSnapshot {
            column_id: "col:discount".to_string(),
            column_name: "Discount".to_string(),
            ordinal: 4,
            body_metadata: TreeCalcTableColumnBodyMetadata::ConstantCells,
            totals_metadata: None,
        });
        column_insert.column_identity_version = "columns:v2".to_string();
        exercise(
            TreeCalcTableUpdateScenarioKind::ColumnInsert,
            Some(column_insert),
        );

        let mut column_delete = baseline_snapshot.clone();
        column_delete
            .columns
            .retain(|column| column.column_id != "col:tax");
        column_delete.column_identity_version = "columns:v3".to_string();
        exercise(
            TreeCalcTableUpdateScenarioKind::ColumnDelete,
            Some(column_delete),
        );

        let mut column_reorder = baseline_snapshot.clone();
        column_reorder.columns[0].ordinal = 3;
        column_reorder.columns[2].ordinal = 2;
        column_reorder.column_identity_version = "columns:v4".to_string();
        exercise(
            TreeCalcTableUpdateScenarioKind::ColumnReorder,
            Some(column_reorder),
        );

        let mut column_rename = baseline_snapshot.clone();
        column_rename.columns[0].column_name = "GrossAmount".to_string();
        column_rename.column_identity_version = "columns:v5".to_string();
        exercise(
            TreeCalcTableUpdateScenarioKind::ColumnRename,
            Some(column_rename.clone()),
        );
        exercise(
            TreeCalcTableUpdateScenarioKind::HeaderTextEdit,
            Some(column_rename),
        );

        let mut totals_toggle = baseline_snapshot.clone();
        totals_toggle.totals_row_present = false;
        exercise(
            TreeCalcTableUpdateScenarioKind::TotalsRowToggle,
            Some(totals_toggle),
        );

        let mut totals_formula = baseline_snapshot.clone();
        totals_formula.columns[0].totals_metadata = Some(TreeCalcTableFormulaMetadata {
            formula_artifact_id: "formula:totals:amount".to_string(),
            bind_artifact_id: Some("bind:totals:amount:v2".to_string()),
            formula_text_version: "v2".to_string(),
        });
        exercise(
            TreeCalcTableUpdateScenarioKind::TotalsFormulaEdit,
            Some(totals_formula),
        );

        let mut table_rename = baseline_snapshot.clone();
        table_rename.table_name = "SalesRenamed".to_string();
        table_rename.table_namespace_version = "namespace:v2".to_string();
        exercise(
            TreeCalcTableUpdateScenarioKind::TableRename,
            Some(table_rename),
        );

        let mut table_move = baseline_snapshot.clone();
        table_move.virtual_anchor.start_col = 5;
        exercise(TreeCalcTableUpdateScenarioKind::TableMove, Some(table_move));

        exercise(TreeCalcTableUpdateScenarioKind::TableDelete, None);

        exercise(
            TreeCalcTableUpdateScenarioKind::SaveReopen,
            Some(baseline_snapshot.clone()),
        );

        let mut structural_rebind = baseline_snapshot;
        structural_rebind.canonical_path = "Root/Archive/SalesTable".to_string();
        structural_rebind.table_namespace_version = "namespace:v3".to_string();
        exercise(
            TreeCalcTableUpdateScenarioKind::StructuralRebind,
            Some(structural_rebind),
        );
    }

    #[test]
    fn table_lifecycle_callback_contract_reports_versions_handles_and_invalidations() {
        let owner = TreeNodeId(30);
        let baseline_snapshot = runtime_treecalc_table_snapshot();
        let baseline = project_treecalc_table_node_snapshot(&baseline_snapshot).unwrap();
        let before_state = TreeCalcTableLifecycleVersionState::from_snapshot_projection(
            &baseline_snapshot,
            &baseline,
        );

        let mut row_reorder = baseline_snapshot.clone();
        row_reorder.rows.reverse();
        row_reorder.row_order_version = "row-order:v4".to_string();
        let row_reorder_projection = project_treecalc_table_node_snapshot(&row_reorder).unwrap();
        let after_state = TreeCalcTableLifecycleVersionState::from_snapshot_projection(
            &row_reorder,
            &row_reorder_projection,
        );

        let report = classify_treecalc_table_lifecycle_callback(
            &TreeCalcTableLifecycleCallbackPacket::new(TreeCalcTableLifecycleEventKind::RowReorder)
                .with_before(before_state.clone())
                .with_after(after_state.clone())
                .with_owner_nodes([owner])
                .with_source_reference_handles(["structured-ref:amount"])
                .with_changed_rows([
                    TreeCalcTableRowId("row:east".to_string()),
                    TreeCalcTableRowId("row:west".to_string()),
                ]),
        );

        assert!(report.diagnostics.is_empty());
        assert_eq!(
            report.event_kind,
            TreeCalcTableLifecycleEventKind::RowReorder
        );
        assert_eq!(
            report.before_state.as_ref().unwrap().table_node_id,
            TreeNodeId(20)
        );
        assert_eq!(
            report.after_state.as_ref().unwrap().row_order_version,
            "row-order:v4"
        );
        assert_eq!(report.source_reference_handles, ["structured-ref:amount"]);
        assert!(report.changed_row_ids.iter().any(|row| row.0 == "row:east"));
        assert!(
            report
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::StructuredTableRowOrder)
        );
        assert!(
            report
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::StructuredTableCallerContext)
        );
        assert!(
            report
                .invalidation_reasons
                .contains(&InvalidationReasonKind::StructuredTableRowOrderChanged)
        );
        assert!(
            report
                .prepared_identity_inputs
                .contains(&TreeCalcTablePreparedIdentityInput::CallerContextIdentity)
        );
        assert!(report.invalidation_seeds.iter().any(|seed| {
            seed.node_id == owner
                && seed.reason == InvalidationReasonKind::StructuredTableRowOrderChanged
        }));
        assert!(
            report
                .callback_identity
                .starts_with("treecalc.table_lifecycle.callback.v1")
        );
        assert_eq!(
            before_state.table_context_identity,
            baseline.table_context_identity
        );
    }

    #[test]
    fn table_lifecycle_contract_covers_create_delete_and_stable_id_diagnostics() {
        let owner = TreeNodeId(30);
        let baseline_snapshot = runtime_treecalc_table_snapshot();
        let baseline = project_treecalc_table_node_snapshot(&baseline_snapshot).unwrap();
        let baseline_state = TreeCalcTableLifecycleVersionState::from_snapshot_projection(
            &baseline_snapshot,
            &baseline,
        );

        let create_report = classify_treecalc_table_lifecycle_callback(
            &TreeCalcTableLifecycleCallbackPacket::new(
                TreeCalcTableLifecycleEventKind::TableCreate,
            )
            .with_after(baseline_state.clone())
            .with_owner_nodes([owner])
            .with_changed_rows(baseline_snapshot.rows.clone())
            .with_changed_columns(["col:amount", "col:region", "col:tax"]),
        );
        assert!(create_report.diagnostics.is_empty());
        assert!(
            create_report
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::StructuredTableIdentity)
        );
        assert!(
            create_report
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::StructuredTableEnclosingTable)
        );
        assert!(
            create_report
                .invalidation_reasons
                .contains(&InvalidationReasonKind::DependencyAdded)
        );
        assert!(
            create_report
                .prepared_identity_inputs
                .contains(&TreeCalcTablePreparedIdentityInput::HostNamespaceVersion)
        );
        assert!(
            create_report
                .prepared_identity_inputs
                .contains(&TreeCalcTablePreparedIdentityInput::StructureContextVersion)
        );

        let delete_report = classify_treecalc_table_lifecycle_callback(
            &TreeCalcTableLifecycleCallbackPacket::new(
                TreeCalcTableLifecycleEventKind::TableDelete,
            )
            .with_before(baseline_state.clone())
            .with_owner_nodes([owner])
            .with_source_reference_handles(["structured-ref:amount"]),
        );
        assert!(delete_report.diagnostics.is_empty());
        assert!(
            delete_report
                .invalidation_reasons
                .contains(&InvalidationReasonKind::StructuralRebindRequired)
        );
        assert!(
            delete_report
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::StructuredTableDataRegion)
        );

        let mut reopen_with_new_row_order_identity = baseline_state.clone();
        reopen_with_new_row_order_identity.row_order_identity =
            "treecalc.table.row_order:reopen-observed".to_string();
        let save_reopen_report = classify_treecalc_table_lifecycle_callback(
            &TreeCalcTableLifecycleCallbackPacket::new(TreeCalcTableLifecycleEventKind::SaveReopen)
                .with_before(baseline_state.clone())
                .with_after(reopen_with_new_row_order_identity)
                .with_owner_nodes([owner]),
        );
        assert!(
            save_reopen_report
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::StructuredTableRowOrder)
        );
        assert!(
            save_reopen_report
                .invalidation_reasons
                .contains(&InvalidationReasonKind::StructuredTableRowOrderChanged)
        );

        let mut wrong_id_snapshot = baseline_snapshot;
        wrong_id_snapshot.table_id = "tree-table:sales-recreated".to_string();
        let wrong_id_projection = project_treecalc_table_node_snapshot(&wrong_id_snapshot).unwrap();
        let wrong_id_state = TreeCalcTableLifecycleVersionState::from_snapshot_projection(
            &wrong_id_snapshot,
            &wrong_id_projection,
        );
        let wrong_id_report = classify_treecalc_table_lifecycle_callback(
            &TreeCalcTableLifecycleCallbackPacket::new(
                TreeCalcTableLifecycleEventKind::TableRename,
            )
            .with_before(baseline_state.clone())
            .with_after(wrong_id_state)
            .with_owner_nodes([owner]),
        );
        assert!(wrong_id_report.diagnostics.contains(
            &TreeCalcTableLifecycleContractDiagnostic::TableIdChangedAcrossLifecycle {
                before: "tree-table:sales".to_string(),
                after: "tree-table:sales-recreated".to_string(),
            }
        ));

        let malformed_report = classify_treecalc_table_lifecycle_callback(
            &TreeCalcTableLifecycleCallbackPacket::new(TreeCalcTableLifecycleEventKind::TableMove)
                .with_before(baseline_state),
        );
        assert!(malformed_report.diagnostics.contains(
            &TreeCalcTableLifecycleContractDiagnostic::MissingAfterState {
                event_kind: TreeCalcTableLifecycleEventKind::TableMove,
            }
        ));
        assert!(malformed_report.diagnostics.contains(
            &TreeCalcTableLifecycleContractDiagnostic::MissingOwnerNode {
                event_kind: TreeCalcTableLifecycleEventKind::TableMove,
            }
        ));
    }

    #[test]
    fn table_update_diagnostics_cover_deleted_missing_context_and_absent_regions() {
        let snapshot = runtime_treecalc_table_snapshot();
        let mut without_optional_regions = snapshot.clone();
        without_optional_regions.header_row_present = false;
        without_optional_regions.totals_row_present = false;
        let projection = project_treecalc_table_node_snapshot(&without_optional_regions).unwrap();

        let missing_column = StructuredTableReferenceIntake::explicit_table(
            "structured-ref:missing-col",
            "tree-table:sales",
        )
        .with_selected_columns(["col:missing".to_string()]);
        let current_row = StructuredTableReferenceIntake::explicit_table(
            "structured-ref:this-row",
            "tree-table:sales",
        )
        .with_selected_columns(["col:amount".to_string()])
        .with_this_row();
        let optional_regions = StructuredTableReferenceIntake::explicit_table(
            "structured-ref:optional-regions",
            "tree-table:sales",
        )
        .with_selected_regions([
            StructuredTableRegionSelection::Headers,
            StructuredTableRegionSelection::Totals,
        ]);

        assert_eq!(
            validate_treecalc_table_reference_after_update(
                "tree-table:sales",
                None,
                &missing_column,
                None
            ),
            vec![TreeCalcTableUpdateDiagnostic::DeletedTable {
                table_id: "tree-table:sales".to_string(),
                reference_handle: "structured-ref:missing-col".to_string(),
            }]
        );
        assert_eq!(
            validate_treecalc_table_reference_after_update(
                "tree-table:sales",
                Some(&projection),
                &missing_column,
                None
            ),
            vec![TreeCalcTableUpdateDiagnostic::MissingColumn {
                table_id: "tree-table:sales".to_string(),
                reference_handle: "structured-ref:missing-col".to_string(),
                column_id: "col:missing".to_string(),
            }]
        );
        assert_eq!(
            validate_treecalc_table_reference_after_update(
                "tree-table:sales",
                Some(&projection),
                &current_row,
                None
            ),
            vec![TreeCalcTableUpdateDiagnostic::MissingCallerTableRegion {
                table_id: "tree-table:sales".to_string(),
                reference_handle: "structured-ref:this-row".to_string(),
            }]
        );
        assert_eq!(
            validate_treecalc_table_reference_after_update(
                "tree-table:sales",
                Some(&projection),
                &optional_regions,
                None
            ),
            vec![
                TreeCalcTableUpdateDiagnostic::HeaderRowAbsent {
                    table_id: "tree-table:sales".to_string(),
                    reference_handle: "structured-ref:optional-regions".to_string(),
                },
                TreeCalcTableUpdateDiagnostic::TotalsRowAbsent {
                    table_id: "tree-table:sales".to_string(),
                    reference_handle: "structured-ref:optional-regions".to_string(),
                },
            ]
        );
    }

    #[test]
    fn body_value_edit_keeps_unaffected_table_reader_identity_stable() {
        let snapshot = runtime_treecalc_table_snapshot();
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let reference = StructuredTableReferenceIntake::explicit_table(
            "structured-ref:region",
            "tree-table:sales",
        )
        .with_selected_columns(["col:region".to_string()]);
        let before = TreeCalcTableSparseReader::from_reference_intake(
            &snapshot,
            &projection,
            &reference,
            None,
            region_values(),
            "SalesTable[Region]",
            "structured-ref:region",
            None,
        )
        .unwrap();
        let after = TreeCalcTableSparseReader::from_reference_intake(
            &snapshot,
            &projection,
            &reference,
            None,
            [
                TreeCalcTableSparseValue::data("row:west", "col:amount", EvalValue::Number(99.0)),
                TreeCalcTableSparseValue::data(
                    "row:west",
                    "col:region",
                    EvalValue::Text(ExcelText::from_interop_assignment("West")),
                ),
            ],
            "SalesTable[Region]",
            "structured-ref:region",
            None,
        )
        .unwrap();

        assert_eq!(before.reader_identity(), after.reader_identity());
    }

    fn assert_update_has(
        scenario: TreeCalcTableUpdateScenarioKind,
        before: &TreeCalcTableNodeProjection,
        after: Option<&TreeCalcTableNodeProjection>,
        expected_reasons: impl IntoIterator<Item = InvalidationReasonKind>,
        expected_identity_inputs: impl IntoIterator<Item = TreeCalcTablePreparedIdentityInput>,
    ) {
        let impact = classify_treecalc_table_update(
            scenario,
            Some(before),
            after,
            [TreeNodeId(30)],
            ["structured-ref:amount".to_string()],
        );
        let expected_reasons = expected_reasons.into_iter().collect::<BTreeSet<_>>();
        let expected_identity_inputs = expected_identity_inputs
            .into_iter()
            .collect::<BTreeSet<_>>();
        assert!(
            expected_reasons.is_subset(&impact.invalidation_reasons),
            "{scenario:?}: expected reasons {expected_reasons:?}, got {:?}",
            impact.invalidation_reasons
        );
        assert!(
            expected_identity_inputs.is_subset(&impact.prepared_identity_inputs),
            "{scenario:?}: expected identity inputs {expected_identity_inputs:?}, got {:?}",
            impact.prepared_identity_inputs
        );
        assert!(
            impact
                .invalidation_seeds
                .iter()
                .all(|seed| seed.node_id == TreeNodeId(30))
        );
    }

    fn assert_column_update(
        scenario: TreeCalcTableUpdateScenarioKind,
        baseline: &TreeCalcTableNodeProjection,
        changed: &TreeCalcTableNodeProjection,
    ) {
        assert_update_has(
            scenario,
            baseline,
            Some(changed),
            [
                InvalidationReasonKind::StructuredTableColumnChanged,
                InvalidationReasonKind::StructuredTableRegionChanged,
                InvalidationReasonKind::StructuredTableContextChanged,
            ],
            [TreeCalcTablePreparedIdentityInput::TableContextIdentity],
        );
    }

    fn amount_column_values() -> Vec<TreeCalcTableSparseValue> {
        vec![
            TreeCalcTableSparseValue::data("row:west", "col:amount", EvalValue::Number(3.0)),
            TreeCalcTableSparseValue::data(
                "row:east",
                "col:amount",
                EvalValue::Text(ExcelText::from_interop_assignment("")),
            ),
        ]
    }

    fn region_values() -> Vec<TreeCalcTableSparseValue> {
        vec![TreeCalcTableSparseValue::data(
            "row:west",
            "col:region",
            EvalValue::Text(ExcelText::from_interop_assignment("West")),
        )]
    }

    fn table_formula_amount_values() -> Vec<TreeCalcTableSparseValue> {
        vec![
            TreeCalcTableSparseValue::data("row:west", "col:amount", EvalValue::Number(10.0)),
            TreeCalcTableSparseValue::data("row:east", "col:amount", EvalValue::Number(20.0)),
            TreeCalcTableSparseValue::data("row:north", "col:amount", EvalValue::Number(30.0)),
        ]
    }

    fn runtime_treecalc_table_snapshot() -> TreeCalcTableNodeSnapshot {
        let mut snapshot = treecalc_table_snapshot();
        snapshot.virtual_anchor.sheet_scope_ref = "sheet:default".to_string();
        snapshot
    }

    fn table_primary_locus(table: &TableDescriptor) -> Locus {
        Locus {
            sheet_id: table.sheet_scope_ref.clone(),
            row: 1,
            col: 1,
        }
    }

    fn table() -> TableDescriptor {
        TableDescriptor {
            table_id: "table:sales".to_string(),
            table_name: "Sales".to_string(),
            workbook_scope_ref: "book:1".to_string(),
            sheet_scope_ref: "sheet:1".to_string(),
            table_range_ref: "A1:C5".to_string(),
            row_membership_identity: Some("rows:sales:membership:v1".to_string()),
            row_order_identity: Some("rows:sales:order:v1".to_string()),
            header_region_ref: Some("A1:C1".to_string()),
            totals_region_ref: Some("A5:C5".to_string()),
            header_row_present: true,
            totals_row_present: true,
            columns: vec![
                TableColumnDescriptor {
                    column_id: "table:sales:col:item".to_string(),
                    column_name: "Item".to_string(),
                    ordinal: 1,
                    column_range_ref: "A2:A4".to_string(),
                },
                TableColumnDescriptor {
                    column_id: "table:sales:col:amount".to_string(),
                    column_name: "Amount".to_string(),
                    ordinal: 2,
                    column_range_ref: "B2:B4".to_string(),
                },
            ],
        }
    }

    fn request(
        reference: StructuredTableReferenceIntake,
    ) -> StructuredTableDependencyLoweringRequest {
        request_with_table(table(), reference)
    }

    fn request_with_table(
        table: TableDescriptor,
        reference: StructuredTableReferenceIntake,
    ) -> StructuredTableDependencyLoweringRequest {
        StructuredTableDependencyLoweringRequest {
            owner_node_id: TreeNodeId(10),
            source_reference_handle: Some("oxfml-structured-ref:1".to_string()),
            context_packet: StructuredTableContextPacket::from_oxfml_table_packet(
                vec![table],
                Some(TableRef {
                    table_id: "table:sales".to_string(),
                }),
                Some(TableCallerRegion {
                    table_id: "table:sales".to_string(),
                    region_kind: TableRegionKind::Data,
                    data_row_offset: Some(2),
                }),
            ),
            reference,
        }
    }

    fn oxfml_bind_record(
        bind_record_handle: &str,
        source_token_text: &str,
        explicit_table_name: Option<&str>,
        omitted_table_name: bool,
        selected_column_ids: impl IntoIterator<Item = String>,
        selected_sections: impl IntoIterator<Item = StructuredSectionKind>,
    ) -> StructuredReferenceBindRecord {
        let selected_sections = selected_sections.into_iter().collect::<Vec<_>>();
        StructuredReferenceBindRecord {
            bind_record_handle: bind_record_handle.to_string(),
            source_span_utf8: TextSpan::new(1, source_token_text.len()),
            source_token_text: source_token_text.to_string(),
            source_token_kind: StructuredReferenceSourceTokenKind::StructuredReference,
            explicit_table_name: explicit_table_name.map(str::to_string),
            omitted_table_name,
            effective_table_id: Some("table:sales".to_string()),
            effective_table_name: Some("Sales".to_string()),
            selected_column_ids: selected_column_ids.into_iter().collect(),
            selected_regions: selected_sections
                .iter()
                .map(|section_kind| StructuredReferenceSelectedRegion {
                    section_kind: *section_kind,
                    region_ref: None,
                    column_range_refs: vec!["B2:B4".to_string()],
                    is_empty: false,
                })
                .collect(),
            selected_sections,
            uses_this_row: false,
            caller_context_dependent: false,
            resolved_reference: None,
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn lowers_available_table_column_data_caller_and_enclosing_facts() {
        let reference = StructuredTableReferenceIntake::omitted_table_name("hostref:table:1")
            .with_selected_columns(["table:sales:col:amount".to_string()])
            .with_selected_regions([
                StructuredTableRegionSelection::Headers,
                StructuredTableRegionSelection::Data,
                StructuredTableRegionSelection::Totals,
            ])
            .with_this_row();

        let lowering = lower_structured_table_dependencies(&request(reference));
        let kinds = lowering
            .facts
            .iter()
            .map(|fact| (fact.kind, fact.status, fact.blocker))
            .collect::<Vec<_>>();

        assert!(kinds.contains(&(
            StructuredTableDependencyFactKind::TableIdentity,
            StructuredTableDependencyFactStatus::Lowered,
            None
        )));
        assert!(kinds.contains(&(
            StructuredTableDependencyFactKind::RowMembership,
            StructuredTableDependencyFactStatus::Lowered,
            None
        )));
        assert!(kinds.contains(&(
            StructuredTableDependencyFactKind::RowOrder,
            StructuredTableDependencyFactStatus::Lowered,
            None
        )));
        assert!(kinds.contains(&(
            StructuredTableDependencyFactKind::ColumnIdentity,
            StructuredTableDependencyFactStatus::Lowered,
            None
        )));
        assert!(kinds.contains(&(
            StructuredTableDependencyFactKind::HeaderText,
            StructuredTableDependencyFactStatus::Lowered,
            None
        )));
        assert!(kinds.contains(&(
            StructuredTableDependencyFactKind::HeaderRegion,
            StructuredTableDependencyFactStatus::Lowered,
            None
        )));
        assert!(kinds.contains(&(
            StructuredTableDependencyFactKind::DataRegion,
            StructuredTableDependencyFactStatus::Lowered,
            None
        )));
        assert!(kinds.contains(&(
            StructuredTableDependencyFactKind::TotalsRegion,
            StructuredTableDependencyFactStatus::Lowered,
            None
        )));
        assert!(kinds.contains(&(
            StructuredTableDependencyFactKind::CallerRowContext,
            StructuredTableDependencyFactStatus::Lowered,
            None
        )));
        assert!(kinds.contains(&(
            StructuredTableDependencyFactKind::OmittedTableNameEnclosingTable,
            StructuredTableDependencyFactStatus::Lowered,
            None
        )));
        assert!(lowering.descriptors.iter().any(
            |descriptor| descriptor.kind == DependencyDescriptorKind::StructuredTableDataRegion
        ));
        assert!(
            lowering.descriptors.iter().any(|descriptor| descriptor.kind
                == DependencyDescriptorKind::StructuredTableRowMembership)
        );
        assert!(
            lowering
                .descriptors
                .iter()
                .any(|descriptor| descriptor.kind
                    == DependencyDescriptorKind::StructuredTableRowOrder)
        );
        assert!(
            lowering.descriptors.iter().any(|descriptor| descriptor.kind
                == DependencyDescriptorKind::StructuredTableHeaderRegion)
        );
        assert!(
            lowering.descriptors.iter().any(|descriptor| descriptor.kind
                == DependencyDescriptorKind::StructuredTableTotalsRegion)
        );
        let details_by_kind = lowering
            .descriptors
            .iter()
            .map(|descriptor| (descriptor.kind, descriptor.carrier_detail.as_str()))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            details_by_kind[&DependencyDescriptorKind::StructuredTableRowMembership],
            "table_row_membership:v1:table=table:sales;identity=rows:sales:membership:v1"
        );
        assert_eq!(
            details_by_kind[&DependencyDescriptorKind::StructuredTableRowOrder],
            "table_row_order:v1:table=table:sales;identity=rows:sales:order:v1"
        );
        assert_eq!(
            details_by_kind[&DependencyDescriptorKind::StructuredTableHeaderRegion],
            "table_header_region:v1:table=table:sales;region=A1:C1"
        );
        assert_eq!(
            details_by_kind[&DependencyDescriptorKind::StructuredTableTotalsRegion],
            "table_totals_region:v1:table=table:sales;region=A5:C5"
        );
        assert!(
            lowering
                .table_context_identity
                .contains("rows:sales:membership:v1")
        );
        assert!(
            lowering
                .table_context_identity
                .contains("rows:sales:order:v1")
        );
        assert!(lowering.table_context_identity.contains("A1:C1"));
        assert!(lowering.table_context_identity.contains("A5:C5"));
        assert!(
            lowering
                .descriptors
                .iter()
                .all(|descriptor| descriptor.target_node_id.is_none()
                    && descriptor.requires_rebind_on_structural_change)
        );
    }

    #[test]
    fn records_missing_row_membership_order_and_region_shape_as_blockers() {
        let reference =
            StructuredTableReferenceIntake::explicit_table("hostref:table:2", "table:sales")
                .with_selected_regions([
                    StructuredTableRegionSelection::Headers,
                    StructuredTableRegionSelection::Totals,
                ]);
        let mut table = table();
        table.row_membership_identity = None;
        table.row_order_identity = None;
        table.header_region_ref = None;
        table.totals_region_ref = None;

        let lowering = lower_structured_table_dependencies(&request_with_table(table, reference));
        let blockers = lowering
            .blocked_facts()
            .into_iter()
            .map(|fact| (fact.kind, fact.blocker.expect("blocked fact has blocker")))
            .collect::<BTreeSet<_>>();

        assert!(blockers.contains(&(
            StructuredTableDependencyFactKind::RowMembership,
            StructuredTableLoweringBlocker::MissingStableRowMembershipAndOrderPacket
        )));
        assert!(blockers.contains(&(
            StructuredTableDependencyFactKind::RowOrder,
            StructuredTableLoweringBlocker::MissingStableRowMembershipAndOrderPacket
        )));
        assert!(blockers.contains(&(
            StructuredTableDependencyFactKind::HeaderRegion,
            StructuredTableLoweringBlocker::MissingHeaderRegionRange
        )));
        assert!(blockers.contains(&(
            StructuredTableDependencyFactKind::TotalsRegion,
            StructuredTableLoweringBlocker::MissingTotalsRegionRange
        )));
    }

    #[test]
    fn context_only_table_descriptors_do_not_create_graph_diagnostics() {
        let reference =
            StructuredTableReferenceIntake::explicit_table("hostref:table:3", "table:sales")
                .with_selected_columns(["table:sales:col:item".to_string()])
                .with_selected_regions([StructuredTableRegionSelection::Data]);
        let lowering = lower_structured_table_dependencies(&request(reference));
        let snapshot = StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            [
                StructuralNode {
                    node_id: TreeNodeId(1),
                    kind: StructuralNodeKind::Root,
                    symbol: "Root".to_string(),
                    parent_id: None,
                    child_ids: vec![TreeNodeId(10)],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(10),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "Total".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
            ],
        )
        .unwrap();

        let graph = DependencyGraph::build(&snapshot, &lowering.descriptors);

        assert!(graph.diagnostics.is_empty());
        assert_eq!(graph.edges_by_owner.len(), 0);
        assert_eq!(graph.descriptors_by_owner[&TreeNodeId(10)].len(), 5);
    }

    #[test]
    fn maps_explicit_oxfml_bind_record_to_table_lowering_request() {
        let record = oxfml_bind_record(
            "structured-ref:explicit-all",
            "Sales[[#All],[Amount]]",
            Some("Sales"),
            false,
            ["table:sales:col:amount".to_string()],
            [StructuredSectionKind::All],
        );

        let request = StructuredTableDependencyLoweringRequest::from_oxfml_bind_record(
            TreeNodeId(10),
            StructuredTableContextPacket::from_oxfml_table_packet(
                vec![table()],
                Some(TableRef {
                    table_id: "table:sales".to_string(),
                }),
                Some(TableCallerRegion {
                    table_id: "table:sales".to_string(),
                    region_kind: TableRegionKind::Data,
                    data_row_offset: Some(1),
                }),
            ),
            &record,
        )
        .expect("resolved structured bind record maps to intake");

        assert_eq!(
            request.reference.explicit_table_ref,
            Some(TableRef {
                table_id: "table:sales".to_string()
            })
        );
        assert_eq!(
            request.reference.source_token_kind,
            StructuredReferenceSourceTokenKind::StructuredReference
        );
        assert_eq!(
            request.source_reference_handle.as_deref(),
            Some("structured-ref:explicit-all")
        );
        assert!(
            request
                .reference
                .selected_regions
                .contains(&StructuredTableRegionSelection::All)
        );

        let lowering = lower_structured_table_dependencies(&request);

        assert!(
            lowering
                .descriptors
                .iter()
                .all(|descriptor| descriptor.source_reference_handle.as_deref()
                    == Some("structured-ref:explicit-all"))
        );
        assert!(
            lowering.descriptors.iter().any(|descriptor| descriptor.kind
                == DependencyDescriptorKind::StructuredTableHeaderRegion)
        );
        assert!(
            lowering.descriptors.iter().any(|descriptor| descriptor.kind
                == DependencyDescriptorKind::StructuredTableTotalsRegion)
        );
    }

    #[test]
    fn all_section_omits_absent_optional_table_regions_without_blocking() {
        let record = oxfml_bind_record(
            "structured-ref:explicit-all-no-totals",
            "Sales[[#All],[Amount]]",
            Some("Sales"),
            false,
            ["table:sales:col:amount".to_string()],
            [StructuredSectionKind::All],
        );
        let mut table = table();
        table.totals_row_present = false;
        table.totals_region_ref = None;

        let request = StructuredTableDependencyLoweringRequest::from_oxfml_bind_record(
            TreeNodeId(10),
            StructuredTableContextPacket::from_oxfml_table_packet(
                vec![table],
                Some(TableRef {
                    table_id: "table:sales".to_string(),
                }),
                Some(TableCallerRegion {
                    table_id: "table:sales".to_string(),
                    region_kind: TableRegionKind::Data,
                    data_row_offset: Some(1),
                }),
            ),
            &record,
        )
        .expect("resolved #All bind record maps to intake");

        let lowering = lower_structured_table_dependencies(&request);

        assert!(lowering.blocked_facts().is_empty());
        assert!(
            lowering.descriptors.iter().any(|descriptor| descriptor.kind
                == DependencyDescriptorKind::StructuredTableHeaderRegion)
        );
        assert!(lowering.descriptors.iter().any(
            |descriptor| descriptor.kind == DependencyDescriptorKind::StructuredTableDataRegion
        ));
        assert!(
            !lowering.descriptors.iter().any(|descriptor| descriptor.kind
                == DependencyDescriptorKind::StructuredTableTotalsRegion)
        );
    }

    #[test]
    fn maps_omitted_this_row_oxfml_bind_record_to_caller_and_enclosing_context() {
        let mut record = oxfml_bind_record(
            "structured-ref:omitted-this-row",
            "[@Amount]",
            None,
            true,
            ["table:sales:col:amount".to_string()],
            [StructuredSectionKind::ThisRow],
        );
        record.uses_this_row = true;
        record.caller_context_dependent = true;

        let request = StructuredTableDependencyLoweringRequest::from_oxfml_bind_record(
            TreeNodeId(10),
            StructuredTableContextPacket::from_oxfml_table_packet(
                vec![table()],
                Some(TableRef {
                    table_id: "table:sales".to_string(),
                }),
                Some(TableCallerRegion {
                    table_id: "table:sales".to_string(),
                    region_kind: TableRegionKind::Data,
                    data_row_offset: Some(2),
                }),
            ),
            &record,
        )
        .expect("omitted table-name record maps to intake");

        assert_eq!(request.reference.explicit_table_ref, None);
        assert_eq!(
            request.reference.effective_table_ref,
            Some(TableRef {
                table_id: "table:sales".to_string()
            })
        );
        assert!(request.reference.uses_omitted_table_name);
        assert!(request.reference.uses_this_row);
        assert!(
            request
                .reference
                .selected_regions
                .contains(&StructuredTableRegionSelection::Data)
        );

        let lowering = lower_structured_table_dependencies(&request);
        let lowered_kinds = lowering
            .facts
            .iter()
            .filter(|fact| fact.status == StructuredTableDependencyFactStatus::Lowered)
            .map(|fact| fact.kind)
            .collect::<BTreeSet<_>>();

        assert!(lowered_kinds.contains(&StructuredTableDependencyFactKind::CallerRowContext));
        assert!(
            lowered_kinds
                .contains(&StructuredTableDependencyFactKind::OmittedTableNameEnclosingTable)
        );
    }

    #[test]
    fn omitted_bind_record_uses_effective_table_and_blocks_enclosing_mismatch() {
        let record = oxfml_bind_record(
            "structured-ref:omitted-mismatch",
            "[Amount]",
            None,
            true,
            ["table:sales:col:amount".to_string()],
            [StructuredSectionKind::Data],
        );
        let mut other_table = table();
        other_table.table_id = "table:inventory".to_string();
        other_table.table_name = "Inventory".to_string();

        let request = StructuredTableDependencyLoweringRequest::from_oxfml_bind_record(
            TreeNodeId(10),
            StructuredTableContextPacket::from_oxfml_table_packet(
                vec![table(), other_table],
                Some(TableRef {
                    table_id: "table:inventory".to_string(),
                }),
                None,
            ),
            &record,
        )
        .expect("omitted table-name record keeps its effective table target");

        assert_eq!(request.reference.explicit_table_ref, None);
        assert_eq!(
            request.reference.effective_table_ref,
            Some(TableRef {
                table_id: "table:sales".to_string()
            })
        );

        let lowering = lower_structured_table_dependencies(&request);

        assert!(lowering.facts.iter().any(|fact| {
            fact.kind == StructuredTableDependencyFactKind::TableIdentity
                && fact.status == StructuredTableDependencyFactStatus::Lowered
                && fact.table_id.as_deref() == Some("table:sales")
        }));
        assert!(!lowering.facts.iter().any(|fact| {
            fact.kind == StructuredTableDependencyFactKind::TableIdentity
                && fact.status == StructuredTableDependencyFactStatus::Lowered
                && fact.table_id.as_deref() == Some("table:inventory")
        }));
        assert!(lowering.facts.iter().any(|fact| {
            fact.kind == StructuredTableDependencyFactKind::OmittedTableNameEnclosingTable
                && fact.status == StructuredTableDependencyFactStatus::Blocked
                && fact.blocker
                    == Some(StructuredTableLoweringBlocker::OmittedTableEnclosingMismatch)
        }));
    }

    #[test]
    fn rejects_diagnostic_oxfml_bind_record_before_dependency_lowering() {
        let mut record = oxfml_bind_record(
            "structured-ref:missing",
            "Missing[Amount]",
            Some("Missing"),
            false,
            [],
            [StructuredSectionKind::Data],
        );
        record.effective_table_id = None;
        record.diagnostics = vec![StructuredReferenceBindDiagnosticLink {
            diagnostic_code: "structured_reference_bind_error".to_string(),
            message: "unknown table".to_string(),
            source_span_utf8: TextSpan::new(1, "Missing[Amount]".len()),
        }];

        let error = StructuredTableDependencyLoweringRequest::from_oxfml_bind_record(
            TreeNodeId(10),
            StructuredTableContextPacket::from_oxfml_table_packet(vec![table()], None, None),
            &record,
        )
        .expect_err("diagnostic bind records are not dependency-lowering inputs");

        assert_eq!(
            error,
            StructuredTableBindRecordIntakeError::UnresolvedStructuredReference {
                bind_record_handle: "structured-ref:missing".to_string(),
                source_token_text: "Missing[Amount]".to_string(),
                diagnostic_codes: vec!["structured_reference_bind_error".to_string()],
            }
        );
    }

    #[test]
    fn omitted_table_name_requires_enclosing_table_ref() {
        let mut request = request(StructuredTableReferenceIntake::omitted_table_name(
            "hostref:table:4",
        ));
        request.context_packet.enclosing_table_ref = None;

        let lowering = lower_structured_table_dependencies(&request);

        assert_eq!(lowering.descriptors.len(), 0);
        assert_eq!(lowering.facts.len(), 1);
        assert_eq!(
            lowering.facts[0].blocker,
            Some(StructuredTableLoweringBlocker::MissingEnclosingTableContext)
        );
    }
}
