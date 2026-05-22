#![forbid(unsafe_code)]

//! OxCalc-owned structured table dependency lowering.
//!
//! This module consumes public OxFml table-context packets. It does not parse
//! structured-reference formula text and does not mirror OxFml grammar.

use std::collections::{BTreeMap, BTreeSet};

use oxfml_core::{
    StructuredReferenceBindRecord, StructuredSectionKind,
    interface::{TableCallerRegion, TableDescriptor, TableRef, TableRegionKind},
};

use crate::dependency::{DependencyDescriptor, DependencyDescriptorKind};
use crate::structural::TreeNodeId;

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
    let table_parts = table_catalog
        .iter()
        .map(|table| {
            let columns = table
                .columns
                .iter()
                .map(|column| {
                    format!(
                        "{}:{}:{}:{}",
                        column.column_id,
                        column.ordinal,
                        column.column_name,
                        column.column_range_ref
                    )
                })
                .collect::<Vec<_>>()
                .join("|");
            format!(
                "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
                table.table_id,
                table.table_name,
                table.workbook_scope_ref,
                table.sheet_scope_ref,
                table.table_range_ref,
                table.row_membership_identity.as_deref().unwrap_or("none"),
                table.row_order_identity.as_deref().unwrap_or("none"),
                table.header_region_ref.as_deref().unwrap_or("none"),
                table.totals_region_ref.as_deref().unwrap_or("none"),
                table.header_row_present,
                table.totals_row_present
            ) + ":"
                + &columns
        })
        .collect::<Vec<_>>()
        .join(";");
    let enclosing = enclosing_table_ref
        .as_ref()
        .map_or("none".to_string(), |table_ref| table_ref.table_id.clone());
    let caller = caller_table_region
        .as_ref()
        .map_or("none".to_string(), |region| {
            format!(
                "{}:{:?}:{}",
                region.table_id,
                region.region_kind,
                region
                    .data_row_offset
                    .map_or("none".to_string(), |offset| offset.to_string())
            )
        });
    format!("oxcalc.table_context.v1:tables={table_parts};enclosing={enclosing};caller={caller}")
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
