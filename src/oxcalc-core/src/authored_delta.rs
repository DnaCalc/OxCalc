#![forbid(unsafe_code)]

//! Authored delta: the neutral "edits since revision R" output (W062 R5.7,
//! D4 §7b).
//!
//! [`workbook_authored_delta`](crate::consumer::OxCalcDocumentContext::workbook_authored_delta)
//! diffs the authored truth of two retained workspace revisions and returns a
//! typed [`WorkbookAuthoredDelta`]: the cells set/cleared/changed, the sheets
//! added/deleted/renamed/moved, the per-sheet name/table/merge/region lifecycle
//! changes, and the workbook-settings changes.
//!
//! # Input-snapshot-only discipline (D1 C6, D4 §7b verbatim)
//!
//! Every field of the delta is computed **exclusively** from authored input
//! snapshots — the structural snapshot (sheet lifecycle, settings meta node
//! inputs) and the per-sheet [`GridInputState`] maps (cells, regions, merges,
//! table overlays, defined names). Derived state — published values, engine
//! sheets, dependency shapes, overlay projections — never enters the diff. The
//! diff function's inputs ([`AuthoredRevisionInputs`]) carry authored truth
//! only; there is no path from here to a `PublishedRuntimeLayerPayload`, a
//! `PublicationSnapshot`, or any engine sheet. This is what lets the delta be a
//! sound basis for a future granular save path (D4 §7b): it carries authored
//! edits, not computed caches. A consumer that wants fresh caches reads the
//! published readout instead (as §7a does).
//!
//! # Fast path (D1 C5 / §7.3)
//!
//! Per-sheet grid diffing is short-circuited by [`GridInputSnapshotId`]
//! equality: a sheet whose content address is unchanged between the two
//! revisions contributes no cell/region/merge/table/name rows and is not walked
//! cell-by-cell. Because the retained `grid_inputs` share one
//! `Arc<GridInputState>` per sheet across a retention window (D1 §7.3), an
//! unedited sheet's id is trivially equal, so a 100-sheet workbook with one
//! edited sheet diffs exactly one sheet's cells.

use std::collections::{BTreeMap, BTreeSet};

use crate::grid::authored::{
    GridInputCell, GridInputDefinedName, GridInputRepeatedRegion, GridInputSnapshotId,
    GridInputState,
};
use crate::grid::coords::ExcelGridCellAddress;
use crate::grid::geometry::GridRect;
use crate::grid::machine::GridTableOverlay;
use crate::structural::{NormalizedSheetName, StructuralSnapshot, TreeNodeId};
use crate::workbook_settings::{WorkbookCalcSettings, WorkbookSettingChanged};
use crate::workspace_revision::{NodeInputSnapshot, WorkspaceRevisionId};

/// A single authored cell edit between two revisions (D4 §7b: "per-sheet
/// cell-input diffs (set/cleared, literal vs formula-text)").
///
/// `old`/`new` carry the authored [`GridInputCell`] records verbatim (literal
/// value or formula source-text + channel) — never the derived normal-form key,
/// never a published value. The variant classifies the transition.
#[derive(Debug, Clone, PartialEq)]
pub struct CellInputDelta {
    /// The sheet the cell lives on (its stable node id).
    pub sheet_node_id: TreeNodeId,
    /// The cell address within that sheet.
    pub address: ExcelGridCellAddress,
    /// The transition kind.
    pub change: CellInputChange,
}

/// How a cell's authored record changed (D4 §7b set/cleared, literal vs
/// formula).
#[derive(Debug, Clone, PartialEq)]
pub enum CellInputChange {
    /// The address held no authored record in `since` and holds one now.
    Set { new: GridInputCell },
    /// The address held an authored record in `since` and holds none now.
    Cleared { old: GridInputCell },
    /// The address held an authored record in both revisions and it changed
    /// (a literal edited, a formula text edited, or a literal↔formula flip).
    Changed {
        old: GridInputCell,
        new: GridInputCell,
    },
}

/// A `set`/`cleared` lifecycle diff of a per-sheet authored collection whose
/// members are compared by value in authoring order (repeated-formula regions,
/// merged regions, table overlays, defined names). Members present only in the
/// new revision are `added`; members present only in the old revision are
/// `removed`. Order-only churn with an unchanged member multiset yields empty
/// `added`/`removed` (the authored content is the same).
#[derive(Debug, Clone, PartialEq)]
pub struct CollectionDelta<T> {
    pub added: Vec<T>,
    pub removed: Vec<T>,
}

impl<T> CollectionDelta<T> {
    #[must_use]
    fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty()
    }
}

/// The per-sheet authored-collection lifecycle diffs for one sheet that exists
/// in both revisions (D4 §7b: "name/table/merge lifecycle diffs"). Only sheets
/// with at least one non-cell collection change appear; a sheet with only cell
/// edits contributes to [`WorkbookAuthoredDelta::cells`] but not here.
#[derive(Debug, Clone, PartialEq)]
pub struct SheetCollectionsDelta {
    pub sheet_node_id: TreeNodeId,
    pub repeated_regions: CollectionDelta<GridInputRepeatedRegion>,
    pub merged_regions: CollectionDelta<GridRect>,
    pub table_overlays: CollectionDelta<GridTableOverlay>,
    pub defined_names: CollectionDelta<GridInputDefinedName>,
}

/// A sheet lifecycle change between two revisions (D4 §7b: "sheet lifecycle
/// (add/delete/rename/reorder from structural + tombstone facts)").
///
/// Computed by diffing the two structural snapshots' Sheet-role node sets and
/// positions — the authored structure, never a derived enumeration.
#[derive(Debug, Clone, PartialEq)]
pub enum SheetLifecycleDelta {
    /// A Sheet-role node present now but not in `since`.
    Added {
        node_id: TreeNodeId,
        display_name: String,
        normalized_name: NormalizedSheetName,
        /// Position in the new revision's sheet order.
        sheet_position: usize,
    },
    /// A Sheet-role node present in `since` but not now. The display/normalized
    /// name is read from `since`'s snapshot (where the node still exists), so
    /// the delta is self-sufficient without consulting a tombstone list.
    Deleted {
        node_id: TreeNodeId,
        display_name: String,
        normalized_name: NormalizedSheetName,
        /// Position in `since`'s sheet order.
        sheet_position: usize,
    },
    /// A sheet whose node id survived but whose normalized name changed
    /// (a rename). The node id is the rename-stable identity (D1 C2).
    Renamed {
        node_id: TreeNodeId,
        old_normalized: NormalizedSheetName,
        new_normalized: NormalizedSheetName,
        old_display: String,
        new_display: String,
    },
    /// A sheet whose node id survived and whose order position changed
    /// (a reorder / move). Positions are the dense Sheet-role order indices.
    ///
    /// This is a faithful per-sheet position diff, not a synthesized "primary
    /// mover": moving one sheet to the front shifts every other sheet's index,
    /// so each shifted sheet reports its own `Moved` row. Two endpoint snapshots
    /// carry no information about *which* single edit produced the reordering,
    /// so the delta reports the observable per-sheet truth.
    Moved {
        node_id: TreeNodeId,
        old_sheet_position: usize,
        new_sheet_position: usize,
    },
}

/// The neutral typed "edits since revision R" output (W062 R5.7, D4 §7b).
///
/// Carries authored edits only. Empty vectors everywhere means the two
/// revisions have identical authored truth.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WorkbookAuthoredDelta {
    /// Per-cell authored edits, ordered by `(sheet_node_id, address)`.
    pub cells: Vec<CellInputDelta>,
    /// Per-sheet name/table/merge/region lifecycle diffs, ordered by
    /// `sheet_node_id`. Only sheets present in both revisions with a non-cell
    /// change appear.
    pub sheet_collections: Vec<SheetCollectionsDelta>,
    /// Sheet lifecycle changes (add/delete/rename/reorder), in a deterministic
    /// order (deletes, adds, renames, moves; each ordered by node id).
    pub sheets: Vec<SheetLifecycleDelta>,
    /// Workbook calc-settings changes, reusing the shared typed old/new change
    /// enum (D1 §5 / C4). Diffed from the settings meta node inputs.
    pub settings: Vec<WorkbookSettingChanged>,
}

impl WorkbookAuthoredDelta {
    /// Whether the two revisions carry identical authored truth (no edits).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
            && self.sheet_collections.is_empty()
            && self.sheets.is_empty()
            && self.settings.is_empty()
    }
}

/// The authored-truth inputs of one revision, borrowed for diffing.
///
/// This bundles **only** authored input snapshots — the structural snapshot,
/// the node-input snapshot (for settings), and the per-sheet grid-input map.
/// There is deliberately no field for published values, engine sheets, or any
/// other derived layer: the type itself enforces the input-only discipline of
/// D1 C6 at the diff boundary.
#[derive(Debug, Clone, Copy)]
pub struct AuthoredRevisionInputs<'a> {
    pub revision_id: &'a WorkspaceRevisionId,
    pub structure: &'a StructuralSnapshot,
    pub node_inputs: &'a NodeInputSnapshot,
    pub grid_inputs: &'a BTreeMap<TreeNodeId, GridInputState>,
}

/// Diff two revisions' authored truth into a [`WorkbookAuthoredDelta`].
///
/// Pure: it reads `since` and `current` and returns the delta. The grid diff is
/// short-circuited per sheet by [`GridInputSnapshotId`] equality (the fast
/// path); the sheet-lifecycle diff walks the two structural snapshots' Sheet-
/// role nodes; the settings diff reads the settings meta node inputs.
#[must_use]
pub fn diff_authored_revisions(
    since: AuthoredRevisionInputs<'_>,
    current: AuthoredRevisionInputs<'_>,
) -> WorkbookAuthoredDelta {
    let mut delta = WorkbookAuthoredDelta::default();

    diff_sheets(since, current, &mut delta);
    diff_grid_inputs(since, current, &mut delta);
    diff_settings(since, current, &mut delta);

    delta
}

/// Ordered Sheet-role nodes of a snapshot: `(node_id, display_name, normalized,
/// sheet_position)`. Sheet order is the root's `child_ids` filtered to Sheet-
/// role children (D1 C3), so `sheet_position` is the dense index into that
/// filtered order.
fn sheet_rows(
    structure: &StructuralSnapshot,
) -> Vec<(TreeNodeId, String, NormalizedSheetName, usize)> {
    structure
        .sheet_nodes()
        .into_iter()
        .enumerate()
        .filter_map(|(position, node_id)| {
            structure.try_get_node(node_id).map(|node| {
                let display = node.symbol.clone();
                let normalized = NormalizedSheetName::from_symbol(&display);
                (node_id, display, normalized, position)
            })
        })
        .collect()
}

fn diff_sheets(
    since: AuthoredRevisionInputs<'_>,
    current: AuthoredRevisionInputs<'_>,
    delta: &mut WorkbookAuthoredDelta,
) {
    let old_rows = sheet_rows(since.structure);
    let new_rows = sheet_rows(current.structure);

    let old_by_id: BTreeMap<TreeNodeId, (&String, &NormalizedSheetName, usize)> = old_rows
        .iter()
        .map(|(id, display, normalized, position)| (*id, (display, normalized, *position)))
        .collect();
    let new_by_id: BTreeMap<TreeNodeId, (&String, &NormalizedSheetName, usize)> = new_rows
        .iter()
        .map(|(id, display, normalized, position)| (*id, (display, normalized, *position)))
        .collect();

    // Deletes: in `since`, gone now. Node id, display, normalized come from
    // `since`'s snapshot (still present there) — self-sufficient, no tombstone.
    for (node_id, display, normalized, position) in &old_rows {
        if !new_by_id.contains_key(node_id) {
            delta.sheets.push(SheetLifecycleDelta::Deleted {
                node_id: *node_id,
                display_name: display.clone(),
                normalized_name: normalized.clone(),
                sheet_position: *position,
            });
        }
    }

    // Adds: present now, not in `since`.
    for (node_id, display, normalized, position) in &new_rows {
        if !old_by_id.contains_key(node_id) {
            delta.sheets.push(SheetLifecycleDelta::Added {
                node_id: *node_id,
                display_name: display.clone(),
                normalized_name: normalized.clone(),
                sheet_position: *position,
            });
        }
    }

    // Renames: surviving node whose normalized name changed. Rename is a
    // normalized-name change on the same node id (D1 C2), never a re-cap only.
    for (node_id, new_display, new_normalized, _) in &new_rows {
        if let Some((old_display, old_normalized, _)) = old_by_id.get(node_id)
            && *old_normalized != new_normalized
        {
            delta.sheets.push(SheetLifecycleDelta::Renamed {
                node_id: *node_id,
                old_normalized: (*old_normalized).clone(),
                new_normalized: (*new_normalized).clone(),
                old_display: (*old_display).clone(),
                new_display: new_display.clone(),
            });
        }
    }

    // Moves: surviving node whose sheet position changed.
    for (node_id, _, _, new_position) in &new_rows {
        if let Some((_, _, old_position)) = old_by_id.get(node_id)
            && *old_position != *new_position
        {
            delta.sheets.push(SheetLifecycleDelta::Moved {
                node_id: *node_id,
                old_sheet_position: *old_position,
                new_sheet_position: *new_position,
            });
        }
    }
}

fn diff_grid_inputs(
    since: AuthoredRevisionInputs<'_>,
    current: AuthoredRevisionInputs<'_>,
    delta: &mut WorkbookAuthoredDelta,
) {
    // Union of grid-backed sheet ids across both revisions, node-ordered so the
    // emitted rows are deterministic.
    let mut sheet_ids: BTreeSet<TreeNodeId> = BTreeSet::new();
    sheet_ids.extend(since.grid_inputs.keys().copied());
    sheet_ids.extend(current.grid_inputs.keys().copied());

    for sheet_node_id in sheet_ids {
        let old_grid = since.grid_inputs.get(&sheet_node_id);
        let new_grid = current.grid_inputs.get(&sheet_node_id);

        // Fast path: identical content address ⇒ no authored change on this
        // sheet, so it is not walked cell-by-cell (D1 C5 / §7.3). Two grids
        // present with equal ids short-circuit; two absent grids trivially do.
        if grid_input_id(old_grid) == grid_input_id(new_grid) {
            continue;
        }

        diff_one_grid(sheet_node_id, old_grid, new_grid, delta);
    }
}

/// The content address of an optional grid, or a sentinel for absence. Two
/// absent grids compare equal (no change); a present/absent pair never does.
fn grid_input_id(grid: Option<&GridInputState>) -> Option<GridInputSnapshotId> {
    grid.map(GridInputState::identity)
}

fn diff_one_grid(
    sheet_node_id: TreeNodeId,
    old_grid: Option<&GridInputState>,
    new_grid: Option<&GridInputState>,
    delta: &mut WorkbookAuthoredDelta,
) {
    let empty_cells: BTreeMap<ExcelGridCellAddress, GridInputCell> = BTreeMap::new();
    let old_cells = old_grid.map_or(&empty_cells, |g| &g.cells);
    let new_cells = new_grid.map_or(&empty_cells, |g| &g.cells);

    // Cells: union of addresses, address-ordered (BTreeMap key order).
    let mut addresses: BTreeSet<&ExcelGridCellAddress> = BTreeSet::new();
    addresses.extend(old_cells.keys());
    addresses.extend(new_cells.keys());
    for address in addresses {
        match (old_cells.get(address), new_cells.get(address)) {
            (None, Some(new)) => delta.cells.push(CellInputDelta {
                sheet_node_id,
                address: address.clone(),
                change: CellInputChange::Set { new: new.clone() },
            }),
            (Some(old), None) => delta.cells.push(CellInputDelta {
                sheet_node_id,
                address: address.clone(),
                change: CellInputChange::Cleared { old: old.clone() },
            }),
            (Some(old), Some(new)) if old != new => delta.cells.push(CellInputDelta {
                sheet_node_id,
                address: address.clone(),
                change: CellInputChange::Changed {
                    old: old.clone(),
                    new: new.clone(),
                },
            }),
            _ => {}
        }
    }

    // Non-cell authored collections: value-multiset set/removed diff.
    let repeated_regions = collection_delta(
        old_grid
            .map(|g| g.repeated_regions.as_slice())
            .unwrap_or(&[]),
        new_grid
            .map(|g| g.repeated_regions.as_slice())
            .unwrap_or(&[]),
    );
    let merged_regions = collection_delta(
        old_grid.map(|g| g.merged_regions.as_slice()).unwrap_or(&[]),
        new_grid.map(|g| g.merged_regions.as_slice()).unwrap_or(&[]),
    );
    let table_overlays = collection_delta(
        old_grid.map(|g| g.table_overlays.as_slice()).unwrap_or(&[]),
        new_grid.map(|g| g.table_overlays.as_slice()).unwrap_or(&[]),
    );
    let defined_names = collection_delta(
        old_grid.map(|g| g.defined_names.as_slice()).unwrap_or(&[]),
        new_grid.map(|g| g.defined_names.as_slice()).unwrap_or(&[]),
    );

    let collections = SheetCollectionsDelta {
        sheet_node_id,
        repeated_regions,
        merged_regions,
        table_overlays,
        defined_names,
    };
    if !collections.repeated_regions.is_empty()
        || !collections.merged_regions.is_empty()
        || !collections.table_overlays.is_empty()
        || !collections.defined_names.is_empty()
    {
        delta.sheet_collections.push(collections);
    }
}

/// Value-multiset diff of two authored collections. A member of `new` not
/// balanced by an equal member of `old` is `added`; a member of `old` not
/// balanced in `new` is `removed`. Order-only churn (same member multiset)
/// yields empty diffs — authored content is unchanged.
fn collection_delta<T: Clone + PartialEq>(old: &[T], new: &[T]) -> CollectionDelta<T> {
    let mut consumed = vec![false; old.len()];
    let mut added = Vec::new();
    for member in new {
        match old
            .iter()
            .enumerate()
            .find(|(index, candidate)| !consumed[*index] && *candidate == member)
        {
            Some((index, _)) => consumed[index] = true,
            None => added.push(member.clone()),
        }
    }
    let removed = old
        .iter()
        .enumerate()
        .filter(|&(index, _member)| !consumed[index])
        .map(|(_index, member)| member.clone())
        .collect();
    CollectionDelta { added, removed }
}

fn diff_settings(
    since: AuthoredRevisionInputs<'_>,
    current: AuthoredRevisionInputs<'_>,
    delta: &mut WorkbookAuthoredDelta,
) {
    let old = read_settings(since.structure, since.node_inputs);
    let new = read_settings(current.structure, current.node_inputs);

    if old.date_system != new.date_system {
        delta.settings.push(WorkbookSettingChanged::DateSystem {
            old: old.date_system,
            new: new.date_system,
        });
    }
    if old.calc_mode != new.calc_mode {
        delta.settings.push(WorkbookSettingChanged::CalcMode {
            old: old.calc_mode,
            new: new.calc_mode,
        });
    }
    if old.iteration != new.iteration {
        delta.settings.push(WorkbookSettingChanged::Iteration {
            old: old.iteration,
            new: new.iteration,
        });
    }
}

// Settings storage layout mirrors `consumer.rs` (`#workbook-settings` meta
// group + one literal-input grandchild per field). This module reads that
// layout independently from a `(structure, node_inputs)` pair — authored input
// only — so the diff never needs a live `OxCalcTreeWorkspaceState`. Kept a
// private reader here rather than sharing the consumer accessor (which takes
// state) to keep the delta module decoupled and the input-only discipline
// self-evident. If the wire encoding ever changes, both readers change with it.
const WORKBOOK_SETTINGS_GROUP_SYMBOL: &str = "#workbook-settings";
const WORKBOOK_SETTING_DATE_SYSTEM: &str = "date-system";
const WORKBOOK_SETTING_CALC_MODE: &str = "calc-mode";
const WORKBOOK_SETTING_ITERATION_ENABLED: &str = "iteration-enabled";
const WORKBOOK_SETTING_ITERATION_MAX_ITERATIONS: &str = "iteration-max-iterations";
const WORKBOOK_SETTING_ITERATION_MAX_CHANGE: &str = "iteration-max-change";

fn read_settings(
    structure: &StructuralSnapshot,
    node_inputs: &NodeInputSnapshot,
) -> WorkbookCalcSettings {
    use crate::workbook_settings::{CalcMode, DateSystem, IterationSettings};

    let defaults = WorkbookCalcSettings::default();
    let Some(group_id) = settings_group_node_id(structure) else {
        return defaults;
    };
    let wire = |symbol: &str| -> Option<String> {
        let node_id = child_by_symbol(structure, group_id, symbol)?;
        node_inputs
            .try_get_record(node_id)
            .and_then(|record| record.text.clone())
    };

    let date_system = wire(WORKBOOK_SETTING_DATE_SYSTEM).map_or(defaults.date_system, |text| {
        DateSystem::from_wire_text(&text)
    });
    let calc_mode = wire(WORKBOOK_SETTING_CALC_MODE)
        .map_or(defaults.calc_mode, |text| CalcMode::from_wire_text(&text));
    let iteration = IterationSettings {
        enabled: wire(WORKBOOK_SETTING_ITERATION_ENABLED)
            .map_or(defaults.iteration.enabled, |text| text == "true"),
        max_iterations: wire(WORKBOOK_SETTING_ITERATION_MAX_ITERATIONS)
            .and_then(|text| text.parse::<u32>().ok())
            .unwrap_or(defaults.iteration.max_iterations),
        max_change: wire(WORKBOOK_SETTING_ITERATION_MAX_CHANGE)
            .and_then(|text| text.parse::<f64>().ok())
            .unwrap_or(defaults.iteration.max_change),
    };
    WorkbookCalcSettings {
        date_system,
        calc_mode,
        iteration,
    }
}

fn settings_group_node_id(structure: &StructuralSnapshot) -> Option<TreeNodeId> {
    let root = structure.try_get_node(structure.root_node_id())?;
    root.child_ids.iter().copied().find(|child_id| {
        structure
            .try_get_node(*child_id)
            .is_some_and(|child| child.is_meta && child.symbol == WORKBOOK_SETTINGS_GROUP_SYMBOL)
    })
}

fn child_by_symbol(
    structure: &StructuralSnapshot,
    parent_id: TreeNodeId,
    symbol: &str,
) -> Option<TreeNodeId> {
    let parent = structure.try_get_node(parent_id)?;
    parent.child_ids.iter().copied().find(|child_id| {
        structure
            .try_get_node(*child_id)
            .is_some_and(|c| c.symbol == symbol)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consumer::{
        GridBackingSeed, OxCalcDocumentContext, OxCalcDocumentError, OxCalcTreeWorkspaceCreate,
        OxCalcTreeWorkspaceId,
    };
    use crate::grid::coords::{ExcelGridBounds, ExcelGridCellAddress};
    use crate::grid::geometry::GridRect;
    use crate::workbook_settings::{CalcMode, WorkbookCalcSettings};

    fn addr(sheet_id: &str, row: u32, col: u32) -> ExcelGridCellAddress {
        ExcelGridCellAddress::new("book:delta", sheet_id, row, col)
    }

    /// Grid-back `node_id` with an empty strict-Excel grid so cell-entry verbs
    /// can author on it.
    fn grid_back(
        context: &mut OxCalcDocumentContext,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
        sheet_id: &str,
    ) {
        context
            .set_node_grid(
                workspace_id,
                node_id,
                GridBackingSeed {
                    workbook_id: "book:delta".to_string(),
                    sheet_id: sheet_id.to_string(),
                    bounds: ExcelGridBounds::strict_excel(),
                    authored: Vec::new(),
                    table_overlays: Vec::new(),
                    merged_regions: Vec::new(),
                },
            )
            .unwrap();
    }

    /// The full scripted edit sequence of D4 §7b / the bead acceptance: enter a
    /// literal, enter a formula, add a sheet, rename a sheet, move a sheet,
    /// delete a sheet, define a name, and change a setting — between two
    /// revisions — yields exactly the expected typed delta rows and nothing else.
    #[test]
    fn scripted_edit_sequence_yields_exact_typed_delta() {
        let mut context = OxCalcDocumentContext::default();
        let workspace = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workbook:delta").as_workbook())
            .unwrap();

        // Baseline workbook: three sheets, the first grid-backed.
        let alpha = context.add_sheet(&workspace, "Alpha").unwrap();
        let beta = context.add_sheet(&workspace, "Beta").unwrap();
        let gamma = context.add_sheet(&workspace, "Gamma").unwrap();
        grid_back(&mut context, &workspace, alpha, "sheet:alpha");

        // Capture the `since` revision at the top of the scripted sequence.
        let since = context
            .workspace_revision(&workspace)
            .unwrap()
            .revision_id()
            .clone();

        // 1. Enter a literal on Alpha!A1.
        context
            .enter_grid_cell(&workspace, alpha, &addr("sheet:alpha", 1, 1), "42")
            .unwrap();
        // 2. Enter a formula on Alpha!B1.
        context
            .enter_grid_cell(&workspace, alpha, &addr("sheet:alpha", 1, 2), "=A1+1")
            .unwrap();
        // 3. Add a sheet (Delta).
        let delta_sheet = context.add_sheet(&workspace, "Delta").unwrap();
        // 4. Rename a sheet (Beta -> Renamed).
        context.rename_sheet(&workspace, beta, "Renamed").unwrap();
        // 5. Move a sheet (Gamma to sheet-position 0).
        context.move_sheet(&workspace, gamma, 0).unwrap();
        // 6. Delete a sheet (the newly added Delta).
        context.delete_sheet(&workspace, delta_sheet).unwrap();
        // 7. Define a workbook name on Alpha's grid.
        let name_target = GridRect::new(
            "book:delta",
            "sheet:alpha",
            1,
            1,
            3,
            1,
            ExcelGridBounds::strict_excel(),
        )
        .unwrap();
        context
            .set_workbook_defined_name(&workspace, alpha, "Total", name_target.clone())
            .unwrap();
        // 8. Change a setting (CalcMode -> Manual).
        context
            .set_workbook_calc_settings(
                &workspace,
                WorkbookCalcSettings {
                    calc_mode: CalcMode::Manual,
                    ..WorkbookCalcSettings::default()
                },
            )
            .unwrap();

        let delta = context.workbook_authored_delta(&workspace, &since).unwrap();

        // Cells: exactly the literal and the formula on Alpha, address-ordered.
        assert_eq!(
            delta.cells,
            vec![
                CellInputDelta {
                    sheet_node_id: alpha,
                    address: addr("sheet:alpha", 1, 1),
                    change: CellInputChange::Set {
                        new: GridInputCell::Literal(oxfunc_core::value::CalcValue::number(42.0)),
                    },
                },
                CellInputDelta {
                    sheet_node_id: alpha,
                    address: addr("sheet:alpha", 1, 2),
                    change: CellInputChange::Set {
                        new: GridInputCell::Formula {
                            source_text: "=A1+1".to_string(),
                            source_channel: oxfml_core::source::FormulaChannelKind::WorksheetA1,
                        },
                    },
                },
            ],
            "cells must be exactly the two authored entries, literal then formula"
        );

        // Sheet collections: exactly the one defined name added on Alpha.
        assert_eq!(delta.sheet_collections.len(), 1);
        let collections = &delta.sheet_collections[0];
        assert_eq!(collections.sheet_node_id, alpha);
        assert_eq!(
            collections.defined_names,
            CollectionDelta {
                added: vec![GridInputDefinedName {
                    scope: crate::grid::authored::GridDefinedNameScope::Workbook,
                    name: "Total".to_string(),
                    target: crate::grid::authored::GridDefinedNameTarget::Static(name_target),
                }],
                removed: Vec::new(),
            }
        );
        assert!(collections.repeated_regions.is_empty());
        assert!(collections.merged_regions.is_empty());
        assert!(collections.table_overlays.is_empty());

        // Sheet lifecycle: the added-then-deleted Delta sheet nets to nothing
        // (it exists in neither endpoint) — authored-truth diffing over
        // endpoints, not an event log. Beta -> Renamed is one Renamed row.
        // Moving Gamma to the front shifts EVERY sheet's position index, so the
        // faithful position diff reports three Moved rows (Gamma 2->0, Alpha
        // 0->1, Beta 1->2). The delta reports the true per-sheet order change,
        // never a synthesized "primary mover" (which two snapshots cannot
        // distinguish). Sheet order now: Gamma(0), Alpha(1), Renamed(2).
        assert_eq!(delta.sheets.len(), 4, "one rename + three position shifts");
        assert!(
            delta.sheets.contains(&SheetLifecycleDelta::Renamed {
                node_id: beta,
                old_normalized: NormalizedSheetName::from_symbol("Beta"),
                new_normalized: NormalizedSheetName::from_symbol("Renamed"),
                old_display: "Beta".to_string(),
                new_display: "Renamed".to_string(),
            }),
            "Beta -> Renamed reported by normalized-name change on the same node id"
        );
        assert!(delta.sheets.contains(&SheetLifecycleDelta::Moved {
            node_id: gamma,
            old_sheet_position: 2,
            new_sheet_position: 0,
        }));
        assert!(delta.sheets.contains(&SheetLifecycleDelta::Moved {
            node_id: alpha,
            old_sheet_position: 0,
            new_sheet_position: 1,
        }));
        assert!(delta.sheets.contains(&SheetLifecycleDelta::Moved {
            node_id: beta,
            old_sheet_position: 1,
            new_sheet_position: 2,
        }));
        assert!(
            !delta
                .sheets
                .iter()
                .any(|s| matches!(s, SheetLifecycleDelta::Added { node_id, .. } if *node_id == delta_sheet)),
            "the added-then-deleted sheet must not appear (endpoints only)"
        );

        // Settings: exactly the CalcMode change.
        assert_eq!(
            delta.settings,
            vec![WorkbookSettingChanged::CalcMode {
                old: CalcMode::Automatic,
                new: CalcMode::Manual,
            }]
        );

        assert!(!delta.is_empty());
    }

    /// The W011 step-4 assertion (bead acceptance): one authored edit reported,
    /// no derived-state leakage. A single literal entry between two revisions is
    /// the ONLY row in the delta — no published value, no sheet churn, nothing
    /// else. (Derived-state non-leakage is structural: the diff's input type
    /// carries authored truth only.)
    #[test]
    fn single_authored_edit_reports_one_row_no_derived_leakage() {
        let mut context = OxCalcDocumentContext::default();
        let workspace = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workbook:w011").as_workbook())
            .unwrap();
        let sheet = context.add_sheet(&workspace, "Sheet1").unwrap();
        grid_back(&mut context, &workspace, sheet, "sheet:1");
        let since = context
            .workspace_revision(&workspace)
            .unwrap()
            .revision_id()
            .clone();

        context
            .enter_grid_cell(&workspace, sheet, &addr("sheet:1", 1, 1), "7")
            .unwrap();

        let delta = context.workbook_authored_delta(&workspace, &since).unwrap();
        assert_eq!(
            delta.cells,
            vec![CellInputDelta {
                sheet_node_id: sheet,
                address: addr("sheet:1", 1, 1),
                change: CellInputChange::Set {
                    new: GridInputCell::Literal(oxfunc_core::value::CalcValue::number(7.0)),
                },
            }]
        );
        assert!(delta.sheet_collections.is_empty());
        assert!(delta.sheets.is_empty());
        assert!(delta.settings.is_empty());
    }

    /// Clearing and editing report the right transition variants.
    #[test]
    fn cleared_and_changed_cell_transitions_are_classified() {
        let mut context = OxCalcDocumentContext::default();
        let workspace = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workbook:xition").as_workbook())
            .unwrap();
        let sheet = context.add_sheet(&workspace, "S").unwrap();
        grid_back(&mut context, &workspace, sheet, "sheet:s");
        // Author A1 and A2 before the `since` point.
        context
            .enter_grid_cell(&workspace, sheet, &addr("sheet:s", 1, 1), "1")
            .unwrap();
        context
            .enter_grid_cell(&workspace, sheet, &addr("sheet:s", 2, 1), "2")
            .unwrap();
        let since = context
            .workspace_revision(&workspace)
            .unwrap()
            .revision_id()
            .clone();

        // Clear A1, change A2 (literal -> formula flip).
        context
            .clear_grid_cell(&workspace, sheet, &addr("sheet:s", 1, 1))
            .unwrap();
        context
            .enter_grid_cell(&workspace, sheet, &addr("sheet:s", 2, 1), "=1+1")
            .unwrap();

        let delta = context.workbook_authored_delta(&workspace, &since).unwrap();
        assert_eq!(delta.cells.len(), 2);
        assert_eq!(
            delta.cells[0],
            CellInputDelta {
                sheet_node_id: sheet,
                address: addr("sheet:s", 1, 1),
                change: CellInputChange::Cleared {
                    old: GridInputCell::Literal(oxfunc_core::value::CalcValue::number(1.0)),
                },
            }
        );
        assert!(matches!(
            delta.cells[1].change,
            CellInputChange::Changed {
                old: GridInputCell::Literal(_),
                new: GridInputCell::Formula { .. },
            }
        ));
    }

    /// Fast path: a 100-sheet workbook with one edited sheet yields cell rows
    /// for exactly that one sheet — every unedited sheet short-circuits on
    /// content-address equality and is never walked cell-by-cell.
    #[test]
    fn unchanged_sheets_short_circuit_on_content_address() {
        let mut context = OxCalcDocumentContext::default();
        let workspace = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workbook:fast").as_workbook())
            .unwrap();

        let mut sheets = Vec::new();
        for index in 0..100 {
            let node = context.add_sheet(&workspace, format!("S{index}")).unwrap();
            grid_back(&mut context, &workspace, node, &format!("sheet:{index}"));
            // Seed a cell on every sheet so unedited sheets are non-trivial.
            context
                .enter_grid_cell(
                    &workspace,
                    node,
                    &addr(&format!("sheet:{index}"), 1, 1),
                    "0",
                )
                .unwrap();
            sheets.push((node, index));
        }
        let since = context
            .workspace_revision(&workspace)
            .unwrap()
            .revision_id()
            .clone();

        // Edit exactly one sheet (index 42).
        let (edited_node, edited_index) = sheets[42];
        context
            .enter_grid_cell(
                &workspace,
                edited_node,
                &addr(&format!("sheet:{edited_index}"), 2, 2),
                "99",
            )
            .unwrap();

        let delta = context.workbook_authored_delta(&workspace, &since).unwrap();
        assert_eq!(
            delta.cells.len(),
            1,
            "only the one edited sheet contributes a cell row"
        );
        assert_eq!(delta.cells[0].sheet_node_id, edited_node);
        assert!(delta.sheet_collections.is_empty());
        assert!(delta.sheets.is_empty());
    }

    /// Direct content-address fast-path evidence: two structurally-equal
    /// `GridInputState`s (distinct allocations) share a `GridInputSnapshotId`,
    /// so [`diff_authored_revisions`] short-circuits the sheet without walking
    /// its cells — the short-circuit is content-address, not pointer, identity.
    #[test]
    fn grid_input_id_equality_short_circuits_the_diff() {
        use crate::structural::{
            StructuralNode, StructuralNodeKind, StructuralSnapshot, StructuralSnapshotId,
        };
        use crate::workspace_revision::{NamespaceSnapshot, NodeInputRecord, WorkspaceRevision};

        // Build a trivial one-node structure + revision shell so we can drive
        // the pure diff directly with hand-built grid-input maps.
        let root = TreeNodeId(1);
        let structure = StructuralSnapshot::create(
            StructuralSnapshotId(1),
            root,
            [StructuralNode {
                node_id: root,
                kind: StructuralNodeKind::Root,
                symbol: "Root".to_string(),
                parent_id: None,
                child_ids: Vec::new(),
                role: None,
                is_meta: false,
            }],
        )
        .unwrap();
        let node_inputs =
            crate::workspace_revision::NodeInputSnapshot::create([NodeInputRecord::empty(root, 1)])
                .unwrap();
        let revision = WorkspaceRevision::new(
            "workbook:fastunit",
            structure.clone(),
            node_inputs.clone(),
            Default::default(),
            NamespaceSnapshot::current_absent(),
        );

        let sheet = TreeNodeId(7);
        // Two independently-built but structurally-identical grids.
        let mut grid_a = GridInputState::new("book:x", "sheet:x", ExcelGridBounds::strict_excel());
        grid_a.cells.insert(
            addr("sheet:x", 1, 1),
            GridInputCell::Literal(oxfunc_core::value::CalcValue::number(5.0)),
        );
        let grid_b = grid_a.clone();
        assert_eq!(
            grid_a.identity(),
            grid_b.identity(),
            "structurally-equal grids share a content address"
        );

        let old_map: BTreeMap<TreeNodeId, GridInputState> = BTreeMap::from([(sheet, grid_a)]);
        let new_map: BTreeMap<TreeNodeId, GridInputState> = BTreeMap::from([(sheet, grid_b)]);

        let since = AuthoredRevisionInputs {
            revision_id: revision.revision_id(),
            structure: &structure,
            node_inputs: &node_inputs,
            grid_inputs: &old_map,
        };
        let current = AuthoredRevisionInputs {
            revision_id: revision.revision_id(),
            structure: &structure,
            node_inputs: &node_inputs,
            grid_inputs: &new_map,
        };
        let delta = diff_authored_revisions(since, current);
        assert!(
            delta.is_empty(),
            "equal content addresses short-circuit: no rows despite distinct allocations"
        );
    }

    /// An out-of-window `since` revision is a typed error (D4 §7b: retained-
    /// revision availability is the natural boundary).
    #[test]
    fn out_of_window_since_revision_is_a_typed_error() {
        let mut context = OxCalcDocumentContext::default();
        let workspace = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workbook:window").as_workbook())
            .unwrap();
        context.add_sheet(&workspace, "S").unwrap();

        // A revision id that was never minted for this workspace is not retained.
        let bogus = WorkspaceRevisionId("workspace-revision:never-existed".to_string());
        let result = context.workbook_authored_delta(&workspace, &bogus);
        assert!(matches!(
            result,
            Err(OxCalcDocumentError::WorkspaceRevisionNotRetained { .. })
        ));
    }
}
