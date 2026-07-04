# W062 D1 — Structural Model Design

## Status

R1 design document for bead `calc-5kqg.4`, authored 2026-07-04 against OxCalc
HEAD `e069136e` (verified anchors below re-checked against the working tree
on the authoring date). Program authority:
`docs/worksets/W062_IDEAL_ENGINE_MODEL_REWORK.md`. Harvested inputs:
`docs/worksets/W062_R0_STASH_TRIAGE.md` (the `calc-uanv` case-insensitivity
interaction, §Sheet registry below). Scope: the structural document model —
roles, sheet registry and order, workbook-as-workspace, meta-children as the
property mechanism, workbook calc-settings home, grid backings entering W057
revision retention, and the serde/migration approach. This document answers
every D1 open question in the W062 plan (§Open questions answered) and ends
with the R2 implementation-wave bead breakdown.

No-legacy stance applies throughout: the design is the ideal shape; downstream
adaptation costs (DnaTreeCalc, tracecalc CLI) are recorded as facts where they
exist, never as constraints.

## Verified starting anchors

- `StructuralNode { node_id, kind, symbol, parent_id, child_ids }` with
  derived `StructuralNodeKind { Root, Container, Calculation, Constant }`
  (`src/oxcalc-core/src/structural.rs:47-62`). Kind is a calc-DAG role
  derived from formula text (`consumer.rs:1599`,
  `node_kind_for_formula_text`), not an authored document fact.
- `NodeBacking { Table, Grid }`, orthogonal to kind, held in a side map
  `node_backings: BTreeMap<TreeNodeId, NodeBacking>`
  (`structural.rs:103-107`, `:261`).
- Serde migration precedent: `StructuralSnapshotWire` with
  `#[serde(default)]` optional maps and a read-side fold of the legacy
  `table_shapes` map into `node_backings` (`structural.rs:265-313`).
- Snapshot validation is centralized in `validate()` + `build_path_index()`
  (`structural.rs:658-761`); every constructor and every `apply_edit` funnels
  through it — the natural home for role invariants.
- Grid backings are live-only: `grids: Arc<BTreeMap<TreeNodeId,
  GridBackingState>>` on `OxCalcTreeWorkspaceState` (`consumer.rs:1329`),
  with the in-code note "per-revision retention (undo/redo of grid edits)
  lands with the grid edit verbs". `RetainedWorkspaceRevisionState`
  (`consumer.rs:1342-1359`) has **no grids field**; retained revisions do
  not capture sheet content. Candidate overlays clone the whole workspace
  state (`consumer.rs:1394`), so grid cost multiplies per candidate today.
- `GridBackingState` bundles authored truth and derived state in one struct:
  `authored_addresses`, `sheet: GridOptimizedSheet`, `published`,
  `differential_mismatches`, `interest`, `published_overlays`, epochs
  (`consumer.rs:1058-1074`).
- Meta-nodes are a workspace-state side set, not a structural fact:
  `meta_node_ids: BTreeSet<TreeNodeId>` (`consumer.rs:1323`), maintained by
  `set_node_meta` (`consumer.rs:1642-1666`) and folded into revision
  identity by stringifying the id set into
  `NamespaceSnapshot::meta_node_membership_version`
  (`workspace_revision.rs:280-292`).
- `WorkspaceRevision` holds three snapshots — structure + node inputs +
  namespace — plus `workspace_id`, identity-minted from the workspace id
  and the three snapshot ids
  (`workspace_revision.rs:325-355`, `:946-972`).
- The warm-recalc OOM lesson is codified on `SnapshotLayerState`: unbounded
  bases must enter identity strings as fixed-width digests, never verbatim
  (`workspace_revision.rs:552-604`).
- The only shipped retention-class enum is
  `EdgeValueCacheRetentionClass::W054PendingEphemeralPerEdgeValueCache`
  (`value_cache.rs:49-51`) — the naming/selector-key precedent the grid-side
  classes follow.
- Prior art for deletion facts that outlive the deleted structure:
  `deleted_table_facts: Vec<TreeCalcTableDeletedFact>` on both live and
  retained workspace state (`consumer.rs:1325`, `:1355`).

---

## 1. Roles: explicit, orthogonal, validated

**Decision:** Add `role: Option<NodeRole>` as a field on `StructuralNode`,
with

```rust
#[non_exhaustive]
pub enum NodeRole {
    Workbook,
    Sheet,
    // later: ChartSheet, and other document-artifact roles
}
```

Roles are **authored document facts**, set at node creation or by an explicit
`StructuralEdit::SetNodeRole` edit, never derived. `None` means "plain tree
node" and stays the default for everything TreeCalc builds today. The derived
`StructuralNodeKind` and the `NodeBacking` enum are untouched — kind keeps
describing the calc-DAG position, backing keeps describing owned sub-models,
role describes the document vocabulary position. All three are orthogonal
axes on the same node.

**Representation alternatives rejected:**

- *Overload `StructuralNodeKind`* — rejected by the program plan itself
  (kind is derived from formula text; a rename of `=A1+1` cannot change
  whether the node is a Sheet) and by bead scope.
- *Side registry map (`node_roles: BTreeMap<TreeNodeId, NodeRole>` like
  `node_backings`)* — workable, but the side-map shape exists for `backings`
  because backings are heavy per-node structs that most nodes lack. A role is
  a two-byte enum that `validate()` must consult while walking nodes anyway;
  a field keeps the invariant checks single-pass and keeps
  clone/edit/builder code from maintaining a second map. The side map buys
  nothing here.
- *Profile-assigned roles* — rejected on principle: a workbook is a workbook
  regardless of which reference profile is bound. Profiles carry *vocabulary*
  (what the role is called, how `!` resolves against it — D2's layer); the
  structure carries the role itself. If roles lived in the profile, two
  profiles could disagree about what the document *is*.

**Assignment and validation.** Role invariants are enforced in `validate()`
(`structural.rs:658`), so they hold for every constructor and every
`apply_edit` product, with new typed `StructuralError` variants (typed
rejection, per program doctrine — never a silent pick):

1. `NodeRole::Workbook` is legal only on the snapshot root
   (`parent_id == None` and `node_id == root_node_id`). Violation:
   `StructuralError::WorkbookRoleRequiresRoot { node_id }`.
2. `NodeRole::Sheet` is legal only on a direct child of a Workbook-role
   root. Violation: `StructuralError::SheetRoleRequiresWorkbookParent
   { node_id }`. (When ChartSheet lands it shares this rule.)
3. Sheet-role siblings must have case-insensitively unique symbols
   (§2). Violation: `StructuralError::DuplicateSheetName { normalized,
   node_ids }`.
4. `StructuralEdit::MoveNode` that would carry a Sheet-role node under a
   non-Workbook parent fails validation (the same rule as 2, reached via the
   normal build-then-validate path — no special-case move code).

Deliberately **not** validated structurally: "Sheet role requires a Grid
backing". A sheet is created empty and gets its backing when content arrives
(ingest ordering, interactive add-sheet). "Every strict-excel sheet has a
grid" is a profile-level projection constraint and belongs to D2's
strict-excel vocabulary, not to the general model. Likewise a Grid backing on
a role-less tree node stays legal — that is today's TreeCalc shape
(`consumer.rs:1329` grids are keyed by arbitrary `TreeNodeId`) and the
general model keeps it.

**Rationale:** The vision (`W062` §Vision 1) requires the general model to
subsume both trees and workbooks. A nullable role field is the smallest
construct that makes "this workspace is a workbook document" a checkable,
persisted, revision-identified fact while leaving every existing tree
workspace valid with zero changes. Putting enforcement in `validate()` means
the role system inherits the snapshot kernel's existing guarantee: no
constructor or edit path can produce an invalid document.

Meta-children (§4) never carry roles: `role.is_some() && is_meta` is a
validation error. Roles are document vocabulary; meta-children are property
plumbing.

## 2. Sheet registry: rename-stable lookup, tombstoned deletion

**Decision:** The sheet-name→node registry is a **derived index inside
`StructuralSnapshot`**, built by the same pass as `path_index`
(`structural.rs:729`): `sheet_index: BTreeMap<NormalizedSheetName,
TreeNodeId>`, populated from Sheet-role children of the root (the map is
key-ordered; enumeration order comes from `sheet_nodes()`, which walks
`child_ids`). It is not serialized (rebuilt on deserialize — and note
`path_index` *is* currently serialized;
the wire rework in §8 stops serializing both derived indexes).

**Name normalization.** `NormalizedSheetName` is a newtype over the
Unicode-simple-case-folded symbol. Display capitalization lives on
`StructuralNode::symbol` untouched; all lookups, uniqueness checks, and
identity keys use the folded form. This is the Excel contract (sheet names
are case-insensitively unique, case-preserving) and it aligns with
`calc-uanv` (case-insensitive sibling uniqueness): when `calc-uanv` lands
for general siblings, Sheet-role siblings are already there. Per the R0
triage note, any test that manufactures collisions via case-only twins
(`Dup`/`dup`) is invalid in this design from day one.

**Rename stability.** `TreeNodeId` is the stable identity — it already
survives renames (`StructuralEdit::RenameNode` keeps `node_id`,
`structural.rs:546-555`). The registry therefore gets rename-stability for
free: a rename changes the index key, not the node. The contract for
reference healing is the `NameIdentity` pattern proven in the grid
(`grid/machine/invalidation.rs:140`, `GridDependency::NameIdentity` at
`:202`): consumers of a sheet name hold an **identity edge keyed on the
normalized name**, which
- follows renames (rename emits a `SheetRenamed { node_id, old_normalized,
  new_normalized, new_display }` structural fact; D2 decides whether
  formula text rewrites or re-displays), and
- survives deletion in a dormant state, healing when a sheet with the same
  normalized name is created again — exactly how deleted defined names heal
  today (`grid/machine.rs:14290` documents the retained-edge-heals-on-
  recreate contract).

**Deletion contract (the registry's side, not D2's semantics).** Deleting a
sheet node produces a **sheet tombstone**, following the
`deleted_table_facts` precedent (`consumer.rs:1325`):

```rust
pub struct DeletedSheetFact {
    pub node_id: TreeNodeId,            // never reused; next_node_id is monotone (consumer.rs:1630)
    pub normalized_name: NormalizedSheetName,
    pub display_name: String,
    pub sheet_position: usize,          // position in sheet order at deletion
    pub deleted_at_snapshot_id: StructuralSnapshotId,
    pub grid_input_identity: Option<GridInputSnapshotId>, // §7; what the sheet held
}
```

Tombstones live on `OxCalcTreeWorkspaceState` as `deleted_sheet_facts`, are
captured into `RetainedWorkspaceRevisionState` (again the
`deleted_table_facts` shape, `consumer.rs:1355`), and are **not** part of the
structural snapshot (deletion history is workspace history, not document
shape — the same reasoning that keeps `deleted_table_facts` out of the
snapshot). This gives D2 everything the `#REF!` policy needs: which node id
died, under what name, at which position, holding what content — while
leaving what a dangling `Sheet1!A1` *evaluates to* entirely D2's decision.
Undo of the deletion (revision navigation, §5) restores the sheet with its
node id, so identity edges keyed on either node id or normalized name heal.

**Rationale:** Deriving the index keeps one source of truth (the tree) —
there is no way for the registry and the structure to disagree, which is the
entire failure mode of a maintained side registry. Tombstones-as-workspace-
facts reuse a shipped pattern rather than inventing snapshot-resident
graveyards that would bloat structural identity with history.

## 3. Sheet order and non-sheet root children

**Decision:** **Sheet order is the root's `child_ids` order filtered to
Sheet-role children.** There is no separate sheet-order vector.

`child_ids` is already ordered and edit-maintained (`structural.rs:61`,
insert-at-index and move-with-index at `:843-966`). The workbook root may
freely interleave non-sheet children — meta-children (`#workbook-settings`,
§4), plain containers, tree calculation nodes — among sheet nodes.
Sheet-facing operations project through the filter:

- `sheets()` enumeration = root children with `role == Some(Sheet)`, in
  `child_ids` order. Exposed on the snapshot
  (`StructuralSnapshot::sheet_nodes()`) and surfaced by the consumer readout.
- "Move sheet to sheet-position *k*" is a helper that maps sheet-position →
  raw child index (position of the *k*-th Sheet-role child) and issues a
  normal `MoveNode`. No new edit variant.
- 3D spans (`Sheet1:Sheet3`, D2/D3) are **defined over the filtered sheet
  order**: the span covers every Sheet-role node between the two endpoints
  inclusive, in sheet order, skipping interleaved non-sheet children. This
  is the exported contract (§Contracts); the grammar and dependency shape
  are D2/D3's.

**Alternative rejected:** a dedicated `sheet_order: Vec<TreeNodeId>` on the
snapshot or workspace. Two ordered lists over the same children is a drift
factory — every insert/move/delete would need twin maintenance and a new
validation rule ("sheet_order is a permutation of Sheet-role children").
The filter costs O(root-children) per enumeration, root fan-out is small,
and the derived `sheet_index` (§2) can carry positions if a hot path ever
appears.

**Rationale:** Single source of truth for order, zero new edit surface, and
Excel's own semantics (sheet tabs are an ordered list; nothing else in the
workbook has document order) fall out of the projection. The general model
keeps its full generality: a workbook root is just a root whose Sheet-role
children happen to be what the strict-excel projection enumerates.

## 4. Meta-children as the property mechanism

**Decision:** Structured properties on nodes are **meta-children**, and the
meta flag is **promoted into the structural node**: `is_meta: bool` on
`StructuralNode` (serde default `false`), replacing the workspace-state side
set `meta_node_ids` (`consumer.rs:1323`) and the stringified
`meta_node_membership_version` namespace hack
(`workspace_revision.rs:280-292`).

Mechanics:

- A property group is a meta child with a reserved-prefix symbol (`#`
  prefix: `#workbook-settings`, `#sheet-view`, …). Individual properties are
  meta grandchildren whose values are ordinary `NodeInputRecord`s (literal
  text / typed literal). The `#` prefix is rejected on non-meta nodes by
  `validate()`, so property paths can never collide with user symbols, and
  meta nodes stay namespace-excluded by construction rather than by side-set
  lookup.
- `is_meta` is inherited-checked: a child of a meta node must itself be meta
  (validation rule). Property subtrees are opaque to name resolution as a
  unit.
- Because meta nodes are ordinary structural nodes with ordinary node
  inputs, they are **automatically** revision-identified (structure snapshot
  id + node-input snapshot id both feed `workspace_revision_identity`,
  `workspace_revision.rs:946`), revision-retained, candidate-overlaid, and
  undoable. No new snapshot layer, no new identity plumbing.

What gets **retired**: `meta_node_ids` on live and retained workspace state,
`set_node_meta`'s side-set maintenance (`consumer.rs:1642-1666` — the verb
survives, now issuing a structural edit), `with_meta_node_ids` /
`meta_node_membership_version` on `NamespaceSnapshot`, and
`refresh_meta_namespace_snapshot`. The namespace snapshot returns to
describing the *namespace environment*; meta membership is a structural
fact and changes structural identity directly. This is a strict
simplification: one mechanism (structure) instead of two (structure + side
set folded into a third layer's identity string).

**Rationale:** The W062 plan already names meta-nodes "the ready-made
property mechanism". The only defect in the shipped mechanism is that
meta-ness lives outside the structure it describes, which forced the
namespace-version workaround to keep revision identity honest. Under the
no-legacy stance the fix is to move the fact where it belongs, not to keep
compensating. Downstream fact: DnaTreeCalc's `SetNodeMeta` edit call sites
keep working (the verb remains); only the wire shape of persisted snapshots
changes (§8).

## 5. Workbook calc-settings home

**Decision:** Workbook calculation settings — date system (1900/1904), calc
mode (automatic/manual), iteration settings (enabled, max iterations, max
change) — live in a **`#workbook-settings` meta-child of the workbook
root**, one meta grandchild per setting, values as typed literal node
inputs. A typed accessor pair on the context wraps the raw nodes:

```rust
pub struct WorkbookCalcSettings {
    pub date_system: DateSystem,          // Excel1900 (default) | Excel1904
    pub calc_mode: CalcMode,              // Automatic (default) | Manual
    pub iteration: IterationSettings,     // enabled=false, max_iterations=100, max_change=0.001
}
context.workbook_calc_settings(&workspace_id) -> WorkbookCalcSettings   // absent nodes ⇒ defaults
context.set_workbook_calc_settings(&workspace_id, settings) -> Result<…>
```

Absence of the meta-child means defaults — a freshly created workbook
carries no settings nodes and is identical to Excel's defaults. The accessors
are the only sanctioned read/write path; the raw meta nodes are the storage
representation, not API.

**Revision-identity participation:** automatic and exact, via §4 — a
settings change is a node-input edit, which changes the node-input snapshot
id, which changes the `WorkspaceRevision` id. No fifth revision component,
no namespace-version threading. Undo of a settings change is ordinary
revision navigation.

**Alternative rejected:** a dedicated settings field or fifth snapshot layer
on `WorkspaceRevision` (`workspace_revision.rs:325`). It would duplicate the
identity, retention, candidate-overlay, and undo plumbing that meta-children
inherit for free, for a handful of scalars — and it would contradict the
vision statement that nodes-with-meta-children *are* the property mechanism.
If a future setting turns out to be so hot that node-input reads measurably
drag, a read-cache on the accessor is the fix, not a structural relocation.

**Invalidation contract (contract only; D3 owns mechanics).** The settings
verbs emit typed invalidation seeds, extending the existing
`pending_invalidation_seeds` channel (`consumer.rs:1332`):

- `WorkbookSettingChanged::DateSystem` — semantic invalidation: every
  date-bearing computation is stale. D1 guarantees the seed carries old and
  new values; D3 decides the dirty cone (the correct-and-simple oracle answer
  is "all"; the optimized lane may narrow to date-tainted subgraphs).
- `WorkbookSettingChanged::CalcMode` — scheduling fact, **no value
  invalidation**. D3's recalc driver consults it; no formula becomes stale
  because evaluation timing changed.
- `WorkbookSettingChanged::Iteration` — invalidates exactly the members of
  existing cycle groups (`dependency.rs` cycle_groups are already revision
  facts, `workspace_revision.rs:1063-1080` folds them into dependency-shape
  identity). D3/W055 own re-evaluation semantics.

D1's guarantee to D3: settings are revision-identified state whose every
change is (a) visible in revision identity, (b) delivered as a typed seed,
and (c) attributable to a transaction in the revision graph. Nothing about
convergence, scheduling, or dirty-cone computation is decided here.

## 6. Workbook-as-workspace

**Decision:** **One workspace is one workbook** (the plan's default,
confirmed — not the multi-workbook container). A workbook is precisely a
workspace whose root carries `NodeRole::Workbook`. The existing multi-
workspace `OxCalcTreeContext` (`consumer.rs:1430-1439`, `workspaces:
BTreeMap<OxCalcTreeWorkspaceId, …>`) **is** the multi-workbook container.

Operationally, "a second workbook" means:

- `create_workspace` grows a workbook flavor (or an option on
  `OxCalcTreeWorkspaceCreate`) that stamps the root with
  `NodeRole::Workbook`; the workspace id is the workbook's document
  identity. Plain (role-less) tree workspaces remain fully supported
  side-by-side in the same context.
- Everything that is per-workspace today is per-workbook by construction:
  revision graph and retention policy (`consumer.rs:1312-1316`), candidate
  overlays, publication state, undo. This is the decisive argument — the
  workbook is exactly the unit users expect to undo, snapshot, and save, and
  the workspace already is that unit. A workbook-inside-a-bigger-workspace
  would need sub-tree-scoped revision identity, which the W057 substrate
  does not have and should not grow.
- Cross-workbook references (`[Book2]Sheet1!A1`) are cross-*workspace*
  references. The seam already exists in the dependency layer:
  `DependencyDescriptor.workspace_target` with workspace handle, target
  node, and availability version is folded into dependency-shape identity
  today (`workspace_revision.rs:1032-1043`), and the namespace snapshot
  carries `workspace_availability_version`/`workspace_alias_version`
  (`workspace_revision.rs:230-231`). Whether external references are in or
  out of strict-excel scope stays **D2's explicit in/out decision** (it is
  on D2's question list); D1's commitment is only that the structural answer
  is "another workspace in the same context", so D2 never needs a new
  container concept.
- Sheet moves *between* workbooks (Excel's move-sheet-to-another-book) are
  out of R2 scope; the general model expresses them as delete+recreate with
  content transfer at the document surface (D4), not as a structural
  cross-workspace `MoveNode`.

**Rationale:** The container question is really "where does revision
identity live?", and W057 answered it: per workspace. Aligning the document
boundary with the revision boundary makes every document-level feature
(undo over sheets, settings identity, neutral output "edits since revision")
fall out of shipped machinery.

## 7. Grid backings enter revision retention

This is the largest structural change. Current shape: grids are live-only
(`consumer.rs:1327-1329`), retained revisions capture no sheet content
(`consumer.rs:1342-1359`), so revision navigation today silently *keeps* the
live grids — undo over sheet edits is unsound the moment grid edit verbs
exist. The in-code note says exactly this lands "with the grid edit verbs";
those verbs are R2/R5 program deliverables, so retention lands in R2.

### 7.1 Split authored truth from derived state

**Decision:** `GridBackingState` (`consumer.rs:1058-1074`) is split along
the same line the tree already draws between `NodeInputSnapshot` (authored,
revision-identified) and publication/runtime layers (derived, recomputable):

- **`GridInputState`** (authored truth, revision-retained): the authored
  cell inputs (address → authored input record: literal/formula text, the
  grid analog of `NodeInputRecord`), merged-region declarations, and other
  authored facts. Content-addressed: `GridInputSnapshotId` minted from a
  streamed fold of the authored records, with the unbounded basis entering
  any downstream identity **only as a digest** — the
  `identity_basis_digest` lesson (`workspace_revision.rs:552-604`) applies
  from day one, because grid bases are unbounded in exactly the way that
  caused the warm-recalc OOM.
- **`GridDerivedState`** (live-only, evictable): `sheet:
  GridOptimizedSheet` engine state, `published`, `published_overlays`,
  `differential_mismatches`, `interest`, epochs. Never retained per
  revision; recomputable from `GridInputState` + a recalc.

### 7.2 Which snapshot layer

**Decision:** Grid authored content joins the **revision tuple itself** — a
new input component alongside the three existing snapshots, not one of the
four derived layer snapshots.
`WorkspaceRevision` (`workspace_revision.rs:325-331`) grows:

```rust
pub struct WorkspaceRevision {
    revision_id: WorkspaceRevisionId,
    pub workspace_id: String,
    pub structure_snapshot: StructuralSnapshot,
    pub node_input_snapshot: NodeInputSnapshot,
    pub grid_input_snapshot: GridInputSnapshot,   // NEW: BTreeMap<TreeNodeId, GridInputSnapshotId> + content identity
    pub namespace_snapshot: NamespaceSnapshot,
}
```

with `workspace_revision_identity` (`workspace_revision.rs:946`) extended by
the `grid_input_snapshot_id` field. `GridInputSnapshot` is a thin identity
map (node id → per-grid content identity); the cell payloads live in the
retained store (§7.3), not inside the revision value. Rationale for
tuple-membership rather than a derived layer: authored cells are *input*
truth exactly like authored node formulas — two revisions that differ only
in a cell literal MUST have different revision ids, or undo, candidate
comparison, and neutral output ("edits since revision R", D4) are all
unsound. The four derived layers (`FormulaBindingSnapshot` …
`RuntimeOverlaySet`) describe computation *about* a revision; grid inputs
help *define* the revision. Workspaces with no grids carry an empty
snapshot whose identity token is a constant — tree-only revision ids
change (a wire-visible fact, §8), but tree-only workflows are otherwise
untouched.

### 7.3 Retained artifact: COW full-state, per-node Arc

**Decision:** The retained artifact is the **full immutable
`Arc<GridInputState>` per grid-backed node**, shared structurally across
revisions — not an edit-log/delta chain.

- The live map becomes `grids: BTreeMap<TreeNodeId, GridNodeState>` where
  `GridNodeState { input: Arc<GridInputState>, derived: GridDerivedState }`.
  An edit to one sheet `Arc::make_mut`s that sheet's input only; every other
  sheet's `Arc` is shared untouched. (Today's single outer
  `Arc<BTreeMap<…>>` with `Arc::make_mut` at `consumer.rs:1378-1381` clones
  the *map* on first write while sharing the `GridBackingState` values only
  until any one of them is touched — per-node `Arc` makes sharing
  per-sheet, which is what makes retention and candidate overlays cheap.)
- `RetainedWorkspaceRevisionState` gains `grid_inputs:
  BTreeMap<TreeNodeId, Arc<GridInputState>>`. Retaining a revision is O(grid
  count) pointer copies. A retention window over N revisions where only one
  sheet was edited costs one `GridInputState` clone per edit, not N × sheets.
- Revision navigation (undo/redo) swaps the live `input` Arcs to the
  retained ones and marks derived state stale; derived state is rebuilt by
  recalc (oracle-correct by construction; the optimized lane may later warm
  from caches — D3's business).

**Edit-log replay rejected as the primary artifact:** navigation must be
O(switch), not O(replay-since-ancestor); replay correctness would need its
own differential harness against full-state; and the content-addressed
identity story (revision id from content identity) is natural for state,
awkward for logs. The door stays open as a *compression* strategy: W054 may
later demote cold retained `GridInputState`s to delta form under memory
pressure, behind the same retention-class contract — an eviction-policy
refinement, not a model change.

### 7.4 Retention classes and the W054 contract

**Decision:** Define the grid-side retention classes now (D1 owns the class
contract; the register records W054 owns eviction/pinning of classes D1
defines), following the `EdgeValueCacheRetentionClass` naming/selector-key
precedent (`value_cache.rs:49-60`):

```rust
pub enum GridRetentionClass {
    /// Authored grid content pinned by a retained workspace revision or a
    /// candidate overlay basis. Evictable ONLY by evicting the owning
    /// revision through the workspace revision retention policy; W054 GC
    /// must treat these as revision-pinned, never age-based.
    RevisionRetainedGridInput,
    /// Derived grid state (engine sheets, published cells, overlay
    /// projections). Recomputable; evictable at any time under memory
    /// pressure at the cost of a recalc.
    EphemeralDerivedGridState,
}
```

Policy shape: **no separate grid retention budget in R2.** The existing
`OxCalcTreeRevisionRetentionPolicy` (`consumer.rs:1316`, enforced by
`enforce_workspace_revision_retention_policy`) transitively bounds
`RevisionRetainedGridInput` — when a revision is evicted from the retained
window, its grid-input Arcs drop with it, and structural sharing means the
bytes free only when the last retaining revision goes. W054's GC re-target:
(a) its eviction machinery gains the two classes above with the stated
pinning rules; (b) candidate-pinned revisions
(`candidate_pinned_workspace_revisions`, `consumer.rs:1315`) pin grid inputs
exactly as they pin revisions today; (c) an optional byte-budget knob over
retained grid bytes is left as a **named seat** in the policy struct for
W054 — D1 defines that evicting under that knob means evicting whole
revisions (oldest-unpinned-first), never tearing individual sheets out of a
retained revision (a revision with holes is not a revision).

## 8. Serde and migration

**Decision:** Follow the `StructuralSnapshotWire` precedent
(`structural.rs:265-313`) — a wire struct with explicit `From` conversions,
`#[serde(default)]` for additive fields — with these explicit calls under
the no-legacy stance:

1. **Additive node fields** (`role`, `is_meta`) deserialize with defaults
   (`None`/`false`). Every snapshot main writes today loads unchanged
   (verified for the `#`-prefix rule too: no `#`-prefixed node *symbols*
   exist in current usage — only structured-reference section tokens inside
   formula text, which are unaffected). No
   role inference on load: roles are authored facts; a pre-role snapshot
   loads as a plain tree, and a host that wants a workbook uses the workbook
   verbs. (Downstream fact: DnaTreeCalc's persisted TreeCalc workspaces are
   in this category and keep loading.)
2. **The pre-`NodeBacking` legacy fold is removed.** The wire-side
   `table_shapes` map, the read-side fold (`structural.rs:297-313`), and the
   back-compat constructor `create_with_table_shapes` /
   `table_shapes()` accessor (`structural.rs:326-337`, `:400-410`) are
   deleted. That migration bridged one generation and its snapshots have had
   a full load-rewrite cycle available. **Explicit readability decision:**
   snapshots serialized before the `NodeBacking` migration become
   unreadable; snapshots of the current generation (`node_backings` form)
   remain readable. Recorded here deliberately, not silently.
3. **Derived indexes leave the wire.** `path_index` is currently serialized
   (`structural.rs:281`) despite being derivable; the new `sheet_index`
   never enters the wire. The wire rework drops `path_index` from the wire
   struct and rebuilds both indexes in `From<Wire>` via the validated
   constructor. Smaller payloads, one less internally-inconsistent-input
   class to defend against.
4. **Revision identity changes are breaking and accepted.** Promoting
   `is_meta` into the structure (§4) and adding the grid-input component
   (§7.2) change `workspace_revision_identity` output for existing content.
   Revision ids are content addresses, not stable external names; persisted
   revision-id strings from before this program do not compare against new
   ones. Any retained-artifact corpus keyed on old ids regenerates
   (local-execution doctrine: regenerate checked-in baselines in the same
   bead that changes identity — the corpus-regeneration env-var mechanism
   already exists for exactly this).
5. **No schema version integer.** The kernel's precedent is field-presence
   migration, which composes (each field migrates independently) where a
   monotone version number forces total ordering of unrelated changes. Keep
   it.

## Open questions answered

Mapping every D1 open question from
`W062_IDEAL_ENGINE_MODEL_REWORK.md` §Open questions for R1:

| D1 question | Answer |
| --- | --- |
| Role representation (kind vs field vs registry) | Field: `role: Option<NodeRole>` on `StructuralNode`, validated in `validate()`; kind-overload and side-registry rejected (§1). |
| Role serde migration | `#[serde(default)]` additive field on the wire struct; no role inference on load; pre-`NodeBacking` legacy fold removed in the same rework (§8). |
| Sheet order vs non-sheet root children | Sheet order = root `child_ids` filtered to Sheet-role children; non-sheet children interleave freely; no second order vector (§3). |
| One-workspace-per-workbook vs multi-workspace container | One workspace per workbook; `OxCalcTreeContext` is the container; cross-workbook = cross-workspace via the existing `workspace_target` seam. External-reference in/out remains D2's decision, on a settled structural substrate (§6). |
| Settings home (meta-nodes vs snapshot layer) and revision-identity participation | Meta-children under the workbook root (`#workbook-settings`); identity participation automatic via node-input snapshot; dedicated snapshot layer rejected (§5). Settings-change invalidation is a typed-seed contract; D3 owns mechanics. |
| Grid retention policy (COW snapshots vs edit-log replay) and W054 GC interplay | COW full-state with per-node `Arc` sharing; edit-log demoted to a possible future W054 compression strategy. Two `GridRetentionClass` variants defined with pinning rules; no separate grid budget in R2 — revision retention policy governs transitively, with a named byte-budget seat for W054 (§7.3–7.4). |

**Deferred (with reasons):** none of the D1 list is deferred. Two adjacent
items are explicitly left to their owners rather than answered here: the
`#REF!`-vs-heal *semantics* of dangling sheet references (D2 — this design
supplies the tombstone/identity information contract only, §2), and the
dirty-cone mechanics of settings invalidation (D3 — this design supplies the
typed seeds only, §5).

## Contracts exported to D2/D3/D4

Other R1 designs may rely on the following without re-deriving them:

- **C1 (D2):** `NodeRole` is a structural fact; profile vocabulary maps
  tokens onto roles (never defines them). Sheet lookup is
  `sheet_index[NormalizedSheetName] -> TreeNodeId`; normalization is Unicode
  simple case fold; display names live on `symbol`.
- **C2 (D2):** Sheet identity for reference healing: `TreeNodeId` is the
  rename-stable identity; normalized-name identity edges follow the grid
  `NameIdentity` heal-on-recreate contract. Deletion delivers
  `DeletedSheetFact { node_id, normalized_name, display_name,
  sheet_position, deleted_at_snapshot_id, grid_input_identity }`, retained
  with revisions.
- **C3 (D2/D3):** Sheet order for enumeration and 3D spans is the filtered
  root `child_ids` order; spans cover Sheet-role nodes between endpoints
  inclusive, skipping interleaved non-sheet children.
- **C4 (D3):** Workbook calc settings are read through
  `workbook_calc_settings()` (defaults on absence); every change is
  revision-visible and arrives as a typed
  `WorkbookSettingChanged::{DateSystem, CalcMode, Iteration}` seed with old
  and new values. CalcMode changes never invalidate values.
- **C5 (D3):** `GridInputSnapshotId` is a content address over authored
  grid truth; two revisions with equal grid-input ids for a node have
  identical authored content for that node at content-address confidence
  (equal ids ⇒ equal content up to digest collision; the R2.6 bead picks
  the hash and may pin a stable algorithm) — valid as a recalc-basis
  and cache key. Unbounded bases enter downstream identities only as
  digests.
- **C6 (D3/D4):** Revision navigation restores authored grid truth exactly
  (Arc swap) and marks derived grid state stale; rebuild-by-recalc is the
  correctness baseline. D4's "edits since revision R" neutral output diffs
  grid-input snapshots, never derived state.
- **C7 (W054):** `GridRetentionClass::RevisionRetainedGridInput` is
  revision-pinned (evict by evicting the owning revision, whole revisions
  only, oldest-unpinned-first); `EphemeralDerivedGridState` is
  freely evictable at recalc cost. Candidate pins imply grid-input pins.
- **C8 (D4):** A workbook is a workspace whose root has
  `NodeRole::Workbook`; the document surface's load/save/enumerate verbs
  target one workspace; multi-document hosting is context-level.

## R2 implementation-wave bead breakdown

Ordered; each lands green on `main` per the W062 execution process (decisive
acceptance check, fresh-eyes review instructions, and
`cargo test -p oxcalc-core` + clean differential in every bead description
when R2 beads are created). Sizes: S ≲ half day, M ≈ 1 day, L ≈ 2 days.

1. **R2.1 — `NodeRole` on `StructuralNode` + validation + wire field (M).**
   Add the field, the enum, the four typed `StructuralError` variants, and
   the `validate()` rules of §1 (including move-under-non-workbook
   rejection). Wire: `#[serde(default)]`. Acceptance: role invariants
   rejected with typed errors from every constructor and `apply_edit` path
   (note: the deserialize path `From<StructuralSnapshotWire>` bypasses
   `validate()` today and is only closed by R2.9 routing it through the
   validated constructor — R2.1's guarantee is complete once R2.9 lands);
   existing snapshots load with `role: None`.
2. **R2.2 — Promote `is_meta` into `StructuralNode`; retire the side set
   (M).** Move meta membership into the structure with the `#`-prefix and
   meta-child-inheritance validation of §4; rewire `set_node_meta` and
   creation-time `is_meta` to structural edits; delete `meta_node_ids`
   (live + retained), `with_meta_node_ids`, `meta_node_membership_version`,
   `refresh_meta_namespace_snapshot`. Acceptance: namespace-exclusion tests
   still pass; revision identity changes when meta membership changes, via
   structural snapshot id alone.
3. **R2.3 — Sheet registry index + case-insensitive Sheet-sibling
   uniqueness (M).** `NormalizedSheetName`, `sheet_index` built alongside
   `path_index`, `sheet_nodes()` ordered enumeration, `DuplicateSheetName`
   validation. Coordinate with `calc-uanv` so the fold rule is shared, not
   duplicated. Acceptance: rename keeps node id and updates the index;
   case-twin sheet names are rejected; enumeration order tracks `child_ids`.
4. **R2.4 — Workbook/sheet lifecycle verbs + deletion facts (L).**
   Workbook-flavored workspace creation; `add_sheet` / `rename_sheet` /
   `move_sheet(sheet_position)` / `delete_sheet` context verbs;
   `DeletedSheetFact` on live and retained state; `SheetRenamed` structural
   fact emission. Type note: `GridInputSnapshotId` is minted in R2.6 — R2.4
   declares it as an opaque newtype (or ships `grid_input_identity: None`)
   and R2.7 starts populating it. Acceptance: verb round-trip test building
   a 3-sheet workbook, reordering, renaming, deleting; tombstone carries
   the §2 fields; revision retention captures deletion facts.
5. **R2.5 — Workbook calc-settings home + typed accessors + seeds (M).**
   `#workbook-settings` meta subtree, `WorkbookCalcSettings` read/write
   accessors with defaults-on-absence, `WorkbookSettingChanged` seed
   emission into `pending_invalidation_seeds` (seed *emission* only — no
   recalc semantics). Acceptance: settings change alters revision id;
   undo restores prior settings; seeds carry old/new values; CalcMode seed
   emits no value invalidation.
6. **R2.6 — Split `GridBackingState` into `GridInputState` /
   `GridDerivedState`; mint `GridInputSnapshotId` (L).** The §7.1 split with
   per-node `Arc<GridInputState>`, content-addressed identity with
   digest-fold discipline, live map reshaped to `GridNodeState`. No
   retention yet. Acceptance: identity equal ⇔ authored content equal
   (property test); derived state rebuilds from input state alone; grid
   read/recalc behavior byte-identical to before the split (existing grid
   consumer tests green unchanged).
7. **R2.7 — `GridInputSnapshot` joins `WorkspaceRevision`; retained
   revisions capture grid inputs; navigation restores them (L).** §7.2 tuple
   extension + identity field; `RetainedWorkspaceRevisionState.grid_inputs`;
   navigate swaps Arcs and invalidates derived state; candidate overlays
   share per-node Arcs. Regenerate identity-keyed baselines in this bead.
   Acceptance: edit-cell → undo → redo round-trip restores authored grid
   truth exactly; a retention window over N revisions with one edited sheet
   holds one extra `GridInputState`, verified by Arc strong-count or
   allocation-count evidence.
8. **R2.8 — `GridRetentionClass` contract + retention-policy wiring +
   W054 seat (S).** The §7.4 enum with selector keys, transitive eviction
   through `enforce_workspace_revision_retention_policy`, the named
   byte-budget seat (documented, not implemented), and the W054 re-target
   note recorded in the workset register. Acceptance: evicting a revision
   drops its otherwise-unshared grid-input Arcs; candidate-pinned revisions
   never drop theirs.
9. **R2.9 — Wire rework: drop legacy `table_shapes` fold and serialized
   `path_index` (S).** §8 items 2–3: delete the legacy wire field, fold,
   `create_with_table_shapes`, `table_shapes()` accessor; rebuild derived
   indexes on deserialize. Acceptance: current-generation snapshot fixtures
   round-trip; a checked-in pre-`NodeBacking` fixture is *removed* with the
   readability decision cited in the commit message.
10. **R2.10 — Consumer readout: `sheets()` enumeration surface (S).**
    Ordered sheet enumeration (node id, display name, normalized name,
    sheet position, grid-backing presence) on the context readout, per C3/C8
    — the surface D4's document verbs and DnaTreeCalc's sheet tabs consume.
    Acceptance: enumeration reflects lifecycle verbs from R2.4 across
    revisions and candidates.

Sequencing notes: R2.1–R2.3 are independent of the grid lane and can
interleave; R2.4 needs R2.1+R2.3; R2.5 needs R2.2; R2.6→R2.7→R2.8 is a
strict chain; R2.9 can land any time after R2.1/R2.2 (it touches the same
wire struct — do it after both to avoid triple-touching), but R2 does not
close without it: R2.9 also closes the deserialize-bypasses-`validate()`
gap that R2.1's acceptance depends on; R2.10 last.
D2 (vocabulary/resolution) consumes C1–C3 and can design in parallel from
this document; D3 consumes C4–C6; nothing in R2 blocks on D2/D3 designs.
