# W057 Workspace Revision And Snapshot-Layer Rework

Status: `live_beads_allocated`

Parent predecessors:
- `W056` TreeCalc full reference and table lowering, especially the
  epoch/snapshot split corrections and the retraction of OxCalc-side formula
  semantic detection.
- `W054` bounded-memory and pinned-epoch GC, for retention surfaces that will
  later key against stable workspace revision and snapshot identities.
- `W050` formula-authority rework, for the OxFml-owned parse/bind/prepared
  artifact boundary.

Live parent epic: `calc-ujl4` (`W057.0` stable workset label).

## 1. Purpose

W057 reworks OxCalc's internal workspace representation around explicit,
immutable snapshot layers and discardable contextual views.

The design is inspired by the red/green-tree architecture used by Roslyn and
the later Rust `rowan`/rust-analyzer stack, but OxCalc should use domain names
rather than color names. The durable idea is:

1. immutable, persistent roots carry authoritative edited truth;
2. edits create a new affected spine and reuse unchanged immutable subtrees;
3. contextual views, lookup helpers, caches, overlays, diagnostics, and
   publications sit outside those roots and are rebuildable or explicitly
   versioned;
4. semantic facts produced by OxFml/OxFunc/FEC are consumed only as typed facts,
   never inferred by OxCalc from formula syntax or function names.

## 2. Naming

W057 adopts these names for the intended model:

1. `WorkspaceRevision`
   - the durable tuple of root snapshots that defines edited workspace truth.
2. `WorkspaceRevisionId`
   - stable identity/hash for a revision tuple.
3. `StructureSnapshot`
   - immutable topology, node identity, parent/child order, symbols, structural
     paths, table shape, anchors, and other engine-visible structural facts.
4. `NodeInputSnapshot`
   - immutable per-node calculation input facts: empty, literal, formula text,
     and later host-owned input variants.
5. `NamespaceSnapshot`
   - immutable host namespace, registry, capability, workspace-availability,
     alias, and caller-context facts that affect binding or prepared identity.
6. `FormulaBindingSnapshot`
   - typed OxFml parse/bind/prepared facts keyed by `NodeInputSnapshot`,
     `StructureSnapshot`, `NamespaceSnapshot`, and declared host context.
7. `DependencyShapeSnapshot`
   - static dependency facts and typed dependency-shape inputs derived from
     structure, inputs, namespace, and formula binding facts.
8. `PublicationSnapshot`
   - last accepted observable values, diagnostics, and dependency-effect
     publication identity.
9. `RuntimeOverlaySet`
   - CTRO/dynamic dependencies, runtime effects, invalidation overlays, and
     other epoch-scoped facts that are not authored truth.
10. `WorkspaceRevisionView` and `EvaluationContextView`
    - discardable contextual views over immutable roots plus publication and
      overlay state.

Avoid public/domain type names such as `GreenTree` or `RedTree`. Those names are
useful design shorthand only.

## 3. Layer Model

The intended root model is:

```text
WorkspaceRevision
  StructureSnapshot
  NodeInputSnapshot
  NamespaceSnapshot
```

Derived immutable or retained artifacts:

```text
FormulaBindingSnapshot
DependencyShapeSnapshot
```

Published and runtime layers:

```text
PublicationSnapshot
RuntimeOverlaySet
```

Discardable views:

```text
WorkspaceRevisionView
EvaluationContextView
```

The model is explicit because the current stepping-stone implementation still
lets structural snapshots, formula text maps, input-value maps, published
values, runtime effects, and caches blur together. W057 turns those implicit
overrides into named state layers with clear authority and compatibility rules.

## 4. Design Principles

### 4.1 Durable Truth Is Immutable

`StructureSnapshot`, `NodeInputSnapshot`, and `NamespaceSnapshot` are immutable.
Any edit produces a new affected spine and a new root identity while reusing
unchanged nodes by stable identity/hash where legal.

### 4.2 Structure Is Not Input

`StructureSnapshot` owns topology and structural metadata only. It must not be
the authority for literal values, formula text, formula attachment, or formula
artifact identity. Those belong to `NodeInputSnapshot` and derived formula
artifact layers.

### 4.3 Input Edits Are Not Structural Edits

Literal value updates, formula text updates, literal-to-formula transitions,
formula-to-literal transitions, and empty/input transitions create successor
`NodeInputSnapshot` roots. They do not create successor `StructureSnapshot`
roots unless they also include a structural operation.

### 4.4 Namespace Mutations Are Their Own Root

Host namespace, registry, capability, workspace availability, alias, and
caller-context mutations create successor `NamespaceSnapshot` roots. They do not
pretend to be structural edits or formula text edits.

### 4.5 Typed Facts Cross The OxFml Boundary

OxCalc does not inspect formula syntax or function names to infer semantic
behavior. OxFml parses and binds formula text. OxFml/OxFunc/FEC expose typed
facts such as host reference packets, structured-reference bind records,
prepared identities, dynamic/runtime-reference declarations, diagnostics, and
capability facts. OxCalc consumes those typed facts.

### 4.6 Views Are Disposable

`WorkspaceRevisionView` and `EvaluationContextView` may expose parent links,
canonical paths, lookups, sparse readers, dependency projections, and evaluator
context helpers. They are red/view-like wrappers. They are not authoritative
truth and may be rebuilt.

### 4.7 Publication Is Atomic And Separate

Accepted publication is separate from edited truth. A candidate evaluates
against an explicit tuple of snapshot identities. On accept, OxCalc atomically
publishes value deltas, dependency-shape/effect deltas, diagnostics, and runtime
overlay facts. On reject/no-publish, the previous `PublicationSnapshot` remains
the observable truth.

### 4.8 Hashing Is A Semantic Tool

Subtree identity and hashing are design goals, not incidental optimizations.
Hashes must distinguish only facts that matter for the layer being hashed. For
example, a `StructureSnapshot` hash must not change for a literal value update,
while a `NodeInputSnapshot` hash must.

## 5. Edit Classification

W057 should make these edit classes first-class:

1. input value update
   - successor `NodeInputSnapshot`;
   - no formula parse/bind;
   - invalidation seeds from published effective graph;
   - structure and namespace roots preserved.
2. formula text update
   - successor `NodeInputSnapshot`;
   - OxFml parse/bind produces successor typed formula facts;
   - dependency-shape delta only from typed binding/dependency facts.
3. literal-to-formula transition
   - successor `NodeInputSnapshot` changing node input kind;
   - previous publication remains reject/no-publish baseline;
   - formula binding artifacts are derived from OxFml typed output.
4. formula-to-literal transition
   - successor `NodeInputSnapshot` changing node input kind;
   - formula dependency facts are released through typed dependency-shape
     publication;
   - dependents invalidate from existing published graph.
5. structural edit
   - successor `StructureSnapshot`;
   - deterministic rebind/rewrite classifier;
   - compatible input, binding, publication, overlay, and cache facts retained
     by stable identity where legal.
6. namespace/registry/capability mutation
   - successor `NamespaceSnapshot`;
   - prepared and bind/runtime artifacts invalidated by explicit compatibility
     rules.

## 6. OxFml Integration Direction

OxFml is expected to be ready for this architecture because formula processing
already has typed parse/bind/prepared identities and host-context inputs.

W057 should connect OxFml results as `FormulaBindingSnapshot` facts rather than
letting OxCalc infer meaning from formula text.

Required boundary:

1. formula text lives in `NodeInputSnapshot`;
2. OxFml parse/bind/prepared output lives in `FormulaBindingSnapshot`;
3. host references and structured references are consumed as typed packets;
4. dynamic/reference-text declarations require typed OxFml/FEC facts before
   OxCalc can classify them as formula-edit dependency-shape changes;
5. OxCalc may compare typed facts and hashes, but not formula function names.

## 7. Benefits

The representation rework is expected to provide:

1. honest edit classification: value, formula, structural, and namespace edits
   no longer share a structural mutation path;
2. stable reader snapshots: old revisions remain valid during evaluation,
   publication, rejection, and replay;
3. cleaner OxFml integration: OxCalc compares typed formula/bind facts instead
   of inspecting formula text;
4. stronger incremental recalculation: subtree identity and hashes identify
   what really changed;
5. cache correctness: prepared formula, dependency graph, sparse reader, and
   value-cache keys can bind to explicit snapshot identities;
6. W054 retention clarity: eviction and pinning can refer to known layers and
   revision ids rather than ad hoc implementation maps;
7. TraceCalc alignment: TraceCalc can rebuild derived facts broadly from the
   same snapshot tuple while optimized OxCalc uses caches;
8. reject safety: no-publish paths preserve the previous publication without
   restoring mutable side state;
9. concurrency readiness: immutable roots and single-publisher publication are
   compatible with later staged parallel evaluation;
10. better formalization: state, transitions, invariants, and hashes have clear
    layer boundaries.

## 8. Relationship To W056

W056 remains the long-running full-reference and table-lowering workset. W057
does not reopen W056 product evidence that is already scoped and exercised.

W057 absorbs the representation lessons from W056:

1. OxCalc-local formula parsing or function-name detection is a boundary error;
2. literal/formula transitions should not be structural topology edits;
3. dynamic CTRO state is a runtime publication/overlay layer;
4. dependency-shape deltas must be typed publication facts;
5. the current side-map implementation is a stepping stone, not the target
   state kernel.

## 9. Relationship To W054

W054 remains the bounded-memory and pinned-epoch GC workset. It should close
bounded retention behavior for the current artifact set without becoming the
workspace representation rewrite.

W057 later retargets W054 retention identities onto:

1. `WorkspaceRevisionId`;
2. `StructureSnapshotId`;
3. `NodeInputSnapshotId`;
4. `NamespaceSnapshotId`;
5. `FormulaBindingSnapshotId`;
6. `DependencyShapeSnapshotId`;
7. `PublicationSnapshot` identity;
8. `RuntimeOverlaySet` epoch.

## 10. Implementation Strategy: Corpus-Driven Hard Cutover

W057 should be implemented as a hard replacement of OxCalc's internal truth
model, not as a long compatibility migration.

The current executable corpus is the safety net. The implementation should
remove the old representation pressure directly, then use compiler failures and
test failures as the work queue. The target is one coherent internal model, not
two partially synchronized models.

The guiding statement for W057 implementation is:

> If a fact belongs to structure, input, namespace, formula binding, dependency
> shape, publication, or runtime overlay, it must live in that layer and nowhere
> else.

### 10.1 Cutover Rules

1. Replace, do not wrap:
   - do not build a long-lived parallel `WorkspaceRevision` path beside the old
     loose maps and structural input/artifact fields;
   - short-lived mechanical duplication is allowed only inside an active
     refactor step and must not be a closure state.
2. Make old truth locations impossible early:
   - remove or de-authorize `StructuralEdit::SetConstantValue`;
   - remove or de-authorize `StructuralEdit::ReplaceFormulaAttachment`;
   - remove `StructuralNode` authority for literal values, formula text,
     formula artifact identity, and bind artifact identity;
   - keep any remaining legacy field only if it is fixture-only and named so it
     cannot be mistaken for engine truth.
3. Let the compiler expose the work:
   - after the old structural-input/artifact hooks are removed, use compile
     failures to find every consumer that still assumes the old model.
4. Use tests as the behavioral contract:
   - preserve public behavior through the corpus rather than preserving old
     internal pathways;
   - when tests reveal a real ambiguity, update the spec or add a clearer guard
     test before patching.
5. Keep OxFml authority intact:
   - do not recover lost behavior by inspecting formula syntax or function
     names in OxCalc;
   - missing formula-language facts must become typed OxFml/FEC handoff
     requirements or explicit exclusions.
6. Delete leftovers before claiming success:
   - unused old maps, structural content fields, compatibility adapters, and
     duplicate identity sources are not harmless;
   - W057 does not close while such leftovers remain in the production path.

### 10.2 Test Corpus Safety Net

The current test corpus should be treated as the refactor harness. At W057
drafting time, the executable surface includes:

1. 264 `oxcalc-core` unit tests;
2. 5 upstream-host integration tests;
3. the checked TraceCalc corpus under
   `docs/test-corpus/core-engine/tracecalc/`;
4. W056 direct-context tests for value/formula edits, dependency-shape
   publication, CTRO invalidation, reject/no-publish, export/import identity,
   table ownership, and host-reference resolution.

The most relevant first guardrail tests are:

1. `treecalc_context_input_value_update_recalculates_dependents_without_full_reset`;
2. `treecalc_context_formula_edit_recalculates_dependents`;
3. `treecalc_context_formula_edit_changed_dependency_preserves_structure_and_recalculates`;
4. `treecalc_context_formula_edit_unresolved_to_resolved_preserves_structure`;
5. `treecalc_context_formula_edit_resolved_to_unresolved_rejects_without_structural_change`;
6. `treecalc_context_formula_edit_cycle_reject_preserves_structure_and_prior_publication`;
7. `treecalc_context_literal_to_formula_preserves_structure_and_publishes_activation`;
8. `treecalc_context_formula_to_literal_preserves_structure_and_publishes_release`;
9. `treecalc_context_literal_to_formula_cycle_reject_preserves_prior_literal_value`;
10. `treecalc_context_indirect_resolves_reference_text_and_records_ctro_edge`;
11. `treecalc_context_export_import_preserves_identity_and_recalc_state`;
12. `treecalc_runner_emits_local_run_artifacts`.

Recommended test rings:

1. direct-context cutover ring:
   `cargo test -p oxcalc-core consumer::tests::treecalc_context_`;
2. optimized runtime ring:
   `cargo test -p oxcalc-core treecalc::tests::`;
3. full local core ring:
   `cargo test -p oxcalc-core`;
4. upstream-host/public facade ring:
   full package tests including `tests/upstream_host_scaffolding.rs`.

### 10.3 W057.1 Field Authority Audit

W057.1 reviewed inbound OxFml observations before editing. The relevant
standing constraints are that OxFml owns formula grammar, parse, bind,
prepared identity, formal-reference facts, dynamic/runtime-reference
declarations, and evaluator diagnostics; OxCalc must consume typed facts rather
than infer formula semantics from text or function names.

Compact current-state classification before code removal:

| Field or surface | Current role | Target authority | W057 action |
|---|---|---|---|
| `StructuralSnapshot.snapshot_id`, root id, parent ids, child order, symbols, projection paths, and table shape anchors | Structural truth | `StructureSnapshot` inside `WorkspaceRevision` | Preserve as structural authority and give it explicit revision identity. |
| `StructuralNode.kind` | Mixed topology/input hint today | Derived structural/input classification | Stop treating it as calculation-input truth; retain only structural categorization that survives W057.3. |
| `StructuralNode.constant_value` | Removed in W057.3 | `NodeInputSnapshot` for literal/input truth; `PublicationSnapshot` for accepted values | Production structure no longer carries literal values; direct tests now assert input truth round-trips through the snapshot/input layer. |
| `StructuralNode.formula_artifact_id` and `StructuralNode.bind_artifact_id` | Removed in W057.3 | `FormulaBindingSnapshot` facts from OxFml parse/bind/prepared output | Production structure no longer carries formula/bind attachments; formula artifacts are rebuilt from formula text/binding facts. |
| `OxCalcTreeWorkspaceState.formula_texts` and `formula_text_versions` | Current direct-context formula/input text authority | `NodeInputSnapshot`, then `FormulaBindingSnapshot` for OxFml-derived facts | W057.2/W057.6 replace loose maps with snapshot-layer roots. |
| `OxCalcTreeWorkspaceState.input_values`, `input_value_epochs`, and `value_epoch` | Current direct-context literal input and value-epoch authority | `NodeInputSnapshot` and revision-level input epoch identities | W057.2/W057.5 replace loose maps with node-input records and deterministic identities. |
| `seeded_published_values`, `seeded_published_runtime_effects`, and `last_result` | Current publication/runtime carry-forward cache | `PublicationSnapshot` and `RuntimeOverlaySet` | W057.10 separates accepted publication from runtime overlays and reject preservation. |
| `pending_invalidation_seeds`, `pending_formula_edit_diagnostics`, and `pending_dependency_shape_updates` | Current transient direct-context edit queues | candidate/edit transition records plus `DependencyShapeSnapshot` publication | W057.8/W057.9/W057.10 split binding, dependency-shape, and publication responsibilities. |
| `table_snapshots`, `deleted_table_facts`, and `table_state_version` | Current direct-context table object and namespace state | `StructureSnapshot` for ownership/shape plus `NamespaceSnapshot` for table/caller context identity | W057.4/W057.7 retarget table lifecycle and prepared-identity inputs. |

W057.3 closure of the W057.1 leftovers:

1. `StructuralEdit::SetConstantValue` and
   `StructuralEdit::ReplaceFormulaAttachment` are removed from production
   structural APIs;
2. `StructuralNode` no longer contains literal value, formula artifact, or bind
   artifact fields;
3. direct tests that used stale structural mirrors were replaced with tests that
   prove input and formula truth round-trip through explicit input/formula
   layers;
4. active TreeCalc fixtures now carry literal inputs in top-level
   `input_values`, while formula artifact ids remain in formula-binding facts.

### 10.4 W057.2 Core Snapshot-Type Execution Note

W057.2 introduces the first code-level destination snapshot roots in
`src/oxcalc-core/src/workspace_revision.rs`:

1. `NodeInputKind`, `NodeInputRecord`, `NodeInputSnapshot`, and
   `NodeInputSnapshotId`;
2. `NamespaceSnapshot` and `NamespaceSnapshotId`;
3. `WorkspaceRevision` and `WorkspaceRevisionId`;
4. skeletal `FormulaBindingSnapshot`, `DependencyShapeSnapshot`,
   `PublicationSnapshot`, and `RuntimeOverlaySet` shells with explicit
   current-absence states.

Direct-context workspace creation now constructs a `WorkspaceRevision`. The root
node has an explicit empty `NodeInputRecord` inside a `NodeInputSnapshot`, and
workspace views expose the revision, node-input, namespace, derived,
publication, and runtime-overlay identities for regression checks.

Constructor rule: `NodeInputRecord` constructors do not inspect text syntax.
`NodeInputRecord::literal(node, "=1+1", epoch)` remains literal, and
`NodeInputRecord::formula_text(node, "plain text", epoch)` remains formula text.
Any text-to-kind classification still lives in direct-context adapter code while
the old API accepts a single text field; W057.5/W057.6 will move value and
formula edits onto explicit `NodeInputSnapshot` transition APIs.

Deliberate non-claims:

1. `formula_texts`, `input_values`, and `value_epoch` remain legacy bridge maps
   until W057.5/W057.6;
2. publication and runtime shells are explicit absence markers, not the final
   W057.10 publication/overlay split;
3. export/import still uses the old serialized shape and rebuilds the revision
   bridge on import until W057.11.

### 10.5 W057.3 Structural Authority Removal Execution Note

W057.3 removed the production structural authority for literal values and
formula/bind artifacts:

1. `StructuralNode` now contains only node id, kind, symbol, parent, and child
   topology fields;
2. `StructuralEdit` no longer has value-setting or formula-attachment variants;
3. the local engine seeds working values only from explicit input and published
   value maps;
4. direct-context add/edit paths write literal truth into input maps and
   `NodeInputSnapshot` records, not into structure;
5. checked-in TreeCalc fixtures moved literal values out of node records and
   into top-level `input_values`; post-edit value changes use successor
   `input_values` and upstream-publication invalidation, not structural edits.

Fresh-eyes correction during W057.3: an initial mechanical edit briefly wrote one
Rust file incorrectly; the file was restored from the committed baseline before
the semantic edits were reapplied. Follow-up validation included source searches
for removed structural authority names and the full `oxcalc-core` test ring.

Deliberate non-claims:

1. `StructuralNode.kind` still contains legacy `Constant`/`Calculation` labels;
   W057.3 stops treating those labels as input/formula authority, but a later
   bead may rename or narrow them;
2. formula text and input maps still exist as direct-context bridge state until
   W057.5/W057.6 complete the snapshot-root transition;
3. historical archives and older workset prose may still mention the removed
   structural variants as past architecture.

## 11. Planned Bead Set

These are the W057 bead definitions. Live `br` records were allocated on
2026-05-27. The generated `br` ids are execution truth; the `W057.*` labels are
stable workset labels.

Live bead mapping:

| Workset label | Live bead id |
|---|---|
| `W057.0` | `calc-ujl4` |
| `W057.1` | `calc-ujl4.1` |
| `W057.2` | `calc-ujl4.2` |
| `W057.3` | `calc-ujl4.3` |
| `W057.4` | `calc-ujl4.4` |
| `W057.5` | `calc-ujl4.5` |
| `W057.6` | `calc-ujl4.6` |
| `W057.7` | `calc-ujl4.7` |
| `W057.8` | `calc-ujl4.8` |
| `W057.9` | `calc-ujl4.9` |
| `W057.10` | `calc-ujl4.10` |
| `W057.11` | `calc-ujl4.11` |
| `W057.12` | `calc-ujl4.12` |
| `W057.13` | `calc-ujl4.13` |
| `W057.14` | `calc-ujl4.14` |
| `W057.15` | `calc-ujl4.15` |
| `W057.16` | `calc-ujl4.16` |

### W057.0 Parent Epic: Snapshot-Layer Hard Cutover

Outcome: replace the current loose structural/input/artifact state model with
the snapshot-layer model for the direct `OxCalcTreeContext` path and the local
optimized TreeCalc runtime.

Gate:

1. direct-context, optimized runtime, full local core, and upstream-host test
   rings pass or exact accepted blockers are recorded;
2. production structure APIs cannot own literal value, formula text, formula
   artifact, or bind artifact truth;
3. W054 has a concrete retention identity map over the new layers.

### W057.1 Corpus Guardrails And Field Authority Audit

Outcome: establish the executable safety net and classify current state fields
before code removal starts.

Touched surfaces:

1. `src/oxcalc-core/src/consumer.rs`;
2. `src/oxcalc-core/src/structural.rs`;
3. `src/oxcalc-core/src/treecalc.rs`;
4. export/import and runner surfaces;
5. W057 workset notes if the audit finds a design gap.

Gate:

1. add or strengthen tests that fail if structural content remains authoritative
   for literal values, formula text, formula artifact identity, or bind artifact
   identity;
2. record a compact field classification in the W057 packet or a linked
   implementation note;
3. run the direct-context cutover ring.

Depends on: W057.0.

### W057.2 Core Snapshot Types And Identity Constructors

Outcome: introduce the destination state model before deleting old authority.

Required types:

1. `NodeInputKind`;
2. `NodeInputRecord`;
3. `NodeInputSnapshot` and `NodeInputSnapshotId`;
4. `NamespaceSnapshot` and `NamespaceSnapshotId`;
5. `WorkspaceRevision` and `WorkspaceRevisionId`;
6. skeletal `FormulaBindingSnapshot` and `FormulaBindingSnapshotId`;
7. skeletal `DependencyShapeSnapshot` and `DependencyShapeSnapshotId`;
8. skeletal `PublicationSnapshot`;
9. skeletal `RuntimeOverlaySet`;
10. temporary construction helpers needed by direct context and tests.

Gate:

1. workspace creation builds a `WorkspaceRevision`;
2. root node input is represented through `NodeInputSnapshot`;
3. derived/publication/runtime layer shells have deterministic identities or
   explicit current-absence markers;
4. identity constructors are deterministic;
5. no formula semantics are inferred in these constructors.

Depends on: W057.1.

### W057.3 Remove Structural Input And Formula-Artifact Authority

Outcome: make the old model unrepresentable in production structure APIs.

Required removals or de-authorizations:

1. `StructuralEdit::SetConstantValue`;
2. `StructuralEdit::ReplaceFormulaAttachment`;
3. `StructuralNode.constant_value`;
4. `StructuralNode.formula_artifact_id`;
5. `StructuralNode.bind_artifact_id`;
6. production code paths that treat structural nodes as formula/input owners.

Allowed exception: fixture-only scaffolding may keep legacy-looking fields only
when renamed or contained so it cannot be confused with production truth.

Gate:

1. `rg` confirms the removed authority is absent from production structural
   paths;
2. structural tests are updated to assert topology/identity behavior only;
3. compiler failures from the removal are fixed by moving callers to snapshot
   roots, not by adding compatibility shims.

Depends on: W057.2.

### W057.4 Workspace Lifecycle And Structural Edits On `WorkspaceRevision`

Outcome: port workspace creation, node insertion, rename, move, reorder,
delete, and table structural changes to operate through `WorkspaceRevision`.

Gate:

1. structural edits advance `StructureSnapshotId` and `WorkspaceRevisionId`;
2. structural edits preserve compatible `NodeInputSnapshot` records by stable
   node identity;
3. delete removes affected node input records and publication/runtime facts
   through explicit logic;
4. direct workspace lifecycle and table ownership tests pass.

Depends on: W057.3.

### W057.5 Node Input Snapshot Edit Path

Outcome: port literal value, empty, clear, and non-formula input updates to
`NodeInputSnapshot`.

Gate:

1. literal value updates advance `NodeInputSnapshotId` and `WorkspaceRevisionId`
   without advancing `StructureSnapshotId`;
2. no formula parse/bind occurs for literal value edits;
3. invalidation seeds come from the old published effective graph;
4. `treecalc_context_input_value_update_recalculates_dependents_without_full_reset`
   passes through the new model.

Depends on: W057.4.

### W057.6 Formula Text And Literal/Formula Transition Path

Outcome: port formula text edits, literal-to-formula, formula-to-literal,
empty-to-formula, and formula-to-empty transitions to `NodeInputSnapshot`.

Gate:

1. all input-kind transitions advance `NodeInputSnapshotId` and preserve
   `StructureSnapshotId` unless paired with a structural edit;
2. rejected literal-to-formula work preserves prior `PublicationSnapshot`;
3. transition records expose enough typed input-kind change information for
   W057.9 to publish activation/release dependency-shape deltas;
4. this bead does not own dependency-shape publication;
5. W056 formula edit and literal/formula transition tests pass.

Depends on: W057.5.

### W057.7 Namespace Snapshot And Prepared-Identity Compatibility

Outcome: introduce `NamespaceSnapshot` as the authority for host namespace,
registry, capability, workspace availability, alias, table/caller context, and
other bind/prepared-identity facts currently carried as loose environment
versions.

Gate:

1. namespace/capability mutations advance `NamespaceSnapshotId` and
   `WorkspaceRevisionId`;
2. prepared identity inputs consume namespace identity from the snapshot root;
3. existing namespace, caller-context, capability, table-context, and
   cross-workspace prepared-identity tests pass.

Depends on: W057.4.

### W057.8 Formula Binding Snapshot Intake

Outcome: represent OxFml parse/bind/prepared outputs as
`FormulaBindingSnapshot` facts keyed by compatible workspace roots and host
context.

Gate:

1. formula catalog generation reads formula text from `NodeInputSnapshot`;
2. OxCalc consumes OxFml host-reference, structured-reference, diagnostic, and
   prepared-identity facts as typed outputs;
3. no OxCalc formula syntax or function-name detection is introduced;
4. raw host-reference and structured-reference formula tests pass.

Depends on: W057.6 and W057.7.

### W057.9 Dependency Shape Snapshot And Static Delta Publication

Outcome: derive and retain `DependencyShapeSnapshot` from workspace roots plus
typed formula-binding facts, then publish static dependency-shape deltas from
that layer.

Gate:

1. `same_dependencies`, `dependency_shape_changed`,
   `unresolved_to_resolved`, `resolved_to_unresolved`,
   `literal_to_formula`, and `formula_to_literal` classifications compare
   typed dependency-shape facts;
2. literal-to-formula publishes activated formula dependency facts;
3. formula-to-literal publishes released formula dependency facts;
4. accepted candidates and publication bundles carry first-class dependency
   shape updates;
5. rejected candidates publish no dependency-shape deltas;
6. `treecalc_runner_emits_local_run_artifacts` still treats dependency-shape
   updates as publish-critical.

Depends on: W057.8.

### W057.10 Publication Snapshot And Runtime Overlay Separation

Outcome: make accepted publication and runtime overlay state explicit around
`PublicationSnapshot` and `RuntimeOverlaySet` concepts in the direct context
and local runtime.

Gate:

1. accepted value deltas, diagnostics, dependency-shape updates, and runtime
   effects publish atomically;
2. reject/no-publish preserves the previous publication snapshot;
3. CTRO dynamic effects stay runtime publication/overlay facts, not authored
   workspace truth;
4. dynamic target value update, dynamic target switch, and cycle-reject tests
   pass.

Depends on: W057.9.

### W057.11 Export, Import, Views, And Direct Context API Cutover

Outcome: update direct context export/import, workspace views, node views, table
views, and diagnostics to expose the snapshot-layer model without leaking old
truth locations.

Gate:

1. export/import round-trips `WorkspaceRevision`, node inputs, namespace facts,
   publication seeds, runtime effects, table snapshots, and deleted-table facts;
2. view APIs read from revision roots plus publication/runtime layers;
3. imports reject snapshots missing required input truth;
4. snapshot export schema is explicitly versioned;
5. public API compatibility or migration behavior is recorded;
6. any DnaTreeCalc, OxReplay, or OxXlPlay impact is handed off or explicitly
   ruled out;
7. `treecalc_context_export_import_preserves_identity_and_recalc_state` and
   import rejection tests pass.

Depends on: W057.10.

### W057.12 Local Optimized Runtime Input Cutover

Outcome: port `LocalTreeCalcInput`, catalog construction, invalidation,
scheduling, sparse readers, edge-value cache keys, and runner artifacts to
consume the revision tuple and derived layers.

Gate:

1. `LocalTreeCalcEngine` consumes workspace revision/layer identities rather
   than loose structural/input maps;
2. published effective graph invalidation still includes static plus prior
   published dynamic dependencies;
3. edge-value cache keys use explicit snapshot/artifact/publication bases;
4. `cargo test -p oxcalc-core treecalc::tests::` passes or has exact accepted
   blockers.

Depends on: W057.11.

Live split guidance: when this planned bead is created in `br`, split it if
needed into runtime input shape, invalidation/effective graph, cache-key, sparse
reader, and runner-artifact tasks. Do not create a compatibility-migration
branch.

### W057.13 TraceCalc And Differential Corpus Migration

Outcome: align TraceCalc scenario state with `WorkspaceRevisionRef`,
`PublicationSnapshot`, `RuntimeOverlaySet`, and dependency-shape/publication
facts so it remains the semantic oracle for the hard cutover.

Gate:

1. TraceCalc corpus schemas and docs name workspace revision rather than
   structural snapshot as the whole truth root;
2. value update, formula update, dynamic target update/switch, unresolved/
   resolved, rename/delete/move, and CTRO cycle reject scenarios have oracle
   coverage or exact blockers;
3. optimized-vs-TraceCalc differential fixtures cover the W056 epoch/snapshot
   scenario set.

Depends on: W057.12.

Live split guidance: when this planned bead is created in `br`, split it if
needed into TraceCalc schema, scenario additions, optimized/differential runner,
and retained artifact refresh tasks.

### W057.14 W054 Retention Identity Retarget

Outcome: produce the W054 retention identity map and update the touched
retention keys/docs for the new layer identities. This bead does not close full
W054 retention behavior.

Gate:

1. W054 docs and touched retention tests name `WorkspaceRevisionId`,
   `StructureSnapshotId`, `NodeInputSnapshotId`, `NamespaceSnapshotId`,
   `FormulaBindingSnapshotId`, `DependencyShapeSnapshotId`,
   `PublicationSnapshot`, and `RuntimeOverlaySet` where relevant;
2. a concrete retention identity map exists for W054 follow-up work;
3. touched cache/overlay keys use the new identities where the W057 cutover
   reaches them;
4. per-edge value-cache eviction evidence still passes if touched;
5. pinned reader lifecycle counters remain deterministic if touched;
6. W053 speculative retention remains routed forward.

Depends on: W057.12.

### W057.15 Legacy Leftover Deletion And Production Path Audit

Outcome: remove compatibility leftovers after the cutover rather than leaving
duplicate truth sources behind.

Gate:

1. `rg` finds no production `StructuralEdit::SetConstantValue`,
   `StructuralEdit::ReplaceFormulaAttachment`, `StructuralNode.constant_value`,
   `StructuralNode.formula_artifact_id`, or `StructuralNode.bind_artifact_id`;
2. no production `OxCalcTreeWorkspaceState` loose maps remain authoritative for
   node input, formula text, formula artifact, bind artifact, namespace, or
   publication truth;
3. any remaining fixture-only compatibility data is quarantined and named as
   fixture-only;
4. direct-context, optimized runtime, and full local core rings pass.

Depends on: W057.13 and W057.14.

### W057.16 Closure Audit And Successor Handoff

Outcome: audit W057 against the hard-cutover objective and decide whether the
snapshot-layer refactor is closed, blocked, or needs a narrowed successor.

Gate:

1. closure report states product status, evidence, still open, and formal
   status separately;
2. all W057 child beads are closed or have exact accepted blockers;
3. test rings and `rg` leftover checks are recorded;
4. OxFml/FEC typed-fact gaps are either absent, handed off, or recorded as
   explicit exclusions;
5. W054 and W049 successor implications are updated.

Depends on: W057.15.

### Dependency Spine

Primary sequence:

```text
W057.0
  -> W057.1
  -> W057.2
  -> W057.3
  -> W057.4
  -> W057.5
  -> W057.6
  -> W057.8
  -> W057.9
  -> W057.10
  -> W057.11
  -> W057.12
  -> W057.13
  -> W057.15
  -> W057.16
```

Parallel branch after `W057.4`:

```text
W057.4 -> W057.7 -> W057.8
```

Retention branch:

```text
W057.12 -> W057.14 -> W057.15
```

Split rule: if the live `br` graph needs smaller beads, split only within
these outcomes and connect the split beads under the relevant live W057 child.
Do not create a separate compatibility-migration lane unless the W057 workset is
explicitly rescoped.

## 12. Non-Goals

W057 does not:

1. implement new formula semantics;
2. widen reference-family product support by itself;
3. close W054 bounded-memory retention;
4. introduce Stage 2 concurrency;
5. replace OxFml as formula parser/binder/evaluator;
6. require public API churn before the internal state kernel has a migration
   path.

## 13. Closure Gate

W057 can close its first scope when:

1. `WorkspaceRevision`, `StructureSnapshot`, `NodeInputSnapshot`, and
   `NamespaceSnapshot` are specified and implemented for the direct
   `OxCalcTreeContext` path;
2. literal value edits, formula text edits, literal-to-formula transitions, and
   formula-to-literal transitions preserve `StructureSnapshot` identity while
   advancing `NodeInputSnapshot` identity;
3. structural edits advance `StructureSnapshot` identity and preserve compatible
   node inputs by stable identity;
4. namespace/capability mutations advance `NamespaceSnapshot` identity and
   invalidate formula artifacts through explicit compatibility rules;
5. OxCalc no longer stores authoritative input truth in mutable side maps or
   content-like structural fields;
6. production `StructuralNode` and `StructuralEdit` APIs cannot act as
   authorities for literal values, formula text, formula artifact identity, or
   bind artifact identity;
7. formula/bind facts are consumed as typed OxFml outputs;
8. publication/reject behavior is preserved by tests and TraceCalc/optimized
   differential evidence for the W056 epoch/snapshot scenarios;
9. the direct-context, optimized runtime, full local core, and upstream-host
   test rings pass or have exact accepted blockers;
10. W054 has an explicit follow-up map for retention identities under the new
   snapshot-layer model.

## 14. Status

Product status: design workset created; W057.1 guardrails/audit executed; W057.2
core snapshot-layer types and direct-context workspace-revision bridge executed;
W057.3 production structural input/formula authority removal executed; W057.4
workspace lifecycle, structural edit, delete cleanup, and table-shape revision
routing executed; W057.5 literal and clear input edits route through
`NodeInputSnapshot` identity. No full snapshot-layer product cutover claim yet.

Evidence: current W056 direct-context tests expose the need for the split; W054
initial retention work exposes the need for stable retention identities; W057.1
adds direct-context guardrails proving stale structural content/artifact fields
do not win over explicit input/formula facts in the current transition model;
W057.2 adds deterministic constructor tests and direct-context workspace-creation
tests proving the root input is represented through `NodeInputSnapshot`; W057.3
removes structural value/formula edit variants, removes structural value and
formula/bind fields, moves fixture literal values to explicit `input_values`,
and passes the full `cargo test -p oxcalc-core` ring; W057.4 adds structural
table-shape facts, routes table attach/clear through structural snapshots,
preserves compatible node-input records across rename/move/reorder/add, and
explicitly prunes delete-scoped input/publication/runtime/table facts with the
focused structural/direct-context tests and full `cargo test -p oxcalc-core`
ring passing; W057.5 makes `NodeInputSnapshot` the input-kind authority for
non-formula input edits, adds explicit clear-to-empty input coverage, and keeps
literal edits out of the formula-edit classifier while the focused
direct-context and full `cargo test -p oxcalc-core` rings pass.

Still open: W057.6-W057.16 implementation, OxFml typed-fact gap audit, subtree
hash design, TraceCalc differential migration, and W054 retention retargeting.

Formal status: no proof claim.
