# HANDOFF-CALC-004 OxCalc Receipt and Integration Note

## Purpose
Record the OxCalc-side receipt of OxFml's acknowledgment for
`HANDOFF-CALC-004` and summarize the local integration consequences for W050
capability-set hole admission, sparse/rich producer identity, and replay field
work.

This note records receiving-repo awareness and migration consequences. It does
not close the remaining OxCalc dependency on OxFunc sparse/rich producer
metadata or concrete rich/sparse execution support.

## Receiving-Side Sources
1. `../OxFml/docs/handoffs/HANDOFF_CALC_004_OXFML_RECEIPT.md`
2. `../OxFml/docs/handoffs/HANDOFF_REGISTER.csv`
3. `../OxFml/docs/spec/OXFML_CANONICAL_ARTIFACT_SHAPES.md`
4. `../OxFml/docs/spec/OXFML_MINIMUM_SEAM_SCHEMAS.md`
5. `../OxFml/docs/spec/OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`
6. `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`
7. `../OxFunc/docs/handoffs/HANDOFF-CALC-004_OXFUNC_RECEIPT.md`
8. `../OxFunc/docs/function-lane/OXFUNC_KERNEL_METADATA_AND_ADMISSION_PROFILE_CONTRACT.md`

## Verified OxFml Response Summary
OxFml acknowledged the handoff with decision
`accept_identity_reservation_defer_activation`.

Accepted OxFml canonical direction:
1. default template-hole taxonomy should include `ValueHole`,
   `RefOrValueHole`, `CallableHole`, `ShapeSensitiveHole`, `SparseRangeHole`,
   and `RichValueHole`,
2. hole identity is part of plan-template identity,
3. wide-by-default mapping from current OxFunc `ArgPreparationProfile` values
   is the correct first rule,
4. `RichValueHole` identity is the required capability set, not producer class,
5. required capability-set keys are replay/template identity,
6. producer and exercised capability columns are reserved but empty until real
   producers and kernels exist,
7. capability mismatch must be typed and replay-visible when producer facts are
   known.

Preferred OxFml field families:
1. `TemplateHole`,
2. `hole_kind_key`,
3. `RichValueCapabilityColumns`,
4. `required_capability_set_keys`,
5. `producer_capability_set_keys`,
6. `exercised_capability_keys`,
7. `CapabilitySetMismatchContext`.

OxFunc has acknowledged its side of the split. OxFunc accepts a metadata shape
equivalent to `RichArgAccepted(required_capability_set)`, accepts sparse-reader
admission metadata as a successor lane, reserves
`arg_admission_metadata_version`, and records producer capability publication
as typed metadata on the producer or returned rich/sparse carrier.

OxFml has consumed that OxFunc response and accepts
`arg_admission_metadata_version` as the OxFunc-owned admission/profile
invalidation bridge. OxFml also records `IMAGE` / `_webimage` producer
capability publication as the preferred first rich producer activation lane,
with sparse range readers deferred.

## OxCalc Integration Consequences
OxCalc should keep W050 sparse/rich hole vocabulary as identity reservation
and compatibility evidence. It should not report sparse reader support,
rich-argument admission, or rich-kernel execution as implemented behavior.

OxCalc now consumes the code-level `arg_admission_metadata_version` bridge in
TreeCalc environment context, local compatibility prepared-callable identity,
OxFml runtime environment input, and diagnostics/artifacts.

OxCalc now consumes OxFml returned-value producer capability facts for
`IMAGE` / `_webimage` through TreeCalc diagnostics and derivation trace
capability columns. Exercised capability facts remain reserved.

## Remaining Open Lanes
1. sparse-reader activation successor work,
2. `RichArgAccepted(required_capability_set)` runtime admission,
3. exercised capability-key emission once kernels consume rich/sparse
   operations,
4. broader rich producer protocol beyond the first `IMAGE` / `_webimage`
   carrier.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - sparse-reader activation successor work
  - RichArgAccepted runtime admission
  - exercised capability-key emission
  - broader rich producer protocol beyond IMAGE/_webimage
