#![forbid(unsafe_code)]

//! OxCalc-owned structured table dependency lowering.
//!
//! This module consumes public OxFml table-context packets. It does not parse
//! structured-reference formula text and does not mirror OxFml grammar.

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use oxfml_core::binding::{
    AreaRef, BindContext, BindRequest, CellRef, StructuredResolvedRef, bind_formula,
};
use oxfml_core::consumer::runtime::{
    RuntimeDryBindVerdict, RuntimeEnvironment, RuntimeFormulaRequest, RuntimeHostFormulaContext,
};
pub use oxfml_core::interface::{
    TableCallerRegion, TableColumnDescriptor, TableDescriptor, TableRef, TableRegionKind,
};
use oxfml_core::{
    EvaluationBackend, StructuredReferenceBindRecord, StructuredReferenceSourceTokenKind,
    StructuredSectionKind,
    red::project_red_view,
    seam::Locus,
    source::{FormulaSourceRecord, StructureContextVersion},
    syntax::parser::{ParseRequest, parse_formula},
    syntax::token::TextSpan,
};
use oxfunc_core::registry::{CapabilityOverlay, FunctionRegistry, builtin_registry};
use oxfunc_core::value::{CalcValue, ExcelText, ReferenceKind, ReferenceLike};

use crate::dependency::{
    DependencyDescriptor, DependencyDescriptorKind, InvalidationReasonKind, InvalidationSeed,
};
use crate::formula::{
    TreeCalcCrossWorkspaceAvailabilityPacket, TreeCalcCrossWorkspaceAvailabilityStatus,
};
use crate::sparse_reader::{
    SparseCellCoord, SparseCellRead, SparseDefinedCell, SparseRangeExtent, SparseRangeReader,
    SparseReaderAccessSummary, SparseReaderIdentity,
};
use crate::structural::TreeNodeId;
use crate::tree_reference_system::{
    TreeCalcReferenceSystemProvider, TreeCalcSparseReferenceCell,
    TreeCalcSparseReferenceValuesBinding,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcTableVirtualAnchor {
    pub workbook_scope_ref: String,
    pub sheet_scope_ref: String,
    pub start_row: u32,
    pub start_col: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TreeCalcTableRowId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcTableFormulaMetadata {
    pub formula_artifact_id: String,
    pub bind_artifact_id: Option<String>,
    pub formula_text_version: String,
    #[serde(default)]
    pub formula_text: String,
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
                ("formula_text", self.formula_text.clone()),
            ],
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcTableColumnSnapshot {
    pub column_id: String,
    pub column_name: String,
    pub ordinal: u32,
    pub body_metadata: TreeCalcTableColumnBodyMetadata,
    pub totals_metadata: Option<TreeCalcTableFormulaMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcTableBodyCellNodeBinding {
    pub row_id: TreeCalcTableRowId,
    pub column_id: String,
    pub node_id: TreeNodeId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcTableTotalsCellNodeBinding {
    pub column_id: String,
    pub node_id: TreeNodeId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcTableNodeSnapshot {
    pub table_node_id: TreeNodeId,
    pub table_id: String,
    pub table_name: String,
    pub display_path: String,
    pub canonical_path: String,
    pub virtual_anchor: TreeCalcTableVirtualAnchor,
    pub rows: Vec<TreeCalcTableRowId>,
    pub columns: Vec<TreeCalcTableColumnSnapshot>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub body_cell_nodes: Vec<TreeCalcTableBodyCellNodeBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub totals_cell_nodes: Vec<TreeCalcTableTotalsCellNodeBinding>,
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
    pub table_namespace_version: String,
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

impl crate::table_backing::TableBacking for TreeCalcTableNodeProjection {
    fn table_spec(&self) -> crate::table_backing::TableSpec {
        crate::table_backing::TableSpec::new(self.table_descriptor.clone())
    }
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
    DuplicateBodyCellNodeBinding { row_id: String, column_id: String },
    UnknownBodyCellRowId { row_id: String },
    UnknownBodyCellColumnId { column_id: String },
    DuplicateTotalsCellNodeBinding { column_id: String },
    UnknownTotalsCellColumnId { column_id: String },
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
    let body_cell_node_identity = treecalc_table_body_cell_node_identity(snapshot);
    let totals_cell_node_identity = treecalc_table_totals_cell_node_identity(snapshot);
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
            ("body_cell_nodes", body_cell_node_identity.clone()),
            ("totals", totals_metadata_identity.clone()),
            ("totals_cell_nodes", totals_cell_node_identity.clone()),
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
            ("body_cell_nodes", body_cell_node_identity),
            ("totals_token", totals_metadata_token.clone()),
            ("totals_cell_nodes", totals_cell_node_identity),
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
        table_namespace_version: snapshot.table_namespace_version.clone(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TreeCalcTableCatalogSelectorKind {
    TableNameOrPath,
    SameNodeTable,
    OmittedTableName,
    StableTableId,
}

impl TreeCalcTableCatalogSelectorKind {
    #[must_use]
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::TableNameOrPath => "table_name_or_path",
            Self::SameNodeTable => "same_node_table",
            Self::OmittedTableName => "omitted_table_name",
            Self::StableTableId => "stable_table_id",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TreeCalcTableCatalogResolutionLayer {
    CurrentWorkspaceTableName,
    CurrentWorkspacePath,
    CurrentWorkspaceRoot,
    SameNodeTable,
    OmittedCallerTable,
    WorkspaceAlias,
    DirectWorkspace,
    StableTableId,
    UnavailableWorkspace,
    DeletedTable,
    Unresolved,
}

impl TreeCalcTableCatalogResolutionLayer {
    #[must_use]
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::CurrentWorkspaceTableName => "current_workspace_table_name",
            Self::CurrentWorkspacePath => "current_workspace_path",
            Self::CurrentWorkspaceRoot => "current_workspace_root",
            Self::SameNodeTable => "same_node_table",
            Self::OmittedCallerTable => "omitted_caller_table",
            Self::WorkspaceAlias => "workspace_alias",
            Self::DirectWorkspace => "direct_workspace",
            Self::StableTableId => "stable_table_id",
            Self::UnavailableWorkspace => "unavailable_workspace",
            Self::DeletedTable => "deleted_table",
            Self::Unresolved => "unresolved",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TreeCalcTableCatalogShapeHint {
    Table,
    CallerContextTable,
    UnavailableWorkspace,
    DeletedTable,
    Unresolved,
}

impl TreeCalcTableCatalogShapeHint {
    #[must_use]
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::Table => "table",
            Self::CallerContextTable => "caller_context_table",
            Self::UnavailableWorkspace => "unavailable_workspace",
            Self::DeletedTable => "deleted_table",
            Self::Unresolved => "unresolved",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TreeCalcTableCatalogDiagnosticCode {
    TableNotFound,
    AmbiguousTableSelector,
    MissingCallerTableContext,
    MissingCallerNodeContext,
    WorkspaceUnavailable,
    TableDeleted,
    HostNameAdjacencyW074Mapping,
    FunctionNameAdjacencyW074Mapping,
    DefinedNameAdjacencyW074Mapping,
    LambdaValuedNodeAdjacencyW074Mapping,
}

impl TreeCalcTableCatalogDiagnosticCode {
    #[must_use]
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::TableNotFound => "treecalc.table_catalog.table_not_found",
            Self::AmbiguousTableSelector => "treecalc.table_catalog.ambiguous_table_selector",
            Self::MissingCallerTableContext => {
                "treecalc.table_catalog.missing_caller_table_context"
            }
            Self::MissingCallerNodeContext => "treecalc.table_catalog.missing_caller_node_context",
            Self::WorkspaceUnavailable => "treecalc.table_catalog.workspace_unavailable",
            Self::TableDeleted => "treecalc.table_catalog.table_deleted",
            Self::HostNameAdjacencyW074Mapping => {
                "treecalc.table_catalog.host_name_adjacency_w074_mapping"
            }
            Self::FunctionNameAdjacencyW074Mapping => {
                "treecalc.table_catalog.function_name_adjacency_w074_mapping"
            }
            Self::DefinedNameAdjacencyW074Mapping => {
                "treecalc.table_catalog.defined_name_adjacency_w074_mapping"
            }
            Self::LambdaValuedNodeAdjacencyW074Mapping => {
                "treecalc.table_catalog.lambda_valued_node_adjacency_w074_mapping"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableCatalogDiagnostic {
    pub code: TreeCalcTableCatalogDiagnosticCode,
    pub message: String,
    pub source_span_utf8: Option<TextSpan>,
    pub selector_token_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableCatalogResolveRequest {
    pub selector_token_text: String,
    pub selector_kind: TreeCalcTableCatalogSelectorKind,
    pub source_span_utf8: Option<TextSpan>,
    pub caller_node_id: Option<TreeNodeId>,
    pub caller_table_region: Option<TableCallerRegion>,
    pub caller_context_id: Option<String>,
}

impl TreeCalcTableCatalogResolveRequest {
    #[must_use]
    pub fn table_name_or_path(selector_token_text: impl Into<String>) -> Self {
        Self {
            selector_token_text: selector_token_text.into(),
            selector_kind: TreeCalcTableCatalogSelectorKind::TableNameOrPath,
            source_span_utf8: None,
            caller_node_id: None,
            caller_table_region: None,
            caller_context_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableCatalogWorkspace {
    pub workspace_handle: String,
    pub availability_packet: TreeCalcCrossWorkspaceAvailabilityPacket,
    pub table_projections: Vec<TreeCalcTableNodeProjection>,
}

impl TreeCalcTableCatalogWorkspace {
    #[must_use]
    pub fn available_current(
        workspace_handle: impl Into<String>,
        table_projections: Vec<TreeCalcTableNodeProjection>,
    ) -> Self {
        let workspace_handle = workspace_handle.into();
        Self {
            availability_packet: TreeCalcCrossWorkspaceAvailabilityPacket::available(
                workspace_handle.clone(),
                "current",
                format!("treecalc-table-workspace-availability:v1:{workspace_handle}:available"),
            ),
            workspace_handle,
            table_projections,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcTableDeletedFact {
    pub workspace_handle: String,
    pub table_id: String,
    pub selector_token_text: String,
    pub table_namespace_version: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeCalcTableNamespaceAdjacency {
    pub host_names: BTreeSet<String>,
    pub function_names: BTreeSet<String>,
    pub defined_names: BTreeSet<String>,
    pub lambda_valued_node_names: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableCatalogResolverContext {
    pub current_workspace_handle: String,
    pub host_namespace_version: String,
    pub structure_context_version: String,
    pub resolution_rule_version: String,
    pub workspaces: Vec<TreeCalcTableCatalogWorkspace>,
    pub workspace_aliases: BTreeMap<String, String>,
    pub namespace_adjacency: TreeCalcTableNamespaceAdjacency,
    pub deleted_tables: Vec<TreeCalcTableDeletedFact>,
}

impl TreeCalcTableCatalogResolverContext {
    #[must_use]
    pub fn for_current_workspace(
        current_workspace_handle: impl Into<String>,
        table_projections: Vec<TreeCalcTableNodeProjection>,
    ) -> Self {
        let current_workspace_handle = current_workspace_handle.into();
        Self {
            current_workspace_handle: current_workspace_handle.clone(),
            host_namespace_version: "treecalc-host-namespace:v1".to_string(),
            structure_context_version: "treecalc-structure:v1".to_string(),
            resolution_rule_version: "treecalc-host-resolution:v1".to_string(),
            workspaces: vec![TreeCalcTableCatalogWorkspace::available_current(
                current_workspace_handle,
                table_projections,
            )],
            workspace_aliases: BTreeMap::new(),
            namespace_adjacency: TreeCalcTableNamespaceAdjacency::default(),
            deleted_tables: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_workspace(mut self, workspace: TreeCalcTableCatalogWorkspace) -> Self {
        self.workspaces.push(workspace);
        self
    }

    #[must_use]
    pub fn with_workspace_alias(
        mut self,
        alias: impl Into<String>,
        workspace_handle: impl Into<String>,
    ) -> Self {
        self.workspace_aliases
            .insert(alias.into(), workspace_handle.into());
        self
    }

    #[must_use]
    pub fn with_namespace_adjacency(
        mut self,
        namespace_adjacency: TreeCalcTableNamespaceAdjacency,
    ) -> Self {
        self.namespace_adjacency = namespace_adjacency;
        self
    }

    #[must_use]
    pub fn with_deleted_table(mut self, deleted_table: TreeCalcTableDeletedFact) -> Self {
        self.deleted_tables.push(deleted_table);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableCatalogResolution {
    pub table_reference_handle: String,
    pub source_span_utf8: Option<TextSpan>,
    pub source_token_text: String,
    pub opaque_selector: String,
    pub resolution_layer: TreeCalcTableCatalogResolutionLayer,
    pub shape_hint: TreeCalcTableCatalogShapeHint,
    pub effective_table_id: Option<String>,
    pub table_node_id: Option<TreeNodeId>,
    pub virtual_anchor_identity: Option<String>,
    pub caller_context_dependency: bool,
    pub caller_context_id: Option<String>,
    pub host_namespace_version: String,
    pub table_namespace_version: Option<String>,
    pub structure_context_version: String,
    pub resolution_rule_version: String,
    pub workspace_availability_version: Option<String>,
    pub diagnostics: Vec<TreeCalcTableCatalogDiagnostic>,
}

pub fn resolve_treecalc_table_catalog_reference(
    context: &TreeCalcTableCatalogResolverContext,
    request: &TreeCalcTableCatalogResolveRequest,
) -> TreeCalcTableCatalogResolution {
    let selector_token_text = request.selector_token_text.trim().to_string();
    let parsed_selector = parse_treecalc_table_catalog_workspace_selector(
        context,
        &selector_token_text,
        request.selector_kind,
    );
    let caller_context_dependency = treecalc_table_catalog_caller_context_dependency(request);
    let caller_context_id =
        caller_context_dependency.then(|| treecalc_table_catalog_caller_context_identity(request));

    let mut diagnostics = Vec::new();
    let mut resolution_layer = parsed_selector.resolution_layer;
    let mut shape_hint = TreeCalcTableCatalogShapeHint::Unresolved;
    let mut effective_projection = None;
    let mut deleted_namespace_version = None;

    let workspace = context
        .workspaces
        .iter()
        .find(|workspace| workspace.workspace_handle == parsed_selector.workspace_handle);

    let availability_packet = workspace.map_or_else(
        || {
            TreeCalcCrossWorkspaceAvailabilityPacket::unavailable(
                parsed_selector.workspace_handle.clone(),
                parsed_selector
                    .workspace_selector_token
                    .clone()
                    .unwrap_or_else(|| parsed_selector.workspace_handle.clone()),
                format!(
                    "treecalc-table-workspace-availability:v1:{}:unavailable",
                    parsed_selector.workspace_handle
                ),
                "workspace was not present in the TreeCalc table catalog resolver context",
            )
        },
        |workspace| workspace.availability_packet.clone(),
    );

    if availability_packet.status != TreeCalcCrossWorkspaceAvailabilityStatus::Available {
        diagnostics.push(treecalc_table_catalog_diagnostic(
            TreeCalcTableCatalogDiagnosticCode::WorkspaceUnavailable,
            format!(
                "workspace '{}' is not available for table lookup",
                availability_packet.workspace_selector_token
            ),
            request,
        ));
        resolution_layer = TreeCalcTableCatalogResolutionLayer::UnavailableWorkspace;
        shape_hint = TreeCalcTableCatalogShapeHint::UnavailableWorkspace;
    } else if let Some(workspace) = workspace {
        match request.selector_kind {
            TreeCalcTableCatalogSelectorKind::SameNodeTable => {
                if let Some(caller_node_id) = request.caller_node_id {
                    effective_projection = workspace
                        .table_projections
                        .iter()
                        .find(|projection| projection.table_node_id == caller_node_id);
                    if effective_projection.is_none() {
                        diagnostics.push(treecalc_table_catalog_diagnostic(
                            TreeCalcTableCatalogDiagnosticCode::TableNotFound,
                            format!(
                                "caller node '{caller_node_id}' does not own a table projection"
                            ),
                            request,
                        ));
                    }
                } else {
                    diagnostics.push(treecalc_table_catalog_diagnostic(
                        TreeCalcTableCatalogDiagnosticCode::MissingCallerNodeContext,
                        "same-node table lookup requires a caller node id".to_string(),
                        request,
                    ));
                }
            }
            TreeCalcTableCatalogSelectorKind::OmittedTableName => {
                if let Some(caller_table_region) = request.caller_table_region.as_ref() {
                    effective_projection = workspace
                        .table_projections
                        .iter()
                        .find(|projection| projection.table_id == caller_table_region.table_id);
                    if effective_projection.is_none() {
                        diagnostics.push(treecalc_table_catalog_diagnostic(
                            TreeCalcTableCatalogDiagnosticCode::TableNotFound,
                            format!(
                                "caller table '{}' is not present in the table catalog",
                                caller_table_region.table_id
                            ),
                            request,
                        ));
                    }
                } else {
                    diagnostics.push(treecalc_table_catalog_diagnostic(
                        TreeCalcTableCatalogDiagnosticCode::MissingCallerTableContext,
                        "omitted-table structured reference requires caller table context"
                            .to_string(),
                        request,
                    ));
                }
            }
            TreeCalcTableCatalogSelectorKind::StableTableId => {
                effective_projection = workspace
                    .table_projections
                    .iter()
                    .find(|projection| projection.table_id == selector_token_text);
                if effective_projection.is_none() {
                    diagnostics.push(treecalc_table_catalog_diagnostic(
                        TreeCalcTableCatalogDiagnosticCode::TableNotFound,
                        format!("stable table id '{selector_token_text}' did not resolve"),
                        request,
                    ));
                }
            }
            TreeCalcTableCatalogSelectorKind::TableNameOrPath => {
                let matches = treecalc_table_catalog_matches(
                    &workspace.table_projections,
                    &parsed_selector.local_selector_token,
                );
                match matches.as_slice() {
                    [projection] => {
                        effective_projection = Some(*projection);
                        resolution_layer =
                            treecalc_table_catalog_explicit_layer(projection, &parsed_selector);
                    }
                    [] => {
                        if let Some(deleted_table) = treecalc_table_catalog_deleted_match(
                            context,
                            &parsed_selector.workspace_handle,
                            &parsed_selector.local_selector_token,
                        ) {
                            diagnostics.push(treecalc_table_catalog_diagnostic(
                                TreeCalcTableCatalogDiagnosticCode::TableDeleted,
                                format!(
                                    "table selector '{}' refers to deleted table '{}'",
                                    parsed_selector.local_selector_token, deleted_table.table_id
                                ),
                                request,
                            ));
                            deleted_namespace_version =
                                Some(deleted_table.table_namespace_version.clone());
                            resolution_layer = TreeCalcTableCatalogResolutionLayer::DeletedTable;
                            shape_hint = TreeCalcTableCatalogShapeHint::DeletedTable;
                        } else {
                            diagnostics.push(treecalc_table_catalog_diagnostic(
                                TreeCalcTableCatalogDiagnosticCode::TableNotFound,
                                format!(
                                    "table selector '{}' did not resolve",
                                    parsed_selector.local_selector_token
                                ),
                                request,
                            ));
                        }
                    }
                    _ => {
                        diagnostics.push(treecalc_table_catalog_diagnostic(
                            TreeCalcTableCatalogDiagnosticCode::AmbiguousTableSelector,
                            format!(
                                "table selector '{}' resolved to {} table projections",
                                parsed_selector.local_selector_token,
                                matches.len()
                            ),
                            request,
                        ));
                    }
                }
            }
        }
    }

    if let Some(projection) = effective_projection {
        shape_hint = if caller_context_dependency {
            TreeCalcTableCatalogShapeHint::CallerContextTable
        } else {
            TreeCalcTableCatalogShapeHint::Table
        };
        if matches!(
            request.selector_kind,
            TreeCalcTableCatalogSelectorKind::SameNodeTable
        ) {
            resolution_layer = TreeCalcTableCatalogResolutionLayer::SameNodeTable;
        } else if matches!(
            request.selector_kind,
            TreeCalcTableCatalogSelectorKind::OmittedTableName
        ) {
            resolution_layer = TreeCalcTableCatalogResolutionLayer::OmittedCallerTable;
        } else if matches!(
            request.selector_kind,
            TreeCalcTableCatalogSelectorKind::StableTableId
        ) {
            resolution_layer = TreeCalcTableCatalogResolutionLayer::StableTableId;
        }
        diagnostics.extend(treecalc_table_catalog_adjacency_diagnostics(
            context,
            &parsed_selector.local_selector_token,
            request,
        ));
        build_treecalc_table_catalog_resolution(
            context,
            request,
            parsed_selector,
            availability_packet,
            resolution_layer,
            shape_hint,
            Some(projection),
            caller_context_dependency,
            caller_context_id,
            diagnostics,
            None,
        )
    } else {
        diagnostics.extend(treecalc_table_catalog_adjacency_diagnostics(
            context,
            &parsed_selector.local_selector_token,
            request,
        ));
        build_treecalc_table_catalog_resolution(
            context,
            request,
            parsed_selector,
            availability_packet,
            resolution_layer,
            shape_hint,
            None,
            caller_context_dependency,
            caller_context_id,
            diagnostics,
            deleted_namespace_version,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedTreeCalcTableCatalogWorkspaceSelector {
    workspace_selector_token: Option<String>,
    local_selector_token: String,
    workspace_handle: String,
    resolution_layer: TreeCalcTableCatalogResolutionLayer,
}

fn parse_treecalc_table_catalog_workspace_selector(
    context: &TreeCalcTableCatalogResolverContext,
    selector_token_text: &str,
    selector_kind: TreeCalcTableCatalogSelectorKind,
) -> ParsedTreeCalcTableCatalogWorkspaceSelector {
    if !matches!(
        selector_kind,
        TreeCalcTableCatalogSelectorKind::TableNameOrPath
    ) {
        return ParsedTreeCalcTableCatalogWorkspaceSelector {
            workspace_selector_token: None,
            local_selector_token: selector_token_text.to_string(),
            workspace_handle: context.current_workspace_handle.clone(),
            resolution_layer: match selector_kind {
                TreeCalcTableCatalogSelectorKind::SameNodeTable => {
                    TreeCalcTableCatalogResolutionLayer::SameNodeTable
                }
                TreeCalcTableCatalogSelectorKind::OmittedTableName => {
                    TreeCalcTableCatalogResolutionLayer::OmittedCallerTable
                }
                TreeCalcTableCatalogSelectorKind::StableTableId => {
                    TreeCalcTableCatalogResolutionLayer::StableTableId
                }
                TreeCalcTableCatalogSelectorKind::TableNameOrPath => unreachable!(),
            },
        };
    }

    if let Some(local_selector_token) = selector_token_text.strip_prefix('!') {
        return ParsedTreeCalcTableCatalogWorkspaceSelector {
            workspace_selector_token: Some("!".to_string()),
            local_selector_token: local_selector_token.trim().to_string(),
            workspace_handle: context.current_workspace_handle.clone(),
            resolution_layer: TreeCalcTableCatalogResolutionLayer::CurrentWorkspaceRoot,
        };
    }

    if let Some((workspace_selector, local_selector)) =
        split_treecalc_table_workspace_selector(selector_token_text)
    {
        let workspace_selector = unescape_treecalc_bracketed_selector(workspace_selector.trim());
        let local_selector_token = local_selector.trim().to_string();
        if let Some(alias_target) =
            treecalc_table_catalog_workspace_alias(context, &workspace_selector)
        {
            return ParsedTreeCalcTableCatalogWorkspaceSelector {
                workspace_selector_token: Some(workspace_selector),
                local_selector_token,
                workspace_handle: alias_target,
                resolution_layer: TreeCalcTableCatalogResolutionLayer::WorkspaceAlias,
            };
        }
        return ParsedTreeCalcTableCatalogWorkspaceSelector {
            workspace_selector_token: Some(workspace_selector.clone()),
            local_selector_token,
            workspace_handle: workspace_selector,
            resolution_layer: TreeCalcTableCatalogResolutionLayer::DirectWorkspace,
        };
    }

    ParsedTreeCalcTableCatalogWorkspaceSelector {
        workspace_selector_token: None,
        local_selector_token: selector_token_text.to_string(),
        workspace_handle: context.current_workspace_handle.clone(),
        resolution_layer: TreeCalcTableCatalogResolutionLayer::CurrentWorkspacePath,
    }
}

fn treecalc_table_catalog_explicit_layer(
    projection: &TreeCalcTableNodeProjection,
    parsed_selector: &ParsedTreeCalcTableCatalogWorkspaceSelector,
) -> TreeCalcTableCatalogResolutionLayer {
    match parsed_selector.resolution_layer {
        TreeCalcTableCatalogResolutionLayer::WorkspaceAlias
        | TreeCalcTableCatalogResolutionLayer::DirectWorkspace
        | TreeCalcTableCatalogResolutionLayer::CurrentWorkspaceRoot => {
            parsed_selector.resolution_layer
        }
        _ if projection
            .table_descriptor
            .table_name
            .eq_ignore_ascii_case(&parsed_selector.local_selector_token) =>
        {
            TreeCalcTableCatalogResolutionLayer::CurrentWorkspaceTableName
        }
        _ => TreeCalcTableCatalogResolutionLayer::CurrentWorkspacePath,
    }
}

fn treecalc_table_catalog_matches<'a>(
    projections: &'a [TreeCalcTableNodeProjection],
    selector_token_text: &str,
) -> Vec<&'a TreeCalcTableNodeProjection> {
    projections
        .iter()
        .filter(|projection| {
            treecalc_table_path_tokens(projection)
                .iter()
                .any(|token| token.eq_ignore_ascii_case(selector_token_text))
        })
        .collect()
}

fn treecalc_table_catalog_workspace_alias(
    context: &TreeCalcTableCatalogResolverContext,
    workspace_selector: &str,
) -> Option<String> {
    context
        .workspace_aliases
        .iter()
        .find(|(alias, _)| alias.eq_ignore_ascii_case(workspace_selector))
        .map(|(_, workspace_handle)| workspace_handle.clone())
}

fn treecalc_table_catalog_deleted_match<'a>(
    context: &'a TreeCalcTableCatalogResolverContext,
    workspace_handle: &str,
    selector_token_text: &str,
) -> Option<&'a TreeCalcTableDeletedFact> {
    context.deleted_tables.iter().find(|deleted_table| {
        deleted_table.workspace_handle == workspace_handle
            && (deleted_table.table_id == selector_token_text
                || deleted_table
                    .selector_token_text
                    .eq_ignore_ascii_case(selector_token_text))
    })
}

fn treecalc_table_catalog_caller_context_dependency(
    request: &TreeCalcTableCatalogResolveRequest,
) -> bool {
    matches!(
        request.selector_kind,
        TreeCalcTableCatalogSelectorKind::SameNodeTable
            | TreeCalcTableCatalogSelectorKind::OmittedTableName
    )
}

fn treecalc_table_catalog_caller_context_identity(
    request: &TreeCalcTableCatalogResolveRequest,
) -> String {
    identity_record(
        "treecalc.table_catalog.caller_context.v1",
        [
            (
                "explicit_caller_context_id",
                request
                    .caller_context_id
                    .clone()
                    .unwrap_or_else(|| "none".to_string()),
            ),
            (
                "caller_table_region",
                request
                    .caller_table_region
                    .as_ref()
                    .map(treecalc_table_catalog_caller_region_identity)
                    .unwrap_or_else(|| "none".to_string()),
            ),
            (
                "caller_node",
                request.caller_node_id.map_or_else(
                    || "none".to_string(),
                    |node_id| {
                        identity_record(
                            "treecalc.table_catalog.caller_node.v1",
                            [("node_id", node_id.to_string())],
                        )
                    },
                ),
            ),
        ],
    )
}

fn treecalc_table_catalog_caller_region_identity(region: &TableCallerRegion) -> String {
    identity_record(
        "treecalc.table_catalog.caller_table_region.v1",
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
                    .map_or_else(|| "none".to_string(), |offset| offset.to_string()),
            ),
        ],
    )
}

fn treecalc_table_catalog_adjacency_diagnostics(
    context: &TreeCalcTableCatalogResolverContext,
    selector_token_text: &str,
    request: &TreeCalcTableCatalogResolveRequest,
) -> Vec<TreeCalcTableCatalogDiagnostic> {
    let mut diagnostics = Vec::new();
    if treecalc_table_catalog_set_contains(
        &context.namespace_adjacency.host_names,
        selector_token_text,
    ) {
        diagnostics.push(treecalc_table_catalog_diagnostic(
            TreeCalcTableCatalogDiagnosticCode::HostNameAdjacencyW074Mapping,
            "explicit table selector is adjacent to a host name; current W074 mapping keeps table selectors on the explicit structured-reference lane".to_string(),
            request,
        ));
    }
    if treecalc_table_catalog_set_contains(
        &context.namespace_adjacency.function_names,
        selector_token_text,
    ) {
        diagnostics.push(treecalc_table_catalog_diagnostic(
            TreeCalcTableCatalogDiagnosticCode::FunctionNameAdjacencyW074Mapping,
            "explicit table selector is adjacent to a function name; current W074 mapping keeps built-ins on the ordinary call-callee lane".to_string(),
            request,
        ));
    }
    if treecalc_table_catalog_set_contains(
        &context.namespace_adjacency.defined_names,
        selector_token_text,
    ) {
        diagnostics.push(treecalc_table_catalog_diagnostic(
            TreeCalcTableCatalogDiagnosticCode::DefinedNameAdjacencyW074Mapping,
            "explicit table selector is adjacent to a defined name; current W074 mapping keeps explicit structured references distinct from bare names".to_string(),
            request,
        ));
    }
    if treecalc_table_catalog_set_contains(
        &context.namespace_adjacency.lambda_valued_node_names,
        selector_token_text,
    ) {
        diagnostics.push(treecalc_table_catalog_diagnostic(
            TreeCalcTableCatalogDiagnosticCode::LambdaValuedNodeAdjacencyW074Mapping,
            "explicit table selector is adjacent to a lambda-valued host node; current W074 mapping keeps lambda-valued host nodes on the defined-name-LAMBDA lane".to_string(),
            request,
        ));
    }
    diagnostics
}

fn treecalc_table_catalog_set_contains(values: &BTreeSet<String>, needle: &str) -> bool {
    values
        .iter()
        .any(|value| value.eq_ignore_ascii_case(needle))
}

fn treecalc_table_catalog_diagnostic(
    code: TreeCalcTableCatalogDiagnosticCode,
    message: String,
    request: &TreeCalcTableCatalogResolveRequest,
) -> TreeCalcTableCatalogDiagnostic {
    TreeCalcTableCatalogDiagnostic {
        code,
        message,
        source_span_utf8: request.source_span_utf8,
        selector_token_text: request.selector_token_text.clone(),
    }
}

#[allow(clippy::too_many_arguments)]
fn build_treecalc_table_catalog_resolution(
    context: &TreeCalcTableCatalogResolverContext,
    request: &TreeCalcTableCatalogResolveRequest,
    parsed_selector: ParsedTreeCalcTableCatalogWorkspaceSelector,
    availability_packet: TreeCalcCrossWorkspaceAvailabilityPacket,
    resolution_layer: TreeCalcTableCatalogResolutionLayer,
    shape_hint: TreeCalcTableCatalogShapeHint,
    projection: Option<&TreeCalcTableNodeProjection>,
    caller_context_dependency: bool,
    caller_context_id: Option<String>,
    diagnostics: Vec<TreeCalcTableCatalogDiagnostic>,
    deleted_namespace_version: Option<String>,
) -> TreeCalcTableCatalogResolution {
    let table_namespace_version = projection
        .map(|projection| projection.table_namespace_version.clone())
        .or(deleted_namespace_version);
    let opaque_selector = identity_record(
        "treecalc.table_catalog.selector.v1",
        [
            ("kind", request.selector_kind.stable_id().to_string()),
            ("source_token", request.selector_token_text.clone()),
            (
                "workspace_selector",
                parsed_selector
                    .workspace_selector_token
                    .clone()
                    .unwrap_or_else(|| "current".to_string()),
            ),
            ("workspace_handle", parsed_selector.workspace_handle.clone()),
            (
                "local_selector",
                parsed_selector.local_selector_token.clone(),
            ),
            ("layer", resolution_layer.stable_id().to_string()),
        ],
    );
    let handle_identity = identity_record(
        "treecalc.table_catalog.reference_handle.v1",
        [
            ("selector", opaque_selector.clone()),
            ("layer", resolution_layer.stable_id().to_string()),
            ("shape", shape_hint.stable_id().to_string()),
            (
                "effective_table_id",
                projection
                    .map(|projection| projection.table_id.clone())
                    .unwrap_or_else(|| "none".to_string()),
            ),
            (
                "table_node_id",
                projection
                    .map(|projection| projection.table_node_id.to_string())
                    .unwrap_or_else(|| "none".to_string()),
            ),
            (
                "virtual_anchor_identity",
                projection
                    .map(|projection| projection.virtual_anchor_identity.clone())
                    .unwrap_or_else(|| "none".to_string()),
            ),
            (
                "host_namespace_version",
                context.host_namespace_version.clone(),
            ),
            (
                "table_namespace_version",
                table_namespace_version
                    .clone()
                    .unwrap_or_else(|| "none".to_string()),
            ),
            (
                "structure_context_version",
                context.structure_context_version.clone(),
            ),
            (
                "resolution_rule_version",
                context.resolution_rule_version.clone(),
            ),
            (
                "workspace_availability_version",
                availability_packet.availability_version.clone(),
            ),
            (
                "caller_context_id",
                caller_context_id
                    .clone()
                    .unwrap_or_else(|| "none".to_string()),
            ),
            (
                "diagnostics",
                identity_list(
                    diagnostics
                        .iter()
                        .map(|diagnostic| diagnostic.code.stable_id().to_string()),
                ),
            ),
        ],
    );
    TreeCalcTableCatalogResolution {
        table_reference_handle: opaque_identity_token(
            "treecalc.table_catalog.reference_handle.token.v1",
            &handle_identity,
        ),
        source_span_utf8: request.source_span_utf8,
        source_token_text: request.selector_token_text.clone(),
        opaque_selector,
        resolution_layer,
        shape_hint,
        effective_table_id: projection.map(|projection| projection.table_id.clone()),
        table_node_id: projection.map(|projection| projection.table_node_id),
        virtual_anchor_identity: projection
            .map(|projection| projection.virtual_anchor_identity.clone()),
        caller_context_dependency,
        caller_context_id,
        host_namespace_version: context.host_namespace_version.clone(),
        table_namespace_version,
        structure_context_version: context.structure_context_version.clone(),
        resolution_rule_version: context.resolution_rule_version.clone(),
        workspace_availability_version: Some(availability_packet.availability_version),
        diagnostics,
    }
}

fn split_treecalc_table_workspace_selector(selector: &str) -> Option<(&str, &str)> {
    let bytes = selector.as_bytes();
    let mut index = 0usize;
    let mut bracket_depth = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'[' => {
                bracket_depth += 1;
                index += 1;
            }
            b']' if bracket_depth > 0 => {
                if bytes.get(index + 1) == Some(&b']') {
                    index += 2;
                } else {
                    bracket_depth -= 1;
                    index += 1;
                }
            }
            b'!' if bracket_depth == 0 => {
                return Some((&selector[..index], &selector[index + 1..]));
            }
            _ => {
                index += 1;
            }
        }
    }
    None
}

fn unescape_treecalc_bracketed_selector(selector: &str) -> String {
    selector
        .strip_prefix('[')
        .and_then(|without_open| without_open.strip_suffix(']'))
        .map_or_else(
            || selector.to_string(),
            |bracketed| bracketed.replace("]]", "]"),
        )
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

    let mut body_cell_keys = BTreeSet::new();
    for cell in &snapshot.body_cell_nodes {
        if !row_ids.contains(&cell.row_id.0) {
            return Err(TreeCalcTableProjectionError::UnknownBodyCellRowId {
                row_id: cell.row_id.0.clone(),
            });
        }
        if !column_ids.contains(&cell.column_id) {
            return Err(TreeCalcTableProjectionError::UnknownBodyCellColumnId {
                column_id: cell.column_id.clone(),
            });
        }
        let key = (cell.row_id.0.clone(), cell.column_id.clone());
        if !body_cell_keys.insert(key) {
            return Err(TreeCalcTableProjectionError::DuplicateBodyCellNodeBinding {
                row_id: cell.row_id.0.clone(),
                column_id: cell.column_id.clone(),
            });
        }
    }

    let mut totals_cell_columns = BTreeSet::new();
    for cell in &snapshot.totals_cell_nodes {
        if !column_ids.contains(&cell.column_id) {
            return Err(TreeCalcTableProjectionError::UnknownTotalsCellColumnId {
                column_id: cell.column_id.clone(),
            });
        }
        if !totals_cell_columns.insert(cell.column_id.clone()) {
            return Err(
                TreeCalcTableProjectionError::DuplicateTotalsCellNodeBinding {
                    column_id: cell.column_id.clone(),
                },
            );
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

fn treecalc_table_body_cell_node_identity(snapshot: &TreeCalcTableNodeSnapshot) -> String {
    identity_record(
        "treecalc.table_body_cell_nodes.v1",
        [
            ("table", snapshot.table_id.clone()),
            (
                "cells",
                identity_list(snapshot.body_cell_nodes.iter().map(|cell| {
                    identity_record(
                        "body_cell_node",
                        [
                            ("row", cell.row_id.0.clone()),
                            ("column", cell.column_id.clone()),
                            ("node", cell.node_id.to_string()),
                        ],
                    )
                })),
            ),
        ],
    )
}

fn treecalc_table_totals_cell_node_identity(snapshot: &TreeCalcTableNodeSnapshot) -> String {
    identity_record(
        "treecalc.table_totals_cell_nodes.v1",
        [
            ("table", snapshot.table_id.clone()),
            (
                "cells",
                identity_list(snapshot.totals_cell_nodes.iter().map(|cell| {
                    identity_record(
                        "totals_cell_node",
                        [
                            ("column", cell.column_id.clone()),
                            ("node", cell.node_id.to_string()),
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

fn treecalc_table_workspace_availability_version(snapshot: &TreeCalcTableNodeSnapshot) -> String {
    identity_record(
        "treecalc.table_workspace_availability.v1",
        [
            (
                "workbook",
                snapshot.virtual_anchor.workbook_scope_ref.clone(),
            ),
            ("sheet", snapshot.virtual_anchor.sheet_scope_ref.clone()),
        ],
    )
}

fn treecalc_table_workspace_alias_version(snapshot: &TreeCalcTableNodeSnapshot) -> String {
    identity_record(
        "treecalc.table_workspace_alias.v1",
        [
            (
                "workbook",
                snapshot.virtual_anchor.workbook_scope_ref.clone(),
            ),
            ("canonical_path", snapshot.canonical_path.clone()),
            ("namespace", snapshot.table_namespace_version.clone()),
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
    pub value: CalcValue,
}

impl TreeCalcTableSparseValue {
    #[must_use]
    pub fn data(row_id: impl Into<String>, column_id: impl Into<String>, value: CalcValue) -> Self {
        Self {
            section: TreeCalcTableSparseSection::Data,
            row_id: Some(TreeCalcTableRowId(row_id.into())),
            column_id: column_id.into(),
            value,
        }
    }

    #[must_use]
    pub fn header(column_id: impl Into<String>, value: CalcValue) -> Self {
        Self {
            section: TreeCalcTableSparseSection::Headers,
            row_id: None,
            column_id: column_id.into(),
            value,
        }
    }

    #[must_use]
    pub fn totals(column_id: impl Into<String>, value: CalcValue) -> Self {
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
    pub sparse_reference_values: TreeCalcSparseReferenceValuesBinding,
    pub scalar_cell_values: BTreeMap<String, CalcValue>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeCalcTableReplayEvidenceStatus {
    RetainedEvidenceAvailable,
    TypedProjectionGap,
    TypedUnavailable,
    BlockedUpstream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeCalcTableReplayEvidenceLane {
    pub lane_id: &'static str,
    pub owner_repo: &'static str,
    pub status: TreeCalcTableReplayEvidenceStatus,
    pub retained_artifacts_or_beads: &'static [&'static str],
    pub view_families: &'static [&'static str],
    pub value_wire_field: Option<&'static str>,
    pub blocker_id: Option<&'static str>,
    pub closure_role: &'static str,
    pub producer_private_string_parsing_allowed: bool,
    pub excel_internal_inference_allowed: bool,
}

pub const TREECALC_TABLE_REPLAY_EVIDENCE_LANES: &[TreeCalcTableReplayEvidenceLane] = &[
    TreeCalcTableReplayEvidenceLane {
        lane_id: "dnatreecalc_retained_table_producer_views",
        owner_repo: "DnaTreeCalc",
        status: TreeCalcTableReplayEvidenceStatus::RetainedEvidenceAvailable,
        retained_artifacts_or_beads: &[
            "dtc-z0i.5.5",
            "dtc-z0i.5.6.1",
            "dtc-z0i.5.7",
            "dtc-z0i.7.1",
            "../DnaTreeCalc/docs/test-runs/w056-table-structured-references-001/",
            "../DnaTreeCalc/docs/test-runs/w056-table-empty-body-001/",
            "../DnaTreeCalc/docs/test-runs/w056-table-lifecycle-001/",
            "../DnaTreeCalc/docs/test-runs/w056-table-dynamic-cross-workspace-001/",
        ],
        view_families: &[
            "table_slice",
            "per_node_value",
            "effective_display_text",
            "execution_outcome",
            "dependency_evidence",
            "invalidation_evidence",
            "retained_artifact_ref",
            "dynamic_table_rebind",
        ],
        value_wire_field: Some("comparison_value"),
        blocker_id: None,
        closure_role: "producer-owned table node, bridge, value, display, dependency, invalidation, lifecycle, and dynamic table evidence through OxCalcTreeContext",
        producer_private_string_parsing_allowed: false,
        excel_internal_inference_allowed: false,
    },
    TreeCalcTableReplayEvidenceLane {
        lane_id: "oxcalc_runtime_packet_and_identity_facts",
        owner_repo: "OxCalc",
        status: TreeCalcTableReplayEvidenceStatus::RetainedEvidenceAvailable,
        retained_artifacts_or_beads: &[
            "calc-4vs8.58",
            "calc-4vs8.59",
            "calc-4vs8.60",
            "src/oxcalc-core/src/structured_table.rs",
        ],
        view_families: &[
            "source_preservation",
            "prepared_identity",
            "dependency_evidence",
            "invalidation_evidence",
            "lifecycle_update",
            "dynamic_table_rebind",
            "sparse_reference_binding",
            "registry_snapshot_identity",
        ],
        value_wire_field: Some("comparison_value"),
        blocker_id: None,
        closure_role: "OxCalc-owned table custody, virtual anchor, dependency, invalidation, sparse reader, prepared identity, and registry/capability identity facts",
        producer_private_string_parsing_allowed: false,
        excel_internal_inference_allowed: false,
    },
    TreeCalcTableReplayEvidenceLane {
        lane_id: "oxxlplay_excel_listobject_oracle_views",
        owner_repo: "OxXlPlay",
        status: TreeCalcTableReplayEvidenceStatus::RetainedEvidenceAvailable,
        retained_artifacts_or_beads: &[
            "oxxlplay-4nd.1",
            "oxxlplay-4nd.2",
            "oxxlplay-4nd.3",
            "oxxlplay-4nd.4",
            "oxxlplay-4nd.5",
            "../OxXlPlay/states/excel/xlplay_workbook_construction_spec_001/",
            "../OxXlPlay/states/excel/xlplay_table_construction_basic_001/",
            "../OxXlPlay/states/excel/xlplay_table_update_oracle_001/",
        ],
        view_families: &[
            "table_slice",
            "comparison_value",
            "effective_display_text",
            "execution_outcome",
            "table_update_oracle",
        ],
        value_wire_field: Some("comparison_value"),
        blocker_id: None,
        closure_role: "clean-room Excel ListObject construction and before/after observation; dependency internals stay unavailable",
        producer_private_string_parsing_allowed: false,
        excel_internal_inference_allowed: false,
    },
    TreeCalcTableReplayEvidenceLane {
        lane_id: "oxreplay_third_pass_table_intake",
        owner_repo: "OxReplay",
        status: TreeCalcTableReplayEvidenceStatus::RetainedEvidenceAvailable,
        retained_artifacts_or_beads: &[
            "oxreplay-qb9",
            "../OxReplay/docs/test-corpus/bundles/host_rollout_w056_table_third_pass_001/",
            "../OxReplay/docs/test-runs/w007-w056-table-third-pass-intake-baseline/",
        ],
        view_families: &[
            "validate_bundle",
            "replay",
            "diff",
            "explain",
            "table_slice",
            "comparison_value",
            "effective_display_text",
            "execution_outcome",
            "dependency_evidence",
            "invalidation_evidence",
            "retained_artifact_ref",
        ],
        value_wire_field: Some("comparison_value"),
        blocker_id: None,
        closure_role: "retained validation/replay/diff/explain intake over declared DnaTreeCalc and OxXlPlay table payloads",
        producer_private_string_parsing_allowed: false,
        excel_internal_inference_allowed: false,
    },
    TreeCalcTableReplayEvidenceLane {
        lane_id: "oxreplay_matched_treecalc_excel_table_views",
        owner_repo: "OxReplay",
        status: TreeCalcTableReplayEvidenceStatus::RetainedEvidenceAvailable,
        retained_artifacts_or_beads: &[
            "oxreplay-p1w.3",
            "../OxReplay/docs/test-corpus/bundles/host_rollout_matched_table_001/",
            "../OxReplay/docs/test-runs/w007-host-rollout-host_rollout_matched_table_001-baseline/",
        ],
        view_families: &[
            "comparison_value",
            "effective_display_text",
            "execution_outcome",
            "table_slice",
            "table_update_oracle",
            "dependency_evidence",
            "invalidation_evidence",
            "retained_artifact_ref",
        ],
        value_wire_field: Some("comparison_value"),
        blocker_id: None,
        closure_role: "matched TreeCalc/Excel replay mechanics over declared normalized payloads; no semantic inference from formulas or Excel internals",
        producer_private_string_parsing_allowed: false,
        excel_internal_inference_allowed: false,
    },
    TreeCalcTableReplayEvidenceLane {
        lane_id: "comparison_value_helper_replacement",
        owner_repo: "OxFunc/OxReplay",
        status: TreeCalcTableReplayEvidenceStatus::BlockedUpstream,
        retained_artifacts_or_beads: &["BLK-REPLAY-003"],
        view_families: &["comparison_value"],
        value_wire_field: Some("comparison_value"),
        blocker_id: Some("BLK-REPLAY-003"),
        closure_role: "future shared OxFunc-owned replay value helper replacement; current local comparator is not TreeCalc table semantics",
        producer_private_string_parsing_allowed: false,
        excel_internal_inference_allowed: false,
    },
    TreeCalcTableReplayEvidenceLane {
        lane_id: "excel_dependency_dirty_event_order_internals",
        owner_repo: "OxXlPlay/OxReplay/OxCalc",
        status: TreeCalcTableReplayEvidenceStatus::TypedUnavailable,
        retained_artifacts_or_beads: &[
            "../OxXlPlay/states/excel/xlplay_table_update_oracle_001/bridge.json",
            "../OxXlPlay/states/excel/xlplay_table_update_oracle_001/views/table-update-oracle.json",
        ],
        view_families: &["dependency_evidence", "invalidation_evidence"],
        value_wire_field: None,
        blocker_id: None,
        closure_role: "Excel COM does not expose dependency graph, dirty-set, or event-order internals; OxCalc facts remain host-owned and are not inferred from Excel",
        producer_private_string_parsing_allowed: false,
        excel_internal_inference_allowed: false,
    },
    TreeCalcTableReplayEvidenceLane {
        lane_id: "legacy_outcome_class_projection_gap",
        owner_repo: "OxXlPlay/OxReplay",
        status: TreeCalcTableReplayEvidenceStatus::TypedProjectionGap,
        retained_artifacts_or_beads: &[
            "../OxXlPlay/states/excel/xlplay_structured_reference_workbook_001/",
            "../OxXlPlay/states/excel/xlplay_workbook_construction_spec_001/",
            "../OxXlPlay/states/excel/xlplay_table_construction_basic_001/",
            "../OxXlPlay/states/excel/xlplay_table_update_oracle_001/",
        ],
        view_families: &["execution_outcome"],
        value_wire_field: None,
        blocker_id: None,
        closure_role: "older retained Excel table artifacts may lack execution_outcome.class_id; table_update_oracle carries class_id and the gap is projection metadata, not table semantics",
        producer_private_string_parsing_allowed: false,
        excel_internal_inference_allowed: false,
    },
    TreeCalcTableReplayEvidenceLane {
        lane_id: "namespace_anchor_workspace_cross_producer_pairing",
        owner_repo: "DnaTreeCalc/OxXlPlay/OxReplay",
        status: TreeCalcTableReplayEvidenceStatus::TypedProjectionGap,
        retained_artifacts_or_beads: &[
            "../DnaTreeCalc/docs/test-runs/w056-table-dynamic-cross-workspace-001/",
            "../OxReplay/docs/test-corpus/bundles/host_rollout_w056_table_third_pass_001/",
            "../OxReplay/docs/test-runs/w007-w056-table-third-pass-intake-baseline/",
        ],
        view_families: &[
            "dynamic_table_rebind",
            "dependency_evidence",
            "invalidation_evidence",
            "retained_artifact_ref",
        ],
        value_wire_field: None,
        blocker_id: None,
        closure_role: "dynamic/cross-workspace table evidence exists, but full namespace/anchor/workspace cross-producer pairing remains a final-audit projection gap",
        producer_private_string_parsing_allowed: false,
        excel_internal_inference_allowed: false,
    },
];

#[must_use]
pub fn treecalc_table_replay_evidence_lane(
    lane_id: &str,
) -> Option<&'static TreeCalcTableReplayEvidenceLane> {
    TREECALC_TABLE_REPLAY_EVIDENCE_LANES
        .iter()
        .find(|lane| lane.lane_id == lane_id)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeCalcTableRolloutLaneStatus {
    ClosedEvidence,
    OpenParentReconciliation,
    OpenAdjacentNonBlocking,
    ExplicitNonImpact,
    FutureExtensionTracked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeCalcTableCrossRepoRolloutLane {
    pub lane_id: &'static str,
    pub repo: &'static str,
    pub promotion_order: u8,
    pub status: TreeCalcTableRolloutLaneStatus,
    pub responsibility: &'static str,
    pub counterpart_anchors: &'static [&'static str],
    pub evidence_obligation: &'static str,
    pub residual_action: &'static str,
    pub blocks_w056_table_semantic_claim: bool,
    pub producer_private_string_parsing_allowed: bool,
    pub semantic_mirror_allowed: bool,
}

pub const TREECALC_TABLE_CROSS_REPO_ROLLOUT_LANES: &[TreeCalcTableCrossRepoRolloutLane] = &[
    TreeCalcTableCrossRepoRolloutLane {
        lane_id: "oxfml_generic_table_packets_and_name_call_lanes",
        repo: "OxFml",
        promotion_order: 10,
        status: TreeCalcTableRolloutLaneStatus::ClosedEvidence,
        responsibility: "structured-reference grammar, generic bind packets, prepared identity inputs, lexical scope, and current W051/W056 name-call handoff",
        counterpart_anchors: &[
            "fml-ds0.12",
            "fml-ds0.13",
            "fml-ds0.15",
            "fml-ds0.16",
            "fml-ds0.6.5",
        ],
        evidence_obligation: "OxFml supplies generic table packets and W074 host-name mapping without TreeCalc semantics",
        residual_action: "Broader W036/W074 table-language or name-call extensions require new versioned evidence before OxCalc consumes them",
        blocks_w056_table_semantic_claim: false,
        producer_private_string_parsing_allowed: false,
        semantic_mirror_allowed: false,
    },
    TreeCalcTableCrossRepoRolloutLane {
        lane_id: "oxfunc_opaque_reference_and_registry_lanes",
        repo: "OxFunc",
        promotion_order: 20,
        status: TreeCalcTableRolloutLaneStatus::OpenAdjacentNonBlocking,
        responsibility: "function semantics, opaque ReferenceLike admission, registry mutation, capability overlays, and typed rejection outcomes",
        counterpart_anchors: &["oxf-ypq2.13", "oxf-ypq2.15", "oxf-ypq2.16", "oxf-ypq2.12"],
        evidence_obligation: "Closed table-specific aggregate and reference-visible lanes consume generic resolver APIs and do not inspect TreeCalc selectors",
        residual_action: "Keep oxf-ypq2.12 as broader formula-call registry migration; it is adjacent unless future table UDF admission depends on it",
        blocks_w056_table_semantic_claim: false,
        producer_private_string_parsing_allowed: false,
        semantic_mirror_allowed: false,
    },
    TreeCalcTableCrossRepoRolloutLane {
        lane_id: "oxcalc_table_custody_runtime_and_hardening",
        repo: "OxCalc",
        promotion_order: 30,
        status: TreeCalcTableRolloutLaneStatus::ClosedEvidence,
        responsibility: "table custody, virtual anchors, resolver, sparse readers, dependency and invalidation facts, dynamic rebind, namespace and caller identity",
        counterpart_anchors: &[
            "calc-4vs8.57",
            "calc-4vs8.58",
            "calc-4vs8.59",
            "calc-4vs8.60",
            "calc-4vs8.61",
        ],
        evidence_obligation: "OxCalc proves the typed table path without CalcValue::Table, eager materialization, or duplicated grammar",
        residual_action: "Proceed to calc-4vs8.63 final audit after rollout reconciliation",
        blocks_w056_table_semantic_claim: false,
        producer_private_string_parsing_allowed: false,
        semantic_mirror_allowed: false,
    },
    TreeCalcTableCrossRepoRolloutLane {
        lane_id: "dnatreecalc_table_product_corpus_and_parent_reconciliation",
        repo: "DnaTreeCalc",
        promotion_order: 40,
        status: TreeCalcTableRolloutLaneStatus::OpenParentReconciliation,
        responsibility: "product table-node model, corpus activation, table operations, persistence, UX evidence, and LiveOxCalc bridge activation",
        counterpart_anchors: &[
            "dtc-z0i.5",
            "dtc-z0i.5.1",
            "dtc-z0i.5.2",
            "dtc-z0i.5.3",
            "dtc-z0i.5.4",
            "dtc-z0i.5.5",
            "dtc-z0i.5.6",
            "dtc-z0i.5.6.1",
            "dtc-z0i.5.7",
            "dtc-z0i.5.8",
            "dtc-z0i.7.1",
        ],
        evidence_obligation: "Closed child beads provide table structured-reference, empty-body, lifecycle, retained artifact, and dynamic/cross-workspace evidence through OxCalcTreeContext",
        residual_action: "Close or narrow DnaTreeCalc parent beads dtc-z0i.5.6 and dtc-z0i.5 in DnaTreeCalc once its unrelated dirty bead state is reconciled",
        blocks_w056_table_semantic_claim: false,
        producer_private_string_parsing_allowed: false,
        semantic_mirror_allowed: false,
    },
    TreeCalcTableCrossRepoRolloutLane {
        lane_id: "oxxlplay_excel_listobject_observation",
        repo: "OxXlPlay",
        promotion_order: 40,
        status: TreeCalcTableRolloutLaneStatus::ClosedEvidence,
        responsibility: "Excel workbook/table construction and black-box ListObject observations",
        counterpart_anchors: &[
            "oxxlplay-4nd.1",
            "oxxlplay-4nd.2",
            "oxxlplay-4nd.3",
            "oxxlplay-4nd.4",
            "oxxlplay-4nd.5",
        ],
        evidence_obligation: "OxXlPlay supplies retained workbook construction, table construction, and table update oracle observations with typed unavailable Excel internals",
        residual_action: "Add new OxXlPlay observation beads only for future dynamic table Excel-comparable extensions not admitted in W056",
        blocks_w056_table_semantic_claim: false,
        producer_private_string_parsing_allowed: false,
        semantic_mirror_allowed: false,
    },
    TreeCalcTableCrossRepoRolloutLane {
        lane_id: "oxreplay_retained_comparison_and_value_wire",
        repo: "OxReplay",
        promotion_order: 50,
        status: TreeCalcTableRolloutLaneStatus::OpenAdjacentNonBlocking,
        responsibility: "retained validation, replay, diff, explain, adapter capability evidence, and value-wire governance",
        counterpart_anchors: &["oxreplay-qb9", "oxreplay-p1w.3", "BLK-REPLAY-003"],
        evidence_obligation: "OxReplay compares declared table payloads and matched TreeCalc/Excel table views without producer-private parsing",
        residual_action: "Keep BLK-REPLAY-003 as shared comparison_value helper cleanup; it must not create an OxCalc value adapter",
        blocks_w056_table_semantic_claim: false,
        producer_private_string_parsing_allowed: false,
        semantic_mirror_allowed: false,
    },
    TreeCalcTableCrossRepoRolloutLane {
        lane_id: "dnaonecalc_no_host_guardrail_and_future_udf_consumption",
        repo: "DnaOneCalc",
        promotion_order: 60,
        status: TreeCalcTableRolloutLaneStatus::ExplicitNonImpact,
        responsibility: "no-host single-formula guardrails, OxFml-internal LET/LAMBDA lexical references, and future registry-backed UDF consumption",
        counterpart_anchors: &[
            "dno-rl7u",
            "dno-7vt4.1",
            "dno-7vt4.4",
            "dno-7vt4.5",
            "dno-7vt4.7",
            "dno-7vt4.9",
        ],
        evidence_obligation: "Ordinary DnaOneCalc formulas require no host table namespace; future VBA/XLL UDFs use OxFunc/OxFml registry surfaces",
        residual_action: "Continue WS-15 future UDF work without introducing table host-reference requirements for ordinary formulas",
        blocks_w056_table_semantic_claim: false,
        producer_private_string_parsing_allowed: false,
        semantic_mirror_allowed: false,
    },
    TreeCalcTableCrossRepoRolloutLane {
        lane_id: "oxvba_udf_descriptor_metadata_future_extension",
        repo: "OxVba",
        promotion_order: 70,
        status: TreeCalcTableRolloutLaneStatus::FutureExtensionTracked,
        responsibility: "VBA/XLL source discovery metadata and host-UDF descriptors for future OxFunc registration requests",
        counterpart_anchors: &[
            "bd-sg5h",
            "docs/worksets/WORKSET_2026-05-10_HOST_PROGRAM_DESIGN_AND_UDF_REWORK.md",
        ],
        evidence_obligation: "OxVba supplies descriptor facts only; it does not receive TreeCalc table selectors or own function/name precedence",
        residual_action: "Transform future descriptors into OxFunc registration requests; do not bypass W093 registry snapshots or TreeCalc table opacity",
        blocks_w056_table_semantic_claim: false,
        producer_private_string_parsing_allowed: false,
        semantic_mirror_allowed: false,
    },
    TreeCalcTableCrossRepoRolloutLane {
        lane_id: "ide_visual_foundation_impact_scan",
        repo: "OxIde/DnaOxIde/DnaVisiCalc/Foundation",
        promotion_order: 80,
        status: TreeCalcTableRolloutLaneStatus::ExplicitNonImpact,
        responsibility: "impact-scan-only consumers and doctrine references",
        counterpart_anchors: &["impact-scan:no direct W056 node-table seam found"],
        evidence_obligation: "No direct interface dependency found for this W056 table hardening spine",
        residual_action: "Open a repo-local bead only if a future UI, visual, doctrine, or shared-interface dependency appears",
        blocks_w056_table_semantic_claim: false,
        producer_private_string_parsing_allowed: false,
        semantic_mirror_allowed: false,
    },
];

#[must_use]
pub fn treecalc_table_cross_repo_rollout_lane(
    lane_id: &str,
) -> Option<&'static TreeCalcTableCrossRepoRolloutLane> {
    TREECALC_TABLE_CROSS_REPO_ROLLOUT_LANES
        .iter()
        .find(|lane| lane.lane_id == lane_id)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeCalcTableFinalAuditStatus {
    SupportedByExecutableEvidence,
    SupportedWithTypedProjectionGap,
    OpenParentReconciliation,
    ExplicitNonImpact,
    FutureExtensionTracked,
    ParentW056NonTableRemaining,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeCalcTableFinalAuditItem {
    pub item_id: &'static str,
    pub status: TreeCalcTableFinalAuditStatus,
    pub product_scope: &'static str,
    pub evidence_anchors: &'static [&'static str],
    pub still_open: &'static str,
    pub parent_w056_implication: &'static str,
    pub blocks_node_table_completion: bool,
    pub blocks_parent_w056_completion: bool,
    pub dense_or_eager_materialization_allowed: bool,
    pub private_formula_parsing_allowed: bool,
    pub oxfml_or_oxfunc_treecalc_branch_allowed: bool,
}

pub const TREECALC_TABLE_FINAL_AUDIT_ITEMS: &[TreeCalcTableFinalAuditItem] = &[
    TreeCalcTableFinalAuditItem {
        item_id: "structured_reference_packets_and_projection",
        status: TreeCalcTableFinalAuditStatus::SupportedByExecutableEvidence,
        product_scope: "node-associated table packets and projections for path[Col], path[@Col], #Headers, #Data, #Totals, #All, composite refs, omitted table names, caller row context, escaped names, and multi-column ranges",
        evidence_anchors: &[
            "calc-4vs8.44",
            "calc-4vs8.46",
            "calc-4vs8.57",
            "fml-ds0.12",
            "fml-ds0.13",
            "fml-ds0.15",
            "fml-ds0.16",
        ],
        still_open: "none for the declared W056 table packet and projection slice",
        parent_w056_implication: "carries the generic OxFml packet contract into parent W056 table status without giving OxCalc formula grammar ownership",
        blocks_node_table_completion: false,
        blocks_parent_w056_completion: false,
        dense_or_eager_materialization_allowed: false,
        private_formula_parsing_allowed: false,
        oxfml_or_oxfunc_treecalc_branch_allowed: false,
    },
    TreeCalcTableFinalAuditItem {
        item_id: "sparse_reference_readers_and_value_transport",
        status: TreeCalcTableFinalAuditStatus::SupportedByExecutableEvidence,
        product_scope: "table ReferenceLike readers for data body, headers, totals, all-sections, current row, single-row, empty-body, and sparse range traversal",
        evidence_anchors: &[
            "calc-4vs8.47",
            "calc-4vs8.48",
            "calc-4vs8.58",
            "calc-4vs8.60",
        ],
        still_open: "none for declared table readers; eager value fallback remains outside closure evidence",
        parent_w056_implication: "keeps CalcValue free of TreeCalc/table variants and preserves reference identity through the first aggregate/function path",
        blocks_node_table_completion: false,
        blocks_parent_w056_completion: false,
        dense_or_eager_materialization_allowed: false,
        private_formula_parsing_allowed: false,
        oxfml_or_oxfunc_treecalc_branch_allowed: false,
    },
    TreeCalcTableFinalAuditItem {
        item_id: "dependency_invalidation_lifecycle_and_identity",
        status: TreeCalcTableFinalAuditStatus::SupportedByExecutableEvidence,
        product_scope: "row membership, row order, column identity, header text, data region, totals region, caller row context, namespace version, structure context, and registry snapshot invalidation facts",
        evidence_anchors: &[
            "calc-4vs8.49",
            "calc-4vs8.50",
            "calc-4vs8.53",
            "calc-4vs8.59",
            "calc-4vs8.61",
        ],
        still_open: "none for OxCalc-owned table lifecycle facts; Excel internal dirty-set/event-order facts are typed unavailable",
        parent_w056_implication: "lets parent W056 distinguish OxCalc-owned invalidation from black-box Excel observation",
        blocks_node_table_completion: false,
        blocks_parent_w056_completion: false,
        dense_or_eager_materialization_allowed: false,
        private_formula_parsing_allowed: false,
        oxfml_or_oxfunc_treecalc_branch_allowed: false,
    },
    TreeCalcTableFinalAuditItem {
        item_id: "table_formulas_functions_and_udf_boundary",
        status: TreeCalcTableFinalAuditStatus::SupportedByExecutableEvidence,
        product_scope: "per-row column formulas, totals formulas, first aggregate/reference-visible function groups, typed exclusions for context-needing functions, and host-UDF boundary metadata",
        evidence_anchors: &[
            "calc-4vs8.23",
            "calc-4vs8.36",
            "calc-4vs8.60",
            "oxf-ypq2.13",
            "oxf-ypq2.15",
            "oxf-ypq2.16",
        ],
        still_open: "broader formula-call registry migration oxf-ypq2.12 remains adjacent unless a future table UDF argument contract depends on it",
        parent_w056_implication: "current table function support is closed through opaque references; future UDF expansion must version through OxFunc/OxFml registry surfaces",
        blocks_node_table_completion: false,
        blocks_parent_w056_completion: false,
        dense_or_eager_materialization_allowed: false,
        private_formula_parsing_allowed: false,
        oxfml_or_oxfunc_treecalc_branch_allowed: false,
    },
    TreeCalcTableFinalAuditItem {
        item_id: "dynamic_table_rebind_namespace_anchor_workspace",
        status: TreeCalcTableFinalAuditStatus::SupportedWithTypedProjectionGap,
        product_scope: "table create/delete/rename/move/reorder, stable virtual anchors, same-node table lookup, dynamic table references, INDIRECT-style table targets, workspace aliases, unavailable workspace degradation, and stable save/reopen identity",
        evidence_anchors: &[
            "calc-4vs8.52",
            "calc-4vs8.54",
            "calc-4vs8.56",
            "calc-4vs8.59",
            "calc-4vs8.61",
            "dtc-z0i.7.1",
        ],
        still_open: "full cross-producer namespace/anchor/workspace pairing remains a typed retained-evidence projection gap, not an OxCalc runtime semantic gap",
        parent_w056_implication: "parent W056 can carry the admitted dynamic table rebind behavior while keeping broader cross-producer evidence pairing explicit",
        blocks_node_table_completion: false,
        blocks_parent_w056_completion: false,
        dense_or_eager_materialization_allowed: false,
        private_formula_parsing_allowed: false,
        oxfml_or_oxfunc_treecalc_branch_allowed: false,
    },
    TreeCalcTableFinalAuditItem {
        item_id: "oracle_replay_and_value_wire",
        status: TreeCalcTableFinalAuditStatus::SupportedWithTypedProjectionGap,
        product_scope: "retained TreeCalc, OxCalc, OxXlPlay, and OxReplay evidence for table slices, comparison_value, effective display, execution outcome, dependency evidence, invalidation evidence, and table update observations",
        evidence_anchors: &[
            "calc-4vs8.61",
            "oxxlplay-4nd.1",
            "oxxlplay-4nd.2",
            "oxxlplay-4nd.3",
            "oxxlplay-4nd.4",
            "oxxlplay-4nd.5",
            "oxreplay-qb9",
            "oxreplay-p1w.3",
        ],
        still_open: "BLK-REPLAY-003 remains shared comparison_value helper cleanup; legacy Excel artifacts may lack execution_outcome.class_id; Excel dependency internals remain unavailable",
        parent_w056_implication: "retained table evidence is sufficient for the declared table slice, with non-semantic projection gaps named before parent closure",
        blocks_node_table_completion: false,
        blocks_parent_w056_completion: false,
        dense_or_eager_materialization_allowed: false,
        private_formula_parsing_allowed: false,
        oxfml_or_oxfunc_treecalc_branch_allowed: false,
    },
    TreeCalcTableFinalAuditItem {
        item_id: "dnatreecalc_product_activation_parent_reconciliation",
        status: TreeCalcTableFinalAuditStatus::OpenParentReconciliation,
        product_scope: "DnaTreeCalc table product corpus, lifecycle bridge, empty-body behavior, dynamic/cross-workspace table artifacts, and retained producer evidence through OxCalcTreeContext",
        evidence_anchors: &[
            "dtc-z0i.5.1",
            "dtc-z0i.5.2",
            "dtc-z0i.5.3",
            "dtc-z0i.5.4",
            "dtc-z0i.5.5",
            "dtc-z0i.5.6.1",
            "dtc-z0i.5.7",
            "dtc-z0i.5.8",
            "dtc-z0i.7.1",
        ],
        still_open: "DnaTreeCalc parent beads dtc-z0i.5.6 and dtc-z0i.5 need repo-local close-or-narrow graph hygiene after unrelated bead-state dirt is reconciled",
        parent_w056_implication: "does not block the OxCalc table semantic claim, but remains visible before broad DnaTreeCalc W004/W005 reporting",
        blocks_node_table_completion: false,
        blocks_parent_w056_completion: false,
        dense_or_eager_materialization_allowed: false,
        private_formula_parsing_allowed: false,
        oxfml_or_oxfunc_treecalc_branch_allowed: false,
    },
    TreeCalcTableFinalAuditItem {
        item_id: "dnaonecalc_no_host_formula_guardrail",
        status: TreeCalcTableFinalAuditStatus::ExplicitNonImpact,
        product_scope: "ordinary DnaOneCalc single-formula execution with OxFml-internal LET/LAMBDA lexical references and no host table namespace",
        evidence_anchors: &[
            "dno-rl7u",
            "dno-7vt4.1",
            "dno-7vt4.4",
            "dno-7vt4.5",
            "dno-7vt4.7",
            "dno-7vt4.9",
        ],
        still_open: "future VBA/XLL UDF registration remains registry-backed work and must not make ordinary formulas depend on host table references",
        parent_w056_implication: "records that table host context remains consumer-optional and does not regress no-host formula use",
        blocks_node_table_completion: false,
        blocks_parent_w056_completion: false,
        dense_or_eager_materialization_allowed: false,
        private_formula_parsing_allowed: false,
        oxfml_or_oxfunc_treecalc_branch_allowed: false,
    },
    TreeCalcTableFinalAuditItem {
        item_id: "vba_xll_udf_descriptor_future_extension",
        status: TreeCalcTableFinalAuditStatus::FutureExtensionTracked,
        product_scope: "future VBA/XLL discovery metadata that can feed OxFunc registration without exposing TreeCalc table selectors",
        evidence_anchors: &[
            "bd-sg5h",
            "docs/worksets/WORKSET_2026-05-10_HOST_PROGRAM_DESIGN_AND_UDF_REWORK.md",
        ],
        still_open: "future descriptors must be transformed into OxFunc registration requests and W093 snapshot/change-set updates",
        parent_w056_implication: "not part of current node-table support, but prevents future UDF work from bypassing the registry-owned lane",
        blocks_node_table_completion: false,
        blocks_parent_w056_completion: false,
        dense_or_eager_materialization_allowed: false,
        private_formula_parsing_allowed: false,
        oxfml_or_oxfunc_treecalc_branch_allowed: false,
    },
    TreeCalcTableFinalAuditItem {
        item_id: "parent_w056_non_table_reference_spine",
        status: TreeCalcTableFinalAuditStatus::ParentW056NonTableRemaining,
        product_scope: "non-table W056 reference families: broad W004/W005 activation, bare host-name/callable lanes, node-as-function, dynamic non-table references, cross-workspace non-table references, and retained non-table replay evidence",
        evidence_anchors: &[
            "calc-4vs8.30",
            "calc-8tox",
            "calc-4vs8.31",
            "calc-4vs8.32",
            "calc-4vs8.33",
            "calc-4vs8.5",
        ],
        still_open: "parent W056 remains open for non-table reference closure even after the node-table topic is complete",
        parent_w056_implication: "prevents broad W056 closure, but does not reopen the node-associated table topic",
        blocks_node_table_completion: false,
        blocks_parent_w056_completion: true,
        dense_or_eager_materialization_allowed: false,
        private_formula_parsing_allowed: false,
        oxfml_or_oxfunc_treecalc_branch_allowed: false,
    },
];

#[must_use]
pub fn treecalc_table_final_audit_item(
    item_id: &str,
) -> Option<&'static TreeCalcTableFinalAuditItem> {
    TREECALC_TABLE_FINAL_AUDIT_ITEMS
        .iter()
        .find(|item| item.item_id == item_id)
}

#[derive(Debug, Clone, PartialEq)]
struct TreeCalcTableSparseSlot {
    coord: SparseCellCoord,
    absolute_row: u32,
    absolute_col: u32,
    section: TreeCalcTableSparseSection,
    row_id: Option<TreeCalcTableRowId>,
    column_id: String,
    value: Option<CalcValue>,
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
    defined_cells: BTreeMap<SparseCellCoord, CalcValue>,
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

    #[allow(clippy::too_many_arguments)]
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
                                CalcValue::text(ExcelText::from_interop_assignment(
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
                    ("reference", reference.target().to_string()),
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
        let sparse_reference_values = TreeCalcSparseReferenceValuesBinding {
            reference: self.reference.clone(),
            declared_rows: usize::try_from(self.extent.row_count).unwrap_or(usize::MAX),
            declared_cols: usize::try_from(self.extent.column_count).unwrap_or(usize::MAX),
            defined_cells: self
                .defined_iter()
                .map(|cell| {
                    TreeCalcSparseReferenceCell::new(
                        usize::try_from(cell.coord.row).unwrap_or(usize::MAX),
                        usize::try_from(cell.coord.column).unwrap_or(usize::MAX),
                        cell.value,
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
    pub fn current_row_cell_values(&self) -> BTreeMap<String, CalcValue> {
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
    pub function_registry: FunctionRegistry,
    pub capability_overlay: Option<CapabilityOverlay>,
}

impl Default for TreeCalcTableFormulaRuntimeContext {
    fn default() -> Self {
        let function_registry = builtin_registry().clone();
        Self {
            dialect_id: "oxcalc.treecalc-v1".to_string(),
            capability_profile_id: "host-capabilities:treecalc-v1".to_string(),
            resolution_rule_version: "treecalc-host-resolution:v1".to_string(),
            host_namespace_version: Some("treecalc-host-namespace:v1".to_string()),
            structure_context_version: "treecalc-structure:v1".to_string(),
            registry_snapshot_identity: Some(function_registry.snapshot_identity().stable_key()),
            function_registry,
            capability_overlay: None,
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
    pub value: CalcValue,
    pub prepared_formula_key: String,
    pub dispatch_skeleton_key: String,
    pub plan_template_key: String,
    pub registry_snapshot_identity: Option<String>,
    pub prepared_identity_facts: TreeCalcTableFormulaPreparedIdentityFacts,
    pub host_formula_context: RuntimeHostFormulaContext,
    pub structured_reference_handles: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TreeCalcTableFormulaDryBindReport {
    pub table_id: String,
    pub target_column_id: String,
    pub formula_stable_id: String,
    pub formula_text_version: u64,
    pub formula_text: String,
    pub table_context_identity: String,
    pub row_id: Option<TreeCalcTableRowId>,
    pub row_offset: Option<u32>,
    pub region_kind: TableRegionKind,
    pub caller_context_id: String,
    pub primary_locus: Locus,
    pub host_formula_context: RuntimeHostFormulaContext,
    pub verdict: RuntimeDryBindVerdict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableFormulaPreparedIdentityFacts {
    pub dialect_id: String,
    pub capability_profile_id: String,
    pub resolution_rule_version: String,
    pub host_namespace_version: Option<String>,
    pub table_namespace_version: String,
    pub structure_context_version: String,
    pub table_context_identity: String,
    pub caller_context_id: String,
    pub host_registry_snapshot_identity: Option<String>,
    pub function_registry_snapshot_identity: Option<String>,
    pub capability_overlay_identity: Option<String>,
    pub prepared_formula_key: String,
    pub dispatch_skeleton_key: String,
    pub plan_template_key: String,
}

impl TreeCalcTableFormulaPreparedIdentityFacts {
    #[must_use]
    pub fn identity_fragment(&self) -> String {
        identity_record(
            "treecalc.table_formula_prepared_identity_facts.v1",
            [
                ("dialect_id", self.dialect_id.clone()),
                ("capability_profile_id", self.capability_profile_id.clone()),
                (
                    "resolution_rule_version",
                    self.resolution_rule_version.clone(),
                ),
                (
                    "host_namespace_version",
                    self.host_namespace_version
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
                ),
                (
                    "table_namespace_version",
                    self.table_namespace_version.clone(),
                ),
                (
                    "structure_context_version",
                    self.structure_context_version.clone(),
                ),
                (
                    "table_context_identity",
                    self.table_context_identity.clone(),
                ),
                ("caller_context_id", self.caller_context_id.clone()),
                (
                    "host_registry_snapshot_identity",
                    self.host_registry_snapshot_identity
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
                ),
                (
                    "function_registry_snapshot_identity",
                    self.function_registry_snapshot_identity
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
                ),
                (
                    "capability_overlay_identity",
                    self.capability_overlay_identity
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
                ),
                ("prepared_formula_key", self.prepared_formula_key.clone()),
                ("dispatch_skeleton_key", self.dispatch_skeleton_key.clone()),
                ("plan_template_key", self.plan_template_key.clone()),
            ],
        )
    }
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
    StructuredReferenceDiagnostics {
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

pub fn dry_bind_treecalc_table_column_formula(
    snapshot: &TreeCalcTableNodeSnapshot,
    projection: &TreeCalcTableNodeProjection,
    request: &TreeCalcTableColumnFormulaRuntimeRequest,
) -> Result<TreeCalcTableFormulaDryBindReport, TreeCalcTableFormulaRuntimeError> {
    ensure_formula_projection_matches_snapshot(snapshot, projection)?;
    ensure_body_formula_column(snapshot, &request.target_column_id)?;
    let (row_index, row_id) = snapshot.rows.iter().enumerate().next().ok_or_else(|| {
        TreeCalcTableFormulaRuntimeError::InvalidTargetColumnRange {
            column_id: request.target_column_id.clone(),
            range_ref: "body formula dry-bind requires a data row caller".to_string(),
        }
    })?;
    let row_offset = u32::try_from(row_index).map_err(|_| {
        TreeCalcTableFormulaRuntimeError::InvalidTargetColumnRange {
            column_id: request.target_column_id.clone(),
            range_ref: "row offset overflow".to_string(),
        }
    })?;
    dry_bind_treecalc_table_formula_at_region(
        projection,
        request,
        TreeCalcTableFormulaRuntimeRegion::Data {
            row_id: row_id.clone(),
            row_offset,
        },
    )
}

pub fn dry_bind_treecalc_table_totals_formula(
    snapshot: &TreeCalcTableNodeSnapshot,
    projection: &TreeCalcTableNodeProjection,
    request: &TreeCalcTableColumnFormulaRuntimeRequest,
) -> Result<TreeCalcTableFormulaDryBindReport, TreeCalcTableFormulaRuntimeError> {
    ensure_formula_projection_matches_snapshot(snapshot, projection)?;
    ensure_totals_formula_column(snapshot, &request.target_column_id)?;
    if !snapshot.totals_row_present {
        return Err(TreeCalcTableFormulaRuntimeError::TotalsRowAbsent);
    }
    dry_bind_treecalc_table_formula_at_region(
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
    let structured_reference_bind_records = bind_treecalc_table_formula_structured_references(
        projection,
        request,
        &caller_region,
        &primary_locus,
    )
    .map_err(|(syntax_count, bind_count)| {
        TreeCalcTableFormulaRuntimeError::RuntimeDiagnostics {
            row_id: row_id.clone(),
            syntax_count,
            bind_count,
        }
    })?;
    let diagnostics = structured_reference_bind_records
        .iter()
        .flat_map(|record| treecalc_structured_record_diagnostics(record).into_iter())
        .collect::<Vec<_>>();
    if !diagnostics.is_empty() {
        return Err(
            TreeCalcTableFormulaRuntimeError::StructuredReferenceDiagnostics {
                row_id,
                diagnostics,
            },
        );
    }

    let mut scalar_cell_values = BTreeMap::new();
    let mut sparse_reference_value_bindings = Vec::new();
    for bind_record in &structured_reference_bind_records {
        let reader = TreeCalcTableSparseReader::from_oxfml_bind_record(
            snapshot,
            projection,
            bind_record,
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
    let reference_system_provider = sparse_reference_value_bindings.iter().fold(
        TreeCalcReferenceSystemProvider::sparse_only(),
        |provider, binding| {
            provider
                .with_sparse_reference_values(binding.reference.clone(), binding.resolved_values())
        },
    );
    let mut runtime_environment = RuntimeEnvironment::new()
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
        .with_function_registry(&request.runtime_context.function_registry);
    if let Some(capability_overlay) = &request.runtime_context.capability_overlay {
        runtime_environment = runtime_environment.with_capability_overlay(capability_overlay);
    }
    let result = runtime_environment
        .execute(
            RuntimeFormulaRequest::new(
                FormulaSourceRecord::new(
                    request.formula_stable_id.clone(),
                    request.formula_text_version,
                    request.formula_text.clone(),
                ),
                oxfml_core::interface::TypedContextQueryBundle::default()
                    .with_reference_system_provider(Some(
                        &reference_system_provider
                            as &dyn oxfunc_core::resolver::ReferenceSystemProvider,
                    )),
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

    let prepared_formula_key = result
        .prepared_formula_identity
        .prepared_formula_key
        .clone();
    let dispatch_skeleton_key = result
        .prepared_formula_identity
        .plan_template
        .dispatch_skeleton_key
        .clone();
    let plan_template_key = result
        .prepared_formula_identity
        .plan_template
        .plan_template_key
        .clone();
    let function_registry_snapshot_identity = result
        .prepared_formula_identity
        .registry_snapshot_identity
        .clone();
    let capability_overlay_identity = result
        .prepared_formula_identity
        .capability_overlay_identity
        .clone();
    let prepared_identity_facts = TreeCalcTableFormulaPreparedIdentityFacts {
        dialect_id: host_formula_context.dialect_id.clone(),
        capability_profile_id: host_formula_context.capability_profile_id.clone(),
        resolution_rule_version: host_formula_context.resolution_rule_version.clone(),
        host_namespace_version: host_formula_context.host_namespace_version.clone(),
        table_namespace_version: projection.table_namespace_version.clone(),
        structure_context_version: request.runtime_context.structure_context_version.clone(),
        table_context_identity: projection.table_context_identity.clone(),
        caller_context_id: caller_context_id.clone(),
        host_registry_snapshot_identity: request.runtime_context.registry_snapshot_identity.clone(),
        function_registry_snapshot_identity: function_registry_snapshot_identity.clone(),
        capability_overlay_identity,
        prepared_formula_key: prepared_formula_key.clone(),
        dispatch_skeleton_key: dispatch_skeleton_key.clone(),
        plan_template_key: plan_template_key.clone(),
    };

    Ok(TreeCalcTableFormulaRuntimeCellResult {
        row_id,
        row_offset: region.row_offset(),
        region_kind: region.region_kind(),
        caller_context_id,
        primary_locus,
        value: result.evaluation.oxfunc_value,
        prepared_formula_key,
        dispatch_skeleton_key,
        plan_template_key,
        registry_snapshot_identity: function_registry_snapshot_identity,
        prepared_identity_facts,
        host_formula_context,
        structured_reference_handles: result
            .structured_reference_bind_records
            .into_iter()
            .map(|record| record.bind_record_handle)
            .collect(),
    })
}

fn dry_bind_treecalc_table_formula_at_region(
    projection: &TreeCalcTableNodeProjection,
    request: &TreeCalcTableColumnFormulaRuntimeRequest,
    region: TreeCalcTableFormulaRuntimeRegion,
) -> Result<TreeCalcTableFormulaDryBindReport, TreeCalcTableFormulaRuntimeError> {
    let caller_region = region.caller_region(&projection.table_id);
    let primary_locus =
        treecalc_table_formula_primary_locus(projection, &request.target_column_id, &region)?;
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
    let mut runtime_environment = RuntimeEnvironment::new()
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
        .with_function_registry(&request.runtime_context.function_registry);
    if let Some(capability_overlay) = &request.runtime_context.capability_overlay {
        runtime_environment = runtime_environment.with_capability_overlay(capability_overlay);
    }
    let verdict = runtime_environment.dry_bind_authored_input(FormulaSourceRecord::new(
        request.formula_stable_id.clone(),
        request.formula_text_version,
        request.formula_text.clone(),
    ));
    Ok(TreeCalcTableFormulaDryBindReport {
        table_id: projection.table_id.clone(),
        target_column_id: request.target_column_id.clone(),
        formula_stable_id: request.formula_stable_id.clone(),
        formula_text_version: request.formula_text_version,
        formula_text: request.formula_text.clone(),
        table_context_identity: projection.table_context_identity.clone(),
        row_id: region.row_id(),
        row_offset: region.row_offset(),
        region_kind: region.region_kind(),
        caller_context_id,
        primary_locus,
        host_formula_context,
        verdict,
    })
}

fn bind_treecalc_table_formula_structured_references(
    projection: &TreeCalcTableNodeProjection,
    request: &TreeCalcTableColumnFormulaRuntimeRequest,
    caller_region: &TableCallerRegion,
    primary_locus: &Locus,
) -> Result<Vec<StructuredReferenceBindRecord>, (usize, usize)> {
    let source = FormulaSourceRecord::new(
        request.formula_stable_id.clone(),
        request.formula_text_version,
        request.formula_text.clone(),
    );
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    if !parse.green_tree.diagnostics.is_empty() {
        return Err((parse.green_tree.diagnostics.len(), 0));
    }
    let red_projection = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection,
        context: BindContext {
            workbook_id: projection.table_descriptor.workbook_scope_ref.clone(),
            sheet_id: projection.table_descriptor.sheet_scope_ref.clone(),
            caller_row: primary_locus.row,
            caller_col: primary_locus.col,
            formula_token: source.formula_token(),
            structure_context_version: StructureContextVersion(
                request.runtime_context.structure_context_version.clone(),
            ),
            table_catalog: vec![projection.table_descriptor.clone()],
            enclosing_table_ref: Some(TableRef {
                table_id: projection.table_id.clone(),
            }),
            caller_table_region: Some(caller_region.clone()),
            ..BindContext::default()
        },
        reference_bind_profile: None,
    });
    if !bind.bound_formula.diagnostics.is_empty()
        && bind
            .bound_formula
            .structured_reference_bind_records
            .is_empty()
    {
        return Err((0, bind.bound_formula.diagnostics.len()));
    }
    Ok(bind.bound_formula.structured_reference_bind_records)
}

fn treecalc_structured_record_diagnostics(
    record: &StructuredReferenceBindRecord,
) -> Vec<TreeCalcTableStructuredReferenceDiagnostic> {
    record
        .diagnostics
        .iter()
        .map(|diagnostic| TreeCalcTableStructuredReferenceDiagnostic {
            diagnostic_code: diagnostic.diagnostic_code.clone(),
            message: diagnostic.message.clone(),
            source_span_utf8: diagnostic.source_span_utf8,
        })
        .collect()
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
    TableResize,
    NodeRename,
    NodeMove,
    NodeDelete,
    SaveReopen,
    WorkspaceOpen,
    WorkspaceClose,
    WorkspaceAliasMutation,
    FunctionRegistrySnapshotMutation,
    StructuralRebind,
}

impl TreeCalcTableUpdateScenarioKind {
    pub const ALL: [Self; 25] = [
        Self::BodyCellEdit,
        Self::BodyFormulaEdit,
        Self::RowInsert,
        Self::RowDelete,
        Self::RowReorder,
        Self::ColumnInsert,
        Self::ColumnDelete,
        Self::ColumnReorder,
        Self::ColumnRename,
        Self::HeaderTextEdit,
        Self::TotalsRowToggle,
        Self::TotalsFormulaEdit,
        Self::TableRename,
        Self::TableMove,
        Self::TableDelete,
        Self::TableResize,
        Self::NodeRename,
        Self::NodeMove,
        Self::NodeDelete,
        Self::SaveReopen,
        Self::WorkspaceOpen,
        Self::WorkspaceClose,
        Self::WorkspaceAliasMutation,
        Self::FunctionRegistrySnapshotMutation,
        Self::StructuralRebind,
    ];

    #[must_use]
    pub const fn stable_id(self) -> &'static str {
        match self {
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
            Self::TableResize => "table_resize",
            Self::NodeRename => "node_rename",
            Self::NodeMove => "node_move",
            Self::NodeDelete => "node_delete",
            Self::SaveReopen => "save_reopen",
            Self::WorkspaceOpen => "workspace_open",
            Self::WorkspaceClose => "workspace_close",
            Self::WorkspaceAliasMutation => "workspace_alias_mutation",
            Self::FunctionRegistrySnapshotMutation => "function_registry_snapshot_mutation",
            Self::StructuralRebind => "structural_rebind",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeCalcTablePreparedIdentityInput {
    HostNamespaceVersion,
    StructureContextVersion,
    TableContextIdentity,
    CallerContextIdentity,
    DynamicSelectorIdentity,
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
    TableResize,
    NodeRename,
    NodeMove,
    NodeDelete,
    SaveReopen,
    WorkspaceOpen,
    WorkspaceClose,
    WorkspaceAliasMutation,
    FunctionRegistrySnapshotMutation,
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
            Self::TableResize => TreeCalcTableUpdateScenarioKind::TableResize,
            Self::NodeRename => TreeCalcTableUpdateScenarioKind::NodeRename,
            Self::NodeMove => TreeCalcTableUpdateScenarioKind::NodeMove,
            Self::NodeDelete => TreeCalcTableUpdateScenarioKind::NodeDelete,
            Self::SaveReopen => TreeCalcTableUpdateScenarioKind::SaveReopen,
            Self::WorkspaceOpen => TreeCalcTableUpdateScenarioKind::WorkspaceOpen,
            Self::WorkspaceClose => TreeCalcTableUpdateScenarioKind::WorkspaceClose,
            Self::WorkspaceAliasMutation => TreeCalcTableUpdateScenarioKind::WorkspaceAliasMutation,
            Self::FunctionRegistrySnapshotMutation => {
                TreeCalcTableUpdateScenarioKind::FunctionRegistrySnapshotMutation
            }
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
            Self::TableResize => "table_resize",
            Self::NodeRename => "node_rename",
            Self::NodeMove => "node_move",
            Self::NodeDelete => "node_delete",
            Self::SaveReopen => "save_reopen",
            Self::WorkspaceOpen => "workspace_open",
            Self::WorkspaceClose => "workspace_close",
            Self::WorkspaceAliasMutation => "workspace_alias_mutation",
            Self::FunctionRegistrySnapshotMutation => "function_registry_snapshot_mutation",
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
            TreeCalcTableUpdateScenarioKind::TableResize => Self::TableResize,
            TreeCalcTableUpdateScenarioKind::NodeRename => Self::NodeRename,
            TreeCalcTableUpdateScenarioKind::NodeMove => Self::NodeMove,
            TreeCalcTableUpdateScenarioKind::NodeDelete => Self::NodeDelete,
            TreeCalcTableUpdateScenarioKind::SaveReopen => Self::SaveReopen,
            TreeCalcTableUpdateScenarioKind::WorkspaceOpen => Self::WorkspaceOpen,
            TreeCalcTableUpdateScenarioKind::WorkspaceClose => Self::WorkspaceClose,
            TreeCalcTableUpdateScenarioKind::WorkspaceAliasMutation => Self::WorkspaceAliasMutation,
            TreeCalcTableUpdateScenarioKind::FunctionRegistrySnapshotMutation => {
                Self::FunctionRegistrySnapshotMutation
            }
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
    pub workspace_availability_version: Option<String>,
    pub workspace_alias_version: Option<String>,
}

impl Default for TreeCalcTableLifecycleContextVersions {
    fn default() -> Self {
        Self {
            host_namespace_version: Some("treecalc-host-namespace:v1".to_string()),
            structure_context_version: "treecalc-structure:v1".to_string(),
            registry_snapshot_identity: "oxfunc-registry:default".to_string(),
            resolution_rule_version: "treecalc-host-resolution:v1".to_string(),
            workspace_availability_version: Some("treecalc-workspace-availability:v1".to_string()),
            workspace_alias_version: Some("treecalc-workspace-alias:v1".to_string()),
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
                (
                    "workspace_availability_version",
                    self.workspace_availability_version
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
                ),
                (
                    "workspace_alias_version",
                    self.workspace_alias_version
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
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
        if self.host_namespace_version.is_some()
            || self.workspace_availability_version.is_some()
            || self.workspace_alias_version.is_some()
        {
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
    pub workspace_availability_version: String,
    pub workspace_alias_version: String,
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
            workspace_availability_version: treecalc_table_workspace_availability_version(snapshot),
            workspace_alias_version: treecalc_table_workspace_alias_version(snapshot),
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
                    "workspace_availability_version",
                    self.workspace_availability_version.clone(),
                ),
                (
                    "workspace_alias_version",
                    self.workspace_alias_version.clone(),
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
        TreeCalcTableLifecycleEventKind::TableDelete
        | TreeCalcTableLifecycleEventKind::NodeDelete
        | TreeCalcTableLifecycleEventKind::WorkspaceClose => {
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
        TreeCalcTableLifecycleEventKind::WorkspaceOpen => {
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
        TreeCalcTableUpdateScenarioKind::RowInsert
        | TreeCalcTableUpdateScenarioKind::RowDelete
        | TreeCalcTableUpdateScenarioKind::TableResize => &[
            Kind::StructuredTableRowMembership,
            Kind::StructuredTableRowOrder,
            Kind::StructuredTableDataRegion,
            Kind::StructuredTableTotalsRegion,
            Kind::StructuredTableCallerContext,
            Kind::StructuredTableIdentity,
        ],
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
        | TreeCalcTableUpdateScenarioKind::NodeRename
        | TreeCalcTableUpdateScenarioKind::WorkspaceAliasMutation
        | TreeCalcTableUpdateScenarioKind::StructuralRebind => &[
            Kind::StructuredTableIdentity,
            Kind::StructuredTableEnclosingTable,
        ],
        TreeCalcTableUpdateScenarioKind::TableMove | TreeCalcTableUpdateScenarioKind::NodeMove => {
            &[
                Kind::StructuredTableIdentity,
                Kind::StructuredTableHeaderRegion,
                Kind::StructuredTableDataRegion,
                Kind::StructuredTableTotalsRegion,
                Kind::StructuredTableEnclosingTable,
            ]
        }
        TreeCalcTableUpdateScenarioKind::TableDelete
        | TreeCalcTableUpdateScenarioKind::NodeDelete
        | TreeCalcTableUpdateScenarioKind::WorkspaceClose => &[
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
        TreeCalcTableUpdateScenarioKind::WorkspaceOpen => &[
            Kind::HostSensitive,
            Kind::StructuredTableIdentity,
            Kind::StructuredTableEnclosingTable,
        ],
        TreeCalcTableUpdateScenarioKind::FunctionRegistrySnapshotMutation => {
            &[Kind::CapabilitySensitive]
        }
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
        TreeCalcTableUpdateScenarioKind::RowInsert
        | TreeCalcTableUpdateScenarioKind::RowDelete
        | TreeCalcTableUpdateScenarioKind::TableResize => &[
            Reason::StructuredTableRowMembershipChanged,
            Reason::StructuredTableRowOrderChanged,
            Reason::StructuredTableRegionChanged,
            Reason::StructuredTableCallerContextChanged,
            Reason::StructuredTableContextChanged,
        ],
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
        | TreeCalcTableUpdateScenarioKind::NodeRename
        | TreeCalcTableUpdateScenarioKind::NodeMove
        | TreeCalcTableUpdateScenarioKind::WorkspaceAliasMutation
        | TreeCalcTableUpdateScenarioKind::StructuralRebind => &[
            Reason::StructuredTableContextChanged,
            Reason::StructuralRebindRequired,
        ],
        TreeCalcTableUpdateScenarioKind::TableDelete
        | TreeCalcTableUpdateScenarioKind::NodeDelete
        | TreeCalcTableUpdateScenarioKind::WorkspaceClose => {
            &[Reason::DependencyRemoved, Reason::StructuralRebindRequired]
        }
        TreeCalcTableUpdateScenarioKind::WorkspaceOpen => &[
            Reason::DependencyAdded,
            Reason::StructuredTableContextChanged,
            Reason::StructuralRebindRequired,
        ],
        TreeCalcTableUpdateScenarioKind::FunctionRegistrySnapshotMutation => {
            &[Reason::DependencyReclassified]
        }
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
        | TreeCalcTableUpdateScenarioKind::RowReorder
        | TreeCalcTableUpdateScenarioKind::TableResize => {
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
        | TreeCalcTableUpdateScenarioKind::NodeRename
        | TreeCalcTableUpdateScenarioKind::NodeMove
        | TreeCalcTableUpdateScenarioKind::NodeDelete
        | TreeCalcTableUpdateScenarioKind::WorkspaceOpen
        | TreeCalcTableUpdateScenarioKind::WorkspaceClose
        | TreeCalcTableUpdateScenarioKind::WorkspaceAliasMutation
        | TreeCalcTableUpdateScenarioKind::StructuralRebind
        | TreeCalcTableUpdateScenarioKind::TableDelete => &[
            Input::HostNamespaceVersion,
            Input::TableContextIdentity,
            Input::ResolutionRuleVersion,
        ],
        TreeCalcTableUpdateScenarioKind::FunctionRegistrySnapshotMutation => {
            &[Input::RegistrySnapshotIdentity]
        }
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
    if before.workspace_availability_version != after.workspace_availability_version {
        changed_dependency_kinds.insert(DependencyDescriptorKind::HostSensitive);
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableEnclosingTable);
        invalidation_reasons.insert(InvalidationReasonKind::StructuredTableContextChanged);
        invalidation_reasons.insert(InvalidationReasonKind::StructuralRebindRequired);
        prepared_identity_inputs.insert(TreeCalcTablePreparedIdentityInput::HostNamespaceVersion);
        prepared_identity_inputs.insert(TreeCalcTablePreparedIdentityInput::ResolutionRuleVersion);
    }
    if before.workspace_alias_version != after.workspace_alias_version {
        changed_dependency_kinds.insert(DependencyDescriptorKind::HostSensitive);
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableEnclosingTable);
        invalidation_reasons.insert(InvalidationReasonKind::StructuredTableContextChanged);
        invalidation_reasons.insert(InvalidationReasonKind::StructuralRebindRequired);
        prepared_identity_inputs.insert(TreeCalcTablePreparedIdentityInput::HostNamespaceVersion);
        prepared_identity_inputs.insert(TreeCalcTablePreparedIdentityInput::ResolutionRuleVersion);
    }
    if before.table_invalidation_identity != after.table_invalidation_identity {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableDataRegion);
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableTotalsRegion);
        invalidation_reasons.insert(InvalidationReasonKind::StructuredTableRegionChanged);
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
                value: CalcValue::from(value.clone()),
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
            .map(CalcValue::from)
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
    RowValue,
    ColumnIdentity,
    ColumnOrder,
    HeaderText,
    HeaderRegion,
    DataRegion,
    TotalsRegion,
    TotalsValue,
    TotalsFormula,
    CallerRowContext,
    OmittedTableNameEnclosingTable,
    VirtualAnchorRange,
    WorkspaceAvailability,
    FunctionRegistrySnapshot,
}

impl StructuredTableDependencyFactKind {
    #[must_use]
    pub fn descriptor_kind(self) -> DependencyDescriptorKind {
        match self {
            Self::TableIdentity => DependencyDescriptorKind::StructuredTableIdentity,
            Self::RowMembership => DependencyDescriptorKind::StructuredTableRowMembership,
            Self::RowOrder => DependencyDescriptorKind::StructuredTableRowOrder,
            Self::RowValue => DependencyDescriptorKind::StructuredTableDataRegion,
            Self::ColumnIdentity => DependencyDescriptorKind::StructuredTableColumnIdentity,
            Self::ColumnOrder => DependencyDescriptorKind::StructuredTableColumnIdentity,
            Self::HeaderText => DependencyDescriptorKind::StructuredTableHeaderText,
            Self::HeaderRegion => DependencyDescriptorKind::StructuredTableHeaderRegion,
            Self::DataRegion => DependencyDescriptorKind::StructuredTableDataRegion,
            Self::TotalsRegion => DependencyDescriptorKind::StructuredTableTotalsRegion,
            Self::TotalsValue => DependencyDescriptorKind::StructuredTableTotalsRegion,
            Self::TotalsFormula => DependencyDescriptorKind::StructuredTableTotalsRegion,
            Self::CallerRowContext => DependencyDescriptorKind::StructuredTableCallerContext,
            Self::OmittedTableNameEnclosingTable => {
                DependencyDescriptorKind::StructuredTableEnclosingTable
            }
            Self::VirtualAnchorRange => DependencyDescriptorKind::StructuredTableIdentity,
            Self::WorkspaceAvailability => DependencyDescriptorKind::HostSensitive,
            Self::FunctionRegistrySnapshot => DependencyDescriptorKind::CapabilitySensitive,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcTableDependencyInventory {
    pub table_context_identity: String,
    pub table_invalidation_identity: String,
    pub table_namespace_version: String,
    pub structure_context_version: String,
    pub host_namespace_version: Option<String>,
    pub workspace_availability_version: Option<String>,
    pub workspace_alias_version: Option<String>,
    pub registry_snapshot_identity: String,
    pub facts: Vec<StructuredTableDependencyFact>,
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
    push_virtual_anchor_range_fact(request, table, &mut facts);
    push_row_membership_and_order_facts(request, table, &mut facts);
    push_column_facts(request, table, &mut facts);
    push_region_facts(request, table, &mut facts);
    push_caller_context_fact(request, table, &mut facts);
    push_enclosing_table_fact(request, table, &mut facts);

    lowering_from_facts(request, facts)
}

#[must_use]
pub fn inventory_treecalc_table_dependency_facts(
    snapshot: &TreeCalcTableNodeSnapshot,
    projection: &TreeCalcTableNodeProjection,
    context_versions: &TreeCalcTableLifecycleContextVersions,
    caller_context_id: Option<&str>,
    include_function_registry_snapshot: bool,
) -> TreeCalcTableDependencyInventory {
    let mut facts = Vec::new();
    let table_id = projection.table_id.clone();
    push_inventory_fact(
        &mut facts,
        snapshot,
        StructuredTableDependencyFactKind::TableIdentity,
        "table_identity",
        None,
        projection.table_context_identity.clone(),
        "table identity covers stable table id, namespace token, anchor token, and generic table packet"
            .to_string(),
    );
    push_inventory_fact(
        &mut facts,
        snapshot,
        StructuredTableDependencyFactKind::VirtualAnchorRange,
        "virtual_anchor_range",
        None,
        projection.virtual_anchor_identity.clone(),
        format!(
            "virtual anchor/range covers workbook={}, sheet={}, range={}",
            projection.table_descriptor.workbook_scope_ref,
            projection.table_descriptor.sheet_scope_ref,
            projection.table_descriptor.table_range_ref
        ),
    );
    push_inventory_fact(
        &mut facts,
        snapshot,
        StructuredTableDependencyFactKind::WorkspaceAvailability,
        "workspace_availability",
        None,
        context_versions
            .workspace_availability_version
            .clone()
            .unwrap_or_else(|| treecalc_table_workspace_availability_version(snapshot)),
        "workspace availability is host-owned and enters prepared identity through host namespace context"
            .to_string(),
    );
    push_inventory_fact(
        &mut facts,
        snapshot,
        StructuredTableDependencyFactKind::RowMembership,
        "row_membership",
        None,
        projection.oxcalc_row_membership_identity.clone(),
        "row membership is an OxCalc table fact, not an OxFml semantic".to_string(),
    );
    push_inventory_fact(
        &mut facts,
        snapshot,
        StructuredTableDependencyFactKind::RowOrder,
        "row_order",
        None,
        projection.oxcalc_row_order_identity.clone(),
        "row order is tracked separately from row membership for caller-row invalidation"
            .to_string(),
    );
    push_inventory_fact(
        &mut facts,
        snapshot,
        StructuredTableDependencyFactKind::RowValue,
        "row_value",
        None,
        projection.body_metadata_identity.clone(),
        "row value changes include data cells and body formula metadata for selected columns"
            .to_string(),
    );
    push_inventory_fact(
        &mut facts,
        snapshot,
        StructuredTableDependencyFactKind::ColumnIdentity,
        "column_identity",
        None,
        projection.oxcalc_column_identity.clone(),
        "column identity covers stable ids, header names, and column ranges".to_string(),
    );
    push_inventory_fact(
        &mut facts,
        snapshot,
        StructuredTableDependencyFactKind::ColumnOrder,
        "column_order",
        None,
        identity_list(snapshot.columns.iter().map(|column| {
            identity_record(
                "treecalc.table_column_order_item",
                [
                    ("column_id", column.column_id.clone()),
                    ("ordinal", column.ordinal.to_string()),
                ],
            )
        })),
        "column order is an explicit invalidation fact because structured references preserve display order"
            .to_string(),
    );
    push_inventory_fact(
        &mut facts,
        snapshot,
        StructuredTableDependencyFactKind::HeaderText,
        "header_text",
        None,
        identity_list(snapshot.columns.iter().map(|column| {
            identity_record(
                "treecalc.table_header_text_item",
                [
                    ("column_id", column.column_id.clone()),
                    ("text", column.column_name.clone()),
                ],
            )
        })),
        "header text changes can rebind structured references by displayed column name".to_string(),
    );
    if let Some(header_region_ref) = projection.table_descriptor.header_region_ref.as_ref() {
        push_inventory_fact(
            &mut facts,
            snapshot,
            StructuredTableDependencyFactKind::HeaderRegion,
            "header_region",
            None,
            header_region_ref.clone(),
            "header region is replay-visible for #Headers references".to_string(),
        );
    }
    push_inventory_fact(
        &mut facts,
        snapshot,
        StructuredTableDependencyFactKind::DataRegion,
        "data_region",
        None,
        identity_list(
            projection
                .table_descriptor
                .columns
                .iter()
                .map(|column| column.column_range_ref.clone()),
        ),
        "data region is replay-visible for data and current-row references".to_string(),
    );
    if let Some(totals_region_ref) = projection.table_descriptor.totals_region_ref.as_ref() {
        push_inventory_fact(
            &mut facts,
            snapshot,
            StructuredTableDependencyFactKind::TotalsRegion,
            "totals_region",
            None,
            totals_region_ref.clone(),
            "totals region is replay-visible for #Totals references".to_string(),
        );
        push_inventory_fact(
            &mut facts,
            snapshot,
            StructuredTableDependencyFactKind::TotalsValue,
            "totals_value",
            None,
            totals_region_ref.clone(),
            "totals values are invalidated by totals row edits and totals formula output changes"
                .to_string(),
        );
    }
    push_inventory_fact(
        &mut facts,
        snapshot,
        StructuredTableDependencyFactKind::TotalsFormula,
        "totals_formula",
        None,
        projection.totals_metadata_identity.clone(),
        "totals formula metadata is OxCalc-owned and feeds structure-context/prepared identity"
            .to_string(),
    );
    if let Some(caller_context_id) = caller_context_id {
        push_inventory_fact(
            &mut facts,
            snapshot,
            StructuredTableDependencyFactKind::CallerRowContext,
            "caller_row_context",
            None,
            caller_context_id.to_string(),
            "caller-row context is required for [@Col] and omitted current-row forms".to_string(),
        );
    }
    push_inventory_fact(
        &mut facts,
        snapshot,
        StructuredTableDependencyFactKind::OmittedTableNameEnclosingTable,
        "omitted_table_enclosing_context",
        None,
        table_id,
        "omitted table-name forms depend on the caller's enclosing table context".to_string(),
    );
    if include_function_registry_snapshot {
        push_inventory_fact(
            &mut facts,
            snapshot,
            StructuredTableDependencyFactKind::FunctionRegistrySnapshot,
            "function_registry_snapshot",
            None,
            context_versions.registry_snapshot_identity.clone(),
            "registered-function calls invalidate prepared table formulas through the OxFunc registry snapshot"
                .to_string(),
        );
    }

    TreeCalcTableDependencyInventory {
        table_context_identity: projection.table_context_identity.clone(),
        table_invalidation_identity: projection.table_invalidation_identity.clone(),
        table_namespace_version: projection.table_namespace_version.clone(),
        structure_context_version: context_versions.structure_context_version.clone(),
        host_namespace_version: context_versions.host_namespace_version.clone(),
        workspace_availability_version: context_versions.workspace_availability_version.clone(),
        workspace_alias_version: context_versions.workspace_alias_version.clone(),
        registry_snapshot_identity: context_versions.registry_snapshot_identity.clone(),
        facts,
    }
}

fn push_inventory_fact(
    facts: &mut Vec<StructuredTableDependencyFact>,
    snapshot: &TreeCalcTableNodeSnapshot,
    kind: StructuredTableDependencyFactKind,
    suffix: &str,
    column_id: Option<String>,
    identity: String,
    detail: String,
) {
    facts.push(StructuredTableDependencyFact::lowered(
        format!(
            "inventory:table_node:{}:{}:{}",
            snapshot.table_node_id.0,
            suffix,
            sanitize_identifier(&identity)
        ),
        kind,
        snapshot.table_id.clone(),
        column_id,
        identity,
        detail,
    ));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeCalcDynamicTableReferenceTargetKind {
    Table,
    Column,
    Section,
    CurrentRow,
    CrossWorkspaceTable,
}

impl TreeCalcDynamicTableReferenceTargetKind {
    #[must_use]
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::Table => "table",
            Self::Column => "column",
            Self::Section => "section",
            Self::CurrentRow => "current_row",
            Self::CrossWorkspaceTable => "cross_workspace_table",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeCalcDynamicTableRebindCause {
    SelectorTextChanged,
    DynamicFunctionResultChanged,
    VolatileReevaluation,
    TableLifecycle(TreeCalcTableUpdateScenarioKind),
    UnsupportedRuntimeStructuredReferenceParsing,
    DynamicTargetNotTable,
}

impl TreeCalcDynamicTableRebindCause {
    #[must_use]
    pub fn stable_id(&self) -> &'static str {
        match self {
            Self::SelectorTextChanged => "selector_text_changed",
            Self::DynamicFunctionResultChanged => "dynamic_function_result_changed",
            Self::VolatileReevaluation => "volatile_reevaluation",
            Self::TableLifecycle(scenario) => {
                TreeCalcTableLifecycleEventKind::from(*scenario).stable_id()
            }
            Self::UnsupportedRuntimeStructuredReferenceParsing => {
                "unsupported_runtime_structured_reference_parsing"
            }
            Self::DynamicTargetNotTable => "dynamic_target_not_table",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeCalcDynamicTableRebindStatus {
    ReferencePreserving,
    RebindRequired,
    DeletedTarget,
    UnavailableTarget,
    TypedExclusion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeCalcDynamicTableRebindDiagnosticKind {
    MissingCallerContext,
    UnsupportedRuntimeStructuredReferenceParsing,
    DynamicTargetNotTable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcDynamicTableRebindDiagnostic {
    pub kind: TreeCalcDynamicTableRebindDiagnosticKind,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcDynamicTableRebindRequest {
    pub selector_handle: String,
    pub selector_identity: String,
    pub source_reference_handle: Option<String>,
    pub target_kind: TreeCalcDynamicTableReferenceTargetKind,
    pub cause: TreeCalcDynamicTableRebindCause,
    pub before_resolved_table_identity: Option<String>,
    pub after_resolved_table_identity: Option<String>,
    pub caller_context_id: Option<String>,
    pub context_versions: TreeCalcTableLifecycleContextVersions,
    pub oxfml_structured_bind_packet_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcDynamicTableRebindReport {
    pub dynamic_rebind_identity: String,
    pub selector_handle: String,
    pub selector_identity: String,
    pub source_reference_handle: Option<String>,
    pub target_kind: TreeCalcDynamicTableReferenceTargetKind,
    pub cause: TreeCalcDynamicTableRebindCause,
    pub status: TreeCalcDynamicTableRebindStatus,
    pub dependency_fact_kinds: BTreeSet<StructuredTableDependencyFactKind>,
    pub changed_dependency_kinds: BTreeSet<DependencyDescriptorKind>,
    pub invalidation_reasons: BTreeSet<InvalidationReasonKind>,
    pub prepared_identity_inputs: BTreeSet<TreeCalcTablePreparedIdentityInput>,
    pub diagnostics: Vec<TreeCalcDynamicTableRebindDiagnostic>,
    pub oxfml_generic_bind_packet_available: bool,
    pub oxfunc_opaque_reference_admitted: bool,
}

#[must_use]
pub fn classify_treecalc_dynamic_table_rebind(
    request: &TreeCalcDynamicTableRebindRequest,
) -> TreeCalcDynamicTableRebindReport {
    let mut dependency_fact_kinds = dynamic_table_target_fact_kinds(request.target_kind);
    let mut changed_dependency_kinds = dependency_fact_kinds
        .iter()
        .map(|kind| kind.descriptor_kind())
        .collect::<BTreeSet<_>>();
    changed_dependency_kinds.insert(DependencyDescriptorKind::DynamicPotential);

    let mut invalidation_reasons = BTreeSet::from([
        InvalidationReasonKind::DynamicDependencyActivated,
        InvalidationReasonKind::DynamicDependencyReleased,
        InvalidationReasonKind::DynamicDependencyReclassified,
    ]);
    let mut prepared_identity_inputs = BTreeSet::from([
        TreeCalcTablePreparedIdentityInput::DynamicSelectorIdentity,
        TreeCalcTablePreparedIdentityInput::StructureContextVersion,
        TreeCalcTablePreparedIdentityInput::ResolutionRuleVersion,
        TreeCalcTablePreparedIdentityInput::TableContextIdentity,
    ]);
    if request.context_versions.host_namespace_version.is_some()
        || request
            .context_versions
            .workspace_availability_version
            .is_some()
        || request.context_versions.workspace_alias_version.is_some()
        || matches!(
            request.target_kind,
            TreeCalcDynamicTableReferenceTargetKind::CrossWorkspaceTable
        )
    {
        prepared_identity_inputs.insert(TreeCalcTablePreparedIdentityInput::HostNamespaceVersion);
    }

    let mut diagnostics = Vec::new();
    match &request.cause {
        TreeCalcDynamicTableRebindCause::TableLifecycle(scenario) => {
            changed_dependency_kinds.extend(scenario_changed_dependency_kinds(*scenario));
            invalidation_reasons.extend(scenario_invalidation_reasons(*scenario));
            prepared_identity_inputs.extend(scenario_prepared_identity_inputs(*scenario));
        }
        TreeCalcDynamicTableRebindCause::UnsupportedRuntimeStructuredReferenceParsing => {
            changed_dependency_kinds.insert(DependencyDescriptorKind::Unresolved);
            invalidation_reasons.insert(InvalidationReasonKind::StructuralRebindRequired);
            diagnostics.push(TreeCalcDynamicTableRebindDiagnostic {
                kind: TreeCalcDynamicTableRebindDiagnosticKind::UnsupportedRuntimeStructuredReferenceParsing,
                detail: "dynamic selector would require runtime parsing of TreeCalc structured-reference syntax; OxFml must supply a generic bind packet or OxCalc must return a typed exclusion".to_string(),
            });
        }
        TreeCalcDynamicTableRebindCause::DynamicTargetNotTable => {
            changed_dependency_kinds.insert(DependencyDescriptorKind::Unresolved);
            invalidation_reasons.insert(InvalidationReasonKind::DependencyReclassified);
            diagnostics.push(TreeCalcDynamicTableRebindDiagnostic {
                kind: TreeCalcDynamicTableRebindDiagnosticKind::DynamicTargetNotTable,
                detail: "dynamic selector resolved to a non-table target; table ReferenceLike lowering is not admitted for this target".to_string(),
            });
        }
        TreeCalcDynamicTableRebindCause::SelectorTextChanged
        | TreeCalcDynamicTableRebindCause::DynamicFunctionResultChanged
        | TreeCalcDynamicTableRebindCause::VolatileReevaluation => {}
    }

    if matches!(
        request.target_kind,
        TreeCalcDynamicTableReferenceTargetKind::CurrentRow
    ) {
        changed_dependency_kinds.insert(DependencyDescriptorKind::StructuredTableCallerContext);
        prepared_identity_inputs.insert(TreeCalcTablePreparedIdentityInput::CallerContextIdentity);
        if request.caller_context_id.is_none() {
            diagnostics.push(TreeCalcDynamicTableRebindDiagnostic {
                kind: TreeCalcDynamicTableRebindDiagnosticKind::MissingCallerContext,
                detail:
                    "dynamic current-row table reference requires caller_context_id and caller table region"
                        .to_string(),
            });
        }
    }

    if matches!(
        request.target_kind,
        TreeCalcDynamicTableReferenceTargetKind::CrossWorkspaceTable
    ) {
        dependency_fact_kinds.insert(StructuredTableDependencyFactKind::WorkspaceAvailability);
        changed_dependency_kinds.insert(DependencyDescriptorKind::HostSensitive);
        invalidation_reasons.insert(InvalidationReasonKind::StructuralRebindRequired);
        prepared_identity_inputs.insert(TreeCalcTablePreparedIdentityInput::HostNamespaceVersion);
    }

    if request.before_resolved_table_identity != request.after_resolved_table_identity {
        invalidation_reasons.insert(InvalidationReasonKind::DynamicDependencyReclassified);
    }

    let status = dynamic_table_rebind_status(request, &diagnostics);
    if matches!(
        status,
        TreeCalcDynamicTableRebindStatus::ReferencePreserving
    ) {
        changed_dependency_kinds.clear();
        invalidation_reasons.clear();
        prepared_identity_inputs.clear();
    }
    if matches!(status, TreeCalcDynamicTableRebindStatus::TypedExclusion) {
        dependency_fact_kinds.clear();
    }

    TreeCalcDynamicTableRebindReport {
        dynamic_rebind_identity: dynamic_table_rebind_identity(request),
        selector_handle: request.selector_handle.clone(),
        selector_identity: request.selector_identity.clone(),
        source_reference_handle: request.source_reference_handle.clone(),
        target_kind: request.target_kind,
        cause: request.cause.clone(),
        status,
        dependency_fact_kinds,
        changed_dependency_kinds,
        invalidation_reasons,
        prepared_identity_inputs,
        diagnostics,
        oxfml_generic_bind_packet_available: request.oxfml_structured_bind_packet_available,
        oxfunc_opaque_reference_admitted: !matches!(
            status,
            TreeCalcDynamicTableRebindStatus::TypedExclusion
        ),
    }
}

fn dynamic_table_target_fact_kinds(
    target_kind: TreeCalcDynamicTableReferenceTargetKind,
) -> BTreeSet<StructuredTableDependencyFactKind> {
    match target_kind {
        TreeCalcDynamicTableReferenceTargetKind::Table => BTreeSet::from([
            StructuredTableDependencyFactKind::TableIdentity,
            StructuredTableDependencyFactKind::RowMembership,
            StructuredTableDependencyFactKind::RowOrder,
            StructuredTableDependencyFactKind::ColumnIdentity,
            StructuredTableDependencyFactKind::DataRegion,
        ]),
        TreeCalcDynamicTableReferenceTargetKind::Column => BTreeSet::from([
            StructuredTableDependencyFactKind::TableIdentity,
            StructuredTableDependencyFactKind::RowMembership,
            StructuredTableDependencyFactKind::RowOrder,
            StructuredTableDependencyFactKind::ColumnIdentity,
            StructuredTableDependencyFactKind::DataRegion,
        ]),
        TreeCalcDynamicTableReferenceTargetKind::Section => BTreeSet::from([
            StructuredTableDependencyFactKind::TableIdentity,
            StructuredTableDependencyFactKind::HeaderRegion,
            StructuredTableDependencyFactKind::DataRegion,
            StructuredTableDependencyFactKind::TotalsRegion,
        ]),
        TreeCalcDynamicTableReferenceTargetKind::CurrentRow => BTreeSet::from([
            StructuredTableDependencyFactKind::TableIdentity,
            StructuredTableDependencyFactKind::RowOrder,
            StructuredTableDependencyFactKind::DataRegion,
            StructuredTableDependencyFactKind::CallerRowContext,
        ]),
        TreeCalcDynamicTableReferenceTargetKind::CrossWorkspaceTable => BTreeSet::from([
            StructuredTableDependencyFactKind::TableIdentity,
            StructuredTableDependencyFactKind::RowMembership,
            StructuredTableDependencyFactKind::RowOrder,
            StructuredTableDependencyFactKind::ColumnIdentity,
            StructuredTableDependencyFactKind::DataRegion,
            StructuredTableDependencyFactKind::WorkspaceAvailability,
            StructuredTableDependencyFactKind::VirtualAnchorRange,
        ]),
    }
}

fn dynamic_table_rebind_status(
    request: &TreeCalcDynamicTableRebindRequest,
    diagnostics: &[TreeCalcDynamicTableRebindDiagnostic],
) -> TreeCalcDynamicTableRebindStatus {
    if !diagnostics.is_empty()
        || !request.oxfml_structured_bind_packet_available
        || matches!(
            request.cause,
            TreeCalcDynamicTableRebindCause::UnsupportedRuntimeStructuredReferenceParsing
                | TreeCalcDynamicTableRebindCause::DynamicTargetNotTable
        )
    {
        return TreeCalcDynamicTableRebindStatus::TypedExclusion;
    }
    match request.cause {
        TreeCalcDynamicTableRebindCause::TableLifecycle(
            TreeCalcTableUpdateScenarioKind::TableDelete
            | TreeCalcTableUpdateScenarioKind::NodeDelete,
        ) => TreeCalcDynamicTableRebindStatus::DeletedTarget,
        TreeCalcDynamicTableRebindCause::TableLifecycle(
            TreeCalcTableUpdateScenarioKind::WorkspaceClose,
        ) => TreeCalcDynamicTableRebindStatus::UnavailableTarget,
        TreeCalcDynamicTableRebindCause::TableLifecycle(
            TreeCalcTableUpdateScenarioKind::SaveReopen,
        ) if request.before_resolved_table_identity == request.after_resolved_table_identity => {
            TreeCalcDynamicTableRebindStatus::ReferencePreserving
        }
        _ => TreeCalcDynamicTableRebindStatus::RebindRequired,
    }
}

fn dynamic_table_rebind_identity(request: &TreeCalcDynamicTableRebindRequest) -> String {
    identity_record(
        "treecalc.dynamic_table_rebind.v1",
        [
            ("selector_handle", request.selector_handle.clone()),
            ("selector_identity", request.selector_identity.clone()),
            (
                "source_reference_handle",
                request
                    .source_reference_handle
                    .clone()
                    .unwrap_or_else(|| "none".to_string()),
            ),
            ("target_kind", request.target_kind.stable_id().to_string()),
            ("cause", request.cause.stable_id().to_string()),
            (
                "before",
                request
                    .before_resolved_table_identity
                    .clone()
                    .unwrap_or_else(|| "none".to_string()),
            ),
            (
                "after",
                request
                    .after_resolved_table_identity
                    .clone()
                    .unwrap_or_else(|| "none".to_string()),
            ),
            (
                "caller_context_id",
                request
                    .caller_context_id
                    .clone()
                    .unwrap_or_else(|| "none".to_string()),
            ),
            ("context", request.context_versions.identity_fragment()),
        ],
    )
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

fn push_virtual_anchor_range_fact(
    request: &StructuredTableDependencyLoweringRequest,
    table: &TableDescriptor,
    facts: &mut Vec<StructuredTableDependencyFact>,
) {
    facts.push(StructuredTableDependencyFact::lowered(
        fact_id(request, "virtual_anchor_range", &table.table_id),
        StructuredTableDependencyFactKind::VirtualAnchorRange,
        table.table_id.clone(),
        None,
        format!(
            "table_virtual_anchor_range:v1:table={};workbook={};sheet={};range={}",
            table.table_id, table.workbook_scope_ref, table.sheet_scope_ref, table.table_range_ref
        ),
        "virtual Excel anchor/range is supplied by table_catalog scope and range fields"
            .to_string(),
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

    let selected_columns = selected_columns_or_all(request, table);
    if !selected_columns.is_empty() {
        let order_identity = identity_list(selected_columns.iter().map(|column| {
            identity_record(
                "table_column_order_item",
                [
                    ("column_id", column.column_id.clone()),
                    ("ordinal", column.ordinal.to_string()),
                ],
            )
        }));
        facts.push(StructuredTableDependencyFact::lowered(
            fact_id(request, "column_order", &table.table_id),
            StructuredTableDependencyFactKind::ColumnOrder,
            table.table_id.clone(),
            None,
            format!(
                "table_column_order:v1:table={};columns={order_identity}",
                table.table_id
            ),
            "column order is supplied by TableColumnDescriptor.ordinal".to_string(),
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
        let row_membership = table
            .row_membership_identity
            .as_deref()
            .unwrap_or("missing-row-membership");
        let row_order = table
            .row_order_identity
            .as_deref()
            .unwrap_or("missing-row-order");
        facts.push(StructuredTableDependencyFact::lowered(
            fact_id(request, "row_value", &table.table_id),
            StructuredTableDependencyFactKind::RowValue,
            table.table_id.clone(),
            None,
            format!(
                "table_row_value:v1:table={};row_membership={row_membership};row_order={row_order};columns={ranges}",
                table.table_id
            ),
            "row value dependency is bounded by selected data ranges plus stable row membership/order identities"
                .to_string(),
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
            facts.push(StructuredTableDependencyFact::lowered(
                fact_id(request, "totals_value", totals_region_ref),
                StructuredTableDependencyFactKind::TotalsValue,
                table.table_id.clone(),
                None,
                format!(
                    "table_totals_value:v1:table={};region={totals_region_ref}",
                    table.table_id
                ),
                "totals value dependency is supplied by the exact totals row region identity"
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
    use std::cell::RefCell;
    use std::collections::{BTreeMap, BTreeSet};

    use oxfml_core::{
        EvaluationBackend, EvaluationTraceMode, HostFunctionInvocation, HostFunctionProvider,
        HostFunctionProviderError, LibraryAvailabilityState, RegistrationSourceKind,
        StructuredReferenceBindDiagnosticLink, StructuredReferenceSelectedRegion,
        consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest},
        interface::{TableColumnDescriptor, TableRegionKind, TypedContextQueryBundle},
        seam::Locus,
        source::FormulaSourceRecord,
        syntax::token::TextSpan,
    };
    use oxfunc_core::{
        function::{
            ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass,
            FecDependencyProfile, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass,
            VolatilityClass,
        },
        registry::{
            FunctionAvailability, ParameterDescriptor, UdfExecutionProfile,
            UdfInvocationTargetDescriptor, UdfRegistrationRequest, UdfRegistrationResult,
            UdfReplacementPolicy, UdfSourceKind,
        },
        value::WorksheetErrorCode,
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
                        formula_text: "=SUM([Amount])".to_string(),
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
                            formula_text: "=[@Amount]*0.1".to_string(),
                        },
                    ),
                    totals_metadata: Some(TreeCalcTableFormulaMetadata {
                        formula_artifact_id: "formula:totals:tax".to_string(),
                        bind_artifact_id: None,
                        formula_text_version: "v1".to_string(),
                        formula_text: "=SUM([Tax])".to_string(),
                    }),
                },
            ],
            body_cell_nodes: Vec::new(),
            totals_cell_nodes: Vec::new(),
            header_row_present: true,
            totals_row_present: true,
            table_namespace_version: "namespace:v1".to_string(),
            row_membership_version: "row-membership:v1".to_string(),
            row_order_version: "row-order:v1".to_string(),
            column_identity_version: "columns:v1".to_string(),
        }
    }

    fn projected_treecalc_table() -> TreeCalcTableNodeProjection {
        project_treecalc_table_node_snapshot(&treecalc_table_snapshot())
            .expect("table-node snapshot projects")
    }

    fn alternate_table_snapshot(
        table_node_id: u64,
        table_id: &str,
        table_name: &str,
        display_path: &str,
        canonical_path: &str,
        namespace_version: &str,
    ) -> TreeCalcTableNodeSnapshot {
        let mut snapshot = treecalc_table_snapshot();
        snapshot.table_node_id = TreeNodeId(table_node_id);
        snapshot.table_id = table_id.to_string();
        snapshot.table_name = table_name.to_string();
        snapshot.display_path = display_path.to_string();
        snapshot.canonical_path = canonical_path.to_string();
        snapshot.table_namespace_version = namespace_version.to_string();
        snapshot.virtual_anchor.sheet_scope_ref =
            format!("treecalc-virtual-sheet:tables:{table_node_id}");
        snapshot.virtual_anchor.start_row = u32::try_from(table_node_id).unwrap_or(20);
        snapshot
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
        assert_eq!(projection.table_namespace_version, "namespace:v1");
    }

    #[test]
    fn t1_grid_and_tree_table_backings_agree_modulo_coordinates() {
        use crate::grid::coords::ExcelGridBounds;
        use crate::grid::geometry::GridRect;
        use crate::grid::machine::{GridTableColumn, GridTableOverlay};
        use crate::table_backing::{TableBacking, descriptors_agree_modulo_coordinates};

        // Tree backing for the SalesTable fixture (table tree-table:sales,
        // columns Region/Amount/Tax, header + totals, at a virtual anchor).
        let tree_descriptor = projected_treecalc_table().table_spec().descriptor;

        // Grid backing for the SAME logical table at DIFFERENT sheet coordinates
        // (5 rows: header + 3 data + totals; 3 columns), so the coordinate refs
        // genuinely differ from the tree's virtual anchor.
        let bounds = ExcelGridBounds::strict_excel();
        let rect = |top, left, bottom, right| {
            GridRect::new("wb", "sheet", top, left, bottom, right, bounds).expect("valid grid rect")
        };
        let grid_descriptor = GridTableOverlay::new(
            "tree-table:sales",
            "SalesTable",
            rect(10, 6, 14, 8),
            vec![
                GridTableColumn::new("col:region", "Region", 1, rect(11, 6, 13, 6)),
                GridTableColumn::new("col:amount", "Amount", 2, rect(11, 7, 13, 7)),
                GridTableColumn::new("col:tax", "Tax", 3, rect(11, 8, 13, 8)),
            ],
        )
        .with_header_rect(rect(10, 6, 10, 8))
        .with_totals_rect(rect(14, 6, 14, 8))
        .table_spec()
        .descriptor;

        // T1: the two backings agree on table identity ...
        assert!(descriptors_agree_modulo_coordinates(
            &grid_descriptor,
            &tree_descriptor
        ));
        // ... while their coordinate refs genuinely differ (the comparator
        // ignores coordinates, so this is a non-trivial agreement).
        assert_ne!(
            grid_descriptor.table_range_ref,
            tree_descriptor.table_range_ref
        );
    }

    #[test]
    fn resolve_section_selects_region_refs_over_the_descriptor() {
        use crate::table_backing::{SectionResolution, TableBacking};
        use oxfml_core::StructuredSectionKind;

        let tree = projected_treecalc_table();
        // Headers / Totals sections -> the header / totals region refs.
        assert_eq!(
            tree.resolve_section(StructuredSectionKind::Headers, &[]),
            SectionResolution::Refs(vec!["B3:D3".to_string()])
        );
        assert_eq!(
            tree.resolve_section(StructuredSectionKind::Totals, &[]),
            SectionResolution::Refs(vec!["B7:D7".to_string()])
        );
        // A column filter -> that column's data range ref.
        assert_eq!(
            tree.resolve_section(StructuredSectionKind::Data, &["col:amount".to_string()]),
            SectionResolution::Refs(vec!["C4:C6".to_string()])
        );
        // All without a column filter -> the whole table range.
        assert_eq!(
            tree.resolve_section(StructuredSectionKind::All, &[]),
            SectionResolution::Refs(vec!["B3:D7".to_string()])
        );
    }

    #[test]
    fn table_catalog_resolver_emits_handles_versions_and_source_for_current_workspace() {
        let projection = projected_treecalc_table();
        let context = TreeCalcTableCatalogResolverContext::for_current_workspace(
            "treecalc-workspace:main",
            vec![projection.clone()],
        );
        let request = TreeCalcTableCatalogResolveRequest {
            selector_token_text: "SalesTable".to_string(),
            selector_kind: TreeCalcTableCatalogSelectorKind::TableNameOrPath,
            source_span_utf8: Some(TextSpan::new(4, "SalesTable".len())),
            caller_node_id: None,
            caller_table_region: None,
            caller_context_id: None,
        };

        let resolved = resolve_treecalc_table_catalog_reference(&context, &request);

        assert_eq!(
            resolved.resolution_layer,
            TreeCalcTableCatalogResolutionLayer::CurrentWorkspaceTableName
        );
        assert_eq!(resolved.shape_hint, TreeCalcTableCatalogShapeHint::Table);
        assert_eq!(
            resolved.table_reference_handle.split(';').next(),
            Some("treecalc.table_catalog.reference_handle.token.v1")
        );
        assert_eq!(resolved.source_span_utf8, request.source_span_utf8);
        assert_eq!(resolved.source_token_text, "SalesTable");
        assert_eq!(
            resolved.effective_table_id.as_deref(),
            Some("tree-table:sales")
        );
        assert_eq!(resolved.table_node_id, Some(TreeNodeId(20)));
        assert_eq!(
            resolved.virtual_anchor_identity.as_deref(),
            Some(projection.virtual_anchor_identity.as_str())
        );
        assert_eq!(
            resolved.host_namespace_version,
            "treecalc-host-namespace:v1"
        );
        assert_eq!(
            resolved.table_namespace_version.as_deref(),
            Some("namespace:v1")
        );
        assert_eq!(resolved.structure_context_version, "treecalc-structure:v1");
        assert_eq!(
            resolved.resolution_rule_version,
            "treecalc-host-resolution:v1"
        );
        assert_eq!(
            resolved.workspace_availability_version.as_deref(),
            Some("treecalc-table-workspace-availability:v1:treecalc-workspace:main:available")
        );
        assert!(!resolved.caller_context_dependency);
        assert!(resolved.caller_context_id.is_none());
        assert!(resolved.diagnostics.is_empty());
        assert!(
            resolved
                .opaque_selector
                .contains("local_selector=10:SalesTable")
        );
    }

    #[test]
    fn table_catalog_resolver_handles_same_node_and_omitted_caller_table() {
        let projection = projected_treecalc_table();
        let context = TreeCalcTableCatalogResolverContext::for_current_workspace(
            "treecalc-workspace:main",
            vec![projection],
        );
        let same_node = TreeCalcTableCatalogResolveRequest {
            selector_token_text: "@TABLE".to_string(),
            selector_kind: TreeCalcTableCatalogSelectorKind::SameNodeTable,
            source_span_utf8: Some(TextSpan::new(0, "@TABLE".len())),
            caller_node_id: Some(TreeNodeId(20)),
            caller_table_region: None,
            caller_context_id: Some("caller:node:20".to_string()),
        };

        let resolved_same_node = resolve_treecalc_table_catalog_reference(&context, &same_node);
        assert_eq!(
            resolved_same_node.resolution_layer,
            TreeCalcTableCatalogResolutionLayer::SameNodeTable
        );
        assert_eq!(
            resolved_same_node.effective_table_id.as_deref(),
            Some("tree-table:sales")
        );
        assert!(resolved_same_node.caller_context_dependency);
        assert!(
            resolved_same_node
                .caller_context_id
                .as_deref()
                .is_some_and(|identity| identity.contains("caller:node:20"))
        );

        let omitted = TreeCalcTableCatalogResolveRequest {
            selector_token_text: "[#This Row]".to_string(),
            selector_kind: TreeCalcTableCatalogSelectorKind::OmittedTableName,
            source_span_utf8: Some(TextSpan::new(2, "[#This Row]".len())),
            caller_node_id: None,
            caller_table_region: Some(TableCallerRegion {
                table_id: "tree-table:sales".to_string(),
                region_kind: TableRegionKind::Data,
                data_row_offset: Some(1),
            }),
            caller_context_id: None,
        };

        let resolved_omitted = resolve_treecalc_table_catalog_reference(&context, &omitted);
        assert_eq!(
            resolved_omitted.resolution_layer,
            TreeCalcTableCatalogResolutionLayer::OmittedCallerTable
        );
        assert_eq!(
            resolved_omitted.shape_hint,
            TreeCalcTableCatalogShapeHint::CallerContextTable
        );
        assert_eq!(
            resolved_omitted.effective_table_id.as_deref(),
            Some("tree-table:sales")
        );
        assert!(resolved_omitted.caller_context_dependency);
        assert!(
            resolved_omitted
                .caller_context_id
                .as_deref()
                .is_some_and(|identity| identity.contains("caller_table_region"))
        );

        let mut omitted_row_two = omitted;
        omitted_row_two.caller_context_id = Some("coarse-caller-context".to_string());
        omitted_row_two.caller_table_region = Some(TableCallerRegion {
            table_id: "tree-table:sales".to_string(),
            region_kind: TableRegionKind::Data,
            data_row_offset: Some(2),
        });
        let mut omitted_row_one_same_coarse_id = omitted_row_two.clone();
        omitted_row_one_same_coarse_id.caller_table_region = Some(TableCallerRegion {
            table_id: "tree-table:sales".to_string(),
            region_kind: TableRegionKind::Data,
            data_row_offset: Some(1),
        });
        let resolved_row_one =
            resolve_treecalc_table_catalog_reference(&context, &omitted_row_one_same_coarse_id);
        let resolved_row_two = resolve_treecalc_table_catalog_reference(&context, &omitted_row_two);
        assert_ne!(
            resolved_row_one.table_reference_handle,
            resolved_row_two.table_reference_handle
        );
    }

    #[test]
    fn table_catalog_resolver_handles_root_alias_and_cross_workspace_paths() {
        let current_projection = projected_treecalc_table();
        let projection_table = project_treecalc_table_node_snapshot(&alternate_table_snapshot(
            30,
            "tree-table:projections",
            "ProjectionTable",
            "Projection Table",
            "Root/Reports/ProjectionTable",
            "namespace:projections:v1",
        ))
        .unwrap();
        let context = TreeCalcTableCatalogResolverContext::for_current_workspace(
            "treecalc-workspace:main",
            vec![current_projection],
        )
        .with_workspace_alias("Proj", "treecalc-workspace:projections")
        .with_workspace(TreeCalcTableCatalogWorkspace {
            workspace_handle: "treecalc-workspace:projections".to_string(),
            availability_packet: TreeCalcCrossWorkspaceAvailabilityPacket::available(
                "treecalc-workspace:projections",
                "Proj",
                "treecalc-table-workspace-availability:v1:projections:loaded",
            ),
            table_projections: vec![projection_table],
        });

        let rooted = resolve_treecalc_table_catalog_reference(
            &context,
            &TreeCalcTableCatalogResolveRequest::table_name_or_path("!Root/SalesTable"),
        );
        assert_eq!(
            rooted.resolution_layer,
            TreeCalcTableCatalogResolutionLayer::CurrentWorkspaceRoot
        );
        assert_eq!(
            rooted.effective_table_id.as_deref(),
            Some("tree-table:sales")
        );

        let aliased = resolve_treecalc_table_catalog_reference(
            &context,
            &TreeCalcTableCatalogResolveRequest::table_name_or_path("Proj!ProjectionTable"),
        );
        assert_eq!(
            aliased.resolution_layer,
            TreeCalcTableCatalogResolutionLayer::WorkspaceAlias
        );
        assert_eq!(
            aliased.effective_table_id.as_deref(),
            Some("tree-table:projections")
        );
        assert_eq!(
            aliased.workspace_availability_version.as_deref(),
            Some("treecalc-table-workspace-availability:v1:projections:loaded")
        );
        assert!(
            aliased
                .opaque_selector
                .contains("workspace_selector=4:Proj")
        );

        let direct = resolve_treecalc_table_catalog_reference(
            &context,
            &TreeCalcTableCatalogResolveRequest::table_name_or_path(
                "treecalc-workspace:projections!ProjectionTable",
            ),
        );
        assert_eq!(
            direct.resolution_layer,
            TreeCalcTableCatalogResolutionLayer::DirectWorkspace
        );
        assert_eq!(
            direct.effective_table_id.as_deref(),
            Some("tree-table:projections")
        );
    }

    #[test]
    fn table_catalog_resolver_matches_stable_table_ids_as_opaque_exact_handles() {
        let context = TreeCalcTableCatalogResolverContext::for_current_workspace(
            "treecalc-workspace:main",
            vec![projected_treecalc_table()],
        );
        let exact_request = TreeCalcTableCatalogResolveRequest {
            selector_token_text: "tree-table:sales".to_string(),
            selector_kind: TreeCalcTableCatalogSelectorKind::StableTableId,
            source_span_utf8: None,
            caller_node_id: None,
            caller_table_region: None,
            caller_context_id: None,
        };
        let resolved_exact = resolve_treecalc_table_catalog_reference(&context, &exact_request);
        assert_eq!(
            resolved_exact.resolution_layer,
            TreeCalcTableCatalogResolutionLayer::StableTableId
        );
        assert_eq!(
            resolved_exact.effective_table_id.as_deref(),
            Some("tree-table:sales")
        );

        let wrong_case_request = TreeCalcTableCatalogResolveRequest {
            selector_token_text: "TREE-TABLE:SALES".to_string(),
            ..exact_request
        };
        let resolved_wrong_case =
            resolve_treecalc_table_catalog_reference(&context, &wrong_case_request);
        assert!(resolved_wrong_case.effective_table_id.is_none());
        assert!(resolved_wrong_case.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == TreeCalcTableCatalogDiagnosticCode::TableNotFound
        }));
    }

    #[test]
    fn table_catalog_resolver_keeps_bang_inside_bracket_escaped_table_token() {
        let bracketed_table = project_treecalc_table_node_snapshot(&alternate_table_snapshot(
            32,
            "tree-table:sales-bang",
            "Sales!Table",
            "Sales!Table",
            "Root/Sales!Table",
            "namespace:sales-bang:v1",
        ))
        .unwrap();
        let context = TreeCalcTableCatalogResolverContext::for_current_workspace(
            "treecalc-workspace:main",
            vec![bracketed_table],
        );

        let resolved = resolve_treecalc_table_catalog_reference(
            &context,
            &TreeCalcTableCatalogResolveRequest::table_name_or_path("[Sales!Table]"),
        );

        assert_eq!(
            resolved.resolution_layer,
            TreeCalcTableCatalogResolutionLayer::CurrentWorkspacePath
        );
        assert_eq!(
            resolved.effective_table_id.as_deref(),
            Some("tree-table:sales-bang")
        );
        assert!(
            !resolved
                .opaque_selector
                .contains("workspace_selector=6:[Sales")
        );
    }

    #[test]
    fn table_catalog_resolver_reports_collisions_and_w074_mapping_adjacency() {
        let projection = projected_treecalc_table();
        let collision = project_treecalc_table_node_snapshot(&alternate_table_snapshot(
            31,
            "tree-table:sales-copy",
            "SalesTable",
            "Sales Table Copy",
            "Root/SalesTableCopy",
            "namespace:copy:v1",
        ))
        .unwrap();
        let context = TreeCalcTableCatalogResolverContext::for_current_workspace(
            "treecalc-workspace:main",
            vec![projection, collision],
        );

        let ambiguous = resolve_treecalc_table_catalog_reference(
            &context,
            &TreeCalcTableCatalogResolveRequest::table_name_or_path("SalesTable"),
        );
        assert!(ambiguous.effective_table_id.is_none());
        assert!(ambiguous.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == TreeCalcTableCatalogDiagnosticCode::AmbiguousTableSelector
        }));

        let adjacency = TreeCalcTableNamespaceAdjacency {
            host_names: BTreeSet::from(["SalesTable".to_string()]),
            function_names: BTreeSet::from(["SalesTable".to_string()]),
            defined_names: BTreeSet::from(["SalesTable".to_string()]),
            lambda_valued_node_names: BTreeSet::from(["SalesTable".to_string()]),
        };
        let context = TreeCalcTableCatalogResolverContext::for_current_workspace(
            "treecalc-workspace:main",
            vec![projected_treecalc_table()],
        )
        .with_namespace_adjacency(adjacency);

        let resolved = resolve_treecalc_table_catalog_reference(
            &context,
            &TreeCalcTableCatalogResolveRequest::table_name_or_path("SalesTable"),
        );
        assert_eq!(
            resolved.effective_table_id.as_deref(),
            Some("tree-table:sales")
        );
        let diagnostic_codes = resolved
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<BTreeSet<_>>();
        assert!(
            diagnostic_codes
                .contains(&TreeCalcTableCatalogDiagnosticCode::HostNameAdjacencyW074Mapping)
        );
        assert!(
            diagnostic_codes
                .contains(&TreeCalcTableCatalogDiagnosticCode::FunctionNameAdjacencyW074Mapping)
        );
        assert!(
            diagnostic_codes
                .contains(&TreeCalcTableCatalogDiagnosticCode::DefinedNameAdjacencyW074Mapping)
        );
        assert!(
            diagnostic_codes.contains(
                &TreeCalcTableCatalogDiagnosticCode::LambdaValuedNodeAdjacencyW074Mapping
            )
        );
    }

    #[test]
    fn table_catalog_resolver_reports_unavailable_and_deleted_tables() {
        let context = TreeCalcTableCatalogResolverContext::for_current_workspace(
            "treecalc-workspace:main",
            vec![projected_treecalc_table()],
        )
        .with_workspace_alias("Closed", "treecalc-workspace:closed")
        .with_workspace(TreeCalcTableCatalogWorkspace {
            workspace_handle: "treecalc-workspace:closed".to_string(),
            availability_packet: TreeCalcCrossWorkspaceAvailabilityPacket::unavailable(
                "treecalc-workspace:closed",
                "Closed",
                "treecalc-table-workspace-availability:v1:closed:unavailable",
                "closed workspace is not loaded",
            ),
            table_projections: Vec::new(),
        });

        let unavailable = resolve_treecalc_table_catalog_reference(
            &context,
            &TreeCalcTableCatalogResolveRequest::table_name_or_path("Closed!SalesTable"),
        );
        assert_eq!(
            unavailable.resolution_layer,
            TreeCalcTableCatalogResolutionLayer::UnavailableWorkspace
        );
        assert_eq!(
            unavailable.shape_hint,
            TreeCalcTableCatalogShapeHint::UnavailableWorkspace
        );
        assert_eq!(
            unavailable.workspace_availability_version.as_deref(),
            Some("treecalc-table-workspace-availability:v1:closed:unavailable")
        );
        assert!(unavailable.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == TreeCalcTableCatalogDiagnosticCode::WorkspaceUnavailable
        }));

        let deleted_context = TreeCalcTableCatalogResolverContext::for_current_workspace(
            "treecalc-workspace:main",
            Vec::new(),
        )
        .with_deleted_table(TreeCalcTableDeletedFact {
            workspace_handle: "treecalc-workspace:main".to_string(),
            table_id: "tree-table:sales".to_string(),
            selector_token_text: "SalesTable".to_string(),
            table_namespace_version: "namespace:deleted:v1".to_string(),
        });
        let deleted = resolve_treecalc_table_catalog_reference(
            &deleted_context,
            &TreeCalcTableCatalogResolveRequest::table_name_or_path("SalesTable"),
        );
        assert_eq!(
            deleted.resolution_layer,
            TreeCalcTableCatalogResolutionLayer::DeletedTable
        );
        assert_eq!(
            deleted.shape_hint,
            TreeCalcTableCatalogShapeHint::DeletedTable
        );
        assert_eq!(
            deleted.table_namespace_version.as_deref(),
            Some("namespace:deleted:v1")
        );
        assert!(deleted.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == TreeCalcTableCatalogDiagnosticCode::TableDeleted
        }));
    }

    #[test]
    fn table_catalog_resolver_handle_changes_on_alias_and_namespace_mutation() {
        let projection_v1 = project_treecalc_table_node_snapshot(&alternate_table_snapshot(
            40,
            "tree-table:projection-v1",
            "ProjectionTable",
            "Projection Table",
            "Root/ProjectionTable",
            "namespace:projection:v1",
        ))
        .unwrap();
        let projection_v2 = project_treecalc_table_node_snapshot(&alternate_table_snapshot(
            41,
            "tree-table:projection-v2",
            "ProjectionTable",
            "Projection Table",
            "Root/ProjectionTable",
            "namespace:projection:v2",
        ))
        .unwrap();
        let context_v1 = TreeCalcTableCatalogResolverContext::for_current_workspace(
            "treecalc-workspace:main",
            Vec::new(),
        )
        .with_workspace_alias("Proj", "treecalc-workspace:projections-v1")
        .with_workspace(TreeCalcTableCatalogWorkspace {
            workspace_handle: "treecalc-workspace:projections-v1".to_string(),
            availability_packet: TreeCalcCrossWorkspaceAvailabilityPacket::available(
                "treecalc-workspace:projections-v1",
                "Proj",
                "treecalc-table-workspace-availability:v1:projections-v1:loaded",
            ),
            table_projections: vec![projection_v1],
        });
        let mut context_v2 = TreeCalcTableCatalogResolverContext::for_current_workspace(
            "treecalc-workspace:main",
            Vec::new(),
        )
        .with_workspace_alias("Proj", "treecalc-workspace:projections-v2")
        .with_workspace(TreeCalcTableCatalogWorkspace {
            workspace_handle: "treecalc-workspace:projections-v2".to_string(),
            availability_packet: TreeCalcCrossWorkspaceAvailabilityPacket::available(
                "treecalc-workspace:projections-v2",
                "Proj",
                "treecalc-table-workspace-availability:v1:projections-v2:loaded",
            ),
            table_projections: vec![projection_v2],
        });
        context_v2.host_namespace_version = "treecalc-host-namespace:v2".to_string();

        let request =
            TreeCalcTableCatalogResolveRequest::table_name_or_path("Proj!ProjectionTable");
        let resolved_v1 = resolve_treecalc_table_catalog_reference(&context_v1, &request);
        let resolved_v2 = resolve_treecalc_table_catalog_reference(&context_v2, &request);

        assert_eq!(
            resolved_v1.effective_table_id.as_deref(),
            Some("tree-table:projection-v1")
        );
        assert_eq!(
            resolved_v2.effective_table_id.as_deref(),
            Some("tree-table:projection-v2")
        );
        assert_ne!(
            resolved_v1.table_reference_handle,
            resolved_v2.table_reference_handle
        );
        assert_eq!(
            resolved_v2.table_namespace_version.as_deref(),
            Some("namespace:projection:v2")
        );
        assert_eq!(
            resolved_v2.host_namespace_version,
            "treecalc-host-namespace:v2"
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
                formula_text: "=[@Amount]*0.1".to_string(),
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
                formula_text: "=[@Amount]*0.1".to_string(),
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
    fn table_sparse_reader_projects_data_column_without_dense_blanks() {
        let snapshot = treecalc_table_snapshot();
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let bind_records =
            table_formula_bind_records(&projection, "=SUM(SalesTable[Amount])", None);
        let reader = TreeCalcTableSparseReader::from_oxfml_bind_record(
            &snapshot,
            &projection,
            &bind_records[0],
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
            SparseCellRead::Defined(CalcValue::number(3.0))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(2, 1)),
            SparseCellRead::Defined(CalcValue::text(ExcelText::from_interop_assignment("")))
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
    fn table_sparse_reader_projects_whole_data_body_with_order_errors_and_contains() {
        let snapshot = treecalc_table_snapshot();
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let whole_data_body = StructuredTableReferenceIntake::explicit_table(
            "structured-ref:data-body",
            "tree-table:sales",
        );
        let reader = TreeCalcTableSparseReader::from_reference_intake(
            &snapshot,
            &projection,
            &whole_data_body,
            None,
            [
                TreeCalcTableSparseValue::data(
                    "row:west",
                    "col:region",
                    CalcValue::text(ExcelText::from_interop_assignment("West")),
                ),
                TreeCalcTableSparseValue::data("row:west", "col:amount", CalcValue::number(3.0)),
                TreeCalcTableSparseValue::data(
                    "row:west",
                    "col:tax",
                    CalcValue::error(WorksheetErrorCode::Value),
                ),
                TreeCalcTableSparseValue::data(
                    "row:east",
                    "col:region",
                    CalcValue::text(ExcelText::from_interop_assignment("")),
                ),
                TreeCalcTableSparseValue::data(
                    "row:east",
                    "col:amount",
                    CalcValue::text(ExcelText::from_interop_assignment("")),
                ),
                TreeCalcTableSparseValue::data("row:north", "col:tax", CalcValue::number(1.5)),
            ],
            "SalesTable",
            "structured-ref:data-body",
            None,
        )
        .expect("whole table data-body reader should build");

        assert_eq!(reader.declared_extent().row_count, 3);
        assert_eq!(reader.declared_extent().column_count, 3);
        assert_eq!(reader.defined_cardinality(), 6);
        assert_eq!(
            reader.reference(),
            &ReferenceLike::new(ReferenceKind::Area, "treecalc-virtual-sheet:tables!B4:D6")
        );
        assert!(reader.contains(SparseCellCoord::new(3, 3)));
        assert!(!reader.contains(SparseCellCoord::new(4, 1)));
        assert_eq!(
            reader.read_at(SparseCellCoord::new(1, 1)),
            SparseCellRead::Defined(CalcValue::text(ExcelText::from_interop_assignment("West")))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(1, 3)),
            SparseCellRead::Defined(CalcValue::error(WorksheetErrorCode::Value))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(2, 2)),
            SparseCellRead::Defined(CalcValue::text(ExcelText::from_interop_assignment("")))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(3, 2)),
            SparseCellRead::Blank
        );
        assert_eq!(
            reader
                .defined_iter()
                .map(|cell| cell.coord)
                .collect::<Vec<_>>(),
            vec![
                SparseCellCoord::new(1, 1),
                SparseCellCoord::new(1, 2),
                SparseCellCoord::new(1, 3),
                SparseCellCoord::new(2, 1),
                SparseCellCoord::new(2, 2),
                SparseCellCoord::new(3, 3),
            ]
        );
        assert_eq!(reader.access_summary().contains_calls, 2);
    }

    #[test]
    fn table_sparse_reader_preserves_single_row_shape_and_row_order_identity() {
        let mut single = treecalc_table_snapshot();
        single.rows = vec![TreeCalcTableRowId("row:west".to_string())];
        single.row_membership_version = "row-membership:single".to_string();
        single.row_order_version = "row-order:single".to_string();
        let single_projection = project_treecalc_table_node_snapshot(&single).unwrap();
        let whole_data_body = StructuredTableReferenceIntake::explicit_table(
            "structured-ref:data-body",
            "tree-table:sales",
        );
        let single_reader = TreeCalcTableSparseReader::from_reference_intake(
            &single,
            &single_projection,
            &whole_data_body,
            None,
            [TreeCalcTableSparseValue::data(
                "row:west",
                "col:amount",
                CalcValue::number(3.0),
            )],
            "SalesTable",
            "structured-ref:data-body",
            None,
        )
        .expect("single-row data-body reader should build");

        assert_eq!(single_reader.declared_extent().row_count, 1);
        assert_eq!(single_reader.declared_extent().column_count, 3);
        assert_eq!(
            single_reader.reference(),
            &ReferenceLike::new(ReferenceKind::Area, "treecalc-virtual-sheet:tables!B4:D4")
        );

        let snapshot = treecalc_table_snapshot();
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let before = TreeCalcTableSparseReader::from_reference_intake(
            &snapshot,
            &projection,
            &whole_data_body,
            None,
            [
                TreeCalcTableSparseValue::data(
                    "row:west",
                    "col:region",
                    CalcValue::text(ExcelText::from_interop_assignment("West")),
                ),
                TreeCalcTableSparseValue::data(
                    "row:north",
                    "col:region",
                    CalcValue::text(ExcelText::from_interop_assignment("North")),
                ),
            ],
            "SalesTable",
            "structured-ref:data-body",
            None,
        )
        .unwrap();
        let mut reordered = snapshot.clone();
        reordered.rows.reverse();
        reordered.row_order_version = "row-order:reversed".to_string();
        let reordered_projection = project_treecalc_table_node_snapshot(&reordered).unwrap();
        let after = TreeCalcTableSparseReader::from_reference_intake(
            &reordered,
            &reordered_projection,
            &whole_data_body,
            None,
            [
                TreeCalcTableSparseValue::data(
                    "row:west",
                    "col:region",
                    CalcValue::text(ExcelText::from_interop_assignment("West")),
                ),
                TreeCalcTableSparseValue::data(
                    "row:north",
                    "col:region",
                    CalcValue::text(ExcelText::from_interop_assignment("North")),
                ),
            ],
            "SalesTable",
            "structured-ref:data-body",
            None,
        )
        .unwrap();

        assert_eq!(
            before.reader_identity().reader_id,
            after.reader_identity().reader_id
        );
        assert_eq!(
            before.reader_identity().source_identity,
            after.reader_identity().source_identity
        );
        assert_ne!(
            before.reader_identity().snapshot_identity,
            after.reader_identity().snapshot_identity
        );
        assert_eq!(
            after.read_at(SparseCellCoord::new(1, 1)),
            SparseCellRead::Defined(CalcValue::text(ExcelText::from_interop_assignment("North")))
        );
        assert_eq!(
            after.read_at(SparseCellCoord::new(3, 1)),
            SparseCellRead::Defined(CalcValue::text(ExcelText::from_interop_assignment("West")))
        );
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
                CalcValue::number(0.0),
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
            SparseCellRead::Defined(CalcValue::text(ExcelText::from_interop_assignment(
                "Amount"
            )))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(2, 1)),
            SparseCellRead::Defined(CalcValue::number(0.0))
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
            ("=SUM(SalesTable[Amount])", CalcValue::number(3.0)),
            ("=COUNT(SalesTable[Amount])", CalcValue::number(1.0)),
            ("=COUNTA(SalesTable[Amount])", CalcValue::number(2.0)),
            ("=COUNTBLANK(SalesTable[Amount])", CalcValue::number(2.0)),
        ];

        for (formula, expected) in cases {
            let snapshot = runtime_treecalc_table_snapshot();
            let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
            let bind_records = table_formula_bind_records(&projection, formula, None);
            let reader = TreeCalcTableSparseReader::from_oxfml_bind_record(
                &snapshot,
                &projection,
                &bind_records[0],
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

            let reference_system_provider = TreeCalcReferenceSystemProvider::sparse_only()
                .with_sparse_reference_values(
                    runtime_binding.sparse_reference_values.reference.clone(),
                    runtime_binding.sparse_reference_values.resolved_values(),
                );
            let result = RuntimeEnvironment::new()
                .with_primary_locus(table_primary_locus(&table_descriptor))
                .with_table_context(vec![table_descriptor], None, None)
                .execute(
                    RuntimeFormulaRequest::new(
                        FormulaSourceRecord::new("runtime:w056-tree-table-aggregate", 1, formula),
                        TypedContextQueryBundle::default().with_reference_system_provider(Some(
                            &reference_system_provider
                                as &dyn oxfunc_core::resolver::ReferenceSystemProvider,
                        )),
                    )
                    .with_backend(EvaluationBackend::OxFuncBacked),
                )
                .expect("table sparse aggregate should execute through OxFml/OxFunc");

            assert_eq!(result.evaluation.oxfunc_value, expected, "{formula}");
            assert_eq!(sparse_reference.target(), "C4:C6");
        }
    }

    #[test]
    fn table_current_row_reference_reaches_registry_backed_host_udf_boundary() {
        let snapshot = runtime_treecalc_table_snapshot();
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let source = "=TABLE_IDENTITY_PROBE([@Amount])";
        let caller_region = TableCallerRegion {
            table_id: "tree-table:sales".to_string(),
            region_kind: TableRegionKind::Data,
            data_row_offset: Some(1),
        };
        let primary_locus = Locus {
            sheet_id: "sheet:default".to_string(),
            row: 5,
            col: 4,
        };
        let bind_records =
            table_formula_bind_records(&projection, source, Some(caller_region.clone()));
        assert_eq!(bind_records.len(), 1);
        assert_eq!(bind_records[0].source_token_text, "[@Amount]");

        let reader = TreeCalcTableSparseReader::from_oxfml_bind_record(
            &snapshot,
            &projection,
            &bind_records[0],
            Some(&caller_region),
            table_formula_amount_values(),
        )
        .expect("current-row table reader should build for UDF argument");
        let runtime_binding = reader.runtime_binding();
        assert_eq!(runtime_binding.reference.kind(), ReferenceKind::A1);
        assert_eq!(runtime_binding.reference.target(), "C5");
        assert_eq!(
            runtime_binding.scalar_cell_values.get("C5"),
            Some(&CalcValue::number(20.0))
        );

        let function_registry = registry_with_host_callback_test_udf();
        let registry_snapshot_identity = function_registry.snapshot_identity().stable_key();
        let provider = RecordingTableUdfProvider::default();
        let host_formula_context = RuntimeHostFormulaContext {
            dialect_id: "oxcalc.treecalc-v1".to_string(),
            capability_profile_id: "host-capabilities:treecalc-v1".to_string(),
            resolution_rule_version: "treecalc-host-resolution:v1".to_string(),
            host_namespace_version: Some("treecalc-host-namespace:v1".to_string()),
            registry_snapshot_identity: Some(registry_snapshot_identity.clone()),
            structure_context_version: Some("treecalc-structure:v1".to_string()),
            caller_context_identity: Some(treecalc_table_formula_caller_context_id(
                &projection,
                "col:tax",
                &TreeCalcTableFormulaRuntimeRegion::Data {
                    row_id: TreeCalcTableRowId("row:east".to_string()),
                    row_offset: 1,
                },
            )),
            table_context_identity: Some(projection.table_context_identity.clone()),
        };
        let reference_system_provider = TreeCalcReferenceSystemProvider::sparse_only()
            .with_sparse_reference_values(
                runtime_binding.sparse_reference_values.reference.clone(),
                runtime_binding.sparse_reference_values.resolved_values(),
            );

        let result = RuntimeEnvironment::new()
            .with_structure_context_version(StructureContextVersion(
                "treecalc-structure:v1".to_string(),
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
            .with_host_formula_context(host_formula_context)
            .with_cell_values(runtime_binding.scalar_cell_values)
            .with_function_registry(&function_registry)
            .execute(
                RuntimeFormulaRequest::new(
                    FormulaSourceRecord::new("runtime:w056-table-host-udf", 1, source),
                    TypedContextQueryBundle::default()
                        .with_host_function_provider(Some(&provider))
                        .with_reference_system_provider(Some(
                            &reference_system_provider
                                as &dyn oxfunc_core::resolver::ReferenceSystemProvider,
                        )),
                )
                .with_backend(EvaluationBackend::OxFuncBacked)
                .with_trace_mode(EvaluationTraceMode::PreparedCalls),
            )
            .expect("registry-backed host UDF should reach the OxFml host callback boundary");

        assert_eq!(
            result.evaluation.oxfunc_value,
            CalcValue::error(WorksheetErrorCode::Value)
        );
        assert_eq!(
            result
                .prepared_formula_identity
                .registry_snapshot_identity
                .as_deref(),
            Some(registry_snapshot_identity.as_str())
        );
        assert_eq!(
            result.structured_reference_bind_records[0].source_token_text,
            "[@Amount]"
        );
        let invocation = provider
            .invocations
            .borrow()
            .last()
            .cloned()
            .expect("host UDF provider should be invoked");
        assert_eq!(invocation.function_name, "TABLE_IDENTITY_PROBE");
        assert_eq!(
            invocation.args,
            vec![CalcValue::error(WorksheetErrorCode::Value)]
        );
        let summary = result
            .semantic_plan
            .availability_summaries
            .iter()
            .find(|summary| {
                summary
                    .surface_name
                    .eq_ignore_ascii_case("TABLE_IDENTITY_PROBE")
            })
            .expect("UDF should enter semantic availability through registry snapshot");
        assert_eq!(
            summary.registration_source_kind,
            Some(RegistrationSourceKind::UserDefined)
        );
        assert_eq!(
            summary.runtime_boundary_kind.as_deref(),
            Some("vba_host_callback")
        );
        assert_eq!(
            summary.runtime_capability_state,
            Some(LibraryAvailabilityState::CatalogKnown)
        );
    }

    #[test]
    fn node_table_path_uses_shared_packet_reader_dependency_and_runtime_surfaces() {
        let snapshot = runtime_treecalc_table_snapshot();
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let source = "=SUM(SalesTable[Amount])";
        let bind_records = table_formula_bind_records(&projection, source, None);

        assert_eq!(bind_records.len(), 1);
        let record = &bind_records[0];
        assert_eq!(record.source_token_text, "SalesTable[Amount]");
        assert_eq!(
            record.effective_table_id.as_deref(),
            Some(projection.table_id.as_str())
        );
        assert_eq!(record.selected_column_ids, vec!["col:amount"]);
        assert_eq!(record.selected_sections, vec![StructuredSectionKind::Data]);
        assert!(record.diagnostics.is_empty());

        let lowering_request = StructuredTableDependencyLoweringRequest::from_oxfml_bind_record(
            TreeNodeId(200),
            projection.context_packet.clone(),
            record,
        )
        .expect("OxCalc table dependency lowering should consume the public bind record");
        let lowering = lower_structured_table_dependencies(&lowering_request);
        assert!(lowering.blocked_facts().is_empty());
        assert_eq!(
            lowering.table_context_identity.as_str(),
            projection.context_packet.table_context_identity.as_str()
        );
        assert!(
            lowering.descriptors.iter().all(|descriptor| {
                descriptor.source_reference_handle.as_deref()
                    == Some(record.bind_record_handle.as_str())
                    && descriptor.target_node_id.is_none()
                    && descriptor.tree_reference_collection.is_none()
            }),
            "table dependency descriptors must stay on generic structured-table handles"
        );
        for kind in [
            DependencyDescriptorKind::StructuredTableRowMembership,
            DependencyDescriptorKind::StructuredTableRowOrder,
            DependencyDescriptorKind::StructuredTableColumnIdentity,
            DependencyDescriptorKind::StructuredTableDataRegion,
        ] {
            assert!(
                lowering
                    .descriptors
                    .iter()
                    .any(|descriptor| descriptor.kind == kind),
                "missing dependency descriptor {kind:?}"
            );
        }

        let reader = TreeCalcTableSparseReader::from_oxfml_bind_record(
            &snapshot,
            &projection,
            record,
            None,
            amount_column_values(),
        )
        .expect("shared table sparse reader should consume the public bind record");
        assert_eq!(
            reader.reference(),
            &ReferenceLike::new(ReferenceKind::Area, "C4:C6")
        );
        assert_eq!(reader.defined_cardinality(), 2);
        let runtime_binding = reader.runtime_binding();
        assert_eq!(runtime_binding.reference, reader.reference().clone());
        assert_eq!(
            runtime_binding.sparse_reference_values.declared_rows,
            reader.declared_extent().row_count as usize
        );
        assert_eq!(
            runtime_binding.sparse_reference_values.defined_cells.len(),
            reader.defined_cardinality()
        );
        assert!(
            runtime_binding
                .sparse_reference_values
                .reader_identity
                .as_deref()
                .is_some_and(|identity| identity.contains(&reader.reader_identity().reader_id))
        );

        let reference_system_provider = TreeCalcReferenceSystemProvider::sparse_only()
            .with_sparse_reference_values(
                runtime_binding.sparse_reference_values.reference.clone(),
                runtime_binding.sparse_reference_values.resolved_values(),
            );
        let result = RuntimeEnvironment::new()
            .with_primary_locus(table_primary_locus(&projection.table_descriptor))
            .with_table_context(vec![projection.table_descriptor.clone()], None, None)
            .execute(
                RuntimeFormulaRequest::new(
                    FormulaSourceRecord::new("runtime:w056-node-table-anti-shim", 1, source),
                    TypedContextQueryBundle::default().with_reference_system_provider(Some(
                        &reference_system_provider
                            as &dyn oxfunc_core::resolver::ReferenceSystemProvider,
                    )),
                )
                .with_backend(EvaluationBackend::OxFuncBacked),
            )
            .expect("table reference should execute through OxFml/OxFunc sparse bindings");

        assert_eq!(result.evaluation.oxfunc_value, CalcValue::number(3.0));
        assert_eq!(result.structured_reference_bind_records.len(), 1);
        assert_eq!(
            result.structured_reference_bind_records[0].source_token_text,
            record.source_token_text
        );
        let sum_admission = treecalc_structured_table_function_admission("SUM").unwrap();
        assert_eq!(
            sum_admission.carrier_mode,
            TreeCalcStructuredTableFunctionCarrierMode::SparseReferenceLike
        );
        assert!(!sum_admission.treecalc_selector_visible_to_oxfunc);
        assert!(!sum_admission.eager_materialization_closure_allowed);
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
    fn table_replay_evidence_inventory_converges_oracle_and_value_wire_lanes() {
        let mut lane_ids = BTreeSet::new();
        let mut view_families = BTreeSet::new();
        for lane in TREECALC_TABLE_REPLAY_EVIDENCE_LANES {
            assert!(
                lane_ids.insert(lane.lane_id),
                "duplicate lane {}",
                lane.lane_id
            );
            assert!(
                !lane.retained_artifacts_or_beads.is_empty(),
                "{} must cite retained artifacts, beads, or exact blockers",
                lane.lane_id
            );
            assert!(
                !lane.view_families.is_empty(),
                "{} must name replay/comparison view families",
                lane.lane_id
            );
            assert!(
                !lane.producer_private_string_parsing_allowed,
                "{} would allow private producer formula parsing",
                lane.lane_id
            );
            assert!(
                !lane.excel_internal_inference_allowed,
                "{} would allow Excel-internal dependency inference",
                lane.lane_id
            );
            view_families.extend(lane.view_families.iter().copied());
        }

        for required_lane in [
            "dnatreecalc_retained_table_producer_views",
            "oxcalc_runtime_packet_and_identity_facts",
            "oxxlplay_excel_listobject_oracle_views",
            "oxreplay_third_pass_table_intake",
            "oxreplay_matched_treecalc_excel_table_views",
            "comparison_value_helper_replacement",
            "excel_dependency_dirty_event_order_internals",
            "legacy_outcome_class_projection_gap",
            "namespace_anchor_workspace_cross_producer_pairing",
        ] {
            assert!(
                lane_ids.contains(required_lane),
                "missing lane {required_lane}"
            );
        }

        for required_family in [
            "table_slice",
            "per_node_value",
            "comparison_value",
            "effective_display_text",
            "execution_outcome",
            "table_update_oracle",
            "dependency_evidence",
            "invalidation_evidence",
            "retained_artifact_ref",
            "source_preservation",
            "prepared_identity",
            "dynamic_table_rebind",
            "registry_snapshot_identity",
        ] {
            assert!(
                view_families.contains(required_family),
                "missing replay/view family {required_family}"
            );
        }

        let matched =
            treecalc_table_replay_evidence_lane("oxreplay_matched_treecalc_excel_table_views")
                .expect("matched TreeCalc/Excel lane should exist");
        assert_eq!(
            matched.status,
            TreeCalcTableReplayEvidenceStatus::RetainedEvidenceAvailable
        );
        assert_eq!(matched.value_wire_field, Some("comparison_value"));
        assert!(
            matched
                .retained_artifacts_or_beads
                .iter()
                .any(|artifact| artifact.contains("host_rollout_matched_table_001"))
        );

        let value_wire_blocker =
            treecalc_table_replay_evidence_lane("comparison_value_helper_replacement")
                .expect("value-wire blocker lane should exist");
        assert_eq!(
            value_wire_blocker.status,
            TreeCalcTableReplayEvidenceStatus::BlockedUpstream
        );
        assert_eq!(value_wire_blocker.blocker_id, Some("BLK-REPLAY-003"));
        assert_eq!(
            value_wire_blocker.value_wire_field,
            Some("comparison_value")
        );

        let excel_internals =
            treecalc_table_replay_evidence_lane("excel_dependency_dirty_event_order_internals")
                .expect("Excel internals typed-unavailable lane should exist");
        assert_eq!(
            excel_internals.status,
            TreeCalcTableReplayEvidenceStatus::TypedUnavailable
        );
        assert!(
            excel_internals
                .view_families
                .contains(&"dependency_evidence")
        );
        assert!(
            excel_internals
                .closure_role
                .contains("not inferred from Excel")
        );

        let namespace_pairing = treecalc_table_replay_evidence_lane(
            "namespace_anchor_workspace_cross_producer_pairing",
        )
        .expect("namespace/anchor/workspace projection gap should exist");
        assert_eq!(
            namespace_pairing.status,
            TreeCalcTableReplayEvidenceStatus::TypedProjectionGap
        );
        assert!(namespace_pairing.blocker_id.is_none());
    }

    #[test]
    fn table_cross_repo_rollout_inventory_records_counterparts_and_seam_rules() {
        let mut lane_ids = BTreeSet::new();
        let mut repos = BTreeSet::new();
        for lane in TREECALC_TABLE_CROSS_REPO_ROLLOUT_LANES {
            assert!(
                lane_ids.insert(lane.lane_id),
                "duplicate lane {}",
                lane.lane_id
            );
            repos.insert(lane.repo);
            assert!(
                !lane.counterpart_anchors.is_empty(),
                "{} must cite counterpart beads, handoffs, or non-impact anchor",
                lane.lane_id
            );
            assert!(
                !lane.responsibility.is_empty() && !lane.evidence_obligation.is_empty(),
                "{} must state owner responsibility and evidence obligation",
                lane.lane_id
            );
            assert!(
                !lane.producer_private_string_parsing_allowed,
                "{} would allow producer-private parsing",
                lane.lane_id
            );
            assert!(
                !lane.semantic_mirror_allowed,
                "{} would allow a semantic mirror or bridge",
                lane.lane_id
            );
            if !matches!(
                lane.status,
                TreeCalcTableRolloutLaneStatus::ClosedEvidence
                    | TreeCalcTableRolloutLaneStatus::ExplicitNonImpact
            ) {
                assert!(
                    !lane.blocks_w056_table_semantic_claim,
                    "{} should not become a hidden semantic blocker without an exact bead",
                    lane.lane_id
                );
                assert!(
                    !lane.residual_action.is_empty(),
                    "{} must record its residual action",
                    lane.lane_id
                );
            }
        }

        for required_repo in [
            "OxFml",
            "OxFunc",
            "OxCalc",
            "DnaTreeCalc",
            "OxXlPlay",
            "OxReplay",
            "DnaOneCalc",
            "OxVba",
            "OxIde/DnaOxIde/DnaVisiCalc/Foundation",
        ] {
            assert!(
                repos.contains(required_repo),
                "missing repo lane {required_repo}"
            );
        }

        let order = |lane_id| {
            treecalc_table_cross_repo_rollout_lane(lane_id)
                .expect("lane should exist")
                .promotion_order
        };
        assert!(
            order("oxfml_generic_table_packets_and_name_call_lanes")
                < order("oxcalc_table_custody_runtime_and_hardening")
        );
        assert!(
            order("oxfunc_opaque_reference_and_registry_lanes")
                < order("oxcalc_table_custody_runtime_and_hardening")
        );
        assert!(
            order("oxcalc_table_custody_runtime_and_hardening")
                < order("dnatreecalc_table_product_corpus_and_parent_reconciliation")
        );
        assert!(
            order("dnatreecalc_table_product_corpus_and_parent_reconciliation")
                < order("oxreplay_retained_comparison_and_value_wire")
        );
        assert!(
            order("oxxlplay_excel_listobject_observation")
                < order("oxreplay_retained_comparison_and_value_wire")
        );

        let dnatreecalc = treecalc_table_cross_repo_rollout_lane(
            "dnatreecalc_table_product_corpus_and_parent_reconciliation",
        )
        .expect("DnaTreeCalc rollout lane should exist");
        assert_eq!(
            dnatreecalc.status,
            TreeCalcTableRolloutLaneStatus::OpenParentReconciliation
        );
        assert!(!dnatreecalc.blocks_w056_table_semantic_claim);
        assert!(dnatreecalc.counterpart_anchors.contains(&"dtc-z0i.5.6.1"));
        assert!(dnatreecalc.counterpart_anchors.contains(&"dtc-z0i.5"));
        assert!(dnatreecalc.residual_action.contains("parent beads"));

        let oxfunc =
            treecalc_table_cross_repo_rollout_lane("oxfunc_opaque_reference_and_registry_lanes")
                .expect("OxFunc rollout lane should exist");
        assert_eq!(
            oxfunc.status,
            TreeCalcTableRolloutLaneStatus::OpenAdjacentNonBlocking
        );
        assert!(oxfunc.counterpart_anchors.contains(&"oxf-ypq2.12"));
        assert!(!oxfunc.blocks_w056_table_semantic_claim);

        let oxreplay =
            treecalc_table_cross_repo_rollout_lane("oxreplay_retained_comparison_and_value_wire")
                .expect("OxReplay rollout lane should exist");
        assert_eq!(
            oxreplay.status,
            TreeCalcTableRolloutLaneStatus::OpenAdjacentNonBlocking
        );
        assert!(oxreplay.counterpart_anchors.contains(&"BLK-REPLAY-003"));
        assert!(!oxreplay.blocks_w056_table_semantic_claim);

        let onecalc = treecalc_table_cross_repo_rollout_lane(
            "dnaonecalc_no_host_guardrail_and_future_udf_consumption",
        )
        .expect("DnaOneCalc rollout lane should exist");
        assert_eq!(
            onecalc.status,
            TreeCalcTableRolloutLaneStatus::ExplicitNonImpact
        );
        assert!(onecalc.residual_action.contains("ordinary formulas"));
    }

    #[test]
    fn table_final_audit_marks_node_table_complete_without_parent_w056_overclaim() {
        let mut item_ids = BTreeSet::new();
        let mut statuses = BTreeSet::new();
        let mut evidence_anchors = BTreeSet::new();

        for item in TREECALC_TABLE_FINAL_AUDIT_ITEMS {
            assert!(
                item_ids.insert(item.item_id),
                "duplicate audit item {}",
                item.item_id
            );
            statuses.insert(item.status);
            evidence_anchors.extend(item.evidence_anchors.iter().copied());

            assert!(
                !item.product_scope.is_empty()
                    && !item.still_open.is_empty()
                    && !item.parent_w056_implication.is_empty(),
                "{} must have product scope, residual status, and parent-W056 implication",
                item.item_id
            );
            assert!(
                !item.evidence_anchors.is_empty(),
                "{} must cite executable or retained evidence",
                item.item_id
            );
            assert!(
                !item.blocks_node_table_completion,
                "{} should not remain a node-table blocker after the hardening spine",
                item.item_id
            );
            assert!(
                !item.dense_or_eager_materialization_allowed,
                "{} would let dense/eager materialization close table behavior",
                item.item_id
            );
            assert!(
                !item.private_formula_parsing_allowed,
                "{} would allow host-side formula parsing",
                item.item_id
            );
            assert!(
                !item.oxfml_or_oxfunc_treecalc_branch_allowed,
                "{} would allow TreeCalc-specific OxFml/OxFunc branches",
                item.item_id
            );

            if item.status == TreeCalcTableFinalAuditStatus::ParentW056NonTableRemaining {
                assert!(item.blocks_parent_w056_completion);
                assert!(item.product_scope.contains("non-table W056 reference"));
            } else {
                assert!(
                    !item.blocks_parent_w056_completion,
                    "{} should not block parent W056 completion unless it is the explicit non-table spine",
                    item.item_id
                );
            }
        }

        for required_status in [
            TreeCalcTableFinalAuditStatus::SupportedByExecutableEvidence,
            TreeCalcTableFinalAuditStatus::SupportedWithTypedProjectionGap,
            TreeCalcTableFinalAuditStatus::OpenParentReconciliation,
            TreeCalcTableFinalAuditStatus::ExplicitNonImpact,
            TreeCalcTableFinalAuditStatus::FutureExtensionTracked,
            TreeCalcTableFinalAuditStatus::ParentW056NonTableRemaining,
        ] {
            assert!(
                statuses.contains(&required_status),
                "missing audit status {required_status:?}"
            );
        }

        for required_item in [
            "structured_reference_packets_and_projection",
            "sparse_reference_readers_and_value_transport",
            "dependency_invalidation_lifecycle_and_identity",
            "table_formulas_functions_and_udf_boundary",
            "dynamic_table_rebind_namespace_anchor_workspace",
            "oracle_replay_and_value_wire",
            "dnatreecalc_product_activation_parent_reconciliation",
            "dnaonecalc_no_host_formula_guardrail",
            "vba_xll_udf_descriptor_future_extension",
            "parent_w056_non_table_reference_spine",
        ] {
            assert!(
                item_ids.contains(required_item),
                "missing final audit item {required_item}"
            );
        }

        for required_anchor in [
            "calc-4vs8.57",
            "calc-4vs8.58",
            "calc-4vs8.59",
            "calc-4vs8.60",
            "calc-4vs8.61",
            "oxreplay-p1w.3",
            "oxxlplay-4nd.5",
            "dtc-z0i.7.1",
            "dno-rl7u",
        ] {
            assert!(
                evidence_anchors.contains(required_anchor),
                "missing final audit evidence anchor {required_anchor}"
            );
        }

        let packets =
            treecalc_table_final_audit_item("structured_reference_packets_and_projection")
                .expect("structured-reference packet item should exist");
        assert!(packets.product_scope.contains("path[Col]"));
        assert!(packets.product_scope.contains("path[@Col]"));
        assert!(packets.product_scope.contains("#Totals"));

        let dynamic =
            treecalc_table_final_audit_item("dynamic_table_rebind_namespace_anchor_workspace")
                .expect("dynamic table audit item should exist");
        assert_eq!(
            dynamic.status,
            TreeCalcTableFinalAuditStatus::SupportedWithTypedProjectionGap
        );
        assert!(dynamic.still_open.contains("cross-producer"));
        assert!(!dynamic.blocks_node_table_completion);

        let replay = treecalc_table_final_audit_item("oracle_replay_and_value_wire")
            .expect("replay audit item should exist");
        assert_eq!(
            replay.status,
            TreeCalcTableFinalAuditStatus::SupportedWithTypedProjectionGap
        );
        assert!(replay.still_open.contains("BLK-REPLAY-003"));
        assert!(!replay.blocks_node_table_completion);

        let parent = treecalc_table_final_audit_item("parent_w056_non_table_reference_spine")
            .expect("parent W056 audit item should exist");
        assert_eq!(
            parent.status,
            TreeCalcTableFinalAuditStatus::ParentW056NonTableRemaining
        );
        assert!(parent.blocks_parent_w056_completion);
        assert!(!parent.blocks_node_table_completion);
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
        let bind_records =
            table_formula_bind_records(&projection, "=[@Amount]+2", caller_region.clone());
        let reader = TreeCalcTableSparseReader::from_oxfml_bind_record(
            &snapshot,
            &projection,
            &bind_records[0],
            caller_region.as_ref(),
            [TreeCalcTableSparseValue::data(
                "row:east",
                "col:amount",
                CalcValue::number(4.0),
            )],
        )
        .expect("current-row table reader should build");
        let runtime_binding = reader.runtime_binding();
        let table_descriptor = projection.table_descriptor.clone();

        assert_eq!(
            runtime_binding.scalar_cell_values,
            BTreeMap::from([("C5".to_string(), CalcValue::number(4.0))])
        );

        let reference_system_provider = TreeCalcReferenceSystemProvider::sparse_only()
            .with_sparse_reference_values(
                runtime_binding.sparse_reference_values.reference.clone(),
                runtime_binding.sparse_reference_values.resolved_values(),
            );
        let result = RuntimeEnvironment::new()
            .with_primary_locus(table_primary_locus(&table_descriptor))
            .with_table_context(vec![table_descriptor], enclosing, caller_region)
            .with_cell_values(runtime_binding.scalar_cell_values)
            .execute(
                RuntimeFormulaRequest::new(
                    FormulaSourceRecord::new(
                        "runtime:w056-tree-table-current-row",
                        1,
                        "=[@Amount]+2",
                    ),
                    TypedContextQueryBundle::default().with_reference_system_provider(Some(
                        &reference_system_provider
                            as &dyn oxfunc_core::resolver::ReferenceSystemProvider,
                    )),
                )
                .with_backend(EvaluationBackend::OxFuncBacked),
            )
            .expect("current-row scalar structured reference should execute");

        assert_eq!(result.evaluation.oxfunc_value, CalcValue::number(6.0));
    }

    #[test]
    fn table_sparse_reader_projects_headers_totals_and_all_regions() {
        let snapshot = treecalc_table_snapshot();
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let headers = StructuredTableReferenceIntake::explicit_table(
            "structured-ref:headers-tax",
            "tree-table:sales",
        )
        .with_selected_columns(["col:tax".to_string()])
        .with_selected_regions([StructuredTableRegionSelection::Headers]);
        let headers_reader = TreeCalcTableSparseReader::from_reference_intake(
            &snapshot,
            &projection,
            &headers,
            None,
            Vec::<TreeCalcTableSparseValue>::new(),
            "SalesTable[[#Headers],[Tax]]",
            "structured-ref:headers-tax",
            None,
        )
        .expect("headers-only reader should build");
        assert_eq!(headers_reader.declared_extent().row_count, 1);
        assert_eq!(headers_reader.declared_extent().column_count, 1);
        assert_eq!(
            headers_reader.reference(),
            &ReferenceLike::new(ReferenceKind::A1, "treecalc-virtual-sheet:tables!D3")
        );
        assert_eq!(
            headers_reader.read_at(SparseCellCoord::new(1, 1)),
            SparseCellRead::Defined(CalcValue::text(ExcelText::from_interop_assignment("Tax")))
        );

        let totals = StructuredTableReferenceIntake::explicit_table(
            "structured-ref:totals-amount",
            "tree-table:sales",
        )
        .with_selected_columns(["col:amount".to_string()])
        .with_selected_regions([StructuredTableRegionSelection::Totals]);
        let totals_reader = TreeCalcTableSparseReader::from_reference_intake(
            &snapshot,
            &projection,
            &totals,
            None,
            [TreeCalcTableSparseValue::totals(
                "col:amount",
                CalcValue::number(7.0),
            )],
            "SalesTable[[#Totals],[Amount]]",
            "structured-ref:totals-amount",
            None,
        )
        .expect("totals-only reader should build");
        assert_eq!(totals_reader.declared_extent().row_count, 1);
        assert_eq!(totals_reader.declared_extent().column_count, 1);
        assert_eq!(
            totals_reader.reference(),
            &ReferenceLike::new(ReferenceKind::A1, "treecalc-virtual-sheet:tables!C7")
        );
        assert_eq!(
            totals_reader.read_at(SparseCellCoord::new(1, 1)),
            SparseCellRead::Defined(CalcValue::number(7.0))
        );

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
                TreeCalcTableSparseValue::data("row:west", "col:amount", CalcValue::number(3.0)),
                TreeCalcTableSparseValue::data("row:east", "col:amount", CalcValue::number(4.0)),
                TreeCalcTableSparseValue::data("row:north", "col:tax", CalcValue::number(1.5)),
                TreeCalcTableSparseValue::totals("col:amount", CalcValue::number(7.0)),
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
            SparseCellRead::Defined(CalcValue::text(ExcelText::from_interop_assignment(
                "Amount"
            )))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(1, 2)),
            SparseCellRead::Defined(CalcValue::text(ExcelText::from_interop_assignment("Tax")))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(5, 1)),
            SparseCellRead::Defined(CalcValue::number(7.0))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(5, 2)),
            SparseCellRead::Blank
        );

        let all_columns = StructuredTableReferenceIntake::explicit_table(
            "structured-ref:all-columns",
            "tree-table:sales",
        )
        .with_selected_regions([StructuredTableRegionSelection::All]);
        let all_columns_reader = TreeCalcTableSparseReader::from_reference_intake(
            &snapshot,
            &projection,
            &all_columns,
            None,
            [
                TreeCalcTableSparseValue::data(
                    "row:west",
                    "col:region",
                    CalcValue::text(ExcelText::from_interop_assignment("West")),
                ),
                TreeCalcTableSparseValue::totals("col:amount", CalcValue::number(7.0)),
            ],
            "SalesTable[#All]",
            "structured-ref:all-columns",
            None,
        )
        .expect("all-column #All reader should build");
        assert_eq!(all_columns_reader.declared_extent().row_count, 5);
        assert_eq!(all_columns_reader.declared_extent().column_count, 3);
        assert_eq!(
            all_columns_reader.reference(),
            &ReferenceLike::new(ReferenceKind::Area, "treecalc-virtual-sheet:tables!B3:D7")
        );
        assert_eq!(
            all_columns_reader.read_at(SparseCellCoord::new(1, 1)),
            SparseCellRead::Defined(CalcValue::text(ExcelText::from_interop_assignment(
                "Region"
            )))
        );
        assert_eq!(
            all_columns_reader.read_at(SparseCellCoord::new(5, 2)),
            SparseCellRead::Defined(CalcValue::number(7.0))
        );
    }

    #[test]
    fn table_sparse_reader_reports_typed_selection_exclusions() {
        let snapshot = treecalc_table_snapshot();
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let missing_column = StructuredTableReferenceIntake::explicit_table(
            "structured-ref:missing",
            "tree-table:sales",
        )
        .with_selected_columns(["col:missing".to_string()]);
        assert_eq!(
            TreeCalcTableSparseReader::from_reference_intake(
                &snapshot,
                &projection,
                &missing_column,
                None,
                Vec::<TreeCalcTableSparseValue>::new(),
                "SalesTable[Missing]",
                "structured-ref:missing",
                None,
            )
            .unwrap_err(),
            TreeCalcTableSparseReaderError::MissingSelectedColumn {
                column_id: "col:missing".to_string()
            }
        );

        let non_contiguous = StructuredTableReferenceIntake::explicit_table(
            "structured-ref:non-contiguous",
            "tree-table:sales",
        )
        .with_selected_columns(["col:region".to_string(), "col:tax".to_string()]);
        assert_eq!(
            TreeCalcTableSparseReader::from_reference_intake(
                &snapshot,
                &projection,
                &non_contiguous,
                None,
                Vec::<TreeCalcTableSparseValue>::new(),
                "SalesTable[[Region],[Tax]]",
                "structured-ref:non-contiguous",
                None,
            )
            .unwrap_err(),
            TreeCalcTableSparseReaderError::NonContiguousColumnSelection {
                column_ids: vec!["col:region".to_string(), "col:tax".to_string()]
            }
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
                CalcValue::number(1.0),
                CalcValue::number(2.0),
                CalcValue::number(3.0)
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
    fn table_formula_prepared_identity_facts_track_context_and_mutations() {
        let snapshot = runtime_treecalc_table_snapshot();
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let base_context = TreeCalcTableFormulaRuntimeContext::default();
        let base_request = tax_formula_runtime_request(
            "=[@Amount]/10",
            table_formula_amount_values(),
            base_context.clone(),
        );
        let base_report =
            evaluate_treecalc_table_column_formula_rows(&snapshot, &projection, &base_request)
                .expect("current-row table formula should evaluate");
        assert_eq!(
            base_report
                .cell_results
                .iter()
                .map(|cell| cell.value.clone())
                .collect::<Vec<_>>(),
            vec![
                CalcValue::number(1.0),
                CalcValue::number(2.0),
                CalcValue::number(3.0)
            ]
        );

        let base_east = table_formula_cell_for_row(&base_report, "row:east");
        let base_facts = &base_east.prepared_identity_facts;
        assert_eq!(base_facts.dialect_id, "oxcalc.treecalc-v1");
        assert_eq!(
            base_facts.capability_profile_id,
            "host-capabilities:treecalc-v1"
        );
        assert_eq!(
            base_facts.resolution_rule_version,
            "treecalc-host-resolution:v1"
        );
        assert_eq!(
            base_facts.host_namespace_version.as_deref(),
            Some("treecalc-host-namespace:v1")
        );
        assert_eq!(base_facts.table_namespace_version, "namespace:v1");
        assert_eq!(
            base_facts.table_context_identity,
            projection.table_context_identity
        );
        assert_eq!(
            base_facts.structure_context_version,
            "treecalc-structure:v1"
        );
        assert_eq!(base_facts.caller_context_id, base_east.caller_context_id);
        assert_eq!(
            base_facts.host_registry_snapshot_identity,
            base_context.registry_snapshot_identity
        );
        assert_eq!(
            base_facts.function_registry_snapshot_identity,
            base_east.registry_snapshot_identity
        );
        assert!(
            base_facts
                .identity_fragment()
                .starts_with("treecalc.table_formula_prepared_identity_facts.v1")
        );

        for context in [
            {
                let mut context = base_context.clone();
                context.host_namespace_version = Some("treecalc-host-namespace:v2".to_string());
                context
            },
            {
                let mut context = base_context.clone();
                context.structure_context_version = "treecalc-structure:v2".to_string();
                context
            },
            {
                let mut context = base_context.clone();
                context.resolution_rule_version = "treecalc-host-resolution:v2".to_string();
                context
            },
            {
                let mut context = base_context.clone();
                context.capability_profile_id = "host-capabilities:treecalc-v2".to_string();
                context
            },
            {
                let mut context = base_context.clone();
                let function_registry = registry_with_test_udf();
                context.registry_snapshot_identity =
                    Some(function_registry.snapshot_identity().stable_key());
                context.function_registry = function_registry;
                context
            },
            {
                let mut context = base_context.clone();
                let mut overlay = CapabilityOverlay::new();
                overlay.set_availability(
                    "xlfSum",
                    FunctionAvailability::Unavailable {
                        reason: "table formula test denies SUM".to_string(),
                    },
                );
                context.capability_overlay = Some(overlay);
                context
            },
        ] {
            let report = evaluate_treecalc_table_column_formula_rows(
                &snapshot,
                &projection,
                &tax_formula_runtime_request(
                    "=[@Amount]/10",
                    table_formula_amount_values(),
                    context.clone(),
                ),
            )
            .expect("identity context variant should evaluate");
            let east = table_formula_cell_for_row(&report, "row:east");
            assert_ne!(
                base_east.prepared_formula_key, east.prepared_formula_key,
                "prepared identity must include changed context {context:?}"
            );
            assert_eq!(
                east.prepared_identity_facts.host_registry_snapshot_identity,
                context.registry_snapshot_identity
            );
            assert_eq!(
                east.prepared_identity_facts
                    .function_registry_snapshot_identity,
                east.registry_snapshot_identity
            );
        }

        let mut renamed_snapshot = snapshot.clone();
        renamed_snapshot.table_name = "SalesRenamed".to_string();
        renamed_snapshot.table_namespace_version = "namespace:v2".to_string();
        let renamed_projection = project_treecalc_table_node_snapshot(&renamed_snapshot).unwrap();
        let renamed_report = evaluate_treecalc_table_column_formula_rows(
            &renamed_snapshot,
            &renamed_projection,
            &tax_formula_runtime_request(
                "=[@Amount]/10",
                table_formula_amount_values(),
                base_context.clone(),
            ),
        )
        .expect("same formula should evaluate after table namespace rename");
        let renamed_east = table_formula_cell_for_row(&renamed_report, "row:east");
        assert_eq!(
            renamed_east.prepared_identity_facts.table_namespace_version,
            "namespace:v2"
        );
        assert_ne!(
            base_east.prepared_formula_key,
            renamed_east.prepared_formula_key
        );
        assert_eq!(
            base_east.dispatch_skeleton_key,
            renamed_east.dispatch_skeleton_key
        );

        let mut row_reorder = snapshot.clone();
        row_reorder.rows = vec![
            TreeCalcTableRowId("row:east".to_string()),
            TreeCalcTableRowId("row:west".to_string()),
            TreeCalcTableRowId("row:north".to_string()),
        ];
        row_reorder.row_order_version = "row-order:v2".to_string();
        let row_reorder_projection = project_treecalc_table_node_snapshot(&row_reorder).unwrap();
        let row_reorder_report = evaluate_treecalc_table_column_formula_rows(
            &row_reorder,
            &row_reorder_projection,
            &tax_formula_runtime_request(
                "=[@Amount]/10",
                table_formula_amount_values(),
                base_context.clone(),
            ),
        )
        .expect("same formula should evaluate after row reorder");
        let reordered_east = table_formula_cell_for_row(&row_reorder_report, "row:east");
        assert_eq!(reordered_east.row_offset, Some(0));
        assert_ne!(
            base_east.caller_context_id,
            reordered_east.caller_context_id
        );
        assert_ne!(
            base_east.prepared_formula_key,
            reordered_east.prepared_formula_key
        );
        assert_eq!(
            base_east.dispatch_skeleton_key,
            reordered_east.dispatch_skeleton_key
        );

        let mut row_insert = snapshot.clone();
        row_insert
            .rows
            .insert(0, TreeCalcTableRowId("row:south".to_string()));
        row_insert.row_membership_version = "row-membership:v2".to_string();
        row_insert.row_order_version = "row-order:v3".to_string();
        let row_insert_projection = project_treecalc_table_node_snapshot(&row_insert).unwrap();
        let mut inserted_values = table_formula_amount_values();
        inserted_values.push(TreeCalcTableSparseValue::data(
            "row:south",
            "col:amount",
            CalcValue::number(40.0),
        ));
        let row_insert_report = evaluate_treecalc_table_column_formula_rows(
            &row_insert,
            &row_insert_projection,
            &tax_formula_runtime_request("=[@Amount]/10", inserted_values, base_context.clone()),
        )
        .expect("same formula should evaluate after row insert");
        let inserted_east = table_formula_cell_for_row(&row_insert_report, "row:east");
        assert_eq!(inserted_east.row_offset, Some(2));
        assert_ne!(base_east.caller_context_id, inserted_east.caller_context_id);
        assert_ne!(
            base_east.prepared_formula_key,
            inserted_east.prepared_formula_key
        );

        let mut row_delete = snapshot;
        row_delete.rows.retain(|row| row.0 != "row:west");
        row_delete.row_membership_version = "row-membership:v3".to_string();
        row_delete.row_order_version = "row-order:v4".to_string();
        let row_delete_projection = project_treecalc_table_node_snapshot(&row_delete).unwrap();
        let row_delete_report = evaluate_treecalc_table_column_formula_rows(
            &row_delete,
            &row_delete_projection,
            &tax_formula_runtime_request(
                "=[@Amount]/10",
                table_formula_amount_values(),
                base_context,
            ),
        )
        .expect("same formula should evaluate after row delete");
        let deleted_east = table_formula_cell_for_row(&row_delete_report, "row:east");
        assert_eq!(deleted_east.row_offset, Some(0));
        assert_ne!(base_east.caller_context_id, deleted_east.caller_context_id);
        assert_ne!(
            base_east.prepared_formula_key,
            deleted_east.prepared_formula_key
        );
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

        assert_eq!(result.value, CalcValue::number(60.0));
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

        let TreeCalcTableFormulaRuntimeError::StructuredReferenceDiagnostics {
            row_id: None,
            diagnostics,
        } = error
        else {
            panic!("expected OxFml structured-reference bind diagnostics, got {error:?}");
        };
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].diagnostic_code,
            "structured_reference_bind_error"
        );
        assert!(diagnostics[0].message.contains("data-region"));
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
                formula_text: "=[@Amount]*0.2".to_string(),
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
            [
                TreeCalcTablePreparedIdentityInput::StructureContextVersion,
                TreeCalcTablePreparedIdentityInput::TableContextIdentity,
            ],
        );

        let mut totals_formula = baseline_snapshot.clone();
        totals_formula.columns[0].totals_metadata = Some(TreeCalcTableFormulaMetadata {
            formula_artifact_id: "formula:totals:amount".to_string(),
            bind_artifact_id: Some("bind:totals:amount:v2".to_string()),
            formula_text_version: "v2".to_string(),
            formula_text: "=SUM([Amount])".to_string(),
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

        let mut table_resize = table_move.clone();
        table_resize
            .rows
            .push(TreeCalcTableRowId("row:south".to_string()));
        table_resize.row_membership_version = "row-membership:resize".to_string();
        table_resize.row_order_version = "row-order:resize".to_string();
        let table_resize_projection = project_treecalc_table_node_snapshot(&table_resize).unwrap();
        assert_update_has(
            TreeCalcTableUpdateScenarioKind::TableResize,
            &baseline,
            Some(&table_resize_projection),
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

        let mut node_rename = baseline_snapshot.clone();
        node_rename.display_path = "Sales Node".to_string();
        node_rename.canonical_path = "Root/Sales Node".to_string();
        node_rename.table_namespace_version = "namespace:node-rename".to_string();
        let node_rename_projection = project_treecalc_table_node_snapshot(&node_rename).unwrap();
        assert_update_has(
            TreeCalcTableUpdateScenarioKind::NodeRename,
            &baseline,
            Some(&node_rename_projection),
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

        let mut node_move = baseline_snapshot.clone();
        node_move.canonical_path = "Root/Archive/SalesTable".to_string();
        node_move.virtual_anchor.sheet_scope_ref = "treecalc-virtual-sheet:archive".to_string();
        let node_move_projection = project_treecalc_table_node_snapshot(&node_move).unwrap();
        assert_update_has(
            TreeCalcTableUpdateScenarioKind::NodeMove,
            &baseline,
            Some(&node_move_projection),
            [
                InvalidationReasonKind::StructuredTableContextChanged,
                InvalidationReasonKind::StructuredTableRegionChanged,
                InvalidationReasonKind::StructuralRebindRequired,
            ],
            [
                TreeCalcTablePreparedIdentityInput::HostNamespaceVersion,
                TreeCalcTablePreparedIdentityInput::TableContextIdentity,
                TreeCalcTablePreparedIdentityInput::ResolutionRuleVersion,
            ],
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

        let node_delete = classify_treecalc_table_update(
            TreeCalcTableUpdateScenarioKind::NodeDelete,
            Some(&baseline),
            None,
            [owner],
            source_handles(),
        );
        assert!(
            node_delete
                .invalidation_reasons
                .contains(&InvalidationReasonKind::DependencyRemoved)
        );
        assert!(
            node_delete
                .prepared_identity_inputs
                .contains(&TreeCalcTablePreparedIdentityInput::HostNamespaceVersion)
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

        let workspace_open = classify_treecalc_table_update(
            TreeCalcTableUpdateScenarioKind::WorkspaceOpen,
            None,
            Some(&baseline),
            [owner],
            source_handles(),
        );
        assert!(
            workspace_open
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::HostSensitive)
        );
        assert!(
            workspace_open
                .invalidation_reasons
                .contains(&InvalidationReasonKind::DependencyAdded)
        );
        assert!(
            workspace_open
                .prepared_identity_inputs
                .contains(&TreeCalcTablePreparedIdentityInput::HostNamespaceVersion)
        );

        let workspace_close = classify_treecalc_table_update(
            TreeCalcTableUpdateScenarioKind::WorkspaceClose,
            Some(&baseline),
            None,
            [owner],
            source_handles(),
        );
        assert!(
            workspace_close
                .invalidation_reasons
                .contains(&InvalidationReasonKind::DependencyRemoved)
        );

        let workspace_alias = classify_treecalc_table_update(
            TreeCalcTableUpdateScenarioKind::WorkspaceAliasMutation,
            Some(&baseline),
            Some(&baseline),
            [owner],
            source_handles(),
        );
        assert!(
            workspace_alias
                .invalidation_reasons
                .contains(&InvalidationReasonKind::StructuralRebindRequired)
        );
        assert!(
            workspace_alias
                .prepared_identity_inputs
                .contains(&TreeCalcTablePreparedIdentityInput::HostNamespaceVersion)
        );

        let registry_mutation = classify_treecalc_table_update(
            TreeCalcTableUpdateScenarioKind::FunctionRegistrySnapshotMutation,
            Some(&baseline),
            Some(&baseline),
            [owner],
            source_handles(),
        );
        assert!(
            registry_mutation
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::CapabilitySensitive)
        );
        assert!(
            registry_mutation
                .prepared_identity_inputs
                .contains(&TreeCalcTablePreparedIdentityInput::RegistrySnapshotIdentity)
        );

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
    fn table_lifecycle_boundary_matrix_states_identity_stability() {
        let owner = TreeNodeId(30);
        let baseline_snapshot = runtime_treecalc_table_snapshot();
        let baseline = project_treecalc_table_node_snapshot(&baseline_snapshot).unwrap();
        let baseline_state = TreeCalcTableLifecycleVersionState::from_snapshot_projection(
            &baseline_snapshot,
            &baseline,
        );
        let amount_reader = |snapshot: &TreeCalcTableNodeSnapshot,
                             projection: &TreeCalcTableNodeProjection,
                             values: Vec<TreeCalcTableSparseValue>| {
            let bind_records =
                table_formula_bind_records(projection, "=SUM(SalesTable[Amount])", None);
            TreeCalcTableSparseReader::from_oxfml_bind_record(
                snapshot,
                projection,
                &bind_records[0],
                None,
                values,
            )
            .expect("amount column reader should build for lifecycle boundary case")
        };
        let baseline_reader = amount_reader(&baseline_snapshot, &baseline, amount_column_values());

        let stable_update_ids = TreeCalcTableUpdateScenarioKind::ALL
            .iter()
            .map(|scenario| scenario.stable_id())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            stable_update_ids.len(),
            TreeCalcTableUpdateScenarioKind::ALL.len(),
            "update scenario stable ids must be unique for retained evidence"
        );

        let mut first_row_insert = baseline_snapshot.clone();
        first_row_insert
            .rows
            .insert(0, TreeCalcTableRowId("row:south".to_string()));
        first_row_insert.row_membership_version = "row-membership:first-insert".to_string();
        first_row_insert.row_order_version = "row-order:first-insert".to_string();
        let first_row_insert_projection =
            project_treecalc_table_node_snapshot(&first_row_insert).unwrap();
        let first_row_reader = amount_reader(&first_row_insert, &first_row_insert_projection, {
            let mut values = table_formula_amount_values();
            values.push(TreeCalcTableSparseValue::data(
                "row:south",
                "col:amount",
                CalcValue::number(40.0),
            ));
            values
        });
        let first_row_impact = classify_treecalc_table_update(
            TreeCalcTableUpdateScenarioKind::RowInsert,
            Some(&baseline),
            Some(&first_row_insert_projection),
            [owner],
            ["structured-ref:amount".to_string()],
        );
        assert!(
            first_row_impact
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::StructuredTableRowMembership)
        );
        assert!(
            first_row_impact
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::StructuredTableRowOrder)
        );
        assert!(
            first_row_impact
                .prepared_identity_inputs
                .contains(&TreeCalcTablePreparedIdentityInput::CallerContextIdentity)
        );
        assert_ne!(
            baseline_reader.reader_identity().reader_id,
            first_row_reader.reader_identity().reader_id
        );
        assert_ne!(
            baseline_reader.reader_identity().source_identity,
            first_row_reader.reader_identity().source_identity
        );
        assert_ne!(
            baseline_reader.reader_identity().snapshot_identity,
            first_row_reader.reader_identity().snapshot_identity
        );
        assert_eq!(first_row_reader.declared_extent().row_count, 4);

        let mut last_row_delete = baseline_snapshot.clone();
        last_row_delete.rows.pop();
        last_row_delete.row_membership_version = "row-membership:last-delete".to_string();
        last_row_delete.row_order_version = "row-order:last-delete".to_string();
        let last_row_delete_projection =
            project_treecalc_table_node_snapshot(&last_row_delete).unwrap();
        let last_row_reader = amount_reader(
            &last_row_delete,
            &last_row_delete_projection,
            amount_column_values(),
        );
        let last_row_impact = classify_treecalc_table_update(
            TreeCalcTableUpdateScenarioKind::RowDelete,
            Some(&baseline),
            Some(&last_row_delete_projection),
            [owner],
            ["structured-ref:amount".to_string()],
        );
        assert!(
            last_row_impact
                .invalidation_reasons
                .contains(&InvalidationReasonKind::StructuredTableRowMembershipChanged)
        );
        assert_eq!(last_row_reader.declared_extent().row_count, 2);
        assert_ne!(
            baseline_reader.reader_identity().snapshot_identity,
            last_row_reader.reader_identity().snapshot_identity
        );

        let mut empty_body = baseline_snapshot.clone();
        empty_body.rows.clear();
        empty_body.row_membership_version = "row-membership:empty".to_string();
        empty_body.row_order_version = "row-order:empty".to_string();
        let empty_body_projection = project_treecalc_table_node_snapshot(&empty_body).unwrap();
        let empty_reader = amount_reader(
            &empty_body,
            &empty_body_projection,
            Vec::<TreeCalcTableSparseValue>::new(),
        );
        let empty_body_impact = classify_treecalc_table_update(
            TreeCalcTableUpdateScenarioKind::RowDelete,
            Some(&baseline),
            Some(&empty_body_projection),
            [owner],
            ["structured-ref:amount".to_string()],
        );
        assert_eq!(empty_reader.declared_extent().row_count, 0);
        assert_eq!(empty_reader.defined_cardinality(), 0);
        assert_eq!(
            empty_reader.reference(),
            &ReferenceLike::new(
                ReferenceKind::Structured,
                "empty-structured:sheet:default:Data:col:amount:1"
            )
        );
        assert!(
            empty_body_impact
                .prepared_identity_inputs
                .contains(&TreeCalcTablePreparedIdentityInput::TableContextIdentity)
        );

        let mut virtual_anchor_move = baseline_snapshot.clone();
        virtual_anchor_move.virtual_anchor.start_row = 13;
        let virtual_anchor_projection =
            project_treecalc_table_node_snapshot(&virtual_anchor_move).unwrap();
        let moved_anchor_reader = amount_reader(
            &virtual_anchor_move,
            &virtual_anchor_projection,
            amount_column_values(),
        );
        let anchor_impact = classify_treecalc_table_update(
            TreeCalcTableUpdateScenarioKind::TableMove,
            Some(&baseline),
            Some(&virtual_anchor_projection),
            [owner],
            ["structured-ref:amount".to_string()],
        );
        assert_eq!(
            moved_anchor_reader.reference(),
            &ReferenceLike::new(ReferenceKind::Area, "C14:C16")
        );
        assert!(
            anchor_impact
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::StructuredTableDataRegion)
        );
        assert!(
            anchor_impact
                .prepared_identity_inputs
                .contains(&TreeCalcTablePreparedIdentityInput::TableContextIdentity)
        );

        let mut availability_state = baseline_state.clone();
        availability_state.workspace_availability_version =
            "treecalc-workspace-availability:v2".to_string();
        let availability_report = classify_treecalc_table_lifecycle_callback(
            &TreeCalcTableLifecycleCallbackPacket::new(
                TreeCalcTableLifecycleEventKind::StructuralRebind,
            )
            .with_before(baseline_state.clone())
            .with_after(availability_state)
            .with_owner_nodes([owner])
            .with_source_reference_handles(["structured-ref:amount"]),
        );
        assert!(availability_report.diagnostics.is_empty());
        assert!(
            availability_report
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::HostSensitive)
        );
        assert!(
            availability_report
                .prepared_identity_inputs
                .contains(&TreeCalcTablePreparedIdentityInput::HostNamespaceVersion)
        );
        assert!(
            availability_report
                .prepared_identity_inputs
                .contains(&TreeCalcTablePreparedIdentityInput::ResolutionRuleVersion)
        );

        let base_report = evaluate_treecalc_table_column_formula_rows(
            &baseline_snapshot,
            &baseline,
            &tax_formula_runtime_request(
                "=[@Amount]/10",
                table_formula_amount_values(),
                TreeCalcTableFormulaRuntimeContext::default(),
            ),
        )
        .expect("baseline current-row formula should evaluate");
        let inserted_report = evaluate_treecalc_table_column_formula_rows(
            &first_row_insert,
            &first_row_insert_projection,
            &tax_formula_runtime_request(
                "=[@Amount]/10",
                {
                    let mut values = table_formula_amount_values();
                    values.push(TreeCalcTableSparseValue::data(
                        "row:south",
                        "col:amount",
                        CalcValue::number(40.0),
                    ));
                    values
                },
                TreeCalcTableFormulaRuntimeContext::default(),
            ),
        )
        .expect("first-row insert current-row formula should evaluate");
        let base_east = table_formula_cell_for_row(&base_report, "row:east");
        let inserted_east = table_formula_cell_for_row(&inserted_report, "row:east");
        assert_eq!(base_east.row_offset, Some(1));
        assert_eq!(inserted_east.row_offset, Some(2));
        assert_ne!(base_east.caller_context_id, inserted_east.caller_context_id);
        assert_ne!(
            base_east.prepared_formula_key,
            inserted_east.prepared_formula_key
        );
        assert_eq!(
            base_east.dispatch_skeleton_key,
            inserted_east.dispatch_skeleton_key
        );

        let dynamic_current_row =
            classify_treecalc_dynamic_table_rebind(&TreeCalcDynamicTableRebindRequest {
                target_kind: TreeCalcDynamicTableReferenceTargetKind::CurrentRow,
                cause: TreeCalcDynamicTableRebindCause::TableLifecycle(
                    TreeCalcTableUpdateScenarioKind::RowReorder,
                ),
                caller_context_id: Some("caller:row-east".to_string()),
                ..dynamic_table_rebind_request(
                    TreeCalcDynamicTableReferenceTargetKind::CurrentRow,
                    TreeCalcDynamicTableRebindCause::VolatileReevaluation,
                )
            });
        assert!(
            dynamic_current_row
                .dependency_fact_kinds
                .contains(&StructuredTableDependencyFactKind::CallerRowContext)
        );
        assert!(
            dynamic_current_row
                .invalidation_reasons
                .contains(&InvalidationReasonKind::StructuredTableRowOrderChanged)
        );
        assert!(
            dynamic_current_row
                .prepared_identity_inputs
                .contains(&TreeCalcTablePreparedIdentityInput::CallerContextIdentity)
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
            if expected.invalidation_reasons.is_empty() {
                assert!(
                    report.invalidation_reasons.is_empty(),
                    "{scenario:?}: expected stable lifecycle with no invalidation reasons, got {:?}",
                    report.invalidation_reasons
                );
            } else {
                assert!(
                    expected
                        .invalidation_reasons
                        .is_subset(&report.invalidation_reasons),
                    "{scenario:?}: expected invalidation reasons {:?}, got {:?}",
                    expected.invalidation_reasons,
                    report.invalidation_reasons
                );
            }
            assert_eq!(
                report.prepared_identity_inputs, expected_inputs,
                "{scenario:?}: prepared identity input set drifted"
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
                formula_text: "=[@Amount]*0.2".to_string(),
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
            formula_text: "=SUM([Amount])".to_string(),
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

        let mut table_resize = baseline_snapshot.clone();
        table_resize
            .rows
            .push(TreeCalcTableRowId("row:south".to_string()));
        table_resize.row_membership_version = "row-membership:resize".to_string();
        table_resize.row_order_version = "row-order:resize".to_string();
        exercise(
            TreeCalcTableUpdateScenarioKind::TableResize,
            Some(table_resize),
        );

        let mut node_rename = baseline_snapshot.clone();
        node_rename.display_path = "Sales Node".to_string();
        node_rename.canonical_path = "Root/Sales Node".to_string();
        node_rename.table_namespace_version = "namespace:node-rename".to_string();
        exercise(
            TreeCalcTableUpdateScenarioKind::NodeRename,
            Some(node_rename),
        );

        let mut node_move = baseline_snapshot.clone();
        node_move.canonical_path = "Root/Archive/SalesTable".to_string();
        node_move.virtual_anchor.sheet_scope_ref = "treecalc-virtual-sheet:archive".to_string();
        exercise(TreeCalcTableUpdateScenarioKind::NodeMove, Some(node_move));

        exercise(TreeCalcTableUpdateScenarioKind::TableDelete, None);
        exercise(TreeCalcTableUpdateScenarioKind::NodeDelete, None);

        exercise(
            TreeCalcTableUpdateScenarioKind::SaveReopen,
            Some(baseline_snapshot.clone()),
        );

        let workspace_open_report = classify_treecalc_table_lifecycle_callback(
            &TreeCalcTableLifecycleCallbackPacket::new(
                TreeCalcTableLifecycleEventKind::WorkspaceOpen,
            )
            .with_after(baseline_state.clone())
            .with_owner_nodes([owner])
            .with_source_reference_handles(["structured-ref:amount"]),
        );
        assert!(workspace_open_report.diagnostics.is_empty());
        assert!(
            workspace_open_report
                .invalidation_reasons
                .contains(&InvalidationReasonKind::DependencyAdded)
        );
        assert!(
            workspace_open_report
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::HostSensitive)
        );

        let workspace_close_report = classify_treecalc_table_lifecycle_callback(
            &TreeCalcTableLifecycleCallbackPacket::new(
                TreeCalcTableLifecycleEventKind::WorkspaceClose,
            )
            .with_before(baseline_state.clone())
            .with_owner_nodes([owner])
            .with_source_reference_handles(["structured-ref:amount"]),
        );
        assert!(workspace_close_report.diagnostics.is_empty());
        assert!(
            workspace_close_report
                .invalidation_reasons
                .contains(&InvalidationReasonKind::DependencyRemoved)
        );

        let mut alias_state = baseline_state.clone();
        alias_state.workspace_alias_version = "treecalc-workspace-alias:v2".to_string();
        let alias_report = classify_treecalc_table_lifecycle_callback(
            &TreeCalcTableLifecycleCallbackPacket::new(
                TreeCalcTableLifecycleEventKind::WorkspaceAliasMutation,
            )
            .with_before(baseline_state.clone())
            .with_after(alias_state)
            .with_owner_nodes([owner])
            .with_source_reference_handles(["structured-ref:amount"]),
        );
        assert!(alias_report.diagnostics.is_empty());
        assert!(
            alias_report
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::HostSensitive)
        );
        assert!(
            alias_report
                .invalidation_reasons
                .contains(&InvalidationReasonKind::StructuralRebindRequired)
        );

        let registry_versions = TreeCalcTableLifecycleContextVersions {
            registry_snapshot_identity: "oxfunc-registry:udf:v2".to_string(),
            ..TreeCalcTableLifecycleContextVersions::default()
        };
        let registry_report =
            classify_treecalc_table_lifecycle_callback(&TreeCalcTableLifecycleCallbackPacket {
                context_versions: registry_versions,
                ..TreeCalcTableLifecycleCallbackPacket::new(
                    TreeCalcTableLifecycleEventKind::FunctionRegistrySnapshotMutation,
                )
                .with_before(baseline_state.clone())
                .with_after(baseline_state.clone())
                .with_owner_nodes([owner])
                .with_source_reference_handles(["structured-ref:amount"])
            });
        assert!(registry_report.diagnostics.is_empty());
        assert!(
            registry_report
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::CapabilitySensitive)
        );
        assert!(
            registry_report
                .prepared_identity_inputs
                .contains(&TreeCalcTablePreparedIdentityInput::RegistrySnapshotIdentity)
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
                TreeCalcTableSparseValue::data("row:west", "col:amount", CalcValue::number(99.0)),
                TreeCalcTableSparseValue::data(
                    "row:west",
                    "col:region",
                    CalcValue::text(ExcelText::from_interop_assignment("West")),
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
            impact.invalidation_reasons == expected_reasons,
            "{scenario:?}: expected exact reasons {expected_reasons:?}, got {:?}",
            impact.invalidation_reasons
        );
        assert!(
            impact.prepared_identity_inputs == expected_identity_inputs,
            "{scenario:?}: expected exact identity inputs {expected_identity_inputs:?}, got {:?}",
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
        let expected_identity_inputs = match scenario {
            TreeCalcTableUpdateScenarioKind::ColumnInsert
            | TreeCalcTableUpdateScenarioKind::ColumnDelete
            | TreeCalcTableUpdateScenarioKind::ColumnReorder => vec![
                TreeCalcTablePreparedIdentityInput::StructureContextVersion,
                TreeCalcTablePreparedIdentityInput::TableContextIdentity,
            ],
            _ => vec![TreeCalcTablePreparedIdentityInput::TableContextIdentity],
        };
        assert_update_has(
            scenario,
            baseline,
            Some(changed),
            [
                InvalidationReasonKind::StructuredTableColumnChanged,
                InvalidationReasonKind::StructuredTableRegionChanged,
                InvalidationReasonKind::StructuredTableContextChanged,
            ],
            expected_identity_inputs,
        );
    }

    fn dynamic_table_rebind_request(
        target_kind: TreeCalcDynamicTableReferenceTargetKind,
        cause: TreeCalcDynamicTableRebindCause,
    ) -> TreeCalcDynamicTableRebindRequest {
        TreeCalcDynamicTableRebindRequest {
            selector_handle: "dynamic-table-selector:1".to_string(),
            selector_identity: "dynamic-selector:Sales[#Data]".to_string(),
            source_reference_handle: Some("structured-ref:dynamic-table".to_string()),
            target_kind,
            cause,
            before_resolved_table_identity: Some("tree-table:sales:v1".to_string()),
            after_resolved_table_identity: Some("tree-table:sales:v2".to_string()),
            caller_context_id: None,
            context_versions: TreeCalcTableLifecycleContextVersions::default(),
            oxfml_structured_bind_packet_available: true,
        }
    }

    fn amount_column_values() -> Vec<TreeCalcTableSparseValue> {
        vec![
            TreeCalcTableSparseValue::data("row:west", "col:amount", CalcValue::number(3.0)),
            TreeCalcTableSparseValue::data(
                "row:east",
                "col:amount",
                CalcValue::text(ExcelText::from_interop_assignment("")),
            ),
        ]
    }

    fn region_values() -> Vec<TreeCalcTableSparseValue> {
        vec![TreeCalcTableSparseValue::data(
            "row:west",
            "col:region",
            CalcValue::text(ExcelText::from_interop_assignment("West")),
        )]
    }

    fn table_formula_amount_values() -> Vec<TreeCalcTableSparseValue> {
        vec![
            TreeCalcTableSparseValue::data("row:west", "col:amount", CalcValue::number(10.0)),
            TreeCalcTableSparseValue::data("row:east", "col:amount", CalcValue::number(20.0)),
            TreeCalcTableSparseValue::data("row:north", "col:amount", CalcValue::number(30.0)),
        ]
    }

    fn tax_formula_runtime_request(
        formula_text: &str,
        values: Vec<TreeCalcTableSparseValue>,
        runtime_context: TreeCalcTableFormulaRuntimeContext,
    ) -> TreeCalcTableColumnFormulaRuntimeRequest {
        TreeCalcTableColumnFormulaRuntimeRequest {
            target_column_id: "col:tax".to_string(),
            formula_stable_id: "formula:body:tax".to_string(),
            formula_text_version: 1,
            formula_text: formula_text.to_string(),
            values,
            runtime_context,
        }
    }

    fn registry_with_test_udf() -> FunctionRegistry {
        let mut registry = FunctionRegistry::built_ins();
        let result = registry.register_udf_request(UdfRegistrationRequest {
            stable_source_registration_id: "test-source:table-formula".to_string(),
            function_id: "udf:table_formula_identity_probe".to_string(),
            surface_name: "TABLE_IDENTITY_PROBE".to_string(),
            source_kind: UdfSourceKind::XllRegisteredFunction,
            source_provenance: Some("calc-4vs8.49 test".to_string()),
            arity: Arity::exact(1),
            parameters: vec![ParameterDescriptor {
                name: "value".to_string(),
                optional: false,
                repeats: false,
                short_description: None,
            }],
            display_signature: None,
            determinism: DeterminismClass::Deterministic,
            volatility: VolatilityClass::NonVolatile,
            host_interaction: HostInteractionClass::None,
            thread_safety: ThreadSafetyClass::SafePure,
            arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
            coercion_lift_profile: CoercionLiftProfile::Custom,
            kernel_signature_class: KernelSignatureClass::Custom,
            fec_dependency_profile: FecDependencyProfile::None,
            surface_fec_dependency_profile: FecDependencyProfile::None,
            short_description: Some("table formula identity probe".to_string()),
            long_description: None,
            category: Some("User Defined".to_string()),
            execution_profile: Some(UdfExecutionProfile {
                host_execution_profile_key: Some("calc-4vs8.49".to_string()),
                required_capability_set_keys: Vec::new(),
                async_invocation: false,
                streaming: false,
                cancellable: false,
                externally_invalidated: false,
            }),
            invocation_target: Some(UdfInvocationTargetDescriptor::Xll {
                module_path: Some("IdentityProbe.xll".to_string()),
                export_name: "TableIdentityProbe".to_string(),
                type_text: Some("Q".to_string()),
                register_id: None,
                opaque_runtime_values: Vec::new(),
            }),
            replacement_policy: UdfReplacementPolicy::RejectOnCollision,
        });
        assert!(
            matches!(result, UdfRegistrationResult::Registered { .. }),
            "test UDF registration should mutate registry snapshot, got {result:?}"
        );
        registry
    }

    fn registry_with_host_callback_test_udf() -> FunctionRegistry {
        let registry = registry_with_test_udf();
        let mut entries = registry.iter().cloned().collect::<Vec<_>>();
        let entry = entries
            .iter_mut()
            .find(|entry| {
                entry
                    .surface_name
                    .eq_ignore_ascii_case("TABLE_IDENTITY_PROBE")
            })
            .expect("test UDF should be present in registry entries");
        entry.registry_metadata.runtime_boundary_kind = Some("vba_host_callback".to_string());
        entry.registry_metadata.interface_contract_ref =
            Some("oxfml.host_function_provider.v1".to_string());
        entry.registry_metadata.preparation_owner = Some("oxfunc-registry".to_string());
        FunctionRegistry::try_from_entries(entries)
            .expect("host-callback test registry entries should remain collision-free")
    }

    #[derive(Default)]
    struct RecordingTableUdfProvider {
        invocations: RefCell<Vec<HostFunctionInvocation>>,
    }

    impl HostFunctionProvider for RecordingTableUdfProvider {
        fn invoke_host_function(
            &self,
            invocation: &HostFunctionInvocation,
        ) -> Result<CalcValue, HostFunctionProviderError> {
            self.invocations.borrow_mut().push(invocation.clone());
            match invocation.args.as_slice() {
                [
                    CalcValue {
                        core: oxfunc_core::value::CoreValue::Number(value),
                        ..
                    },
                ] if invocation
                    .function_name
                    .eq_ignore_ascii_case("TABLE_IDENTITY_PROBE") =>
                {
                    Ok(CalcValue::number(*value))
                }
                _ => Err(HostFunctionProviderError::new(
                    "unsupported table UDF test invocation",
                )),
            }
        }
    }

    fn table_formula_cell_for_row<'a>(
        report: &'a TreeCalcTableFormulaRuntimeReport,
        row_id: &str,
    ) -> &'a TreeCalcTableFormulaRuntimeCellResult {
        report
            .cell_results
            .iter()
            .find(|cell| cell.row_id.as_ref().is_some_and(|row| row.0 == row_id))
            .unwrap_or_else(|| panic!("missing runtime cell for row {row_id}"))
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

    fn table_formula_bind_records(
        projection: &TreeCalcTableNodeProjection,
        formula_text: &str,
        caller_region: Option<TableCallerRegion>,
    ) -> Vec<StructuredReferenceBindRecord> {
        let caller_region = caller_region.unwrap_or_else(|| TableCallerRegion {
            table_id: projection.table_id.clone(),
            region_kind: TableRegionKind::Data,
            data_row_offset: Some(0),
        });
        let request = TreeCalcTableColumnFormulaRuntimeRequest {
            target_column_id: "col:amount".to_string(),
            formula_stable_id: "test:table-formula-bind-records".to_string(),
            formula_text_version: 1,
            formula_text: formula_text.to_string(),
            values: Vec::new(),
            runtime_context: TreeCalcTableFormulaRuntimeContext::default(),
        };
        bind_treecalc_table_formula_structured_references(
            projection,
            &request,
            &caller_region,
            &table_primary_locus(&projection.table_descriptor),
        )
        .expect("OxFml structured-reference binding should produce records")
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
            StructuredTableDependencyFactKind::RowValue,
            StructuredTableDependencyFactStatus::Lowered,
            None
        )));
        assert!(kinds.contains(&(
            StructuredTableDependencyFactKind::ColumnIdentity,
            StructuredTableDependencyFactStatus::Lowered,
            None
        )));
        assert!(kinds.contains(&(
            StructuredTableDependencyFactKind::ColumnOrder,
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
            StructuredTableDependencyFactKind::TotalsValue,
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
        assert!(kinds.contains(&(
            StructuredTableDependencyFactKind::VirtualAnchorRange,
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
        assert!(
            lowering
                .descriptors
                .iter()
                .any(|descriptor| descriptor.carrier_detail
                    == "table_totals_region:v1:table=table:sales;region=A5:C5")
        );
        assert!(
            lowering
                .descriptors
                .iter()
                .any(|descriptor| descriptor.carrier_detail
                    == "table_totals_value:v1:table=table:sales;region=A5:C5")
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
    fn treecalc_table_dependency_inventory_covers_full_w056_fact_surface() {
        let snapshot = runtime_treecalc_table_snapshot();
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let context_versions = TreeCalcTableLifecycleContextVersions {
            registry_snapshot_identity: "oxfunc-registry:udf:v2".to_string(),
            workspace_availability_version: Some("treecalc-workspace-availability:v2".to_string()),
            workspace_alias_version: Some("treecalc-workspace-alias:alias-v2".to_string()),
            ..TreeCalcTableLifecycleContextVersions::default()
        };

        let inventory = inventory_treecalc_table_dependency_facts(
            &snapshot,
            &projection,
            &context_versions,
            Some("caller-context:row-east"),
            true,
        );
        let kinds = inventory
            .facts
            .iter()
            .map(|fact| fact.kind)
            .collect::<BTreeSet<_>>();

        for kind in [
            StructuredTableDependencyFactKind::TableIdentity,
            StructuredTableDependencyFactKind::RowMembership,
            StructuredTableDependencyFactKind::RowOrder,
            StructuredTableDependencyFactKind::RowValue,
            StructuredTableDependencyFactKind::ColumnIdentity,
            StructuredTableDependencyFactKind::ColumnOrder,
            StructuredTableDependencyFactKind::HeaderText,
            StructuredTableDependencyFactKind::HeaderRegion,
            StructuredTableDependencyFactKind::DataRegion,
            StructuredTableDependencyFactKind::TotalsRegion,
            StructuredTableDependencyFactKind::TotalsValue,
            StructuredTableDependencyFactKind::TotalsFormula,
            StructuredTableDependencyFactKind::CallerRowContext,
            StructuredTableDependencyFactKind::OmittedTableNameEnclosingTable,
            StructuredTableDependencyFactKind::VirtualAnchorRange,
            StructuredTableDependencyFactKind::WorkspaceAvailability,
            StructuredTableDependencyFactKind::FunctionRegistrySnapshot,
        ] {
            assert!(kinds.contains(&kind), "inventory missing {kind:?}");
        }

        assert_eq!(
            inventory.registry_snapshot_identity,
            "oxfunc-registry:udf:v2"
        );
        assert_eq!(
            inventory.workspace_availability_version.as_deref(),
            Some("treecalc-workspace-availability:v2")
        );
        assert!(inventory.facts.iter().any(|fact| {
            fact.kind == StructuredTableDependencyFactKind::TotalsFormula
                && fact
                    .identity
                    .as_deref()
                    .is_some_and(|identity| identity.contains("formula:totals:amount"))
        }));
        assert!(
            inventory
                .facts
                .iter()
                .all(|fact| fact.status == StructuredTableDependencyFactStatus::Lowered)
        );

        let no_registry_inventory = inventory_treecalc_table_dependency_facts(
            &snapshot,
            &projection,
            &context_versions,
            Some("caller-context:row-east"),
            false,
        );
        assert!(
            !no_registry_inventory.facts.iter().any(
                |fact| fact.kind == StructuredTableDependencyFactKind::FunctionRegistrySnapshot
            )
        );
        assert_eq!(
            StructuredTableDependencyFactKind::WorkspaceAvailability.descriptor_kind(),
            DependencyDescriptorKind::HostSensitive
        );
    }

    #[test]
    fn dynamic_table_rebind_covers_targets_and_stable_save_reopen() {
        let column = classify_treecalc_dynamic_table_rebind(&dynamic_table_rebind_request(
            TreeCalcDynamicTableReferenceTargetKind::Column,
            TreeCalcDynamicTableRebindCause::SelectorTextChanged,
        ));
        assert_eq!(
            column.status,
            TreeCalcDynamicTableRebindStatus::RebindRequired
        );
        assert!(
            column
                .dependency_fact_kinds
                .contains(&StructuredTableDependencyFactKind::ColumnIdentity)
        );
        assert!(
            column
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::DynamicPotential)
        );
        assert!(
            column
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::StructuredTableDataRegion)
        );
        assert!(
            column
                .invalidation_reasons
                .contains(&InvalidationReasonKind::DynamicDependencyActivated)
        );
        assert!(
            column
                .invalidation_reasons
                .contains(&InvalidationReasonKind::DynamicDependencyReleased)
        );
        assert!(
            column
                .prepared_identity_inputs
                .contains(&TreeCalcTablePreparedIdentityInput::TableContextIdentity)
        );
        assert!(
            column
                .prepared_identity_inputs
                .contains(&TreeCalcTablePreparedIdentityInput::DynamicSelectorIdentity)
        );
        assert!(
            !column
                .prepared_identity_inputs
                .contains(&TreeCalcTablePreparedIdentityInput::CallerContextIdentity)
        );
        assert!(column.oxfml_generic_bind_packet_available);
        assert!(column.oxfunc_opaque_reference_admitted);

        let same_table_selector_changed =
            classify_treecalc_dynamic_table_rebind(&TreeCalcDynamicTableRebindRequest {
                selector_identity: "dynamic-selector:Sales[Amount]".to_string(),
                after_resolved_table_identity: Some("tree-table:sales:v1".to_string()),
                ..dynamic_table_rebind_request(
                    TreeCalcDynamicTableReferenceTargetKind::Column,
                    TreeCalcDynamicTableRebindCause::SelectorTextChanged,
                )
            });
        assert_eq!(
            same_table_selector_changed.status,
            TreeCalcDynamicTableRebindStatus::RebindRequired
        );
        assert!(
            same_table_selector_changed
                .prepared_identity_inputs
                .contains(&TreeCalcTablePreparedIdentityInput::DynamicSelectorIdentity)
        );
        assert_ne!(
            column.dynamic_rebind_identity,
            same_table_selector_changed.dynamic_rebind_identity
        );

        let current_row =
            classify_treecalc_dynamic_table_rebind(&TreeCalcDynamicTableRebindRequest {
                target_kind: TreeCalcDynamicTableReferenceTargetKind::CurrentRow,
                cause: TreeCalcDynamicTableRebindCause::TableLifecycle(
                    TreeCalcTableUpdateScenarioKind::RowReorder,
                ),
                caller_context_id: Some("caller:row-east".to_string()),
                ..dynamic_table_rebind_request(
                    TreeCalcDynamicTableReferenceTargetKind::CurrentRow,
                    TreeCalcDynamicTableRebindCause::VolatileReevaluation,
                )
            });
        assert_eq!(
            current_row.status,
            TreeCalcDynamicTableRebindStatus::RebindRequired
        );
        assert!(
            current_row
                .dependency_fact_kinds
                .contains(&StructuredTableDependencyFactKind::CallerRowContext)
        );
        assert!(
            current_row
                .invalidation_reasons
                .contains(&InvalidationReasonKind::StructuredTableRowOrderChanged)
        );
        assert!(
            current_row
                .prepared_identity_inputs
                .contains(&TreeCalcTablePreparedIdentityInput::CallerContextIdentity)
        );

        let stable = classify_treecalc_dynamic_table_rebind(&TreeCalcDynamicTableRebindRequest {
            cause: TreeCalcDynamicTableRebindCause::TableLifecycle(
                TreeCalcTableUpdateScenarioKind::SaveReopen,
            ),
            after_resolved_table_identity: Some("tree-table:sales:v1".to_string()),
            ..dynamic_table_rebind_request(
                TreeCalcDynamicTableReferenceTargetKind::Table,
                TreeCalcDynamicTableRebindCause::VolatileReevaluation,
            )
        });
        assert_eq!(
            stable.status,
            TreeCalcDynamicTableRebindStatus::ReferencePreserving
        );
        assert!(stable.changed_dependency_kinds.is_empty());
        assert!(stable.invalidation_reasons.is_empty());
        assert!(stable.prepared_identity_inputs.is_empty());
    }

    #[test]
    fn dynamic_table_rebind_covers_lifecycle_workspace_and_typed_exclusions() {
        for (scenario, expected_status) in [
            (
                TreeCalcTableUpdateScenarioKind::TableRename,
                TreeCalcDynamicTableRebindStatus::RebindRequired,
            ),
            (
                TreeCalcTableUpdateScenarioKind::TableMove,
                TreeCalcDynamicTableRebindStatus::RebindRequired,
            ),
            (
                TreeCalcTableUpdateScenarioKind::TableDelete,
                TreeCalcDynamicTableRebindStatus::DeletedTarget,
            ),
            (
                TreeCalcTableUpdateScenarioKind::NodeDelete,
                TreeCalcDynamicTableRebindStatus::DeletedTarget,
            ),
        ] {
            let report =
                classify_treecalc_dynamic_table_rebind(&TreeCalcDynamicTableRebindRequest {
                    cause: TreeCalcDynamicTableRebindCause::TableLifecycle(scenario),
                    ..dynamic_table_rebind_request(
                        TreeCalcDynamicTableReferenceTargetKind::Table,
                        TreeCalcDynamicTableRebindCause::VolatileReevaluation,
                    )
                });
            assert_eq!(report.status, expected_status, "{scenario:?}");
            assert!(
                report
                    .prepared_identity_inputs
                    .contains(&TreeCalcTablePreparedIdentityInput::HostNamespaceVersion)
            );
            assert!(
                report
                    .prepared_identity_inputs
                    .contains(&TreeCalcTablePreparedIdentityInput::ResolutionRuleVersion)
            );
        }

        let unavailable =
            classify_treecalc_dynamic_table_rebind(&TreeCalcDynamicTableRebindRequest {
                target_kind: TreeCalcDynamicTableReferenceTargetKind::CrossWorkspaceTable,
                cause: TreeCalcDynamicTableRebindCause::TableLifecycle(
                    TreeCalcTableUpdateScenarioKind::WorkspaceClose,
                ),
                after_resolved_table_identity: None,
                ..dynamic_table_rebind_request(
                    TreeCalcDynamicTableReferenceTargetKind::CrossWorkspaceTable,
                    TreeCalcDynamicTableRebindCause::VolatileReevaluation,
                )
            });
        assert_eq!(
            unavailable.status,
            TreeCalcDynamicTableRebindStatus::UnavailableTarget
        );
        assert!(
            unavailable
                .dependency_fact_kinds
                .contains(&StructuredTableDependencyFactKind::WorkspaceAvailability)
        );
        assert!(
            unavailable
                .dependency_fact_kinds
                .contains(&StructuredTableDependencyFactKind::RowMembership)
        );
        assert!(
            unavailable
                .dependency_fact_kinds
                .contains(&StructuredTableDependencyFactKind::RowOrder)
        );
        assert!(
            unavailable
                .dependency_fact_kinds
                .contains(&StructuredTableDependencyFactKind::ColumnIdentity)
        );
        assert!(
            unavailable
                .dependency_fact_kinds
                .contains(&StructuredTableDependencyFactKind::DataRegion)
        );
        assert!(
            unavailable
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::HostSensitive)
        );
        assert!(
            unavailable
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::StructuredTableRowMembership)
        );
        assert!(
            unavailable
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::StructuredTableDataRegion)
        );
        assert!(
            unavailable
                .invalidation_reasons
                .contains(&InvalidationReasonKind::DependencyRemoved)
        );

        let unsupported =
            classify_treecalc_dynamic_table_rebind(&TreeCalcDynamicTableRebindRequest {
                cause:
                    TreeCalcDynamicTableRebindCause::UnsupportedRuntimeStructuredReferenceParsing,
                oxfml_structured_bind_packet_available: false,
                ..dynamic_table_rebind_request(
                    TreeCalcDynamicTableReferenceTargetKind::Section,
                    TreeCalcDynamicTableRebindCause::VolatileReevaluation,
                )
            });
        assert_eq!(
            unsupported.status,
            TreeCalcDynamicTableRebindStatus::TypedExclusion
        );
        assert!(unsupported.dependency_fact_kinds.is_empty());
        assert!(!unsupported.oxfunc_opaque_reference_admitted);
        assert!(unsupported.diagnostics.iter().any(|diagnostic| {
            diagnostic.kind
                == TreeCalcDynamicTableRebindDiagnosticKind::UnsupportedRuntimeStructuredReferenceParsing
        }));

        let non_table =
            classify_treecalc_dynamic_table_rebind(&TreeCalcDynamicTableRebindRequest {
                cause: TreeCalcDynamicTableRebindCause::DynamicTargetNotTable,
                ..dynamic_table_rebind_request(
                    TreeCalcDynamicTableReferenceTargetKind::Table,
                    TreeCalcDynamicTableRebindCause::VolatileReevaluation,
                )
            });
        assert_eq!(
            non_table.status,
            TreeCalcDynamicTableRebindStatus::TypedExclusion
        );
        assert!(non_table.dependency_fact_kinds.is_empty());
        assert!(!non_table.oxfunc_opaque_reference_admitted);
        assert!(
            non_table
                .changed_dependency_kinds
                .contains(&DependencyDescriptorKind::Unresolved)
        );
        assert!(non_table.diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == TreeCalcDynamicTableRebindDiagnosticKind::DynamicTargetNotTable
        }));

        let missing_caller = classify_treecalc_dynamic_table_rebind(&dynamic_table_rebind_request(
            TreeCalcDynamicTableReferenceTargetKind::CurrentRow,
            TreeCalcDynamicTableRebindCause::DynamicFunctionResultChanged,
        ));
        assert_eq!(
            missing_caller.status,
            TreeCalcDynamicTableRebindStatus::TypedExclusion
        );
        assert!(missing_caller.diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == TreeCalcDynamicTableRebindDiagnosticKind::MissingCallerContext
        }));
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
                    role: None,
                    is_meta: false,
                },
                StructuralNode {
                    node_id: TreeNodeId(10),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "Total".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                    role: None,
                    is_meta: false,
                },
            ],
        )
        .unwrap();

        let graph = DependencyGraph::build(&snapshot, &lowering.descriptors);

        assert!(graph.diagnostics.is_empty());
        assert_eq!(graph.edges_by_owner.len(), 0);
        assert_eq!(graph.descriptors_by_owner[&TreeNodeId(10)].len(), 8);
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
