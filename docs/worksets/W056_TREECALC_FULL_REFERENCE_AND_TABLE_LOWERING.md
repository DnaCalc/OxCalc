# W056 TreeCalc Full Reference And Table Lowering

Status: `in_progress`

Parent predecessor: `W051` first reference-carrier pattern

Parent epic: `calc-4vs8`.

Initial successor beads:

1. `calc-4vs8.1` — TreeReference variant inventory and host-reference correlation.
2. `calc-4vs8.2` — structured table dependency lowering.
3. `calc-4vs8.3` — dependency invalidation and dynamic rebind widening.

## 1. Purpose

W056 widens the W051 first `ChildrenV1` carrier into the full TreeCalc
reference and table-lowering scope.

W051 proves the first pattern: an OxCalc-owned reference collection can be
correlated to an OxFml host-reference handle, lowered into membership and
member-value dependency facts, invalidated on membership/order changes, and
transported as an opaque `ReferenceLike` without OxFunc learning TreeCalc
syntax.

W056 owns the broader closure: all admitted TreeCalc reference variants,
dynamic rebinding, host namespace versioning, caller-context identity, and
structured table row/column/header/totals dependency lowering.

## 2. Scope

In scope:

1. TreeCalc reference variants beyond `ChildrenV1`, including ancestor,
   sibling, preceding/following, explicit path, dynamic, unresolved,
   host-sensitive, capability-sensitive, cross-workspace, and future
   recursive/set-producing selectors.
2. Dependency descriptors and reverse edges for scalar references,
   set-membership references, member-value references, dynamic rebind
   outcomes, unresolved references, and host-sensitive references.
3. Invalidation facts for namespace mutation, caller-context mutation,
   dynamic rebind, base deletion, membership change, order change, member value
   publication, cross-workspace availability, and capability denial.
4. Host namespace versioning and caller-context identity as prepared-identity
   inputs.
5. Correlation of OxCalc reference carriers to OxFml host-reference handles
   and formal-reference/runtime-input records.
6. Structured table lowering using the converged first packet:
   `table_catalog`, `enclosing_table_ref`, and `caller_table_region`.
7. Table dependencies for row membership, column identity/range,
   header-region identity, totals-row identity, `#This Row` caller-region
   facts, and omitted-table-name enclosing-table facts.

Out of scope:

1. OxFml grammar or parser ownership.
2. OxFunc worksheet function semantics or TreeCalc-specific function branches.
3. DNA TreeCalc product UI, persistence, or orchestration.
4. Private OxFml/OxFunc shims or TreeCalc-only evaluator bridges.

## 3. Ownership

OxCalc owns TreeCalc model custody, reference resolution, dependency edges,
invalidation facts, dynamic rebind policy, host namespace versioning, caller
context identity, and table lowering into calculation dependencies.

OxFml owns formula grammar, generic host formula context consumption,
structured-reference grammar and bind normalization, name/call precedence, and
canonical evaluator/runtime artifacts.

OxFunc owns values, references as function inputs, worksheet function
semantics, and argument-admission metadata.

Integration rule: OxCalc talks to OxFml through generic `HostFormulaContext`
and public runtime/replay surfaces only. W056 must not add OxFml-private
bridges, OxFunc TreeCalc semantics, or parser shims in host repos.

## 4. Initial Lanes

1. full TreeReference variant inventory and compatibility matrix,
2. reference carrier to OxFml host-reference correlation contract,
3. dependency descriptor and reverse-edge widening,
4. invalidation fact widening,
5. dynamic rebind and host namespace versioning,
6. caller-context identity and relative-reference replay,
7. cross-workspace reference availability and degradation,
8. structured table packet intake,
9. row/column/header/totals dependency lowering,
10. W074/W036 OxFml evidence intake and handoff watch,
11. end-to-end TreeCalc reference/table scenarios.

## 4A. `calc-4vs8.1` Implementation-Input Surface

The first W056 tranche turns the full TreeCalc reference inventory into typed
OxCalc-owned implementation inputs in
`src/oxcalc-core/src/formula.rs`:

1. `TreeReferenceInventoryVariant` names the admitted and blocked reference
   families, including current concrete carriers, set-producing selectors,
   cross-workspace references, structured table references, and bare
   name/callable references.
2. `TreeReferenceImplementationInput` records, per variant, whether the
   family is a current carrier, an admitted implementation input, or a typed
   exclusion; the host-reference correlation need; namespace and caller-context
   identity needs; dependency descriptor facts; invalidation facts; and any
   successor bead.
3. Existing `TreeReference` values map back to this inventory through
   `TreeReference::inventory_variant()` and
   `TreeReference::implementation_input()`.

Current admitted implementation inputs are not a product-complete full
TreeCalc reference claim. They are the typed work inputs for the remaining
W056 beads. In particular:

1. structured table references are a typed exclusion linked to
   `calc-4vs8.2`,
2. dependency/reverse-edge, dynamic rebind, namespace, and caller-context
   widening continues in `calc-4vs8.3`,
3. formula-call registry-view admission and capability-denied runtime
   classification have landed in OxFml W074 `fml-ds0.7`; final bare
   name/callable precedence remains blocked on the broader W074 Excel oracle
   matrix,
4. cross-workspace and recursive selectors remain typed exclusions until the
   workspace availability and selector dependency models are specified and
   exercised.

## 4B. `calc-4vs8.2` Structured Table Lowering Surface

The second W056 tranche adds the first OxCalc-owned typed structured table
dependency-lowering surface in `src/oxcalc-core/src/structured_table.rs`.

Current implemented scope:

1. consumes only public OxFml table-context packet types:
   `table_catalog`, `enclosing_table_ref`, `caller_table_region`, stable row
   membership/order identities, and exact header/totals region refs,
2. accepts public OxFml `StructuredReferenceBindRecord` packets and maps them
   into normalized `StructuredTableReferenceIntake` values rather than parsing
   formula text or mirroring structured-reference grammar,
3. lowers available facts for table identity, stable row membership, stable row
   order, selected column identity, header text, exact header region, data
   region, exact totals region, caller row context for `#This Row`, and
   omitted-table-name enclosing-table dependency,
4. preserves table dependencies as context-only dependency descriptors so the
   dependency graph retains them without inventing TreeNodeId reverse edges,
5. records typed blockers only when optional packet facts are absent or when the
   packet states that the requested header/totals/caller row context does not
   exist.

Current non-claim:

This is an implemented OxCalc intake/lowering surface for the current generic
packet, including the stable table fact fields added by OxFml `fml-ds0.8` and
the exercised normalized structured-reference bind packets added by OxFml
`fml-ds0.9`. It is not full structured table behavior. Full behavior remains
blocked until DnaTreeCalc/OxReplay retained table evidence runs through the
real bridge and the remaining W056 cross-workspace/name/selector surfaces are
exercised.

## 4C. `calc-4vs8.3` Dependency, Invalidation, And Rebind Surface

The third W056 tranche adds the first OxCalc-owned typed projection over the
dependency graph for the admitted TreeReference carrier families in
`src/oxcalc-core/src/tree_reference_rebind.rs`.

Current implemented scope:

1. projects existing `DependencyGraph` descriptors into typed W056 descriptor
   facts with source-reference handle, target, descriptor kind, namespace
   identity need, caller-context identity need, invalidation facts, and
   prepared-identity invalidation consequence,
2. exposes target reverse-edge facts for concrete `TreeNodeId` dependencies
   and context reverse-edge facts for retained context-only descriptors such
   as structured table and runtime-fact carriers,
3. exposes dynamic rebind facts that distinguish unresolved dynamic potential
   from resolved dynamic target edges and list the activation, release,
   reclassification, and upstream-publication invalidation facts,
4. groups host namespace, structure-context, resolution-rule,
   capability-profile, table-context, cross-workspace, and caller-context
   identity requirements as prepared-identity invalidation inputs,
5. preserves cross-workspace references as a typed blocker requiring a
   versioned cross-workspace availability/degradation model rather than
   inventing an executable fallback.

Current non-claim:

This is an implemented typed OxCalc surface over current descriptors and graph
facts. It is not full runtime behavior for every W056 carrier. End-to-end
runtime closure remains blocked where the cross-repo program has not yet
emitted exercised generic host-reference, final name/call precedence,
cross-workspace oracle packet surfaces, selector dependency models, or
retained full-bridge evidence. The bounded W074 registry/capability slice is
no longer missing: OxFml `fml-ds0.7` at commit `9da8456` proves runtime
registry-view formula-call admission and capability-denied classification, and
OxFml `fml-ds0.9` at commit `6895e6a` proves normalized structured-reference
bind packet projection, but neither freezes built-in/UDF/defined-name or
defined-name-`LAMBDA` precedence or TreeCalc name/call semantics.

## 4D. `calc-4vs8.6` Runtime Prepared-Identity Contribution

The fourth W056 tranche consumes the typed dependency/rebind identity needs
during local TreeCalc runtime preparation in `src/oxcalc-core/src/treecalc.rs`.

Current implemented scope:

1. derives W056 namespace and caller-context identity needs from the translated
   TreeCalc reference carriers and runtime-fact carriers before calling the
   public OxFml runtime prepare path,
2. projects host namespace, resolution-rule, capability-profile, structure
   context, caller-context, table-context, and cross-workspace availability
   identity inputs through OxFml's public `RuntimeHostFormulaContext` where the
   current public surface can carry them,
3. keeps the first `ChildrenV1` host-reference bind results and sparse
   reference-values path on the same public OxFml runtime surface,
4. records W056 prepared-identity requirement diagnostics so the runtime path
   exposes which namespace/caller-context classes contributed,
5. includes the OxFml `prepared_formula_key` in the local per-edge value-cache
   call-site key so host-context and prepared-package identity changes cannot
   reuse stale cached values under the same plan-template/hole-binding pair,
6. preserves cross-workspace as a typed projected identity input only; no
   executable cross-workspace fallback is invented.

Current non-claim:

This is runtime prepared-identity/cache invalidation for the carriers that
OxCalc can currently project through public OxFml runtime context fields. It is
not full W056 closure. The stable structured-table packet facts added by OxFml
`fml-ds0.8` are now consumed by the OxCalc lowering surface; broader upstream
packet/oracle surfaces, including final name/call precedence, exercised
normalized structured-reference bind packets, and public cross-workspace
availability/degradation semantics, remain blocked by `calc-4vs8.5`. The
registry-view/capability-denial portion of that upstream dependency is now
evidenced by OxFml `fml-ds0.7` and should not be treated as an open blocker.

## 4E. `calc-4vs8.7` Raw Formula-Text Children Prebind Surface

The fifth W056 tranche adds an OxCalc-owned public prebind surface in
`src/oxcalc-core/src/formula.rs` for the first DnaTreeCalc raw formula-text
pressure point.

Current implemented scope:

1. `prebind_treecalc_formula_text(owner_node_id, source_text)` accepts original
   TreeCalc formula text and returns a `TreeFormula` suitable for the existing
   OxFml runtime path,
2. recognizes free-standing `@CHILDREN` and `.*` as explicit host references
   whose base is the formula owner/caller context,
3. rewrites only the OxFml-submitted formula source to neutral formal tokens
   such as `TREE_REF_<owner>_<n>`,
4. preserves the exact source token text and UTF-8 span on
   `TreeCalcChildrenReferenceCollection`,
5. emits `TreeFormula::opaque_oxfml` with
   `TreeFormulaReferenceCarrier::named` carrying
   `TreeCalcReferenceCollection::ChildrenV1`,
6. preserves the existing TreeCalc host-context identities and the existing
   public OxFml sparse reference-values path,
7. returns typed diagnostics for unsupported raw TreeCalc reference families
   and for qualified `base.@CHILDREN` / `base.*` syntax instead of guessing a
   name/path precedence rule.
8. `prebind_treecalc_formula_text_with_resolved_bases(...)` admits qualified
   `base.@CHILDREN` / `base.*` when the caller supplies an exact
   UTF-8-span-keyed resolved-base packet with `base_node_id`, base span,
   selector span, resolution layer, and resolution identity.

Current non-claim:

This is not a full raw TreeCalc formula parser. The qualified children path
does not resolve raw `base` text; it consumes a typed resolved base supplied by
the caller or a future OxCalc-owned resolver. Recursive/sibling/preceding/
following selectors, structured table references, full explicit path
resolution, and bare name/callable precedence remain W056/W074 successor scope
until they can be resolved through typed caller-supplied path and namespace
surfaces.

Qualified children blocker:

`calc-4vs8.8` closes the narrow blocker by adding the caller-supplied resolved
base contract. The default `prebind_treecalc_formula_text(...)` still receives
only owner node and formula text, so it continues to reject qualified syntax.
The new resolved-base entry point lets an upstream caller or future OxCalc
path resolver provide the exact source span and stable base `TreeNodeId`
without freezing broader TreeCalc name/path precedence or asking a host repo to
mirror OxCalc reference semantics. Full OxCalc-owned explicit path resolution
over a pinned structural snapshot remains successor W056 scope.

## 5. Closure Gate

W056 closes only for a declared full-reference/table-lowering scope when:

1. admitted TreeCalc reference variants have implemented carriers or explicit
   typed exclusions,
2. dependency and invalidation facts are replay-visible and tested for each
   admitted variant,
3. dynamic rebind and host namespace version changes produce deterministic
   prepared-identity invalidation,
4. caller-context-sensitive references are exercised with stable
   caller-context identity,
5. structured table row/column/header/totals dependencies are lowered from the
   generic OxFml table packet without OxCalc parsing formula language,
6. OxFml/OxFunc integration remains through public generic host-context and
   reference/value surfaces,
7. known exclusions and any new cross-repo handoffs are explicit.

## 6. Status

Product status: in progress through `calc-4vs8.8`. W051 is closed for the first
OxCalc `ChildrenV1` carrier pattern; W056 now has a typed Rust
implementation-input inventory for the broader reference family, a first
structured table-context dependency-lowering surface for the current generic
OxFml table packet, and a typed dependency/reverse-edge/invalidation/rebind
projection over current OxCalc graph facts. Runtime preparation now consumes
the typed W056 identity needs through public OxFml `RuntimeHostFormulaContext`
fields where available, and the local edge-value cache includes the resulting
prepared formula identity in its call-site key. OxCalc now also exposes a
public raw TreeCalc formula-text prebind for free-standing `@CHILDREN` and
`.*`, plus a caller-supplied resolved-base prebind contract for
`base.@CHILDREN` and `base.*`, producing a neutral OxFml source plus a
source-preserving `ChildrenV1` carrier for the existing OxFml/OxFunc path.
This is not a full-reference/table-lowering product claim.

Evidence: W051 focused tests cover the first carrier's local membership/member
value dependency descriptors, reference-preserving formal input binding,
OxFml sparse reference-values binding, OxFunc aggregate consumption, and
membership/order invalidation facts. `calc-4vs8.1` adds focused Rust tests for
the W056 inventory, concrete `TreeReference` mapping, and `ChildrenV1`
handle/dependency/invalidation correlation facts. `calc-4vs8.2` adds focused
Rust tests for available table fact lowering, typed blocker emission for missing
row/region packet facts, graph retention of context-only descriptors, and
omitted-table-name enclosing context failure. `calc-4vs8.3` adds focused Rust
tests for target reverse-edge facts, context reverse-edge facts,
namespace/caller-context prepared-identity inputs, dynamic potential versus
resolved dynamic rebind facts, and cross-workspace typed blocker preservation.
`calc-4vs8.6` adds focused Rust tests for host namespace and caller-context
prepared-key changes, capability-profile prepared-key changes, table-context
and cross-workspace public host-context projection, and prepared-formula-key
participation in the local edge-value cache key. `calc-4vs8.7` adds focused
Rust tests proving `=SUM(@CHILDREN)` and `=SUM(.*)` prebind to neutral
`TREE_REF_*` OxFml source, preserve source token text/spans, produce
`ChildrenV1` carriers, reject unsupported raw TreeCalc reference families, and
execute end-to-end through the existing OxCalc/OxFml/OxFunc reference path.
`calc-4vs8.8` adds focused Rust tests proving `=SUM(base.@CHILDREN)` and
`=SUM(base.*)` can prebind through exact source-span-keyed resolved-base
packets, preserve qualified token text/spans, bind to the supplied base
`TreeNodeId`, reject qualified syntax without a matching resolved base, and
execute end-to-end through the same reference-preserving path.
`calc-4vs8.4` consumes OxFml `fml-ds0.8` stable table packet facts and adds
focused Rust coverage proving row membership, row order, exact header region,
and exact totals region lower as context dependency descriptors when supplied,
while typed blockers remain for legacy packets where those optional facts are
absent.
`calc-4vs8.9` consumes OxFml `fml-ds0.9` structured-reference bind packets and
adds focused Rust coverage proving explicit table, omitted table-name,
`#This Row`, selected section/region, selected column, handle-correlation, and
diagnostic-bearing unresolved records map into OxCalc table lowering without
formula-text parsing. Omitted table-name packets preserve OxFml's bound
effective table id as the lowering target and separately validate the enclosing
table context, surfacing a typed blocker if those packet facts disagree.

Still open: W074 final name/call precedence evidence, exercised OxFml
host-reference packets beyond the admitted children/table slices, a versioned
cross-workspace availability/degradation model, selector dependency models for
recursive and sibling/preceding/following set selectors, full OxCalc-owned
explicit path resolution for raw `base` text, DnaTreeCalc receiving-side corpus
activation for the new OxCalc prebind surface and table packets, and broader
end-to-end scenarios. Blocker `calc-4vs8.5` remains open after
`calc-4vs8.9` closure.

Formal status: no proof claim.
