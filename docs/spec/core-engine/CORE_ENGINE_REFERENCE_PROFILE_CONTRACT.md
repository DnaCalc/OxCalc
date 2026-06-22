# CORE_ENGINE_REFERENCE_PROFILE_CONTRACT

Status: **OxCalc-local contract target** for W061/W077 alignment. This document records the
OxCalc requirements for the generic OxFml reference-profile seam. It is not the canonical
OxFml ABI; if OxFml names differ, the OxFml contract owns those names. The architectural split
below is binding for OxCalc's grid work.

Current planning decision: **generalize reference binding into OxFml, not grid behavior**.
OxFml hosts the reference lifecycle and profile dispatch. OxCalc's grid profile answers grid
questions. DnaTreeCalc remains the canary that the lifecycle is not accidentally grid-shaped.

## 1. Role Split

OxFml owns formula syntax, binding lifecycle, reference-expression slots, profile dispatch,
source span/text preservation, normal-form key envelopes, dependency/result envelopes, render
envelopes, and the lifecycle hooks that let a profile participate in those operations.

OxFml does not own grid address semantics. It must not own grid storage, grid dependency
closure, spill arbitration, hidden-row context, grid dereference, row/column edit semantics, or
Excel-specific bounds. Those belong to OxCalc's `strict-excel-grid` profile and runtime
providers.

DnaTreeCalc remains the non-grid proof case. If the reference-profile seam cannot carry tree
references cleanly, the seam is too grid-shaped.

## 2. Reference Lifecycle

The generic lifecycle is:

```text
source formula text
  -> parsed formula with unbound reference expressions
  -> bind using ReferenceProfile + BindContext
  -> bound formula with profile-owned reference payloads
  -> normal-form identity / dependency emission / evaluation prep / edit transform / rendering
```

OxFml core may see a profile id, source span, surface text, and opaque/profile-owned payload.
It should not see a `GridRow`, `GridColumn`, semantic `SheetId`, Excel bounds, A1 conversion
rules, structured-reference rules, or `#SPILL!` mechanics as core types.

## 3. Required Capabilities

The seam should be capability-shaped rather than one large mandatory trait:

1. **Syntax recognition:** profile-gated recognition of host references in formula grammar.
   OxFml owns generic formula parsing and reference operators; the profile owns whether a
   token is an A1/R1C1/table/name/tree reference and what it means.
2. **Binding:** bind unbound references with a context containing profile id/version, formula
   anchor identity, formula source hash, structure version, namespace version, dialect options,
   and host identity. The result preserves both bound target identity and the relative or
   symbolic descriptor used for identity.
3. **Normal form:** profile-owned equivalence key for formula templates, compiled-plan cache
   identity, shared-formula grouping, copy/fill equivalence, and bind hashing. For grid this is
   R1C1-like and caller-relative; for tree it is tree-relative or symbolic.
4. **Dependency emission:** generic dependency envelopes. Grid dependencies may represent
   cells, finite ranges, whole rows/columns, names, table regions, spill anchors/extents,
   dynamic host queries, external references, or unknown/host-sensitive requests. OxFml must
   not scalarize grid ranges.
5. **Evaluation argument preparation:** function arguments can request values, arrays,
   preserved references, or reference/value hybrids. OxFunc function metadata and host
   providers decide dereference/coercion; the evaluator must not blindly dereference every
   reference expression.
6. **Structural edit transform:** OxFml hosts the lifecycle
   `bound_formula + host_structural_edit + old/new anchor context -> transform result`, while
   the profile owns the coordinate system and edit semantics. Grid row/column insert/delete,
   sheet/name/table edits, deletion holes, and `#REF!` generation are OxCalc responsibilities.
7. **Rendering:** rendering is separate from normal form. Unchanged formulas preserve source
   text where possible; changed formulas ask the profile for preferred source rendering. For
   `.xlsx`, the preferred grid render is A1-style unless a caller explicitly requests another
   channel.

## 4. Public ABI Packet Sketch

W077 owns the canonical names, but the public shape must be at least this expressive:

- `ReferenceProfileId` and `ProfileVersion`: stable profile identity and versioning for bind
  compatibility.
- `BindContext`: formula anchor identity, source hash, structure version, namespace version,
  host identity, dialect options, and bind options. Ordinary cell values must not enter the
  bind key.
- `UnboundReferenceExpr`: source span, surface text, profile candidate, and parser payload.
- `BoundReferenceExpr`: profile id/version plus a profile-owned payload. Invalid references
  such as grid `#REF!` are first-class bound reference payloads, not parse failures.
- `ReferenceNormalFormKey`: profile-owned identity/equivalence key used for shared-formula
  grouping, template identity, copy/fill equivalence, bind hashes, and compiled-plan cache
  keys.
- `ReferenceDependencySet`: generic dependency envelope; host profiles decide whether entries
  are cells, ranges, whole axes, names, table regions, tree nodes, subtrees, dynamic queries, or
  unknown/host-sensitive requests.
- `ReferenceArgumentPreparation`: value, array, preserved-reference, or reference/value-hybrid
  request used by OxFml/OxFunc before function invocation.
- `ReferenceTransformRequest` / `ReferenceTransformResult`: lifecycle envelope for host
  structural edits. The profile owns the coordinate system, deletion holes, expansion/shrink
  rules, and invalid-reference outcomes.
- `ReferenceRenderRequest` / `ReferenceRenderResult`: source-channel rendering separate from
  normal-form identity, with unchanged-source preservation as the default policy.

Before OxCalc broadens grid behavior beyond the current executable floor, this packet shape or
an equivalent OxFml W077 shape must be acknowledged, and the seam must be exercised by a
non-grid DnaTreeCalc profile plus a tiny fake-profile test. Otherwise new grid behavior stays
OxCalc-local evidence rather than shared ABI proof.

## 5. Grid Profile Obligations

The first `strict-excel-grid` profile lives in OxCalc and supplies:

1. bounded grid context: workbook, sheet, anchor row/column, bounds, name/table namespace
   versions, and dialect options;
2. A1 and R1C1 point/range recognition, `$` fidelity, sheet qualification, whole-row/column
   forms, structured references, spill suffixes, union/intersection, and `#REF!` as the slice
   admits them;
3. caller-independent grid normal-form identity under `strict-excel-grid`;
4. cell/range/name/table/spill/dynamic dependency emission without OxFml scalarization;
5. runtime behavior through OxFunc's existing `ReferenceSystemProvider` implementation
   (`GridReferenceSystemProvider`) for dereference, enumeration, facts, and transform/compose
   requests;
6. hidden-sensitive aggregate context through `GridHostInfoProvider` over `AxisState`;
7. edit-transform algebra for fill, paste, row/column insert/delete, deletion holes, and
   `#REF!` outcomes.

Current OxCalc floor: the grid profile exposes `transform_reference` for an
`excel-grid-structural-edit.v1` payload. The payload carries workbook/sheet row-or-column
insert/delete plus formula-anchor-before/after context. The implemented transform covers point
references, finite area references, whole-row references, and whole-column references, including
deleted-target `#REF!` payloads and R1C1-relative shape preservation. Spill anchors, names, and
structured references intentionally return host-sensitive structural-transform outcomes until
their ledger/namespace edit owners are wired. At runtime, the first same-sheet defined-name
value/invalidation floor is wired through OxCalc's `GridReferenceSystemProvider`: OxFml binds a
name as a symbolic `excel.grid.v1` payload, while OxCalc resolves only names present in the
provider namespace, including `INDIRECT("Name")` text resolution. OxCalc's
GridInvalidation-Ref owns finite `Name(name, extent)` dirty closure and row/column edit
transforms for same-sheet name extents; namespace versioning and rename/delete semantics remain
host-owned follow-up work.
The current reference-grid consumer (`GridCalcRefSheet::apply_axis_edit`) applies those profile
records back to authored formula source spans for moved formulas and then rebinds at the new
anchor. This keeps OxFml in the lifecycle role while OxCalc owns the grid edit semantics and
source-render choice.

## 6. Tree Profile Obligations

DnaTreeCalc's profile adaptation proves that the seam remains non-grid:

1. existing relative, global, and tree-local references bind through the generic lifecycle;
2. tree dependencies emit through the generic dependency envelope;
3. invalid tree references remain first-class bound reference errors, not parse failures;
4. bind hashes change for structure/namespace version changes and do not change for ordinary
   value changes;
5. tree structural transforms either implement their semantics or return an explicit
   unsupported/host-sensitive transform result.

## 7. Near-Term Sequence

Current reality: the OxCalc grid floor already implements more than the minimal same-sheet A1
profile, including optimized storage, defined names, table overlays, spill facts, visibility,
and a first row/column structural-edit slice. Treat that as OxCalc-owned evidence for the
profile boundary, not as permission to push grid semantics into OxFml.

The next implementation plan is:

1. Freeze the W077-compatible ABI packet shape above, including default-preserving
   `BindProfile`, symbolic bound-reference records, source text preservation, normal-form
   identity, dependency envelopes, argument-preparation hooks, edit-transform envelopes, and
   render envelopes.
2. Prove the seam with DnaTreeCalc tree references and a tiny fake OxFml test profile. This is
   the guard against a grid-shaped abstraction.
3. Stabilize the existing OxCalc grid profile under that ABI: same-sheet A1/R1C1, mixed `$`
   fidelity, first-class `#REF!`, names, table overlays, spill-anchor facts, hidden-row host
   context, whole-axis references, and the current structural-edit slice remain profile-owned.
4. Keep value correctness and invalidation correctness separate: `GridCalc-Ref` is the
   mark-all-dirty value/effects oracle; `GridInvalidation-Ref` is the scalar dirty-closure
   oracle.
5. Expand structural edits only through the edit-transform algebra, including deletion holes,
   namespace/table/name effects, spill anchors/extents, feature-rendered regions, and explicit
   unsupported/host-sensitive outcomes.
6. Render transformed formulas only after bind/transform semantics are correct, preserving
   unchanged source text and preferring A1 for `.xlsx` write-back.

## 8. Non-Goals For OxFml Core

OxFml core must not answer "what is A1?", "what row shifts when row 3 is deleted?", "what is
Excel's maximum column?", or "how does `A1#` dereference?". OxFml may ask the active profile
for reference recognition, binding, normal form, dependencies, argument preparation,
transform results, and rendering.
