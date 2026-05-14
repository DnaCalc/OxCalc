# HANDOFF-CALC-003 OxCalc Receipt and Integration Note

## Purpose
Record the OxCalc-side receipt of OxFml's acknowledgment for
`HANDOFF-CALC-003` and summarize the local integration consequences for W050
correctness-floor selector, numerical reduction, and error-algebra work.

This note records receiving-repo awareness and migration consequences. It does
not close the remaining OxCalc dependency on OxFunc kernel metadata, exact
selector semantics, or canonical invalidation/version surfaces.

## Receiving-Side Sources
1. `../OxFml/docs/handoffs/HANDOFF_CALC_003_OXFML_RECEIPT.md`
2. `../OxFml/docs/handoffs/HANDOFF_REGISTER.csv`
3. `../OxFml/docs/spec/OXFML_CANONICAL_ARTIFACT_SHAPES.md`
4. `../OxFml/docs/spec/OXFML_MINIMUM_SEAM_SCHEMAS.md`
5. `../OxFml/docs/spec/OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`
6. `../OxFml/docs/spec/formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
7. `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`

## Verified OxFml Response Summary
OxFml acknowledged the handoff with decision `adapt_and_split_by_owner`.

Accepted OxFml canonical direction:
1. admit a replay-visible `CorrectnessFloorContext` or equivalent profile
   context carrying `profile_version`, `numerical_reduction_policy`, and
   `error_algebra`,
2. treat those selectors as semantic evaluation state rather than optimization
   hints,
3. thread the context through semantic-plan, runtime/session, prepared-call
   trace, and replay projection surfaces where relevant,
4. make selector mismatch replay-invalid unless a migration proof is attached.

Preferred OxFml field family:
1. `CorrectnessFloorContext`,
2. `numerical_reduction_policy`,
3. `error_algebra`,
4. `profile_version`.

OxFml explicitly split the broader packet into a two-owner replacement plan:
1. OxFml owns context carriage and replay identity.
2. OxFunc owns kernel metadata, selector semantics, and invalidation
   versioning.

## OxCalc Integration Consequences
OxCalc should keep W050 correctness-floor selector artifacts as local replay
hooks and compatibility evidence until OxFml/OxFunc expose canonical
`CorrectnessFloorContext` and kernel metadata/version surfaces.

OxCalc should not claim that pairwise or compensated numerical policies, error
algebra precedence, or affected-function invalidation are enforced by current
kernels. Current OxCalc selector diagnostics prove selection and replay
mismatch detection only.

## Remaining Open Lanes
1. OxFunc acknowledgment for metadata and kernel obligations,
2. concrete replay fields for non-left-fold policies,
3. prepared-package invalidation signal for selector behavior changes,
4. OxCalc migration from local `CorrectnessFloorReplayRecord` compatibility
   fields to canonical OxFml replay fields.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - OxFunc metadata and kernel acknowledgment
  - canonical replay field names for non-left-fold policies
  - selector-behavior invalidation version signal
  - OxCalc migration from local selector replay artifacts
