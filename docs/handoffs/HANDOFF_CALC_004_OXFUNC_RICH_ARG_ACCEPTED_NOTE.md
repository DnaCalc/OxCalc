*Posted by Codex agent on behalf of @govert*

# HANDOFF-CALC-004 OxFunc Note: RichArgAccepted Reservation

## Purpose
This supporting note records the OxFunc-owned vocabulary movement needed by
`HANDOFF-CALC-004` for capability-set hole admission.

W050 admits `RichValueHole(required_capability_set)` and reserves the matching
kernel-side argument-preparation profile:

`ArgPreparationProfile::RichArgAccepted(capability_set)`

The reservation is identity and handoff evidence only. It does not claim that
OxFunc has added the enum variant, that OxFml threads it, or that any current
kernel consumes rich values.

## Read-Only Sibling Observation
The current OxFunc source has:

- `../OxFunc/crates/oxfunc_core/src/function.rs:33`
- variants: `ValuesOnlyPreAdapter`, `RefsVisibleInAdapter`

The current OxFml source consumes `ArgPreparationProfile` in semantic-plan,
binding, evaluation, and scheduler paths:

- `../OxFml/crates/oxfml_core/src/semantics/mod.rs`
- `../OxFml/crates/oxfml_core/src/binding/mod.rs`
- `../OxFml/crates/oxfml_core/src/eval/mod.rs`
- `../OxFml/crates/oxfml_core/src/scheduler/mod.rs`

## Requested OxFunc-Owned Shape
OxFunc should own the canonical variant or equivalent function metadata:

```text
ArgPreparationProfile::RichArgAccepted(capability_set)
```

Required semantics:

1. `capability_set` is typed and stable-serialized using the W050 capability
   vocabulary: `Indexable`, `Enumerable`, `Shaped`, and `Materialisable`.
2. A rich argument is admitted when its published capability set is a
   stable-key superset of the required capability set.
3. The required capability set is the `RichValueHole` identity member; the
   producer's concrete rich-value class and full producer capability set do
   not replace that identity member.
4. Adding the variant is additive only if no existing function silently changes
   to it without a bind-visible metadata version bump.
5. No W050 kernel is expected to consume this profile; first activation belongs
   in successor work such as W051 sparse readers or W052 sensitivity.

## Requested OxFml Threading
When OxFunc owns the variant, OxFml should thread it through:

1. function metadata lookup,
2. semantic-plan `FunctionPlanBinding`,
3. capability requirements,
4. prepared argument records,
5. trace/replay columns,
6. bind-visible `ArgPreparationProfile` metadata versioning.

OxCalc may continue using conservative all-formula rebind when only a global
argument-preparation metadata version is available. Narrower invalidation
requires an OxFml/OxFunc-owned affected-function or affected-callable surface.

## Evidence
OxCalc-local evidence for this reservation:

- `docs/spec/core-engine/CORE_ENGINE_RICH_VALUE_CAPABILITY_VOCABULARY.md`
- `docs/handoffs/HANDOFF_CALC_004_OXFML_CAPABILITY_SET_HOLE_ADMISSION.md`
- `docs/test-runs/core-engine/w050-g4-richargaccepted-reservation-001/reservation_artifact.json`
- `src/oxcalc-core/src/rich_value_capability.rs`

Validation command:

```powershell
cargo test -p oxcalc-core rich_argaccepted_reservation -- --nocapture
```

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - `HANDOFF-CALC-004` receiving-repo acknowledgment
  - OxFunc-owned enum/metadata change
  - OxFml-owned semantic-plan, prepared-argument, and trace/replay threading
  - successor-work activation of the first rich-value-aware kernel
