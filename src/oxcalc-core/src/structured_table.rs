#![forbid(unsafe_code)]

//! OxCalc-owned structured table dependency lowering.
//!
//! This module consumes public OxFml table-context packets. It does not parse
//! structured-reference formula text and does not mirror OxFml grammar.

use std::collections::{BTreeMap, BTreeSet};

pub use oxfml_core::interface::{
    TableCallerRegion, TableColumnDescriptor, TableDescriptor, TableRef, TableRegionKind,
};
use oxfml_core::{
    StructuredReferenceBindDiagnosticLink, StructuredReferenceBindRecord,
    StructuredReferenceSelectedRegion, StructuredSectionKind, syntax::token::TextSpan,
};

use crate::dependency::{DependencyDescriptor, DependencyDescriptorKind};
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
    EmptyDataBodyNotRepresentableByCurrentTableDescriptor,
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
    let table_end_row = snapshot
        .virtual_anchor
        .start_row
        .checked_add(total_rows - 1)
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
    let data_end_row = data_start_row
        .checked_add(row_count - 1)
        .ok_or(TreeCalcTableProjectionError::RangeOverflow)?;

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
                column_range_ref: a1_range_ref(data_start_row, col, data_end_row, col)?,
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
    if snapshot.rows.is_empty() {
        return Err(
            TreeCalcTableProjectionError::EmptyDataBodyNotRepresentableByCurrentTableDescriptor,
        );
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
            resolve_treecalc_structured_column(
                column_name,
                table,
                tail_span,
                &mut selected_column_ids,
                &mut diagnostics,
            );
            continue;
        }
        resolve_treecalc_structured_column(
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
                            .map(|column| column.column_range_ref.clone())
                    })
                })
                .collect(),
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
    let normalized = column_name.replace("]]", "]").trim().to_ascii_uppercase();
    if let Some(column) = table.columns.iter().find(|column| {
        column.column_id.eq_ignore_ascii_case(&normalized)
            || column.column_name.to_ascii_uppercase() == normalized
    }) {
        selected_column_ids.push(column.column_id.clone());
    } else {
        diagnostics.push(treecalc_table_structured_diagnostic(
            "treecalc.table.unknown_column",
            format!(
                "structured reference column '{}' is not present in table '{}'",
                column_name, table.table_name
            ),
            tail_span,
        ));
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
pub enum StructuredTableRegionSelection {
    All,
    Headers,
    Data,
    Totals,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredTableReferenceIntake {
    pub reference_handle: String,
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
                || record.caller_context_dependent
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
    use oxfml_core::{
        StructuredReferenceBindDiagnosticLink, StructuredReferenceSelectedRegion,
        interface::{TableColumnDescriptor, TableRegionKind},
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
    fn table_projection_rejects_snapshot_shapes_that_would_create_private_semantics() {
        let mut empty_data_body = treecalc_table_snapshot();
        empty_data_body.rows.clear();
        assert_eq!(
            project_treecalc_table_node_snapshot(&empty_data_body).unwrap_err(),
            TreeCalcTableProjectionError::EmptyDataBodyNotRepresentableByCurrentTableDescriptor
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
