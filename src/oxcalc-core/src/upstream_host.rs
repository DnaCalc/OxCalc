#![forbid(unsafe_code)]

//! Minimal OxFml upstream host interfaces for deterministic scaffolding.

use std::collections::BTreeMap;

use oxfml_core::EvaluationBackend;
use oxfml_core::binding::{BindContext, NameKind};
use oxfml_core::consumer::replay::{
    ReplayProjectionRequest, ReplayProjectionResult, ReplayProjectionService,
};
use oxfml_core::consumer::runtime::{
    RuntimeEnvironment, RuntimeFormulaRequest, RuntimeFormulaResult,
};
use oxfml_core::eval::DefinedNameBinding;
use oxfml_core::interface::{
    TableCallerRegion, TableDescriptor, TableRef, TypedContextQueryBundle,
};
use oxfml_core::semantics::LibraryContextSnapshot;
use oxfml_core::source::{
    FormulaChannelKind, FormulaSourceRecord, FormulaToken, StructureContextVersion,
};
use oxfunc_core::functions::rtd_fn::{RtdProvider, RtdProviderResult, RtdRequest};
use oxfunc_core::host_info::{CellInfoQuery, HostInfoError, HostInfoProvider, InfoQuery};
use oxfunc_core::locale_format::{LocaleFormatContext, current_excel_host_context, en_us_context};
use oxfunc_core::value::{EvalValue, ExcelText, ReferenceLike, WorksheetErrorCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpstreamHostAnchor {
    pub row: u32,
    pub col: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinimalAddressMode {
    A1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpstreamDefinedNameBinding {
    Value(EvalValue),
    Reference(ReferenceLike),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimalFormulaSlotFacts {
    pub fixture_input_id: String,
    pub formula_slot_id: Option<String>,
    pub formula_stable_id: String,
    pub formula_token: String,
    pub bind_artifact_id: Option<String>,
    pub formula_text: String,
    pub formula_text_version: u64,
    pub formula_channel_kind: FormulaChannelKind,
    pub address_mode: MinimalAddressMode,
    pub caller_anchor: UpstreamHostAnchor,
    pub active_selection_anchor: Option<UpstreamHostAnchor>,
    pub structure_context_version: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MinimalBindingWorld {
    pub cell_fixture: BTreeMap<String, EvalValue>,
    pub defined_name_bindings: BTreeMap<String, UpstreamDefinedNameBinding>,
    pub table_catalog: Vec<TableDescriptor>,
    pub enclosing_table_ref: Option<TableRef>,
    pub caller_table_region: Option<TableCallerRegion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MinimalHostInfoMode {
    Disabled,
    UnsupportedQueries,
    ProviderFailure { detail: String },
    DirectoryValue { value: String },
    FilenameProviderFailure { detail: String },
    DirectoryValueAndFilenameProviderFailure { value: String, detail: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MinimalRtdMode {
    Disabled,
    CapabilityDenied,
    NoValueYet,
    ConnectionFailed,
    ProviderError { code: WorksheetErrorCode },
    Value(EvalValue),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MinimalLocaleContextKind {
    #[default]
    Disabled,
    EnUs,
    CurrentExcelHost,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimalTypedQueryFacts {
    pub host_info_mode: MinimalHostInfoMode,
    pub rtd_mode: MinimalRtdMode,
    pub locale_context_kind: MinimalLocaleContextKind,
    pub now_serial: Option<f64>,
    pub random_value: Option<f64>,
    pub registered_external_present: bool,
}

impl Default for MinimalTypedQueryFacts {
    fn default() -> Self {
        Self {
            host_info_mode: MinimalHostInfoMode::Disabled,
            rtd_mode: MinimalRtdMode::Disabled,
            locale_context_kind: MinimalLocaleContextKind::Disabled,
            now_serial: None,
            random_value: None,
            registered_external_present: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MinimalRuntimeCatalogFacts {
    pub library_context_snapshot: Option<LibraryContextSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimalUpstreamHostPacket {
    pub formula_slot: MinimalFormulaSlotFacts,
    pub binding_world: MinimalBindingWorld,
    pub typed_query_facts: MinimalTypedQueryFacts,
    pub runtime_catalog: MinimalRuntimeCatalogFacts,
}

impl MinimalUpstreamHostPacket {
    #[must_use]
    pub fn build_bind_context(&self) -> BindContext {
        BindContext {
            caller_row: self.formula_slot.caller_anchor.row,
            caller_col: self.formula_slot.caller_anchor.col,
            formula_token: FormulaToken(self.formula_slot.formula_token.clone()),
            structure_context_version: StructureContextVersion(
                self.formula_slot.structure_context_version.clone(),
            ),
            names: self
                .binding_world
                .defined_name_bindings
                .iter()
                .map(|(name, binding)| {
                    let kind = match binding {
                        UpstreamDefinedNameBinding::Value(_) => NameKind::ValueLike,
                        UpstreamDefinedNameBinding::Reference(_) => NameKind::ReferenceLike,
                    };
                    (name.clone(), kind)
                })
                .collect(),
            table_catalog: self.binding_world.table_catalog.clone(),
            enclosing_table_ref: self.binding_world.enclosing_table_ref.clone(),
            caller_table_region: self.binding_world.caller_table_region.clone(),
            ..BindContext::default()
        }
    }

    #[must_use]
    pub fn build_runtime_environment(&self) -> RuntimeEnvironment<'static> {
        let defined_names = self
            .binding_world
            .defined_name_bindings
            .iter()
            .map(|(name, binding)| {
                let binding = match binding {
                    UpstreamDefinedNameBinding::Value(value) => {
                        DefinedNameBinding::Value(value.clone())
                    }
                    UpstreamDefinedNameBinding::Reference(reference) => {
                        DefinedNameBinding::Reference(reference.clone())
                    }
                };
                (name.clone(), binding)
            })
            .collect();

        let mut environment = RuntimeEnvironment::new()
            .with_structure_context_version(StructureContextVersion(
                self.formula_slot.structure_context_version.clone(),
            ))
            .with_caller_position(
                self.formula_slot.caller_anchor.row,
                self.formula_slot.caller_anchor.col,
            )
            .with_defined_names(defined_names)
            .with_cell_values(self.binding_world.cell_fixture.clone())
            .with_table_context(
                self.binding_world.table_catalog.clone(),
                self.binding_world.enclosing_table_ref.clone(),
                self.binding_world.caller_table_region.clone(),
            );

        if let Some(snapshot) = &self.runtime_catalog.library_context_snapshot {
            environment = environment.with_inline_library_context_snapshot(snapshot.clone());
        }

        environment
    }

    #[must_use]
    pub fn build_formula_source_record(&self) -> FormulaSourceRecord {
        FormulaSourceRecord::new(
            self.formula_slot.formula_stable_id.clone(),
            self.formula_slot.formula_text_version,
            self.formula_slot.formula_text.clone(),
        )
        .with_formula_channel_kind(self.formula_slot.formula_channel_kind)
    }

    pub fn recalc(&self, backend: EvaluationBackend) -> Result<RuntimeFormulaResult, String> {
        let host_info_provider = PacketHostInfoProvider {
            mode: self.typed_query_facts.host_info_mode.clone(),
        };
        let rtd_provider = PacketRtdProvider {
            mode: self.typed_query_facts.rtd_mode.clone(),
        };
        let locale_ctx = locale_context(self.typed_query_facts.locale_context_kind);
        let query_bundle = TypedContextQueryBundle::new(
            (!matches!(
                self.typed_query_facts.host_info_mode,
                MinimalHostInfoMode::Disabled
            ))
            .then_some(&host_info_provider as &dyn HostInfoProvider),
            (!matches!(self.typed_query_facts.rtd_mode, MinimalRtdMode::Disabled))
                .then_some(&rtd_provider as &dyn RtdProvider),
            locale_ctx.as_ref(),
            self.typed_query_facts.now_serial,
            self.typed_query_facts.random_value,
        );

        self.build_runtime_environment().execute(
            RuntimeFormulaRequest::new(self.build_formula_source_record(), query_bundle)
                .with_backend(backend),
        )
    }

    pub fn recalc_with_replay_projection(
        &self,
        backend: EvaluationBackend,
    ) -> Result<(RuntimeFormulaResult, ReplayProjectionResult), String> {
        let output = self.recalc(backend)?;
        let projection =
            ReplayProjectionService::project(ReplayProjectionRequest::runtime_result(&output));
        Ok((output, projection))
    }
}

#[derive(Debug, Clone)]
struct PacketHostInfoProvider {
    mode: MinimalHostInfoMode,
}

impl HostInfoProvider for PacketHostInfoProvider {
    fn query_cell_info(
        &self,
        query: CellInfoQuery,
        _reference: Option<&ReferenceLike>,
    ) -> Result<EvalValue, HostInfoError> {
        match (&self.mode, query) {
            (
                MinimalHostInfoMode::FilenameProviderFailure { detail }
                | MinimalHostInfoMode::DirectoryValueAndFilenameProviderFailure { detail, .. },
                CellInfoQuery::Filename,
            ) => Err(HostInfoError::ProviderFailure {
                detail: detail.clone(),
            }),
            (_, query) => Err(HostInfoError::UnsupportedCellInfoQuery(query)),
        }
    }

    fn query_info(&self, query: InfoQuery) -> Result<EvalValue, HostInfoError> {
        match (&self.mode, query) {
            (MinimalHostInfoMode::Disabled, _) => Err(HostInfoError::ProviderFailure {
                detail: "host_info.disabled".to_string(),
            }),
            (MinimalHostInfoMode::UnsupportedQueries, query) => {
                Err(HostInfoError::UnsupportedInfoQuery(query))
            }
            (MinimalHostInfoMode::ProviderFailure { detail }, _) => {
                Err(HostInfoError::ProviderFailure {
                    detail: detail.clone(),
                })
            }
            (MinimalHostInfoMode::DirectoryValue { value }, InfoQuery::Directory)
            | (
                MinimalHostInfoMode::DirectoryValueAndFilenameProviderFailure { value, .. },
                InfoQuery::Directory,
            ) => Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                value.encode_utf16().collect(),
            ))),
            (MinimalHostInfoMode::FilenameProviderFailure { .. }, query)
            | (MinimalHostInfoMode::DirectoryValue { .. }, query)
            | (MinimalHostInfoMode::DirectoryValueAndFilenameProviderFailure { .. }, query) => {
                Err(HostInfoError::UnsupportedInfoQuery(query))
            }
        }
    }
}

#[derive(Debug, Clone)]
struct PacketRtdProvider {
    mode: MinimalRtdMode,
}

impl RtdProvider for PacketRtdProvider {
    fn resolve_rtd(&self, _request: &RtdRequest) -> RtdProviderResult {
        match &self.mode {
            MinimalRtdMode::Disabled => RtdProviderResult::CapabilityDenied,
            MinimalRtdMode::CapabilityDenied => RtdProviderResult::CapabilityDenied,
            MinimalRtdMode::NoValueYet => RtdProviderResult::NoValueYet,
            MinimalRtdMode::ConnectionFailed => RtdProviderResult::ConnectionFailed,
            MinimalRtdMode::ProviderError { code } => RtdProviderResult::ProviderError(*code),
            MinimalRtdMode::Value(value) => RtdProviderResult::Value(value.clone()),
        }
    }
}

fn locale_context(kind: MinimalLocaleContextKind) -> Option<LocaleFormatContext<'static>> {
    match kind {
        MinimalLocaleContextKind::Disabled => None,
        MinimalLocaleContextKind::EnUs => Some(en_us_context()),
        MinimalLocaleContextKind::CurrentExcelHost => Some(current_excel_host_context()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxfml_core::interface::{
        HostProviderOutcomeKind, LibraryContextSnapshotRef, ReturnedValueSurfaceKind,
        TableCallerRegion, TableColumnDescriptor, TableDescriptor, TableRef, TableRegionKind,
        TypedContextQueryFamily,
    };
    use oxfml_core::semantics::{LibraryContextSnapshotEntry, RegistrationSourceKind};
    use oxfunc_core::value::WorksheetErrorCode;

    fn packet(formula_text: &str) -> MinimalUpstreamHostPacket {
        MinimalUpstreamHostPacket {
            formula_slot: MinimalFormulaSlotFacts {
                fixture_input_id: "fixture:host:001".to_string(),
                formula_slot_id: Some("node:slot:1".to_string()),
                formula_stable_id: "formula:host:001".to_string(),
                formula_token: "formula:host:001:1".to_string(),
                bind_artifact_id: Some("bind:host:001".to_string()),
                formula_text: formula_text.to_string(),
                formula_text_version: 1,
                formula_channel_kind: FormulaChannelKind::WorksheetA1,
                address_mode: MinimalAddressMode::A1,
                caller_anchor: UpstreamHostAnchor { row: 1, col: 1 },
                active_selection_anchor: Some(UpstreamHostAnchor { row: 1, col: 1 }),
                structure_context_version: "treecalc.struct:v1".to_string(),
            },
            binding_world: MinimalBindingWorld::default(),
            typed_query_facts: MinimalTypedQueryFacts::default(),
            runtime_catalog: MinimalRuntimeCatalogFacts::default(),
        }
    }

    fn snapshot_with_entry(surface_name: &str) -> LibraryContextSnapshot {
        LibraryContextSnapshot {
            snapshot_id: "snapshot:test".to_string(),
            snapshot_version: "v1".to_string(),
            entries: vec![LibraryContextSnapshotEntry {
                surface_name: surface_name.to_string(),
                canonical_id: Some(format!("FUNC.{surface_name}")),
                surface_stable_id: Some(format!("surface:{surface_name}")),
                name_resolution_table_ref: Some("name-table:v1".to_string()),
                semantic_trait_profile_ref: Some("traits:v1".to_string()),
                gating_profile_ref: Some("gating:v1".to_string()),
                metadata_status: Some("runtime".to_string()),
                special_interface_kind: None,
                admission_interface_kind: Some("ordinary".to_string()),
                preparation_owner: Some("oxfunc".to_string()),
                runtime_boundary_kind: Some("host_query".to_string()),
                arity_shape_note: None,
                interface_contract_ref: Some("iface:v1".to_string()),
                registration_source_kind: RegistrationSourceKind::BuiltIn,
                parse_bind_state: oxfml_core::semantics::LibraryAvailabilityState::CatalogKnown,
                semantic_plan_state: oxfml_core::semantics::LibraryAvailabilityState::CatalogKnown,
                runtime_capability_state: Some(
                    oxfml_core::semantics::LibraryAvailabilityState::CatalogKnown,
                ),
                post_dispatch_state: Some(
                    oxfml_core::semantics::LibraryAvailabilityState::CatalogKnown,
                ),
            }],
        }
    }

    #[test]
    fn minimal_upstream_host_packet_projects_bind_context() {
        let mut packet = packet("=SUM(InputValue,2)");
        packet.binding_world.defined_name_bindings.insert(
            "InputValue".to_string(),
            UpstreamDefinedNameBinding::Value(EvalValue::Number(5.0)),
        );
        packet.binding_world.table_catalog = vec![TableDescriptor {
            table_id: "table:1".to_string(),
            table_name: "Sales".to_string(),
            workbook_scope_ref: "book:1".to_string(),
            sheet_scope_ref: "sheet:1".to_string(),
            table_range_ref: "A1:B4".to_string(),
            header_row_present: true,
            totals_row_present: false,
            columns: vec![TableColumnDescriptor {
                column_id: "table:1:col:1".to_string(),
                column_name: "Amount".to_string(),
                ordinal: 1,
                column_range_ref: "B2:B4".to_string(),
            }],
        }];
        packet.binding_world.enclosing_table_ref = Some(TableRef {
            table_id: "table:1".to_string(),
        });
        packet.binding_world.caller_table_region = Some(TableCallerRegion {
            table_id: "table:1".to_string(),
            region_kind: TableRegionKind::Data,
            data_row_offset: Some(0),
        });

        let bind_context = packet.build_bind_context();

        assert_eq!(bind_context.caller_row, 1);
        assert_eq!(bind_context.caller_col, 1);
        assert_eq!(bind_context.formula_token.0, "formula:host:001:1");
        assert_eq!(
            bind_context.structure_context_version.0,
            "treecalc.struct:v1"
        );
        assert_eq!(
            packet.formula_slot.formula_stable_id,
            "formula:host:001".to_string()
        );
        assert_eq!(
            packet.formula_slot.formula_channel_kind,
            FormulaChannelKind::WorksheetA1
        );
        assert_eq!(packet.formula_slot.address_mode, MinimalAddressMode::A1);
        assert_eq!(bind_context.names["InputValue"], NameKind::ValueLike);
        assert_eq!(bind_context.table_catalog.len(), 1);
        assert_eq!(
            packet.formula_slot.bind_artifact_id.as_deref(),
            Some("bind:host:001")
        );
        assert_eq!(
            bind_context.enclosing_table_ref,
            Some(TableRef {
                table_id: "table:1".to_string()
            })
        );
        assert_eq!(
            bind_context.caller_table_region,
            Some(TableCallerRegion {
                table_id: "table:1".to_string(),
                region_kind: TableRegionKind::Data,
                data_row_offset: Some(0),
            })
        );
    }

    #[test]
    fn minimal_upstream_host_packet_drives_real_oxfml_recalc() {
        let mut packet = packet("=SUM(InputValue,2)");
        packet.binding_world.defined_name_bindings.insert(
            "InputValue".to_string(),
            UpstreamDefinedNameBinding::Value(EvalValue::Number(5.0)),
        );
        packet.runtime_catalog.library_context_snapshot = Some(snapshot_with_entry("SUM"));

        let output = packet.recalc(EvaluationBackend::OxFuncBacked).unwrap();

        assert_eq!(
            output.candidate_result.value_delta.published_payload,
            oxfml_core::seam::ValuePayload::Number("7".to_string())
        );
        assert_eq!(
            output
                .library_context_snapshot_ref
                .as_ref()
                .unwrap()
                .snapshot_id,
            "snapshot:test"
        );
        assert!(matches!(
            output.commit_decision,
            oxfml_core::seam::AcceptDecision::Accepted(_)
        ));
    }

    #[test]
    fn minimal_upstream_host_packet_exposes_typed_host_query_families() {
        let mut packet = packet("=INFO(\"directory\")");
        packet.typed_query_facts.host_info_mode = MinimalHostInfoMode::FilenameProviderFailure {
            detail: "fixture.host_info".to_string(),
        };
        packet.typed_query_facts.locale_context_kind = MinimalLocaleContextKind::CurrentExcelHost;
        packet.typed_query_facts.now_serial = Some(46000.0);
        packet.typed_query_facts.random_value = Some(0.25);
        packet.typed_query_facts.registered_external_present = true;

        let output = packet.recalc(EvaluationBackend::OxFuncBacked).unwrap();

        assert_eq!(
            output.returned_value_surface.kind,
            ReturnedValueSurfaceKind::TypedHostProviderOutcome
        );
        assert!(
            output
                .typed_query_bundle_spec
                .families
                .contains(&TypedContextQueryFamily::Info)
        );
        assert!(
            output
                .typed_query_bundle_spec
                .families
                .contains(&TypedContextQueryFamily::LocaleFormatContext)
        );
        assert!(
            output
                .typed_query_bundle_spec
                .families
                .contains(&TypedContextQueryFamily::NowSerial)
        );
        assert!(
            output
                .typed_query_bundle_spec
                .families
                .contains(&TypedContextQueryFamily::RandomValue)
        );
        assert!(packet.typed_query_facts.registered_external_present);
    }

    #[test]
    fn minimal_upstream_host_packet_supports_reference_and_rtd_stand_ins() {
        let mut packet = packet("=RTD(\"prog\",\"server\",\"topic\")");
        packet.binding_world.defined_name_bindings.insert(
            "RefValue".to_string(),
            UpstreamDefinedNameBinding::Reference(ReferenceLike {
                kind: oxfunc_core::value::ReferenceKind::A1,
                target: "A1".to_string(),
            }),
        );
        packet.typed_query_facts.rtd_mode = MinimalRtdMode::ProviderError {
            code: WorksheetErrorCode::Value,
        };

        let bind_context = packet.build_bind_context();
        let output = packet.recalc(EvaluationBackend::OxFuncBacked).unwrap();

        assert_eq!(bind_context.names["RefValue"], NameKind::ReferenceLike);
        assert_eq!(
            output.returned_value_surface.payload_summary,
            "ProviderError(Value)"
        );
        assert_eq!(
            output
                .returned_value_surface
                .host_provider_outcome
                .unwrap()
                .worksheet_error,
            Some(WorksheetErrorCode::Value)
        );
    }

    #[test]
    fn minimal_upstream_host_packet_supports_host_info_values_and_replay_projection() {
        let mut packet = packet("=INFO(\"directory\")");
        packet.typed_query_facts.host_info_mode =
            MinimalHostInfoMode::DirectoryValueAndFilenameProviderFailure {
                value: "C:\\Work".to_string(),
                detail: "filename_unavailable".to_string(),
            };
        packet.runtime_catalog.library_context_snapshot = Some(snapshot_with_entry("INFO"));

        let (output, replay_projection) = packet
            .recalc_with_replay_projection(EvaluationBackend::OxFuncBacked)
            .unwrap();

        assert_eq!(
            output.returned_value_surface.kind,
            ReturnedValueSurfaceKind::OrdinaryValue
        );
        assert_eq!(
            replay_projection.library_context_snapshot_ref,
            Some(LibraryContextSnapshotRef::new("snapshot:test", "v1"))
        );
        assert_eq!(replay_projection.formula_stable_id, "formula:host:001");
    }

    #[test]
    fn minimal_upstream_host_packet_supports_unsupported_query_outcomes() {
        let mut packet = packet("=INFO(\"system\")");
        packet.typed_query_facts.host_info_mode = MinimalHostInfoMode::UnsupportedQueries;

        let output = packet.recalc(EvaluationBackend::OxFuncBacked).unwrap();

        assert_eq!(
            output.returned_value_surface.kind,
            ReturnedValueSurfaceKind::TypedHostProviderOutcome
        );
        assert_eq!(
            output
                .returned_value_surface
                .host_provider_outcome
                .as_ref()
                .map(|surface| surface.outcome_kind),
            Some(HostProviderOutcomeKind::UnsupportedQuery)
        );
    }
}
