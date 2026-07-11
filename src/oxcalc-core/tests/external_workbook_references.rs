#![forbid(unsafe_code)]

//! W062 R3.7 (`calc-5kqg.26`) — external workbook references, two-workspace
//! acceptance (D2 §5).
//!
//! These tests exercise the external-workbook reference **routing seam** across
//! two independently-computed sibling grid workspaces:
//!
//! - Workbook B (`[Book2]`) is a real [`GridCalcRefSheet`] that computes its own
//!   `Sheet1!A1` value.
//! - Workbook A holds a formula that references `[Book2]Sheet1!A1`. The external
//!   atom's dormant-external identity (`extbook:book2`, D2 §10) is routed
//!   through the shared context-level [`WorkspaceAliasCatalog`] by
//!   [`route_external_workbook`], and the sibling's live value is pulled by
//!   [`gather_external_cells`].
//!
//! Acceptance (bead):
//! - `[Book2]Sheet1!A1` is LIVE when Book2 is loaded+aliased (routes to B's
//!   workspace, gathers B's actual computed value).
//! - `[Book2]Sheet1!A1` is a TYPED `#REF!` when Book2 is absent (never silent).
//! - Heals on load: registering the alias flips the same reference from typed
//!   `#REF!` to live, with no key change.
//!
//! # Scope boundary (honest pin, D2 §5 typed exclusions)
//!
//! This test drives the reference-layer routing + value-gather directly against
//! two real sibling grids. The **final consumption of the `External` atom by
//! the evaluating grid's formula machine** — i.e. making `=[Book2]Sheet1!A1*2`
//! flow the gathered value into cell evaluation — is cross-workbook coordinator
//! wiring owned by the R4.10 consumer lane (`consumer.rs` / `machine.rs` /
//! `workbook_coordinator.rs`), not the reference layer. Value-on-recalc
//! *correctness at the routing seam* is proven here; automatic cross-WORKBOOK
//! dirty propagation A←B is the R4 coordination lane's job, pinned there. No
//! file loading, no external-value cache, and no link manager are built (the D2
//! §5 exclusions) — routing reads the already-loaded alias catalog only.

use std::collections::BTreeMap;

use oxcalc_core::grid::authored::GridFormulaCell;
use oxcalc_core::grid::coords::{ExcelGridBounds, ExcelGridCellAddress};
use oxcalc_core::grid::machine::{GridCalcRefSheet, GridDependency};
use oxcalc_core::reference_vocabulary::ExternalBookToken;
use oxcalc_core::workbook_reference_catalog::{
    EXTERNAL_WORKBOOK_NOT_LOADED_DIAGNOSTIC, ExternalWorkbookRouting, WorkspaceAliasCatalog,
    gather_external_cells, route_external_workbook,
};
use oxfml_core::source::FormulaChannelKind;
use oxfunc_core::value::CalcValue;

/// Build sibling workbook B and compute its `Sheet1!A1 = A2 + A3` so the value
/// is genuinely *derived* (not a planted literal), then return B's published
/// computed store keyed by address. This is the live sibling's own state.
fn sibling_book2_computed() -> (
    ExcelGridCellAddress,
    BTreeMap<ExcelGridCellAddress, CalcValue>,
) {
    let bounds = ExcelGridBounds::strict_excel();
    let a1 = ExcelGridCellAddress::new("book:b2", "Sheet1", 1, 1);
    let a2 = ExcelGridCellAddress::new("book:b2", "Sheet1", 2, 1);
    let a3 = ExcelGridCellAddress::new("book:b2", "Sheet1", 3, 1);

    let mut b_sheet1 = GridCalcRefSheet::new("book:b2", "Sheet1", bounds);
    b_sheet1
        .set_literal(a2.clone(), CalcValue::number(40.0))
        .unwrap();
    b_sheet1
        .set_literal(a3.clone(), CalcValue::number(2.0))
        .unwrap();
    b_sheet1
        .set_formula(
            a1.clone(),
            GridFormulaCell::new("=A2+A3", "excel.grid.v1:cell:Sheet1:R2C1")
                .with_source_channel(FormulaChannelKind::WorksheetA1),
        )
        .unwrap();
    b_sheet1.recalculate_mark_all_dirty_with_oxfml().unwrap();

    // Sanity: B computed its own A1 = 42 live.
    assert_eq!(b_sheet1.read_cell(&a1), CalcValue::number(42.0));

    (a1, b_sheet1.computed().clone())
}

#[test]
fn external_reference_is_live_when_sibling_workbook_is_loaded_and_aliased() {
    let (b2_a1, b2_computed) = sibling_book2_computed();

    // Context-level shared alias catalog with Book2 registered+loaded (the
    // R3.6 alias verbs populate this; here we register it directly).
    let mut aliases = WorkspaceAliasCatalog::new();
    aliases.register_alias("Book2", "workspace:book2");

    // Workbook A's formula references [Book2]Sheet1!A1. Its dormant-external
    // identity token is the `{workbook}` component a bound external record
    // carries (§10). The dependency A holds points at B's A1 address.
    let external_component = ExternalBookToken::from_alias("Book2");
    assert_eq!(external_component.as_str(), "extbook:book2");
    let external_dependency = GridDependency::Cell(b2_a1.clone());

    // Route the external reference through the shared alias catalog: Book2 is
    // loaded, so it routes LIVE to B's workspace.
    let routing = route_external_workbook(&aliases, external_component.as_str())
        .expect("an external component routes");
    let ExternalWorkbookRouting::Routed { workspace_id, .. } = &routing else {
        panic!("Book2 is loaded — must route live, got {routing:?}");
    };
    assert_eq!(workspace_id, "workspace:book2");

    // Gather the sibling's LIVE value for the referenced cell — B's actual
    // computed A1 (42), not a cache or a fabricated value.
    let gathered = gather_external_cells(&[external_dependency], &b2_computed);
    assert_eq!(
        gathered.get(&b2_a1),
        Some(&CalcValue::number(42.0)),
        "external reference reads Book2's live Sheet1!A1 = 42"
    );
}

#[test]
fn external_reference_is_typed_ref_error_when_sibling_workbook_is_absent() {
    let (b2_a1, _b2_computed) = sibling_book2_computed();

    // No alias registered — Book2 is not loaded in this context.
    let aliases = WorkspaceAliasCatalog::new();
    let external_component = ExternalBookToken::from_alias("Book2");

    let routing = route_external_workbook(&aliases, external_component.as_str())
        .expect("an external component routes");
    // A TYPED #REF! carrying the stable diagnostic — never a silent empty.
    let ExternalWorkbookRouting::RefError { diagnostic, .. } = &routing else {
        panic!("absent Book2 must be a typed #REF!, got {routing:?}");
    };
    assert_eq!(*diagnostic, EXTERNAL_WORKBOOK_NOT_LOADED_DIAGNOSTIC);
    assert_eq!(*diagnostic, "excel.grid.external.workbook_not_loaded");
    assert!(routing.is_ref_error());
    assert_eq!(routing.workspace_id(), None);

    // Because it is a typed #REF! (not a value), no sibling value is gathered.
    // Even if a caller mistakenly gathered against an empty store, the absent
    // cell is not fabricated.
    let gathered = gather_external_cells(&[GridDependency::Cell(b2_a1.clone())], &BTreeMap::new());
    assert!(gathered.is_empty());
}

#[test]
fn external_reference_heals_on_sibling_load_without_key_change() {
    let (b2_a1, b2_computed) = sibling_book2_computed();
    let external_component = ExternalBookToken::from_alias("Book2");

    // Start absent: typed #REF!.
    let mut aliases = WorkspaceAliasCatalog::new();
    let before = route_external_workbook(&aliases, external_component.as_str()).unwrap();
    assert!(
        before.is_ref_error(),
        "starts as typed #REF! before B loads"
    );

    // Load B under the SAME alias — routing flips to live. The external
    // component (the key's `{workbook}` token) is byte-for-byte unchanged; only
    // the alias catalog state changed. That is heal-on-load (§10): no rebind
    // storm, the key is stable.
    aliases.register_alias("Book2", "workspace:book2");
    let after = route_external_workbook(&aliases, external_component.as_str()).unwrap();
    assert_eq!(after.workspace_id(), Some("workspace:book2"));

    let gathered = gather_external_cells(&[GridDependency::Cell(b2_a1.clone())], &b2_computed);
    assert_eq!(gathered.get(&b2_a1), Some(&CalcValue::number(42.0)));

    // Unload again (workspace closed): back to typed #REF!, same key.
    aliases.unregister_alias("Book2");
    let unloaded = route_external_workbook(&aliases, external_component.as_str()).unwrap();
    assert!(unloaded.is_ref_error(), "unload returns to typed #REF!");
}
