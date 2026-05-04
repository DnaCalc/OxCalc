#![forbid(unsafe_code)]

//! Checked-in fixture loading for minimal upstream-host scaffolding packets.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::upstream_host::{
    MinimalAddressMode, MinimalFormulaSlotFacts, MinimalHostInfoMode, MinimalLocaleContextKind,
    MinimalRtdMode, MinimalRuntimeCatalogFacts, MinimalTypedQueryFacts, MinimalUpstreamHostPacket,
    UpstreamDefinedNameBinding, UpstreamHostAnchor,
};
use oxfml_core::EvaluationBackend;
use oxfml_core::interface::{TableCallerRegion, TableColumnDescriptor, TableDescriptor, TableRef};
use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use oxfml_core::source::FormulaChannelKind;
use oxfunc_core::value::{ArrayCellValue, EvalArray, EvalValue, ExcelText, WorksheetErrorCode};

const UPSTREAM_HOST_FIXTURE_MANIFEST_SCHEMA_V1: &str = "oxcalc.upstream_host.fixture_manifest.v1";
const UPSTREAM_HOST_FIXTURE_CASE_SCHEMA_V1: &str = "oxcalc.upstream_host.fixture_case.v1";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpstreamHostFixtureManifest {
    pub schema_version: String,
    pub corpus_id: String,
    pub base_path: String,
    pub cases: Vec<UpstreamHostFixtureManifestCase>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpstreamHostFixtureManifestCase {
    pub case_id: String,
    pub path: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpstreamHostFixtureCase {
    pub schema_version: String,
    pub case_id: String,
    pub description: String,
    pub formula_slot: UpstreamHostFixtureFormulaSlot,
    #[serde(default)]
    pub binding_world: UpstreamHostFixtureBindingWorld,
    #[serde(default)]
    pub typed_query_facts: UpstreamHostFixtureTypedQueryFacts,
    #[serde(default)]
    pub runtime_catalog: UpstreamHostFixtureRuntimeCatalogFacts,
    #[serde(default = "default_backend")]
    pub evaluation_backend: String,
    pub expected: UpstreamHostFixtureExpected,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpstreamHostFixtureFormulaSlot {
    pub fixture_input_id: String,
    pub formula_slot_id: Option<String>,
    pub formula_stable_id: String,
    #[serde(default)]
    pub formula_token: Option<String>,
    #[serde(default)]
    pub bind_artifact_id: Option<String>,
    pub formula_text: String,
    pub formula_text_version: u64,
    pub formula_channel_kind: String,
    #[serde(default)]
    pub address_mode: Option<String>,
    pub caller_anchor: UpstreamHostFixtureAnchor,
    pub active_selection_anchor: Option<UpstreamHostFixtureAnchor>,
    pub structure_context_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpstreamHostFixtureAnchor {
    pub row: u32,
    pub col: u32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UpstreamHostFixtureBindingWorld {
    #[serde(default)]
    pub cell_fixture_numbers: BTreeMap<String, f64>,
    #[serde(default)]
    pub cell_fixture_arrays: Vec<UpstreamHostFixtureArrayBinding>,
    #[serde(default)]
    pub defined_name_values: BTreeMap<String, f64>,
    #[serde(default)]
    pub table_catalog: Vec<UpstreamHostFixtureTableDescriptor>,
    pub enclosing_table_ref: Option<String>,
    pub caller_table_region: Option<UpstreamHostFixtureCallerTableRegion>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpstreamHostFixtureTableDescriptor {
    pub table_id: String,
    pub table_name: String,
    pub workbook_scope_ref: String,
    pub sheet_scope_ref: String,
    pub table_range_ref: String,
    pub header_row_present: bool,
    pub totals_row_present: bool,
    #[serde(default)]
    pub columns: Vec<UpstreamHostFixtureTableColumnDescriptor>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpstreamHostFixtureTableColumnDescriptor {
    pub column_id: String,
    pub column_name: String,
    pub ordinal: u32,
    pub column_range_ref: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpstreamHostFixtureCallerTableRegion {
    pub table_id: String,
    pub region_kind: String,
    pub data_row_offset: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpstreamHostFixtureArrayBinding {
    pub target: String,
    pub rows: Vec<Vec<UpstreamHostFixtureArrayCell>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UpstreamHostFixtureArrayCell {
    Number { value: f64 },
    Text { value: String },
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UpstreamHostFixtureHostInfoMode {
    #[default]
    Disabled,
    UnsupportedQueries,
    ProviderFailure {
        detail: String,
    },
    DirectoryValue {
        value: String,
    },
    FilenameProviderFailure {
        detail: String,
    },
    DirectoryValueAndFilenameProviderFailure {
        value: String,
        detail: String,
    },
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UpstreamHostFixtureRtdMode {
    #[default]
    Disabled,
    CapabilityDenied,
    NoValueYet,
    ConnectionFailed,
    ProviderError {
        worksheet_error: String,
    },
    Value {
        number: f64,
    },
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UpstreamHostFixtureTypedQueryFacts {
    #[serde(default)]
    pub host_info_mode: UpstreamHostFixtureHostInfoMode,
    #[serde(default)]
    pub rtd_mode: UpstreamHostFixtureRtdMode,
    #[serde(default)]
    pub locale_context_kind: String,
    pub now_serial: Option<f64>,
    pub random_value: Option<f64>,
    #[serde(default)]
    pub registered_external_present: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UpstreamHostFixtureRuntimeCatalogFacts {
    #[serde(default)]
    pub surface_names: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpstreamHostFixtureExpected {
    pub returned_surface_kind: String,
    #[serde(default)]
    pub payload_summary: Option<String>,
    #[serde(default)]
    pub host_provider_outcome_kind: Option<String>,
    #[serde(default)]
    pub worksheet_error: Option<String>,
    #[serde(default)]
    pub capture_snapshot_ref: Option<UpstreamHostFixtureSnapshotRef>,
    #[serde(default)]
    pub bind_name_kinds: BTreeMap<String, String>,
    pub formula_token: Option<String>,
    pub bind_artifact_id: Option<String>,
    pub table_catalog_len: Option<usize>,
    pub enclosing_table_ref: Option<String>,
    pub caller_table_region: Option<UpstreamHostFixtureCallerTableRegion>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpstreamHostFixtureSnapshotRef {
    pub snapshot_id: String,
    pub snapshot_version: String,
}

#[derive(Debug, Clone)]
pub struct UpstreamHostFixtureExecution {
    pub packet: MinimalUpstreamHostPacket,
    pub bind_context: oxfml_core::binding::BindContext,
    pub recalc_output: oxfml_core::consumer::runtime::RuntimeFormulaResult,
    pub replay_projection: Option<oxfml_core::consumer::replay::ReplayProjectionResult>,
}

#[derive(Debug, Error)]
pub enum UpstreamHostFixtureError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse json from {path}: {source}")]
    Parse {
        path: String,
        source: serde_json::Error,
    },
    #[error("unsupported manifest schema version '{schema_version}'")]
    UnsupportedManifestSchema { schema_version: String },
    #[error("unsupported fixture case schema version '{schema_version}'")]
    UnsupportedCaseSchema { schema_version: String },
    #[error("unsupported formula channel kind '{kind}'")]
    UnsupportedFormulaChannelKind { kind: String },
    #[error("unsupported locale context kind '{kind}'")]
    UnsupportedLocaleContextKind { kind: String },
    #[error("unsupported table region kind '{kind}'")]
    UnsupportedTableRegionKind { kind: String },
    #[error("unsupported evaluation backend '{backend}'")]
    UnsupportedEvaluationBackend { backend: String },
    #[error("unsupported worksheet error code '{code}'")]
    UnsupportedWorksheetErrorCode { code: String },
    #[error("fixture case '{case_id}' failed to recalc: {detail}")]
    Runtime { case_id: String, detail: String },
}

pub fn load_manifest(path: &Path) -> Result<UpstreamHostFixtureManifest, UpstreamHostFixtureError> {
    let text = fs::read_to_string(path).map_err(|source| UpstreamHostFixtureError::Read {
        path: path.display().to_string(),
        source,
    })?;
    let manifest =
        serde_json::from_str::<UpstreamHostFixtureManifest>(&text).map_err(|source| {
            UpstreamHostFixtureError::Parse {
                path: path.display().to_string(),
                source,
            }
        })?;
    if manifest.schema_version != UPSTREAM_HOST_FIXTURE_MANIFEST_SCHEMA_V1 {
        return Err(UpstreamHostFixtureError::UnsupportedManifestSchema {
            schema_version: manifest.schema_version,
        });
    }
    Ok(manifest)
}

pub fn load_case(path: &Path) -> Result<UpstreamHostFixtureCase, UpstreamHostFixtureError> {
    let text = fs::read_to_string(path).map_err(|source| UpstreamHostFixtureError::Read {
        path: path.display().to_string(),
        source,
    })?;
    let case = serde_json::from_str::<UpstreamHostFixtureCase>(&text).map_err(|source| {
        UpstreamHostFixtureError::Parse {
            path: path.display().to_string(),
            source,
        }
    })?;
    if case.schema_version != UPSTREAM_HOST_FIXTURE_CASE_SCHEMA_V1 {
        return Err(UpstreamHostFixtureError::UnsupportedCaseSchema {
            schema_version: case.schema_version,
        });
    }
    Ok(case)
}

pub fn execute_fixture_case(
    case: &UpstreamHostFixtureCase,
) -> Result<UpstreamHostFixtureExecution, UpstreamHostFixtureError> {
    let packet = build_packet(case)?;
    let bind_context = packet.build_bind_context();
    let backend = parse_backend(&case.evaluation_backend)?;
    let (recalc_output, replay_projection) = if case.expected.capture_snapshot_ref.is_some() {
        let (output, replay_projection) =
            packet
                .recalc_with_replay_projection(backend)
                .map_err(|detail| UpstreamHostFixtureError::Runtime {
                    case_id: case.case_id.clone(),
                    detail,
                })?;
        (output, Some(replay_projection))
    } else {
        let output =
            packet
                .recalc(backend)
                .map_err(|detail| UpstreamHostFixtureError::Runtime {
                    case_id: case.case_id.clone(),
                    detail,
                })?;
        (output, None)
    };

    Ok(UpstreamHostFixtureExecution {
        packet,
        bind_context,
        recalc_output,
        replay_projection,
    })
}

fn build_packet(
    case: &UpstreamHostFixtureCase,
) -> Result<MinimalUpstreamHostPacket, UpstreamHostFixtureError> {
    Ok(MinimalUpstreamHostPacket {
        formula_slot: MinimalFormulaSlotFacts {
            fixture_input_id: case.formula_slot.fixture_input_id.clone(),
            formula_slot_id: case.formula_slot.formula_slot_id.clone(),
            formula_stable_id: case.formula_slot.formula_stable_id.clone(),
            formula_token: case.formula_slot.formula_token.clone().unwrap_or_else(|| {
                format!(
                    "{}:{}",
                    case.formula_slot.formula_stable_id, case.formula_slot.formula_text_version
                )
            }),
            bind_artifact_id: case.formula_slot.bind_artifact_id.clone(),
            formula_text: case.formula_slot.formula_text.clone(),
            formula_text_version: case.formula_slot.formula_text_version,
            formula_channel_kind: parse_formula_channel_kind(
                &case.formula_slot.formula_channel_kind,
            )?,
            address_mode: parse_address_mode(case.formula_slot.address_mode.as_deref())?,
            caller_anchor: UpstreamHostAnchor {
                row: case.formula_slot.caller_anchor.row,
                col: case.formula_slot.caller_anchor.col,
            },
            active_selection_anchor: case.formula_slot.active_selection_anchor.as_ref().map(
                |anchor| UpstreamHostAnchor {
                    row: anchor.row,
                    col: anchor.col,
                },
            ),
            structure_context_version: case.formula_slot.structure_context_version.clone(),
        },
        binding_world: crate::upstream_host::MinimalBindingWorld {
            cell_fixture: build_cell_fixture(&case.binding_world)?,
            defined_name_bindings: case
                .binding_world
                .defined_name_values
                .iter()
                .map(|(name, value)| {
                    (
                        name.clone(),
                        UpstreamDefinedNameBinding::Value(EvalValue::Number(*value)),
                    )
                })
                .collect(),
            table_catalog: case
                .binding_world
                .table_catalog
                .iter()
                .map(to_table_descriptor)
                .collect(),
            enclosing_table_ref: case
                .binding_world
                .enclosing_table_ref
                .as_ref()
                .map(|table_id| TableRef {
                    table_id: table_id.clone(),
                }),
            caller_table_region: case
                .binding_world
                .caller_table_region
                .as_ref()
                .map(to_caller_table_region)
                .transpose()?,
        },
        typed_query_facts: MinimalTypedQueryFacts {
            host_info_mode: parse_host_info_mode(&case.typed_query_facts.host_info_mode),
            rtd_mode: parse_rtd_mode(&case.typed_query_facts.rtd_mode)?,
            locale_context_kind: parse_locale_context_kind(
                &case.typed_query_facts.locale_context_kind,
            )?,
            now_serial: case.typed_query_facts.now_serial,
            random_value: case.typed_query_facts.random_value,
            registered_external_present: case.typed_query_facts.registered_external_present,
        },
        runtime_catalog: MinimalRuntimeCatalogFacts {
            library_context_snapshot: (!case.runtime_catalog.surface_names.is_empty())
                .then(|| snapshot_with_entries(&case.runtime_catalog.surface_names)),
        },
    })
}

fn build_cell_fixture(
    binding_world: &UpstreamHostFixtureBindingWorld,
) -> Result<BTreeMap<String, EvalValue>, UpstreamHostFixtureError> {
    let mut cell_fixture = binding_world
        .cell_fixture_numbers
        .iter()
        .map(|(target, value)| (target.clone(), EvalValue::Number(*value)))
        .collect::<BTreeMap<_, _>>();

    for array_binding in &binding_world.cell_fixture_arrays {
        let rows = array_binding
            .rows
            .iter()
            .map(|row| row.iter().map(to_array_cell_value).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let array =
            EvalArray::from_rows(rows).ok_or_else(|| UpstreamHostFixtureError::Runtime {
                case_id: "fixture-array-shape".to_string(),
                detail: "invalid array fixture shape".to_string(),
            })?;
        cell_fixture.insert(array_binding.target.clone(), EvalValue::Array(array));
    }

    Ok(cell_fixture)
}

fn parse_formula_channel_kind(kind: &str) -> Result<FormulaChannelKind, UpstreamHostFixtureError> {
    match kind {
        "worksheet_a1" => Ok(FormulaChannelKind::WorksheetA1),
        _ => Err(UpstreamHostFixtureError::UnsupportedFormulaChannelKind {
            kind: kind.to_string(),
        }),
    }
}

fn parse_address_mode(mode: Option<&str>) -> Result<MinimalAddressMode, UpstreamHostFixtureError> {
    match mode.unwrap_or("a1") {
        "a1" => Ok(MinimalAddressMode::A1),
        _ => Err(UpstreamHostFixtureError::UnsupportedFormulaChannelKind {
            kind: mode.unwrap_or_default().to_string(),
        }),
    }
}

fn parse_locale_context_kind(
    kind: &str,
) -> Result<MinimalLocaleContextKind, UpstreamHostFixtureError> {
    match kind {
        "" | "disabled" => Ok(MinimalLocaleContextKind::Disabled),
        "en_us" => Ok(MinimalLocaleContextKind::EnUs),
        "current_excel_host" => Ok(MinimalLocaleContextKind::CurrentExcelHost),
        _ => Err(UpstreamHostFixtureError::UnsupportedLocaleContextKind {
            kind: kind.to_string(),
        }),
    }
}

fn parse_host_info_mode(mode: &UpstreamHostFixtureHostInfoMode) -> MinimalHostInfoMode {
    match mode {
        UpstreamHostFixtureHostInfoMode::Disabled => MinimalHostInfoMode::Disabled,
        UpstreamHostFixtureHostInfoMode::UnsupportedQueries => {
            MinimalHostInfoMode::UnsupportedQueries
        }
        UpstreamHostFixtureHostInfoMode::ProviderFailure { detail } => {
            MinimalHostInfoMode::ProviderFailure {
                detail: detail.clone(),
            }
        }
        UpstreamHostFixtureHostInfoMode::DirectoryValue { value } => {
            MinimalHostInfoMode::DirectoryValue {
                value: value.clone(),
            }
        }
        UpstreamHostFixtureHostInfoMode::FilenameProviderFailure { detail } => {
            MinimalHostInfoMode::FilenameProviderFailure {
                detail: detail.clone(),
            }
        }
        UpstreamHostFixtureHostInfoMode::DirectoryValueAndFilenameProviderFailure {
            value,
            detail,
        } => MinimalHostInfoMode::DirectoryValueAndFilenameProviderFailure {
            value: value.clone(),
            detail: detail.clone(),
        },
    }
}

fn parse_rtd_mode(
    mode: &UpstreamHostFixtureRtdMode,
) -> Result<MinimalRtdMode, UpstreamHostFixtureError> {
    Ok(match mode {
        UpstreamHostFixtureRtdMode::Disabled => MinimalRtdMode::Disabled,
        UpstreamHostFixtureRtdMode::CapabilityDenied => MinimalRtdMode::CapabilityDenied,
        UpstreamHostFixtureRtdMode::NoValueYet => MinimalRtdMode::NoValueYet,
        UpstreamHostFixtureRtdMode::ConnectionFailed => MinimalRtdMode::ConnectionFailed,
        UpstreamHostFixtureRtdMode::ProviderError { worksheet_error } => {
            MinimalRtdMode::ProviderError {
                code: parse_worksheet_error_code(worksheet_error)?,
            }
        }
        UpstreamHostFixtureRtdMode::Value { number } => {
            MinimalRtdMode::Value(EvalValue::Number(*number))
        }
    })
}

fn parse_backend(backend: &str) -> Result<EvaluationBackend, UpstreamHostFixtureError> {
    match backend {
        "oxfunc_backed" => Ok(EvaluationBackend::OxFuncBacked),
        _ => Err(UpstreamHostFixtureError::UnsupportedEvaluationBackend {
            backend: backend.to_string(),
        }),
    }
}

fn parse_worksheet_error_code(code: &str) -> Result<WorksheetErrorCode, UpstreamHostFixtureError> {
    match code {
        "value" => Ok(WorksheetErrorCode::Value),
        _ => Err(UpstreamHostFixtureError::UnsupportedWorksheetErrorCode {
            code: code.to_string(),
        }),
    }
}

fn to_table_descriptor(descriptor: &UpstreamHostFixtureTableDescriptor) -> TableDescriptor {
    TableDescriptor {
        table_id: descriptor.table_id.clone(),
        table_name: descriptor.table_name.clone(),
        workbook_scope_ref: descriptor.workbook_scope_ref.clone(),
        sheet_scope_ref: descriptor.sheet_scope_ref.clone(),
        table_range_ref: descriptor.table_range_ref.clone(),
        header_row_present: descriptor.header_row_present,
        totals_row_present: descriptor.totals_row_present,
        columns: descriptor
            .columns
            .iter()
            .map(|column| TableColumnDescriptor {
                column_id: column.column_id.clone(),
                column_name: column.column_name.clone(),
                ordinal: column.ordinal,
                column_range_ref: column.column_range_ref.clone(),
            })
            .collect(),
    }
}

fn to_array_cell_value(cell: &UpstreamHostFixtureArrayCell) -> ArrayCellValue {
    match cell {
        UpstreamHostFixtureArrayCell::Number { value } => ArrayCellValue::Number(*value),
        UpstreamHostFixtureArrayCell::Text { value } => {
            ArrayCellValue::Text(ExcelText::from_interop_assignment(value))
        }
    }
}

fn to_caller_table_region(
    region: &UpstreamHostFixtureCallerTableRegion,
) -> Result<TableCallerRegion, UpstreamHostFixtureError> {
    Ok(TableCallerRegion {
        table_id: region.table_id.clone(),
        region_kind: parse_table_region_kind(&region.region_kind)?,
        data_row_offset: region.data_row_offset,
    })
}

fn parse_table_region_kind(
    kind: &str,
) -> Result<oxfml_core::interface::TableRegionKind, UpstreamHostFixtureError> {
    match kind {
        "data" => Ok(oxfml_core::interface::TableRegionKind::Data),
        "header" | "headers" => Ok(oxfml_core::interface::TableRegionKind::Headers),
        "totals" => Ok(oxfml_core::interface::TableRegionKind::Totals),
        _ => Err(UpstreamHostFixtureError::UnsupportedTableRegionKind {
            kind: kind.to_string(),
        }),
    }
}

fn snapshot_with_entries(surface_names: &[String]) -> LibraryContextSnapshot {
    LibraryContextSnapshot {
        snapshot_id: "snapshot:fixture".to_string(),
        snapshot_version: "v1".to_string(),
        entries: surface_names
            .iter()
            .map(|surface_name| LibraryContextSnapshotEntry {
                surface_name: surface_name.clone(),
                canonical_id: Some(format!("FUNC.{surface_name}")),
                surface_stable_id: Some(format!("surface:{surface_name}")),
                name_resolution_table_ref: Some("name-table:fixture:v1".to_string()),
                semantic_trait_profile_ref: Some("traits:fixture:v1".to_string()),
                gating_profile_ref: Some("gating:fixture:v1".to_string()),
                metadata_status: Some("runtime".to_string()),
                special_interface_kind: None,
                admission_interface_kind: Some("ordinary".to_string()),
                preparation_owner: Some("oxfunc".to_string()),
                runtime_boundary_kind: Some("host_query".to_string()),
                interface_contract_ref: Some("iface:fixture:v1".to_string()),
                registration_source_kind: RegistrationSourceKind::BuiltIn,
                parse_bind_state: LibraryAvailabilityState::CatalogKnown,
                semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
                runtime_capability_state: Some(LibraryAvailabilityState::CatalogKnown),
                post_dispatch_state: Some(LibraryAvailabilityState::CatalogKnown),
            })
            .collect(),
    }
}

fn default_backend() -> String {
    "oxfunc_backed".to_string()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use oxfml_core::binding::NameKind;
    use oxfml_core::interface::{
        HostProviderOutcomeKind, LibraryContextSnapshotRef, ReturnedValueSurfaceKind,
    };

    #[test]
    fn checked_in_upstream_host_fixtures_execute_against_public_packet() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let manifest_path =
            repo_root.join("docs/test-fixtures/core-engine/upstream-host/MANIFEST.json");
        let manifest = load_manifest(&manifest_path).unwrap();

        assert_eq!(manifest.cases.len(), 9);

        for entry in &manifest.cases {
            let case_path = repo_root
                .join("docs/test-fixtures/core-engine/upstream-host")
                .join(entry.path.replace('/', "\\"));
            let case = load_case(&case_path).unwrap();
            let execution = execute_fixture_case(&case).unwrap();

            assert_eq!(
                execution.recalc_output.returned_value_surface.kind,
                parse_returned_surface_kind(&case.expected.returned_surface_kind)
            );

            if let Some(expected_payload_summary) = &case.expected.payload_summary {
                assert_eq!(
                    execution
                        .recalc_output
                        .returned_value_surface
                        .payload_summary,
                    *expected_payload_summary
                );
            }

            if let Some(expected_outcome_kind) = &case.expected.host_provider_outcome_kind {
                assert_eq!(
                    execution
                        .recalc_output
                        .returned_value_surface
                        .host_provider_outcome
                        .as_ref()
                        .map(|surface| surface.outcome_kind),
                    Some(parse_host_provider_outcome_kind(expected_outcome_kind))
                );
            }

            if let Some(expected_worksheet_error) = &case.expected.worksheet_error {
                assert_eq!(
                    execution
                        .recalc_output
                        .returned_value_surface
                        .host_provider_outcome
                        .as_ref()
                        .and_then(|surface| surface.worksheet_error),
                    Some(parse_worksheet_error_code(expected_worksheet_error).unwrap())
                );
            }

            if let Some(expected_capture_snapshot_ref) = &case.expected.capture_snapshot_ref {
                assert_eq!(
                    execution
                        .replay_projection
                        .as_ref()
                        .and_then(|packet| packet.library_context_snapshot_ref.clone()),
                    Some(LibraryContextSnapshotRef::new(
                        expected_capture_snapshot_ref.snapshot_id.clone(),
                        expected_capture_snapshot_ref.snapshot_version.clone(),
                    ))
                );
            }

            for (name, expected_kind) in &case.expected.bind_name_kinds {
                assert_eq!(
                    execution.bind_context.names.get(name),
                    Some(&parse_name_kind(expected_kind))
                );
            }

            if let Some(expected_table_catalog_len) = case.expected.table_catalog_len {
                assert_eq!(
                    execution.bind_context.table_catalog.len(),
                    expected_table_catalog_len
                );
            }

            if let Some(expected_enclosing_table_ref) = &case.expected.enclosing_table_ref {
                assert_eq!(
                    execution
                        .bind_context
                        .enclosing_table_ref
                        .as_ref()
                        .map(|table_ref| table_ref.table_id.clone()),
                    Some(expected_enclosing_table_ref.clone())
                );
            }

            if let Some(expected_caller_table_region) = &case.expected.caller_table_region {
                let observed_region = execution.bind_context.caller_table_region.as_ref().unwrap();
                assert_eq!(
                    observed_region.table_id,
                    expected_caller_table_region.table_id
                );
                assert_eq!(
                    observed_region.region_kind,
                    parse_table_region_kind(&expected_caller_table_region.region_kind).unwrap()
                );
                assert_eq!(
                    observed_region.data_row_offset,
                    expected_caller_table_region.data_row_offset
                );
            }
        }
    }

    fn parse_returned_surface_kind(kind: &str) -> ReturnedValueSurfaceKind {
        match kind {
            "ordinary_value" => ReturnedValueSurfaceKind::OrdinaryValue,
            "typed_host_provider_outcome" => ReturnedValueSurfaceKind::TypedHostProviderOutcome,
            other => panic!("unsupported returned surface kind in test: {other}"),
        }
    }

    fn parse_host_provider_outcome_kind(kind: &str) -> HostProviderOutcomeKind {
        match kind {
            "unsupported_query" => HostProviderOutcomeKind::UnsupportedQuery,
            other => panic!("unsupported host provider outcome kind in test: {other}"),
        }
    }

    fn parse_name_kind(kind: &str) -> NameKind {
        match kind {
            "value_like" => NameKind::ValueLike,
            "reference_like" => NameKind::ReferenceLike,
            other => panic!("unsupported name kind in test: {other}"),
        }
    }
}
