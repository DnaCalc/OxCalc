*Posted by Codex agent on behalf of @govert*

# HANDOFF-CALC-005: Generic Host Formula Context And Namespace Resolution For W051

## Purpose
This handoff packet requests canonical OxFml-side support for W051's generic
host formula context and name/call resolution model.

The goal is not to add a TreeCalc parser mode to OxFml. The goal is to let
OxFml keep formula grammar, lexical binding, call structure, and prepared
identity ownership while consuming an OxCalc-supplied host context for
reference/name syntax. OxCalc remains the custodian of the TreeCalc model and
resolves host references against that model. OxFunc remains opaque to TreeCalc
syntax and structure.

## Source Scope
- Source workset: `W051_SPARSE_RANGE_READERS_AND_DEFINED_ENTRY_SEMANTICS`
- Driving local spec surfaces:
  - `docs/worksets/W051_SPARSE_RANGE_READERS_AND_DEFINED_ENTRY_SEMANTICS.md`
    section 3.1
  - `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md` section 10.2
  - `docs/spec/core-engine/CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md`
    section 2.1
  - `docs/upstream/NOTES_FOR_OXFML.md` section 81
- Supporting downstream surfaces:
  - `../DnaTreeCalc/docs/model/CORE_MODEL_SPEC.md` sections 3.8, 3.9, 4, and 6
  - `../DnaTreeCalc/docs/handovers/HANDOVER_OXFML_lambda_node_invocation.md`
  - `../DnaTreeCalc/docs/handovers/HANDOVER_OXFML_constant_input.md`
- OxFml references reviewed read-only:
  - `../OxFml/docs/worksets/W068_canonical_function_registry_consumption_cleanup.md`
  - `../OxFml/docs/worksets/W074_registry_mutation_and_name_resolution_invalidation.md`
  - `../OxFml/docs/IN_PROGRESS_FEATURE_WORKLIST.md`

## Current Compatibility Position
W051 now has a concrete cross-repo pressure case:

1. TreeCalc formula text such as `=SUM(@CHILDREN)` must be parsed and bound by
   OxFml, not by DNA TreeCalc.
2. OxCalc supplies the active host formula context, owns the TreeCalc model,
   and resolves host reference/name bind artifacts against that model.
3. OxFml should not know the semantics of `@CHILDREN`, node walk-up, sibling
   selectors, or set membership. It should call a host reference/name interface
   and carry the resulting opaque bind objects with source spans and replay
   identity.
4. Function/name lookup must account for OxFunc built-ins, registered UDFs,
   workbook/sheet defined names, and defined-name `LAMBDA` invocation.
5. The exact shadowing order is not frozen by this packet. OxFml must match
   observed Excel behavior before promoting product semantics, then expose the
   resulting resolution layer and invalidation consequences.
6. A host name may still be callable when it resolves to a single
   lambda-valued node, but its admission and precedence must be mapped against
   the Excel-defined-name/LAMBDA behavior or explicitly documented as a
   TreeCalc extension.
6. Current OxCalc code has no final reference-collection carrier, no
   set-membership dependency edge, and scalarizes current synthetic bindings
   before OxFml can preserve reference identity. Those are OxCalc-side W051
   gaps, not reasons for OxFml to grow TreeCalc-specific semantics.

## Requested Canonical OxFml Clauses

### 1. Generic Host Formula Context
OxFml should accept a host formula context, supplied by the consumer, for
formula channels whose reference/name surface is not native WorksheetA1.

Requested fields or equivalent accessors:

1. `dialect_id` / `capability_profile_id`
2. reference-expression parser hook for host reference syntax in operand and
   callee-prefix positions
3. host namespace resolver for names, paths, selectors, defined names, and
   host-sensitive references
4. function registry view backed by OxFunc built-ins and registered UDFs
5. caller context needed for relative references and lexical walk-up
6. context/version identity suitable for prepared-formula cache keys and replay

Requested clause direction:

1. OxFml owns the surrounding formula grammar: calls, argument lists,
   operators, literals, arrays, `LET`, `LAMBDA`, and formula-source spans.
2. The host hook owns only the host reference/name surface and returns opaque
   syntax/bind objects. OxFml does not inspect TreeCalc selectors or model
   structure.
3. The context identity participates in prepared callable / semantic-plan
   identity when it can affect binding.

### 2. Name And Call Resolution Hierarchy
OxFml should document and implement a stable resolution hierarchy for bare
names and call callees in host-context formulas. The hierarchy must be
Excel-oracle-derived for built-in functions, registered UDFs, workbook/sheet
defined names, and defined-name `LAMBDA` invocation.

Requested hierarchy constraints:

1. OxFml-owned special forms and lexical bindings keep their existing
   precedence.
2. Bare call position follows the Excel-matched shadowing order across
   built-ins, registered UDFs, defined names, and defined-name lambdas.
3. Only when that Excel-matched order admits a host-name lane does OxFml ask
   the host namespace whether the callee resolves to a callable host reference.
4. Non-call bare-name behavior follows Excel where Excel defines a function,
   UDF, or defined-name distinction; TreeCalc host names then map to the
   defined-name-like lane unless a documented extension says otherwise.
5. Explicit host-reference syntax and explicit paths bind through the host
   namespace and can select a node whose name collides with a function or UDF.
6. Ambiguity, capability denial, unknown function, unresolved host name, and
   set-reference-used-as-callable failures are typed bind diagnostics, not
   runtime guesses.
7. The Excel oracle matrix and the chosen resolution rule participate in
   replay/spec evidence before this becomes product semantics.

### 3. Host Reference Bind Output
OxFml should expose host-reference bind output through a stable prepared/bind
artifact shape, or an equivalent public facade shape.

Requested fields:

1. host reference handle / formal reference id
2. source span and source text or source token identity
3. active `dialect_id` / `capability_profile_id`
4. opaque host selector payload supplied by the host resolver
5. resolution layer (`function`, `host_name`, `explicit_host_ref`, `lexical`,
   `unresolved`, or equivalent)
6. shape hint (`single`, `collection`, `dynamic`, `unknown`, or equivalent)
7. caller-context-dependent flag
8. replay-visible diagnostics when the reference is denied or unresolved

OxCalc will own final lowering from those artifacts into TreeCalc references,
dependency descriptors, set-membership invalidation, and runtime
dereference/iterator behavior.

### 4. Runtime Reference Transport
OxFml should preserve a path for host references to reach function invocation
as references when function metadata permits reference visibility.

Requested clause direction:

1. host references may materialize to values for values-only calls,
2. reference-sensitive or reference-preserving calls receive an opaque
   `ReferenceLike` plus resolver/reader authority,
3. OxFunc never parses host reference text or TreeCalc selector payloads,
4. eager value-array materialization is acceptable only as an explicitly
   temporary fallback and does not close the W051 reference-preserving
   scenario.

### 5. Registry And Cache Invalidation
This handoff should align with OxFml W074.

Requested clause direction:

1. the function registry view identity participates in formula-call bind
   identity,
2. UDF registration, unregistration, and capability overlay changes invalidate
   any cached bind result whose call/name classification may change,
3. host context identity and caller context participate in bind identity when
   host name/reference resolution can change,
4. replay can distinguish a formula changed by function-registry mutation from
   one changed by host namespace/model mutation.

## Migration And Fallback Impact
1. DNA TreeCalc's temporary prepared-formula smoke carrier remains temporary
   and should not become a production parse/bind contract.
2. OxCalc can stage an eager materialization fallback for `SUM(@CHILDREN)`,
   but that fallback does not close W051 unless the reference-preserving path
   is also specified or explicitly deferred.
3. Existing WorksheetA1 parsing and strict-Excel channels should remain
   unchanged.
4. No OxFunc TreeCalc-specific change is requested. A new OxFunc reference
   kind should be considered only if the existing opaque reference/resolver
   model cannot safely carry host references.

## Open Questions For OxFml
1. Should the host reference hook operate at tokenization/parse time, bind
   time over generic name/path syntax, or both?
2. Which public consumer facade should carry the host formula context and its
   version identity?
3. Can existing defined-name callable transport be generalized for callable
   host references, or is a distinct callable-host-reference carrier needed?
4. How should explicit host-reference syntax be represented in the generic AST
   so that a node/function collision can be resolved intentionally?
5. What does Excel do for collisions across built-in functions, registered
   UDFs, workbook/sheet defined names, and defined-name lambdas in bare call
   and non-call positions?
6. Which W074 UDF-vs-defined-name precedence tests should be widened to cover
   host namespaces supplied by OxCalc?

## Requested Disposition
Please review against OxFml's current registry, name-resolution, callable, and
consumer-runtime worksets and determine:

1. which requested clauses can be promoted into OxFml canonical spec text,
2. which pieces should land under W074 versus a new host-context workset,
3. the Excel oracle cases OxFml will use to settle built-in/UDF/defined-name/
   defined-name-LAMBDA shadowing before freezing the binder contract,
4. the preferred public type names for the context, bind output, and
   diagnostics,
5. any OxCalc-side shape changes needed before W051 implementation starts.

## OxCalc Receipt Intake And Local Disposition
OxFml receipt
`../OxFml/docs/handoffs/HANDOFF_CALC_005_OXFML_RECEIPT.md` accepts this
direction as `accept_as_w051_plan_with_w074_evidence_gate`.

OxCalc local disposition:

1. `dialect_id = oxcalc.treecalc-v1`,
   `capability_profile_id = host-capabilities:treecalc-v1`, and
   `resolution_rule_version = treecalc-host-resolution:v1` are the first W051
   host-context identity inputs.
2. The first local reference carrier is implemented as
   `TreeCalcReferenceCollection::ChildrenV1`.
3. OxCalc now lowers that carrier into collection-membership and current-member
   value dependency descriptors, preserves the runtime formal input as an
   opaque structured `ReferenceLike`, and emits local membership/order
   invalidation facts.
4. End-to-end `SUM(@CHILDREN)` through generic OxFml host-context parsing and
   OxFml/OxFunc resolver/admission remains open W051 work.
5. Full TreeCalc reference families and structured table lowering are routed
   to successor W056, not added to this handoff.
