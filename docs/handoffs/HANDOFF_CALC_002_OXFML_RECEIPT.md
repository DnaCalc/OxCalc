# HANDOFF-CALC-002 OxCalc Receipt and Integration Note

## Purpose
Record the OxCalc-side receipt of OxFml's acknowledgment for
`HANDOFF-CALC-002` and summarize the local integration consequences for W050
session, prepared-callable, plan-template, hole-binding, formal-reference, and
trace/replay work.

This note records receiving-repo awareness and migration consequences. It does
not close the remaining OxCalc implementation dependency on canonical runtime
and replay fields.

## Receiving-Side Sources
1. `../OxFml/docs/handoffs/HANDOFF_CALC_002_OXFML_RECEIPT.md`
2. `../OxFml/docs/handoffs/HANDOFF_REGISTER.csv`
3. `../OxFml/docs/spec/OXFML_CANONICAL_ARTIFACT_SHAPES.md`
4. `../OxFml/docs/spec/OXFML_MINIMUM_SEAM_SCHEMAS.md`
5. `../OxFml/docs/spec/OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`
6. `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`

## Verified OxFml Response Summary
OxFml acknowledged the handoff with decision
`adapt_and_promote_narrower_successor_plan`.

Accepted OxFml canonical direction:
1. the public runtime/session facade is the ordinary OxCalc surface,
2. OxFml should expose prepared formula identity without private binder access,
3. prepared identity should split package identity, plan-template identity, and
   binding identity,
4. formal reference/input transport should replace synthetic A1 and defined-name
   compatibility inputs,
5. managed-session result paths should expose or link to the same
   coordinator-relevant truth as one-shot execution,
6. candidate, commit, reject, trace, returned-value, template, and hole facts
   should be structured fields rather than diagnostic strings,
7. bind-visible metadata changes need invalidation signals,
8. compile-time folding and template reuse should be OxFml-owned trace or
   identity facts if admitted.

Preferred OxFml terminology:
1. `PreparedFormulaPackage` or equivalent for OxCalc `PreparedCallable`,
2. `PlanTemplate` for reusable plan-template identity,
3. `HoleBindingSet` for OxCalc `HoleBindings`,
4. `FormalReference` / `FormalReferenceSet` for canonical reference/input
   transport.

## OxCalc Integration Consequences
OxCalc should treat the W050 local `PreparedCallable`, `PlanTemplate`,
`HoleBindings`, formal-reference bridges, diagnostic correlation strings, and
plan-template reuse diagnostics as compatibility projections until OxFml emits
the canonical runtime/replay fields.

OxCalc should not add a private long-lived adapter around OxFml internals. The
next migration point is the public runtime/session facade fields accepted in
OxFml's narrower successor plan.

## Remaining Open Lanes
1. canonical prepared package, plan template, hole binding, and formal-reference
   fields are not yet implemented in the consumed OxFml runtime facade,
2. OxFunc metadata/version cooperation is still required for targeted
   argument-preparation invalidation,
3. OxCalc migration away from W050 compatibility fingerprints waits until the
   canonical fields land.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - canonical OxFml runtime/replay field implementation
  - OxFunc metadata/version cooperation for argument-preparation invalidation
  - OxCalc migration from W050 compatibility projections
