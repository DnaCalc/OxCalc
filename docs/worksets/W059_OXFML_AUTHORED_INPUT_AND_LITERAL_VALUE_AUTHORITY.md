# W059 OxFml Authored Input And Literal Value Authority

## 1. Purpose

W059 is a discussion and design-prep workset for moving OxCalc authored input
handling onto the same authority boundary already used for formula text:
OxFml owns interpretation of Excel-style cell-entry text, whether the text
starts with `=` or not.

Live parent epic: `calc-rqoq`.

The immediate correction target is the current OxCalc literal path where
non-formula input strings are converted locally with small `f64` / `bool` /
text heuristics. That is still evaluator semantics inside OxCalc. It should be
replaced by an OxFml-authored interpretation path that returns either a
`CalcValue` for value-like entry or an OxFml-owned formula/edit/runtime artifact
for formula-like entry.

## 2. Current Problem Statement

The current OxCalc input path splits text too early:

1. `set_node_input_value` rejects leading-`=` input and stores only non-formula
   text as `NodeInputKind::Literal`.
2. formula text is stored separately through the formula-input path and only
   `NodeInputKind::FormulaText` enters the OxFml catalog.
3. literal strings later flow through OxCalc-local conversion helpers such as
   `string_to_eval_value` and `published_string_to_calc_value`.

That creates a private OxCalc interpretation rule for numbers, logicals, and
text. It misses or risks diverging from OxFml/Excel cell-entry semantics for
apostrophe-forced text, error literals, locale-sensitive entries, dates,
quoted text, blank/empty handling, and future rich/value extensions.

This also weakens the W057 snapshot-layer model. `NodeInputSnapshot` should
hold durable authored input truth, while formula binding and value derivation
should be explicit derived layers from OxFml/OxFunc-owned semantics. A local
string-to-value shortcut blurs those layers.

## 3. Target Doctrine

OxCalc should treat every authored text input as Excel-style cell-entry text:

1. raw entered text is durable input truth,
2. OxFml receives that text with the appropriate host context and
   `FormulaChannelKind`,
3. OxFml classifies and parses it,
4. OxCalc records the resulting typed value and/or formula artifact in derived
   snapshot/publication state.

The target outcome is not a new language object invented in OxCalc. For the
first authored-input call, OxFml returns one of exactly three outcomes:

1. literal input accepted as a `CalcValue`;
2. formula input accepted as a `BoundFormula`;
3. formula input rejected with diagnostics, for example `=1+`.

This first call is not evaluation. It is authored-input interpretation and
formula binding. Later runtime calls may produce `RuntimeFormulaResult`,
`RuntimePreparedFormulaIdentity`, `RuntimePreparedFormulaPackage`, and
`RuntimeFormulaResult::published_calc_value()`, but those are not the first
W059 branch result.

## 3A. Required Pre-Evaluation Structural Artifact

A crucial checkpoint in W059 is the design of the OxFml return shape for
authored input before OxCalc evaluates the node.

OxCalc needs an OxFml-produced intermediate artifact that is past raw text and
past formula-visible reference resolution, but not yet evaluated. For W059,
that artifact is `BoundFormula` unless a later design pass proves a narrower
opaque access pattern is needed.

1. OxFml receives authored input text, which may be a literal value entry or a
   formula entry.
2. If the input is a literal value entry, OxFml returns a clear `CalcValue`
   result for that authored entry.
3. If the input is a formula entry, OxFml returns `BoundFormula`.
4. `BoundFormula` includes or exposes formula-visible references after OxFml
   has resolved them through the host context.
5. OxCalc then uses that artifact, not the raw formula string, to derive
   dependency descriptors, dependency-shape snapshots, and scheduling work.
6. Evaluation happens later through the normal OxFml runtime path after OxCalc
   has dependency and scheduling authority.

This is intentionally different from the DNA OneCalc path. DNA OneCalc can use
OxFml as a live formula editor back-end and then immediately evaluate the
single formula, returning result artifacts. DNA TreeCalc and OxCalc will also
use OxFml directly as the live formula string editor back-end, but the end
result of editing must be `CalcValue`, `BoundFormula`, or diagnostics handed to
the OxCalc engine. OxCalc must not be forced to evaluate immediately just to
obtain the dependency-relevant structure.

The return contract must make the branch unambiguous:

1. literal entry -> `CalcValue`,
2. formula entry -> `BoundFormula`,
3. formula acceptance error -> diagnostics, with no invented local fallback
   parser in OxCalc.

## 3B. Current OxFml Pipeline And W059 Artifact Shape

Current OxFml already builds the relevant pre-evaluation layers:

1. `FormulaSourceRecord` carries stable source identity, text version,
   `FormulaChannelKind`, entered text, and optional stored/projected text.
2. `parse_formula` lexes and parses into a full-fidelity `GreenTreeRoot`. For
   `WorksheetA1`, non-empty text that does not start with `=` first passes
   through the cell-entry literal path and becomes a literal formula root.
3. `RedProjection` provides the typed navigable projection used by editor and
   bind services.
4. `bind_formula` produces `BoundFormula`. This is the closest current
   "expression tree plus binding info" object. It contains:
   - `root: BoundExpr`,
   - `normalized_references`,
   - `structured_reference_bind_records`,
   - `dependency_seeds`,
   - `unresolved_references`,
   - diagnostics and function-call source records.
5. later runtime preparation/evaluation may produce `SemanticPlan`,
   `RuntimePreparedFormulaIdentity`, `RuntimePreparedFormulaPackage`, and
   `RuntimeFormulaResult`, but those are downstream of the first W059
   authored-input result.

DNA Calc policy allows tight Rust type coupling between the Rust projects as
long as the responsibility boundary remains clear:

1. OxFml owns authored-text interpretation, parse, bind, formula-visible
   reference resolution, and evaluator artifact meaning.
2. OxCalc owns immutable workspace/structure versions, dependency graph
   construction, invalidation, scheduling, publication, and runtime overlay
   state.
3. OxFml should not retain OxCalc context as hidden mutable state. OxCalc passes
   the required host context into OxFml; OxFml returns an artifact/result.
4. A returned type may be an OxFml-internal-shaped Rust type, an opaque OxFml
   handle with OxFml query helpers, or a bounded-access view over an OxFml type.
5. The design should not add a broad wrapper layer merely to avoid Rust type
   coupling. The boundary is semantic ownership, not type-package distance.

Therefore the first W059 design choice is not "wrap `BoundFormula` to make it
independent". The choice is what access OxCalc needs:

1. direct access to `BoundFormula` fields,
2. access through methods on an opaque `BoundFormula`-owning artifact,
3. or OxFml helper methods that answer dependency-relevant questions from
   `BoundFormula`.

Any of those is acceptable if OxCalc can build the dependency graph without
parsing or interpreting formula text.

## 3C. What OxCalc Needs From BoundFormula

The OxCalc -> OxFml call already carries the OxCalc owner node context through
the call site and source identity. Owner identity is therefore not a separate
dependency fact returned by OxFml. Prepared/runtime identity is also not part of
the first W059 return branch; it belongs to later runtime preparation/cache
work.

OxCalc does not need to evaluate `BoundExpr`. It needs to turn `BoundFormula`
into dependency descriptors for a new dependency-shape version. The minimum
needed from `BoundFormula` is:

1. the `BoundExpr` tree, so OxCalc or OxFml helper methods can walk the formula
   shape and find reference-bearing positions;
2. `normalized_references`, which are OxFml's resolved formula-visible
   references;
3. `structured_reference_bind_records`, which carry table/structured reference
   bind results and diagnostics;
4. `unresolved_references`, which carry formula tokens OxFml could not bind;
5. bind diagnostics.

Everything else is derived downstream by OxCalc against its current immutable
structure snapshot. "Reference collection facts" are not a separate OxFml
return requirement. They are an OxCalc dependency-graph concept used when a
reference denotes a collection, for example a TreeCalc children selector or a
reference-literal array like `SUM({A,B,A})`. OxCalc derives collection
membership, member-value edges, and ordering/duplicate semantics from the
bound references plus the current structure snapshot.

Today, OxCalc's `oxfml_dependency_descriptors` path builds
`DependencyDescriptor`s from `RuntimePreparedFormulaIdentity.formal_references`
plus TreeCalc-translated reference, unresolved, host-value, collection, and
residual bindings. In the W059 target shape, the same logical descriptor build
should consume `BoundFormula` directly or through OxFml helper methods. The
result is:

1. OxFml binds formula-visible references using the host context and returns
   `BoundFormula`.
2. OxCalc projects bound reference facts into `DependencyDescriptor`s.
3. OxCalc builds a new immutable dependency-shape snapshot against the current
   immutable structure snapshot.
4. Evaluation is scheduled only after the dependency graph says the node's
   prerequisites are available.

This fits the broader immutable-version model. A structure edit may create a
new immutable `StructureSnapshot` while reusing unchanged node objects or
unchanged formula artifacts by identity. A formula edit creates a new authored
input / formula-binding version. OxCalc then computes a new dependency-shape
version from `BoundFormula` and the current structure snapshot,
rather than mutating an existing graph in place or re-reading formula text.

## 3D. Function Names, Host References, And Callable Nodes

W059 must preserve three separate namespaces and timing points:

1. OxFunc-managed function surfaces;
2. OxCalc-managed host references such as node names, node paths, and TreeCalc
   selectors;
3. runtime values that may or may not be callable.

OxFunc does not own a hidden mutable global "current function list." Built-in
metadata, UDF registration, capability overlays, and function availability are
facts in the host running context. A UDF registration updates the host running
context or a derived namespace snapshot. OxFunc may interpret that context and
return an updated function-registration view, but OxFunc must not retain hidden
state that later OxFml calls consult implicitly.

When OxFml accepts formula text, function-name recognition must therefore use
the host-provided running context:

1. OxFml owns syntax, parse, and bind.
2. OxFml asks OxFunc, with the host running context, whether a syntactic call
   token is an admitted built-in function surface or UDF surface in the current
   function namespace. OxFunc's interpreted function list must distinguish
   built-ins from UDFs.
3. OxFml asks OxCalc, with the host running context or immutable snapshot
   identity, whether a token resolves to a host reference such as a node name,
   node path, structured TreeCalc selector, or other host-visible reference.
4. Current name precedence policy is:
   - built-in functions win and cannot be shadowed by node names or UDFs;
   - then OxCalc host references / node names win;
   - then UDF function surfaces are accepted.
   This is the current deterministic TreeCalc rule and should remain explicit
   in tests and diagnostics.
5. The OxCalc host-reference answer is a bind/reference fact, not an OxCalc
   dependency descriptor and not an OxFml `DependencySeed`.

For a syntactic call such as `MyNode(12)`, if `MyNode` does not resolve to a
built-in function and does resolve as an OxCalc host reference, OxFml should
bind the expression as invocation of that reference, even if a UDF with the same
surface name exists. During the structural bind phase neither OxFml nor OxCalc
should require proof that the referenced node currently has a callable value. In
a fresh structure build the referenced node may not have been evaluated yet.

Callable validation is a calculation-time responsibility:

1. the structural dependency graph includes the dependency on `MyNode` because
   the bound formula references `MyNode`;
2. scheduling evaluates `MyNode` before the invocation node when the dependency
   graph requires it;
3. at evaluation/preparation time OxFml/OxCalc can inspect the referenced
   node's `CalcValue`;
4. if that value contains `RichValue::Callable`, the invocation may bind or
   compile to the callable artifact;
5. if the value is not callable, the invocation produces the appropriate
   calculation error, not a parse/bind rejection.

Therefore node-produced callables are not part of the OxFunc function registry
at formula parse time. They are host references that are later invoked as
callable values if calculation makes such a value available.

This policy motivates a formal host-resolution interface rather than the
current mixed packet/context bridge:

1. function-surface lookup: OxFml -> OxFunc with host running context;
2. host-reference lookup: OxFml -> OxCalc with owner/source/snapshot context;
3. namespace precedence: built-in function, then host reference / node name,
   then UDF;
4. dependency graph build: OxCalc maps bound reference facts to
   `DependencyDescriptor`s after binding;
5. callable specialization: runtime/preparation validates and specializes
   invocation only after dependency-ordered values are available.

## 4. DnaOneCalc Evidence Baseline

DNA OneCalc already models the intended direction for a single-formula host:

1. its editor accepts any Excel cell-entry text, not only leading-`=` formulas;
2. host docs say direct value entry and apostrophe-forced string entry remain
   upstream semantic responsibility rather than host-local parsing;
3. `LiveOxfmlBridge::apply_formula_edit` builds a `FormulaSourceRecord` from
   raw `entered_text`, marks it `FormulaChannelKind::WorksheetA1`, calls
   `EditorEditService::apply_edit`, then optionally calls
   `RuntimeEnvironment::execute(RuntimeFormulaRequest)`;
4. OxFml's parser has an explicit `WorksheetA1` cell-entry literal path:
   non-empty, non-`=` text is converted into a literal formula root before
   normal bind/evaluation;
5. OxFml's DNA OneCalc consumer contract says this classification is OxFml-owned
   so OneCalc does not need a host-side fallback evaluator for literal cell
   entries.

One caveat remains relevant to W098: DNA OneCalc's host bridge still exposes
`EvalValue` in `FormulaValuePresentation`, while OxFml now exposes
`RuntimeFormulaResult::published_calc_value()`. OxCalc should target the
`CalcValue` surface directly rather than copying the older OneCalc host mirror.

## 5. OxCalc Migration Questions

Open questions for the refactor:

1. Should `set_node_input_value` accept all authored text, including leading
   `=`, and immediately invoke OxFml interpretation?
2. Should formula-specific APIs remain as convenience wrappers over the same
   authored-input path?
3. What becomes durable in `NodeInputSnapshot`: raw entered text only, or raw
   text plus an entry mode derived by OxFml?
4. Where does OxCalc store the immediate `CalcValue` result for value-like
   entry before recalculation: node-value table, publication layer, or a derived
   input-value cache?
5. For formula-like entry, should OxCalc store `BoundFormula` directly or store
   an opaque OxFml-owned artifact with helper/query access to the same
   `BoundFormula` facts?
6. How should typed API inputs bypass text interpretation while still landing
   in the same `CalcValue` node-value table?
7. How should file-imported values distinguish stored typed values from
   authored cell-entry text?
8. Which host-context facts are required for literal interpretation now:
   locale, date system, caller context, namespace, structured reference context,
   registered functions, table context, and capability overlays?
9. What exact host-resolution interface should replace the current
   packet/context bridge so OxFml can distinguish OxFunc function surfaces from
   OxCalc host references without either project retaining hidden context?
10. What host running-context version fields should invalidate formula binding
    when UDF registration, function capability, or TreeCalc namespace facts
    change?
11. How should runtime callable specialization be represented so `MyNode(12)`
    is structurally a reference invocation, but calculation can later bake in
    the referenced node's callable `CalcValue` when available?

## 6. First Execution Slices

1. Evidence slice:
   write or identify tests showing DNA OneCalc/OxFml live behavior for direct
   non-`=` entries: number, boolean, text, apostrophe-forced text, quoted text,
   empty entry, and an error literal if admitted.
2. OxFml surface slice:
   define and test the first authored-input result enum:
   `CalcValue`, `BoundFormula`, or diagnostics. This slice must prove OxCalc
   can receive `BoundFormula` after OxFml reference binding and use it for
   dependency-tree construction.
3. OxCalc input slice:
   replace local string conversion in `treecalc.rs`, `repository.rs`, and any
   coordinator/replay helpers with OxFml-owned cell-entry interpretation.
   Formula acceptance diagnostics are rejection results, not accepted cell
   values: an authored formula such as `=1+` must leave the node input snapshot
   unchanged and surface `AuthoredInputDiagnostics` (or the API equivalent)
   rather than storing formula text whose evaluation later produces `#VALUE!`
   or another worksheet error. Partial/unparsable strings may exist in formula
   editing surfaces, but they must not pass the formula acceptance boundary.
4. Typed-input slice:
   add or clarify the API path for callers that already hold a `CalcValue`.
   Typed inputs should not be routed through text just to reach the value table.
5. Callable slice:
   confirm that a node assigned `=LAMBDA(...)` records a callable
   `CalcValue`/RichValue result and retains the OxFml prepared callable package
   needed for `=MyNode(12)` style invocation from other formulas.
6. Host-resolution interface slice:
   replace the current ad hoc combination of `BindContext.names`,
   host-reference syntax packets, direct-name carrier probes, and static
   OxFunc metadata lookup with an explicit bind-time interface: OxFml consults
   OxFunc for function surfaces using the host running context, consults OxCalc
   for host references using owner/source/snapshot context, and returns
   `BoundFormula` with bound reference/invocation facts that OxCalc can map to
   dependency descriptors.
7. Runtime callable specialization slice:
   add canonical cases for `=MyNode(12)` where `MyNode` evaluates to a callable
   and where it evaluates to a non-callable. Both cases must share the same
   structural dependency path; only the calc-time value check differs.

## 7. Exit Gate

W059 exits its design-prep scope when:

1. OxCalc no longer owns string-to-typed-value parsing for authored cell-entry
   text;
2. non-`=` literal text and leading-`=` formula text enter one OxFml-authored
   input interpretation path;
3. formula entries yield `BoundFormula` after formula-visible reference
   resolution, and OxCalc uses `BoundFormula` for dependency-tree work rather
   than inspecting raw formula text;
4. typed API inputs have an explicit `CalcValue` path that does not depend on
   text round-tripping;
5. node assigned callable values have a documented value/artifact retention
   shape across OxFunc, OxFml, and OxCalc;
6. function namespace lookup is host-running-context based, including UDF
   registration, with no hidden OxFunc global current-registry state;
7. host-reference lookup is explicit and produces bound reference/invocation
   facts rather than dependency descriptors;
8. callable node invocation is validated at calculation time after dependency
   ordering has made referenced node values available;
9. tests cover the admitted literal/formula/callable cases and the remaining
   unsupported cases have typed diagnostics or explicit follow-up beads.

## 7A. Current Implementation Checkpoint

As of the current OxCalc migration slice:

1. OxFml exposes the authored-input branch shape as literal `CalcValue`,
   formula `BoundFormula`, or diagnostics.
2. OxCalc routes public string input through OxFml authored-input
   interpretation and rejects diagnostic formula acceptance without mutating the
   stored node input.
3. Publication and coordinator candidate values are `CalcValue`-first, with
   string `published_values` retained only as display/replay projection.
4. Dependency construction no longer runs an OxCalc-local parse/bind probe for
   context formulas. OxCalc resolves TreeCalc host-reference packets, asks OxFml
   for the accepted formula `BoundFormula`, and derives remaining unresolved
   name/function-call facts from that returned artifact.
5. The remaining non-closed W059 gate is the final replacement of the legacy
   `TreeFormulaCatalog` lowering container with direct `BoundFormula`
   consumption or OxFml helper queries for the dependency descriptor build.

## 8. Relationship To Existing Worksets

W059 depends on W050 and W057 doctrine:

1. W050 established that OxFml owns formula grammar, parse, bind, evaluator
   session, and artifact meaning.
2. W057 established the durable snapshot/derived-layer split for structure,
   node input, namespace, formula binding, dependency shape, publication, and
   runtime overlays.

W059 narrows a residual contradiction exposed by W098/W057 work: literal
string interpretation is part of the formula/value semantic boundary and should
not remain as an OxCalc-local helper just because the input text lacks a
leading `=`.
