# W060 Calc-Time Reference Representation And Host Reference System

## 1. Purpose

W060 is a design and execution-planning workset for replacing the remaining
calc-time `HOST_REF_*` formal-token bridge with a typed reference
representation based on `CalcValue::Reference`.

The workset is intentionally narrower than W059. W059 owns authored-input
interpretation and the pre-evaluation `BoundFormula` artifact used for
dependency graph construction. W060 owns the runtime value representation and
reference-system interface used after the dependency graph has scheduled a node
for calculation.

## 2. Boundary Statement

This workset does not reopen the `BoundFormula` reference representation used
for dependency graph construction.

At structure/bind time:

1. OxFml parses and binds formula text.
2. OxFml calls the host context to resolve formula-visible host references.
3. OxFml returns `BoundFormula`.
4. OxCalc consumes `BoundFormula.root` and bound reference facts to build the
   dependency graph.

At calc time:

1. a compiled/evaluated reference expression may produce
   `CalcValue::Reference`;
2. OxFunc receives that reference as an opaque value;
3. OxFunc uses the function execution context to dereference, enumerate, query,
   resolve, or transform the reference when the function semantics require it;
4. OxCalc implements the host reference system for the active profile.

These two representations may converge later, but they are not required to be
the same type now.

## 3. Current Problem

The current migration has already removed host-reference formula text
projection as the main parser surface. OxCalc now sends the original authored
formula text to OxFml and enriches the returned bound expression tree.

However, the runtime bridge still uses synthetic formal-token strings:

1. OxFml emits internal formal ids such as `HOST_REF_5_9`.
2. OxCalc maps those ids to sparse/reference-value bindings.
3. runtime evaluation recognizes the synthetic reference by string key.

That is better than rewriting formula text, but it is still a stringly typed
runtime identity for something that should be a typed host-owned reference
handle.

## 4. Target Model

`CoreValue::Reference` is the right value lane for references.

The target is not `RichValue::Reference`. References are part of the core
XLOPER-style value gamut and should remain visible to ordinary function
argument preparation, reference-preserving functions, and function execution
context dereference paths.

The type that needs expansion is the reference payload. Today OxFunc's
reference shape is effectively:

```rust
pub struct ReferenceLike {
    pub kind: ReferenceKind,
    pub target: String,
}
```

That is sufficient for textual A1/range-style references, but not sufficient
for TreeCalc host references, structural selectors, sparse collections, or
future profile-specific reference systems without inventing fake strings.

The desired direction is:

```rust
pub struct ReferenceLike {
    pub system: ReferenceSystemId,
    pub identity: ReferenceIdentity,
    pub display: Option<ReferenceDisplay>,
}

pub enum ReferenceIdentity {
    Textual(TextualReferenceIdentity),
    Opaque(ReferenceHandle),
    Composite(CompositeReferenceIdentity),
}
```

Names are provisional. The requirements are not:

1. identity is typed and host-owned;
2. display text is informative only and never the functional identity;
3. OxFunc treats the handle as opaque;
4. OxCalc implements profile-specific dereference and transform behavior;
5. `HOST_REF_*` disappears as a runtime identity.

## 5. Host Reference System

The reference system is a host/profile capability surfaced through the function
execution context. OxCalc owns the current TreeCalc implementation and should
leave room for future profiles such as an Excel-compatible grid reference
system.

From OxFunc's perspective, a reference handle supports these operations through
FEC/reference-system interfaces:

1. describe the reference for diagnostics, trace, replay, and display:
   non-functional metadata only;
2. dereference the reference to a non-reference `CalcValue`, which may be a
   scalar, array, error, empty, or rich value, but not another reference unless
   a specific reference-preserving function requests such behavior;
3. enumerate or stream sparse/lazy values with shape, defined-cell positions,
   blank/empty distinction, ordering, duplicate semantics where applicable, and
   reader identity;
4. query reference facts such as area count, extent, anchor/address facts,
   caller-context-sensitive address facts, or shape facts;
5. resolve text to a reference in the current execution context, which is the
   `INDIRECT`-style lane and must go through OxFml parsing/binding and OxCalc
   host-reference resolution rather than OxFunc parsing reference text;
6. compose or transform references by typed request, for functions such as
   `OFFSET`, reference-form `INDEX`, union, intersection, and structural
   selector application.

Composition and transformation are host reference-system operations, not OxFml
semantics. OxFml owns syntax and binding; OxCalc owns the reference systems
for active profiles.

### 5.1 Implementation Ownership

`ReferenceSystemProvider` is a host capability, not an OxFml-owned reference
universe.

The target implementation layout is:

1. OxFunc owns the `ReferenceSystemProvider` trait, the request/result/error
   types, and any truly host-neutral helpers.
2. OxCalc implements the real TreeCalc provider, using OxCalc-owned calc state,
   node value tables, structure/profile context, CTRO machinery, and
   sparse/lazy reference readers.
3. OxFml accepts an optional borrowed provider on its runtime/evaluation
   context and attaches it to every OxFunc function execution context.
4. OxFml does not dereference, enumerate, or otherwise keep host references on
   behalf of OxCalc.
5. OxFml may keep temporary compatibility adapters only while legacy
   `ReferenceResolver`/`ReferenceTextResolver` call paths remain in OxFunc and
   OxFml tests.

This means the real TreeCalc path should look like:

```text
OxCalc calc session
  -> TreeCalcReferenceSystemProvider
  -> OxFml evaluation context
  -> OxFunc FunctionExecutionContext
  -> OxFunc function asks provider for reference operations
  -> OxCalc provider performs host/profile-specific reference work
```

OxFml's job in that flow is orchestration. It evaluates the bound expression
and calls OxFunc with the right FEC. It must not reconstruct TreeCalc reference
identity from source text, `HOST_REF_*` ids, or display strings.

### 5.2 OxCalc TreeCalc Provider

The first real provider should be an OxCalc-owned TreeCalc implementation,
named along the lines of `TreeCalcReferenceSystemProvider`.

It should implement the OxFunc trait approximately as follows:

1. `dereference`:
   map typed `CalcValue::Reference` / `ReferenceLike` identity to the current
   non-reference `CalcValue` for the referenced node, range, collection, or
   selector result. If the reference requires a value that is not available in
   the current calc pass, this is where CTRO-style interruption/re-entry is
   surfaced through the appropriate error/effect path.
2. `enumerate_values`:
   expose sparse/lazy values with declared extent, defined cells, ordering,
   duplicates where semantically relevant, and reader identity. This is the
   lane for reference-literal arrays and large/sparse collections.
3. `resolve_text`:
   handle `INDIRECT`-style runtime text resolution in the current execution
   context. OxFunc must not parse reference text itself. The provider should
   route through the same host/formula reference-resolution machinery used by
   OxFml binding, with OxCalc retaining host-reference authority.
4. `facts`:
   expose non-mutating reference facts for diagnostics, replay, shape checks,
   and reference-sensitive function behavior.
5. transform/compose operations:
   route reference-form `INDEX`, `OFFSET`, union/intersection, and structural
   selector transforms to OxCalc/profile code rather than to OxFml grammar
   code. If the first OxFunc trait version does not yet include explicit
   transform methods, the API should leave room for them rather than baking
   transformations into display strings.

The provider may borrow the active calc execution state. It should not be a
global registry and should not require OxFunc or OxFml to know TreeCalc
internals.

For tables, the provider follows the W056 table-subtree policy:

1. the table is not a single `CalcValue`;
2. table shape, row/column identity, headers, totals, and body-cell node
   mapping are OxCalc structure/table facts;
3. a `CalcValue::Reference` may point to a table or structured table
   selection by opaque host-owned identity;
4. dereferencing a bare table reference returns the data body as a
   non-reference `CalcValue` array, excluding headers and totals;
5. dereferencing explicit structured selections returns the selected region
   according to the bound structured-reference facts;
6. provider-backed table dereference reads the current `CalcValue`s of table
   body cell nodes from the active calc state and surfaces CTRO/re-entry if a
   structurally required cell value is not available at calc time.

Current implementation note: OxCalc now exercises this policy for ordinary
node formulas of the form `=SUM(SalesTable[Amount])` and
`=COUNTA(SalesTable[[#Headers],[Amount]])`, and
`=SalesTable[[#Totals],[Amount]]`. Data-body bind records are projected into
dependencies on the selected table body cell nodes, totals bind records are
projected into dependencies on selected summary-row cell nodes, and header
selections are shape-backed table/column facts with derived literal cells. The
runtime TreeCalc provider exposes current body-cell `CalcValue`s, current
totals-cell `CalcValue`s, or derived header cells as sparse reference values
for OxFml/OxFunc execution. Broader table selections, table-formula row
context, and retained replay remain scoped by W056.

Provider-owned reference descriptors should be keyed by stable host-owned
reference identity, not by formula occurrence. For TreeCalc today, the
descriptor identity is the opaque host reference handle carried by the
`ReferenceLike`; equivalent formula occurrences may construct equivalent
descriptors, but the provider interns them behind that handle. This keeps the
runtime value small and prepares the path for a later structure-snapshot
binding interner without making OxFml cache host context.

This is a preparation for optimization, not a new binding responsibility for
OxFml. OxFml may carry the stable handle returned by the host resolver in
`BoundFormula`, but OxCalc owns deciding whether that handle maps to an
existing descriptor or a newly materialized descriptor for the current
structure/runtime snapshot.

### 5.3 Formula-Only / Null Reference Profile

The null-reference universe should be explicit rather than an ad hoc fallback.

The proposed split is:

1. OxFunc owns a host-neutral `NullReferenceSystemProvider` implementation of
   `ReferenceSystemProvider`.
2. OxFml owns a formula-only hosting profile that selects the null provider and
   does not expose a host reference namespace.
3. DnaOneCalc can start from the OxFml formula-only profile when it is acting
   as a single-formula proving host with no workbook/tree/grid reference
   universe mounted.

The null provider's intended behavior:

1. `dereference` returns an unresolved or unsupported-reference error.
2. `enumerate_values` returns `Ok(None)` or unsupported, depending on the final
   OxFunc operation contract.
3. `resolve_text` returns unsupported/unresolved.
4. `facts` can return default facts derived from the reference payload.

This profile is not a TreeCalc substitute. If DnaOneCalc or another host wants
named inputs, workbook cells, table data, or scenario variables to behave as
references, that host needs its own provider or must mount an OxCalc/grid
provider. If those values are supplied as formal inputs, defined names, or
literal bindings, the formula-only/null provider remains appropriate.

## 6. Profile Direction

The API shapes should assume multiple reference systems:

1. `dna.treecalc.v1` for the current TreeCalc node/path/selector substrate;
2. a future Excel-compatible grid profile for sheet/workbook ranges and
   multi-area references;
3. future host systems as explicitly versioned profile capabilities.

OxFunc should not branch on TreeCalc versus grid mechanics. It should request
capabilities from the FEC/reference system:

1. value-only dereference;
2. sparse/value reader;
3. reference facts;
4. text resolution;
5. transform/composition.

OxCalc decides what those requests mean for the active host profile.

## 7. Relationship To Dependency Graph Work

The dependency graph work remains structure-time work and should continue to
consume already-bound references from `BoundFormula`.

Current state of the migration:

1. OxFml has `BoundExpr::HostReference`,
   `BoundExpr::HostStructuralSelector`, and
   `BoundExpr::HostReferenceCollection` for the host-reference shapes needed
   by TreeCalc.
2. OxCalc's current dependency projector walks `BoundFormula.root` /
   `BoundExpr` rather than relying on rewritten formula text.
3. simple references such as `=A`, `=A+1`, `=SUM(A)`, `=MyUdf(A)`, and
   `=A(3)` can all flow through the same bound host-reference leaf shape for
   dependency purposes.
4. TreeCalc selectors such as `=A.@CHILDREN` are now represented as a bound
   host structural selector whose base is a normal host-reference leaf.
5. reference-literal arrays such as `SUM({A,B,A})` now produce static
   structural dependencies for `A` and `B`; duplicate/order semantics are
   preserved for the runtime sparse value reader.
6. `SUM({A,INDIRECT("B"),A})` must keep only the structural `A` dependency at
   graph-build time; `B` is not a dependency until calc-time reference
   resolution produces CTRO effects.

Open cleanup in the dependency-graph lane:

1. restore legacy TreeCalc-only selector syntaxes only by teaching OxFml to
   parse/bind them; do not reintroduce an OxCalc source-token fallback;
2. remove remaining source-token/formal-reference correlation needs from the
   structure-time path where a typed bound host reference handle is available;
3. keep OxFml free of dynamic/CTRO dependency declarations for `INDIRECT` and
   other calc-time reference resolution;
4. ensure parse failures such as `=1+` remain rejected authored inputs, not
   accepted formula nodes whose value is an error.

## 8. Execution Plan

### Lane A: Specify the reference-system API

Define the OxFunc-facing reference operations and decide whether they live as
one `ReferenceSystem` trait or as a small family of existing FEC traits.

Minimum operations:

1. describe;
2. dereference to non-reference `CalcValue`;
3. enumerate sparse/lazy values;
4. query reference facts;
5. resolve reference text in current context;
6. transform/compose references.

Record the ownership rule with the API: OxFunc defines the trait; hosts
implement it; OxFml passes it through.

### Lane B: Expand `ReferenceLike`

Replace string identity with typed identity while preserving textual references
as one identity family.

The first implementation can keep backwards-compatible constructors for A1
tests, but the runtime host-reference path must stop using synthetic
`HOST_REF_*` strings.

### Lane C: Replace runtime formal-token bindings

Replace the current maps keyed by `HOST_REF_*` with typed host reference
handles or typed reference-system binding ids.

Target removal points include:

1. OxFml runtime sparse-reference binding keys;
2. OxCalc TreeCalc sparse-reference binding construction;
3. helper methods that generate `HOST_REF_{start}_{len}`;
4. tests whose only assertion is the synthetic id.

### Lane C2: Plumb provider through OxFml

Add an optional borrowed `ReferenceSystemProvider` to OxFml runtime/evaluation
context surfaces and attach it to every `FunctionExecutionContextBundle`
created by OxFml.

This lane should be mechanical and should not make OxFml implement TreeCalc
reference semantics. During migration, OxFml may still build the legacy local
resolver because OxFunc dispatch paths still consume `ReferenceResolver`, but
provider-aware code should receive the host provider when one is supplied.

### Lane C3: Implement OxCalc TreeCalc provider

Implement the first real provider in OxCalc and pass it into OxFml during
TreeCalc node evaluation.

The provider owns:

1. current node/reference value lookup;
2. sparse/lazy collection enumeration;
3. runtime text-to-reference resolution in current context;
4. CTRO interaction for unresolved calc-time references;
5. TreeCalc/profile-specific reference facts and future transforms.

### Lane C4: Add formula-only null profile

Add a host-neutral null provider in OxFunc and an OxFml formula-only profile
that wires it by default for standalone formula execution.

DnaOneCalc should consume this profile for pure single-formula evaluation until
it deliberately mounts a richer host reference universe.

### Lane D: Preserve dependency graph separation

Keep the dependency graph consuming `BoundFormula` / `BoundExpr` and current
structure snapshots. Do not move graph construction onto calc-time
`CalcValue::Reference`.

Add/keep canonical tests:

1. `SUM({A,B,A})` has structural dependencies on `A` and `B`;
2. `SUM({A,INDIRECT("B"),A})` has only structural dependency `A` before
   calculation and reaches `B` through CTRO at calc time;
3. `A.@CHILDREN` graph edges come from bound host structural selector facts,
   not from source-text parsing in OxCalc.

### Lane E: Profile-ready transform requests

Move `OFFSET`, reference-form `INDEX`, union/intersection, and structural
selector transformation toward typed host-reference-system requests. OxFunc
may know the function's semantic request; OxCalc decides host/profile-specific
reference mechanics.

## 9. Evidence Plan

Initial useful checks:

1. OxFunc value-type tests for textual and opaque host reference identities;
2. OxFunc function-call tests proving value-only functions dereference through
   FEC while reference-visible functions preserve/query the reference;
3. OxFml runtime tests proving compiled host references no longer use
   `HOST_REF_*` string identities;
4. OxCalc TreeCalc tests for `A.@CHILDREN`, `SUM({A,B,A})`, and
   `SUM({A,INDIRECT("B"),A})`;
5. a source scan proving no remaining `HOST_REF_` generation in active runtime
   code.
6. OxFml provider-plumbing tests proving a supplied provider is visible through
   the OxFunc FEC without OxFml dereferencing it;
7. OxFunc null-provider tests proving unsupported reference operations fail
   explicitly and facts remain diagnostic-only;
8. DnaOneCalc or OxFml formula-only smoke tests proving pure formulas run with
   the null profile and reference operations fail as rejected/unsupported host
   capability, not as TreeCalc behavior.

## 10. Inbound Observations Reviewed

Reviewed `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`.

Relevant current observations:

1. OxFml's consumer-facing runtime facade is the intended downstream surface;
2. OxCalc should retire wrong-shape formula/reference compatibility
   projections after adopting the current OxFml runtime fields;
3. OxCalc remains owner of TreeCalc selector parsing, table catalogs,
   reference carriers, dependency facts, and invalidation;
4. sparse-reader API/runtime/replay shape is still a cross-repo seam that must
   be made explicit rather than inferred from local formal tokens.

## 11. Closure Condition

W060 closes its first scope when:

1. the reference-system API is specified and implemented for the TreeCalc
   runtime path;
2. `CalcValue::Reference` can carry opaque host-owned reference identity
   without `HOST_REF_*`;
3. OxFunc dereference, sparse enumeration, fact query, text-resolution, and
   transform/composition requests route through FEC/reference-system traits;
4. OxCalc implements the TreeCalc reference system for the exercised
   `@CHILDREN`, reference-literal array, and direct host-reference paths;
5. OxFml passes the host provider through to OxFunc and does not implement the
   TreeCalc reference universe itself;
6. a formula-only/null reference profile exists for standalone OxFml/DnaOneCalc
   execution where no host reference universe is mounted;
7. dependency graph construction still uses `BoundFormula` / `BoundExpr` and
   does not depend on calc-time value materialization;
8. focused OxFunc/OxFml/OxCalc tests cover the canonical structural and CTRO
   examples;
9. any remaining grid/Excel-compatible profile work is explicitly routed to a
   successor lane.

## 12. Current First-Scope Closure Status

Product status: W060's first calc-time reference-system scope is implemented
for the exercised TreeCalc runtime path and host-profile reference floor.
Runtime reference values use `CalcValue::Reference` / `ReferenceLike` with
typed textual, opaque, or composite identity. The active TreeCalc path supplies
an OxCalc-owned `TreeCalcReferenceSystemProvider` to OxFml runtime execution,
and OxFml passes the provider through to OxFunc without owning TreeCalc
reference semantics. Formula-only/no-host execution has explicit unsupported
reference behavior: pure formulas run without a mounted host reference
universe, while runtime reference operations fail as unsupported/reference
errors rather than falling into TreeCalc behavior. OxFunc's
`NullReferenceSystemProvider` separately defines the host-neutral null-provider
operation outcomes.

Evidence:

1. active source scan over `OxCalc/src`, `OxFml/crates`, and `OxFunc/crates`
   has no `HOST_REF_` matches; remaining `HOST_REF_*` text is historical
   workset/bead/status prose only;
2. `cargo test -p oxfunc_core
   null_reference_system_provider_rejects_reference_capabilities` covers null
   provider dereference, sparse enumeration, text resolution, facts,
   transform, and compose outcomes;
3. `cargo test -p oxfml_core
   runtime_formula_only_without_host_provider_runs_pure_formulas_and_rejects_indirect_refs`
   covers formula-only runtime behavior without a supplied host provider;
4. `cargo test -p oxcalc-core
   treecalc_provider_keeps_transform_and_compose_as_typed_unsupported_requests`
   covers TreeCalc transform/compose as typed provider requests rather than
   display-string mechanics;
5. existing TreeCalc/OxCalc tests cover direct node references,
   `@CHILDREN`/`A.@CHILDREN`, reference-literal arrays such as
   `SUM({A,B,A})`, and `INDIRECT` CTRO dynamic reference effects;
6. existing OxFml language-service tests cover profile-symbolic reference
   editor completions and reference info, including source span/text, render
   hint, normal-form key, diagnostics, and profile payload access.

Still open:

1. strict Excel grid runtime semantics, GridCalc-Ref, GridInvalidation-Ref,
   full A1/R1C1 dependency lowering, grid bounds/materialization tests, spill
   anchors, hidden-row behavior, and structural-edit matrices remain W061;
2. broad TreeCalc retained replay and non-table corpus evidence remain W056 /
   W058, not W060 closure gates;
3. broader provider-backed reference transforms such as TreeCalc-specific
   selector transforms, full `OFFSET`, reference-form `INDEX`, and
   union/intersection semantics remain profile/provider implementation work
   beyond this first floor unless a host profile claims them;
4. public prepared-package polish and broader OxFml/OxFunc metadata lanes remain
   successor seam work noted in the OxFml upstream ledger.

Formal status: W060 has implementation and focused regression evidence for the
declared first scope. It does not claim a Green profile, pack-grade replay,
formal proof, strict Excel grid conformance, or full reference-transform
semantic closure.

Reviewed inbound observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`
was reviewed for this pass. The relevant constraints are preserved: use the
consumer runtime/editor surfaces, keep OxFml as formula/editor orchestration
rather than host reference owner, do not over-read caller/address-mode carriage
as full relative-reference closure, and keep strict-grid behavior on the W061
successor lane.
