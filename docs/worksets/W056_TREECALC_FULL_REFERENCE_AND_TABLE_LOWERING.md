# W056 TreeCalc Full Reference And Table Lowering

Status: `queued_successor`

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

Product status: queued successor. W051 implements the first `ChildrenV1`
carrier pattern locally; W056 is the registered place for the full TreeCalc
reference family and table-lowering product scope.

Evidence: W051 focused tests cover the first carrier's local
membership/member-value dependency descriptors, reference-preserving formal
input binding, and membership/order invalidation facts.

Still open: all W056 lanes above.

Formal status: no proof claim.
