# W062 D2 — Reference Architecture Design

## Status

R1 design document for bead `calc-5kqg.5`, authored 2026-07-04 against OxCalc
HEAD `e069136e` and OxFml HEAD `2183651` (sibling repo, anchors re-verified on
the authoring date). Program authority:
`docs/worksets/W062_IDEAL_ENGINE_MODEL_REWORK.md`. Fixed upstream interfaces:
D1's exported contracts C1–C8
(`docs/design/W062_D1_STRUCTURAL_MODEL.md` §Contracts) — especially C1
(vocabulary maps tokens onto roles, never defines them), C2 (sheet identity /
`DeletedSheetFact` tombstones), C3 (sheet order = filtered root `child_ids`),
and C8 (workbook = workspace). Harvested inputs:
`docs/worksets/W062_R0_STASH_TRIAGE.md` Harvest 1 (tree-profile name
precedence, adopted verbatim-in-substance in §7) and Harvest 2 (callable
invalidation-edge requirements, adopted in §8).

Scope: the profile structural-vocabulary layer, `!` reconciliation,
grammar-gating, strict-excel completion (cross-sheet catalog routing, 3D
shape, scoped names, W077 identity adoption), external workbook references,
sheet-deletion `#REF!` policy, LAMBDA-valued names, the tree cross-workspace
path, and normal-form-key stability. Every D2 open question from the W062
plan is answered (§Open questions answered) or explicitly deferred with
reason. The document ends with the R3 bead breakdown and the W077 gate
mapping.

**Seam guardrail (inviolable):** the structural vocabulary must not move grid
semantics into OxFml. The governing precedent is the W077 GridBounds
ratification (OxFml `docs/worksets/W077_strict_excel_grid_bind_profile_and_
r1c1_identity.md` §GridBounds ratification): OxFml owns grammar and the bind
lifecycle and preserves syntax facts, capability facts, and pass-through
shape facts; grid/model semantics stay in OxCalc. Every decision below is
audited against this line; §Seam audit summarizes the result.

## Verified starting anchors (and two corrections to the program doc)

- **`ReferenceBindProfile` is the frozen dyn seam.** Trait, `BindProfile`,
  `ReferencePolicy`, `ReferenceFingerprintPolicy::{IncludeCallerAnchor,
  ExcludeCallerAnchorForTemplate}`, `ReferenceSyntaxCapabilities`,
  the four identity newtypes (`FormulaSourceIdentity`,
  `FormulaTemplateIdentity`, `PlacedFormulaIdentity`,
  `RuntimeDependencyIdentity`), `transform_reference` /
  `instantiate_reference` — all at OxFml
  `crates/oxfml_core/src/binding/profile.rs` (trait at `:452`). The shape is
  frozen of record in the W077 doc §Frozen public shape.
- **Correction 1 — W077 is BUILT, green, and CLOSED**, not
  "specced-but-unstarted" as the W062 plan's survey section says. OxFml
  commits `965cec0` (shape freeze + GridBounds ratification), `3dd605c`
  (caller-independence acceptance), `b16cb6c` (transform/plan-reuse
  coverage); all four fml-7t6 beads closed, suite green;
  `reference_profile_api_tests` green (18 tests as of `b16cb6c`, which
  added the transform/plan-reuse coverage). The 3D grammar now has its own
  planned upstream workset, **W078** (`fml-k9s`). Decisive as-built fact this
  design adopts: **the strict profile keeps
  `fingerprint_policy() = IncludeCallerAnchor`** (OxCalc
  `grid/reference_engine.rs:622-624`) and achieves caller-independent
  template identity on the R1C1 channel via the caller-independent normal
  form; A1 deliberately rebinds per placement for `$`-fidelity.
  `ExcludeCallerAnchorForTemplate` is shipped and consumed by the **tree**
  profiles (`tree_reference_system.rs:144`, `:232`).
- **Correction 2 — the normal-form-key anchor is OxCalc-side.**
  `normal_form_key_for_reference` lives at OxCalc
  `grid/reference_engine/parse.rs:438-455` and already embeds
  `workbook:sheet` in every cell/area key
  (`{profile}:cell:{workbook}:{sheet}:R..C..`). The key space is
  cross-sheet-ready today.
- **Sheet-qualified grammar already exists in OxFml, semantics-free.**
  `SyntaxKind::QualifiedReferenceExpr` splits `qualifier!target`
  (`binding/mod.rs:2906-2913`, `try_parse_simple_reference_fragment`
  `:2922`), and `parse_reference_qualifier` (`:2973-3004`) already parses
  the external `[Book2]Sheet1` bracket form into
  `ParsedQualifier { sheet_id, external_target_id, is_external }`. OxFml
  hands the qualifier through as a fact; it attaches no model meaning —
  exactly the guardrail shape.
- **The strict profile currently punts external ranges**: `bind_range`
  returns `LegacyCompatibility` when either endpoint has an
  `external_target_id` (`grid/reference_engine.rs:717-719`).
- **`ReferenceSyntaxCapabilities` is inert.** The flags are defined
  (`profile.rs:34-52`) but consulted nowhere in `binding/mod.rs` (verified
  by search on the authoring date). Grammar-gating is a genuine open design
  point, not a wiring gap.
- **The tree profile rejects `!` today**: any token containing `!` returns
  `ContextHostNameResolution::Unsupported("cross_workspace_host_path_
  pending")` (`tree_reference_resolution.rs:191-193`); `Ambiguous` maps to
  `ReferenceAtomBindResult::Rejected` (`tree_reference_system.rs:302-305`).
- **Callable machinery on main**: `runtime_binding_for_node` →
  `callable_binding_from_calc_value` → `DefinedNameBinding::Callable`
  (`treecalc.rs:5361-5409`); callables travel as `RichValue::Callable` on
  `working_calc_values`.
- **Grid dependency vocabulary**: `GridDependency::{Cell, Range, Name,
  NameIdentity, Table, TableIdentity, SpillFact, SpillBlocker, …}`
  (`grid/machine/invalidation.rs:198-211`); `Cell`/`Range` carry full
  `ExcelGridCellAddress` sheet identity, `Name`/`Table` are bare strings;
  no 3D/span variant exists.
- **Cross-workspace seam (tree side)**: `SyntheticReferenceBinding.
  workspace_target` already produces a `ReferenceLike` with
  `ReferenceKind::ThreeD` handles (`treecalc.rs:5385-5390`), and
  `DependencyDescriptor.workspace_target` is folded into dependency-shape
  identity (D1 §6).

---

## 1. The structural vocabulary layer: placement

**Decision:** The vocabulary is an **OxCalc-side sibling trait attached by
subtrait**, not an OxFml trait method and not a passive config struct:

```rust
// oxcalc-core (new module: reference_vocabulary.rs)
pub trait StructuralVocabulary {
    /// Profile term for a container role: how this profile names the
    /// model's roles ("Workbook", "Sheet" for strict-excel; "Workspace"
    /// for the tree profile). Display/diagnostic vocabulary only.
    fn role_term(&self, role: NodeRole) -> Option<&str>;

    /// Resolve a `!` qualifier against the model. The catalog is the
    /// D1-C1 lookup surface (sheet registry, workspace aliases).
    fn resolve_container_qualifier(
        &self,
        qualifier: &ParsedContainerQualifier,   // raw text + optional [external] part
        catalog: &dyn ContainerCatalog,          // §4.1 / §9
    ) -> ContainerResolution;                    // Sheet(TreeNodeId) | Workspace(WorkspaceHandle)
                                                 // | External{book, sheet} | Dormant(NormalizedName)
                                                 // | Rejected{code, message}

    /// Deletion semantics for dangling container-qualified references (§6).
    fn container_deletion_policy(&self) -> ContainerDeletionPolicy;
}

pub trait OxCalcReferenceProfile: ReferenceBindProfile {
    fn vocabulary(&self) -> &dyn StructuralVocabulary;
}
```

OxCalc call sites that need vocabulary (bind construction, catalog routing,
deletion transforms, rendering) hold `&dyn OxCalcReferenceProfile`; every
crossing of the OxFml seam upcasts to `&dyn ReferenceBindProfile`. Both
shipped profile families (`StrictExcelGridReferenceProfile`,
`TreeCalcReferenceBindProfile`/`TreeCalcContextReferenceBindProfile`)
implement the subtrait; the profile's own `bind_*` methods consult the same
vocabulary object internally, so **bind-time and resolution-time see one
vocabulary by construction** — there is no second registry that can drift.

**Alternatives rejected:**

- *Trait method on `ReferenceBindProfile` (OxFml)* — rejected on the seam
  guardrail. `NodeRole`, `TreeNodeId`, sheet registries, and workspace
  handles are OxCalc model types; a vocabulary method on the OxFml trait
  either drags those types upstream (guardrail violation, the exact thing
  the GridBounds ratification withdrew) or forces a stringly-typed
  vocabulary ABI that erases the type safety the layer exists to provide.
  The W077 shape is also frozen of record; reopening it needs cause, and
  there is none — OxFml already delivers everything the vocabulary needs
  (the parsed qualifier fact).
- *Config struct carried in `BindProfile`* — rejected because vocabulary is
  behavior (resolution against a live catalog), not data. A struct of
  role-name strings cannot resolve `Sheet1` to a `TreeNodeId` or apply a
  deletion policy; it would push the real logic into per-call-site helpers,
  which is the drift factory the subtrait avoids.
- *Side registry keyed by `profile_id`* — workable but strictly worse: two
  sources of truth (the profile object actually bound vs the registry
  entry), a lookup that can miss, and lifetime friction for
  context-carrying profiles like `TreeCalcContextReferenceBindProfile<'a>`
  which already hold the structural snapshot they resolve against
  (`tree_reference_system.rs:54-59`). The subtrait keeps vocabulary on the
  object that already has the context.

**Availability at bind and resolution time.** Bind time: the profile impl
consults `self.vocabulary()` inside `bind_atom`/`bind_name`/`bind_range`
when a `parsed_qualifier` is present — it classifies the qualifier
(local-sheet / cross-sheet / external / dormant) and stamps the result into
the record's normal-form key and payload. Resolution time: the workbook
catalog router (§4.1) and the deletion transform driver (§6) consult the
same object through `&dyn OxCalcReferenceProfile`. `dyn`-compatibility is
preserved: the subtrait adds one object-safe method.

**Rationale:** C1 fixes the division of labor — profiles map tokens onto
roles, the structure owns the roles. The subtrait is the smallest construct
that makes that mapping a first-class, testable object while leaving the
OxFml ABI untouched and keeping one vocabulary per bound profile instance.

## 2. `!` reconciliation: container-role qualification

**Decision:** `!` has **one meaning across all profiles**: *the left side
names a container in the profile's structural vocabulary; the right side
resolves within that container's namespace.* What differs per profile is
only which container roles the vocabulary admits:

| Profile | `!` left side resolves to | Right side |
| --- | --- | --- |
| `excel.grid.v1` | a Sheet-role node in this workbook (via D1-C1 `sheet_index`), or `[Book]Sheet` → external workspace (§5) | A1/R1C1 atom, range, or sheet-scoped defined name |
| `dna.treecalc.v1` | a workspace in the same `OxCalcTreeContext` (alias catalog, §9) | a tree path (`A.B.C`) resolved by Harvest-1 precedence *within that workspace* |
| `formula-only` | nothing — vocabulary admits no containers; qualifier ⇒ typed rejection | — |

OxFml's role stays exactly what it is today: parse `qualifier!target` into a
`QualifiedReferenceExpr` and deliver `parsed_qualifier` /
`external_target_id` as facts (`binding/mod.rs:2906`, `:2973`). No OxFml
change is needed for `!` reconciliation — the reconciliation is entirely a
vocabulary-layer fact. The tree profile's
`cross_workspace_host_path_pending` stub is retired by §9; the strict
profile's already-working sheet qualifier is re-expressed through
`resolve_container_qualifier` with zero behavior change for the
single-sheet case.

**Rationale:** the two profiles never actually disagreed about what `!`
*is* — both treat it as container qualification; the tree profile simply
had no container catalog to resolve against. Naming that shared semantics
once, in the vocabulary trait, prevents the real risk: a third profile
inventing a subtly different `!`.

## 3. Grammar-gating vs resolution-only

**Decision:** `ReferenceSyntaxCapabilities` becomes real — but as
**binder-routing gates in OxFml's bind lifecycle, not parser/lexer forks.**
The grammar stays single and shared (a W062 axiom: OxFml owns grammar,
profiles own resolution). Concretely, in `binding/mod.rs`:

- Before routing a construct to a profile `bind_*` method, the binder
  consults the active profile's capabilities: `structured_references:
  false` ⇒ `[...]` constructs are not offered to
  `bind_structured_reference` and fall to the name/error path with a typed
  diagnostic; `spill_references: false` ⇒ `#` selector routing is
  suppressed; `host_references: false` ⇒ host-extended atoms are not
  offered; `r1c1_references: false` ⇒ the R1C1 channel bind is rejected
  with a typed capability diagnostic rather than silently mis-parsed.
- The parse tree is identical regardless of profile. Tooling (spans,
  language service, render) never sees a profile-dependent grammar.
- **One genuine grammar addition exists — 3D references** — and it is gated
  by a **new additive flag** `sheet_span_3d_references: bool` on
  `ReferenceSyntaxCapabilities` (default `false`, so
  `worksheet_legacy()` behavior is bit-identical). Profiles that don't opt
  in never see the production's output routed to them.

**Alternative rejected:** per-profile grammar (capability flags consulted by
the lexer/parser). A profile-dependent parse tree forks every downstream
consumer — incremental reparse, completion, span-based diagnostics — and
breaks the "parse once, bind per profile" lifecycle that plan caching
(W075/W077) is built on. Resolution-only (status quo, flags stay inert) is
also rejected: it leaves capability truth as dead configuration, and the 3D
production genuinely needs a gate so tree/formula-only profiles never
receive `Sheet1:Sheet3!A1` routings they cannot mean.

**Seam audit:** capability flags are syntax/capability facts — exactly what
the guardrail says OxFml owns. No grid semantics move.

## 4. Strict-excel completion

### 4.1 Cross-sheet resolution: the workbook catalog router

Sheet-qualified references already parse and bind with sheet identity in
the normal-form key; what is single-sheet today is resolution — each
`GridCalcRefSheet` resolves keys against itself only (scoping "an accident
of one-graph-per-sheet composition", W062 survey).

**Decision:** Add a **`WorkbookReferenceCatalog`** in oxcalc-core consumer
territory (new module beside `consumer.rs`), the single implementation of
the `ContainerCatalog` surface the vocabulary resolves against:

- Built from D1's structures: `sheet_index[NormalizedSheetName] →
  TreeNodeId` (C1), sheet enumeration in C3 order, tombstones from C2. It
  maps a resolved sheet node to that node's grid engine handle.
- **Bind stays existence-blind.** The strict profile binds `Sheet1!A1`
  without checking that `Sheet1` exists — validity
  `ValidAfterInstantiation`, exactly as today
  (`reference_engine.rs:3736`). Existence is a resolution-time catalog
  fact. This keeps bound records and template identities stable across
  rename and delete and keeps plan caches hot. One transition is not free:
  when a never-existed sheet name is later created, records bound in the
  name-keyed dormant form migrate to the token-keyed form via a
  heal-triggered rebind (the `NameIdentity` precedent) — R3.3 owns that
  migration.
- **Routing at dependency extraction / instantiation:** the router maps a
  dependency key's sheet component to the target sheet node and yields a
  cross-sheet dependency descriptor `{ target_sheet_node, GridDependency }`
  for D3's coordination layer. `GridDependency::Cell/Range` already carry
  full sheet identity in `ExcelGridCellAddress`; the router is what finally
  *consults* it.
- **Unknown sheet name** ⇒ a **dormant sheet-identity edge** keyed on the
  normalized name (the `GridDependency::NameIdentity` pattern,
  `invalidation.rs:202`), value semantics `#REF!` while dormant. Whether it
  may heal is the §6 policy (for strict-excel: a *never-existed* name may
  heal on creation; a *deleted* sheet's references are hard-`#REF!` — see
  §6 for the distinction and rationale).
- Sheet **rename** touches no keys and no edges: keys carry the stable
  sheet identity token (§10), the catalog re-indexes by the new normalized
  name (D1 C2 rename-stability), and only render output changes.

### 4.2 3D references

**Grammar (delegated upstream, separate workset).** The `Sheet1:Sheet3!A1`
production collides with `:`-as-range-operator and needs bounded lookahead
(the 3-token-lookahead `SheetSpan3DReferenceExpr` direction: identifier,
`:`, identifier, `!` commits the span reading; otherwise the range operator
wins). **Decision:** this is a **separate OxFml workset — W078**
(`OxFml/docs/worksets/W078_three_dimensional_sheet_range_reference_
grammar.md`, bead `fml-k9s`, planned; W077 itself is closed-shape and
CLOSED), delivering:
the production, the `sheet_span_3d_references` capability flag (§3), and a
`ReferenceSheetSpanBindRequest { start_qualifier, end_qualifier, target… }`
routed to a new default-`Unsupported` `bind_sheet_span` method on the trait
(additive, default-preserving — the frozen-shape freeze allows additive
default methods; the W077 doc's migration rule is default-preserving
extension). Answering the plan's lexer-vs-post-parse question: **a parser
production with bounded lookahead** — not a lexer token (the lexer cannot
disambiguate `:` without context) and not post-parse reinterpretation
(rewriting an already-built range node smears spans and breaks incremental
reparse). Boundary note carried from W078: a 3D sheet span is a **distinct
reference class**, not sugar for a same-sheet multi-area union — the
OxFunc boundary keeps multi-area (union) semantics separate, and the
span's value semantics (per-sheet aggregation, no implicit union
flattening) belong to the consumer.

**OxCalc dependency shape.** **Decision: one stored 3D edge, expanded to
per-sheet edges at closure time — never a materialized per-sheet fan.** A
new variant:

```rust
GridDependency::SheetSpan(GridSheetSpanDependency {
    workbook_id: String,
    start_sheet: SheetIdentityToken,   // §10
    end_sheet: SheetIdentityToken,
    rect: GridRect,  // NOTE: GridRect (geometry.rs:70-77) embeds
                     // workbook_id/sheet_id; R3.9 either mints a
                     // sheet-agnostic rect type or defines the rule that
                     // the span ignores the rect's embedded sheet identity
})
```

The workbook coordination layer (D3) expands the span against the *current*
C3 sheet order whenever it computes dirty closure or evaluation order.
Rationale: span membership is a function of sheet order, and sheet
insert/move/delete changes it. A stored fan must be rewritten on every
sheet lifecycle edit (and is wrong between the edit and the rewrite); a
stored span re-expands correctly for free, and the stored fact matches the
authored fact — `SUM(Sheet1:Sheet3!A1)` *means* "whatever sheets lie
between", which is precisely why Excel makes inserting a sheet inside a
span change results. Cost: expansion is O(sheets-in-span) per closure pass
over a root fan-out that D1 §3 already argues is small. The normal-form key
gains an additive shape (§10). Endpoint deletion transforms are in §6.

### 4.3 Consumer-level scoped defined names

Engine-side scoping is mature (workbook- and sheet-scoped with precedence,
dynamic names, `NameIdentity` healing — W062 survey); nothing reaches
`OxCalcTreeContext`. D2 owns the **resolution semantics**; D4 owns the full
verb surface (seeding is a Direction-5 document-surface deliverable). R3
lands the minimal verb needed to make resolution testable end-to-end.

**Decision — resolution order for an unqualified name in a strict-excel
cell on sheet S:**

1. caller-local structured-reference column (existing behavior,
   `reference_engine.rs:560-588` — unchanged, it is not a defined name);
2. sheet-scoped name on S;
3. workbook-scoped name;
4. **tree-node names under the workbook projection**: a non-sheet,
   non-meta Calculation/Constant node that is a direct child of the
   workbook root participates as a workbook-scoped defined name (symbol
   folded by the shared C1 fold rule). This is the
   tree-node≈defined-name unification made concrete: workbook-scope names
   and root-level tree nodes are one namespace, and a collision between a
   seeded workbook name and a root node symbol is a **typed rejection at
   definition time** (never a silent shadow);
5. unresolved ⇒ `#NAME?` (the shipped 34c7219c contract).

Qualified `Sheet1!Name` forces sheet scope of Sheet1 (via the §2
vocabulary); a qualifier naming the workbook forces workbook scope. The
minimal R3 verb: `define_name(workspace, scope: NameScope, name, formula_or_
target)` with typed collision rejection; enumeration/readout rides D4.

### 4.4 W077 caller-independent template identity: adoption

**Decision:** Adopt the **as-built** W077 ABI unchanged. Specifically:

- The strict profile **keeps `IncludeCallerAnchor`** and keeps deriving
  caller-independent `FormulaTemplateIdentity` from the R1C1 normal form
  (ratified as-built; witnessed by
  `strict_profile_r1c1_template_identity_is_caller_independent`). We do
  **not** switch it to `ExcludeCallerAnchorForTemplate`; A1
  rebind-per-placement is deliberate `$`-fidelity behavior
  (`strict_profile_a1_incremental_bind_rebinds_when_caller_anchor_changes`).
- What R3 actually *adopts* is downstream use: template-identity-keyed
  **compiled-plan sharing** for region-stamped formulas (fill/paste/ingest
  shared-formula regions store one template identity + per-cell placed
  identities, so N placements of one formula compile once), and the
  ingest handshake for D4's Tier-A shared-formula regions (D4 hands the
  region + one source text; R3's machinery mints template identity once
  and instantiates per cell).
- `GridBounds` stays consumer-side per the ratified decision; nothing in
  this design asks OxFml for bounds.

No upstream landing is required for any of this — the gate named in the
W062 plan ("generic BindProfile ABI landed + `ExcludeCallerAnchorForTemplate`
usable by a shipped profile") is **already satisfied** (tree profiles
consume the policy; strict profile consumes the seam end-to-end).

## 5. External workbook references (`[Book2]Sheet1!A1`)

**Decision: IN for binding, identity, and routing to loaded workbooks; OUT
for file loading and cached external values.** Explicit boundary:

**In (R3):**
- The strict profile binds external references instead of punting: the
  `bind_range` external punt (`reference_engine.rs:717-719`) is replaced
  with a real bind; external *atoms* are handled binder-side in OxFml
  (`binding\mod.rs:801`, `:3015`), so the atom-path change lands there,
  not as a profile punt. Validity:
  `DynamicOrHostSensitive` (resolution depends on host/context state —
  whether the workbook is loaded). The OxFml side already parses the
  bracket form (`parse_reference_qualifier`, `binding/mod.rs:2973`);
  nothing upstream is needed.
- Normal-form keys carry the external target in the existing workbook
  component (§10) — the key shape does not change, the workbook component
  stops being constant.
- Resolution: an external workbook is **another workspace in the same
  `OxCalcTreeContext`** (D1 §6/C8 — the structural answer is settled). The
  catalog router maps `external_target_id` through a context-level
  workbook-alias catalog (filename/alias → workspace id) to the target
  workspace's own `WorkbookReferenceCatalog`, then routes as §4.1. The
  dependency crosses the workspace seam via the existing
  `DependencyDescriptor.workspace_target` machinery.
- **Unloaded target ⇒ `#REF!`** with a typed diagnostic
  (`excel.grid.external.workbook_not_loaded`) and a dormant identity edge
  that heals when a workspace with that alias is loaded.

**Out (typed exclusions, recorded here deliberately):**
- No file loading triggered by evaluation (loading is a D4/R6 document
  verb; evaluation never does I/O).
- **No cached-external-value store** — *scoped by owner reconciliation
  with D4 §14 (2026-07-04)*: this exclusion means no separate runtime
  store or link manager. Values ingested from a file's external-link
  cache ARE published, through the ordinary publication channel with
  `FileCached` provenance (D4's design), pinned until an explicit
  refresh or a sibling-workspace load — at which point THIS section's
  routing applies and an unavailable target becomes typed `#REF!`.
  Newly authored external references with no cache and no loaded
  sibling evaluate to `#REF!` from the start, as written below. A cache
  is otherwise a possible future D4 ingest artifact
  (`DocumentEvent::ExternalLink`
  carries cached values upstream); if it lands, it lands as an ingest-fed
  overlay, not an evaluation-side mechanism. Until then D4's ingest tier
  must ledger `ExternalLink` cached values as not-consumed (Cross-design
  tensions, T5).
- No external-reference *editing* affordances (link management, relink).

**Rationale:** typed partial-in beats both extremes. Full-out contradicts
the vision ("full Excel coverage") and wastes the fact that grammar,
qualifier parsing, workspace targeting, and dependency identity all exist;
full-in (value caching, link manager) drags D4 document-surface work into
the reference layer.

## 6. Sheet-deletion `#REF!` policy

D1 C2 delivers the information contract: `DeletedSheetFact { node_id,
normalized_name, display_name, sheet_position, deleted_at_snapshot_id,
grid_input_identity }`, retained with revisions. D2 decides what a dangling
reference *means*, and it is a **vocabulary-carried, per-profile policy**:

```rust
pub enum ContainerDeletionPolicy {
    HardRefError,        // strict-excel
    DormantIdentityHeal, // tree profile (and default for lenient profiles)
}
```

**Strict-excel: `HardRefError` — Excel-faithful.**

- On `delete_sheet`, the consumer drives `transform_reference` with a
  `StructuralEdit` payload (`SheetDeleted { sheet: SheetIdentityToken }`)
  over bound records targeting the dead sheet. Point/range references on
  the deleted sheet ⇒ `ReferenceTransformOutcome::FullyInvalid`, the bound
  record becomes a `RefError` record, and rendering shows `#REF!`
  *in the formula text* — Excel's exact behavior.
- **Recreating a sheet with the same name does NOT heal** these
  references. Excel does not heal them, and the strict profile's charter
  is fidelity. This deliberately narrows D1 C2's heal-on-recreate wording:
  C2 supplies the *mechanism* (normalized-name identity edges that can
  heal); this profile's vocabulary declines to use it for
  deleted-sheet references. (It still uses dormant-heal for
  **never-existed** sheet names bound before the sheet exists — §4.1 —
  this is this design's own policy — Excel itself typically intercepts
  unknown sheet names at formula entry rather than committing a dormant
  `#REF!`, so no Excel-fidelity claim is made for the dormant form.
  The distinction is: deletion is a destructive
  transform of the bound record; never-existed is an unresolved dormant
  edge.) Flagged as tension T1.
- **Undo heals — recreate does not.** Revision navigation restores the
  pre-delete revision wholesale (D1 C6: authored truth restored exactly,
  bound records rebound from restored inputs), so undo-of-delete restores
  working references. This is consistent, not contradictory: undo restores
  *state*; recreate-by-name creates *new* state.
- **3D span interaction** (with §4.2): deleting an *interior* sheet of a
  span needs no transform at all — the stored span re-expands against the
  new C3 order (membership shrinks, Excel-correct). Deleting an *endpoint*
  sheet transforms the span: the endpoint moves to the nearest surviving
  sheet that was inside the old span (Excel's behavior); a span whose two
  endpoints both die collapses to `FullyInvalid`/`#REF!`.

**Tree profile: `DormantIdentityHeal`.** Tree references already live in a
`#NAME?`-self-healing namespace (unresolved commits as `#NAME?`, heals on
symbol appearance — 34c7219c); deleted-target references keep the same
lenient contract, matching the grid's retained-edge-heals-on-recreate
defined-name precedent (`grid/machine.rs:14290`). Cross-workspace dangling
(workspace unloaded, §9) is likewise dormant-heal.

**Rationale:** one global policy would be wrong twice — Excel users need
hard `#REF!` (silently healing a deleted sheet's references on
name-coincidence is data corruption from their standpoint), tree users
already live in a heal-by-name world. The policy is exactly the kind of
fact the vocabulary layer exists to carry.

## 7. Tree-profile name precedence (Harvest 1, adopted)

The R0 Harvest 1 spec is adopted **verbatim in substance** as the
tree-profile precedence contract (anchored to DnaTreeCalc
`CORE_MODEL_SPEC §3.2/§3.4/§3.7/§3.9/§10.3`):

1. **Nearest-scope-wins.** A symbol resolves by walking up from the
   referencing node's scope; the first scope containing the name wins.
2. **Ancestor-by-own-name has no priority.** An ancestor whose own name
   matches does not outrank a name defined in a nearer scope.
3. **Within-scope collision ⇒ Ambiguous**, a typed rejection — never a
   silent pick. Delivery channel is main's existing shape:
   `ReferenceAtomBindResult::Rejected` (`tree_reference_system.rs:302`),
   with a **stable diagnostic code** minted in R3
   (`treecalc.name.ambiguous`) so corpora assert on the code, not prose.
4. **Self-reference resolves to self**, then participates in normal cycle
   handling.

**calc-uanv / fold-rule interaction (binding decisions):**

- Symbol folding uses **one shared fold function** — the same Unicode
  simple case fold D1 §2 specifies for `NormalizedSheetName` and that
  `calc-uanv` will apply to sibling uniqueness. It is exported once
  (contract V3 below) and used by sheet lookup, tree name resolution, and
  defined-name keys alike. Two folds would eventually disagree on some
  code point; one fold cannot.
- The R3 precedence corpus **must not construct ambiguity via case-only
  twins** (`Dup`/`dup`) — those become structurally invalid when
  `calc-uanv` lands. Ambiguity fixtures are built from genuinely distinct
  colliding paths (e.g. a name reachable through two sibling containers at
  equal walk-up distance).
- The old stash corpus asserted an `ambiguous_host_name` diagnostic string
  that no longer exists; the R3 corpus is authored fresh against the
  `Rejected` path (per the R0 triage verdict).

## 8. LAMBDA-valued names under the tree-node≈defined-name unification

A defined name — workbook-scoped, sheet-scoped, or a tree node acting as a
name (§4.3 rule 4) — may bind to a **callable**, not a range or scalar. The
machinery exists on main: `DefinedNameBinding::Callable` delivered through
`callable_binding_from_calc_value` (`treecalc.rs:5400-5409`), callables
carried as `RichValue::Callable` on working calc values.

**Decision — the callable dependency contract** (adopting Harvest 2's two
requirements as design invariants):

1. **Caller→captured invalidation edges are first-class graph edges.** If
   caller node/cell F invokes callable name C, and C's definition captures
   references {X…}, then editing any X must re-evaluate F **through the
   published dependency graph** — not via whole-recalc accident. Contract
   shape: the callable's bound/published value exposes its **captured
   reference set as normal-form dependency keys** (a
   `captured_dependency_keys: BTreeSet<String>` face on the published
   callable binding), so the graph layer can register edges
   `F → C` *and* `F → each X` without evaluating C's body at
   graph-construction time. Name granularity, matching the workbook graph
   join (W062 Direction 3).
2. **Transitive-capture closure.** Callable-calls-callable chains
   (C₁ captures C₂ which captures X) require the closure of captured sets:
   the exposed set on C₁ is its direct captures plus the exposed sets of
   captured callables, computed at publish time (cycle-guarded; a capture
   cycle is a typed cycle-group fact for D3/W055, not a hang).

Edge *mechanics* (where the edges live in the federated graph, seed
propagation) are D3's; D2 fixes the information contract (the exposed
captured-key set) and the semantic requirement (edit-X-reevaluates-F).

**Open verification, carried explicitly (from R0 Harvest 2):** whether
main's `callable_binding_from_calc_value` path already invalidates callers
when a captured node is edited is **unverified as of this design**. The
R3.11 bead's first act is the verification test; per the fail-until-fixed
policy, if the edge is missing the test lands failing and the defect is
D3/R4's to close (the contract above is then the fix's spec, not a new
feature). Flagged as tension T4.

Strict-excel LET/LAMBDA locals are *not* names in this sense (they are
formula-scoped, OxFml's business); only LAMBDA values *assigned to defined
names or nodes* enter this contract.

## 9. Tree-profile cross-workspace path

**Decision:** Retire the `cross_workspace_host_path_pending` stub
(`tree_reference_resolution.rs:191-193`) in R3, implementing
`Workspace!Path.To.Node` through the §2 vocabulary:

- The qualifier resolves through a **context-level workspace alias
  catalog** (the `ContainerCatalog` face over `OxCalcTreeContext`'s
  workspace map; the namespace snapshot already carries
  `workspace_availability_version` / `workspace_alias_version` for
  identity, `workspace_revision.rs:230-231`).
- The right side resolves by Harvest-1 precedence **rooted at the target
  workspace's root** (a cross-workspace reference enters at the top; there
  is no walk-up across workspace boundaries — scopes do not leak between
  documents).
- Unknown workspace alias ⇒ typed rejection (`#NAME?`-class,
  `treecalc.workspace.unknown`); a previously-resolved workspace that is
  unloaded/unavailable ⇒ `#REF!`-class dormant edge with
  `DormantIdentityHeal` (§6) — workspaces come and go in a context, and
  the tree profile's contract is lenient by charter.
- The dependency crosses via the existing
  `DependencyDescriptor.workspace_target` seam (already folded into
  dependency-shape identity, D1 §6) and the
  `SyntheticReferenceBinding.workspace_target` runtime path
  (`treecalc.rs:5385-5390`). Evaluation-side scheduling of cross-workspace
  edges is D3's.
- Sequencing: after the vocabulary layer (R3.1) and alias catalog (R3.2);
  the strict-excel external-workbook path (§5) reuses the same alias
  catalog — one catalog, two vocabularies.

## 10. Normal-form-key format stability

**Decision: the key format STAYS STABLE; the sheet component's semantics
are pinned; new shapes are additive.**

- **Format stability.** Existing shapes
  (`{profile}:cell:{workbook}:{sheet}:R{r}C{c}`, area/wholerow/wholecol/
  name/… — `grid/reference_engine/parse.rs:438` ff.) do not change.
  Existing baselines, plan caches, and dependency-key corpora survive
  untouched.
- **Sheet-component semantics pinned (clarification, not revision).** The
  `{sheet}` component is a **stable sheet identity token**
  (`SheetIdentityToken`, minted from the sheet node's `TreeNodeId` at
  registration — e.g. `sheet-node:{id}` — with the current fixture
  constants like `sheet:default` remaining valid tokens), **never the
  display name**. Today's single-sheet fixtures already pass opaque
  id-ish strings, so this pins existing practice rather than changing it.
  Payoff: **sheet rename never touches a normal-form key, a dependency
  key, or a cached plan** — rename is a pure render/catalog event (C2).
  Display names appear only in `source_text`/`render_hint`/diagnostics.
- **Additive shapes** for the new reference classes:
  - 3D: `{profile}:sheetspan:{workbook}:{start_sheet}:{end_sheet}:{rect}`
    (endpoints as identity tokens; expansion is closure-time, §4.2 — the
    key deliberately does NOT enumerate member sheets, or it would change
    on every sheet insert).
  - External: no new shape — the existing `{workbook}` component carries
    the external workspace's identity token once the alias catalog
    resolves it; pre-resolution dormant keys carry the
    normalized alias (`extbook:{normalized_alias}`).
- **Cache impact:** zero for existing keys. New shapes only ever miss cold
  (no old cache can contain them). The one *identity-adjacent* change in
  the program — revision-id changes from D1 §8.4 — is D1's, already
  ratified, and orthogonal to reference keys.

**Alternative rejected:** revising keys to carry normalized display names
(readability) — rename would then invalidate every key and cached plan
touching the sheet, converting a metadata edit into a whole-sheet rebind
storm. Rejected without hesitation.

---

## Cross-design tensions (flagged, not resolved)

- **T1 (D1 C2 vs §6).** D1 C2's wording — normalized-name identity edges
  "heal when a sheet with the same normalized name is created again" —
  reads as a blanket contract. §6 narrows it per-profile: strict-excel
  declines heal for *deleted*-sheet references (hard `#REF!`), while using
  dormant-heal for never-existed names. D1 need not change (C2 describes
  the available mechanism), but the C2 text should gain a sentence noting
  the policy is vocabulary-selected when D1 is next touched.
- **T2 (§4.2 vs D3 Direction-3 default).** The W062 plan's D3 default
  ("new dependency variants carrying target sheet identity") reads as
  materialized per-sheet edges. §4.2's `SheetSpan` variant requires D3's
  coordination layer to perform **closure-time span expansion** against C3
  order. If D3's federation design assumes fully materialized edges, the
  span variant needs an expansion hook in the closure path — interface
  friction to settle when D3 lands, not a blocker (the stored-span
  decision is D2's to make; the expansion cost argument is in §4.2).
  **RESOLVED 2026-07-04:** D3 landed the opposite shape first; owner
  arbitration ruled for this design. D3 §2.3 now adopts the stored
  `SheetSpan` edge with a derived span-interval index for closure-time
  expansion; D3 R4.12 implements it.
- **T3 (§3/§4.2 vs upstream lanes).** Binder-gating and the 3D production
  are OxFml changes. Gating is small and additive (R3.8); 3D is **OxFml
  W078** (`fml-k9s`, planned) — the W062 plan's R0 lane text folds 3D
  into "W077 execution", but W077 is closed; the register should point at
  W078 as its own fml-lane entry gating R3.9 only.
- **T4 (§8 vs D3). RESOLVED 2026-07-04:** D3 §0 ran the verification by
  live probe — the direct case invalidates correctly; the transitive
  callable case is a live evaluation-transport defect (whole recalc
  transaction rejects). Test + fix consolidate in D3 R4.1/R4.9 (R3.11 is
  discharged, see the bead list); V6's mechanism is superseded (see V6
  errata). The fail-until-fixed policy applies in R4.9 as specified in
  D3 §0.
- **T5 (§5 vs D4 ingest tiers). RESOLVED 2026-07-04 (owner
  reconciliation, see §5 errata):** ingest-cached external values are
  consumed after all — published with `FileCached` provenance through
  the ordinary channel, pinned until explicit refresh or sibling-load,
  then §5 routing/`#REF!` applies (D4 §14/T8 records the same contract).
  Original text kept for the record: the no-external-value-cache
  decision meant D4's `DocumentEvent::ExternalLink` tier must **ledger
  cached external values as not-consumed** (never silently dropped, per
  Direction 6) and map link declarations to §5's dormant alias references.
  D4's tier table should cite §5 explicitly.
- **T6 (§1/§7 vs D1 R2.2 sequencing).**
  `TreeCalcContextReferenceBindProfile::new` takes
  `meta_node_ids: &BTreeSet<TreeNodeId>` (`tree_reference_system.rs:56`);
  D1's R2.2 retires that side set in favor of structural `is_meta`. R3
  beads touching the tree profile (R3.1, R3.5, R3.6) must land after
  R2.2 or carry a trivial re-plumb — sequencing fact, not friction.
- **T7 (§4.3 vs D4 verb ownership).** D2 claims the minimal `define_name`
  verb for R3 testability while Direction 5 assigns defined-name seeding
  to the document surface (D4/R5). The split intended here: R3 lands the
  engine-visible verb + resolution semantics; D4/R5 wraps it in the
  document-surface API (scoping enumeration, readout, OxDoc round-trip).
  D4 should consume, not re-design, the §4.3 precedence order.

## Open questions answered

Every D2 question from `W062_IDEAL_ENGINE_MODEL_REWORK.md` §Open questions
for R1:

| D2 question | Answer |
| --- | --- |
| Vocabulary placement (trait method vs sibling trait vs config) | OxCalc-side sibling trait `StructuralVocabulary`, attached via subtrait `OxCalcReferenceProfile: ReferenceBindProfile`; OxFml trait method rejected on the seam guardrail (GridBounds precedent), config struct rejected as behavior-free, side registry rejected as a drift source (§1). |
| Grammar-gating vs resolution-only | Capabilities become real **binder-routing gates** in OxFml's bind lifecycle; grammar/lexer stays single and shared; one additive flag (`sheet_span_3d_references`) rides the 3D production (§3). |
| 3D grammar design (lexer token vs post-parse reinterpretation) | Neither: a parser production (`SheetSpan3DReferenceExpr`) with bounded 3-token lookahead, delivered by **OxFml W078** (`fml-k9s`) with an additive default-`Unsupported` `bind_sheet_span` trait method; spans stay distinct from same-sheet multi-area per the OxFunc boundary (§4.2). |
| 3D dependency shape (per-sheet fan vs one edge) | One stored `GridDependency::SheetSpan` edge; per-sheet fan computed at closure time against C3 sheet order; materialized fan rejected (rewrite-on-every-lifecycle-edit) (§4.2). |
| Tree-profile `!`/cross-workspace path | Implemented in R3 through the container-qualification vocabulary: workspace alias catalog, resolution rooted at target workspace root, typed rejection for unknown alias, dormant-heal for unloaded; stub retired (§2, §9). |
| Normal-form-key format stability vs revision | Stay stable. Sheet component semantics pinned to a stable identity token (rename never touches keys/caches); additive shapes for 3D and dormant-external only; zero cache impact for existing keys (§10). |
| External workbook references — in/out | Typed partial-IN: bind + identity + routing-to-loaded-workspace in R3 (external workbook = sibling workspace, per D1 C8); OUT: evaluation-triggered loading, cached external values (unloaded ⇒ `#REF!`, typed), link management (§5). |
| Sheet-deletion policy for dangling sheet-qualified references | Vocabulary-carried `ContainerDeletionPolicy`: strict-excel `HardRefError` (Excel-faithful, no heal-on-recreate; undo heals via revision restore; 3D endpoint-shrink transforms), tree profile `DormantIdentityHeal`; consumes D1's `DeletedSheetFact` (§6). |
| LAMBDA-valued names under tree-node≈defined-name unification | Names may bind `Callable`; published callable exposes its transitive captured-reference set as normal-form dependency keys; caller→captured edges are first-class at name granularity (Harvest 2 adopted); main's-behavior verification carried to R3.11 as a possibly-fail-until-fixed test (§8). |

**Deferrals (explicit):** the 3D grammar's internal design (token
sequences, error recovery, span shapes) is deferred to OxFml W078
(`fml-k9s`) — this document fixes its direction (bounded-lookahead production +
capability flag + additive bind method) and its OxCalc-side consumption
contract only. Cross-sheet/cross-workspace *edge scheduling and dirty-seed
mechanics* are deferred to D3 by design (this document hands D3 the
descriptor and expansion contracts V5–V7). No other D2 question is
deferred.

## Contracts exported (V-series, for R3/D3/D4)

- **V1 (all):** `OxCalcReferenceProfile: ReferenceBindProfile` with
  `vocabulary() -> &dyn StructuralVocabulary` is the OxCalc-internal
  profile handle; the OxFml seam receives the upcast `&dyn
  ReferenceBindProfile` only. Vocabulary types never cross into OxFml.
- **V2 (all):** `!` = container-role qualification: left side resolves
  through `resolve_container_qualifier` against the profile's admitted
  container roles; right side resolves inside the resolved container's
  namespace. Profiles differ only in admitted roles.
- **V3 (R3/D1/calc-uanv):** one shared symbol-fold function (Unicode
  simple case fold) serves sheet lookup (D1 C1), tree name resolution
  (§7), and defined-name keys. Exported once from oxcalc-core; no second
  fold may be written.
- **V4 (D3/D4):** `SheetIdentityToken` is the stable, rename-immune sheet
  component of every normal-form/dependency key; display names never enter
  keys. Rename is a render/catalog event with zero graph impact.
- **V5 (D3):** the catalog router yields cross-sheet dependency
  descriptors `{ target_sheet_node: TreeNodeId, dependency:
  GridDependency }`; `GridDependency::SheetSpan` members are **not**
  materialized — D3's closure computation expands spans against C3 order
  at closure time.
- **V6 (D3): MECHANISM SUPERSEDED (errata 2026-07-04).** As written, V6
  prescribed `captured_dependency_keys` exposure + caller→captured graph
  edges. D3's live probes (D3 §0) verified invalidation already composes
  correctly through the defining node — the real defect is evaluation
  transport (callables lack captured environments), fixed per D3 §8.3.
  Owner arbitration: the *semantic invariant* (edit-captured ⇒
  re-evaluate caller, oracle-first) stands and is guarded by D3
  R4.1/R4.9's tests, which also discharge R3.11's verification act;
  the key-exposure mechanism is dropped as an invalidation dependency
  (may return as diagnostics only).
- **V7 (D3/D4):** deletion transforms: strict-excel sheet deletion drives
  `transform_reference(StructuralEdit: SheetDeleted)` producing
  `FullyInvalid`/`RefError` records (points/ranges) and endpoint-shrink
  (3D spans); tree/workspace deletion produces dormant identity edges.
  `DeletedSheetFact` (D1 C2) is the information source; D4's readout
  renders `#REF!` from the record, never from ad-hoc state.
- **V8 (D4):** the §4.3 name-resolution precedence (local table column →
  sheet scope → workbook scope → root tree-node names → `#NAME?`) is
  fixed; D4's seeding/readout verbs implement scoping against it, and
  definition-time collisions (seeded name vs root node symbol) are typed
  rejections.
- **V9 (upstream):** OxFml receives only: capability-flag consultation in
  binder routing (existing flags + `sheet_span_3d_references`), the 3D
  production, and the additive `bind_sheet_span` default method. No model
  types, no bounds, no vocabulary.

## R3 bead breakdown (ordered)

Sizes: S ≲ half day, M ≈ 1 day, L ≈ 2 days. Every bead lands green on
`main` (`cargo test -p oxcalc-core`, differential clean) with fresh-eyes
review instructions per the W062 execution process. Tree-profile beads
sequence after D1's R2.2 (tension T6); catalog beads after R2.3/R2.4.

1. **R3.1 — Vocabulary layer: traits + both profile impls (M).**
   `StructuralVocabulary`, `OxCalcReferenceProfile`,
   `ContainerDeletionPolicy`, `ContainerResolution`; implement for the
   strict and tree profiles (strict: sheet-qualifier reclassification with
   zero single-sheet behavior change; tree: vocabulary admitting Workspace
   containers, resolution still stubbed until R3.6). Export the shared
   fold (V3), coordinating with `calc-uanv`. Acceptance: existing profile
   tests green unchanged; vocabulary unit tests for role terms and
   qualifier classification.
2. **R3.2 — Sheet identity tokens + `WorkbookReferenceCatalog` (M).**
   `SheetIdentityToken` minting at sheet registration; catalog over D1's
   `sheet_index`/C3 order; context-level workspace alias catalog (shared
   seat for §5/§9); dormant sheet-identity edges for unknown names.
   Acceptance: rename changes no keys and no edges (property test);
   catalog routes name→node→engine across a 3-sheet workbook fixture.
3. **R3.3 — Strict-excel cross-sheet resolution end-to-end (L).**
   Router consulted at dependency extraction/instantiation; cross-sheet
   descriptors (V5) handed to the D3 seam (behind a narrow interface if
   D3's coordinator hasn't landed: descriptor emission + a
   mark-all-fallback oracle path so cross-sheet values are correct
   immediately). Acceptance: `Sheet2!A1` from Sheet1 evaluates correctly
   and re-evaluates when Sheet2!A1 changes; oracle/differential clean.
4. **R3.4 — Sheet-deletion policy transforms (M).** `delete_sheet` drives
   V7 transforms per vocabulary policy: strict `FullyInvalid`/`#REF!`
   (no heal-on-recreate — explicit negative test), 3D endpoint-shrink seat
   (activates with R3.9), undo-restores-references test via revision
   navigation. Consumes `DeletedSheetFact`. Acceptance: Excel-fidelity
   fixture — delete, observe `#REF!` text; recreate same-name, still
   `#REF!`; undo, working again.
5. **R3.5 — Consumer-scoped defined names + tree-node unification (M).**
   Minimal `define_name(scope, …)` verb with typed collision rejection;
   §4.3 precedence wired for strict cells (sheet → workbook → root-node
   names); resolution corpus. Acceptance: shadowing fixtures per V8;
   collision at definition time is a typed error.
6. **R3.6 — Tree-profile precedence corpus + cross-workspace `!` (L).**
   Retire `cross_workspace_host_path_pending`; workspace-alias resolution
   rooted at target root; `treecalc.name.ambiguous` /
   `treecalc.workspace.unknown` diagnostic codes; fresh Harvest-1 corpus
   (rules 1–4, no case-only-twin constructions). Acceptance: corpus green
   against the `Rejected` path; cross-workspace value flows through the
   `workspace_target` seam.
7. **R3.7 — External workbook references (M).** Replace the external
   punts in the strict profile with real binds
   (`DynamicOrHostSensitive`); alias-catalog routing to loaded sibling
   workspaces; unloaded ⇒ `#REF!` + typed diagnostic + heal-on-load.
   Typed exclusions (no loading, no value cache) recorded in the bead.
   Acceptance: two-workbook context fixture; `[Book2]Sheet1!A1` live when
   Book2 loaded, `#REF!` when absent, heals on load.
8. **R3.8 — [U] OxFml binder capability gating (S, upstream lane).**
   Consult `ReferenceSyntaxCapabilities` at binder routing (§3);
   default-profile guardrail fixtures prove `worksheet_legacy()` behavior
   bit-identical. Acceptance: a `structured_references: false` profile
   receives no structured-reference routings; OxFml suite green.
9. **R3.9 — [U-gated] 3D references (L, gated on OxFml W078 /
   `fml-k9s`).** OxCalc side: `GridDependency::SheetSpan`, sheetspan
   normal-form shape, closure-time expansion hook (V5), endpoint
   delete/shrink transforms (activating R3.4's seat), `SUM(Sheet1:
   Sheet3!A1)` fixtures incl. insert-inside-span membership change.
   Starts only when W078 lands the production +
   `sheet_span_3d_references` + `bind_sheet_span`.
10. **R3.10 — W077 template-identity adoption: plan sharing (M).**
    Template-identity-keyed compiled-plan reuse for region-stamped
    formulas; per-cell placed-identity instantiation; the D4 shared-formula
    ingest handshake contract documented on the bead. Acceptance: N
    placements of one R1C1 template compile once (counter evidence);
    A1 `$`-fidelity behavior unchanged.
11. **R3.11 — LAMBDA-valued-name dependency contract (M) — DISCHARGED
    (errata 2026-07-04): D3 §0's live probes performed the verification
    (direct case works; transitive case is a live evaluation-transport
    defect) and D3 R4.1/R4.9 own the test + fix. R3.11 is not created as
    an R3 bead.** Original scope kept for the record: first act was the
    Harvest-2 verification test
    (edit captured node ⇒ caller re-evaluates) against main's callable
    path. If green: wire `captured_dependency_keys` exposure (V6) +
    transitive-closure tests. If red: the test **stays red** (policy),
    the defect is recorded for D3/R4 with V6 as its spec, and the
    exposure contract still lands. Acceptance: capture and
    transitive-capture fixtures; cycle-guarded closure.

Sequencing: R3.1 → R3.2 → {R3.3, R3.4, R3.5} (parallel-safe); R3.6 and
R3.7 after R3.2 (shared alias catalog); R3.8 independent, any time; R3.9
last, on its upstream gate; R3.10 and R3.11 independent of the catalog
chain (R3.10 after R3.1 only; R3.11 after R3.1).

## W077 gate mapping

The W062 plan's named entry gate — "generic BindProfile ABI landed +
`ExcludeCallerAnchorForTemplate` usable by a shipped profile" — is
**formally MET**: W077 converged and CLOSED in OxFml (four beads, suite
green; shape frozen `965cec0`, acceptance recorded `3dd605c`, coverage
`b16cb6c`; tree profiles ship the policy; strict profile consumes the seam
end-to-end). The 3D grammar (now W078) was always allowed to trail R3's
start and gates only its own slice.

| R3 bead | OxFml surface consumed | Gate state |
| --- | --- | --- |
| R3.1–R3.7, R3.10, R3.11 | As-built frozen ABI only: `ReferenceBindProfile`, `ProfileReferenceRecord`, `ReferenceTransform*`, `parsed_qualifier`/`external_target_id` facts, `FormulaTemplateIdentity`/`PlacedFormulaIdentity`, R1C1 caller-independent normal form | **OPEN** — zero new upstream landings required |
| R3.8 | New: binder consultation of existing `ReferenceSyntaxCapabilities` flags | **OPEN to start** — R3.8 *is* the upstream landing (small, additive, default-preserving) |
| R3.9 | OxFml **W078** (`fml-k9s`, planned): `SheetSpan3DReferenceExpr` production, `sheet_span_3d_references` flag, additive `bind_sheet_span` default method | **GATED on W078** — R3.9 starts when it lands; nothing else in R3 waits on it |

## Seam audit (guardrail closure)

Everything that moved, and where it lives: vocabulary traits, catalogs,
identity tokens, deletion policies, precedence rules, captured-key
contracts — **all OxCalc**. OxFml receives exactly V9: flag consultation,
one grammar production behind a default-off flag, one default-`Unsupported`
trait method. No grid bounds, no roles, no sheet registries, no `#REF!`
semantics cross upstream. The GridBounds ratification line holds.
