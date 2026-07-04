# W062: Ideal Engine Model Rework — Preliminary Plan

## Status

Preliminary plan, authored 2026-07-04 from a five-lane code survey of OxCalc
HEAD `e069136e` plus OxFml and oxdoc-model (evidence pointers in the
appendix). The R1 design beads produce the full designs; this document fixes
the vision, the verified starting map, the program structure, and the open
questions. Owner directive: rework OxCalc to its ideal end state with **no
legacy, compatibility, or bridging constraints** — downstream users
(DnaTreeCalc, tracecalc CLI) adapt afterward. Work happens on `main`.
DnaTreeCalc W011 (host/notebook proof) is deliberately paused until this
program delivers its model; W011's handover asks are absorbed here.

## Purpose

Make OxCalc the best calculation engine model we can build now: one general
structural document model (more general than Excel's, projecting onto the
Excel object model), a multi-profile reference architecture whose
strict-excel profile fully covers Excel referencing, a workbook-scoped
calculation system that preserves and extends the two-model principle
(simple-correct reference oracle + extremely-optimizable engine, permanent
differential), a document-level consumer surface, and full-scope
`oxdoc-model` ingestion.

## Vision (owner-approved)

1. **One general structural model.** A workspace is a tree of nodes. A
   workbook is a workspace whose root plays the *Workbook* role and whose
   child nodes ARE the sheets (grid backing on sheet nodes). Nodes have
   meta-children storing properties. Later node kinds: chart sheets, other
   artifacts. Our object model is more general than Excel's; Excel's OM is a
   projection.
2. **Reference profiles with structural vocabulary.** The multi-profile
   abstraction stays. A profile carries a vocabulary that tells resolution
   how profile tokens map onto model structure ("root is called Workbook,
   its top-level children are called Sheets"). The strict-excel profile
   grows to full Excel coverage: sheet-qualified references, 3D references,
   workbook- and sheet-scoped defined names, structured references, spill
   references, dynamic references.
3. **Two-model calculation, workbook-scoped.** One simple model that is
   correct (the oracle); one model that will be extremely optimized over
   time (concurrent evaluation, intricate caching, rearrangement); a
   permanent differential harness between them. The dependency graph becomes
   workbook-scoped — cell granularity for grid↔grid edges, name granularity
   where the tree joins (tree node ≈ defined name) — and cross-sheet edges
   are real engine edges, never host-composed.
4. **Document surface and ingestion.** A document-level context API with
   native verbs for public formula binding, defined-name seeding, authored
   readout, clear-cell, and neutral output; OxCalc implements the
   `oxdoc_model::OxCalcIngestSink` seam (new `oxdoc-model` dependency) and
   loads a workbook document in one call.

## The verified starting map (survey 2026-07-04, HEAD e069136e)

What exists and is load-bearing:

- **Structural substrate is closer than expected.** `StructuralNode
  { node_id, kind, symbol, parent_id, child_ids(ordered) }` with
  `StructuralNodeKind { Root, Container, Calculation, Constant }` (a
  calc-DAG role, *derived from formula text*) and orthogonal, explicitly
  extensible `NodeBacking { Table, Grid }`. One grid engine per node
  (`grids: Arc<BTreeMap<TreeNodeId, GridBackingState>>`). "Sheet node = node
  with grid backing" is already the implemented shape. Missing: any
  Workbook/Sheet *role* concept, sheet-name→node registry, sheet order as a
  first-class fact, and `workbook.sheets()`-style enumeration.
- **The tree IS a defined-name namespace.** Tree references resolve node
  symbols with walk-up/descent; unresolved names commit as `#NAME?`
  (34c7219c) — the same mental model as Excel defined names. This is the
  natural unification point between tree nodes and workbook names.
- **Reference profiles are a real seam, one layer short.** A profile is
  OxFml's `ReferenceBindProfile` trait (resolution, not grammar; grammar is
  OxFml's shared parser). Three profiles exist (`formula-only`,
  `dna.treecalc.v1`, `excel.grid.v1`). Sheet-qualified references
  (`Sheet1!A1`) **already parse and bind** with sheet identity threaded into
  normal-form keys — only resolution/catalog is single-sheet. 3D references
  have **no grammar production** (the `:`-as-range collision; genuine OxFml
  work). `ReferenceSyntaxCapabilities` flags exist but do not gate the
  parser. `normal_form_key` already embeds `workbook:sheet` — the key space
  is cross-sheet-ready. The two profiles disagree on `!` (grid:
  sheet-qualifier; tree: rejected `cross_workspace_host_path_pending`).
- **Defined names are mature engine-side, absent consumer-side.**
  Workbook-scoped and sheet-scoped names (with precedence), dynamic names,
  `NameIdentity` edges with `#NAME?` self-healing — all present at the sheet
  machine level; unreachable through `OxCalcTreeContext`/`GridBackingSeed`.
- **The two-model principle is grid-local and pays a hidden cost.**
  `GridCalcRefSheet` (oracle) vs `GridOptimizedSheet` with the
  `GridEngineMode::Both` differential is real and load-bearing — but the
  live consumer runs BOTH engines mark-all-dirty on every recalc; interest
  probes narrow only the readout. A fully test-proven incremental path
  (`recalculate_dirty_compact_with_oxfml` + `GridDirtySeed`) exists and is
  never called by the consumer. Tree-side calculation has an **offline**
  two-model pair (the `oxcalc-tracecalc` reference machine +
  `oracle_baseline`/`engine_diff` conformance) but no live in-consumer
  differential analogous to `GridEngineMode::Both`.
- **The CTRO effective graph is workbook-ready in vocabulary, single-sheet
  by construction.** `ExcelGridCellAddress` carries `workbook_id, sheet_id,
  row, col`, so `GridDependency`'s cell/range variants already carry full
  sheet identity; the name/table/dynamic variants are bare strings, and —
  decisively — sheet identity is never consulted for routing: scoping is an
  accident of one-graph-per-sheet composition (`GridInvalidationRef` lives
  inside one `GridCalcRefSheet`). Cycle detection
  (`EffectiveDependencyCycleDetected`), worklist readiness,
  volatile/external seeds, spill facts: all present, per-sheet.
- **Versioning prior art (W057) is the substrate for document state.**
  Content-addressed `WorkspaceRevision` + four snapshot layers + candidate
  overlays. Gap: grid backings are live-only (not revision-retained), so
  workbook undo does not cover sheet edits yet. Meta-nodes exist
  (`is_meta`, namespace-excluded, revision-captured) — the ready-made
  property mechanism. Workbook calc settings (date system 1900/1904, calc
  mode, iteration) have **no home anywhere**.
- **OxFml already contains the identity machinery the grid needs.**
  Four-tier identity (`FormulaSourceIdentity` / `FormulaTemplateIdentity` /
  `PlacedFormulaIdentity` / `RuntimeDependencyIdentity`) and
  `ReferenceFingerprintPolicy::ExcludeCallerAnchorForTemplate` exist, but no
  shipped profile uses the caller-independent policy. **OxFml W077**
  ("strict Excel grid BindProfile and R1C1 identity", epic `fml-7t6`, all
  beads open, seeded by HANDOFF-DNATREECALC-001) is the specced-but-unstarted
  upstream workset that delivers exactly this. W061 already depends on it.
- **oxdoc-model ingestion seam is complete upstream.** 29 `DocumentEvent`
  variants; `OxCalcIngestSink { workbook, sheet_begin, cell_chunk,
  sheet_end, feature(default no-op) }` with borrowed, validated,
  stream-ordered driving (`drive_oxcalc_ingest[_from_model_access]`);
  `WorkbookHeader` carries `date_system` + `calc_mode` (no iteration
  settings — an upstream gap to raise). OxCalc-side homes exist for cells,
  formulas, merged regions, tables, names (flat), sheet identity; **no home**
  for strings(interning), styles, notes, controls, customXml (customXml is
  deliberately package-level, not in the model stream), date system, calc
  settings. The grid overlay enum in `grid/machine/overlay.rs` already
  reserves inert seats (`Cse`, `ConditionalFormat`, `RichObject`) — note a
  second, unrelated `OverlayKind` exists in `recalc.rs`; cite the grid one.
  `DocumentEvent::ExternalLink` exists upstream and needs an explicit
  ingest-tier decision (external workbook references are otherwise absent
  from OxCalc).

Program-hygiene facts: register + beads doctrine per `docs/BEADS.md` and
`WORKSET_REGISTER.md` §4-5 (this workset must splice into the sequencing
chain); no CI — local-execution evidence model with checked-in baselines;
`grid/machine.rs` is a 24.5k-line file that is mostly its own test suite;
`stash@{0}` (unreviewed, 1,531 lines: C3 walk-up + node-as-function + A1/A2)
touches consumer.rs/treecalc.rs/formula.rs/tree_reference_resolution.rs —
squarely in this rework's blast radius.

## Target architecture direction (preliminary)

These are the R1 design inputs — directions with rationale, not final
designs. Each carries its open questions into the R1 beads.

1. **Workbook as workspace (default; D1 revisits container semantics);
   roles are explicit and orthogonal.** Default: one workspace is one
   document. Add an explicit role concept (Workbook root,
   Sheet children; later ChartSheet, etc.) *orthogonal* to the derived
   `StructuralNodeKind` — an explicitly-set attribute or role registry, not
   an overload of kind or backing. Sheet order defaults to root
   `child_ids` order; a rename-stable sheet-name→node registry (feeding
   `NameIdentity`-style self-healing for sheet-qualified references) becomes
   part of workspace state. Tree nodes remain first-class name-scoped
   citizens anywhere in the tree — the general model subsumes both TreeCalc
   trees and workbooks; an Excel workbook is the profile-constrained
   projection.
2. **Vocabulary as an additive profile layer; two binders stay.** Keep
   OxFml-owns-grammar / profile-owns-resolution. Add the structural
   vocabulary (role naming + sheet-name lookup + `!` semantics) as a layer
   the profile carries — pluggable into the existing dyn-object seam.
   Reconcile `!`: in the general model `!` qualifies a *container role*
   (workbook/sheet for strict-excel; workspace for the tree profile once
   cross-workspace lands), resolved through the vocabulary rather than
   hard-coded per profile. Grammar gating via the existing capability flags
   becomes real where needed; 3D references get a grammar production in
   OxFml (coordinate with W077). Strict-excel completion = cross-sheet
   resolution + 3D + consumer-level scoped names + the W077
   caller-independent template identity.
3. **Workbook-scoped graph — federation as the default shape (D3 revisits
   federation vs flat index), oracle first.** Keep per-sheet
   engines and per-sheet graphs; add a workbook coordination layer that owns
   cross-sheet edges (new dependency variants carrying target sheet
   identity), routes dirty-seed closure across sheets, dependency-orders
   evaluation across sheet boundaries, and extends cycle detection
   workbook-wide. Extend the two-model principle to that scope: a
   workbook-level reference oracle (mark-all across sheets — simple,
   correct) and the optimized coordinator (incremental, seeded), compared by
   the same differential discipline. Tree evaluation joins the workbook
   graph at name granularity (tree node ≈ defined name), making
   tree-formula↔grid-cell references first-class edges. Workbook-wide
   volatile semantics are in scope: NOW()/RAND() tick once, coherently, per
   workbook recalc across sheets. Wire the consumer onto the existing
   incremental optimized path (dirty seeds + previous valuation) — D3 may
   split this out as an earlier independent landing rather than gating the
   cheapest win behind the federation build; the acceptance bar is
   measurable either way: a single-cell edit recalc touches O(dirty cone),
   not O(authored cells), evidenced by the existing perf counters.
4. **Concurrency preparation, not construction.** The optimized lane is
   *designed* for future concurrent evaluation: deterministic worklist,
   pure providers, no ambient interior mutability in evaluation paths
   (runtime_trace's `RefCell` capture moves to per-evaluation buffers),
   Send-auditable state. No concurrent executor is built in W062.
5. **Document surface.** `OxCalcTreeContext` evolves into the document
   context (it already owns multiple workspaces, candidates, revisions).
   New native verbs: load-workbook-model (ingest), public
   `bind_grid_formula`-class binding, defined-name seeding (both scopes +
   dynamic), authored readout (per-cell kind/source/editability),
   `ClearCell`, workbook calc-settings access, neutral output (authored
   state + edits since a revision) for OxDoc save. Grid backings enter
   revision retention so document undo covers sheet edits. Properties and
   workbook settings live as meta-children under the owning node (settings
   participate in revision identity and invalidate dependents — e.g. date
   system flips invalidate date-dependent formulas).
6. **Full-scope ingestion with honest tiers.** OxCalc implements
   `OxCalcIngestSink` in a new `oxdoc_ingest` module (dep: `oxdoc-model`
   only). Tier A (calculation-bearing): header/date-system/calc-mode,
   strings (resolved at ingest), cells/literals/formulas (+ topology +
   shared-formula regions), defined names with scoping, tables, merged
   regions, sheet lifecycle/order. Tier B (inert overlays/meta): styles
   presence, notes, controls, conditional formats — stored as inert
   overlay/meta facts (the reserved `OverlayKind` seats) or explicitly
   ledgered as not-ingested; never silently dropped, never calc-visible.
   Formula binding at ingest goes through the strict-excel profile (W077
   identity), with `#NAME?`/`#REF!` degradation rather than ingest failure.

## Program structure

Waves are sequential program phases; each becomes an epic lane with beads.
R1 designs may reorder R2-R6 details.

- **R0 — Program bootstrap (this workset landing).** Register entry
  (with closure condition and rollout_mode per §4) + a **rewrite** of the
  §5.1 go-forward sequence, not an append; stash@{0} triage
  (harvest-or-drop with recorded reason); workset reconciliation with a
  named disposition for EVERY open workset: W059 → absorbed, owned by D4
  (authored-input path of the document surface); W060 → absorbed, owned by
  D3 (calc-time reference representation lane); W061 → becomes this
  program's strict-excel execution arm (D2/R3); W055/W056 in-flight beads
  get explicit pause/continue decisions (W055 cycle-engine design is
  co-owned by D3); W054/W057 dependencies re-affirmed (grid revision
  retention must re-target W054's retention classes); W049/W052/W053
  re-sequenced explicitly — W049's "formalize against the settled engine"
  premise now waits for W062, and W053 (staged concurrency) folds into
  Direction 4's concurrency-prep constraints; W051/W058 positions restated.
  OxFml W077 activation handover: W077 + the 3D grammar production run as
  an explicit **parallel upstream lane** with a named entry gate before R3
  consumes them.
- **R1 — Architecture designs (four documents, each Fable-reviewed).**
  D1 structural model: roles, sheet registry/order, meta-properties,
  settings home, revision retention of grids. D2 reference architecture:
  vocabulary layer, `!` reconciliation, strict-excel completion, 3D
  grammar coordination, W077 adoption, normal-form-key stability. D3
  calculation: workbook graph federation, cross-sheet edges/seeds/cycles,
  workbook oracle, incremental consumer wiring, differential extension,
  concurrency-prep constraints, tree-evaluation unification. D4 document
  surface + ingestion: context API verbs, output contract, ingest tiers,
  `#NAME?`/degradation policy, oxdoc-model gaps to raise upstream
  (iterative-calc settings).
- **R2 — Structural model implementation** (roles, registry, settings,
  meta-properties, grid revision retention).
- **R3 — Reference/vocabulary implementation** (vocabulary layer, strict
  excel cross-sheet resolution, scoped-name consumer verbs; 3D once OxFml
  grammar lands; W077 adoption for template identity).
- **R4 — Workbook calculation** (federated graph, cross-sheet closure,
  workbook oracle + differential, incremental consumer path, cycle scope).
- **R5 — Document surface** (binding/seeding/readout/clear/output verbs,
  undo over grids, settings invalidation).
- **R6 — oxdoc ingestion full scope** (sink implementation, tiers, load
  entry point, round-trip readout evidence with OxDoc fixtures).
- **R7 — Downstream adaptation** (DnaTreeCalc adapts; W011 resumes on the
  new surface; tracecalc CLI updates).

Two-model discipline throughout: every R4+ behavior lands in the oracle
first (or simultaneously), the differential harness extends with it, and
optimized-lane divergence is a stop-the-line defect.

## Execution process

- **Coordination:** beads under epic W062 in this repo, created per wave.
  Every implementation bead description MUST include: (a) the decisive
  acceptance check, (b) **fresh-eyes review instructions** — which artifacts
  a reviewer with no session context reads, what properties they verify,
  and what evidence they record in the bead before close — and (c) the
  instruction to commit on `main` with a green suite
  (`cargo test -p oxcalc-core`) and the differential harness clean.
- **Model policy:** Sonnet for mechanical surveys/inventories/corpus runs;
  Opus for standard implementation beads; Fable for architecture designs,
  design reviews, reference-resolution and dependency-graph internals, and
  final review gates on important artifacts.
- **Evidence:** local-execution doctrine (checked-in baselines, repo-relative
  artifact paths); no CI assumed.
- **Main-branch discipline:** small, green, committed increments; no long
  side branches.

## Open questions for R1

D1: role representation (kind vs field vs registry) and serde migration;
sheet order vs non-sheet root children; one-workspace-per-workbook vs
multi-workspace container semantics; settings home (meta-nodes vs snapshot
layer) and revision-identity participation; grid retention policy
(COW snapshots vs edit-log replay) and W054 GC interplay.
D2: vocabulary placement (trait method vs sibling trait vs config);
grammar-gating vs resolution-only; 3D grammar design (lexer token vs
post-parse reinterpretation) and 3D dependency shape (per-sheet fan vs one
edge); tree-profile `!`/cross-workspace path; normal-form-key format
stability vs revision; external workbook references (`[Book2]Sheet1!A1`) —
explicit in/out decision (typed exclusion acceptable, but decided); sheet
deletion policy for dangling sheet-qualified references (Excel: `#REF!`)
alongside the rename-stable registry; LAMBDA-valued names (a name resolving
to a callable, not a range) under the tree-node≈defined-name unification.
D3: federation vs flat workbook index; cross-sheet `GridDependency`
variants; optimized engine growing a persistent graph vs transient
derivation; tree-side two-model (own oracle pair vs joining the grid
family); W055 cycle-engine interplay (joint redesign vs retrofit);
concurrency prerequisites (trace buffers, Send audit).
D4: ingest targeting the new structural model directly vs staging; Tier B
storage shape (overlays vs meta-children vs ledger-only); string-interning
policy; how much of `machine.rs` decomposition rides along vs defers.

## Appendix — survey evidence

Five survey lanes (2026-07-04, session `083289bf`, workflow
`wf_8c4c46b0-56b` + follow-ups): repo/coordination inventory; tree/consumer
model; reference/profile architecture (OxFml + OxCalc); calc/dependency
machinery; OxFml/oxdoc-model seams. Key anchors verified in code:
`structural.rs` (nodes/backings/kinds), `consumer.rs:1430`
(`OxCalcTreeContext`), `tree_reference_system.rs` /
`grid/reference_engine.rs` (profiles), OxFml `binding/profile.rs:452`
(`ReferenceBindProfile`), `binding/mod.rs:3238` (template identity),
`grid/machine/invalidation.rs` (dependency layers),
`grid/machine/differential.rs` (`GridEngineMode`),
`optimized_sheet.rs:1729` (unused incremental path),
`workspace_revision.rs` (W057 layers), oxdoc-model `lib.rs:2152`
(`OxCalcIngestSink`), OxFml `docs/worksets/W077_*.md` + epic `fml-7t6`
(open). DnaTreeCalc-side context: `DnaTreeCalc/docs/ux/
DNACALC_HOST_CORE_XLSX_NOTEBOOK_PROOF.md` (W011, paused at Wave 0/1).
