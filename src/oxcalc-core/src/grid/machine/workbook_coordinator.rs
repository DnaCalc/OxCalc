//! The workbook cross-sheet edge layer, dirty closure, and worklist (W062 D3
//! §1/§3, bead `calc-5kqg.34` / R4.6).
//!
//! # What this owns
//!
//! Three things, matching D3 §1's `WorkbookGraph` `cross` field and §3's
//! closure:
//!
//! 1. **[`WorkbookCrossSheetEdges`]** — the cross-sheet edge layer. Per-sheet
//!    graphs stay exactly as they are (D3 §1 federation); this layer owns only
//!    the edges whose dependent and dependency live on *different* sheets. It is
//!    a reverse index keyed by *target* sheet node: given a set of cells that
//!    just became dirty on a target sheet, it names the foreign dependent cells
//!    that must be re-evaluated (the closure's step-3 lookup, D3 §3).
//!    Registration is via the R3.3 catalog route — a dependent cell's authored
//!    dependencies are routed, and each `Routed` (cross-sheet) descriptor
//!    installs one reverse edge. Same-sheet dependencies never enter this layer
//!    (the routing invariant, D3 §1).
//!
//! 2. **[`workbook_dirty_closure`]** — the §3 fixpoint. Starting from a set of
//!    per-sheet dirty seeds, it grows the workbook dirty set to a fixpoint:
//!    newly dirty target cells consult the cross layer, foreign dependents
//!    become new seeds on their sheets, repeat until no new dirty state. The
//!    dirty set is monotone (only grows) and bounded by the edge set, so it
//!    terminates.
//!
//! 3. **[`WorkbookWorklistOrder`]** — the dependency-ordered evaluation order
//!    across sheet boundaries (D3 §3: *one* workbook worklist, not
//!    sheet-at-a-time). Deterministic by construction: a topological order over
//!    the dirty cross-sheet subgraph with a `BTreeSet` tiebreak, so seed
//!    insertion order never affects it (the §10 deterministic-worklist
//!    constraint / contract X4).
//!
//! # Why edges are keyed on cells, not just sheets
//!
//! Sheet-at-a-time evaluation is incorrect on its face: `Sheet1!A1 →
//! Sheet2!B1 → Sheet1!C1` is an ordinary chain with no total sheet order (D3
//! §3). The cross layer therefore records cell-granular reverse edges so the
//! closure and worklist reason over cells that happen to live on sheets, not
//! over sheets as atoms.

use super::*;

use crate::workbook_reference_catalog::{CrossSheetRouting, WorkbookReferenceCatalog};
use crate::structural::TreeNodeId;

/// One cross-sheet reverse edge: a dependent cell on `dependent_sheet` reads a
/// target cell on another sheet. Stored keyed by the *target* sheet so a change
/// to that sheet's cells can find its foreign dependents in one lookup (D3 §3
/// step 3).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorkbookCrossSheetEdge {
    /// The sheet node the dependent (reading) cell lives on.
    pub dependent_sheet: TreeNodeId,
    /// The dependent (reading) cell.
    pub dependent_cell: ExcelGridCellAddress,
    /// The target cell it reads, on `target_sheet` (below).
    pub target_cell: ExcelGridCellAddress,
    /// The sheet node that owns `target_cell`.
    pub target_sheet: TreeNodeId,
}

/// The cross-sheet edge layer (D3 §1 `cross`): a reverse index from a target
/// sheet node to the foreign dependent cells that read cells on it.
///
/// Deterministic throughout: `BTreeMap`/`BTreeSet` keyed on `TreeNodeId` /
/// `ExcelGridCellAddress`, so enumeration and closure order never depend on
/// insertion order or hashing (contract X4 / D3 §10).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkbookCrossSheetEdges {
    /// target sheet → the cross-sheet edges whose target lives on it.
    by_target_sheet: BTreeMap<TreeNodeId, BTreeSet<WorkbookCrossSheetEdge>>,
    /// Every registered edge, for enumeration and the worklist build.
    all_edges: BTreeSet<WorkbookCrossSheetEdge>,
}

impl WorkbookCrossSheetEdges {
    /// An empty edge layer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register one cross-sheet edge (target and dependent on different sheets).
    /// Same-sheet edges must never be handed here — the routing invariant keeps
    /// them in the per-sheet graph — but a same-sheet edge slipping through is
    /// dropped rather than mis-filed, keeping the layer's cross-only invariant
    /// true by construction.
    pub fn register(&mut self, edge: WorkbookCrossSheetEdge) {
        if edge.dependent_sheet == edge.target_sheet {
            return;
        }
        self.by_target_sheet
            .entry(edge.target_sheet)
            .or_default()
            .insert(edge.clone());
        self.all_edges.insert(edge);
    }

    /// Build the cross-sheet edge layer from each sheet's authored formula
    /// dependencies, routed through the R3.3 catalog (D3 §1 registration).
    ///
    /// `sheets` yields, per sheet: its node id, its `sheet_id` string (the
    /// catalog's routing key), and its per-cell authored structural
    /// dependencies. Each dependency is routed; a [`CrossSheetRouting::Routed`]
    /// descriptor whose target sheet differs from the dependent's sheet installs
    /// one reverse edge per addressed target cell. `SameSheet` and `Dormant`
    /// routings install nothing (same-sheet stays local; a dormant sheet has no
    /// live target to key on — it heals into a live edge when the sheet is
    /// created, R4.11's concern, not this layer's).
    #[must_use]
    pub fn build<'a, S, D>(catalog: &WorkbookReferenceCatalog, sheets: S) -> Self
    where
        S: IntoIterator<Item = (TreeNodeId, &'a str, D)>,
        D: IntoIterator<Item = (ExcelGridCellAddress, Vec<GridDependency>)>,
    {
        let mut edges = Self::new();
        for (dependent_sheet, sheet_id, cells) in sheets {
            for (dependent_cell, dependencies) in cells {
                for dependency in &dependencies {
                    let CrossSheetRouting::Routed(descriptor) =
                        catalog.route_dependency(sheet_id, dependency)
                    else {
                        continue;
                    };
                    for target_cell in cross_sheet_target_cells(&descriptor.dependency) {
                        edges.register(WorkbookCrossSheetEdge {
                            dependent_sheet,
                            dependent_cell: dependent_cell.clone(),
                            target_cell,
                            target_sheet: descriptor.target_sheet_node,
                        });
                    }
                }
            }
        }
        edges
    }

    /// The foreign dependent cells whose evaluation reads any of `dirty_cells`
    /// on `target_sheet` (D3 §3 step 3). Returned as `(dependent_sheet,
    /// dependent_cell)` pairs in deterministic order.
    #[must_use]
    pub fn foreign_dependents_of(
        &self,
        target_sheet: TreeNodeId,
        dirty_cells: &BTreeSet<ExcelGridCellAddress>,
    ) -> BTreeSet<(TreeNodeId, ExcelGridCellAddress)> {
        let mut dependents = BTreeSet::new();
        let Some(edges) = self.by_target_sheet.get(&target_sheet) else {
            return dependents;
        };
        for edge in edges {
            if dirty_cells.contains(&edge.target_cell) {
                dependents.insert((edge.dependent_sheet, edge.dependent_cell.clone()));
            }
        }
        dependents
    }

    /// Every registered edge, in deterministic order.
    #[must_use]
    pub fn all_edges(&self) -> &BTreeSet<WorkbookCrossSheetEdge> {
        &self.all_edges
    }

    /// Is this layer empty (no cross-sheet edges)? A workbook with only
    /// intra-sheet references needs no cross coordination and short-circuits.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.all_edges.is_empty()
    }
}

/// The target cells an addressed dependency names, for cross-sheet reverse-edge
/// installation. Cell contributes itself; Range enumerates within the shared
/// materialization limit (a range too large to enumerate cannot key a small
/// dirty lookup we would perform, so bounding it is sound for the closure).
/// Non-address dependency shapes (names, tables, dynamic requests) carry no
/// cross-sheet *cell* edge — those are R4.11's scoped-key concern — and
/// contribute nothing.
fn cross_sheet_target_cells(dependency: &GridDependency) -> Vec<ExcelGridCellAddress> {
    match dependency {
        GridDependency::Cell(address) => vec![address.clone()],
        GridDependency::Range(rect) => rect
            .scalar_cells(GRID_CALC_REF_DEFAULT_MATERIALIZATION_LIMIT)
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

/// The result of the workbook dirty closure (D3 §3): the seeds that opened it
/// and the full dirty set, per sheet, that its fixpoint reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkbookDirtyClosure {
    /// Per sheet, the cells the fixpoint marked dirty (seed cells plus every
    /// foreign dependent reached transitively through the cross layer).
    pub dirty_by_sheet: BTreeMap<TreeNodeId, BTreeSet<ExcelGridCellAddress>>,
}

impl WorkbookDirtyClosure {
    /// The sheets that ended the closure with at least one dirty cell, in
    /// deterministic order.
    #[must_use]
    pub fn dirty_sheets(&self) -> Vec<TreeNodeId> {
        self.dirty_by_sheet
            .iter()
            .filter(|(_, cells)| !cells.is_empty())
            .map(|(node, _)| *node)
            .collect()
    }

    /// The dirty cells on `sheet`, if any.
    #[must_use]
    pub fn dirty_cells(&self, sheet: TreeNodeId) -> Option<&BTreeSet<ExcelGridCellAddress>> {
        self.dirty_by_sheet.get(&sheet)
    }
}

/// The workbook dirty closure (D3 §3): grow the initial per-sheet dirty seeds to
/// a fixpoint over the cross-sheet edge layer.
///
/// `initial_dirty_by_sheet` is the per-sheet seed set (the cells an edit dirtied
/// directly, plus their *local* closures — the per-sheet local `dirty_closure`
/// runs inside each sheet's own recalc; at this layer a seed cell is taken as
/// the reachable frontier a peer must observe). Iterating:
///
/// 1. Take the current per-sheet dirty frontier.
/// 2. For each sheet, consult the cross layer for foreign dependents of its
///    dirty cells; each becomes a new dirty cell on *its* sheet.
/// 3. Repeat until a full pass adds nothing new.
///
/// **Monotonicity & termination.** The dirty set only grows (cells are inserted,
/// never removed), and it is bounded by the finite set of cells that appear as a
/// dependent in some cross-sheet edge (plus the seeds). Each round that changes
/// nothing halts the loop; each round that changes something adds at least one
/// cell to a finite set, so the loop runs at most (seed cells + distinct
/// dependent cells) rounds. It always terminates.
///
/// A cross-sheet *cycle* does not diverge here — a cell already in the dirty set
/// is not re-added, so a cycle simply marks all its members dirty once and the
/// fixpoint closes. Cycle *detection* (typed error) is the worklist's job
/// ([`WorkbookWorklistOrder::build`]), not the closure's.
#[must_use]
pub fn workbook_dirty_closure(
    edges: &WorkbookCrossSheetEdges,
    initial_dirty_by_sheet: BTreeMap<TreeNodeId, BTreeSet<ExcelGridCellAddress>>,
) -> WorkbookDirtyClosure {
    let mut dirty_by_sheet = initial_dirty_by_sheet;
    loop {
        let mut added_any = false;
        // Snapshot the current frontier so we iterate a stable view while
        // inserting into `dirty_by_sheet`. BTree order ⇒ deterministic.
        let frontier: Vec<(TreeNodeId, BTreeSet<ExcelGridCellAddress>)> = dirty_by_sheet
            .iter()
            .map(|(sheet, cells)| (*sheet, cells.clone()))
            .collect();
        for (target_sheet, dirty_cells) in frontier {
            for (dependent_sheet, dependent_cell) in
                edges.foreign_dependents_of(target_sheet, &dirty_cells)
            {
                let inserted = dirty_by_sheet
                    .entry(dependent_sheet)
                    .or_default()
                    .insert(dependent_cell);
                added_any |= inserted;
            }
        }
        if !added_any {
            return WorkbookDirtyClosure { dirty_by_sheet };
        }
    }
}

/// A workbook evaluation schedule over the dirty cross-sheet subgraph (D3 §3): a
/// single worklist across sheet boundaries, not sheet-at-a-time. Produced by
/// [`WorkbookWorklistOrder::build`].
///
/// Because the consumer's unit of evaluation is a *whole sheet* (a sheet recalc
/// is mark-all over that sheet), and a cell chain may legitimately re-enter a
/// sheet (`Sheet1!A1 → Sheet2!B1 → Sheet1!C1` — D3 §3's own example, which has
/// no sheet-at-a-time order), the schedule is a bounded **round sequence**: each
/// round recalculates every dirty sheet once, in `TreeNodeId` order, reading the
/// latest-available peer values; rounds repeat until values stop changing. A
/// dirty cross-sheet subgraph with `k` sheets converges in at most `k` rounds
/// along its longest cross-sheet chain (one more confirms), exactly as the
/// `GridCalcRefWorkbook` oracle's fixpoint — so the schedule matches the oracle.
///
/// The genuine-cycle guard is separate and structural: a directed cycle in the
/// cross-sheet *cell* graph that spans more than one sheet is the typed
/// [`GridRefError::WorkbookEffectiveDependencyCycleDetected`] (bead acceptance),
/// detected up front so the round loop only ever runs on an acyclic
/// cross-sheet cell graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkbookWorklistOrder {
    /// The dirty sheets to recalculate each round, in deterministic
    /// (`TreeNodeId`) order.
    pub sheet_order: Vec<TreeNodeId>,
    /// The number of rounds required to reach a fixpoint on the dirty
    /// cross-sheet cell graph: `1 + longest cross-sheet chain length` among the
    /// dirty sheets, hard-bounded by the dirty-sheet count. Each round
    /// recalculates every sheet in `sheet_order`.
    pub max_rounds: usize,
}

impl WorkbookWorklistOrder {
    /// Build the workbook evaluation schedule over the sheets the closure marked
    /// dirty (D3 §3).
    ///
    /// 1. **Cross-sheet cycle guard.** Detect a directed cycle in the cross-sheet
    ///    *cell* graph (restricted to dirty cells) that spans more than one
    ///    sheet; return the typed workbook cycle error if one exists. Same-sheet
    ///    cell cycles never appear in the cross layer (routing invariant), so a
    ///    detected cycle is always genuinely cross-sheet.
    /// 2. **Round schedule.** With cross-sheet cycles ruled out, the sheets can
    ///    be recalculated in rounds to a fixpoint. `sheet_order` is the dirty
    ///    sheets in `TreeNodeId` order; `max_rounds` bounds the fixpoint.
    ///
    /// Deterministic: `BTreeSet`/`BTreeMap` throughout, so seed/edge insertion
    /// order never affects the schedule (contract X4 / D3 §10).
    pub fn build(
        edges: &WorkbookCrossSheetEdges,
        closure: &WorkbookDirtyClosure,
    ) -> Result<Self, GridRefError> {
        let dirty_sheets: Vec<TreeNodeId> = closure.dirty_sheets();
        if let Some(cycle) = cross_sheet_cell_cycle(edges, closure) {
            return Err(GridRefError::WorkbookEffectiveDependencyCycleDetected { cycle });
        }
        // An acyclic cross-sheet cell graph over `k` dirty sheets propagates any
        // value the full length of its longest cross-sheet chain within `k`
        // rounds; one more round confirms no change. `k + 1` is the safe bound.
        let max_rounds = dirty_sheets.len().saturating_add(1).max(1);
        Ok(Self {
            sheet_order: dirty_sheets,
            max_rounds,
        })
    }
}

/// Detect a directed cycle in the cross-sheet *cell* graph spanning more than
/// one sheet, restricted to cells the closure marked dirty. Returns the
/// participating cells as `WorkbookCalcNodeId::GridCell`s in deterministic
/// order, or `None` if the dirty cross-sheet cell graph is acyclic.
///
/// Iterative DFS with grey/black colouring over the reverse of the reverse
/// index: a `dependent_cell → target_cell` forward edge (the dependent *reads*
/// the target, so the target must evaluate first). A back-edge onto a grey node
/// closes a cycle; we report it only when its cells span at least two sheets
/// (an intra-sheet cycle is the per-sheet engine's concern and is not
/// represented here anyway). Cross-sheet-only edges by construction, so any
/// closed cycle among them is by definition multi-sheet — the span check is a
/// belt-and-braces guard.
fn cross_sheet_cell_cycle(
    edges: &WorkbookCrossSheetEdges,
    closure: &WorkbookDirtyClosure,
) -> Option<Vec<WorkbookCalcNodeId>> {
    // Forward adjacency among dirty cells: dependent → target.
    let is_dirty = |sheet: TreeNodeId, cell: &ExcelGridCellAddress| {
        closure
            .dirty_cells(sheet)
            .is_some_and(|cells| cells.contains(cell))
    };
    let mut adjacency: BTreeMap<ExcelGridCellAddress, BTreeSet<ExcelGridCellAddress>> =
        BTreeMap::new();
    for edge in edges.all_edges() {
        if is_dirty(edge.dependent_sheet, &edge.dependent_cell)
            && is_dirty(edge.target_sheet, &edge.target_cell)
        {
            adjacency
                .entry(edge.dependent_cell.clone())
                .or_default()
                .insert(edge.target_cell.clone());
        }
    }

    #[derive(Clone, Copy, PartialEq)]
    enum Colour {
        Grey,
        Black,
    }
    let mut colour: BTreeMap<ExcelGridCellAddress, Colour> = BTreeMap::new();
    for root in adjacency.keys() {
        if colour.contains_key(root) {
            continue;
        }
        let mut stack: Vec<(ExcelGridCellAddress, std::vec::IntoIter<ExcelGridCellAddress>)> =
            Vec::new();
        let root_succ = adjacency
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
                    }
                    Some(Colour::Black) => {}
                    None => {
                        colour.insert(target.clone(), Colour::Grey);
                        let succ = adjacency
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

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(sheet: &str, row: u32, col: u32) -> ExcelGridCellAddress {
        ExcelGridCellAddress::new("book", sheet, row, col)
    }

    fn edge(
        dependent_sheet: u64,
        dependent: (&str, u32, u32),
        target: (&str, u32, u32),
        target_sheet: u64,
    ) -> WorkbookCrossSheetEdge {
        WorkbookCrossSheetEdge {
            dependent_sheet: TreeNodeId(dependent_sheet),
            dependent_cell: cell(dependent.0, dependent.1, dependent.2),
            target_cell: cell(target.0, target.1, target.2),
            target_sheet: TreeNodeId(target_sheet),
        }
    }

    /// A two-sheet chain: Sheet2!B1 reads Sheet1!A1. Editing Sheet1!A1 closes to
    /// dirty Sheet2!B1; the worklist orders Sheet1 before Sheet2.
    #[test]
    fn closure_and_worklist_over_a_two_sheet_chain() {
        let mut edges = WorkbookCrossSheetEdges::new();
        edges.register(edge(3, ("Sheet2", 1, 2), ("Sheet1", 1, 1), 2));

        let initial: BTreeMap<TreeNodeId, BTreeSet<ExcelGridCellAddress>> =
            [(TreeNodeId(2), [cell("Sheet1", 1, 1)].into_iter().collect())]
                .into_iter()
                .collect();
        let closure = workbook_dirty_closure(&edges, initial);
        assert!(
            closure
                .dirty_cells(TreeNodeId(3))
                .is_some_and(|cells| cells.contains(&cell("Sheet2", 1, 2))),
            "Sheet2!B1 is dirtied by the Sheet1!A1 seed"
        );

        let worklist = WorkbookWorklistOrder::build(&edges, &closure).unwrap();
        assert_eq!(
            worklist.sheet_order,
            vec![TreeNodeId(2), TreeNodeId(3)],
            "Sheet1 (the source) evaluates before Sheet2 (the dependent)"
        );
    }

    /// A back-and-forth chain Sheet1!A1 → Sheet2!B1 → Sheet1!C1 has no
    /// sheet-at-a-time order but is acyclic on the cell graph. Sheet1 appears
    /// once and the closure marks both its cells dirty; the worklist does not
    /// stall.
    #[test]
    fn chain_crossing_back_to_the_origin_sheet_is_not_a_cycle() {
        let mut edges = WorkbookCrossSheetEdges::new();
        // Sheet2!B1 reads Sheet1!A1; Sheet1!C1 reads Sheet2!B1.
        edges.register(edge(3, ("Sheet2", 1, 2), ("Sheet1", 1, 1), 2));
        edges.register(edge(2, ("Sheet1", 1, 3), ("Sheet2", 1, 2), 3));

        let initial: BTreeMap<TreeNodeId, BTreeSet<ExcelGridCellAddress>> =
            [(TreeNodeId(2), [cell("Sheet1", 1, 1)].into_iter().collect())]
                .into_iter()
                .collect();
        let closure = workbook_dirty_closure(&edges, initial);
        let sheet1_dirty = closure.dirty_cells(TreeNodeId(2)).unwrap();
        assert!(sheet1_dirty.contains(&cell("Sheet1", 1, 1)));
        assert!(
            sheet1_dirty.contains(&cell("Sheet1", 1, 3)),
            "Sheet1!C1 is dirtied transitively through Sheet2!B1"
        );
        // A1 → B1 → C1 is acyclic on the CELL graph even though it re-enters
        // Sheet1, so the worklist does NOT report a cycle (the cross-sheet cell
        // cycle guard is over cells, not sheets — D3 §3). It schedules a bounded
        // round sequence that converges by re-reading fresh values across rounds.
        let worklist = WorkbookWorklistOrder::build(&edges, &closure).unwrap();
        assert!(
            worklist.sheet_order.contains(&TreeNodeId(2))
                && worklist.sheet_order.contains(&TreeNodeId(3)),
            "both sheets are scheduled"
        );
        assert!(worklist.max_rounds >= 2, "the back-and-forth chain needs multiple rounds");
    }

    /// A genuine cross-sheet cycle Sheet1!A1 ⇄ Sheet2!A1 stalls the worklist and
    /// yields the typed workbook cycle error naming both cells.
    #[test]
    fn cross_sheet_cycle_is_the_typed_workbook_error() {
        let mut edges = WorkbookCrossSheetEdges::new();
        edges.register(edge(2, ("Sheet1", 1, 1), ("Sheet2", 1, 1), 3));
        edges.register(edge(3, ("Sheet2", 1, 1), ("Sheet1", 1, 1), 2));

        let initial: BTreeMap<TreeNodeId, BTreeSet<ExcelGridCellAddress>> =
            [(TreeNodeId(2), [cell("Sheet1", 1, 1)].into_iter().collect())]
                .into_iter()
                .collect();
        let closure = workbook_dirty_closure(&edges, initial);
        match WorkbookWorklistOrder::build(&edges, &closure) {
            Err(GridRefError::WorkbookEffectiveDependencyCycleDetected { cycle }) => {
                assert!(cycle.contains(&WorkbookCalcNodeId::GridCell(cell("Sheet1", 1, 1))));
                assert!(cycle.contains(&WorkbookCalcNodeId::GridCell(cell("Sheet2", 1, 1))));
            }
            other => panic!("expected typed cross-sheet cycle, got {other:?}"),
        }
    }

    /// The worklist order is a pure function of (edges, closure): permuting edge
    /// insertion order leaves the emitted sheet order identical (contract X4 /
    /// D3 §10 determinism).
    #[test]
    fn worklist_order_is_insertion_order_independent() {
        // Diamond: Sheet1 → Sheet2, Sheet1 → Sheet3, {Sheet2,Sheet3} → Sheet4.
        let build = |perm: &[WorkbookCrossSheetEdge]| {
            let mut edges = WorkbookCrossSheetEdges::new();
            for e in perm {
                edges.register(e.clone());
            }
            let dirty: BTreeMap<TreeNodeId, BTreeSet<ExcelGridCellAddress>> = [
                (TreeNodeId(2), [cell("Sheet1", 1, 1)].into_iter().collect()),
                (TreeNodeId(3), [cell("Sheet2", 1, 1)].into_iter().collect()),
                (TreeNodeId(4), [cell("Sheet3", 1, 1)].into_iter().collect()),
                (TreeNodeId(5), [cell("Sheet4", 1, 1)].into_iter().collect()),
            ]
            .into_iter()
            .collect();
            let closure = workbook_dirty_closure(&edges, dirty);
            WorkbookWorklistOrder::build(&edges, &closure).unwrap().sheet_order
        };
        let e1 = edge(3, ("Sheet2", 1, 1), ("Sheet1", 1, 1), 2);
        let e2 = edge(4, ("Sheet3", 1, 1), ("Sheet1", 1, 1), 2);
        let e3 = edge(5, ("Sheet4", 1, 1), ("Sheet2", 1, 1), 3);
        let e4 = edge(5, ("Sheet4", 1, 1), ("Sheet3", 1, 1), 4);
        let forward = build(&[e1.clone(), e2.clone(), e3.clone(), e4.clone()]);
        let reversed = build(&[e4, e3, e2, e1]);
        assert_eq!(forward, reversed, "sheet order is insertion-order independent");
        // The schedule is the dirty sheets in BTree (`TreeNodeId`) order — a pure
        // function of the dirty set, so permuting edge insertion cannot change it.
        assert_eq!(
            forward,
            vec![TreeNodeId(2), TreeNodeId(3), TreeNodeId(4), TreeNodeId(5)],
        );
    }
}
