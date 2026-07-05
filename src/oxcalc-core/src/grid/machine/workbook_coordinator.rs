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

/// One cross-sheet **name** reverse edge (W062 D3 §2.2, R4.11): a dependent cell
/// on `dependent_sheet` resolves a defined name whose authoritative binding lives
/// on `defining_sheet`. Stored keyed by the *defining* sheet + name key so that
/// when that name's binding or resolved value changes (a redefinition, a shadow,
/// a heal, or a value change on the target the name points at), its foreign
/// dependents are found in one lookup — the exact analogue of the cell lane, one
/// resolution level up.
///
/// `name_key` is the engine-normalized defined-name key (the same `String` the
/// per-sheet [`GridDependency::Name`]/[`GridDependency::NameIdentity`] carry).
/// The scope qualification (workbook vs sheet) is carried by *which* defining
/// sheet the edge is keyed under: a workbook-scoped name is authoritative on the
/// sheet its `define_name` verb targeted; a sheet-scoped name is authoritative on
/// its own sheet. This is [`ScopedNameKey`] made concrete at the edge level —
/// `(defining_sheet, name_key)` *is* the resolved scope key D3 §2.2 stores.
///
/// **Consumption status (W062 R4.11).** The scoped-name cross-sheet *behavior* the
/// bead ships — a workbook-scoped name visible from every sheet, shadowed by a
/// sheet-scoped name, healed on delete, re-driven on a target-value edit — is
/// delivered by the consumer's value-anchor **projection**
/// (`register_cross_sheet_workbook_names_into_grids`): a workbook name's resolved
/// value is projected into every peer grid's name namespace, so a peer `=N`
/// resolves through its own engine's native `NameIdentity` heal machinery and the
/// ordinary cell edge/closure carries the cross-sheet value edit. This edge lane is
/// the *explicit graph form* of that same relation — the D3 §2.2
/// `ScopedNameKey`-aware cross-layer edge — landed as the first-class representation
/// (variant + registration + closure seam in [`workbook_dirty_closure_with_names`]
/// + deterministic unit tests) for the closure to route on directly, exactly as the
/// [`GridDependency::SheetSpan`] variant landed ahead of its R4.12 closure
/// consumption. The projection is the correct honest slice today; a later refinement
/// routes the closure on this lane to make the name edge O(name-cone) rather than
/// re-projecting eagerly. It is not dead code: its registration/lookup/closure
/// contract is pinned by tests and is the shape that refinement builds on.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorkbookCrossSheetNameEdge {
    /// The sheet node the dependent (name-reading) cell lives on.
    pub dependent_sheet: TreeNodeId,
    /// The dependent (name-reading) cell.
    pub dependent_cell: ExcelGridCellAddress,
    /// The engine-normalized defined-name key the dependent resolves.
    pub name_key: String,
    /// The sheet node whose name namespace authoritatively binds `name_key`.
    pub defining_sheet: TreeNodeId,
}

/// The cross-sheet edge layer (D3 §1 `cross`): a reverse index from a target
/// sheet node to the foreign dependent cells that read cells on it, plus (R4.11,
/// D3 §2.2) a parallel reverse index from a `(defining sheet, name key)` to the
/// foreign dependent cells that resolve that name.
///
/// Deterministic throughout: `BTreeMap`/`BTreeSet` keyed on `TreeNodeId` /
/// `ExcelGridCellAddress` / `String`, so enumeration and closure order never
/// depend on insertion order or hashing (contract X4 / D3 §10).
///
/// The **cell** lane carries both statically-authored cross-sheet cell/range
/// reads and runtime-realized cross-sheet reads (an `INDIRECT("Sheet2!A1")`
/// resolves to a cross-sheet [`GridDependency::Cell`] that registers here exactly
/// like a static one — this is the D3 §2.2 dynamic-request cross-sheet path: the
/// realized dependency is a sheet-qualified cell edge in the cross layer, so
/// editing the target sheet's cell recalculates the dynamic dependent). The
/// **name** lane carries scoped-name / table cross-sheet resolution, so a name
/// redefinition, shadow, or heal on the defining sheet dirties its foreign
/// dependents.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkbookCrossSheetEdges {
    /// target sheet → the cross-sheet edges whose target lives on it.
    by_target_sheet: BTreeMap<TreeNodeId, BTreeSet<WorkbookCrossSheetEdge>>,
    /// Every registered cell edge, for enumeration and the worklist build.
    all_edges: BTreeSet<WorkbookCrossSheetEdge>,
    /// (defining sheet, name key) → the cross-sheet name edges keyed on it.
    by_defining_name: BTreeMap<(TreeNodeId, String), BTreeSet<WorkbookCrossSheetNameEdge>>,
    /// Every registered name edge, for enumeration.
    all_name_edges: BTreeSet<WorkbookCrossSheetNameEdge>,
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

    /// Register one cross-sheet **name** edge (W062 D3 §2.2, R4.11): a dependent
    /// cell on one sheet resolves a defined name authoritatively bound on
    /// `defining_sheet`. A same-sheet name resolution never enters here — the
    /// per-sheet index already carries it (the routing invariant, one level up) —
    /// so an edge whose dependent and defining sheet coincide is dropped rather
    /// than mis-filed, keeping the cross-only invariant true by construction.
    pub fn register_name_edge(&mut self, edge: WorkbookCrossSheetNameEdge) {
        if edge.dependent_sheet == edge.defining_sheet {
            return;
        }
        self.by_defining_name
            .entry((edge.defining_sheet, edge.name_key.clone()))
            .or_default()
            .insert(edge.clone());
        self.all_name_edges.insert(edge);
    }

    /// The foreign dependent cells that resolve any of `dirty_name_keys` bound on
    /// `defining_sheet` (W062 D3 §2.2, R4.11). When a name's binding or resolved
    /// value changes on the sheet that owns it, this names the cross-sheet
    /// dependents that must re-resolve, in deterministic order.
    #[must_use]
    pub fn foreign_dependents_of_names(
        &self,
        defining_sheet: TreeNodeId,
        dirty_name_keys: &BTreeSet<String>,
    ) -> BTreeSet<(TreeNodeId, ExcelGridCellAddress)> {
        let mut dependents = BTreeSet::new();
        for name_key in dirty_name_keys {
            let Some(edges) = self
                .by_defining_name
                .get(&(defining_sheet, name_key.clone()))
            else {
                continue;
            };
            for edge in edges {
                dependents.insert((edge.dependent_sheet, edge.dependent_cell.clone()));
            }
        }
        dependents
    }

    /// Every registered name edge, in deterministic order.
    #[must_use]
    pub fn all_name_edges(&self) -> &BTreeSet<WorkbookCrossSheetNameEdge> {
        &self.all_name_edges
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

/// One authored 3D sheet-span dependency, as the span index consumes it (W062
/// R4.12): the dependent (span-reading) cell plus the stored span edge. This is
/// the input to the index — the ONE stored edge, never a materialized fan.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorkbookSheetSpanDependent {
    /// The sheet node the span-reading cell lives on.
    pub dependent_sheet: TreeNodeId,
    /// The span-reading cell (e.g. `Sheet4!A1 = SUM(Sheet1:Sheet3!A1)`).
    pub dependent_cell: ExcelGridCellAddress,
    /// The stored span edge (endpoints + sheet-agnostic target).
    pub span: GridSheetSpanDependency,
}

/// The **derived span-interval index** (W062 D3 §2.3, D2 §4.2 / V5): a small
/// structure over the D1 sheet-registry order mapping each member sheet position
/// to the span-reading cells whose span currently covers it.
///
/// This is the resolution of the "one stored edge vs closure-time member
/// enumeration" tension: the graph stores exactly ONE
/// [`GridDependency::SheetSpan`] edge per span (never a materialized per-sheet
/// fan), and this index — rebuilt on sheet-lifecycle edits (cheap: spans are
/// few, lifecycle edits are rare) — makes closure walks an interval probe per
/// dirtied sheet rather than a member enumeration:
///
/// - **Closure integration** ([`Self::span_dependents_of_sheet`]): an edit to any
///   member sheet's covered cells dirties the span's dependents — a single
///   lookup by the dirtied member sheet node, no per-span rescan.
/// - **Membership-change invalidation** ([`Self::membership_change_dependents`]):
///   inserting/moving/deleting a sheet inside a span's interval changes which
///   sheets it covers with NO content edit; that change falls out of the index
///   *rebuild diff* — a span-reading cell whose covered-sheet set differs between
///   the old and new index must re-evaluate.
///
/// Built purely from the registered span dependents + the catalog's current C3
/// order via [`WorkbookReferenceCatalog::sheet_span_member_nodes`] — a pure
/// function of the two, so it rebuilds deterministically after any lifecycle
/// edit rebuilds the catalog.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkbookSheetSpanIndex {
    /// member sheet node → the span-reading cells whose span currently covers it.
    /// The closure-time interval probe: dirty a member sheet's cells, find the
    /// span dependents in one lookup.
    by_member_sheet: BTreeMap<TreeNodeId, BTreeSet<(TreeNodeId, ExcelGridCellAddress)>>,
    /// span-reading cell → the ordered member nodes its span currently covers.
    /// The membership-diff basis: comparing this across a rebuild yields the
    /// dependents whose coverage changed (no content edit).
    coverage_by_dependent: BTreeMap<(TreeNodeId, ExcelGridCellAddress), Vec<TreeNodeId>>,
}

impl WorkbookSheetSpanIndex {
    /// An empty span index (a workbook with no 3D span references).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the derived span-interval index from the registered span dependents
    /// and the catalog's current C3 sheet order (W062 D3 §2.3).
    ///
    /// Each span dependent's endpoints are probed against the *current* registry
    /// ([`WorkbookReferenceCatalog::sheet_span_member_nodes`]); the resulting
    /// member nodes are recorded both forward (member → dependents, for closure)
    /// and per-dependent (dependent → covered members, for the membership diff).
    /// A span whose endpoints no longer resolve (dangling) contributes no member
    /// coverage — its dependent reads an empty aggregation / `#REF!` handled
    /// elsewhere — but is still recorded with an empty coverage so a later
    /// re-resolution shows up as a membership change.
    #[must_use]
    pub fn build<I>(catalog: &WorkbookReferenceCatalog, dependents: I) -> Self
    where
        I: IntoIterator<Item = WorkbookSheetSpanDependent>,
    {
        let mut by_member_sheet: BTreeMap<TreeNodeId, BTreeSet<(TreeNodeId, ExcelGridCellAddress)>> =
            BTreeMap::new();
        let mut coverage_by_dependent: BTreeMap<
            (TreeNodeId, ExcelGridCellAddress),
            Vec<TreeNodeId>,
        > = BTreeMap::new();
        for dependent in dependents {
            let key = (dependent.dependent_sheet, dependent.dependent_cell.clone());
            let members = catalog
                .sheet_span_member_nodes(&dependent.span.start_sheet, &dependent.span.end_sheet)
                .unwrap_or_default();
            for member in &members {
                by_member_sheet
                    .entry(*member)
                    .or_default()
                    .insert(key.clone());
            }
            // Last write wins if a dependent cell authored two spans — coverage
            // is the union across its spans. Merge rather than overwrite.
            coverage_by_dependent
                .entry(key)
                .or_default()
                .extend(members);
        }
        // Normalize each coverage vector (dedup + sorted) so the membership diff
        // compares canonical member sets.
        for coverage in coverage_by_dependent.values_mut() {
            coverage.sort_unstable();
            coverage.dedup();
        }
        Self {
            by_member_sheet,
            coverage_by_dependent,
        }
    }

    /// The span-reading cells whose span currently covers `member_sheet` (W062
    /// D3 §2.3 closure integration). An edit to `member_sheet`'s covered cells
    /// dirties exactly these dependents — the interval probe, one lookup.
    #[must_use]
    pub fn span_dependents_of_sheet(
        &self,
        member_sheet: TreeNodeId,
    ) -> BTreeSet<(TreeNodeId, ExcelGridCellAddress)> {
        self.by_member_sheet
            .get(&member_sheet)
            .cloned()
            .unwrap_or_default()
    }

    /// The span-reading cells whose covered-sheet set differs between `self` (the
    /// pre-edit index) and `rebuilt` (the post-lifecycle-edit index) — W062 D3
    /// §2.3 membership-change invalidation.
    ///
    /// Inserting/moving/deleting a sheet inside a span's interval changes which
    /// sheets the span covers with NO content edit; those dependents must
    /// re-evaluate. This is the *rebuild diff*: a dependent present in either
    /// index whose coverage vector changed (including gaining/losing all
    /// coverage) is returned. Deterministic (`BTreeSet`/sorted vectors).
    #[must_use]
    pub fn membership_change_dependents(
        &self,
        rebuilt: &Self,
    ) -> BTreeSet<(TreeNodeId, ExcelGridCellAddress)> {
        let mut changed = BTreeSet::new();
        let keys = self
            .coverage_by_dependent
            .keys()
            .chain(rebuilt.coverage_by_dependent.keys());
        for key in keys {
            let before = self.coverage_by_dependent.get(key);
            let after = rebuilt.coverage_by_dependent.get(key);
            if before != after {
                changed.insert(key.clone());
            }
        }
        changed
    }

    /// Whether the index holds no span dependents (a workbook with no 3D spans).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.coverage_by_dependent.is_empty()
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
    workbook_dirty_closure_with_names(edges, initial_dirty_by_sheet, BTreeMap::new())
}

/// The workbook dirty closure with a **scoped-name** seed lane (W062 D3 §2.2,
/// R4.11): identical to [`workbook_dirty_closure`], but additionally takes, per
/// defining sheet, the set of defined-name keys whose binding or resolved value
/// changed. Each such name's foreign dependents (cross-sheet name edges) become
/// dirty cells on their own sheets in the very first pass, and from there the
/// fixpoint proceeds exactly as the cell-only closure does.
///
/// The name seeds only *inject* dirty cells; the fixpoint itself is still over
/// the cell lane (names resolve to cells, and a cell that becomes dirty because
/// it read a re-bound name is thereafter an ordinary dirty cell). Termination and
/// monotonicity are unchanged: the name lane adds a bounded, one-shot frontier of
/// dependent cells before the cell fixpoint runs, and the cell fixpoint's own
/// bound is untouched.
#[must_use]
pub fn workbook_dirty_closure_with_names(
    edges: &WorkbookCrossSheetEdges,
    initial_dirty_by_sheet: BTreeMap<TreeNodeId, BTreeSet<ExcelGridCellAddress>>,
    initial_dirty_names_by_sheet: BTreeMap<TreeNodeId, BTreeSet<String>>,
) -> WorkbookDirtyClosure {
    let mut dirty_by_sheet = initial_dirty_by_sheet;
    // Fold the name-seed frontier into dirty cells before the cell fixpoint.
    // A name re-bound on its defining sheet dirties every foreign dependent cell
    // that resolves it; those dependent cells then participate in the cell
    // fixpoint like any other dirty cell.
    for (defining_sheet, name_keys) in &initial_dirty_names_by_sheet {
        for (dependent_sheet, dependent_cell) in
            edges.foreign_dependents_of_names(*defining_sheet, name_keys)
        {
            dirty_by_sheet
                .entry(dependent_sheet)
                .or_default()
                .insert(dependent_cell);
        }
    }
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

/// The workbook dirty closure with the 3D-span-interval index folded in (W062
/// R4.12, D3 §2.3). Extends [`workbook_dirty_closure_with_names`] with the span
/// closure integration: whenever a member sheet gains dirty cells, the span
/// index's interval probe ([`WorkbookSheetSpanIndex::span_dependents_of_sheet`])
/// names the span-reading cells that must re-aggregate, and they enter the cell
/// fixpoint like any other dirty cell.
///
/// `membership_change_dependents` are the span-reading cells whose covered-sheet
/// set changed due to a sheet-lifecycle edit (the index rebuild diff): they are
/// seeded up front with NO content edit, which is exactly the acceptance
/// criterion "inserting a sheet inside Sheet1:Sheet3 dirties span dependents
/// with no content edit". Pass an empty set for a pure content edit.
#[must_use]
pub fn workbook_dirty_closure_with_spans(
    edges: &WorkbookCrossSheetEdges,
    span_index: &WorkbookSheetSpanIndex,
    initial_dirty_by_sheet: BTreeMap<TreeNodeId, BTreeSet<ExcelGridCellAddress>>,
    initial_dirty_names_by_sheet: BTreeMap<TreeNodeId, BTreeSet<String>>,
    membership_change_dependents: BTreeSet<(TreeNodeId, ExcelGridCellAddress)>,
) -> WorkbookDirtyClosure {
    let mut dirty_by_sheet = initial_dirty_by_sheet;
    // Membership-change invalidation (D3 §2.3): a sheet entering/leaving a span's
    // interval dirties that span's dependents with no content edit. Seed them
    // before the fixpoint so their own downstream cones close too.
    for (dependent_sheet, dependent_cell) in membership_change_dependents {
        dirty_by_sheet
            .entry(dependent_sheet)
            .or_default()
            .insert(dependent_cell);
    }
    // Name-seed frontier (R4.11), unchanged.
    for (defining_sheet, name_keys) in &initial_dirty_names_by_sheet {
        for (dependent_sheet, dependent_cell) in
            edges.foreign_dependents_of_names(*defining_sheet, name_keys)
        {
            dirty_by_sheet
                .entry(dependent_sheet)
                .or_default()
                .insert(dependent_cell);
        }
    }
    loop {
        let mut added_any = false;
        let frontier: Vec<(TreeNodeId, BTreeSet<ExcelGridCellAddress>)> = dirty_by_sheet
            .iter()
            .map(|(sheet, cells)| (*sheet, cells.clone()))
            .collect();
        for (target_sheet, dirty_cells) in frontier {
            // Ordinary cross-sheet cell dependents.
            for (dependent_sheet, dependent_cell) in
                edges.foreign_dependents_of(target_sheet, &dirty_cells)
            {
                let inserted = dirty_by_sheet
                    .entry(dependent_sheet)
                    .or_default()
                    .insert(dependent_cell);
                added_any |= inserted;
            }
            // Span dependents: any span covering this now-dirty member sheet
            // must re-aggregate. The interval probe is a single index lookup —
            // no member enumeration, no per-span rescan (D3 §2.3).
            if !dirty_cells.is_empty() {
                for (dependent_sheet, dependent_cell) in
                    span_index.span_dependents_of_sheet(target_sheet)
                {
                    let inserted = dirty_by_sheet
                        .entry(dependent_sheet)
                        .or_default()
                        .insert(dependent_cell);
                    added_any |= inserted;
                }
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

    /// W062 R4.14 (D3 §10 constraint 3, deterministic worklist): permuting the
    /// *seed insertion order* leaves both the closure (dirty cells per sheet)
    /// and the emitted worklist order byte-identical. This pins the constraint
    /// the design names explicitly — a future concurrent executor (W053) must
    /// reproduce the sequential worklist's observable schedule and reports
    /// regardless of the order seeds were discovered/inserted.
    #[test]
    fn worklist_and_closure_are_seed_insertion_order_independent() {
        // Diamond dirty cone: seeds on Sheet1..Sheet4 dirtied through the
        // cross-sheet edges. The seeds are fed in two opposite insertion orders;
        // both the closure and the schedule must match exactly.
        let mut edges = WorkbookCrossSheetEdges::new();
        edges.register(edge(3, ("Sheet2", 1, 1), ("Sheet1", 1, 1), 2));
        edges.register(edge(4, ("Sheet3", 1, 1), ("Sheet1", 1, 1), 2));
        edges.register(edge(5, ("Sheet4", 1, 1), ("Sheet2", 1, 1), 3));
        edges.register(edge(5, ("Sheet4", 1, 1), ("Sheet3", 1, 1), 4));

        // The seeds as (sheet, cell) pairs, inserted into the `BTreeMap` in a
        // caller-chosen order. Determinism must not depend on that order.
        let seeds = [
            (TreeNodeId(2), cell("Sheet1", 1, 1)),
            (TreeNodeId(3), cell("Sheet2", 1, 1)),
            (TreeNodeId(4), cell("Sheet3", 1, 1)),
            (TreeNodeId(5), cell("Sheet4", 1, 1)),
        ];
        let build = |order: &[usize]| {
            let mut initial: BTreeMap<TreeNodeId, BTreeSet<ExcelGridCellAddress>> =
                BTreeMap::new();
            for &i in order {
                let (sheet, addr) = &seeds[i];
                initial.entry(*sheet).or_default().insert(addr.clone());
            }
            let closure = workbook_dirty_closure(&edges, initial);
            // Capture the full closure as a comparable, order-stable structure
            // (the "identical reports" half of the constraint) alongside the
            // worklist order.
            let dirty: BTreeMap<TreeNodeId, BTreeSet<ExcelGridCellAddress>> = closure
                .dirty_sheets()
                .into_iter()
                .map(|sheet| {
                    let cells = closure
                        .dirty_cells(sheet)
                        .cloned()
                        .unwrap_or_default();
                    (sheet, cells)
                })
                .collect();
            let order = WorkbookWorklistOrder::build(&edges, &closure).unwrap();
            (dirty, order.sheet_order, order.max_rounds)
        };

        let forward = build(&[0, 1, 2, 3]);
        let reversed = build(&[3, 2, 1, 0]);
        let shuffled = build(&[2, 0, 3, 1]);
        assert_eq!(
            forward, reversed,
            "closure + worklist are identical under reversed seed insertion"
        );
        assert_eq!(
            forward, shuffled,
            "closure + worklist are identical under shuffled seed insertion"
        );
        assert_eq!(
            forward.1,
            vec![TreeNodeId(2), TreeNodeId(3), TreeNodeId(4), TreeNodeId(5)],
            "the schedule is the dirty sheets in BTree order"
        );
    }

    // --- W062 R4.11 — scoped-name cross-sheet edge lane (D3 §2.2) ----------

    fn name_edge(
        dependent_sheet: u64,
        dependent: (&str, u32, u32),
        name_key: &str,
        defining_sheet: u64,
    ) -> WorkbookCrossSheetNameEdge {
        WorkbookCrossSheetNameEdge {
            dependent_sheet: TreeNodeId(dependent_sheet),
            dependent_cell: cell(dependent.0, dependent.1, dependent.2),
            name_key: name_key.to_string(),
            defining_sheet: TreeNodeId(defining_sheet),
        }
    }

    /// A cross-sheet name edge (a `=N` on Sheet2 resolving a name bound on
    /// Sheet1) is found when that name's key is dirtied on its defining sheet.
    #[test]
    fn name_edge_finds_foreign_dependents_of_a_dirtied_name() {
        let mut edges = WorkbookCrossSheetEdges::new();
        // Sheet2!A1 resolves name "n" bound authoritatively on Sheet1 (node 2).
        edges.register_name_edge(name_edge(3, ("Sheet2", 1, 1), "n", 2));

        let dirty_names: BTreeSet<String> = ["n".to_string()].into_iter().collect();
        let dependents = edges.foreign_dependents_of_names(TreeNodeId(2), &dirty_names);
        assert!(
            dependents.contains(&(TreeNodeId(3), cell("Sheet2", 1, 1))),
            "the name redefinition on Sheet1 finds the Sheet2 dependent"
        );
        // A name dirtied on a DIFFERENT defining sheet does not match.
        assert!(
            edges
                .foreign_dependents_of_names(TreeNodeId(4), &dirty_names)
                .is_empty(),
            "the same name key on a different defining sheet is a different scope key"
        );
    }

    /// A same-sheet name resolution never enters the cross layer (routing
    /// invariant); a dropped same-sheet name edge keeps the layer cross-only.
    #[test]
    fn same_sheet_name_edge_is_dropped() {
        let mut edges = WorkbookCrossSheetEdges::new();
        edges.register_name_edge(name_edge(2, ("Sheet1", 1, 1), "n", 2));
        assert!(edges.all_name_edges().is_empty(), "same-sheet name edge dropped");
        assert!(edges.is_empty(), "no cross edges at all");
    }

    /// The name-seed lane folds a dirtied name's foreign dependents into the
    /// dirty closure as cells before the cell fixpoint runs, and the cell fixpoint
    /// then transitively closes over them.
    #[test]
    fn name_seed_lane_seeds_the_closure_and_transitively_closes() {
        let mut edges = WorkbookCrossSheetEdges::new();
        // Sheet2!A1 resolves name "n" bound on Sheet1 (node 2).
        edges.register_name_edge(name_edge(3, ("Sheet2", 1, 1), "n", 2));
        // Sheet3!Z9 (node 4) reads Sheet2!A1 through an ordinary cell edge.
        edges.register(edge(4, ("Sheet3", 9, 26), ("Sheet2", 1, 1), 3));

        // Seed: name "n" was re-bound on Sheet1 (node 2).
        let mut dirty_names: BTreeMap<TreeNodeId, BTreeSet<String>> = BTreeMap::new();
        dirty_names.insert(TreeNodeId(2), ["n".to_string()].into_iter().collect());
        let closure = workbook_dirty_closure_with_names(&edges, BTreeMap::new(), dirty_names);

        // The name dependent (Sheet2!A1) is dirty …
        assert!(
            closure
                .dirty_cells(TreeNodeId(3))
                .is_some_and(|cells| cells.contains(&cell("Sheet2", 1, 1))),
            "the name re-bind dirties its cross-sheet dependent Sheet2!A1"
        );
        // … and the cell fixpoint transitively dirties Sheet3!Z9 (reads Sheet2!A1).
        assert!(
            closure
                .dirty_cells(TreeNodeId(4))
                .is_some_and(|cells| cells.contains(&cell("Sheet3", 9, 26))),
            "the cell fixpoint transitively closes over the name-seeded dependent"
        );
    }

    // ---- W062 R4.12: derived span-interval index + closure integration ----

    /// A workbook snapshot whose Sheet-role children are `child_ids` (nodes 2..5
    /// named Sheet1..Sheet4), for span-interval-index tests over a given C3 order.
    fn span_index_snapshot(child_ids: Vec<TreeNodeId>) -> crate::structural::StructuralSnapshot {
        use crate::structural::{
            NodeRole, StructuralNode, StructuralNodeKind, StructuralSnapshot, StructuralSnapshotId,
        };
        fn sheet_node(id: u64, symbol: &str) -> StructuralNode {
            StructuralNode {
                node_id: TreeNodeId(id),
                kind: StructuralNodeKind::Container,
                symbol: symbol.to_string(),
                parent_id: Some(TreeNodeId(1)),
                child_ids: Vec::new(),
                role: Some(NodeRole::Sheet),
                is_meta: false,
            }
        }
        let root = StructuralNode {
            node_id: TreeNodeId(1),
            kind: StructuralNodeKind::Root,
            symbol: "Book".to_string(),
            parent_id: None,
            child_ids,
            role: Some(NodeRole::Workbook),
            is_meta: false,
        };
        StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            [
                root,
                sheet_node(2, "Sheet1"),
                sheet_node(3, "Sheet2"),
                sheet_node(4, "Sheet3"),
                sheet_node(5, "Sheet4"),
            ],
        )
        .unwrap()
    }

    fn span_dependent(
        dependent_sheet: u64,
        dependent: (&str, u32, u32),
        start: &str,
        end: &str,
        target: &str,
    ) -> WorkbookSheetSpanDependent {
        WorkbookSheetSpanDependent {
            dependent_sheet: TreeNodeId(dependent_sheet),
            dependent_cell: cell(dependent.0, dependent.1, dependent.2),
            span: GridSheetSpanDependency::new("book", start, end, target),
        }
    }

    #[test]
    fn span_index_interval_probe_names_dependents_of_a_covered_member_sheet() {
        // Sheet4!A1 = SUM(Sheet1:Sheet3!A1). Order Sheet1,Sheet2,Sheet3,Sheet4:
        // the span covers nodes 2,3,4. An edit to any of those member sheets must
        // name Sheet4!A1 as a dependent via the interval probe.
        let snapshot =
            span_index_snapshot(vec![TreeNodeId(2), TreeNodeId(3), TreeNodeId(4), TreeNodeId(5)]);
        let catalog = WorkbookReferenceCatalog::build(&snapshot);
        let index = WorkbookSheetSpanIndex::build(
            &catalog,
            [span_dependent(5, ("Sheet4", 1, 1), "Sheet1", "Sheet3", "A1")],
        );
        for member in [TreeNodeId(2), TreeNodeId(3), TreeNodeId(4)] {
            assert!(
                index
                    .span_dependents_of_sheet(member)
                    .contains(&(TreeNodeId(5), cell("Sheet4", 1, 1))),
                "member sheet {member:?} probes to the span dependent Sheet4!A1"
            );
        }
        // Sheet4 itself is not a member (it is outside the span), so it names no
        // span dependent.
        assert!(index.span_dependents_of_sheet(TreeNodeId(5)).is_empty());
    }

    #[test]
    fn span_index_rebuild_diff_yields_membership_change_dependents_on_reorder() {
        // Baseline order Sheet1,Sheet2,Sheet3,Sheet4: span covers 2,3,4.
        let baseline =
            span_index_snapshot(vec![TreeNodeId(2), TreeNodeId(3), TreeNodeId(4), TreeNodeId(5)]);
        let catalog_before = WorkbookReferenceCatalog::build(&baseline);
        let index_before = WorkbookSheetSpanIndex::build(
            &catalog_before,
            [span_dependent(5, ("Sheet4", 1, 1), "Sheet1", "Sheet3", "A1")],
        );

        // Move Sheet2 (node 3) after Sheet3: order Sheet1,Sheet3,Sheet4,Sheet2.
        // The span Sheet1:Sheet3 now covers only 2,4 — node 3 left the interval.
        let reordered =
            span_index_snapshot(vec![TreeNodeId(2), TreeNodeId(4), TreeNodeId(5), TreeNodeId(3)]);
        let catalog_after = WorkbookReferenceCatalog::build(&reordered);
        let index_after = WorkbookSheetSpanIndex::build(
            &catalog_after,
            [span_dependent(5, ("Sheet4", 1, 1), "Sheet1", "Sheet3", "A1")],
        );

        // The rebuild diff flags Sheet4!A1: its covered-sheet set changed with NO
        // content edit — membership is structural.
        let changed = index_before.membership_change_dependents(&index_after);
        assert!(
            changed.contains(&(TreeNodeId(5), cell("Sheet4", 1, 1))),
            "reordering a member out of the interval flags the span dependent"
        );
    }

    #[test]
    fn span_closure_dirties_span_dependent_on_member_sheet_edit() {
        // Editing a covered member sheet's cell dirties the span dependent through
        // the closure's interval probe — with an ordinary content edit.
        let snapshot =
            span_index_snapshot(vec![TreeNodeId(2), TreeNodeId(3), TreeNodeId(4), TreeNodeId(5)]);
        let catalog = WorkbookReferenceCatalog::build(&snapshot);
        let index = WorkbookSheetSpanIndex::build(
            &catalog,
            [span_dependent(5, ("Sheet4", 1, 1), "Sheet1", "Sheet3", "A1")],
        );
        // Seed: Sheet2!A1 (node 3) edited.
        let initial: BTreeMap<TreeNodeId, BTreeSet<ExcelGridCellAddress>> =
            [(TreeNodeId(3), [cell("Sheet2", 1, 1)].into_iter().collect())]
                .into_iter()
                .collect();
        let closure = workbook_dirty_closure_with_spans(
            &WorkbookCrossSheetEdges::new(),
            &index,
            initial,
            BTreeMap::new(),
            BTreeSet::new(),
        );
        assert!(
            closure
                .dirty_cells(TreeNodeId(5))
                .is_some_and(|cells| cells.contains(&cell("Sheet4", 1, 1))),
            "editing member Sheet2!A1 dirties the span dependent Sheet4!A1"
        );
    }

    #[test]
    fn span_closure_dirties_span_dependent_on_membership_change_with_no_content_edit() {
        // Acceptance (D3 §2.3): inserting/moving a sheet inside the span dirties
        // its dependents with NO content edit. The membership-change dependents
        // (from the index rebuild diff) seed the closure directly.
        let snapshot =
            span_index_snapshot(vec![TreeNodeId(2), TreeNodeId(3), TreeNodeId(4), TreeNodeId(5)]);
        let catalog = WorkbookReferenceCatalog::build(&snapshot);
        let index = WorkbookSheetSpanIndex::build(
            &catalog,
            [span_dependent(5, ("Sheet4", 1, 1), "Sheet1", "Sheet3", "A1")],
        );
        let membership_changed: BTreeSet<(TreeNodeId, ExcelGridCellAddress)> =
            [(TreeNodeId(5), cell("Sheet4", 1, 1))].into_iter().collect();
        let closure = workbook_dirty_closure_with_spans(
            &WorkbookCrossSheetEdges::new(),
            &index,
            BTreeMap::new(),
            BTreeMap::new(),
            membership_changed,
        );
        assert!(
            closure
                .dirty_cells(TreeNodeId(5))
                .is_some_and(|cells| cells.contains(&cell("Sheet4", 1, 1))),
            "a membership change dirties the span dependent with no content edit"
        );
    }
}
