//! `GridCalcRefWorkbook`: the workbook reference oracle (W062 D3 §5, bead
//! `calc-5kqg.33` / R4.5).
//!
//! # The one generalization
//!
//! This is the per-sheet oracle ([`GridCalcRefSheet`]) with **exactly one**
//! generalization: the value table a formula evaluation reads is no longer a
//! single sheet's `computed` map but a **workbook value table** — the
//! `computed` maps of *every* sheet, indexed by the sheet's stable
//! [`TreeNodeId`]. Every other line of evaluation is the unchanged per-sheet
//! algorithm: each sheet still runs its own
//! [`GridCalcRefSheet::recalculate_mark_all_dirty_with_oxfml`] mark-all pass,
//! its own worklist, its own spill repair, its own cycle stall extraction. The
//! oracle owns only the *coordination*: before a sheet recalculates, it seeds
//! that sheet's cross-sheet view (R3.3's `set_cross_sheet_cells` seam) from the
//! current workbook value table, using the exact same catalog route the R3.3
//! two-sheet test performs by hand — lifted from "one formula" to "every
//! formula on the sheet".
//!
//! Mark-all across all sheets, every run. No incrementality, no caching, no
//! cleverness. Correctness is meant to be obvious top-to-bottom: a reviewer
//! reads [`GridCalcRefWorkbook::recalculate`] and sees a bounded loop of
//! whole-sheet mark-all passes that stops when nothing changed.
//!
//! # Fixpoint and termination
//!
//! [`recalculate`](GridCalcRefWorkbook::recalculate) repeats *rounds*. A round
//! recalculates every sheet once, in [`TreeNodeId`] order, each against the
//! workbook value table as it stood at the **start** of the round (values a
//! sheet publishes this round are visible to later sheets and later rounds, but
//! the round-boundary snapshot is what the fixpoint test compares against). A
//! round that changes no sheet's `computed` map is a fixpoint: nothing a
//! further round could read has changed, so a further round would reproduce the
//! same values — the loop stops and publishes.
//!
//! **Cycle detection comes first, structurally.** Before the fixpoint loop
//! runs, the oracle builds the effective cell dependency graph across every
//! sheet and looks for a directed cycle that visits more than one sheet — a
//! **cross-sheet cycle**. This is a structural fact (it exists or it does not,
//! independent of any value), so detecting it up front means the fixpoint loop
//! only ever runs on an *acyclic* cross-sheet graph. A found cross-sheet cycle
//! is reported as
//! [`GridRefError::WorkbookEffectiveDependencyCycleDetected`]. An *intra*-sheet
//! cycle is left to the per-sheet mark-all, whose worklist stall extracts a
//! tighter cycle and returns
//! [`GridRefError::EffectiveDependencyCycleDetected`]; the oracle catches that
//! inside the loop and widens it to the workbook node space.
//!
//! **Termination.** With cross-sheet cycles ruled out, the cross-sheet
//! dependency graph over sheets is a DAG. A value crosses at most `n = sheet_count`
//! sheet boundaries along its longest chain, so after `n` rounds every value
//! has propagated the full length of its longest cross-sheet chain, and the
//! `(n+1)`-th round is a confirming no-change round: the loop is hard-bounded
//! at `n + 1` rounds and always reaches a fixpoint within it. The loop
//! therefore always terminates — at a fixpoint, or (for an intra-sheet cycle)
//! with the per-sheet mark-all's typed error.

use super::*;

use crate::structural::{StructuralSnapshot, TreeNodeId};
use crate::workbook_reference_catalog::{
    WorkbookReferenceCatalog, gather_cross_sheet_cells,
};

/// The workbook reference oracle: multiple [`GridCalcRefSheet`]s plus the
/// workbook value table, evaluated mark-all across sheets in dependency order
/// to a fixpoint (W062 D3 §5).
#[derive(Debug, Clone)]
pub struct GridCalcRefWorkbook {
    /// The member sheets, keyed by their stable [`TreeNodeId`]. This map's
    /// values' `computed` fields together *are* the workbook value table; the
    /// oracle never stores a second copy.
    sheets: BTreeMap<TreeNodeId, GridCalcRefSheet>,
    /// The cross-sheet routing catalog, a pure function of the workbook's
    /// structural snapshot (R3.3 / D2 §4.1). Routes each sheet's authored
    /// dependencies to the sheets that own their targets.
    catalog: WorkbookReferenceCatalog,
}

/// A per-sheet recalc summary the oracle aggregates over a workbook recalc
/// (W062 D3 §5). Deterministic by construction: keyed by [`TreeNodeId`] in a
/// [`BTreeMap`].
#[derive(Debug, Clone)]
pub struct GridCalcRefWorkbookRecalcReport {
    /// The number of full-workbook rounds run to reach the fixpoint (≥ 1).
    pub rounds: usize,
    /// Each member sheet's final-round recalc report, keyed by node id.
    pub per_sheet: BTreeMap<TreeNodeId, GridCalcRefRecalcReport>,
}

impl GridCalcRefWorkbook {
    /// Build a workbook oracle over `sheets`, keyed by the [`TreeNodeId`] each
    /// sheet is registered under in `snapshot`. The catalog is built from
    /// `snapshot` (the same pure `WorkbookReferenceCatalog::build` R3.3 uses),
    /// so cross-sheet references resolve exactly as they do in the single-sheet
    /// R3.3 slice.
    #[must_use]
    pub fn new(
        snapshot: &StructuralSnapshot,
        sheets: impl IntoIterator<Item = (TreeNodeId, GridCalcRefSheet)>,
    ) -> Self {
        Self {
            sheets: sheets.into_iter().collect(),
            catalog: WorkbookReferenceCatalog::build(snapshot),
        }
    }

    /// The member sheet for `node`, if present.
    #[must_use]
    pub fn sheet(&self, node: TreeNodeId) -> Option<&GridCalcRefSheet> {
        self.sheets.get(&node)
    }

    /// Mutable access to a member sheet, for edits between recalcs. An edit
    /// followed by [`recalculate`](Self::recalculate) is the reference behavior
    /// R4.6's incremental lane must match: the oracle is mark-all, so a Sheet1
    /// edit then a workbook recalc yields fresh Sheet2 values.
    #[must_use]
    pub fn sheet_mut(&mut self, node: TreeNodeId) -> Option<&mut GridCalcRefSheet> {
        self.sheets.get_mut(&node)
    }

    /// The member sheets, keyed by node id (the workbook value table's carrier).
    #[must_use]
    pub fn sheets(&self) -> &BTreeMap<TreeNodeId, GridCalcRefSheet> {
        &self.sheets
    }

    /// Read a cell's committed value from whichever member sheet owns it. Empty
    /// if the sheet is not a member or the cell is unpopulated.
    #[must_use]
    pub fn read_cell(&self, node: TreeNodeId, address: &ExcelGridCellAddress) -> CalcValue {
        self.sheets
            .get(&node)
            .map(|sheet| sheet.read_cell(address))
            .unwrap_or_else(CalcValue::empty)
    }

    /// Recalculate the whole workbook mark-all to a fixpoint (W062 D3 §5).
    ///
    /// Two obvious steps, in order:
    ///
    /// 1. **Cross-sheet cycle check.** Build the effective cell dependency
    ///    graph across every sheet (each authored formula's static structural
    ///    dependencies, target cells only) and look for a cycle that visits
    ///    more than one sheet. A cross-sheet cycle is a structural fact — it
    ///    exists or it does not, independent of values — so detecting it here,
    ///    up front, means the fixpoint loop only ever runs on an acyclic
    ///    cross-sheet graph and is guaranteed to converge. (An *intra*-sheet
    ///    cycle is left to the per-sheet mark-all's own stall extraction, which
    ///    reports a tighter cycle; it is caught inside the loop and widened.)
    /// 2. **The fixpoint loop.** Each round refreshes every sheet's cross-sheet
    ///    view from the *live* workbook value table and mark-all-recalculates
    ///    that sheet. The loop stops the first round nothing changed. Because
    ///    step 1 removed cross-sheet cycles, convergence is guaranteed within
    ///    `sheet_count + 1` rounds (module docs give the propagation argument);
    ///    exceeding it is an internal invariant violation, not a user cycle.
    pub fn recalculate(
        &mut self,
    ) -> Result<GridCalcRefWorkbookRecalcReport, GridRefError> {
        if let Some(cycle) = self.cross_sheet_cycle() {
            return Err(GridRefError::WorkbookEffectiveDependencyCycleDetected { cycle });
        }

        // `sheet_count + 1` is the propagation bound for an acyclic cross-sheet
        // graph: a value crosses at most `sheet_count` sheet boundaries, and
        // one more round confirms no change. Step 1 guarantees acyclicity, so
        // this loop always converges within the bound.
        let max_rounds = self.sheets.len() + 1;
        let mut per_sheet = BTreeMap::new();
        let mut rounds = 0usize;

        loop {
            rounds += 1;
            // The workbook value table as it stood at the start of this round —
            // the snapshot the fixpoint test compares against. A round that
            // reproduces this snapshot is a fixpoint.
            let value_table_at_round_start: BTreeMap<TreeNodeId, BTreeMap<_, _>> = self
                .sheets
                .iter()
                .map(|(node, sheet)| (*node, sheet.computed().clone()))
                .collect();

            per_sheet.clear();
            let node_order: Vec<TreeNodeId> = self.sheets.keys().copied().collect();
            for node in node_order {
                // Seed this sheet's cross-sheet view from the *live* workbook
                // value table (peers already recalculated this round are
                // visible — latest-available values, D3 §3/§4 sweep model),
                // using the catalog route (exactly the R3.3 gather step).
                let cross = self.gather_cross_sheet_view(node);
                let sheet = self
                    .sheets
                    .get_mut(&node)
                    .expect("node came from self.sheets keys");
                sheet.set_cross_sheet_cells(cross);
                let report = sheet.recalculate_mark_all_dirty_with_oxfml().map_err(
                    Self::widen_intra_sheet_cycle,
                )?;
                per_sheet.insert(node, report);
            }

            // Fixpoint test: did any sheet's computed values change this round?
            let changed = self.sheets.iter().any(|(node, sheet)| {
                value_table_at_round_start
                    .get(node)
                    .map(|previous| previous != sheet.computed())
                    .unwrap_or(true)
            });
            if !changed {
                return Ok(GridCalcRefWorkbookRecalcReport { rounds, per_sheet });
            }
            if rounds >= max_rounds {
                // Unreachable given step 1's acyclicity guarantee; surface it
                // as a typed non-convergence rather than loop forever, so an
                // invariant break is loud, not a hang.
                return Err(GridRefError::IncrementalRecalcDidNotConverge {
                    iteration_limit: max_rounds,
                });
            }
        }
    }

    /// Gather the cross-sheet resolved values `node`'s formulas reference, from
    /// the live workbook value table, via the catalog route (R3.3
    /// `route_dependencies` + `gather_cross_sheet_cells`). Own-sheet targets
    /// route as `SameSheet` and are dropped; `set_cross_sheet_cells` filters
    /// own-sheet entries anyway.
    fn gather_cross_sheet_view(
        &self,
        node: TreeNodeId,
    ) -> BTreeMap<ExcelGridCellAddress, CalcValue> {
        let Some(sheet) = self.sheets.get(&node) else {
            return BTreeMap::new();
        };
        // The live workbook value table: every member sheet's current computed
        // map, keyed by node. Peers recalculated earlier this round are already
        // fresh here (latest-available-values sweep).
        let value_table: BTreeMap<TreeNodeId, BTreeMap<ExcelGridCellAddress, CalcValue>> = self
            .sheets
            .iter()
            .map(|(peer, peer_sheet)| (*peer, peer_sheet.computed().clone()))
            .collect();
        let dependencies = sheet.authored_formula_structural_dependencies();
        let routed = self
            .catalog
            .route_dependencies(sheet.sheet_id(), dependencies.iter());
        gather_cross_sheet_cells(&routed.routed, &value_table)
    }

    /// A per-sheet mark-all that stalled reports an *intra*-sheet cycle in the
    /// cell-address space; widen it to the workbook node space so the oracle's
    /// caller sees one uniform cross-node cycle error. Every other error passes
    /// through unchanged.
    fn widen_intra_sheet_cycle(error: GridRefError) -> GridRefError {
        match error {
            GridRefError::EffectiveDependencyCycleDetected { cycle } => {
                GridRefError::WorkbookEffectiveDependencyCycleDetected {
                    cycle: cycle
                        .into_iter()
                        .map(WorkbookCalcNodeId::GridCell)
                        .collect(),
                }
            }
            other => other,
        }
    }

    /// The effective cell dependency graph across all sheets: each authored
    /// formula cell mapped to the target cells its static structural
    /// dependencies name (same-sheet and cross-sheet alike). Cell/Range
    /// dependencies contribute their addressed cells; non-address dependency
    /// shapes (names, tables, dynamic requests, …) carry no cross-sheet cell
    /// edge and are dropped — the oracle's cycle check is over the cell space,
    /// which is where cross-sheet cycles live.
    ///
    /// This is the same per-formula `grid_structural_dependencies_for_formula`
    /// extraction the recalc worklist uses, gathered over every formula and
    /// projected to target cells. Deterministic: `BTreeMap`/`BTreeSet` keyed on
    /// `ExcelGridCellAddress`.
    fn effective_cell_edges(
        &self,
    ) -> BTreeMap<ExcelGridCellAddress, BTreeSet<ExcelGridCellAddress>> {
        let mut edges: BTreeMap<ExcelGridCellAddress, BTreeSet<ExcelGridCellAddress>> =
            BTreeMap::new();
        for sheet in self.sheets.values() {
            let bounds = sheet.bounds();
            let profile = StrictExcelGridReferenceProfile::with_bounds(bounds);
            for (address, cell) in sheet.authored() {
                let GridAuthoredCell::Formula(formula) = cell else {
                    continue;
                };
                let provider = sheet.reference_system_provider(address.row, address.col);
                let dependencies = grid_structural_dependencies_for_formula(
                    formula, address, &profile, bounds, &provider,
                );
                let targets = edges.entry(address.clone()).or_default();
                for dependency in &dependencies {
                    match dependency {
                        GridDependency::Cell(target) => {
                            targets.insert(target.clone());
                        }
                        GridDependency::Range(rect) => {
                            // Bound the fan-out at the same scalar-invalidation
                            // limit the rest of the machine uses; a range too
                            // large to enumerate cannot form a small cycle we
                            // would report, so dropping it is safe for the
                            // oracle's detection purpose.
                            if let Ok(cells) = rect
                                .scalar_cells(GRID_CALC_REF_DEFAULT_MATERIALIZATION_LIMIT)
                            {
                                targets.extend(cells);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        edges
    }

    /// Detect a **cross-sheet** cycle in the effective cell graph: a directed
    /// cycle among authored formula cells that visits more than one sheet.
    /// Returns the participating cells as `WorkbookCalcNodeId::GridCell`s in
    /// deterministic (BTree) order, or `None` if no cross-sheet cycle exists.
    ///
    /// Plain iterative DFS with a recursion stack (grey/black colouring). A
    /// back-edge closes a cycle; the cycle's cells are the current stack from
    /// the back-edge target down to the current node. We report it only if
    /// those cells span at least two distinct sheets — a same-sheet cycle is
    /// left to the per-sheet mark-all's tighter stall extraction (which runs
    /// inside the fixpoint loop and is widened on the way out). Iteration order
    /// is the `BTreeMap` key order, so the reported cycle is deterministic.
    fn cross_sheet_cycle(&self) -> Option<Vec<WorkbookCalcNodeId>> {
        let edges = self.effective_cell_edges();

        #[derive(Clone, Copy, PartialEq)]
        enum Colour {
            Grey,
            Black,
        }
        let mut colour: BTreeMap<ExcelGridCellAddress, Colour> = BTreeMap::new();

        // Each frame holds a node plus the not-yet-visited successors, so the
        // walk is an explicit stack (no recursion depth risk on large graphs).
        for root in edges.keys() {
            if colour.contains_key(root) {
                continue;
            }
            let mut stack: Vec<(ExcelGridCellAddress, std::vec::IntoIter<ExcelGridCellAddress>)> =
                Vec::new();
            let root_succ = edges
                .get(root)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect::<Vec<_>>();
            colour.insert(root.clone(), Colour::Grey);
            stack.push((root.clone(), root_succ.into_iter()));

            while let Some((node, successors)) = stack.last_mut() {
                let node = node.clone();
                if let Some(target) = successors.next() {
                    match colour.get(&target) {
                        Some(Colour::Grey) => {
                            // Back-edge: the cycle is the stack from `target`
                            // down to `node`, plus the closing edge back to
                            // `target`.
                            let start = stack
                                .iter()
                                .position(|(cell, _)| *cell == target)
                                .expect("grey node is on the stack");
                            let members: BTreeSet<ExcelGridCellAddress> = stack[start..]
                                .iter()
                                .map(|(cell, _)| cell.clone())
                                .collect();
                            let sheets: BTreeSet<&str> =
                                members.iter().map(|cell| cell.sheet_id.as_str()).collect();
                            if sheets.len() > 1 {
                                return Some(
                                    members
                                        .into_iter()
                                        .map(WorkbookCalcNodeId::GridCell)
                                        .collect(),
                                );
                            }
                            // Same-sheet cycle: not ours to report here.
                        }
                        Some(Colour::Black) => {}
                        None => {
                            colour.insert(target.clone(), Colour::Grey);
                            let succ = edges
                                .get(&target)
                                .cloned()
                                .unwrap_or_default()
                                .into_iter()
                                .collect::<Vec<_>>();
                            stack.push((target, succ.into_iter()));
                        }
                    }
                } else {
                    colour.insert(node, Colour::Black);
                    stack.pop();
                }
            }
        }
        None
    }
}
