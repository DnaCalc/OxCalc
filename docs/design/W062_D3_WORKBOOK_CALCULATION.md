# W062 D3 — Workbook Calculation Design

## Status

R1 design document for bead `calc-5kqg.6`, authored 2026-07-04 against OxCalc
HEAD `e069136e` (anchors re-verified against the working tree on the authoring
date; two live probe tests were compiled and executed as part of §0's
verification and then removed — their sources are inline below). Program
authority: `docs/worksets/W062_IDEAL_ENGINE_MODEL_REWORK.md`. Fixed inputs:
D1's exported contracts C1–C8 (`docs/design/W062_D1_STRUCTURAL_MODEL.md`),
especially C4 (typed `WorkbookSettingChanged` seeds), C5
(`GridInputSnapshotId` content address), C6 (revision navigation restores
authored truth, marks derived state stale). Harvested input:
`docs/worksets/W062_R0_STASH_TRIAGE.md` Harvest 2, whose open verification is
resolved in §0. Absorbed lane: W060 calc-time reference representation
(`docs/worksets/W060_CALC_TIME_REFERENCE_REPRESENTATION_AND_HOST_REFERENCE_SYSTEM.md`,
first scope closed — §9 states what this design carries forward).

Scope: the workbook-scoped calculation architecture — effective graph shape,
cross-sheet dependency/seed representation, workbook dirty closure and
dependency-ordered evaluation, cycle-detection scope and the W055 joint
shape, the workbook reference oracle and workbook-scope differential,
consumer wiring onto the incremental optimized path, the O(dirty cone)
measurable bar, workbook volatile semantics, tree evaluation joining at name
granularity, and concurrency-prep constraints. Every D3 open question from
the W062 plan is answered in §Open questions answered; the document ends with
the R4 bead breakdown.

No-legacy stance applies: this is the ideal shape; downstream adaptation is a
recorded fact, never a constraint.

## Verified starting anchors

All paths under `src/oxcalc-core/src/` unless noted.

- **Two dependency layers, one sheet.** `GridInvalidationRef { structural,
  calc_overlay, volatile_roots, external_pending_roots }`
  (`grid/machine/invalidation.rs:449-454`), each layer a
  `GridDependencyIndex` with block-compressed reverse indexes keyed on
  `(u32, u32)` row/col blocks (`invalidation.rs:395-440`).
- **Sheet identity is carried but never consulted.** `ExcelGridCellAddress
  { workbook_id, sheet_id, row, col }` (`grid/coords.rs:52-57`), so
  `GridDependency::Cell`/`Range`(via dependent addresses) carry full sheet
  identity — but `GridDependency::Name/NameIdentity/Table/TableIdentity`
  wrap typed structs whose *keys* are bare `String`s (as are the
  reverse-index keys, e.g. `name_dependents_by_key`), and
  `GridDependency::DynamicRequest` is a bare
  request string (`invalidation.rs:198-211`). Nothing in closure routing
  reads `sheet_id`; scoping is an accident of one-`GridInvalidationRef`-
  per-sheet construction.
- **Seed vocabulary.** `GridDirtySeed { Cell, Range, SpillFact, SpillBlocker,
  AxisVisibility, AxisValue, Name(String), Table(String),
  DynamicRequest(String), Volatile, External }` (`invalidation.rs:214-226`);
  `dirty_closure_for_seeds` (`invalidation.rs:3298`);
  `GridDirtySeed::Volatile` closes over `volatile_roots`
  (`invalidation.rs:3291`).
- **The oracle.** `GridCalcRefSheet` (`grid/machine/calc_ref_sheet.rs`),
  worklist-readiness evaluation with stall-extracted
  `GridRefError::EffectiveDependencyCycleDetected { cycle:
  Vec<ExcelGridCellAddress> }` (`calc_ref_sheet.rs:1354,1388,1635,1669`;
  `grid/error.rs:84`).
- **The proven-but-unreachable incremental path.**
  `GridOptimizedSheet::recalculate_dirty_compact_with_oxfml(&self, previous:
  &GridOptimizedValuation, seeds, materialization_limit)`
  (`grid/machine/optimized_sheet.rs:1729`), with typed escalation to
  mark-all when the previous valuation is not full-coverage (`:1743`) or its
  graph was never installed (`:1778`). The consumer never calls it:
  `GridBackingState::recalc` runs `run_engine_mode_with_oxfml(
  GridEngineMode::Both, …)` — both engines, mark-all, every recalc
  (`consumer.rs:1081-1095`) — and `GridBackingState`
  (`consumer.rs:1058-1074`) retains a `published` value map but **no
  `GridOptimizedValuation`**, which is precisely why the incremental
  entry point is unreachable from the consumer today.
- **The differential harness.** `GridEngineMode::{Reference, Optimized,
  Both}` and `compare_grid_engine_readouts`
  (`grid/machine/differential.rs:10-14,176`); the second differential —
  dirty-vs-mark-all inside the optimized engine — is
  `GridDirtyRecalcDifferentialRunReport` with readout, spill-fact,
  graph-equality, dynamic-name, spill-epoch-ledger, and registry-effect
  comparisons (`differential.rs:133-174`).
- **The perf counters for the bar.** `GridOptimizedRecalcReport
  { cells_evaluated, formula_evaluations, formula_cells, … }`
  (`grid/machine/optimized_provider.rs:658-687`).
- **The concurrency blocker.** `GridTracingReferenceSystemProvider { inner,
  trace: RefCell<GridRuntimeDependencyTrace> }`
  (`grid/machine/runtime_trace.rs:2109-2112`) — interior mutability in the
  runtime dependency-trace capture on the evaluation path.
- **Tree side.** `dependency.rs:252` `cycle_groups: Vec<Vec<TreeNodeId>>`
  with `find_cycle_groups` (`dependency.rs:579`), folded into
  dependency-shape revision identity (`workspace_revision.rs:1063-1080`).
  Tree recalc scheduling is full-sweep: every recalc reports
  `scheduling_policy:pull_full_closure` /
  `scheduling_semantic_equivalence_scope:full_closure` (observed in the §0
  probe diagnostics). Runtime host-name bindings for a node's evaluation
  session are assembled by **scanning the node's own source text** for
  visible symbols (`treecalc.rs:5282-5308`), then
  `runtime_binding_for_node` / `callable_binding_from_calc_value`
  (`treecalc.rs:5361`, `:5400`) turn a target's working `CalcValue` into a
  value or callable binding. The offline tree oracle is the
  `oxcalc-tracecalc` crate (reference machine + conformance corpus); there
  is no live in-consumer tree differential.
- **Invalidation seed channel.** `pending_invalidation_seeds:
  Vec<InvalidationSeed>` on live workspace state (`consumer.rs:1332`) — the
  channel D1's C4 `WorkbookSettingChanged` seeds enter through.

---

## 0. The Harvest-2 verification: callable capture and invalidation

The R0 triage left an open verification: does main's callable path
(`callable_binding_from_calc_value`, `treecalc.rs:5361/:5400`) invalidate
callers when a captured node is edited? Verified 2026-07-04 by compiling and
running three probe tests against HEAD (probe file created under
`src/oxcalc-core/tests/`, executed, then removed; sources below).

**Answer, part 1 — the direct case WORKS.** With `A = =3`,
`F = =LAMBDA(x,x+A)`, `Result = =F(2)`:

- `F` carries a `StaticDirect` dependency edge to `A` — the lambda body's
  free host reference is projected from `BoundFormula` like any other
  reference (probe asserted `edges_by_owner[F]` contains `target == A`).
- `Result` carries a `StaticDirect` edge to `F` (the shipped test at
  `consumer.rs:16887` already asserts this shape).
- Editing `A` to `=10` recomputes `Result` from 5 to 12 (probe green).

So caller invalidation for captured-node edits is covered **by edge
composition**: dirty closure of `A` reaches `F` (capture edge) and then
`Result` (call edge). One honesty caveat: today's tree scheduling is
`pull_full_closure` — a full sweep — so the value-level probe alone would
pass even without edges; the edge-level probe is what proves the graph is
correct and therefore that the property survives the move to seeded
closure (§8). Harvest-2 item 1 (first-class caller→captured invalidation
edges) is **already satisfied by main**; no fix needed, but the two passing
probes must land as committed regressions so the property is pinned before
tree evaluation moves off full-sweep (bead R4.1).

**Answer, part 2 — the transitive case is BROKEN, and worse than feared.**
With `A = =3`, `F = =LAMBDA(x,x+A)`, `G = =LAMBDA(y,F(y)*2)`,
`Result = =G(2)`: the **baseline first evaluation** of `Result` fails —

```
OxFml host run for node node:5 failed: OxFml runtime invocation failed:
no callable binding available for callable token
oxfml.callable.0.helper::arity=1;required_arity=1;params=x;
optional_params=-;captures=-;body=Binary
```

This is not an invalidation defect; the value is never right even once.
Root cause chain:

1. Runtime bindings for `Result`'s evaluation session come from scanning
   `Result`'s source text (`treecalc.rs:5282-5308`). `=G(2)` mentions `G`
   but not `F`, so `F`'s callable binding is absent from the session.
2. `G`'s published callable value does not carry its captured environment —
   the callable token itself records `captures=-` — so when `G`'s body
   invokes `F(y)` inside `Result`'s session, there is no binding to find.

The dependency graph is *not* the problem: `G`'s bound formula projects the
`F` reference (the run diagnostics show the `w056_host_name_bind_result`
for `G` targeting `F`), so if evaluation worked, invalidation would too.
Harvest-2 item 2 (transitive-capture closure) is a **live evaluation-
transport defect**. This design closes it in §8.3, and the reproduction is
a **fail-until-fixed test** (policy:
`feedback_fail_until_fixed_tests`) landing with its fix in bead R4.9 — the
test must never be neutered to assert the buggy behavior.

Probe sources (for R4.1/R4.9 to reinstate verbatim):

```rust
// probe 1 (green): edit captured node, caller recomputes
// A="=3", F="=LAMBDA(x,x+A)", Result="=F(2)"; recalc; assert Result=="5";
// set A="=10"; recalc; assert Result=="12".
// probe 2 (green): edges — F has edge->A, Result has edge->F
// (result.dependency_graph.edges_by_owner).
// probe 3 (RED, fail-until-fixed): G="=LAMBDA(y,F(y)*2)", Result="=G(2)";
// first recalc must publish Result=="10" ((2+3)*2); after A="=10",
// Result=="24". Today the whole first recalc TRANSACTION rejects
// (run_state=Rejected, nothing published) with "no callable binding
// available for callable token …captures=-" — the R4.9 test asserts on
// that transaction-rejection shape, then on the published values.
```

---

## 1. Graph shape: federation of per-sheet graphs (decision)

**Decision:** The workbook effective graph is a **federation**: per-sheet
`GridInvalidationRef`s stay exactly as they are for sheet-local edges, plus
a workbook coordination layer (`WorkbookGraph`) that owns every edge whose
dependent and dependency live on different sheets (or at workbook scope).
The alternative — one flat index keyed by full `ExcelGridCellAddress` — is
rejected.

```rust
pub struct WorkbookGraph {
    /// Sheet-local graphs, keyed by sheet node id (D1 C1/C8: sheets are
    /// Sheet-role children; TreeNodeId is the stable identity).
    sheets: BTreeMap<TreeNodeId, GridInvalidationRef>,   // unchanged type
    /// Cross-sheet edge layer: for each *target* sheet, a reverse index of
    /// foreign dependents, reusing the per-sheet index machinery with
    /// workbook-qualified dependents.
    cross: WorkbookCrossSheetEdges,
    /// Workbook-scope roots (volatile/external) for tree nodes and
    /// workbook-scoped names; per-sheet volatile roots stay per-sheet.
    workbook_volatile_roots: BTreeSet<WorkbookCalcNodeId>,
    workbook_external_roots: BTreeSet<WorkbookCalcNodeId>,
}
```

**The routing invariant (new, and the heart of the design):** an edge lives
in sheet S's local `GridInvalidationRef` **iff** both its dependent and its
dependency resolve to sheet S; every other edge lives in `cross`. Sheet
identity on `ExcelGridCellAddress` becomes **authoritative and consulted**:
registration partitions a cell's dependency set by target sheet, local
targets go into the sheet index, foreign targets into the cross layer.
Per-sheet indexes gain a typed rejection
(`GridRefError::ForeignSheetDependency`) if handed an edge whose address
sheet identity disagrees with the owning sheet — today's silent
never-consulted carriage becomes an enforced invariant, so a routing bug is
a loud error, not a wrong answer.

**Rationale (federation over flat):**

- The per-sheet `GridDependencyIndex` is deeply block-indexed —
  `scalar_dependents_by_block`, `compressed_range_dependents_by_block`,
  axis indexes, all keyed on `(u32,u32)` or `(GridAxis,u32)`
  (`invalidation.rs:401-419`). A flat index must thread sheet identity
  through every block key, every report type
  (`GridInvalidationStructuralEditReport` and friends), and every query.
  Federation reuses all of it byte-for-byte for the dominant edge class.
- Sheet lifecycle maps cleanly: D1's add/delete/move sheet verbs become
  add/remove a federation member plus a cross-layer fixup keyed by the
  target sheet; a flat index would mass-rekey on sheet deletion.
- Axis edits (insert row on Sheet2) stay sheet-local in the local graph and
  touch exactly the `cross` partition for target Sheet2 — bounded work.
- The workload argument: in real workbooks intra-sheet edges dominate
  cross-sheet edges by orders of magnitude; the coordination layer should
  be sized to the minority class.

**Revisit trigger (recorded, per the plan's "revisit vs flat" mandate):**
if a measured corpus shows cross-sheet edge counts at rough parity with
intra-sheet counts, or the cross layer's closure hop (§3) shows up as the
dominant term in the perf counters, re-open flat-index as an optimized-lane
alternative — behind the differential, with the oracle unchanged.

**Persistent graph, not transient derivation (D3 open question 3).** The
optimized lane keeps a **persistent, incrementally-maintained** graph: the
retained `GridOptimizedValuation` per sheet (whose `runtime_dependencies`
already persist between recalcs once the consumer retains it, §6) plus the
persistent `cross` layer. Rederiving the workbook graph per recalc is
rejected outright: derivation is O(authored formulas) and would forfeit the
O(dirty cone) bar before evaluation even starts. The oracle, by contrast,
remains free to rebuild everything every run — that is its job.

## 2. Cross-sheet dependency and seed representation

The cell/range story is already paid for; the string-keyed variants are the
design work.

### 2.1 Node space and seed type

```rust
/// The workbook calculation node space. Grid cells at cell granularity;
/// the tree joins at name granularity (§8); scoped names are nodes so
/// name→name and name→cell edges are first-class.
pub enum WorkbookCalcNodeId {
    GridCell(ExcelGridCellAddress),          // carries workbook+sheet already
    Name(ScopedNameKey),
    TreeNode(TreeNodeId),
}

pub enum NameScope { Workbook, Sheet(TreeNodeId) }

pub struct ScopedNameKey { pub scope: NameScope, pub normalized: String }

/// Workbook-level dirty seeds. Sheet-local seeds are addressed, not global.
pub enum WorkbookDirtySeed {
    Sheet { sheet: TreeNodeId, seed: GridDirtySeed },  // reuse per-sheet vocab
    Name(ScopedNameKey),
    TreeNode(TreeNodeId),
    Volatile,                        // one workbook-wide tick (§7)
    External,
    Setting(WorkbookSettingChanged), // D1 C4 typed seeds
}
```

`WorkbookDirtySeed::Setting` routing implements C4's contract:
`DateSystem` — the oracle answer is mark-all (semantic staleness is
workbook-wide; a date-tainted-subgraph narrowing is a later optimized-lane
refinement, valid only behind the differential); `CalcMode` — **no seeds at
all**, the recalc driver consults the mode (manual mode accumulates seeds
and defers the recalc transaction; automatic recalcs on commit); C4
guarantees CalcMode never invalidates values and this design honors it.
`Iteration` — seeds exactly the members of existing cycle groups (§4).

### 2.2 Scoped names, tables, dynamic requests (the string variants)

**Decision:** name/table keys become **scope-qualified at the workbook
layer**. Per-sheet indexes keep their `String` keys unchanged (they are
sheet-scoped by construction); the workbook layer introduces
`ScopedNameKey` and owns the mapping. Resolution — which scope a bare
`Revenue` in a formula on Sheet1 binds to, with Excel's
sheet-scope-shadows-workbook-scope precedence — is **D2's vocabulary
output**; D3's contract is that edge registration receives a
*scope-resolved* name reference and registers against the resolved scope
key. To keep shadowing changes sound, a formula that resolved a bare token
to a workbook-scoped name also registers a **`NameIdentity` edge on the
(potential) sheet-scope key** of its own sheet — the identity edge is
exactly the heal-on-create mechanism the grid already ships
(`GridDependency::NameIdentity`, `invalidation.rs:202`): when a
sheet-scoped `Revenue` is later created and shadows the workbook one, the
identity edge dirties the dependent and rebinding re-routes the value edge.
Symmetric on delete. This is the same pattern D1 C2 prescribes for sheet
names.

`DynamicRequest` keys become workbook-qualified in normal form:
`{workbook}:{requesting-sheet}:{request-text}` — a CTRO dynamic resolution
(e.g. `INDIRECT("Sheet2!A1")`) records realized dependencies that are
already sheet-qualified cell/range dependencies, and the request identity
itself must not collide across sheets. The runtime trace provider (§9)
supplies the qualification.

### 2.3 3D spans (`Sheet1:Sheet3!A1`)

C3 fixes the semantics: a span covers Sheet-role nodes between the
endpoints in filtered root `child_ids` order. Dependency shape decision
(flagged by D2's open list, decided here because it is a graph question):

**Decision (reconciled 2026-07-04, program-owner arbitration): one stored
`SheetSpan` edge, expanded at closure time — adopting D2 §4.2 / contract
V5.** This design originally decided a per-sheet fan + span-identity edge;
D2 (committed `2bc6ea7e`) decided the opposite and exported it as V5. The
fresh-eyes review surfaced the conflict and the owner ruled in D2's favor:
the stored dependency is a single `GridDependency::SheetSpan` edge (never
a materialized fan — a fan must be rewritten on every sheet
insert/delete/move touching the span). This design's own objection — that
closure-time expansion pushes member enumeration into every closure walk —
is answered with a **derived span-interval index**: a small structure over
the D1 sheet-registry order mapping each sheet position to the spans
covering it, rebuilt on sheet-lifecycle edits (cheap: spans are few,
lifecycle edits are rare). Closure walks do an O(log n) interval probe per
dirtied sheet, not member enumeration; membership-change invalidation
falls out of the index rebuild diff (a sheet entering/leaving a span's
interval dirties that span's dependents) — no separate span-identity edge
needed. Closure output depending on structural state is inherent to 3D
references (membership *is* structural); the index makes the dependency
explicit and cheap rather than hiding it in stored-edge churn.

## 3. Workbook dirty closure and cross-sheet evaluation order

**Closure algorithm (optimized lane):** iterate to fixpoint —

1. Partition seeds: `Sheet{…}` seeds go to their sheet; `Name`/`TreeNode`/
   workbook `Volatile`/`External` seeds enter the workbook layer.
2. Per seeded sheet, run the existing local
   `dirty_closure_for_seeds` (`invalidation.rs:3298`) — unchanged code.
3. For each sheet with newly dirty cells (or newly dirty
   names/spill-facts/etc.), consult the `cross` partition for that target
   sheet: foreign dependents matched by the same cell/range/name matching
   rules become new seeds on *their* sheets (or dirty workbook-layer
   name/tree nodes).
4. Repeat until no new dirty state. Termination: the dirty set is monotone
   and bounded by the node space.

The closure result is a `WorkbookDirtyClosure { seeds, dirty:
BTreeSet<WorkbookCalcNodeId> }`.

**Evaluation order:** one **workbook-level worklist**, dependency-ordered
across sheet boundaries — *not* sheet-at-a-time. Sheet-at-a-time is
incorrect on its face: `Sheet1!A1 → Sheet2!B1 → Sheet1!C1` is an ordinary
chain, so no total sheet order exists. The optimized coordinator owns
readiness scheduling over `WorkbookCalcNodeId`; per-sheet engines remain
the evaluators (formula plans, dense regions, spill repair stay per-sheet),
but a cell becomes ready only when its cross-sheet dependencies are
published to the coordinator's value view. Deterministic total order:
readiness first, `BTreeSet<WorkbookCalcNodeId>` order as tiebreak — no
hash-map iteration anywhere in scheduling (this is the §10 deterministic
worklist constraint, satisfied by construction).

Spill repair note: spill extents are sheet-local facts, but a cross-sheet
consumer of a spill range must observe post-repair extents; therefore
spill-repair passes run inside the per-sheet evaluation step and their
resulting `SpillFact`/`Range` deltas feed back through the cross layer as
step-3 seeds in the same closure loop. The oracle gets this for free
(below); the optimized lane inherits the existing per-sheet repair loop
plus the cross feedback.

## 4. Cycle detection scope and the W055 joint shape

**Cycle scope:** workbook-wide, over the effective (post-CTRO) graph, on
the `WorkbookCalcNodeId` space. `EffectiveDependencyCycleDetected` widens
its payload from `Vec<ExcelGridCellAddress>` to
`Vec<WorkbookCalcNodeId>` — a cycle through `Sheet1!A1 → Name(Revenue) →
TreeNode(Total) → Sheet1!A1` is one cycle, reported with full identities.
In the oracle, detection stays exactly what it is today: worklist stall ⇒
extract the cycle from the unready remainder (`calc_ref_sheet.rs:1354`
pattern, address type widened). Obvious correctness is preserved because
the algorithm does not change — only the node type does.

**W055 co-design (decision: joint redesign, not retrofit).** The paused
`calc-9ouy.2` (W055 general cycle engine design) resumes against this
section; this is the joint shape:

- **One cycle engine, workbook-scoped, W055-owned.** There is no tree
  cycle engine and grid cycle engine; there is one engine whose member
  space is `WorkbookCalcNodeId`. D3/R4 provides the substrate (node space,
  effective-graph cycle-group computation, the C4 `Iteration` seed
  channel); W055 Tranche A provides the engine semantics and lands under
  its own beads.
- **Cycle groups are computed on the effective graph** (static edges plus
  CTRO-realized overlay edges), workbook-wide strongly-connected
  components; the tree's existing `cycle_groups`
  (`dependency.rs:252,579`) and the grid's stall detection both become
  views of this one computation. Cycle groups remain revision facts
  (dependency-shape identity already folds them,
  `workspace_revision.rs:1063-1080`) so C4's `Iteration` seed can target
  "members of existing cycle groups" deterministically.
- **Profile data (the calc-9ouy.2 gate list, made concrete):**
  - *member ordering:* strict-excel — sheets in C3 filtered order, then
    row-major within sheet; tree profile — `TreeNodeId` order; names —
    scope then normalized key. Total and deterministic by construction.
  - *initial vector:* profile-supplied; Excel-match default is zero for
    numeric, previous published value when iterating from a prior state.
  - *update model:* one sweep per iteration in member order using
    latest-available values (Excel's observed in-order sweep), value table
    double-buffered per iteration boundary for the stop metric.
  - *stop metric:* `max |Δ|` over members `< max_change`, OR iteration
    count `== max_iterations` — both from C4 `IterationSettings`.
  - *terminal state:* profile-specific — strict-excel publishes the last
    iterate (Excel behavior); the tree profile keeps its current typed
    rejection default until W055 flips it deliberately.
  - *publish/reject:* atomic per cycle group — the whole group's iterates
    publish, or none do.
  - *downstream invalidation:* the cycle group is a super-node in closure:
    any member dirty ⇒ whole group re-iterates; group output deltas seed
    dependents outside the group.
  - *diagnostics/replay:* per-group summary (members in order, iterations
    run, final max Δ, converged flag) surfaced through the recalc report
    and replay identity.
- **Iteration disabled** (the default): any workbook-scope cycle is a
  typed calculation outcome (strict-excel: `#REF!`-adjacent circular
  diagnostic per Excel's circular-warning semantics — exact surface is a
  W055 Tranche B evidence question; tree: existing rejection), never a
  hang.

The oracle iterates cycle groups with the identical profile data and a
brutally simple loop; the differential compares group membership, iterate
counts, and published values. Divergence is stop-the-line, as everywhere.

## 5. The workbook reference oracle and the differential at workbook scope

**Oracle:** `GridCalcRefWorkbook` — the two-model principle extended, not
reinvented. It is the per-sheet `GridCalcRefSheet` algorithm with exactly
one generalization: the value lookup a formula evaluation performs consults
a **workbook value table** (`BTreeMap<WorkbookCalcNodeId, CalcValue>`)
instead of a sheet-local table, and the worklist holds
`WorkbookCalcNodeId`s. Mark-all across all sheets and names and tree nodes,
every run, no incrementality, no caching, no cleverness. Its correctness
must remain obvious — reviewers should be able to read it top to bottom;
anything clever belongs in the optimized coordinator behind the
differential.

**Differential, both families, workbook scope:**

- **Reference vs optimized** (`GridEngineMode::Both` generalized to
  `WorkbookEngineMode::Both`): full-workbook readout comparison, per-sheet
  mismatch lists aggregated with sheet identity, plus overlay-blockage and
  spill-fact comparison per sheet — the existing
  `compare_grid_engine_readouts` machinery lifted over the sheet map.
- **Dirty vs mark-all** (the incremental-correctness differential,
  `GridDirtyRecalcDifferentialRunReport` generalized): identical readouts,
  identical per-sheet graphs, **identical cross-sheet edge sets** (new
  comparison — the cross layer of the dirty run must equal the cross layer
  of the mark-all rebuild), dynamic-name state, spill epoch ledgers,
  registry effects.

Every R4+ behavior lands oracle-first (or simultaneously) with its
differential extension in the same bead — restating the program doctrine as
a hard bead-acceptance requirement.

## 6. Consumer onto the incremental path — an independent slice, and it lands FIRST

**Decision:** wiring the consumer onto
`recalculate_dirty_compact_with_oxfml` lands **before** federation, as R4's
first implementation beads, single-sheet scope. The W062 plan explicitly
invited this split ("D3 may split this out as an earlier independent
landing"), and the code says it is independent: the incremental entry point
is per-sheet, fully tested, and blocked only by the consumer discarding the
valuation each recalc (`GridBackingState` retains `published` but no
`GridOptimizedValuation`, `consumer.rs:1058-1074`). Nothing about
federation changes this seam; gating the cheapest and most measurable win
on the largest build would be sequencing malpractice.

The slice:

1. **Retain the valuation.** D1's R2.6 split puts engine state in
   `GridDerivedState`; the retained `GridOptimizedValuation` (and its
   formula-plan cache) lives there. **Basis stamping:** the valuation
   records the `GridInputSnapshotId` (C5) it was computed from; the
   incremental path is legal only when the stamp matches current authored
   truth — any mismatch (including C6 revision navigation, which swaps
   authored truth and marks derived state stale) escalates to mark-all.
   This composes with the existing typed escalations (`optimized_sheet.rs
   :1743,1778`); escalation is always *to correctness*.
2. **Edits produce seeds.** Grid edit verbs (R2/R5 lane) emit
   `GridDirtySeed` batches: cell edit ⇒ `Cell`; region ops ⇒ `Range`; name
   lifecycle ⇒ `Name`; table ops ⇒ `Table`; D1 C4 settings ⇒ mapped per
   §2.1. Seeds accumulate on the backing between recalcs.
3. **Recalc becomes seeded.** `GridBackingState::recalc` calls
   `recalculate_dirty_compact_with_oxfml(previous, seeds, …)` on the
   optimized lane and refreshes `published` from the delta.
4. **Differential policy becomes explicit.** Running the reference engine
   mark-all on every live edit would erase the bar the slice exists to
   meet. The consumer gains a typed knob:
   `GridDifferentialPolicy { EveryRecalc, Sampled { one_in: u32 }, Off }`.
   The test suite and the corpus harness run `EveryRecalc` (both
   differentials); the perf evidence lane runs `Off`; the default for
   embedding hosts is `Sampled` (divergence detection stays live at
   bounded cost). The oracle's authority is not weakened — every behavior
   is still differentially proven in the suite; the policy governs only
   the *live consumer's* per-recalc spend.
5. **The bar, evidenced.** A perf-counter test builds an N-formula sheet
   (N ≥ 10⁴), recalculates once (mark-all, `cells_evaluated == N`-ish),
   edits one cell, recalculates, and asserts
   `report.cells_evaluated ≤ cone_size` where `cone_size` is the exact
   dirty closure of the edit — and `≪ N` by construction of the fixture.
   Counters exist today (`optimized_provider.rs:662-663`); the checked-in
   baseline records both numbers per local-execution doctrine. The same
   assertion re-runs at workbook scope after federation (edit on Sheet1
   must not evaluate anything on an unrelated Sheet3).

## 7. Workbook-wide volatile semantics

**Decision:** volatiles tick **once per workbook recalc transaction**.

- Each workbook recalc mints a `WorkbookRecalcTick { tick_id, timestamp,
  rng_seed }` before evaluation starts. `NOW()`/`TODAY()` read
  `timestamp`; `RAND()`/`RANDBETWEEN()`/`RANDARRAY()` draw from a stream
  deterministically derived from `rng_seed` + evaluating node id (so
  evaluation *order* does not change values — a concurrency-prep property:
  two cells' draws are independent of which evaluates first). `rng_seed`
  and `timestamp` are recorded in the recalc report for replay.
- Coherence: every sheet, name, and tree evaluation in the same recalc
  observes the same tick. `NOW()` on Sheet1 equals `NOW()` on Sheet3
  equals `NOW()` in a tree node, per recalc. This is the Excel contract.
- Seeding: an automatic-mode recalc implicitly includes
  `WorkbookDirtySeed::Volatile`, which expands to per-sheet
  `GridDirtySeed::Volatile` (closing over each sheet's `volatile_roots`,
  `invalidation.rs:3291`) plus the workbook-layer volatile roots (dynamic
  names, tree volatiles). A recalc with zero authored edits still ticks
  volatiles — Excel F9 semantics.
- The oracle evaluates everything anyway; it simply reads the same tick.
  The differential therefore compares volatile values exactly (same
  timestamp, same seeded streams) — volatiles are **not** excluded from
  comparison, which keeps the harness honest.

## 8. Tree evaluation joins at name granularity

### 8.1 The join

Tree nodes enter the workbook graph as `WorkbookCalcNodeId::TreeNode` —
name-granularity nodes (the plan's "tree node ≈ defined name"). Edges
become first-class in both directions:

- tree formula → grid cell/range (`=SUM(Sheet1!A1:A10)` on a tree node):
  cross-layer edges from `TreeNode` to sheet-qualified targets, produced by
  the same BoundFormula projection that builds tree edges today;
- grid formula → tree node (a grid cell referencing a defined name that
  *is* a tree node): the D2 vocabulary resolves the name to
  `TreeNode(id)`; the edge registers in the cross layer; the existing
  per-sheet `Name`/`NameIdentity` machinery covers the sheet-local half.

The tree's existing `DependencyDescriptor` graph and the workbook layer
converge: tree edges *are* workbook-layer edges (tree→tree edges stay in
the tree's own index, exactly parallel to sheet-local edges staying in
sheet indexes — the federation principle applied to the tree as one more
member).

### 8.2 The tree-side two-model question (decision: join the grid family)

**Decision:** the tree does **not** grow its own live differential pair.
The workbook oracle (§5) evaluates tree nodes as ordinary
name-granularity nodes — simple recursive/worklist evaluation over the
same value table — so the workbook-scope differential subsumes live
tree-vs-oracle comparison the day tree nodes join the worklist. The
offline `oxcalc-tracecalc` reference machine and its conformance corpus
remain exactly what they are: a third, independent, corpus-level check
(defense in depth, different codebase lineage). Rationale: a second live
differential family would double harness surface for zero marginal
coverage once tree nodes are workbook nodes; and the tracecalc oracle's
value is precisely that it is *not* built from the same abstractions.

Tree scheduling moves from `pull_full_closure` (full sweep every recalc —
verified in §0 probe diagnostics) to seeded closure **only after** the
grid slice has proven the pattern (R4 ordering, §R4). The O(dirty) bar
then extends to tree edits.

### 8.3 The callable fix (closing §0's defect)

Two sub-decisions:

- **(a) Semantic: callables capture their environment at definition
  evaluation.** Excel LAMBDA is lexically scoped; a callable value must
  carry the host-name bindings its body's free names resolved to when the
  defining formula evaluated. This is the principled fix for
  `captures=-`: `G`'s callable carries `F`'s binding (which `G`'s own
  session had — `=LAMBDA(y,F(y)*2)` mentions `F`, so the source-scan found
  it); invoking `G` anywhere then works, transitively, because each
  callable carries what its body needs. **This is an OxFml ask** (callable
  value representation and invocation environment are OxFml-owned) — a
  small, named upstream item *in addition to* W077, flagged in
  Cross-design tensions.
- **(b) Invalidation: unchanged.** §0 verified the defining node carries
  edges to captured names, and callers carry edges to the defining node;
  dependency-ordered evaluation re-evaluates definitions before callers,
  so captured environments are refreshed before any caller can invoke a
  stale one. No new invalidation machinery.

**Fallback (recorded, not preferred):** an OxCalc-only fix — derive a
node's runtime binding set from the *transitive* static dependency closure
over callable-valued targets instead of the source-text scan
(`treecalc.rs:5282-5308`). It fixes the failure without touching OxFml,
but it transports bindings by caller-side closure rather than
definition-site capture, which is the wrong scoping rule the moment a
callable is passed as a value out of its defining subtree. Use only if the
OxFml lane is congested; the fail-until-fixed test is fix-agnostic.

Independent of which fix lands, the **source-text scan dies** in the tree
unification: runtime bindings must derive from bound-formula facts (the
dependency projection already has them), not from substring matching on
source text — the scan is over-approximate (`contains` on uppercased
source text: a visible symbol `A` "appears" in `=SUM(BAR)`, producing
spurious candidate bindings) and under-approximate (§0: names reached
only through a callable's body never appear in the caller's text). Bead
R4.9 covers this replacement.

## 9. W060 lane (absorbed): what calc-time references mean at workbook scope

W060's first scope is **closed** (typed `CalcValue::Reference` /
`ReferenceLike` identity, `TreeCalcReferenceSystemProvider`, `HOST_REF_*`
eliminated — W060 doc §12 with test evidence). D3 absorbs the lane's
go-forward items:

- **The workbook reference system provider.** The strict-excel runtime
  provider (`ExcelGridReferenceSystemProvider`,
  `grid/reference_engine.rs:114` — currently one sheet's cells behind a
  `Cow` map) grows to workbook scope: `resolve_text` for
  `INDIRECT("Sheet2!A1")` routes through D2's vocabulary (profile parsing
  + sheet registry lookup per C1), and dereference consults the
  coordinator's workbook value view. CTRO dynamic dependencies realized
  through it are sheet-qualified by construction (cells) or
  workbook-qualified (names/requests, §2.2).
- **Trace capture is the CTRO edge source.** The tracing wrapper
  (`GridTracingReferenceSystemProvider`, `runtime_trace.rs:2109`) is where
  realized dependencies enter the `CalcOverlay` layer; at workbook scope
  its recorded `GridDependency` values must carry the §2 scoped forms. Its
  `RefCell` is §10's first target.
- **Transforms stay typed provider requests.** `OFFSET`, reference-form
  `INDEX`, union/intersection, and 3D composition are host reference-system
  requests (W060 §5); the workbook provider implements sheet-aware
  transforms as profiles claim them; unclaimed operations remain typed
  unsupported (the shipped
  `treecalc_provider_keeps_transform_and_compose_as_typed_unsupported_requests`
  pattern).

Nothing else from W060 is re-opened; the `BoundFormula`-owns-graph /
calc-time-values-own-runtime boundary (W060 §2) is preserved verbatim —
graph construction never depends on calc-time materialization.

## 10. Concurrency preparation (constraints, no executor)

Constraints the R4 lanes must satisfy; no concurrent executor is built.

1. **Per-evaluation trace buffers.** The
   `RefCell<GridRuntimeDependencyTrace>` in
   `GridTracingReferenceSystemProvider` (`runtime_trace.rs:2111`) is
   replaced by value-returning accumulation: the provider is constructed
   per cell evaluation (already the usage pattern — `new(inner)` /
   `finish()` bracketing), so the trace becomes an owned buffer threaded
   through `&mut` or returned from the evaluation call, never interior
   mutability. Acceptance is mechanical: no `RefCell`/`Cell` in any type
   reachable from the optimized evaluation path.
2. **Send audit.** `WorkbookGraph`, `GridOptimizedValuation`, worklist
   state, and every type the coordinator holds across an evaluation step
   must be `Send` (compile-time `assert_send::<T>()` tests). No `Rc`, no
   thread-local state in evaluation.
3. **Deterministic worklist.** Already satisfied by §3's construction
   (BTree ordering everywhere); pinned by a test that permutes seed
   insertion order and asserts identical evaluation order and identical
   reports.
4. **Pure providers.** Evaluation reads go through provider views of an
   immutable pre-recalc snapshot plus the coordinator's published-value
   table; no evaluation writes anything but its own result and trace.
   (This is the property that later lets independent worklist ready-sets
   evaluate in parallel; W053's staged-concurrency lane inherits it.)
5. **Order-independent volatiles** (§7): RAND streams keyed by node id,
   not by evaluation sequence.

## Open questions answered

Every D3 question from `W062_IDEAL_ENGINE_MODEL_REWORK.md` §Open questions
for R1, plus the scope items the bead adds:

| D3 question | Answer |
| --- | --- |
| Federation vs flat workbook index | Federation: per-sheet `GridInvalidationRef`s + workbook cross-sheet edge layer; sheet identity on addresses becomes authoritative with a typed foreign-edge rejection; flat index rejected (index/report rework, sheet-lifecycle rekeying); revisit trigger recorded (§1). |
| Cross-sheet `GridDependency` variants | Cell/Range already sheet-qualified; Name/Table become `ScopedNameKey` at the workbook layer with `NameIdentity` shadow-healing edges; `DynamicRequest` keys workbook-qualified; 3D = one stored `SheetSpan` edge + derived span-interval index, expanded at closure time (D2 V5, reconciled §2.3); `WorkbookDirtySeed`/`WorkbookCalcNodeId` defined (§2). |
| Optimized engine: persistent graph vs transient derivation | Persistent — retained per-sheet valuations + persistent cross layer, basis-stamped with `GridInputSnapshotId` (C5) and escalating to mark-all on mismatch; transient derivation rejected (forfeits O(dirty cone)) (§1, §6). |
| Tree-side two-model: own oracle pair vs joining the grid family | Joins the grid family: workbook oracle evaluates tree nodes at name granularity; no second live differential; offline tracecalc oracle retained as independent third check (§8.2). |
| W055 cycle-engine interplay: joint redesign vs retrofit | Joint redesign: one workbook-scoped cycle engine on `WorkbookCalcNodeId`, D3 provides substrate, W055 owns engine semantics; concrete profile-data table supplied so `calc-9ouy.2` resumes against §4. |
| Concurrency prerequisites (trace buffers, Send audit) | Five constraints specified (per-evaluation trace buffers replacing the `runtime_trace.rs:2111` RefCell, Send audit, deterministic worklist, pure providers, order-independent volatiles); no executor (§10). |
| Consumer incremental wiring: before federation as an independent slice? | Yes — lands first, single-sheet, R4.1-R4.2: retained valuation + seeds + differential policy knob + perf-counter bar (§6). |
| The O(dirty cone) bar | `cells_evaluated`/`formula_evaluations` counter assertions with checked-in baselines, single-sheet first, re-asserted at workbook scope (§6.5). |
| Workbook volatile semantics | One `WorkbookRecalcTick` per recalc transaction; coherent NOW across sheets/tree; node-id-keyed deterministic RAND streams; recorded for replay; volatiles compared exactly in the differential (§7). |
| W060 absorbed lane | First scope closed upstream; D3 carries the workbook-scope provider, trace-sourced CTRO edges in scoped form, and typed transform requests (§9). |
| Harvest-2 open verification | Resolved by live probes: direct captured-node invalidation WORKS (edges verified); transitive callable-calls-callable is a live baseline-evaluation defect (`captures=-`); fail-until-fixed test specified; fix = OxFml definition-site capture (preferred) with an OxCalc transitive-binding fallback (§0, §8.3). |

**Deferred (with reasons):** (1) The exact strict-excel *surface* for
circular-reference diagnostics with iteration disabled (warning vs error
value) — W055 Tranche B's Excel observation matrix owns the evidence; §4
fixes only that it is a typed outcome, never a hang. (2) Date-tainted
narrowing of the `DateSystem` seed — optimized-lane refinement behind the
differential; the oracle answer (mark-all) ships first per C4. (3) External
workbook references in the workbook graph — D2 owns the in/out decision;
D1 C8/§6 already guarantees the structural answer (another workspace), and
the cross-layer shape here extends to cross-workspace edges without rework
if D2 rules them in.

## Cross-design tensions

- **RESOLVED — 3D dependency shape (was a silent conflict with D2 §4.2/V5).**
  This doc's original per-sheet-fan decision contradicted D2's committed
  single-`SheetSpan`-edge contract V5. Owner arbitration (2026-07-04)
  ruled in D2's favor; §2.3 now specifies the stored span edge plus a
  derived span-interval index, and R4.12 implements that shape. D2 §4.2,
  V5, and R3.9 stand unchanged.
- **RESOLVED — callable capture contract (supersedes D2 §8/V6 mechanism).**
  D2's V6 prescribed `captured_dependency_keys` exposure + caller→captured
  graph edges as the invalidation mechanism; §0's live probes show
  invalidation already composes correctly through the defining node and
  the actual defect is evaluation transport (§8.3). Owner arbitration:
  V6's *mechanism* is superseded by §0/§8.3 (D2 carries an errata note);
  the semantic invariant — editing a captured node re-evaluates callers —
  stands and is guarded by R4.1/R4.9's tests, which also discharge D2
  R3.11's verification act. `captured_dependency_keys` exposure may
  return later as a diagnostics surface, not an invalidation dependency.
- **New upstream OxFml ask beyond W077 (owner attention).** §8.3's
  preferred callable fix (definition-site captured environments on
  callable values) is OxFml work. MEMORY/plan currently record OxFml W077
  as the *sole* hard upstream dependency of W062; this adds a second,
  small, well-scoped one. The OxCalc-only fallback keeps R4 unblocked if
  the OxFml lane is congested, at the cost of the wrong scoping rule for
  escaped callables. Decision needed at R4.9 latest.
- **C5 basis stamping is a note to R2.6.** §6.1 needs the retained
  valuation stamped with the `GridInputSnapshotId` it was computed from.
  C5/C6 make this sound, but D1's R2.6/R2.7 beads should carry the stamp
  field so R4.1 does not retrofit it. Friction is additive, not
  contradictory.
- **Differential policy vs "permanent differential" doctrine.** §6.4's
  `Off/Sampled` modes in the live consumer could be read as weakening the
  two-model principle. They do not — every behavior remains
  oracle-differentially proven in the suite and corpus, and divergence
  remains stop-the-line — but the distinction (harness always, live
  consumer by policy) should be stated in the workset register when R4
  opens, so it is a recorded decision, not drift.
- **D2 sequencing.** §2.2 scoped-name registration and §2.3's 3D span
  edge consume D2 *design* outputs (scope resolution precedence, 3D
  grammar shape). R4 orders cell/range federation first so nothing hard-blocks on
  D2 implementation (R3); the name-scope and 3D beads gate on the D2
  document only. If D2 lands a different `!`/scope vocabulary than
  assumed, the cross layer is unaffected (it stores resolved scopes, not
  tokens).
- **CalcMode stays scheduling-only (C4) — confirmed, one consumer-surface
  consequence for D4/D2a-style hosts:** manual mode means seeds accumulate
  and `recalculate` becomes the explicit transaction; D4's document verbs
  must expose "calculate now" without implying any value invalidation
  occurred at mode-flip time.
- **`machine.rs` decomposition (D4 question).** R4 adds workbook modules
  as siblings (`grid/workbook/…`); it neither requires nor performs the
  24.5k-line `machine.rs` split. No dependency either way.
- **W056 residual O(n²) diagnostics** (memory note on warm-recalc work):
  the §6.5 perf-counter bar asserts on engine counters, not wall time, so
  diagnostic-lane costs cannot mask or fake the bar; wall-time evidence in
  the perf harness should still note the known diagnostics term.

## Contracts exported

Other designs and W055 may rely on these without re-deriving:

- **X1 (D4, D1):** `WorkbookDirtySeed` is the single workbook-level seed
  vocabulary; D1 C4 `WorkbookSettingChanged` seeds enter as
  `Setting(…)`; document-surface verbs emit seeds, never touch dirty state
  directly. CalcMode consultation lives in the recalc driver; manual mode
  defers transactions, seeds are never dropped.
- **X2 (W055, D2):** `WorkbookCalcNodeId { GridCell, Name(ScopedNameKey),
  TreeNode }` is the calculation node space: cycle-group membership,
  cycle diagnostics, and cross-sheet edges are expressed in it. D2's
  resolution output for a name reference is a scope-resolved
  `ScopedNameKey` (or a cell/range with sheet identity).
- **X3 (D1/R2):** the retained `GridOptimizedValuation` carries the
  `GridInputSnapshotId` basis it was computed from; incremental recalc is
  legal iff the stamp equals current authored identity, else typed
  escalation to mark-all. Revision navigation (C6) therefore composes with
  incrementality with zero special cases.
- **X4 (W053, future executor):** scheduling order is a pure function of
  (graph, seeds): readiness + BTree order, no insertion-order or
  hash-order dependence; evaluation is pure-provider reads + own-result
  writes; volatile draws are node-keyed. Any future concurrent executor
  must preserve exactly the sequential worklist's observable results.
- **X5 (D4, replay):** `WorkbookRecalcTick { tick_id, timestamp, rng_seed }`
  is minted once per workbook recalc transaction, observed coherently by
  every evaluation in it, and recorded in the recalc report.
- **X6 (hosts):** `GridDifferentialPolicy { EveryRecalc, Sampled, Off }`
  governs live-consumer differential spend only; the test/corpus lanes
  always run full differentials.

## R4 implementation-wave bead breakdown

Ordered; sizes S ≲ half day, M ≈ 1 day, L ≈ 2 days. Every bead lands green
on `main` per the W062 execution process (decisive acceptance check,
fresh-eyes review instructions, `cargo test -p oxcalc-core` + clean
differential recorded in the bead when R4 beads are created). Oracle-first
discipline: beads that add behavior land the oracle change and the
differential extension in the same bead as the optimized change.

1. **R4.1 — Callable regression pinning (S).** Land §0 probes 1 and 2
   (direct capture invalidation + edge assertions) as committed
   regressions, so the property is pinned before scheduling changes.
   Acceptance: both tests green; edge assertions reference
   `edges_by_owner` shape.
2. **R4.2 — Consumer incremental wiring, single sheet (L).** §6 items 1–4:
   valuation retained in `GridDerivedState` with X3 basis stamp, edit
   verbs emit `GridDirtySeed`s, `recalc` drives
   `recalculate_dirty_compact_with_oxfml`, `GridDifferentialPolicy` knob
   (suite pinned to `EveryRecalc`). Acceptance: dirty-vs-mark-all
   differential clean across an edit-sequence corpus; revision-navigation
   escalation test (undo ⇒ mark-all, values correct).
3. **R4.3 — The O(dirty cone) bar, evidenced (M).** §6.5 perf-counter
   harness: ≥10⁴-formula fixture, single-cell edit,
   `cells_evaluated ≤ cone`, checked-in baseline with both numbers.
   Acceptance: baseline committed; assertion fails if the consumer
   regresses to mark-all.
4. **R4.4 — Workbook node space and routing invariant (M).**
   `WorkbookCalcNodeId`, `ScopedNameKey`, `WorkbookDirtySeed`; sheet
   identity consulted at registration; typed
   `ForeignSheetDependency` rejection in per-sheet indexes. Acceptance:
   property test — no edge with foreign sheet identity can enter a local
   index.
5. **R4.5 — Workbook reference oracle (L).** `GridCalcRefWorkbook` (§5):
   mark-all across sheets, workbook value table, widened cycle error with
   `WorkbookCalcNodeId` path. Acceptance: multi-sheet fixtures (chain,
   diamond, cross-sheet cycle) with hand-computed expected values;
   reviewer confirms the oracle reads as obviously correct.
6. **R4.6 — Cross-sheet edge layer + workbook closure + cross-sheet
   worklist in the optimized coordinator (L).** §1 `cross` layer, §3
   fixpoint closure and workbook worklist, spill-delta feedback;
   reference-vs-optimized differential at workbook scope. Acceptance:
   Sheet1→Sheet2→Sheet1 chain evaluates correctly; differential clean on
   the multi-sheet corpus.
7. **R4.7 — Workbook dirty-vs-mark-all differential + multi-sheet
   incremental consumer (M).** §5 second family incl. cross-layer edge-set
   equality; seeds crossing sheet boundaries through the consumer.
   Acceptance: single-cell edit on Sheet1 with a Sheet3 dependent
   evaluates the cross-cone only (counters), differential clean; the R4.3
   bar re-asserted at workbook scope.
8. **R4.8 — Workbook volatile semantics (M).** §7 `WorkbookRecalcTick`,
   implicit Volatile seed in automatic mode, node-keyed RAND streams,
   replay recording. Acceptance: NOW coherent across three sheets + tree
   node in one recalc; two recalcs differ; differential compares volatiles
   exactly and stays clean; permuted evaluation order leaves RAND values
   unchanged.
9. **R4.9 — Tree evaluation unification part 1: bindings from bound facts
   + callable capture fix + fail-until-fixed test (L).** §8.3: retire the
   source-text scan (`treecalc.rs:5282-5308`) in favor of bound-formula-
   derived bindings; land the §0 probe-3 transitive-callable test failing
   in the same bead as the fix (OxFml capture preferred; fallback
   recorded). The upstream decision (Cross-design tensions item 1) is made
   at this bead's opening, not later. Acceptance: probe 3 green; no
   substring-scan binding path remains.
10. **R4.10 — Tree joins the workbook graph (L).** §8.1/8.2: `TreeNode`
    nodes in closure and worklist, tree↔grid cross edges, tree scheduling
    from `pull_full_closure` to seeded closure; workbook differential now
    covers tree nodes; tracecalc corpus re-run as the independent check.
    Acceptance: tree-edit dirty cone measured by counters; grid cell
    referencing a tree-node name recalculates on tree edit and vice versa.
11. **R4.11 — Scoped names, tables, dynamic requests (M).** §2.2 scoped
    keys at the workbook layer, shadow-healing `NameIdentity` edges,
    workbook-qualified `DynamicRequest` normal form. Gates on the D2
    design document (not R3 implementation). Acceptance: sheet-scoped name
    created over a workbook-scoped binding re-routes dependents; INDIRECT
    to another sheet realizes sheet-qualified CTRO edges; differential
    extended to scoped-name scenarios (oracle-first in the same bead).
12. **R4.12 — 3D span edges (M).** §2.3 reconciled shape: stored
    `GridDependency::SheetSpan` edge + derived span-interval index over
    the sheet-registry order (rebuilt on lifecycle edits); closure-time
    expansion via interval probe; membership-change invalidation from the
    index rebuild diff. Gates on D2's 3D grammar for end-to-end text, but
    the edge layer lands earlier against synthetic bindings. Acceptance:
    inserting a sheet inside `Sheet1:Sheet3` dirties span dependents with
    no content edit; differential extended to span-edge scenarios
    (oracle-first in the same bead).
13. **R4.13 — Cycle-engine substrate handover (M).** §4: workbook-wide
    cycle groups on the effective graph, `Iteration` seed targeting group
    members, widened diagnostics plumbing. `calc-9ouy.2` resumes against
    §4 and W055 builds the engine on this substrate under its own beads.
    Acceptance: cross-sheet and tree↔grid cycles detected with full
    `WorkbookCalcNodeId` paths; iteration-settings seed dirties exactly
    group members.
14. **R4.14 — Concurrency-prep closure (M).** §10: trace RefCell →
    value-returned buffers, `assert_send` tests over coordinator state,
    seed-permutation determinism test. Acceptance: no interior mutability
    reachable from the optimized evaluation path; determinism test green.

Sequencing notes: R4.2→R4.3 is the independent single-sheet consumer slice
(nothing gates on D2 or federation, but R4.2 consumes D1 R2.6/R2.7 outputs
— `GridDerivedState` and the `GridInputSnapshotId` stamp — so it starts
when those land). R4.1 (tree callable regression pinning) has no
dependency on R4.2/R4.3 and starts immediately. R4.4→R4.5→R4.6→
R4.7 is the federation spine (strict chain). R4.8 needs R4.6; R4.9 is
independent of federation (tree-runtime lane) and can run in parallel with
R4.4-R4.7; R4.10 needs R4.6 + R4.9; R4.11/R4.12 need R4.6 + the D2 design;
R4.13 needs R4.10 (full node space); R4.14 can land any time after R4.6
but before any W053 concurrency work opens. The W055 engine build is
outside R4 (owned by `calc-9ouy` beads) and unblocks at R4.13.
