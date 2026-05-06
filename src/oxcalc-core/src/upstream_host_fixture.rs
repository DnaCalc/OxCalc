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
use oxfml_core::binding::NameKind;
use oxfml_core::interface::{
    HostProviderOutcomeKind, LibraryContextSnapshotRef, ReturnedValueSurfaceKind,
    TableCallerRegion, TableColumnDescriptor, TableDescriptor, TableRef,
};
use oxfml_core::publication::{
    AverageRuleOptions, ColorScaleRuleOptions, ColorScaleRuleStop, ConditionalFormattingRank,
    ConditionalFormattingThreshold, ConditionalFormattingTypedRule, DataBarDirection,
    DataBarRuleOptions, IconSetRuleOptions, RankRuleOptions, VerificationConditionalFormattingRule,
    VerificationPublicationContext,
};
use oxfml_core::seam::ValuePayload;
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
    #[serde(default)]
    pub publication_context: Option<UpstreamHostFixturePublicationContext>,
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

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UpstreamHostFixturePublicationContext {
    pub format_profile: Option<String>,
    pub number_format_code: Option<String>,
    pub style_id: Option<String>,
    #[serde(default)]
    pub style_hierarchy: Vec<String>,
    pub font_color: Option<String>,
    pub fill_color: Option<String>,
    #[serde(default)]
    pub conditional_formatting_rules: Vec<UpstreamHostFixtureConditionalFormattingRule>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UpstreamHostFixtureConditionalFormattingRule {
    #[serde(default)]
    pub target_ranges: Vec<String>,
    pub rule_kind: String,
    pub operator: Option<String>,
    #[serde(default)]
    pub thresholds: Vec<String>,
    pub typed_rule: Option<UpstreamHostFixtureConditionalFormattingTypedRule>,
    pub font_color: Option<String>,
    pub fill_color: Option<String>,
    pub effective_display_text: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UpstreamHostFixtureConditionalFormattingTypedRule {
    pub color_scale: Option<UpstreamHostFixtureColorScaleRuleOptions>,
    pub data_bar: Option<UpstreamHostFixtureDataBarRuleOptions>,
    pub icon_set: Option<UpstreamHostFixtureIconSetRuleOptions>,
    pub rank: Option<UpstreamHostFixtureRankRuleOptions>,
    pub average: Option<UpstreamHostFixtureAverageRuleOptions>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UpstreamHostFixtureColorScaleRuleOptions {
    #[serde(default)]
    pub stops: Vec<UpstreamHostFixtureColorScaleRuleStop>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpstreamHostFixtureColorScaleRuleStop {
    pub position: UpstreamHostFixtureConditionalFormattingThreshold,
    pub color: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UpstreamHostFixtureDataBarRuleOptions {
    pub minimum: Option<UpstreamHostFixtureConditionalFormattingThreshold>,
    pub maximum: Option<UpstreamHostFixtureConditionalFormattingThreshold>,
    pub bar_color: Option<String>,
    pub direction: Option<String>,
    #[serde(default)]
    pub show_bar_only: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UpstreamHostFixtureIconSetRuleOptions {
    pub set_kind: String,
    #[serde(default)]
    pub thresholds: Vec<UpstreamHostFixtureConditionalFormattingThreshold>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UpstreamHostFixtureConditionalFormattingThreshold {
    Min,
    Mid,
    Max,
    Percent { value: f64 },
    Percentile { value: f64 },
    Number { value: f64 },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpstreamHostFixtureRankRuleOptions {
    pub kind: String,
    pub value: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpstreamHostFixtureAverageRuleOptions {
    pub include_equal: bool,
    pub stddev_multiplier: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpstreamHostFixtureExpected {
    pub returned_surface_kind: String,
    #[serde(default)]
    pub payload_summary: Option<String>,
    #[serde(default)]
    pub candidate_value_payload: Option<String>,
    #[serde(default)]
    pub trace_function_ids: Vec<String>,
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
    #[serde(default)]
    pub conditional_formatting_typed_rule_families: Vec<String>,
    #[serde(default)]
    pub conditional_formatting_effective_fill_colors: Option<Vec<Vec<Option<String>>>>,
    #[serde(default)]
    pub conditional_formatting_data_bars:
        Option<Vec<Vec<Option<UpstreamHostFixtureDataBarExpectation>>>>,
    #[serde(default)]
    pub conditional_formatting_icons: Option<Vec<Vec<Option<UpstreamHostFixtureIconExpectation>>>>,
    #[serde(default)]
    pub format_delta_present: Option<bool>,
    #[serde(default)]
    pub display_delta_present: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpstreamHostFixtureDataBarExpectation {
    pub fill_ratio: f64,
    pub bar_color: String,
    pub direction: String,
    pub show_bar_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamHostFixtureIconExpectation {
    pub set_kind: String,
    pub icon_index: usize,
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

#[must_use]
pub fn fixture_expectation_mismatches(
    case: &UpstreamHostFixtureCase,
    execution: &UpstreamHostFixtureExecution,
) -> Vec<String> {
    let mut mismatches = Vec::new();
    let expected = &case.expected;
    let output = &execution.recalc_output;

    match parse_returned_surface_kind(&expected.returned_surface_kind) {
        Some(expected_kind) if output.returned_value_surface.kind != expected_kind => {
            mismatches.push(format!(
                "returned_surface_kind expected {:?}, observed {:?}",
                expected_kind, output.returned_value_surface.kind
            ));
        }
        Some(_) => {}
        None => mismatches.push(format!(
            "unsupported expected returned_surface_kind '{}'",
            expected.returned_surface_kind
        )),
    }

    if let Some(expected_payload_summary) = &expected.payload_summary
        && output.returned_value_surface.payload_summary != *expected_payload_summary
    {
        mismatches.push(format!(
            "payload_summary expected '{}', observed '{}'",
            expected_payload_summary, output.returned_value_surface.payload_summary
        ));
    }

    if let Some(expected_value_payload) = &expected.candidate_value_payload {
        let observed =
            value_payload_summary(&output.candidate_result.value_delta.published_payload);
        if observed != *expected_value_payload {
            mismatches.push(format!(
                "candidate_value_payload expected '{}', observed '{}'",
                expected_value_payload, observed
            ));
        }
    }

    if !expected.trace_function_ids.is_empty() {
        let observed = trace_function_ids(output);
        if observed != expected.trace_function_ids {
            mismatches.push(format!(
                "trace_function_ids expected {:?}, observed {:?}",
                expected.trace_function_ids, observed
            ));
        }
    }

    if let Some(expected_outcome_kind) = &expected.host_provider_outcome_kind {
        match parse_host_provider_outcome_kind(expected_outcome_kind) {
            Some(expected_kind) => {
                let observed = output
                    .returned_value_surface
                    .host_provider_outcome
                    .as_ref()
                    .map(|surface| surface.outcome_kind);
                if observed != Some(expected_kind) {
                    mismatches.push(format!(
                        "host_provider_outcome_kind expected {:?}, observed {:?}",
                        expected_kind, observed
                    ));
                }
            }
            None => mismatches.push(format!(
                "unsupported expected host_provider_outcome_kind '{}'",
                expected_outcome_kind
            )),
        }
    }

    if let Some(expected_worksheet_error) = &expected.worksheet_error {
        match parse_worksheet_error_code(expected_worksheet_error) {
            Ok(expected_error) => {
                let observed = output
                    .returned_value_surface
                    .host_provider_outcome
                    .as_ref()
                    .and_then(|surface| surface.worksheet_error);
                if observed != Some(expected_error) {
                    mismatches.push(format!(
                        "worksheet_error expected {:?}, observed {:?}",
                        expected_error, observed
                    ));
                }
            }
            Err(error) => mismatches.push(error.to_string()),
        }
    }

    if let Some(expected_capture_snapshot_ref) = &expected.capture_snapshot_ref {
        let expected_ref = LibraryContextSnapshotRef::new(
            expected_capture_snapshot_ref.snapshot_id.clone(),
            expected_capture_snapshot_ref.snapshot_version.clone(),
        );
        let observed = execution
            .replay_projection
            .as_ref()
            .and_then(|packet| packet.library_context_snapshot_ref.clone());
        if observed != Some(expected_ref.clone()) {
            mismatches.push(format!(
                "capture_snapshot_ref expected {:?}, observed {:?}",
                expected_ref, observed
            ));
        }
    }

    for (name, expected_kind) in &expected.bind_name_kinds {
        match parse_name_kind(expected_kind) {
            Some(expected_kind) => {
                let observed = execution.bind_context.names.get(name);
                if observed != Some(&expected_kind) {
                    mismatches.push(format!(
                        "bind_name_kind for '{}' expected {:?}, observed {:?}",
                        name, expected_kind, observed
                    ));
                }
            }
            None => mismatches.push(format!(
                "unsupported expected bind_name_kind '{}' for '{}'",
                expected_kind, name
            )),
        }
    }

    if let Some(expected_table_catalog_len) = expected.table_catalog_len
        && execution.bind_context.table_catalog.len() != expected_table_catalog_len
    {
        mismatches.push(format!(
            "table_catalog_len expected {}, observed {}",
            expected_table_catalog_len,
            execution.bind_context.table_catalog.len()
        ));
    }

    if let Some(expected_enclosing_table_ref) = &expected.enclosing_table_ref {
        let observed = execution
            .bind_context
            .enclosing_table_ref
            .as_ref()
            .map(|table_ref| table_ref.table_id.clone());
        if observed.as_ref() != Some(expected_enclosing_table_ref) {
            mismatches.push(format!(
                "enclosing_table_ref expected '{}', observed {:?}",
                expected_enclosing_table_ref, observed
            ));
        }
    }

    if let Some(expected_caller_table_region) = &expected.caller_table_region {
        match execution.bind_context.caller_table_region.as_ref() {
            Some(observed_region) => {
                if observed_region.table_id != expected_caller_table_region.table_id {
                    mismatches.push(format!(
                        "caller_table_region.table_id expected '{}', observed '{}'",
                        expected_caller_table_region.table_id, observed_region.table_id
                    ));
                }
                match parse_table_region_kind(&expected_caller_table_region.region_kind) {
                    Ok(expected_kind) if observed_region.region_kind != expected_kind => {
                        mismatches.push(format!(
                            "caller_table_region.region_kind expected {:?}, observed {:?}",
                            expected_kind, observed_region.region_kind
                        ));
                    }
                    Ok(_) => {}
                    Err(error) => mismatches.push(error.to_string()),
                }
                if observed_region.data_row_offset != expected_caller_table_region.data_row_offset {
                    mismatches.push(format!(
                        "caller_table_region.data_row_offset expected {:?}, observed {:?}",
                        expected_caller_table_region.data_row_offset,
                        observed_region.data_row_offset
                    ));
                }
            }
            None => mismatches.push("caller_table_region expected but absent".to_string()),
        }
    }

    if !expected
        .conditional_formatting_typed_rule_families
        .is_empty()
    {
        let observed = conditional_formatting_typed_rule_families(output);
        if observed != expected.conditional_formatting_typed_rule_families {
            mismatches.push(format!(
                "conditional_formatting_typed_rule_families expected {:?}, observed {:?}",
                expected.conditional_formatting_typed_rule_families, observed
            ));
        }
    }

    if let Some(expected_fills) = &expected.conditional_formatting_effective_fill_colors {
        let observed = array_cell_effective_fill_colors(output);
        if observed.as_ref() != Some(expected_fills) {
            mismatches.push(format!(
                "conditional_formatting_effective_fill_colors expected {:?}, observed {:?}",
                expected_fills, observed
            ));
        }
    }

    if let Some(expected_bars) = &expected.conditional_formatting_data_bars {
        let observed = array_cell_data_bars(output);
        if !data_bar_grid_matches(expected_bars, observed.as_ref()) {
            mismatches.push(format!(
                "conditional_formatting_data_bars expected {:?}, observed {:?}",
                expected_bars, observed
            ));
        }
    }

    if let Some(expected_icons) = &expected.conditional_formatting_icons {
        let observed = array_cell_icons(output);
        if observed.as_ref() != Some(expected_icons) {
            mismatches.push(format!(
                "conditional_formatting_icons expected {:?}, observed {:?}",
                expected_icons, observed
            ));
        }
    }

    if let Some(expected_present) = expected.format_delta_present {
        let observed = output
            .verification_publication_surface
            .format_delta
            .is_some();
        if observed != expected_present {
            mismatches.push(format!(
                "format_delta_present expected {}, observed {}",
                expected_present, observed
            ));
        }
    }

    if let Some(expected_present) = expected.display_delta_present {
        let observed = output
            .verification_publication_surface
            .display_delta
            .is_some();
        if observed != expected_present {
            mismatches.push(format!(
                "display_delta_present expected {}, observed {}",
                expected_present, observed
            ));
        }
    }

    mismatches
}

pub fn trace_function_ids(
    output: &oxfml_core::consumer::runtime::RuntimeFormulaResult,
) -> Vec<String> {
    output
        .evaluation
        .trace
        .prepared_calls
        .iter()
        .map(|call| call.function_id.to_string())
        .collect()
}

pub fn value_payload_summary(payload: &ValuePayload) -> String {
    match payload {
        ValuePayload::Number(value) => format!("Number({value})"),
        ValuePayload::Text(value) => format!("Text({value})"),
        ValuePayload::Logical(value) => format!("Logical({value})"),
        ValuePayload::ErrorCode(value) => format!("Error({value})"),
        ValuePayload::Blank => "Blank".to_string(),
    }
}

pub fn conditional_formatting_typed_rule_families(
    output: &oxfml_core::consumer::runtime::RuntimeFormulaResult,
) -> Vec<String> {
    output
        .verification_publication_surface
        .conditional_formatting_rules
        .iter()
        .filter_map(|rule| rule.typed_rule.as_ref())
        .flat_map(|typed_rule| {
            let mut families = Vec::new();
            if typed_rule.color_scale.is_some() {
                families.push("color_scale".to_string());
            }
            if typed_rule.data_bar.is_some() {
                families.push("data_bar".to_string());
            }
            if typed_rule.icon_set.is_some() {
                families.push("icon_set".to_string());
            }
            if typed_rule.rank.is_some() {
                families.push("rank".to_string());
            }
            if typed_rule.average.is_some() {
                families.push("average".to_string());
            }
            families
        })
        .collect()
}

pub fn array_cell_effective_fill_colors(
    output: &oxfml_core::consumer::runtime::RuntimeFormulaResult,
) -> Option<Vec<Vec<Option<String>>>> {
    output
        .verification_publication_surface
        .array_cell_format
        .as_ref()
        .map(|grid| {
            grid.rows
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|cell| cell.effective_fill_color.clone())
                        .collect()
                })
                .collect()
        })
}

pub fn array_cell_data_bars(
    output: &oxfml_core::consumer::runtime::RuntimeFormulaResult,
) -> Option<Vec<Vec<Option<UpstreamHostFixtureDataBarExpectation>>>> {
    output
        .verification_publication_surface
        .array_cell_format
        .as_ref()
        .map(|grid| {
            grid.rows
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|cell| {
                            cell.data_bar.as_ref().map(|bar| {
                                UpstreamHostFixtureDataBarExpectation {
                                    fill_ratio: bar.fill_ratio,
                                    bar_color: bar.bar_color.clone(),
                                    direction: match bar.direction {
                                        DataBarDirection::Left => "left".to_string(),
                                        DataBarDirection::Right => "right".to_string(),
                                    },
                                    show_bar_only: bar.show_bar_only,
                                }
                            })
                        })
                        .collect()
                })
                .collect()
        })
}

pub fn array_cell_icons(
    output: &oxfml_core::consumer::runtime::RuntimeFormulaResult,
) -> Option<Vec<Vec<Option<UpstreamHostFixtureIconExpectation>>>> {
    output
        .verification_publication_surface
        .array_cell_format
        .as_ref()
        .map(|grid| {
            grid.rows
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|cell| {
                            cell.icon
                                .as_ref()
                                .map(|icon| UpstreamHostFixtureIconExpectation {
                                    set_kind: icon.set_kind.clone(),
                                    icon_index: icon.icon_index,
                                })
                        })
                        .collect()
                })
                .collect()
        })
}

fn data_bar_grid_matches(
    expected: &[Vec<Option<UpstreamHostFixtureDataBarExpectation>>],
    observed: Option<&Vec<Vec<Option<UpstreamHostFixtureDataBarExpectation>>>>,
) -> bool {
    let Some(observed) = observed else {
        return false;
    };
    expected.len() == observed.len()
        && expected
            .iter()
            .zip(observed)
            .all(|(expected_row, observed_row)| {
                expected_row.len() == observed_row.len()
                    && expected_row.iter().zip(observed_row).all(
                        |(expected_cell, observed_cell)| {
                            data_bar_cell_matches(expected_cell.as_ref(), observed_cell.as_ref())
                        },
                    )
            })
}

fn data_bar_cell_matches(
    expected: Option<&UpstreamHostFixtureDataBarExpectation>,
    observed: Option<&UpstreamHostFixtureDataBarExpectation>,
) -> bool {
    match (expected, observed) {
        (None, None) => true,
        (Some(expected), Some(observed)) => {
            (expected.fill_ratio - observed.fill_ratio).abs() <= 0.000_000_001
                && expected.bar_color == observed.bar_color
                && expected.direction == observed.direction
                && expected.show_bar_only == observed.show_bar_only
        }
        _ => false,
    }
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
        publication_context: case
            .publication_context
            .as_ref()
            .map(to_publication_context),
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

fn parse_returned_surface_kind(kind: &str) -> Option<ReturnedValueSurfaceKind> {
    match kind {
        "ordinary_value" => Some(ReturnedValueSurfaceKind::OrdinaryValue),
        "typed_host_provider_outcome" => Some(ReturnedValueSurfaceKind::TypedHostProviderOutcome),
        "value_with_presentation" => Some(ReturnedValueSurfaceKind::ValueWithPresentation),
        "rich_value" => Some(ReturnedValueSurfaceKind::RichValue),
        _ => None,
    }
}

fn parse_host_provider_outcome_kind(kind: &str) -> Option<HostProviderOutcomeKind> {
    match kind {
        "value" => Some(HostProviderOutcomeKind::Value),
        "unsupported_query" => Some(HostProviderOutcomeKind::UnsupportedQuery),
        "provider_failure" => Some(HostProviderOutcomeKind::ProviderFailure),
        "provider_error" => Some(HostProviderOutcomeKind::ProviderError),
        "capability_denied" => Some(HostProviderOutcomeKind::CapabilityDenied),
        "no_value_yet" => Some(HostProviderOutcomeKind::NoValueYet),
        "connection_failed" => Some(HostProviderOutcomeKind::ConnectionFailed),
        _ => None,
    }
}

fn parse_name_kind(kind: &str) -> Option<NameKind> {
    match kind {
        "value_like" => Some(NameKind::ValueLike),
        "reference_like" => Some(NameKind::ReferenceLike),
        _ => None,
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

fn to_publication_context(
    context: &UpstreamHostFixturePublicationContext,
) -> VerificationPublicationContext {
    VerificationPublicationContext {
        format_profile: context.format_profile.clone(),
        number_format_code: context.number_format_code.clone(),
        style_id: context.style_id.clone(),
        style_hierarchy: context.style_hierarchy.clone(),
        font_color: context.font_color.clone(),
        fill_color: context.fill_color.clone(),
        conditional_formatting_rules: context
            .conditional_formatting_rules
            .iter()
            .map(to_conditional_formatting_rule)
            .collect(),
    }
}

fn to_conditional_formatting_rule(
    rule: &UpstreamHostFixtureConditionalFormattingRule,
) -> VerificationConditionalFormattingRule {
    VerificationConditionalFormattingRule {
        target_ranges: rule.target_ranges.clone(),
        rule_kind: rule.rule_kind.clone(),
        operator: rule.operator.clone(),
        thresholds: rule.thresholds.clone(),
        typed_rule: rule.typed_rule.as_ref().map(to_typed_rule),
        font_color: rule.font_color.clone(),
        fill_color: rule.fill_color.clone(),
        effective_display_text: rule.effective_display_text.clone(),
        applies: None,
        effective_font_color: None,
        effective_fill_color: None,
    }
}

fn to_typed_rule(
    typed_rule: &UpstreamHostFixtureConditionalFormattingTypedRule,
) -> ConditionalFormattingTypedRule {
    ConditionalFormattingTypedRule {
        color_scale: typed_rule
            .color_scale
            .as_ref()
            .map(to_color_scale_rule_options),
        data_bar: typed_rule.data_bar.as_ref().map(to_data_bar_rule_options),
        icon_set: typed_rule.icon_set.as_ref().map(to_icon_set_rule_options),
        rank: typed_rule.rank.as_ref().map(to_rank_rule_options),
        average: typed_rule.average.as_ref().map(to_average_rule_options),
    }
}

fn to_color_scale_rule_options(
    options: &UpstreamHostFixtureColorScaleRuleOptions,
) -> ColorScaleRuleOptions {
    ColorScaleRuleOptions {
        stops: options
            .stops
            .iter()
            .map(|stop| ColorScaleRuleStop {
                position: to_conditional_formatting_threshold(&stop.position),
                color: stop.color.clone(),
            })
            .collect(),
    }
}

fn to_data_bar_rule_options(options: &UpstreamHostFixtureDataBarRuleOptions) -> DataBarRuleOptions {
    DataBarRuleOptions {
        minimum: options
            .minimum
            .as_ref()
            .map(to_conditional_formatting_threshold),
        maximum: options
            .maximum
            .as_ref()
            .map(to_conditional_formatting_threshold),
        bar_color: options.bar_color.clone(),
        direction: options.direction.as_deref().map(to_data_bar_direction),
        show_bar_only: options.show_bar_only,
    }
}

fn to_icon_set_rule_options(options: &UpstreamHostFixtureIconSetRuleOptions) -> IconSetRuleOptions {
    IconSetRuleOptions {
        set_kind: options.set_kind.clone(),
        thresholds: options
            .thresholds
            .iter()
            .map(to_conditional_formatting_threshold)
            .collect(),
    }
}

fn to_conditional_formatting_threshold(
    threshold: &UpstreamHostFixtureConditionalFormattingThreshold,
) -> ConditionalFormattingThreshold {
    match threshold {
        UpstreamHostFixtureConditionalFormattingThreshold::Min => {
            ConditionalFormattingThreshold::Min
        }
        UpstreamHostFixtureConditionalFormattingThreshold::Mid => {
            ConditionalFormattingThreshold::Mid
        }
        UpstreamHostFixtureConditionalFormattingThreshold::Max => {
            ConditionalFormattingThreshold::Max
        }
        UpstreamHostFixtureConditionalFormattingThreshold::Percent { value } => {
            ConditionalFormattingThreshold::Percent(*value)
        }
        UpstreamHostFixtureConditionalFormattingThreshold::Percentile { value } => {
            ConditionalFormattingThreshold::Percentile(*value)
        }
        UpstreamHostFixtureConditionalFormattingThreshold::Number { value } => {
            ConditionalFormattingThreshold::Number(*value)
        }
    }
}

fn to_data_bar_direction(direction: &str) -> DataBarDirection {
    if direction.eq_ignore_ascii_case("right") {
        DataBarDirection::Right
    } else {
        DataBarDirection::Left
    }
}

fn to_rank_rule_options(options: &UpstreamHostFixtureRankRuleOptions) -> RankRuleOptions {
    let rank = match options.kind.as_str() {
        "count" => ConditionalFormattingRank::Count(options.value.max(0.0) as usize),
        "percent" => ConditionalFormattingRank::Percent(options.value),
        _ => ConditionalFormattingRank::Count(0),
    };
    RankRuleOptions { rank }
}

fn to_average_rule_options(options: &UpstreamHostFixtureAverageRuleOptions) -> AverageRuleOptions {
    AverageRuleOptions {
        include_equal: options.include_equal,
        stddev_multiplier: options.stddev_multiplier,
    }
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

        assert_eq!(manifest.cases.len(), 16);

        for entry in &manifest.cases {
            let case_path = repo_root
                .join("docs/test-fixtures/core-engine/upstream-host")
                .join(entry.path.replace('/', "\\"));
            let case = load_case(&case_path).unwrap();
            let execution = execute_fixture_case(&case).unwrap();

            assert_eq!(
                fixture_expectation_mismatches(&case, &execution),
                Vec::<String>::new()
            );

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
