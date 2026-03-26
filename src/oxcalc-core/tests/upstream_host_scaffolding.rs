#![forbid(unsafe_code)]

use oxcalc_core::upstream_host::{
    MinimalFormulaSlotFacts, MinimalHostInfoMode, MinimalLocaleContextKind, MinimalRtdMode,
    MinimalRuntimeCatalogFacts, MinimalTypedQueryFacts, MinimalUpstreamHostPacket,
    UpstreamHostAnchor,
};
use oxcalc_core::upstream_host_fixture;
use oxfml_core::interface::{
    HostProviderOutcomeKind, LibraryContextSnapshotRef, ReturnedValueSurfaceKind,
};
use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use oxfml_core::source::FormulaChannelKind;
use oxfml_core::EvaluationBackend;
use oxfunc_core::value::{EvalValue, WorksheetErrorCode};
use std::path::PathBuf;

fn packet(formula_text: &str) -> MinimalUpstreamHostPacket {
    MinimalUpstreamHostPacket {
        formula_slot: MinimalFormulaSlotFacts {
            fixture_input_id: "fixture:host:integration".to_string(),
            formula_slot_id: Some("node:slot:integration".to_string()),
            formula_stable_id: "formula:host:integration".to_string(),
            formula_text: formula_text.to_string(),
            formula_text_version: 1,
            formula_channel_kind: FormulaChannelKind::WorksheetA1,
            caller_anchor: UpstreamHostAnchor { row: 2, col: 2 },
            active_selection_anchor: Some(UpstreamHostAnchor { row: 2, col: 2 }),
            structure_context_version: "treecalc.struct:integration:v1".to_string(),
        },
        binding_world: Default::default(),
        typed_query_facts: Default::default(),
        runtime_catalog: Default::default(),
    }
}

fn snapshot_with_entry(surface_name: &str) -> LibraryContextSnapshot {
    LibraryContextSnapshot {
        snapshot_id: "snapshot:integration".to_string(),
        snapshot_version: "v1".to_string(),
        entries: vec![LibraryContextSnapshotEntry {
            surface_name: surface_name.to_string(),
            canonical_id: Some(format!("FUNC.{surface_name}")),
            surface_stable_id: Some(format!("surface:{surface_name}")),
            name_resolution_table_ref: Some("name-table:integration:v1".to_string()),
            semantic_trait_profile_ref: Some("traits:integration:v1".to_string()),
            gating_profile_ref: Some("gating:integration:v1".to_string()),
            metadata_status: Some("runtime".to_string()),
            special_interface_kind: None,
            admission_interface_kind: Some("ordinary".to_string()),
            preparation_owner: Some("oxfunc".to_string()),
            runtime_boundary_kind: Some("host_query".to_string()),
            arity_shape_note: None,
            interface_contract_ref: Some("iface:integration:v1".to_string()),
            registration_source_kind: RegistrationSourceKind::BuiltIn,
            parse_bind_state: LibraryAvailabilityState::CatalogKnown,
            semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
            runtime_capability_state: Some(LibraryAvailabilityState::CatalogKnown),
            post_dispatch_state: Some(LibraryAvailabilityState::CatalogKnown),
        }],
    }
}

#[test]
fn scaffolding_packet_drives_capture_from_runtime_catalog_snapshot() {
    let mut packet = packet("=INFO(\"directory\")");
    packet.typed_query_facts = MinimalTypedQueryFacts {
        host_info_mode: MinimalHostInfoMode::DirectoryValueAndFilenameProviderFailure {
            value: "C:\\Scaffold".to_string(),
            detail: "filename_unavailable".to_string(),
        },
        locale_context_kind: MinimalLocaleContextKind::EnUs,
        ..Default::default()
    };
    packet.runtime_catalog = MinimalRuntimeCatalogFacts {
        library_context_snapshot: Some(snapshot_with_entry("INFO")),
    };

    let (output, capture_packet) = packet
        .recalc_with_capture_packet(EvaluationBackend::OxFuncBacked)
        .unwrap();

    assert_eq!(
        output.returned_value_surface.kind,
        ReturnedValueSurfaceKind::OrdinaryValue
    );
    assert_eq!(
        capture_packet.library_context_snapshot_ref,
        Some(LibraryContextSnapshotRef::new("snapshot:integration", "v1"))
    );
    assert_eq!(capture_packet.formula_stable_id, "formula:host:integration");
}

#[test]
fn scaffolding_packet_exposes_typed_host_provider_outcomes() {
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

#[test]
fn scaffolding_packet_exposes_rtd_provider_surfaces() {
    let mut packet = packet("=RTD(\"prog\",\"server\",\"topic\")");
    packet.typed_query_facts.rtd_mode = MinimalRtdMode::ProviderError {
        code: WorksheetErrorCode::Value,
    };
    packet.runtime_catalog = MinimalRuntimeCatalogFacts {
        library_context_snapshot: Some(snapshot_with_entry("RTD")),
    };

    let output = packet.recalc(EvaluationBackend::OxFuncBacked).unwrap();

    assert_eq!(
        output.returned_value_surface.kind,
        ReturnedValueSurfaceKind::TypedHostProviderOutcome
    );
    assert_eq!(
        output
            .returned_value_surface
            .host_provider_outcome
            .unwrap()
            .worksheet_error,
        Some(WorksheetErrorCode::Value)
    );
    assert_eq!(
        output.returned_value_surface.payload_summary,
        "ProviderError(Value)"
    );
}

#[test]
fn scaffolding_packet_preserves_public_bindings_surface() {
    let mut packet = packet("=SUM(InputValue, 2)");
    packet
        .binding_world
        .cell_fixture
        .insert("A1".to_string(), EvalValue::Number(3.0));
    packet.binding_world.defined_name_bindings.insert(
        "InputValue".to_string(),
        oxcalc_core::upstream_host::UpstreamDefinedNameBinding::Value(EvalValue::Number(5.0)),
    );

    let bind_context = packet.build_bind_context();

    assert_eq!(bind_context.caller_row, 2);
    assert_eq!(bind_context.caller_col, 2);
    assert_eq!(
        bind_context.names.get("InputValue"),
        Some(&oxfml_core::binding::NameKind::ValueLike)
    );
}

#[test]
fn public_fixture_loader_executes_checked_in_upstream_host_corpus() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let manifest_path =
        repo_root.join("docs/test-fixtures/core-engine/upstream-host/MANIFEST.json");
    let manifest = upstream_host_fixture::load_manifest(&manifest_path).unwrap();

    assert_eq!(manifest.cases.len(), 6);

    for entry in &manifest.cases {
        let case_path = repo_root
            .join("docs/test-fixtures/core-engine/upstream-host")
            .join(entry.path.replace('/', "\\"));
        let case = upstream_host_fixture::load_case(&case_path).unwrap();
        let execution = upstream_host_fixture::execute_fixture_case(&case).unwrap();

        assert_eq!(
            execution.packet.formula_slot.formula_stable_id,
            case.formula_slot.formula_stable_id
        );
    }
}
