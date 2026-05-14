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

## OxCalc Integration Consequences
OxCalc should keep W050 sparse/rich hole vocabulary and capability columns as
identity reservation and compatibility evidence. It should not report sparse
reader support, rich-value producer support, or rich-kernel execution as
implemented behavior.

The next useful migration point is an OxFml/OxFunc successor that emits
producer and exercised capability facts through canonical runtime/replay
fields.

## Remaining Open Lanes
1. OxFunc rich/sparse metadata acknowledgment,
2. sparse/rich producer activation successor work,
3. canonical runtime/replay field implementation,
4. migration of OxCalc reserved local columns to canonical emitted fields.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - OxFunc rich/sparse metadata acknowledgment
  - sparse/rich producer activation successor work
  - canonical OxFml runtime/replay field implementation
  - OxCalc migration from reserved capability columns
