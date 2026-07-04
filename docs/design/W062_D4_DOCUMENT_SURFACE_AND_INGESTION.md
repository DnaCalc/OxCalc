# W062 D4 — Document Surface and Ingestion Design

## Status

R1 design document for bead `calc-5kqg.7`, authored 2026-07-04 against OxCalc
HEAD `e069136e`, oxdoc-model at the current OxDoc sibling working tree, and
OxFml's shipped authored-input surface (anchors below verified on the
authoring date). Program authority:
`docs/worksets/W062_IDEAL_ENGINE_MODEL_REWORK.md` (Vision pillar 4, Target
directions 5 and 6). Companion design: `docs/design/W062_D1_STRUCTURAL_MODEL.md`
— its exported contracts C1–C8 are treated as FIXED here, especially C3/C8
(sheets() readout and workbook-as-workspace), C4 (calc-settings home), C6
("edits since revision R" diffs grid-input snapshots, never derived state),
and the R2.4 workbook lifecycle verbs. D2 (reference architecture) and D3
(calculation) are authored in parallel; frictions with their default
directions are collected in §Cross-design tensions rather than silently
assumed away.

Scope: the evolution of `OxCalcTreeContext` into the document context — the
document-surface verbs (load-workbook-model, public formula binding,
defined-name seeding, authored readout, ClearCell, calc-settings access,
neutral output) — the W059 authored-input lane (absorbed here in full), and
full-scope `oxdoc-model` ingestion with honest tiers, including a named
disposition for every one of the 29 `DocumentEvent` variants. Ends with the
R5/R6 implementation-wave bead breakdown.

No-legacy stance applies: this is the ideal shape. Downstream facts
(DnaTreeCalc W011's paused plan, its hand-keyed fixture const, the tracecalc
CLI) are recorded where they exist, never treated as constraints.

## Verified starting anchors

OxCalc consumer surface (`src/oxcalc-core/src/consumer.rs`):

- `OxCalcTreeContext` (`consumer.rs:1430`) already owns multiple workspaces,
  candidates, and revision graphs — D1 §6 confirms it IS the multi-workbook
  container (C8).
- Grid verbs today: `set_node_grid(node, GridBackingSeed)` (`:3290`),
  `clear_node_grid` (`:3383`), `grid_view` (`:3423`), `register_grid_interest`
  (`:3467`), `poll_grid_changes` (`:3491`), `apply_grid_edit` with
  `OxCalcTreeGridOp::{SetCell, FillRange}` (`:3532`). `apply_grid_edit` does
  **not** advance the workspace revision (grid edits are revision-invisible
  until D1 R2.6/R2.7 land) and there is **no** ClearCell, no authored readout
  (`OxCalcTreeGridCellReadout` is value+epoch only, `:999`), no consumer-level
  defined-name verb, and no public formula binding.
- `GridAuthoredCell::{Literal(CalcValue), Formula(GridFormulaCell)}` with
  `GridFormulaCell { source_text, normal_form_key, source_channel }`
  (`grid/authored.rs`) — the caller must manufacture `normal_form_key` today;
  the only mint, `bind_grid_formula_for_transform`, is `pub(super)`
  (`grid/machine/optimized_sheet.rs:7241`), binding through
  `parse_formula`/`bind_formula` with the strict-excel profile
  (`EXCEL_GRID_PROFILE_ID = "excel.grid.v1"`, `grid/reference_engine.rs:43`).
- Defined names are complete at the sheet-machine level and unreachable from
  the consumer: `set_defined_name` / `set_sheet_defined_name` /
  `set_dynamic_defined_name` / `set_sheet_dynamic_defined_name` /
  `rename_defined_name` / `delete_defined_name`
  (`grid/machine/calc_ref_sheet.rs:461-807`, mirrored on the optimized
  sheet), returning `GridNameLifecycleReport`s with dirty seeds; `#NAME?`
  self-healing via retained `NameIdentity` edges is a documented machine
  contract (`grid/machine.rs:14290`).
- The W059 authored-input authority is already partially wired:
  `interpret_authored_input` → `RuntimeAuthoredInputResult::{Literal,
  Formula, Diagnostics}` is consumed by the tree input path
  (`consumer.rs:1693-1700`, `treecalc.rs:8532`), with typed
  `AuthoredInputDiagnostics` rejection (`consumer.rs:776`) and
  no-mutation-on-rejection. The grid path bypasses it entirely.
- The grid overlay enum reserves inert seats: `OverlayKind::{Cse,
  ConditionalFormat, RichObject, Extension}` with the `GridOverlayExtension`
  carrier `{ kind_tag, claimed_rect, block_mode, refuses_axis_edit, payload }`
  (`grid/machine/overlay.rs:40-102`). Beware: a second, unrelated
  `OverlayKind` exists in `recalc.rs:24` (tree runtime overlays) — every
  reference in this design means the grid one.
- `oxcalc-core` depends on `oxfml_core` + `oxfunc_core` only
  (`src/oxcalc-core/Cargo.toml:15-16`); there is no oxdoc dependency edge
  anywhere in the workspace today.

oxdoc-model (`OxDoc/crates/oxdoc-model/src/lib.rs`):

- `DocumentEvent`, `#[non_exhaustive]`, 29 variants (`lib.rs:2262-2292`) —
  enumerated one-for-one in §12.
- `OxCalcIngestSink { workbook, sheet_begin, cell_chunk, sheet_end,
  feature(default no-op) }` (`lib.rs:2152-2163`), driven by
  `drive_oxcalc_ingest` (`:2616`, borrowed + stream-validated) and
  `drive_oxcalc_ingest_from_model_access` (`:2723`, eager-events only today).
  The driver pre-resolves the prelude (header + string table + style table)
  and hands cells as `OxCalcCellChunk` with typed
  `OxCalcCellInput::{Empty, Literal(OxCalcCellValue), Formula(
  OxCalcFormulaInput), RichStub(u32)}`; formula text arrives as
  `FormulaTextKind::SpreadsheetMlA1` (no leading `=`) or
  `R1C1RelativeTemplate`, with optional `FileCached` cached values
  (`:2062-2101`, `:2563-2584`).
- `WorkbookHeader { date_system, calc_mode, schema }` (`lib.rs:947`) — **no
  iteration settings**; upstream gap, raised in §15.
- `SharedFormulaRegion { region_id, anchor, extent, r1c1_text }` (`:1880`)
  and `FormulaTopology` (`:1893`) with `FormulaRecordKind::{Normal, Shared,
  Array, DataTable, Unknown}` — file-format facts, explicitly "not a parsed
  AST, no binding semantics".
- Output side: `WorkbookModelOutput` with
  `WorkbookModelOutputEntry::{WholeModelProjection,
  SelectedSurfaceMaterialization, ModeledEdit(WorkbookModelEdit)}`
  (`lib.rs:407-459`) and `WorkbookModelEditKind` (`:525`); there is **no
  cell-granular modeled edit** — `Replace(WorkbookSurfacePayload::CellChunk)`
  is a supported *chunk*-granular save path in oxdoc-xlsx
  (`oxdoc-xlsx/src/lib.rs:4172`); the whole-model projection remains the
  path the W011 save recipe uses, and the §15.2 upstream ask is for
  cell-granular edits specifically.
- `LoadProfile::{values_only(default), strict_values_only, full}`
  (`lib.rs:141-238`).

DnaTreeCalc W011 (paused; the fixture class this design must round-trip):
`DnaTreeCalc/docs/ux/DNACALC_HOST_CORE_XLSX_NOTEBOOK_PROOF.md` — fixture
`A1 = 7`, `B1 = =A1*3`; proof = edit A1→10, B1 renders 30, saved workbook
reopens with A1 changed, **B1 formula text preserved and B1 cached value
refreshed to 30**. Its upstream asks (b) public binding, (e) name seeding,
(c) authored readout, (d) output readout are exactly the verbs this design
makes native.

---

# Part I — The document surface

## 1. The document context

**Decision:** `OxCalcTreeContext` is renamed **`OxCalcDocumentContext`** and
becomes the document context. No parallel type, no wrapper: the existing
struct gains the document verbs and loses the tree-only name. Per D1 C8 the
context is the multi-document container; every document verb below is
addressed `(&workspace_id, …)` and targets one workbook workspace. Plain
tree workspaces remain first-class citizens of the same context; document
verbs on a workspace whose root lacks `NodeRole::Workbook` fail with a typed
`NotAWorkbookWorkspace` error (never silently degrade).

Rationale: the W062 plan says the context "evolves into" the document
context precisely because it already owns the right state (workspaces,
candidates, revisions, retention). A second context type would force every
downstream host to hold two handles over one state pot. The rename is
deliberately last in the R5 wave (R5.7) so every functional bead lands
against a stable name and the mechanical rename commits alone.

**Sheet addressing.** Grid verbs keep `TreeNodeId` as the sheet handle (the
rename-stable identity, D1 C2). Convenience resolution by sheet name goes
through the D1 sheet registry (`sheet_index[NormalizedSheetName]`, C1) at
the API edge only — nothing below the context ever routes on a display name.

## 2. Authored input: one OxFml authority (the W059 lane, absorbed)

W059's target doctrine becomes the grid's law, exactly as it already is the
tree's: **OxCalc owns no string-to-value interpretation.** Every authored
text enters OxFml (`interpret_authored_input`, worksheet channel) and comes
back as exactly one of literal `CalcValue`, `BoundFormula`, or diagnostics.
The tree path already does this (`consumer.rs:1693`); the grid path gets the
same spine.

**Decision — three entry verbs:**

```rust
// 1. The universal text entry (Excel cell-entry semantics, OxFml-owned).
pub fn enter_grid_cell(
    &mut self, workspace_id, node_id, address: ExcelGridCellAddress,
    entered_text: &str,
) -> Result<GridCellEntryOutcome, OxCalcDocumentError>;

pub enum GridCellEntryOutcome {
    Literal { value: CalcValue, view: OxCalcTreeGridView },
    Formula { normal_form_key: String, view: OxCalcTreeGridView },
    // Diagnostics are an Err (AuthoredInputDiagnostics), not an outcome:
    // the stored input is untouched, matching the tree contract.
}

// 2. The typed bypass (W059 slice 4): callers already holding a CalcValue
//    never round-trip through text.
pub fn set_grid_cell_value(
    &mut self, workspace_id, node_id, address, value: CalcValue,
) -> Result<OxCalcTreeGridView, OxCalcDocumentError>;

// 3. ClearCell: removes the authored record entirely (authored kind returns
//    to Empty), dirties dependents through the normal engine seed path, and
//    is revision-visible like any other grid input edit.
pub fn clear_grid_cell(
    &mut self, workspace_id, node_id, address,
) -> Result<OxCalcTreeGridView, OxCalcDocumentError>;
```

Semantics, fixed by contract:

- `enter_grid_cell` calls OxFml `interpret_authored_input` with
  `FormulaChannelKind::WorksheetA1` and the full host bind context (caller
  address, workbook/sheet identity, names, tables — the same context §3's
  bind verb assembles). Literal → stored as `Literal(CalcValue)`; formula →
  bound, stored as authored formula (§3's derived-key doctrine); diagnostics
  → typed `Err`, **no mutation** (`=1+` never becomes a stored cell that
  later evaluates to `#VALUE!` — W059 slice 3 verbatim).
- Apostrophe-forced text, error literals, locale/date entries: all OxFml's
  classification. OxCalc's residual grid-local heuristics (none exist in the
  grid path today — the grid only ever accepted typed `GridAuthoredCell`,
  which is why it dodged W059's tree-side bug class) stay banned.
- `ClearCell` vs "set empty text": `enter_grid_cell` with `""` is defined as
  ClearCell (Excel's contract for committing an empty edit), so hosts need
  no special case.
- All three verbs mutate **`GridInputState`** (D1 §7.1 authored truth),
  advance the workspace revision (D1 §7.2 — closing today's
  revision-invisible `apply_grid_edit` gap), and recalculate per the calc
  mode (§6).

**`apply_grid_edit` disposition:** `OxCalcTreeGridOp::SetCell` taking a
pre-built `GridAuthoredCell` remains as the *typed* region edit for hosts
and for ingest (it is the layer under verbs 1–3); `FillRange` stays the
repeated-formula region verb, now fed by `bind_grid_formula` (§3) instead of
hand-built keys. The verbs above are the sanctioned host surface; direct
`GridAuthoredCell::Formula` construction with a hand-written key becomes
impossible outside the crate (§3).

Rationale: this is the smallest surface that satisfies W059's exit gate for
the grid (one interpretation path, typed bypass, diagnostics-as-rejection)
while keeping the region-granular machinery underneath unchanged. The W011
`EditGridCell` intent maps 1:1 onto `enter_grid_cell` and its two typed
rejections (`FormulaEditingNotYetSupported`, no-ClearCell) simply cease to
exist.

**W059 residue not owned here:** the final replacement of the legacy
`TreeFormulaCatalog` lowering container with direct `BoundFormula`
consumption (W059 §7A item 5) is tree-side dependency-build work and belongs
to D3's tree-evaluation unification, not to this surface. Recorded, not
absorbed.

## 3. Public formula binding and the derived-key doctrine

**Decision:** a public context verb

```rust
pub fn bind_grid_formula(
    &self, workspace_id, node_id, address: ExcelGridCellAddress,
    source_text: &str, channel: FormulaChannelKind,
) -> Result<BoundGridFormula, OxCalcDocumentError>;

pub struct BoundGridFormula {
    pub formula: GridFormulaCell,        // source_text + minted key + channel
    pub unresolved_names: Vec<String>,   // will evaluate #NAME? until seeded
    pub diagnostics: Vec<…>,             // non-fatal bind notes
}
```

wrapping the `bind_grid_formula_for_transform` recipe
(`optimized_sheet.rs:7241`) — parse + red projection + `bind_formula` under
the strict-excel profile — but assembled with the **real** workspace context
(names catalog, tables, bounds) rather than the transform path's synthetic
tokens. The key is minted from the bound formula's template identity
(`BoundFormula.formula_template_identity.key`), exactly the W011 ask (b)
shape. (Corrected per D2 §4.4: W077 is built and CLOSED, and the strict
profile deliberately keeps `IncludeCallerAnchor` — caller-independence
comes from the R1C1 normal form; no key-policy migration is planned. The
doctrine still earns its keep: if any future key-affecting change lands,
this verb picks it up without signature change — key *format* is not
part of this verb's contract, which
brings us to:

**Decision — the derived-key doctrine.** `GridInputState`'s authored formula
record stores **source text + channel only**:

```rust
pub enum GridCellInput {           // the GridInputState cell record (D1 §7.1)
    Literal(CalcValue),
    Formula { source_text: String, channel: FormulaChannelKind },
    RichStub(u32),                 // §12, CellChunk row
}
```

`normal_form_key` is **derived state**, minted by bind at load/edit time and
living in `GridDerivedState` alongside the engine sheets. This is the W057
layer doctrine applied to the grid: raw entered text is durable input truth;
binding is a derived layer (W059 §2 named exactly this blur as the defect).
Consequences, all intended:

- Revision identity (D1 C5, content address over authored grid truth) is
  stable across key-policy changes — W077's key migration re-mints derived
  state, not history. Without this, adopting W077 would rewrite every
  revision id retroactively.
- Hand-keying is structurally impossible: nothing outside the bind step can
  inject a key. The W011 fixture const
  (`W011_FIXTURE_NORMAL_FORM_KEY = "excel.grid.v1:cell:R[0]C[-1]*3"`) and
  its deletion bead become moot on adaptation.
- The engine's `set_formula(GridFormulaCell)` seam is untouched — the bind
  step constructs the `GridFormulaCell` immediately before feeding the
  engine; the pair simply never persists in authored truth.

Alternative rejected: storing the minted key in authored truth "to skip
rebinding on revision navigation". Rebind-on-restore is exactly D1 C6's
rebuild-by-recalc baseline, the cost is one bind per formula cell per
restore, and the stability cost of key-in-identity is permanent. Rejected.

`unresolved_names` is a first-class return field, not a failure: unresolved
names bind and later evaluate `#NAME?` with self-healing (the shipped
machine contract). Binding **fails** (typed `Err`) only for text OxFml
rejects as a formula (parse/acceptance diagnostics) — mirroring
`enter_grid_cell`.

## 4. Defined-name seeding

**Decision: consumer verbs, not seed fields.** The context grows the full
name lifecycle, routed to the (per-sheet, pre-D3) machines:

```rust
pub fn set_workbook_defined_name(&mut self, workspace_id, name, target: GridRect_on_sheet)
pub fn set_sheet_defined_name(&mut self, workspace_id, sheet_node, name, target)
pub fn set_workbook_dynamic_defined_name(&mut self, workspace_id, name, source_text)
pub fn set_sheet_dynamic_defined_name(&mut self, workspace_id, sheet_node, name, source_text)
pub fn rename_defined_name(…), pub fn delete_defined_name(…)
pub fn defined_names(&self, workspace_id) -> Vec<DefinedNameReadout>   // both scopes + dynamic + metadata presence
```

Dynamic-name formulas route through §3's bind verb (source text in, bound
formula in, key derived) — one binding authority for cells and names alike.
Every verb returns/propagates the machine's `GridNameLifecycleReport` dirty
seeds into the normal invalidation channel and advances the revision (names
are authored document truth; D1 §4's meta/namespace machinery already
carries name-shaped identity).

`GridBackingSeed` does **not** grow a `defined_names` field (the W011 ask's
alternative shape): names are workbook-scoped facts that outlive any one
sheet's seed, and ingest (§9) arrives at workbook scope anyway. One seeding
path, verb-shaped. Where a name's target sheet does not yet exist mid-ingest
the ingest builder defers name installation to its commit step (§9) — hosts
using the public verbs never see that ordering problem.

**Scope precedence and shadowing** are the machine's existing contract
(sheet-scoped wins inside its sheet); D4 adds no policy. LAMBDA-valued names
(a name bound to a callable, not a rect) are **D2's open question** under
the tree-node≈defined-name unification; this surface deliberately types
`set_*_dynamic_defined_name` around source text so a LAMBDA payload is not a
new verb later, just a new bind result. Deferred to D2, with the verb shape
already compatible.

## 5. Authored readout and editability

**Decision:** a per-cell authored readout, windowable, reading
**`GridInputState` only** (never derived state — the readout must be exact
under D1 C6):

```rust
pub fn grid_authored_view(
    &self, workspace_id, node_id, window: Option<GridRect>,
) -> Result<Vec<GridAuthoredCellReadout>, OxCalcDocumentError>;

pub struct GridAuthoredCellReadout {
    pub address: ExcelGridCellAddress,
    pub kind: GridAuthoredKind,          // Empty | Literal | Formula | RichStub
    pub literal: Option<CalcValue>,
    pub source_text: Option<String>,     // formula display text ("=A1*3")
    pub channel: Option<FormulaChannelKind>,
    pub editability: GridCellEditability,
}

pub enum GridCellEditability {
    Editable,
    RepeatedRegionMember { anchor: ExcelGridCellAddress }, // FillRange/shared-formula member
    MergedFollower { anchor: ExcelGridCellAddress },
    SpillDisplay { anchor: ExcelGridCellAddress },         // value shown, nothing authored here
    TableStructural { table_id: String },                  // header/totals machinery cells
}
```

Editability is **derived and advisory-plus-enforced**: the edit verbs (§2)
enforce the same classification with typed rejections (editing a spill
display cell is `SpillDisplacedCellNotEditable`, not a silent overwrite), so
a skin can gray cells out but the contract does not depend on skins being
honest. This is W011 ask (c) made native, replacing the host-side authored
mirror.

Note the deliberate asymmetry with `grid_view`: computed values + epochs
come from the published readout (derived); authored facts come from input
state. A cell can appear in both; the two never merge into one struct,
because they have different staleness and identity rules.

## 6. Calc-settings access, FileCached publication, and recalc policy

Settings **home and identity** are D1 §5 (C4): `#workbook-settings`
meta-child, `workbook_calc_settings()` /
`set_workbook_calc_settings()` accessors, typed `WorkbookSettingChanged`
seeds. D4 adds nothing to storage; it fixes the surface obligations:

- **Ingest writes settings through the same accessors** (§9): a loaded
  workbook's date system and calc mode are ordinary revision-identified
  settings from revision 1, not ingest-private state.
- **Manual calc mode is honored by the document surface.** Edit verbs under
  `CalcMode::Manual` mutate authored truth and mark dirty but do not
  evaluate; a new verb `recalculate_workbook(workspace_id)` is the explicit
  trigger (it exists regardless of mode — it is Excel's F9). D3 owns what
  "recalculate" traverses; D4 owns that the verb exists and that Manual mode
  suppresses implicit evaluation from edits and **from interest
  registration** (see tension T3 — today `register_grid_interest`
  unconditionally recalcs).
- **`CachedValueProvenance::FileCached` becomes a published-value
  provenance.** Loaded cached values enter the published readout tagged
  `FileCached`; the first genuine evaluation replaces them with engine
  values. This is what lets a loaded workbook render instantly (and under
  Manual mode, indefinitely) without pretending the engine computed those
  numbers, and it is what the save path reads for never-recalculated cells.
  The differential harness ignores `FileCached` values by construction (they
  are pre-engine, not an engine disagreement) — contract C15, flagged to D3.

## 7. Neutral output: full projection and the authored delta

Two output shapes, two consumers:

**Decision 7a — save projection (the OxDoc handoff).** The `oxdoc_ingest`
module (§8) owns the export half:

```rust
pub fn project_workbook_model_output(
    &self, workspace_id,
) -> Result<oxdoc_model::WorkbookModelOutput, OxCalcDocumentError>;
```

producing a `WholeModelProjection` event stream assembled from:

1. Tier A authored truth read back from the model: header from calc
   settings, sheets in registry order (C3), cells from `GridInputState`
   (literals typed; formulas as `CellPayload::Formula { text:
   source_text-without-leading-`=`, cached: current published value }`),
   defined names, tables, merged regions. **Cached-value refresh is
   structural here**: the cached payload is read from the published readout
   at projection time, so an edited-and-recalculated workbook saves fresh
   caches by construction — the W011 stale-cache trap cannot occur. A cell
   whose published value is `FileCached` writes that value back verbatim
   (it was never recomputed; writing anything else would launder staleness).
   `#NAME?` values arising from *unseeded* names are written back as the
   retained `FileCached` value when one exists, never as a freshly minted
   error cache (the W011 dtc-hj2.10 guard, engine-side now).
2. Tier B facts replayed **verbatim** from the inert store (§13), in the
   stream positions the validator requires.
3. Explicit exclusions re-emitted per their §12 disposition (CalcChainHint:
   omitted; OxDoc/writers regenerate or drop it — it is a performance hint
   with no fidelity content).

Whole-model projection is chosen because it is the only cell-edit save path
oxdoc-model supports today (verified: no granular cell modeled edit exists);
a granular `CellEdit` entry is upstream gap (§15) and slots in as an
alternative entry kind later without changing this verb's signature.

**Decision 7b — authored delta ("edits since revision R").**

```rust
pub fn workbook_authored_delta(
    &self, workspace_id, since: WorkspaceRevisionId,
) -> Result<WorkbookAuthoredDelta, OxCalcDocumentError>;
```

Neutral OxCalc types: per-sheet cell-input diffs (set/cleared, literal vs
formula-text), name/table/merge lifecycle diffs, settings diffs, sheet
lifecycle (add/delete/rename/reorder from structural + tombstone facts, D1
C2). Computed **exclusively** by diffing grid-input snapshots and structural
snapshots between the two revisions — D1 C6 verbatim; derived state never
enters the diff. Consumers: host edit ledgers (replacing W011's
`WorkbookEditLedger` mirror), future granular save, sync/collab surfaces.
The delta deliberately does **not** carry computed values; a consumer that
wants fresh caches reads the published readout, same as 7a does.

---

# Part II — oxdoc-model ingestion

## 8. Module and dependency shape

**Decision:** new module `src/oxcalc-core/src/oxdoc_ingest.rs` (splitting
into `oxdoc_ingest/` if it outgrows one file), with `oxdoc-model` added as a
sibling-path dependency of `oxcalc-core` — dependency on `oxdoc-model`
**only** (it is byte-free by design; no oxdoc-xlsx, no file formats). Not
feature-gated: under no-legacy every downstream (DnaTreeCalc host, tracecalc)
wants the document surface, oxdoc-model is a small pure-data crate, and a
feature flag would bifurcate the test matrix for zero users of the "without"
half. The module boundary is kept strict (only `consumer.rs` types and
`oxdoc_model` types cross it) so a later crate split is mechanical if wasm
size ever argues for one.

The module owns both directions: `OxCalcWorkbookIngestSink` (the
`OxCalcIngestSink` impl) and `project_workbook_model_output` (§7a). Load and
save living in one module is deliberate — the tier table (§12) is one table,
and round-trip tests live next to it.

## 9. Direct-to-model, single-transaction load

**Decision: ingest targets the new structural model directly — no staging
model — via a bulk builder that commits one revision.**

```rust
pub fn load_workbook_model(
    &mut self, create: OxCalcWorkbookCreate,
    events: &[oxdoc_model::DocumentEvent],          // or &dyn WorkbookModelAccess
) -> Result<WorkbookLoadReport, OxCalcDocumentError>;
```

Mechanics:

- The sink does **not** call the public verbs per event (that would mint a
  revision per cell and re-validate per edit). It accumulates directly into
  the D1-shaped target values: one `StructuralSnapshot` build (workbook
  root + Sheet-role nodes in stream order), one `GridInputState` per sheet
  (cells, merges), the name/table sets, the settings values, and the Tier B
  store — then commits **one transaction**: validate once, mint revision 1,
  install derived state. Loading IS an edit transaction in the revision
  graph, so "edits since load" (§7b) has a well-defined basis.
- Sheet mapping: `SheetRef.sheet_id: u32` → fresh `TreeNodeId` per sheet,
  held in an ingest-local map; `SheetRef.name` → node symbol → sheet
  registry (D1 C1). The engine-facing sheet identity string is derived from
  the node id, not the display name (tension T1 records the D2 dependency).
- Deferred installs: defined names and tables land at commit (after all
  sheets exist), so forward references (`Sheet2!`-targeting names arriving
  before Sheet2's `SheetBegin`) are ordering-proof. Note: the stream
  validator (`validate_event_stream`, `lib.rs:2800`) order-constrains
  header/string/style tables but leaves `DefinedName`/`ExternalLink`/
  `CalcChainHint`/`OpaquePartNotice` position-free — the commit-time
  deferred install above is what makes ingest ordering-proof; it does
  not rely on any validator ordering guarantee for those events.
- Evaluation policy at load: **bind, don't evaluate.** All formulas bind
  (§10) during commit; published values are seeded from `FileCached` caches
  (§6). Then: `CalcMode::Automatic` → one `recalculate_workbook` is issued
  as part of load (Excel's open-recalc); `Manual` → none, the workbook
  renders from caches until F9. The load report records which path ran.
- `WorkbookLoadReport` carries: sheets/cells/names/tables counts, the
  **ingest fidelity ledger** (every Tier B retention and every exclusion,
  §12/§13), every formula **bind degradation** (§10), and the upstream
  `DocumentFidelityLedger` if the caller supplied model access. Nothing is
  droppable without a ledger row — the no-silent-loss invariant is a report
  invariant, tested as one.

Staging rejected because the D1 model *is* the staging-quality target:
content-addressed input snapshots, validated structure, one-transaction
commit. A separate staging model would exist only to be immediately
projected into this one, and its divergences would need their own tests.
Sequencing consequence (not a design compromise): R6 requires D1 R2.4/R2.6/
R2.7 (roles+verbs, `GridInputState`, revision membership) to have landed.

## 10. Formula binding at ingest: degradation, never failure

**Decision:** every formula binds through the strict-excel profile at load
commit, and **no bind outcome fails ingest**:

- `SpreadsheetMlA1` text gets its leading `=` restored and enters the same
  bind path as §3. `R1C1RelativeTemplate` text (shared regions) binds as a
  repeated-formula template.
- Unresolvable names → cell ingests, evaluates `#NAME?`, self-heals when the
  name appears (shipped machine contract). Out-of-bounds / dangling
  references → `#REF!`. Both are *evaluation* outcomes, not ingest errors.
- Text OxFml cannot accept as a formula at all (corrupt/unsupported
  grammar): the cell is retained as authored formula text (round-trip safe),
  is published as its `FileCached` value if present (else `#NAME?`-class
  typed error value), and produces a `BindDegradation { address, text,
  diagnostics }` ledger row. The formula is never discarded and never
  silently rewritten.
- `SharedFormulaRegion` → `put_repeated_formula_region` (the existing
  FillRange machinery is exactly this shape: one R1C1 template, region
  extent, per-cell expansion proven by the permanent-pair differential).
  `FormulaTopology` records route the exceptions: `Array` (legacy CSE) →
  the cells ingest as normal formulas plus an inert `Cse` overlay extension
  claiming the array rect (§13) — evaluation semantics for legacy CSE are
  not built here; `DataTable` → cells retain authored topology facts in the
  Tier B store and publish cached values, ledgered `NotCalcModeled`;
  `Unknown{formula_type}` → same retention + ledger row.
- Error-code mapping: `CellPayload::Error(u8)` / `OxCalcCellValue::Error(u8)`
  carry BIFF error codes; ingest maps the known set (0x00 `#NULL!`, 0x07
  `#DIV/0!`, 0x0F `#VALUE!`, 0x17 `#REF!`, 0x1D `#NAME?`, 0x24 `#NUM!`,
  0x2A `#N/A`, plus the newer codes oxfunc models) to typed `CalcValue`
  errors; unknown codes retain the raw byte in the Tier B store and publish
  `#VALUE!` with a ledger row — mapped, never guessed silently.

## 11. Strings: resolve at ingest, no model string table

**Decision:** `SharedText(u32)` indices resolve against the prelude string
table **during ingest**; the model stores plain owned text in `CalcValue`s.
There is no OxCalc-side shared-string table, no interning layer in authored
truth, and the save projection (§7a) derives a fresh shared-string table
from the authored text it walks (dedup at write time, where it is a pure
serialization concern).

Rationale: an interned table is identity-bearing state — it would enter
snapshots, revision identity, and the candidate-overlay clone paths, all to
optimize memory for a workload (massive duplicated text) we have not
measured. The digest-not-verbatim lesson (D1 §7.1) already bounds identity
cost. The door explicitly left open: if load profiling shows duplicated-text
pressure, an `Arc<str>`-level dedupe **inside ingest** (transient, dropped
at commit) or an oxfunc-level shared-text `CalcValue` representation are
both compatible with this contract because the contract is "no *persistent,
identity-visible* table", not "no sharing". Recorded as a perf watch item,
not a seat.

## 12. The tier taxonomy and the 29-variant disposition table

Three tiers, all ledgered:

- **A — calculation-bearing:** enters the calc model (authored truth,
  names, tables, merges, settings, lifecycle). Round-trips from the model.
- **B — inert document fact:** retained verbatim in the inert store (§13),
  never calc-visible, replayed verbatim at save. Sub-annotation
  `B(overlay)`: additionally claims a rect via the grid overlay `Extension`
  seat. Sub-annotation `B(gap:…)`: retained inert **and** carrying a named,
  known calc-semantics gap so the honesty is inspectable.
- **X — typed exclusion:** not retained; ledgered with the reason;
  regenerable or meaningless at save. (Exactly one variant earns this.)

| # | `DocumentEvent` variant | Tier | Disposition |
|---|---|---|---|
| 1 | `WorkbookHeader` | **A** | date system + calc mode → `#workbook-settings` via C4 accessors; iteration settings absent upstream (gap §15, defaults apply). |
| 2 | `StringTable` | **A** (consumed) | Resolved at ingest (§11); not stored; regenerated at save from authored text. |
| 3 | `StyleTable` | **B** | Retained verbatim (number formats included — display-only; see T5 for the date-formatting note). |
| 4 | `DifferentialStyleTable` | **B** | Retained verbatim. |
| 5 | `SheetBegin` | **A** | Sheet node creation (R2.4 lifecycle), stream order = sheet order (C3). |
| 6 | `SheetEnd` | **A** | Closes the per-sheet accumulation; structural no-op beyond ordering. |
| 7 | `SheetDimension` | **B** | Retained; grid bounds are set by profile policy (strict-excel full A1 space), not by the file's used-range claim; save writes the recomputed authored extent, retained value used only if authored extent shrank (never lie smaller than content). |
| 8 | `ColumnProps` | **B(gap: SUBTOTAL/AGGREGATE hidden-column semantics)** | Hidden/width/outline runs retained inert; hidden-ness is genuinely calc-bearing for the 100-series SUBTOTAL family — named gap, upgrade path is a later engine fact, not a D4 invention. |
| 9 | `RowProps` | **B(gap: SUBTOTAL 100-series ignores hidden rows)** | Same as #8, row axis. |
| 10 | `MergedCellRegions` | **A** | `add_merged_region` per rect (spill blocking + edit admission are live engine semantics). |
| 11 | `SheetViewState` | **B** | Retained verbatim (frozen panes, selection). |
| 12 | `Hyperlinks` | **B** | Retained verbatim. |
| 13 | `DataValidations` | **B** | Retained verbatim (validation formulas are NOT bound — they are UI-gate facts, not calc graph members). |
| 14 | `AutoFilter` | **B(gap: filter-hidden rows interact with SUBTOTAL)** | Retained verbatim; same named gap family as #9. |
| 15 | `SortState` | **B** | Retained verbatim. |
| 16 | `CommentNotice` | **B** | Retained verbatim. |
| 17 | `ThreadedCommentPeople` | **B** | Retained verbatim. |
| 18 | `SheetReviewComments` | **B** | Retained verbatim. |
| 19 | `DrawingFormControls` | **B(overlay: RichObject)** | Spec retained verbatim; controls with cell anchors additionally claim inert `Extension(RichObject)` rects so spills/axis edits see them (inert `SpillBlock::None` today, per the overlay seam's construction). |
| 20 | `CellFormatRuns` | **B** | Retained verbatim (per-cell style presence). |
| 21 | `ConditionalFormatRegion` | **B(overlay: ConditionalFormat)** | Full spec retained in the store; each region claims an inert `Extension(ConditionalFormat)` rect, `payload` = store key. CF rule formulas are NOT bound in R6 (they are display-band facts until a CF engine exists — CF-1's seat is reserved, not filled). |
| 22 | `FormulaTopology` | **A (metadata)** | Routes formula record kinds: `Shared`→#24 handling, `Array`→CSE cells + inert `Cse` overlay rect, `DataTable`/`Unknown`→retain+cached+ledger (§10); attrs and unsupported fragments retained in the B store for round-trip. |
| 23 | `CellChunk` | **A** | Literals → typed `CalcValue`; formulas → bound (§10) with `FileCached` caches → publication (§6); `Empty` → no record; `RichStub(u32)` → authored `GridCellInput::RichStub` retained for round-trip, publishes its cached value if the file carried one else blank, ledgered `RichValueNotModeled` — never calc-visible as a fake value. |
| 24 | `SharedFormulaRegion` | **A** | `put_repeated_formula_region` from the R1C1 template (existing engine machinery, differential-proven). |
| 25 | `TableOverlay` | **A** | `set_table_overlay` (name/sheet/range); structured-reference resolution is live engine semantics. |
| 26 | `DefinedName` | **A** (+ metadata **B**) | `scope_sheet_id` → workbook vs sheet scope verbs; rect-denoting formulas → static names; others → dynamic names bound via §3; `DefinedNameMetadataSpec` (comment/hidden/function flags/raw attrs) retained in the B store keyed by name. |
| 27 | `ExternalLink` | **B** (decision §14) | Link targets retained verbatim; formulas referencing external workbooks bind-degrade per §14. |
| 28 | `CalcChainHint` | **X** | Excluded with reason `EngineDerivesCalculationOrder`: OxCalc's dependency graph owns ordering; the hint is a performance artifact with no fidelity content; omitted from save (writers regenerate or omit). The single X in the table. |
| 29 | `OpaquePartNotice` | **B** | Notices retained verbatim and surfaced in the load report; `GeometryCoupling::{SheetAnchor, SourceRange}` notices are flagged prominently (axis edits on their sheets go stale against the opaque part — the ledger says so, honestly, per the upstream `GeometryCoupledOpaqueLeftStale` loss kind). |

Count check: 29 rows, every variant named, one X, zero silent paths. The
sink's `feature()` arm ends in an exhaustive match over
`OxCalcDocumentFeature` with **no wildcard**, so a 30th upstream variant is
a compile error in `oxdoc_ingest`, not a silent drop — that is the
enforcement mechanism for `#[non_exhaustive]` growth (the driver maps
events→features; a new feature variant breaks our build loudly, which is
exactly what we want from an honesty regime).

## 13. Tier B storage shape

**Decision: a typed inert store + overlay rects for the spatial families +
ledger rows for everything — the hybrid, with one owner each.**

```rust
pub struct IngestedDocumentFacts {      // per workspace, Arc-shared
    // typed, per-variant retention: styles, dxfs, format runs, views,
    // hyperlinks, validations, filters, sorts, comments, controls, CF specs,
    // name metadata, external links, topology attrs, opaque notices, …
}
```

- Held as `Arc<IngestedDocumentFacts>` on live workspace state and on
  retained revisions (pointer copies, the `deleted_table_facts` retention
  shape). **Immutable after load** in R6 — no edit verbs touch it — so its
  identity is a load-time digest.
- Identity participation: the digest is written into a
  `#workbook-ingest` meta-child at load commit (one grandchild:
  `facts_digest`). Via D1 §4/§5 machinery this makes the inert payload
  revision-identified with zero new snapshot plumbing, and a future
  Tier-B-editing feature (styling verbs, say) has a ready-made identity
  seam: edit store → new digest → node-input edit → new revision.
- Overlay seats: only the **rect-claiming** families project inert
  `GridOverlayExtension` values (`ConditionalFormat`, `RichObject`, `Cse`
  per §12), `payload` = store key, blockage/admission inert as constructed
  today. The overlay seat is a *spatial index* into the store, never the
  retention home — `payload: String` is too lossy to own a
  `ConditionalFormatRegion`, and the store must survive even for families
  with no rect (styles, people, links).
- Meta-children rejected as the retention home: property subtrees are
  node-input records (literal text/typed values) — encoding a `StyleTableSpec`
  into node inputs would be serialization theater. Meta carries the digest
  (identity); the store carries the payload (bytes). Ledger-only rejected
  for anything with save fidelity: a ledger row cannot be replayed at §7a.

## 14. `DocumentEvent::ExternalLink`: Tier B, with a bind-degradation contract

**Decision (reconciled with D2 §5, owner arbitration 2026-07-04 — see
T8):** external workbook references are **out of calc scope in this
program's R6**. D2 §5 has *decided* the in/out question (typed
partial-IN: bind + identity + routing-to-loaded-sibling-workspace in R3;
evaluation-triggered loading and link management out); this section's
`FileCached` pinning is the ingest-side complement, scoped by
provenance:

- `ExternalLinkSpec` targets retained verbatim (Tier B) and surfaced in the
  load report.
- A formula whose bound references include an external-workbook token
  ingests normally (authored text retained), binds with the external
  reference recorded as unresolved-external, and **publishes its
  `FileCached` value pinned** — recalc does not evaluate the cell (it cannot,
  honestly) and does not clobber the cached value with an invented error;
  the cell is ledgered `ExternalReferenceNotLinked` and its readout carries
  the `FileCached` provenance **until an explicit refresh or a
  sibling-workspace load** — the reconciled boundary: at that point D2
  §5's routing applies, and an unavailable target becomes typed `#REF!`
  (D2's honesty rule), while a loaded sibling gives live values. Pinning
  mirrors Excel-without-the-source-open (Excel shows the last-fetched
  external cache), stays inside the "degradation never failure"
  doctrine, and — critically — writes the same cache back at save.
  D2's `#REF!` rule applies from the start to newly *authored* external
  references that have no cache and no loaded sibling.
- When D2 lands cross-workspace resolution, these pins upgrade to real
  cross-workspace edges; the ledger rows are the worklist.

## 15. Upstream gaps to raise (oxdoc-model / OxDoc handovers)

Raised as OxDoc-repo handover items in R6.7, cited from this design:

1. **Iterative-calc settings missing from `WorkbookHeader`** (verified:
   `lib.rs:947` has date_system + calc_mode only). OxCalc defaults
   (enabled=false, 100, 0.001 per C4) until the header grows them; W055's
   Excel-match closure will need the real values.
2. **Granular cell modeled edit** (`WorkbookModelEditKind` has no cell-level
   entry): the §7b authored delta is the natural producer; a
   `CellEdit`-class entry would let save skip whole-model projection.
   Non-blocking (7a works today), echoing the W011 ask already on record.
3. **Lazy model access for ingest**: `drive_oxcalc_ingest_from_model_access`
   requires eager events (`lib.rs:2731`); a chunked/deferred drive path
   matters for large-workbook wasm loads. Non-blocking.
4. **Sheet-order/rename representation in output**: whole-model projection
   re-emits sheets in current order so this is expressible, but a
   reorder/rename *edit* has no modeled-edit shape either; folded into (2).

## 16. `machine.rs` decomposition: defers

**Decision: defers.** The R5/R6 work touches `consumer.rs`, the new
`oxdoc_ingest` module, and the already-extracted machine submodules
(`calc_ref_sheet.rs`, `optimized_sheet.rs`, `overlay.rs`); nothing in this
design requires editing the 26k-line `machine.rs` (which is ~90% its own
test corpus — verified 26,007 lines against 23k+ of `#[test]`/fixture
content). Decomposing it inside R5/R6 would couple a mechanical hygiene
migration to the program's most externally-visible wave for zero design
benefit. It is recorded as an independent, any-time hygiene bead
(test-corpus extraction to `grid/machine/tests/`), off every critical path.
If a D3 bead later needs to split evaluation internals out of `machine.rs`,
that bead carries its own slice.

---

## The W011 round-trip contract (fixture-class acceptance)

The output contract is proven against the W011 fixture class as R6's
closing acceptance (R6.6):

1. Load the two-cell workbook (`A1 = 7`, `B1 = =A1*3`, cached 21) through
   `load_workbook_model` — B1 binds via strict-excel; published values are
   FileCached (7, 21) until the automatic-mode load recalc replaces them
   with engine values (7, 21 — agreeing, per the differential).
2. `enter_grid_cell(A1, "10")` → literal branch → revision advances → recalc
   → B1 publishes 30.
3. `project_workbook_model_output` → event stream where A1 is
   `Number(10.0)`, B1 is `Formula { text: Some("A1*3") /* preserved */,
   cached: Some(Number(30.0)) /* refreshed from publication */ }`, and every
   Tier B event of the source replays verbatim.
4. `workbook_authored_delta(since = load revision)` reports exactly one
   cell-input edit (A1 literal), from grid-input snapshot diffs only —
   B1 appears nowhere in the delta (its authored truth never changed).
5. Reload the projected stream into a fresh context: authored views equal,
   published values equal after recalc — the full circle.

Steps 1–3 are what un-pauses W011 with zero hand-keyed constants; step 4 is
its edit-ledger replacement; step 5 is ours.

## Cross-design tensions

- **T8 (D2 §5 external references — RESOLVED by owner arbitration
  2026-07-04).** As first authored, §14 pinned `FileCached` external
  values indefinitely while D2 §5 (committed first) evaluated unloaded
  externals to `#REF!` with "no cached-external-value store" — a genuine
  contradiction the fresh-eyes review caught. Reconciliation by
  provenance scope: ingest-cached values publish through the ordinary
  channel with `FileCached` provenance (no separate store — D2's
  exclusion is scoped to runtime stores/link managers, see D2 §5
  errata), pinned until explicit refresh or sibling-workspace load; from
  that boundary D2 routing/`#REF!` semantics own the cell. Newly
  authored externals with no cache follow D2 from the start. Both docs
  now record the same contract (§14 here; D2 §5 + T5 errata).
- **T1 (D2 — sheet identity inside reference keys).**
  `ExcelGridCellAddress` and today's normal-form keys embed `workbook_id` /
  `sheet_id` **strings**. If those strings are display names, every sheet
  rename rewrites bound keys and breaks D1 C2's rename-stability story.
  This design derives the engine-facing sheet token from `TreeNodeId`
  (§9). **RESOLVED:** D2 (committed `2bc6ea7e`) ratified exactly this in
  its §10 normal-form-key decision — `SheetIdentityToken` is minted from
  the node id, rename-immune; key format stays stable/engine-internal.
  R5/R6 build on the ratified token; §3's derived-key doctrine keeps
  format revisable regardless.
- **T2 (D1 C5 wording).** D1 C5 content-addresses "authored grid truth"; D1
  §7.1 sketched the authored record as "literal/formula text". §3's
  derived-key doctrine sharpens this: the authored formula record is
  source_text+channel, **excluding** the minted key. This is a refinement
  within C5's letter (the key was never authored) but R2.6's implementer
  must adopt it — flagged so D1's bead does not freeze the key into
  `GridInputState`.
- **T3 (D3 — interest registration forces evaluation).**
  `register_grid_interest` unconditionally recalcs (`consumer.rs:3482`),
  and the live consumer runs both engines mark-all per recalc. Under §6's
  Manual mode and §14's FileCached pins, read-shaping must stop implying
  evaluation: D3's incremental consumer wiring needs "materialize readout
  without evaluating" as a first-class mode. Flagged as a D3 requirement
  originating here.
- **T4 (D3 — FileCached and the differential).** C15: `FileCached` published
  values are pre-engine and must be invisible to the oracle/optimized
  differential (they are not an engine disagreement) and to D3's
  workbook-volatile tick (a load is not a tick). D3's differential
  extension should assert provenance-awareness explicitly.
- **T5 (D2/W061 — number formats vs date semantics).** Styles are Tier B,
  but Excel's date-ness lives partly in number formats (a serial is "a
  date" for display only). Calc semantics (date system arithmetic) are C4
  settings and unaffected; but any future TEXT()/CELL("format") support
  would need read access into the Tier B style store — a named, deliberate
  crack in the "never calc-visible" wall, to be opened (if ever) by a D2/D3
  decision, not by ingest.
- **T6 (D1 R2 sequencing).** §9's single-transaction load writes
  `GridInputState` and revision-1 grid membership — R6 hard-depends on
  R2.4, R2.6, R2.7. Candidate overlays cloning whole grid maps
  (`consumer.rs:1394` behavior) would make pre-R2.6 ingest of large
  workbooks quadratically expensive; the dependency is welcome, not
  incidental.
- **T7 (W059 residue → D3).** The `TreeFormulaCatalog` lowering replacement
  (W059 §7A.5) is deliberately left to D3's tree-evaluation unification;
  this design closes W059's grid-side and input-side gates only.

## Open questions answered

Every D4 question from `W062_IDEAL_ENGINE_MODEL_REWORK.md` §Open questions,
plus the questions the bead scope added:

| D4 question | Answer |
| --- | --- |
| Ingest targeting the new structural model directly vs staging | Directly — single-transaction bulk builder committing one revision against D1 structures; staging rejected as a duplicate model with its own test burden (§9). |
| Tier B storage shape (overlays vs meta-children vs ledger-only) | Hybrid with one owner each: typed `IngestedDocumentFacts` store (Arc, immutable post-load) is the retention home; digest in a `#workbook-ingest` meta-child for revision identity; overlay `Extension` seats as spatial indexes for CF/RichObject/CSE rects; ledger rows always (§13). |
| String-interning policy | Resolve at ingest, owned text in the model, no persistent/identity-visible string table; save re-derives the shared table; transient ingest dedupe permitted if profiling demands (§11). |
| `machine.rs` decomposition rides along vs defers | Defers — nothing in R5/R6 edits machine.rs; independent hygiene bead, off critical path (§16). |
| `DocumentEvent::ExternalLink` tier | Tier B retained; referencing formulas keep authored text, publish pinned FileCached values, ledgered `ExternalReferenceNotLinked`; upgrades to real cross-workspace edges when D2 opts in (§14). |
| `#NAME?`/`#REF!` degradation policy at ingest | Bind everything through strict-excel; unresolved names/refs are evaluation outcomes (self-healing `#NAME?`, `#REF!`); only OxFml-rejected text degrades to retained-text + cached-value + ledger row; ingest never fails on formula content (§10). |
| oxdoc-model gaps to raise upstream | Iteration settings in `WorkbookHeader` (blocking-adjacent for W055), granular cell modeled edit, lazy model-access ingest drive, reorder/rename modeled edits (§15). |

**Deferred (with owners):** LAMBDA-valued names (D2 — verb shape here is
already compatible, §4); external-reference calc semantics (D2 in/out, §14);
legacy-CSE evaluation semantics and CF rule evaluation (future engine
worksets; seats reserved, §12); SUBTOTAL hidden-row/filter semantics (named
Tier B gaps, §12 rows 8/9/14); `TreeFormulaCatalog` retirement (D3, T7).

## Contracts exported

Downstream designs and R5/R6 beads may rely on these without re-derivation:

- **C9 (hosts, D3):** Authored input is a three-way branch —
  `enter_grid_cell` yields Literal / Formula / typed-diagnostics-Err with
  **no mutation on Err**; empty text = ClearCell; typed `CalcValue` entry
  never round-trips through text. One OxFml authority for tree and grid.
- **C10 (hosts, R3):** `bind_grid_formula` is the only key mint; key
  format is engine-internal and NOT part of the verb contract; keys never
  persist in authored truth (derived-key doctrine) — any future
  key-affecting change (none planned: W077 is closed and the strict
  profile keeps `IncludeCallerAnchor`, per D2 §4.4) re-mints derived
  state without touching revision history.
- **C11 (hosts, skins):** `grid_authored_view` reads `GridInputState` only;
  per-cell kind/source_text/channel/editability; editability classifications
  are enforced by the edit verbs with matching typed rejections, not
  advisory-only.
- **C12 (OxDoc, hosts):** `project_workbook_model_output` round-trips Tier
  A from the model and Tier B verbatim; formula cached values are read from
  publication at projection time (fresh-cache-by-construction); `FileCached`
  and unseeded-`#NAME?` cells write back their retained caches, never
  freshly minted staleness. `workbook_authored_delta` diffs input/structural
  snapshots only (D1 C6 honored).
- **C13 (program):** Every one of the 29 `DocumentEvent` variants has the
  §12 disposition; the sink's feature match is wildcard-free so upstream
  variant growth is a compile error; every retention/exclusion/degradation
  is a `WorkbookLoadReport` ledger row. No silent-loss path exists by
  construction and is tested as a report invariant. One trust point,
  named: the compile-error tripwire fires only if upstream maps a new
  `DocumentEvent` variant to a new `OxCalcDocumentFeature` variant (the
  feature enum is exhaustive-matchable today); a variant absorbed inside
  the upstream driver never reaches the sink — R6.7 carries the upstream
  ask to keep the event→feature mapping total. Failure-class boundary,
  named: formula-content problems degrade-with-ledger (§10); structurally
  invalid streams (e.g. case-fold-duplicate sheet names rejected by D1
  `validate()`) fail the single-transaction load with a typed `Err` —
  load-fail vs degrade classes are enumerated in R6.1's acceptance.
- **C14 (D2, hosts):** Defined-name lifecycle is consumer-level
  (workbook/sheet scope + dynamic + rename/delete + readout), one bind
  authority with cells, dirty seeds through the normal channel, revision-
  visible.
- **C15 (D3):** Published values carry provenance; `FileCached` values are
  pre-engine — excluded from the differential, replaced by first genuine
  evaluation, pinned indefinitely for external-reference cells. Manual calc
  mode suppresses implicit evaluation from edit and read-shaping verbs;
  `recalculate_workbook` is the explicit trigger.
- **C16 (program):** `oxdoc_ingest` is the only module that names
  `oxdoc_model` types; the dependency edge is `oxcalc-core → oxdoc-model`,
  nothing else, unconditional.

## R5/R6 bead breakdown

Ordered; sizes S ≲ half day, M ≈ 1 day, L ≈ 2 days. Every bead lands green
on `main` per W062 execution doctrine (decisive acceptance check, fresh-eyes
review instructions, `cargo test -p oxcalc-core` + clean differential in the
bead description at creation time).

**R5 — Document surface** (requires R2 complete; R5.1–R5.2 also consume
D1 R2.6's `GridInputState`):

1. **R5.1 — Public `bind_grid_formula` + `BoundGridFormula` (M).** The §3
   verb over the real workspace context; typed bind errors;
   `unresolved_names` surfaced. Acceptance: binds the W011 fixture formula
   producing a key equal to the engine's own mint; name-referencing formula
   reports the unresolved name and evaluates `#NAME?` then self-heals.
2. **R5.2 — Derived-key doctrine in `GridInputState` (M).** `GridCellInput`
   as §3 (source_text+channel authored; key derived at bind); rebind on
   revision navigation. Coordinates with R2.6's implementer (tension T2).
   Acceptance: revision id unchanged by a key-policy-only rebind; undo/redo
   restores formulas exactly; existing grid tests green.
3. **R5.3 — `enter_grid_cell` / `set_grid_cell_value` / `clear_grid_cell`
   (L).** The §2 verbs: OxFml authored-input branch, no-mutation-on-
   diagnostics, empty-text-as-clear, revision advance, editability
   enforcement with typed rejections. Acceptance: `=1+` leaves state
   untouched with diagnostics; `'123` stores text (OxFml classification);
   clear dirties dependents; every path revision-visible.
4. **R5.4 — Consumer defined-name lifecycle verbs + readout (L).** §4
   verbs over the machine setters, dynamic names through R5.1's bind, dirty
   seeds propagated, revision participation. Acceptance: the W011 name
   fixture (`TheInput -> Sheet1!A1`, `D1 = =TheInput*2`): seed → value,
   unseeded → `#NAME?` → self-heal on seed.
5. **R5.5 — Authored readout + editability (M).** §5 `grid_authored_view`
   from `GridInputState`, editability derivation shared with R5.3's
   enforcement. Acceptance: readout of a seeded workbook distinguishes
   literal/formula/spill-display/merged-follower; edit rejections match
   readout classifications exactly (one classifier, property-tested).
6. **R5.6 — Calc-mode discipline + `recalculate_workbook` + FileCached
   provenance (M).** §6: Manual suppresses implicit evaluation (edits and
   interest registration — the T3 seam, consumer-side half), explicit
   recalc verb, provenance on published cells. Acceptance: Manual-mode edit
   leaves stale published values with dirty marks; F9 verb refreshes;
   provenance visible in readout.
7. **R5.7 — `workbook_authored_delta` (M).** §7b from snapshot diffs only.
   Acceptance: the W011 step-4 assertion (one authored edit reported, no
   derived-state leakage), including sheet lifecycle and settings diffs.
8. **R5.8 — Rename to `OxCalcDocumentContext` + surface doc (S).**
   Mechanical rename, `NotAWorkbookWorkspace` gating on document verbs,
   rustdoc index of the document surface. Acceptance: crate compiles with
   no legacy alias exported; doc lists every verb of §§1–7.

**R6 — oxdoc ingestion full scope** (requires R5.1–R5.4; R6.1 can start
against R5.1+R5.2):

1. **R6.1 — Dependency edge + `oxdoc_ingest` skeleton + tier ledger +
   Tier A cells (L).** `oxdoc-model` path dep; sink with wildcard-free
   feature match; single-transaction builder (§9) for
   header/sheets/literals; `WorkbookLoadReport` with the ledger invariant
   test ("every variant consumed or ledgered"); error-code mapping.
   Acceptance: a literals-only workbook loads in one revision; report
   accounts for 29/29 variants on a synthetic all-variant stream.
2. **R6.2 — Formula ingest: bind, shared regions, topology routing,
   degradation (L).** §10 complete: A1 + R1C1 template binds, repeated
   regions, CSE/DataTable/Unknown routing, `BindDegradation` rows,
   FileCached publication seeding. Acceptance: W011 fixture loads and
   renders cached values pre-recalc; corrupt-formula fixture retains text +
   cache + ledger row; differential clean post-recalc.
3. **R6.3 — Names, tables, merges, settings ingest (M).** §12 rows
   1/10/25/26 through the R5.4/R2.5 verbs at commit time (deferred
   installs). Acceptance: scoped-name fixture resolves per scope precedence;
   forward-referencing name (name before target sheet in stream) loads
   clean.
4. **R6.4 — Tier B inert store + overlay seats + digest meta-child (M).**
   §13: `IngestedDocumentFacts`, Arc retention on revisions,
   `#workbook-ingest` digest, CF/RichObject/Cse extension rects.
   Acceptance: revision id changes iff store digest changes; overlay
   readout shows inert CF rects; store survives undo/redo by pointer.
5. **R6.5 — `load_workbook_model` one-call verb + load recalc policy +
   ExternalLink pins (M).** §9 entry points (events + model access), §6
   Automatic/Manual behavior, §14 external pins. Acceptance: Manual-mode
   load renders FileCached values with zero engine runs (perf counter
   evidence); external-ref fixture pins its cache across recalc with a
   ledger row.
6. **R6.6 — Output projection + fixture-class round trip (L).** §7a
   `project_workbook_model_output`; the five-step W011 contract as the
   acceptance test, including step 5's reload-and-compare and Tier B
   verbatim replay (event-level equality against the source stream for
   untouched surfaces).
7. **R6.7 — Upstream gap handovers + register reconciliation (S).** §15
   items filed as OxDoc handover docs; workset register updated; W011's
   upstream-ask ledger annotated "superseded by native verbs" with
   pointers here. Acceptance: handover docs exist and are cited from the
   register; the W011 plan's asks (b)/(c)/(e) map to landed verbs.

Off-critical-path (any time): **H1 — `machine.rs` test-corpus extraction
(S-M, hygiene)** per §16; explicitly not an R5/R6 dependency.

Sequencing notes: R5.1→R5.2→R5.3 is the input-authority chain; R5.4–R5.7
are independent of each other after R5.2; R5.8 last in R5. R6 is a chain
except R6.4 (parallel to R6.2/R6.3 after R6.1). D3's incremental-consumer
work should land the engine-side half of T3 before large-workbook loads are
routine, but nothing in R6 gates on D3.
