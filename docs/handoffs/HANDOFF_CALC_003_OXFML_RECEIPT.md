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
8. `../OxFunc/docs/handoffs/HANDOFF-CALC-003_OXFUNC_RECEIPT.md`
9. `../OxFunc/docs/function-lane/OXFUNC_KERNEL_METADATA_AND_ADMISSION_PROFILE_CONTRACT.md`

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

OxFunc has acknowledged its side of the split. OxFunc accepts ownership of
reduction-sensitive and error-collapse-sensitive kernel metadata, exact
`NumericalReductionPolicy` semantics, exact `ErrorAlgebra` semantics, and the
initial affected-function family review set. OxFunc reserves
`semantic_kernel_metadata_version` as the prepared-package invalidation signal.

OxFml has consumed that OxFunc response and accepts
`semantic_kernel_metadata_version` as the OxFunc-owned invalidation bridge that
OxFml runtime/replay artifacts should consume once the field is emitted from a
real metadata source.

## OxCalc Integration Consequences
OxCalc should keep W050 correctness-floor selector artifacts as local replay
hooks and compatibility evidence until OxFml/OxFunc expose full canonical
`CorrectnessFloorContext` and kernel metadata surfaces. OxCalc now consumes the
code-level `semantic_kernel_metadata_version` bridge in TreeCalc environment
context, local compatibility prepared-callable identity, OxFml runtime
environment input, and diagnostics/artifacts.
OxCalc replay validation also rejects a recorded
`semantic_kernel_metadata_version` when it differs from the active replay
context.

OxCalc should not claim that pairwise or compensated numerical policies, error
algebra precedence, or affected-function invalidation are enforced by current
kernels. Current OxCalc selector diagnostics prove selection and replay
mismatch detection only.

## Remaining Open Lanes
1. selector enforcement evidence in OxFunc kernels,
2. concrete replay fields for non-left-fold policies,
3. broader OxFml runtime/replay projection of `CorrectnessFloorContext`
   selector fields,
4. migration from local `CorrectnessFloorReplayRecord` compatibility fields to
   canonical OxFml replay fields when those fields exist.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - OxFunc selector enforcement evidence
  - canonical replay field names for non-left-fold policies
  - broader OxFml runtime/replay emission of selector context fields
  - migration from local selector replay artifacts to canonical OxFml replay fields
