# Core Engine Skin IR And OxCalcTree Boundary Guide

## 1. Purpose

This document defines the functional boundary between a downstream host skin
model, such as DNA TreeCalc's Skin IR, and the OxCalc-owned `OxCalcTree`
runtime contract.

It is a work guide for preventing drift between:

1. a host-facing command/render layer that skins can consume, and
2. the engine-owned semantic state that gives those commands their meaning.

The short rule is:

> OxCalc owns semantic truth. The host Skin IR owns typed user requests and
> renderable projections.

That means a skin or host command surface may name an action such as
`AddNode`, `RenameNode`, or `EditContent`, but OxCalc must own the meaning of
that action: structure mutation, stable node identity, revision advancement,
dependency invalidation, formula rebind impact, candidate/publication behavior,
and retained-version navigation.

## 2. High-Level Intent

DNA TreeCalc needs a strong intermediate representation so different skins can
drive and view the same calculation model without depending on OxCalc internals.
That IR should be real and typed, not a string fallback layer.

At the same time, the IR must not become a second calculation engine. Any
behavior that changes calculation meaning, structural identity, dependency
meaning, or published values belongs in OxCalc.

The intended stack is:

```text
Skin gesture
  -> Skin IR intent
  -> host command transaction
  -> OxCalcTree typed operation
  -> OxCalc workspace revision / candidate / publication
  -> host projection
  -> Skin IR WorkspaceState
  -> skin render
```

The command can be host-visible and undo-label-friendly. The semantic result is
OxCalc-owned.

## 3. Ownership Definition

| Surface | OxCalc ownership | Host / Skin IR ownership | Current status | To-do / next guide |
|---|---|---|---|---|
| Canonical tree structure | Owns `StructureSnapshot`, node ids, parent/child order, symbols, table shape, structural paths, and successor snapshots. | May present paths, labels, visible rows, sort/filter views, and skin layout, but those are projections or facade state. | Done in core substrate: immutable structural snapshots and direct-context structural edit APIs exist. | Strengthen stable host-visible identity across rename/move and expose enough revision metadata for host command history. |
| Authored node input | Owns `NodeInputSnapshot`, formula text, literal/non-formula input, input epochs, and formula/literal transitions. | Sends typed content edits and displays editor text/drafts. Live drafts may ask OxFml for diagnostics without publishing. | Done for direct context formula/literal edit paths; exposed through `set_node_formula_text`, `set_node_input_value`, and `clear_node_input_value`. | Add richer editor-facing diagnostics/preview packets without making host drafts durable engine truth. |
| Namespace/capability context | Owns engine-visible namespace, capability, workspace availability, table context, and caller-context facts that affect binding/prepared identity. | Chooses product policy/profile and supplies host facts through declared options or context packets. | Partly done through `OxCalcTreeContextOptions`, table context packets, and W051/W056 reference lanes. | Widen typed namespace mutation/version APIs and dependency invalidation facts for profile/host-policy changes. |
| Formula parse/bind/eval meaning | Consumes OxFml facts and runtime facade outputs; owns coordination around those facts. | Does not parse formulas for semantic meaning. May provide editor UI and pass formula text/context. | Done as boundary doctrine; implementation has OxFml-backed bind/prep paths for current TreeCalc slices. | Continue W056/W060 reference-family widening; keep TreeCalc syntax support in OxFml/OxCalc, not in skins. |
| User command/action names | Does not own skin gesture vocabulary, menu labels, or UI grouping names. | Owns typed requests such as `AddNode`, `RenameNode`, `MoveNode`, `DeleteNode`, `EditContent`, `Recalculate`, `SelectNode`. | DnaTreeCalc has begun this as Skin IR intents; OxCalc consumer contract already has corresponding typed operations. | Keep Skin IR action names as requests only; route all calculation-affecting requests to OxCalcTree operations. |
| Structural edit semantics | Owns legality, application, successor revision, affected nodes, invalidation consequences, and compatible retention. | Supplies requested parent/symbol/index/target and chooses UI prompts. | Done for current add/rename/move/reorder/delete direct-context APIs. | Add explicit legality/preview APIs for collision, affected references, rebind impact, and invalidation summary before commit. |
| Recalculation and publication | Owns dependency graph, invalidation closure, scheduling, candidate, accept/reject, publication snapshot, runtime overlays, diagnostics, and stable published values. | Decides when to request recalc and how to display stale/pending/rejected states. | Done for synchronous sequential `recalculate` returning `OxCalcTreeCalculationOutcome`. | Add step/progress/cancellation APIs without introducing ambient engine progress outside host calls. |
| Renderable workspace projection | Owns authoritative `workspace_view`, `node_view`, table views, snapshot ids, values, diagnostics, and calc states. | Projects OxCalc views into skin-friendly `WorkspaceState` / row/card/table models. | Done at first OxCalcTree view level; DnaTreeCalc now projects richer Skin IR from OxCalc views/outcomes. | Enrich typed diagnostics, value shapes, dependency drill, and table render models while preserving OxCalc as source. |
| Dependency graph and drill | Owns descriptors, reverse edges, dependency diagnostics, collection membership/order facts, dynamic potential/rebind facts, and cycle groups. | Displays counts, wires, drill panels, and explanations derived from engine packets. | Done for current dependency graph and W051/W056 carrier/dependency patterns; DnaTreeCalc projects counts/summaries. | Add stable host-facing drill/explain packet instead of making skins interpret internal descriptors. |
| Table identity and lifecycle | Owns table snapshot normalization, table projection, row/column/header/totals identity, table context packets, table dependency inventory, and dynamic table rebind classification. | Owns product editing UI and table display choices; supplies table content/snapshots. | Done for W056 table snapshot/projection and direct-context table APIs. | Add host-facing table mutation conveniences and richer table render projections; keep dependency/lifecycle facts in OxCalc. |
| Metadata, formats, templates, skin state | Owns only engine-visible facts when they affect binding, dependencies, publication, or table/reference identity. | Owns selection, collapse, pins, skin layout, format/template product services, and meta-node persistence when calc-ignored. | Partly done in host docs/skin framework; OxCalc supports meta-node invisibility and engine snapshot persistence for engine-owned state. | Keep literal format/template edits host-level unless they emit regular structural/input edits or a declared engine-visible namespace change. |
| Save/reopen | Owns export/import of OxCalc-owned workspace snapshot: structure, input, namespace, derived/publication/runtime/table state. | Owns product document wrapper, skin state, selection, layout, command history metadata, unknown host payloads. | Done for current `export_workspace_snapshot` / `import_workspace_snapshot`. | Add version-navigation history and command metadata policy without mixing host facade data into OxCalc snapshots. |
| Undo/redo | Owns retained revision graph, revision navigation, and publication/runtime compatibility. | Owns command stack labels, grouping, selection restoration, and non-engine facade-state restoration. | Substrate done: revisions/snapshots exist. Consumer contract names undo/redo as successor version-navigation operation. | Implement edit transaction ids and `navigate_workspace_revision` or equivalent. Do not implement undo by replaying host-forged inverse `WorkspaceIntent`s. |
| Command transactions | Should return or expose predecessor revision id, successor revision id, invalidation summary, and typed failure. | Groups one or more Skin IR intents into user-facing commands and undo labels. | To-do. Current direct APIs apply individual operations and views expose revision ids. | Add `apply_edit_batch` or transaction envelope over structural/input/table edits. |
| External updates / RTD-like inputs | Owns invalidation and publication effects once an external event is admitted as a typed engine-visible input. | Receives product/external events and submits them as typed updates; renders pending/stale status. | To-do for OxCalcTree consumer API. | Add explicit context operation for external updates; no callbacks into host and no hidden background mutation. |
| Async, progress, cancellation | Owns candidate/progress token semantics, cancellation effect, no-partial-publication rule, and stable prior publication reads. | Decides UI budgets and calls step/cancel operations. | To-do; passive host-driven direction is specified. | Add step/bounded-progress and cancellation APIs under the passive boundary. |
| Action availability | Owns semantic availability: can move, can rename, collision, unresolved impact, table lifecycle legality. | Owns disabled/enabled UI affordances and prompts. | To-do. | Add preview/legality APIs so skins do not discover failures only by committing or by reimplementing rules. |
| Replay/audit | Owns replay-visible engine facts: revisions, dependency facts, invalidation, publication, rejection, table/reference facts. | Owns product command history and UI audit presentation. | Partly done through TraceCalc/replay surfaces and OxCalcTree outcomes. | Align command transaction ids and Skin IR request ids with replay projection without making replay depend on UI skins. |

## 4. Undo/Redo Boundary

Undo and redo must be revision navigation over OxCalc-owned state, not inverse
Skin IR replay.

Skin IR actions are useful and should be reified, but only as request and
command-history objects:

```text
WorkspaceIntent::AddNode
  -> host command transaction
  -> OxCalc structural/input/table operation
  -> predecessor WorkspaceRevisionId
  -> successor WorkspaceRevisionId
```

The host may record:

1. command id,
2. user-facing label,
3. original Skin IR intent or command group,
4. predecessor/successor OxCalc revision ids,
5. selection and facade-state before/after,
6. non-engine host side effects.

OxCalc must own:

1. the actual workspace revision graph,
2. retained publication/runtime compatibility,
3. whether a revision is still available under retention policy,
4. restoration or navigation to the requested revision,
5. typed failure when a revision has been evicted or is incompatible.

Replaying inverse host commands is not acceptable as the semantic undo basis
because inverse commands can drift:

1. rename propagation rules can change,
2. references can rebind differently after later edits,
3. template sync can expand into multiple edits,
4. table lifecycle facts can be released or reallocated,
5. dependency and publication state may need exact rollback, not approximation.

## 5. Work Sequence

Use this guide as the work order for hardening the boundary:

| Step | Product outcome | OxCalc work | Host / Skin IR work | Status |
|---|---|---|---|---|
| 1 | Skins can drive tree/content edits without direct engine access. | Keep direct `OxCalcTreeContext` structural/input APIs as authoritative. | Route Skin IR `AddNode` / `EditContent` / `RenameNode` / `MoveNode` / `ReorderNode` / `DeleteNode` to those APIs. | In progress downstream; OxCalc APIs exist for current scope. |
| 2 | Skins can render current engine state. | Expose stable `workspace_view`, `node_view`, table views, outcomes, diagnostics, and revision ids. | Project those into typed Skin IR `WorkspaceState`. | In progress downstream; OxCalc view surface exists for current scope. |
| 3 | Skins can show dependency and table facts without interpreting internals. | Add stable drill/explain packets and richer typed table render/dependency packets. | Render counts, wires, table summaries, and drill UI from those packets. | To-do beyond current summaries. |
| 4 | UI can preview legality and impact before committing edits. | Add structural/input/table legality and impact-preview APIs. | Use previews for disabled states, prompts, and affected-reference UI. | To-do. |
| 5 | Undo/redo is exact and engine-owned. | Add edit transaction ids, predecessor/successor revision ids, and version-navigation API. | Store command groups, labels, selection/facade restoration, and request revision navigation. | To-do; substrate exists. |
| 6 | Long-running work remains host-driven and cancellable. | Add step/progress/cancellation APIs with no ambient scheduler. | Call under UI budget and display progress/cancel states. | To-do. |
| 7 | Save/reopen and replay preserve the split. | Keep OxCalc snapshot export/import engine-owned; add transaction/revision replay facts. | Store host document wrapper, skins, layout, selection, and command metadata separately. | Partly done; transaction/revision history to-do. |

## 6. Non-Drift Rules

1. If a behavior changes calculation meaning, dependency meaning, structural
   identity, table/reference identity, or published values, it belongs in
   OxCalc.
2. If a behavior changes how the user sees, selects, arranges, filters, pins,
   or commands the model, it belongs in the host/Skin IR.
3. If a host action affects OxCalc, Skin IR names the request, the host routes
   it, and OxCalc owns the semantic result.
4. A host mirror of nodes, values, dependencies, diagnostics, or table facts is
   a render cache. If it disagrees with an OxCalc view for engine-visible
   facts, the OxCalc view wins.
5. Host command history may store Skin IR actions, but undo/redo must navigate
   retained OxCalc revisions.
6. Format/template/meta edits remain host-level only while calc-ignored. Once
   they affect binding, dependencies, namespace facts, or publication, they
   need an OxCalc-owned typed operation or namespace input.
7. Tests for product host behavior should prefer driving Skin IR intents and
   reading Skin IR projections, but the expected semantic outcomes should be
   derived from OxCalc-owned views/outcomes.

## 7. Current Product Status

Product status: OxCalc has the core substrate and first consumer APIs needed
for the Skin IR boundary: structural snapshots, workspace revisions, direct
tree structural/input edits, table snapshots/projections, dependency graph,
invalidation closure, calculation outcomes, workspace views, and snapshot
export/import.

Evidence: current OxCalcTree consumer contract and Rust implementation expose
the direct-context APIs in `src/oxcalc-core/src/consumer.rs`, structural
snapshots in `src/oxcalc-core/src/structural.rs`, dependency graph in
`src/oxcalc-core/src/dependency.rs`, workspace revisions in
`src/oxcalc-core/src/workspace_revision.rs`, and table projection/lowering in
`src/oxcalc-core/src/structured_table.rs`.

Still open: undo/redo version navigation, explicit edit transactions, legality
and impact preview APIs, richer typed drill/explain packets, external update
operation, step/progress/cancellation APIs, and full host-facing table/value
render packets.

Formal status: no new proof claim. This guide organizes the boundary for later
formalization of operation transactions, revision navigation, retention, and
host projection invariants.
