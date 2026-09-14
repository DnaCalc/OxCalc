//! The serialisable wire projection of a grid-backed node's authored truth
//! for the workspace snapshot (`oxcalc.tree.workspace_snapshot.v3`, W063-1 /
//! calc-1q03.1).
//!
//! [`SnapshotGridState`] is what `export_workspace_snapshot` writes per
//! grid-backed sheet node and what `import_workspace_snapshot` rebuilds a
//! [`GridInputState`] from. It carries exactly two things:
//!
//! 1. the **authored truth** ([`SnapshotGridInputState`]) — every field of
//!    [`GridInputState`], byte-faithfully: single authored cells, repeated-
//!    formula regions, merged regions, table overlays and consumer-authored
//!    defined names, plus the grid's coordinate identity and bounds;
//! 2. the **pre-engine pinned publications** ([`SnapshotGridPinnedPublication`])
//!    — the `FileCached` / `Degraded` values of cells the engine does *not*
//!    evaluate (ingest-degraded formulas, non-calc-modeled DataTable/Unknown
//!    cells, external-workbook references). These are not authored truth, but
//!    without them an imported workbook would lose exactly the cells the engine
//!    cannot compute — the class of loss the Measured Excel campaign exists to
//!    make visible, never silent.
//!
//! Everything else in a grid backing (published engine values, epochs, the
//! retained valuation, spill facts, interest windows) is derived state: a pure
//! function of the authored truth plus a recalc, re-derived on import.
//!
//! The wire types are deliberate **mirrors**, not serde derives on the
//! in-memory types: the in-memory address/rect keys are structs (unusable as
//! JSON map keys), `FormulaChannelKind` and `CalcValue` come from sibling
//! crates without serde support, and a persisted format must not move when an
//! in-memory type is renamed. Every conversion is total in the import direction
//! and checked in the export direction ([`SnapshotGridCaptureError`]), so
//! `capture` then `restore` reproduces the [`GridInputState`] exactly — which is
//! what lets import verify each payload against the revision's recorded
//! [`GridInputSnapshotId`](crate::grid::authored::GridInputSnapshotId).

use std::collections::BTreeMap;

use oxfml_core::source::FormulaChannelKind;
use oxfunc_core::value::{CalcValue, CoreValue};
use serde::{Deserialize, Serialize};

use crate::consumer::SnapshotCalcValue;
use crate::grid::authored::{
    GridDefinedNameScope, GridDefinedNameTarget, GridFormulaCell, GridInputCell,
    GridInputDefinedName, GridInputRepeatedRegion, GridInputState,
};
use crate::grid::coords::{ExcelGridBounds, ExcelGridCellAddress};
use crate::grid::geometry::GridRect;
use crate::grid::machine::{GridTableColumn, GridTableOverlay};
use crate::workbook_settings::PublishedValueProvenance;

/// One grid-backed node's persisted state: authored truth plus pre-engine pins.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotGridState {
    pub input: SnapshotGridInputState,
    /// Pinned pre-engine publications (`FileCached` / `Degraded`), address-
    /// ordered. Empty for a plain (non-ingest) grid backing.
    #[serde(default)]
    pub pinned_publications: Vec<SnapshotGridPinnedPublication>,
}

/// Wire form of [`GridInputState`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotGridInputState {
    pub workbook_id: String,
    pub sheet_id: String,
    pub bounds: ExcelGridBounds,
    /// Address-ordered (the in-memory `BTreeMap` order), so the export is
    /// deterministic and a re-import rebuilds the same map.
    pub cells: Vec<SnapshotGridCellRecord>,
    pub repeated_regions: Vec<SnapshotGridRepeatedRegion>,
    pub merged_regions: Vec<SnapshotGridRect>,
    pub table_overlays: Vec<SnapshotGridTableOverlay>,
    pub defined_names: Vec<SnapshotGridDefinedName>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotGridCellRecord {
    pub address: SnapshotGridAddress,
    pub cell: SnapshotGridCell,
}

/// Wire form of [`GridInputCell`]: a formula record carries source text and
/// channel only — never the engine-minted normal-form key (D4 §3 derived-key
/// doctrine); import re-mints it through the engine bind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SnapshotGridCell {
    Literal(SnapshotCalcValue),
    Formula {
        source_text: String,
        source_channel: SnapshotFormulaChannel,
    },
}

/// Wire form of [`ExcelGridCellAddress`] (full coordinate identity, so the
/// round trip is byte-faithful even for an address whose workbook/sheet tokens
/// differ from the owning grid's).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotGridAddress {
    pub workbook_id: String,
    pub sheet_id: String,
    pub row: u32,
    pub col: u32,
}

/// Wire form of [`GridRect`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotGridRect {
    pub workbook_id: String,
    pub sheet_id: String,
    pub top_row: u32,
    pub left_col: u32,
    pub bottom_row: u32,
    pub right_col: u32,
}

/// Wire form of [`FormulaChannelKind`] (a sibling-crate type without serde).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotFormulaChannel {
    WorksheetA1,
    WorksheetR1C1,
    ConditionalFormatting,
    DataValidation,
}

/// Wire form of [`GridInputRepeatedRegion`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotGridRepeatedRegion {
    pub rect: SnapshotGridRect,
    pub source_text: String,
    pub source_channel: SnapshotFormulaChannel,
}

/// Wire form of [`GridTableOverlay`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotGridTableOverlay {
    pub table_id: String,
    pub table_name: String,
    pub table_range: SnapshotGridRect,
    pub header_rect: Option<SnapshotGridRect>,
    pub totals_rect: Option<SnapshotGridRect>,
    pub columns: Vec<SnapshotGridTableColumn>,
}

/// Wire form of [`GridTableColumn`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotGridTableColumn {
    pub column_id: String,
    pub column_name: String,
    pub ordinal: u32,
    pub data_rect: SnapshotGridRect,
}

/// Wire form of [`GridDefinedNameScope`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotGridDefinedNameScope {
    Workbook,
    Sheet(String),
}

/// Wire form of [`GridInputDefinedName`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotGridDefinedName {
    pub scope: SnapshotGridDefinedNameScope,
    pub name: String,
    pub target: SnapshotGridDefinedNameTarget,
}

/// Wire form of [`GridDefinedNameTarget`]. A dynamic name's defining formula
/// is a host-supplied [`GridFormulaCell`] held verbatim in authored truth
/// (key included — `define_dynamic_name` stores what the host bound), so it is
/// persisted verbatim too; re-minting would change the authored identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotGridDefinedNameTarget {
    Static(SnapshotGridRect),
    Dynamic(SnapshotGridFormula),
}

/// Wire form of a full [`GridFormulaCell`] (source text, minted key, channel).
/// Used only where authored truth itself holds a keyed formula cell (dynamic
/// defined names); authored *cells* never persist a key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotGridFormula {
    pub source_text: String,
    pub normal_form_key: String,
    pub source_channel: SnapshotFormulaChannel,
}

/// One pinned pre-engine publication: the cell, its value and its provenance
/// (`FileCached` or `Degraded` — never `Calculated`, which the engine re-derives).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotGridPinnedPublication {
    pub address: SnapshotGridAddress,
    pub value: SnapshotCalcValue,
    pub provenance: PublishedValueProvenance,
}

/// Why a grid backing could not be captured losslessly. Export surfaces this
/// as a typed error rather than writing a payload whose identity would not
/// verify on import (a save that fails is honest; a load that fails is not).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotGridCaptureError {
    /// An authored literal carries content [`SnapshotCalcValue`] cannot
    /// represent exactly: rich metadata, a reference value, or text that is not
    /// well-formed UTF-16.
    LiteralNotRepresentable {
        address: ExcelGridCellAddress,
        detail: String,
    },
    /// A pinned publication's value is not exactly representable (same classes).
    PinNotRepresentable {
        address: ExcelGridCellAddress,
        detail: String,
    },
    /// A pinned publication carries an engine provenance (`Calculated` /
    /// `Stale`), which the pin seam never mints; the backing is inconsistent.
    PinProvenanceNotPinnable {
        address: ExcelGridCellAddress,
        provenance: PublishedValueProvenance,
    },
}

impl SnapshotGridState {
    /// Capture a grid backing's authored truth and pins for the snapshot.
    pub(crate) fn capture(
        input: &GridInputState,
        pins: &BTreeMap<ExcelGridCellAddress, (CalcValue, PublishedValueProvenance)>,
    ) -> Result<Self, SnapshotGridCaptureError> {
        let mut pinned_publications = Vec::with_capacity(pins.len());
        for (address, (value, provenance)) in pins {
            if !matches!(
                provenance,
                PublishedValueProvenance::FileCached | PublishedValueProvenance::Degraded
            ) {
                return Err(SnapshotGridCaptureError::PinProvenanceNotPinnable {
                    address: address.clone(),
                    provenance: *provenance,
                });
            }
            if let Err(detail) = check_value_representable(value) {
                return Err(SnapshotGridCaptureError::PinNotRepresentable {
                    address: address.clone(),
                    detail,
                });
            }
            pinned_publications.push(SnapshotGridPinnedPublication {
                address: SnapshotGridAddress::from_address(address),
                value: SnapshotCalcValue::from_calc_value(value),
                provenance: *provenance,
            });
        }
        Ok(Self {
            input: SnapshotGridInputState::capture(input)?,
            pinned_publications,
        })
    }

    /// Rebuild the pin map for the derived state. Total.
    #[must_use]
    pub(crate) fn restore_pins(
        &self,
    ) -> BTreeMap<ExcelGridCellAddress, (CalcValue, PublishedValueProvenance)> {
        self.pinned_publications
            .iter()
            .map(|pin| {
                (
                    pin.address.to_address(),
                    (pin.value.to_calc_value(), pin.provenance),
                )
            })
            .collect()
    }
}

impl SnapshotGridInputState {
    /// Capture authored truth. Fails (typed) only for a literal
    /// [`SnapshotCalcValue`] cannot carry exactly.
    pub(crate) fn capture(input: &GridInputState) -> Result<Self, SnapshotGridCaptureError> {
        let mut cells = Vec::with_capacity(input.cells.len());
        for (address, cell) in &input.cells {
            let wire = match cell {
                GridInputCell::Literal(value) => {
                    if let Err(detail) = check_value_representable(value) {
                        return Err(SnapshotGridCaptureError::LiteralNotRepresentable {
                            address: address.clone(),
                            detail,
                        });
                    }
                    SnapshotGridCell::Literal(SnapshotCalcValue::from_calc_value(value))
                }
                GridInputCell::Formula {
                    source_text,
                    source_channel,
                } => SnapshotGridCell::Formula {
                    source_text: source_text.clone(),
                    source_channel: SnapshotFormulaChannel::from_channel(*source_channel),
                },
            };
            cells.push(SnapshotGridCellRecord {
                address: SnapshotGridAddress::from_address(address),
                cell: wire,
            });
        }
        Ok(Self {
            workbook_id: input.workbook_id.clone(),
            sheet_id: input.sheet_id.clone(),
            bounds: input.bounds,
            cells,
            repeated_regions: input
                .repeated_regions
                .iter()
                .map(|region| SnapshotGridRepeatedRegion {
                    rect: SnapshotGridRect::from_rect(&region.rect),
                    source_text: region.source_text.clone(),
                    source_channel: SnapshotFormulaChannel::from_channel(region.source_channel),
                })
                .collect(),
            merged_regions: input
                .merged_regions
                .iter()
                .map(SnapshotGridRect::from_rect)
                .collect(),
            table_overlays: input
                .table_overlays
                .iter()
                .map(SnapshotGridTableOverlay::from_overlay)
                .collect(),
            defined_names: input
                .defined_names
                .iter()
                .map(SnapshotGridDefinedName::from_name)
                .collect(),
        })
    }

    /// Rebuild the in-memory authored truth. Total: every wire variant maps to
    /// exactly one in-memory value. A later cell record for the same address
    /// wins (the map is address-keyed), matching the engine's overwrite rule —
    /// an export never emits duplicates, so this only matters for hand-edited
    /// documents.
    #[must_use]
    pub(crate) fn restore(&self) -> GridInputState {
        let mut input =
            GridInputState::new(self.workbook_id.clone(), self.sheet_id.clone(), self.bounds);
        for record in &self.cells {
            let cell = match &record.cell {
                SnapshotGridCell::Literal(value) => GridInputCell::Literal(value.to_calc_value()),
                SnapshotGridCell::Formula {
                    source_text,
                    source_channel,
                } => GridInputCell::Formula {
                    source_text: source_text.clone(),
                    source_channel: source_channel.to_channel(),
                },
            };
            input.cells.insert(record.address.to_address(), cell);
        }
        input.repeated_regions = self
            .repeated_regions
            .iter()
            .map(|region| GridInputRepeatedRegion {
                rect: region.rect.to_rect(),
                source_text: region.source_text.clone(),
                source_channel: region.source_channel.to_channel(),
            })
            .collect();
        input.merged_regions = self
            .merged_regions
            .iter()
            .map(SnapshotGridRect::to_rect)
            .collect();
        input.table_overlays = self
            .table_overlays
            .iter()
            .map(SnapshotGridTableOverlay::to_overlay)
            .collect();
        input.defined_names = self
            .defined_names
            .iter()
            .map(SnapshotGridDefinedName::to_name)
            .collect();
        input
    }
}

impl SnapshotGridAddress {
    fn from_address(address: &ExcelGridCellAddress) -> Self {
        Self {
            workbook_id: address.workbook_id.clone(),
            sheet_id: address.sheet_id.clone(),
            row: address.row,
            col: address.col,
        }
    }

    fn to_address(&self) -> ExcelGridCellAddress {
        ExcelGridCellAddress {
            workbook_id: self.workbook_id.clone(),
            sheet_id: self.sheet_id.clone(),
            row: self.row,
            col: self.col,
        }
    }
}

impl SnapshotGridRect {
    fn from_rect(rect: &GridRect) -> Self {
        Self {
            workbook_id: rect.workbook_id.clone(),
            sheet_id: rect.sheet_id.clone(),
            top_row: rect.top_row,
            left_col: rect.left_col,
            bottom_row: rect.bottom_row,
            right_col: rect.right_col,
        }
    }

    /// Rebuilds the rect field-for-field (no bounds re-validation here: the
    /// rect was validated when authored, and the import's sheet rebuild
    /// re-checks every region against the grid bounds).
    fn to_rect(&self) -> GridRect {
        GridRect {
            workbook_id: self.workbook_id.clone(),
            sheet_id: self.sheet_id.clone(),
            top_row: self.top_row,
            left_col: self.left_col,
            bottom_row: self.bottom_row,
            right_col: self.right_col,
        }
    }
}

impl SnapshotFormulaChannel {
    fn from_channel(channel: FormulaChannelKind) -> Self {
        match channel {
            FormulaChannelKind::WorksheetA1 => Self::WorksheetA1,
            FormulaChannelKind::WorksheetR1C1 => Self::WorksheetR1C1,
            FormulaChannelKind::ConditionalFormatting => Self::ConditionalFormatting,
            FormulaChannelKind::DataValidation => Self::DataValidation,
        }
    }

    fn to_channel(self) -> FormulaChannelKind {
        match self {
            Self::WorksheetA1 => FormulaChannelKind::WorksheetA1,
            Self::WorksheetR1C1 => FormulaChannelKind::WorksheetR1C1,
            Self::ConditionalFormatting => FormulaChannelKind::ConditionalFormatting,
            Self::DataValidation => FormulaChannelKind::DataValidation,
        }
    }
}

impl SnapshotGridTableOverlay {
    fn from_overlay(overlay: &GridTableOverlay) -> Self {
        Self {
            table_id: overlay.table_id.clone(),
            table_name: overlay.table_name.clone(),
            table_range: SnapshotGridRect::from_rect(&overlay.table_range),
            header_rect: overlay
                .header_rect
                .as_ref()
                .map(SnapshotGridRect::from_rect),
            totals_rect: overlay
                .totals_rect
                .as_ref()
                .map(SnapshotGridRect::from_rect),
            columns: overlay
                .columns
                .iter()
                .map(|column| SnapshotGridTableColumn {
                    column_id: column.column_id.clone(),
                    column_name: column.column_name.clone(),
                    ordinal: column.ordinal,
                    data_rect: SnapshotGridRect::from_rect(&column.data_rect),
                })
                .collect(),
        }
    }

    fn to_overlay(&self) -> GridTableOverlay {
        GridTableOverlay {
            table_id: self.table_id.clone(),
            table_name: self.table_name.clone(),
            table_range: self.table_range.to_rect(),
            header_rect: self.header_rect.as_ref().map(SnapshotGridRect::to_rect),
            totals_rect: self.totals_rect.as_ref().map(SnapshotGridRect::to_rect),
            columns: self
                .columns
                .iter()
                .map(|column| GridTableColumn {
                    column_id: column.column_id.clone(),
                    column_name: column.column_name.clone(),
                    ordinal: column.ordinal,
                    data_rect: column.data_rect.to_rect(),
                })
                .collect(),
        }
    }
}

impl SnapshotGridDefinedName {
    fn from_name(name: &GridInputDefinedName) -> Self {
        Self {
            scope: match &name.scope {
                GridDefinedNameScope::Workbook => SnapshotGridDefinedNameScope::Workbook,
                GridDefinedNameScope::Sheet(sheet_id) => {
                    SnapshotGridDefinedNameScope::Sheet(sheet_id.clone())
                }
            },
            name: name.name.clone(),
            target: match &name.target {
                GridDefinedNameTarget::Static(rect) => {
                    SnapshotGridDefinedNameTarget::Static(SnapshotGridRect::from_rect(rect))
                }
                GridDefinedNameTarget::Dynamic(formula) => {
                    SnapshotGridDefinedNameTarget::Dynamic(SnapshotGridFormula {
                        source_text: formula.source_text.clone(),
                        normal_form_key: formula.normal_form_key.clone(),
                        source_channel: SnapshotFormulaChannel::from_channel(
                            formula.source_channel,
                        ),
                    })
                }
            },
        }
    }

    fn to_name(&self) -> GridInputDefinedName {
        GridInputDefinedName {
            scope: match &self.scope {
                SnapshotGridDefinedNameScope::Workbook => GridDefinedNameScope::Workbook,
                SnapshotGridDefinedNameScope::Sheet(sheet_id) => {
                    GridDefinedNameScope::Sheet(sheet_id.clone())
                }
            },
            name: self.name.clone(),
            target: match &self.target {
                SnapshotGridDefinedNameTarget::Static(rect) => {
                    GridDefinedNameTarget::Static(rect.to_rect())
                }
                SnapshotGridDefinedNameTarget::Dynamic(formula) => GridDefinedNameTarget::Dynamic(
                    GridFormulaCell::new(
                        formula.source_text.clone(),
                        formula.normal_form_key.clone(),
                    )
                    .with_source_channel(formula.source_channel.to_channel()),
                ),
            },
        }
    }
}

/// Whether [`SnapshotCalcValue::from_calc_value`] then
/// [`SnapshotCalcValue::to_calc_value`] reproduces `value` exactly. Rich
/// metadata is dropped by the wire form; a reference has no wire form; text
/// with lone surrogates does not survive the UTF-16 → `String` → UTF-16 trip.
/// Arrays are checked element-wise.
fn check_value_representable(value: &CalcValue) -> Result<(), String> {
    if value.rich.is_some() {
        return Err("value carries rich metadata the snapshot wire form drops".to_string());
    }
    match &value.core {
        CoreValue::Number(_)
        | CoreValue::Logical(_)
        | CoreValue::Error(_)
        | CoreValue::Empty
        | CoreValue::Missing => Ok(()),
        CoreValue::Text(text) => {
            if String::from_utf16(text.utf16_code_units()).is_ok() {
                Ok(())
            } else {
                Err("text is not well-formed UTF-16 (lone surrogate)".to_string())
            }
        }
        CoreValue::Array(array) => array
            .iter_row_major()
            .try_for_each(check_value_representable),
        CoreValue::Reference(_) => {
            Err("a reference value has no exact snapshot wire form".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::authored::GridInputCell;
    use oxfunc_core::value::{ExcelText, WorksheetErrorCode};

    fn bounds() -> ExcelGridBounds {
        ExcelGridBounds::strict_excel()
    }

    fn rect(top_row: u32, left_col: u32, bottom_row: u32, right_col: u32) -> GridRect {
        GridRect::new(
            "book:s",
            "Sheet1",
            top_row,
            left_col,
            bottom_row,
            right_col,
            bounds(),
        )
        .unwrap()
    }

    fn address(row: u32, col: u32) -> ExcelGridCellAddress {
        ExcelGridCellAddress::new("book:s", "Sheet1", row, col)
    }

    /// Every field of `GridInputState` survives capture → restore exactly, and
    /// the content identity (what import verifies against the revision) is
    /// unchanged by the trip.
    #[test]
    fn grid_snapshot_capture_restore_reproduces_input_state_and_identity() {
        let mut input = GridInputState::new("book:s", "Sheet1", bounds());
        input.cells.insert(
            address(1, 1),
            GridInputCell::Literal(CalcValue::number(7.0)),
        );
        input.cells.insert(
            address(1, 2),
            GridInputCell::Literal(CalcValue::text(ExcelText::from_interop_assignment(
                "héllo ✓",
            ))),
        );
        input.cells.insert(
            address(1, 3),
            GridInputCell::Literal(CalcValue::logical(true)),
        );
        input.cells.insert(
            address(1, 4),
            GridInputCell::Literal(CalcValue::error(WorksheetErrorCode::NA)),
        );
        input.cells.insert(
            address(2, 1),
            GridInputCell::Formula {
                source_text: "=A1*3".to_string(),
                source_channel: FormulaChannelKind::WorksheetA1,
            },
        );
        input.repeated_regions.push(GridInputRepeatedRegion {
            rect: rect(3, 1, 5, 1),
            source_text: "=R[-1]C+1".to_string(),
            source_channel: FormulaChannelKind::WorksheetR1C1,
        });
        input.merged_regions.push(rect(7, 1, 7, 3));
        input.table_overlays.push(
            GridTableOverlay::new(
                "table:t1",
                "T1",
                rect(10, 1, 12, 2),
                vec![GridTableColumn::new("col:a", "A", 0, rect(11, 1, 12, 1))],
            )
            .with_header_rect(rect(10, 1, 10, 2)),
        );
        input.defined_names.push(GridInputDefinedName {
            scope: GridDefinedNameScope::Workbook,
            name: "Rate".to_string(),
            target: GridDefinedNameTarget::Static(rect(1, 1, 1, 1)),
        });
        input.defined_names.push(GridInputDefinedName {
            scope: GridDefinedNameScope::Sheet("Sheet1".to_string()),
            name: "Dyn".to_string(),
            target: GridDefinedNameTarget::Dynamic(
                GridFormulaCell::new("=OFFSET(A1,0,0,2,1)", "host-key:dyn")
                    .with_source_channel(FormulaChannelKind::WorksheetA1),
            ),
        });

        let wire = SnapshotGridInputState::capture(&input).unwrap();
        let restored = wire.restore();
        assert_eq!(restored, input);
        assert_eq!(restored.identity(), input.identity());

        // And the wire form itself round-trips through JSON unchanged.
        let json = serde_json::to_string(&wire).unwrap();
        let reparsed: SnapshotGridInputState = serde_json::from_str(&json).unwrap();
        assert_eq!(reparsed, wire);
        assert_eq!(reparsed.restore().identity(), input.identity());
    }

    #[test]
    fn grid_snapshot_pins_round_trip_with_provenance() {
        let input = GridInputState::new("book:s", "Sheet1", bounds());
        let mut pins = BTreeMap::new();
        pins.insert(
            address(4, 4),
            (
                CalcValue::number(42.0),
                PublishedValueProvenance::FileCached,
            ),
        );
        pins.insert(
            address(5, 5),
            (
                CalcValue::error(WorksheetErrorCode::Name),
                PublishedValueProvenance::Degraded,
            ),
        );
        let wire = SnapshotGridState::capture(&input, &pins).unwrap();
        assert_eq!(wire.pinned_publications.len(), 2);
        assert_eq!(wire.restore_pins(), pins);
    }

    #[test]
    fn grid_snapshot_capture_rejects_engine_provenance_pins() {
        let input = GridInputState::new("book:s", "Sheet1", bounds());
        let mut pins = BTreeMap::new();
        pins.insert(
            address(1, 1),
            (
                CalcValue::number(1.0),
                PublishedValueProvenance::Calculated { tick_id: 9 },
            ),
        );
        let err = SnapshotGridState::capture(&input, &pins).unwrap_err();
        assert!(matches!(
            err,
            SnapshotGridCaptureError::PinProvenanceNotPinnable { .. }
        ));
    }

    #[test]
    fn grid_snapshot_capture_rejects_rich_literal_rather_than_dropping_it() {
        use oxfunc_core::value::PresentationHint;
        let mut input = GridInputState::new("book:s", "Sheet1", bounds());
        let rich = CalcValue::with_presentation(
            CoreValue::Number(1.0),
            PresentationHint {
                number_format: None,
                style: None,
            },
        );
        input
            .cells
            .insert(address(1, 1), GridInputCell::Literal(rich));
        let err = SnapshotGridInputState::capture(&input).unwrap_err();
        assert!(matches!(
            err,
            SnapshotGridCaptureError::LiteralNotRepresentable { .. }
        ));
    }
}
