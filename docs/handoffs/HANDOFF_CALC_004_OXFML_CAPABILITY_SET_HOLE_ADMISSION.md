*Posted by Codex agent on behalf of @govert*

# HANDOFF-CALC-004: Capability-Set Hole Admission For W050

## Purpose
This handoff packet requests canonical OxFml and OxFunc support for W050
capability-set hole admission.

W050 commits the identity discipline for:

1. the default hole taxonomy, including `SparseRangeHole` and
   `RichValueHole`,
2. the initial rich-value capability vocabulary,
3. capability-set composition in `plan_template_key`,
4. trace/replay columns for required, producer, and exercised capability
   sets,
5. the OxFunc-owned `ArgPreparationProfile::RichArgAccepted(capability_set)`
   reservation.

The request is identity and seam-contract work. It does not claim that OxFml
or OxFunc currently emits rich-value holes, publishes producer capability
sets, or executes rich-value-aware kernels.

## Source Scope
- Source workset: `W050_OXCALC_OXFML_FORMULA_AUTHORITY_REWORK`
- Source bead: `calc-cwpl.H3`
- Driving local evidence:
  - `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md` section 22.13
  - `docs/spec/core-engine/CORE_ENGINE_RICH_VALUE_CAPABILITY_VOCABULARY.md`
  - `docs/worksets/W050_OXCALC_OXFML_FORMULA_AUTHORITY_REWORK.md` sections 10.7 and 10.12
  - `src/oxcalc-core/src/formula_identity.rs`
  - `src/oxcalc-core/src/rich_value_capability.rs`
  - `src/oxcalc-core/src/treecalc.rs`
  - `src/oxcalc-core/src/treecalc_runner.rs`
  - `docs/test-runs/core-engine/w050-g1-rich-capability-vocabulary-001/run_artifact.json`
  - `docs/test-runs/core-engine/w050-g2-rich-value-hole-capability-requirements-001/run_artifact.json`
  - `docs/test-runs/core-engine/w050-g3-capability-trace-replay-columns-001/run_artifact.json`
  - `docs/test-runs/core-engine/w050-g4-richargaccepted-reservation-001/reservation_artifact.json`
- Supporting OxFunc note:
  - `docs/handoffs/HANDOFF_CALC_004_OXFUNC_RICH_ARG_ACCEPTED_NOTE.md`
- OxFml / OxFunc references reviewed read-only:
  - `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`
  - `../OxFml/docs/spec/OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`
  - `../OxFml/crates/oxfml_core/src/semantics/mod.rs`
  - `../OxFml/crates/oxfml_core/src/binding/mod.rs`
  - `../OxFml/crates/oxfml_core/src/eval/mod.rs`
  - `../OxFml/crates/oxfml_core/src/scheduler/mod.rs`
  - `../OxFunc/crates/oxfunc_core/src/function.rs`

## Current Compatibility Position
OxCalc now has local identity and replay evidence:

1. `PlanTemplateHoleKind` has stable-keyed variants `ValueHole`,
   `RefOrValueHole`, `CallableHole`, `ShapeSensitiveHole`,
   `SparseRangeHole`, and `RichValueHole`.
2. `RichValueHole(required_capability_set)` carries a typed
   `RichValueCapabilitySet`; the sorted required-set stable key participates
   in `plan_template_key` material.
3. `RichValueCapabilitySet` sorts and deduplicates typed capability stable
   keys, and producer admission is a stable-key superset check.
4. `rich_value_capability_columns` are reserved on prepared identity and
   derivation trace surfaces.
5. Current V1 production paths emit no `SparseRangeHole`, no
   `RichValueHole`, no producer capability sets, and no rich-kernel
   activation.

Read-only sibling inspection found no current `RichValueHole`,
`RichValueCapability`, `SparseRangeHole`, `SparseRangeReader`,
`SparseIteratorOk`, or `RichArgAccepted` symbols in OxFml or OxFunc. Current
OxFunc `ArgPreparationProfile` variants are `ValuesOnlyPreAdapter` and
`RefsVisibleInAdapter`.

## Requested Canonical OxFml Clauses

### 1. Default Hole Taxonomy
OxFml should own or expose canonical names and stable serialization for the
default W050 hole taxonomy:

1. `ValueHole(value_class_bound)`
2. `RefOrValueHole(ref_observability)`
3. `CallableHole(callable_signature)`
4. `ShapeSensitiveHole(extent_class)`
5. `SparseRangeHole(extent_class, cardinality_class)`
6. `RichValueHole(required_capability_set)`

Requested clause direction:

1. Hole kind is part of `PlanTemplate` identity.
2. `hole_id`, `ordinal`, `path`, and stable `hole_kind` serialization are
   replay-visible.
3. Literal values, concrete references, omitted arguments, and helper names
   remain binding payloads unless a future evidence-gated narrower producer
   explicitly makes them template identity.
4. Current V1 compatibility projections in OxCalc should be retired once
   OxFml exposes canonical prepared-callable, plan-template, and hole-binding
   fields.

### 2. Wide-By-Default Mapping
OxFml should confirm the wide-by-default mapping from function argument
preparation to template holes:

1. `ValuesOnlyPreAdapter` arguments map to `ValueHole(AnyValue)` unless a
   future narrower producer is explicitly admitted.
2. `RefsVisibleInAdapter` arguments map to
   `RefOrValueHole(ReferenceIdentityVisible)`.
3. Invocation callees map to `CallableHole(AnyCallable)` with the concrete
   callee payload retained in `HoleBindings`.
4. Shape-sensitive calls may use `ShapeSensitiveHole(extent_class)` when array
   shape participates in semantics.

Requested clause direction:

1. Widening is the default sharing and replay rule.
2. Narrowing is additive only through explicit new stable keys and evidence.
3. A bind-visible change to a function's argument-preparation profile requires
   metadata/version invalidation for prepared callables.

### 3. SparseRangeHole Admission
OxFml should admit `SparseRangeHole(extent_class, cardinality_class)` as a
canonical hole kind, even before any current kernel consumes a sparse reader.

Requested protocol shape:

```text
declared_extent
defined_cardinality
defined_iter
read_at(coord) -> Defined(EvalValue) | Blank
contains(coord)
```

Requested clause direction:

1. `Defined` includes all assigned cell values, including empty-string text
   `""`.
2. `Blank` covers never-assigned and assigned-then-cleared cells at the
   cell-value layer.
3. Sheet-structural state that persists across clear operations remains owned
   by repository/structure surfaces, not by the sparse value reader.
4. Kernel activation for sparse readers is successor work. The W050 contract
   is hole-kind identity and admission, not aggregation-kernel execution.

### 4. RichValueHole Capability Identity
OxFml should admit `RichValueHole(required_capability_set)` as a canonical
hole kind.

Initial capability selectors:

| selector | typed parameters |
|---|---|
| `Indexable` | `rank`, `index_type`, `element_value_class` |
| `Enumerable` | `element_value_class`, `order_guarantee` |
| `Shaped` | `extent_class` |
| `Materialisable` | `target_class` |

Requested clause direction:

1. Each capability emits a typed stable key containing selector and parameter
   values.
2. A required capability set sorts stable keys by byte order and deduplicates
   identical keys.
3. The sorted required-set stable key is part of `RichValueHole` stable
   identity and therefore part of `plan_template_key` material.
4. Producer admission is a stable-key superset check.
5. The `RichValueHole` identity remains the required capability set, not the
   producer's concrete rich-value class and not the producer's full published
   capability set.

### 5. Capability Trace And Replay Columns
OxFml should expose canonical trace/replay fields equivalent to OxCalc's
reserved local schema:

```text
rich_value_capability_columns.required_capability_set_keys
rich_value_capability_columns.producer_capability_set_keys
rich_value_capability_columns.exercised_capability_keys
```

Requested clause direction:

1. Required capability-set keys are template/replay identity.
2. Producer capability-set keys are recorded when rich-value producers exist.
3. Exercised capability keys are recorded when rich kernels invoke capability
   operations.
4. Empty current V1 output remains an honest reserved state and should not be
   interpreted as producer support.

### 6. Capability Mismatch And Replay Failure
OxFml should expose a deterministic mismatch path when a producer cannot
satisfy a required capability set.

Requested clause direction:

1. Capability mismatch is a typed reject or bind/evaluation diagnostic with
   replay-visible detail.
2. Replay under a missing or different required capability-set key is invalid
   unless an explicit migration proof is attached.
3. Producer-superset admission must not alter the required-set identity
   recorded by the template.

## Requested OxFunc-Owned Shape
OxFunc should own the kernel-side vocabulary or equivalent function metadata
for rich argument admission:

```text
ArgPreparationProfile::RichArgAccepted(capability_set)
```

Required semantics:

1. `capability_set` uses the same typed stable-key vocabulary as
   `RichValueHole(required_capability_set)`.
2. A rich argument is admitted when the producer publishes a stable-key
   superset of the required capability set.
3. Switching an existing function to `RichArgAccepted` is bind-visible and
   requires `ArgPreparationProfile` metadata versioning.
4. No W050 kernel is expected to consume this profile; first activation
   belongs in successor work such as W051 sparse readers or W052 sensitivity.

Sparse reader kernel activation is also successor work. If OxFunc admits a
`SparseIteratorOk` profile or equivalent metadata, it should be additive,
bind-visible, and paired with the same trace/replay identity discipline.

## Migration And Fallback Impact

### If Accepted
1. OxCalc can replace local hole-taxonomy compatibility projections with
   OxFml-owned `PlanTemplate` and hole identity fields.
2. Rich-value and sparse-reader successor work can add concrete producers
   without retrofitting plan-template identity or replay columns.
3. Capability mismatch, producer capability sets, and exercised capability
   operations can become canonical replay facts.
4. `ArgPreparationProfile` metadata changes can drive prepared-callable
   invalidation through OxFml/OxFunc-owned versioning.

### If Deferred
1. OxCalc can keep the W050 local identity reservation and empty/reserved
   trace columns.
2. OxCalc cannot claim OxFml/OxFunc capability-set hole support or rich-kernel
   enforcement.
3. W050 aggregate closure remains blocked on receiving-side acknowledgment or
   an explicit replacement plan.
4. No OxCalc adapter workaround should be added to emulate rich-value kernel
   semantics outside OxFml/OxFunc.

## Evidence And References
Current OxCalc evidence:

1. G1 capability vocabulary:
   `docs/test-runs/core-engine/w050-g1-rich-capability-vocabulary-001/run_artifact.json`
2. G2 `RichValueHole(required_capability_set)` identity:
   `docs/test-runs/core-engine/w050-g2-rich-value-hole-capability-requirements-001/run_artifact.json`
3. G3 capability trace/replay columns:
   `docs/test-runs/core-engine/w050-g3-capability-trace-replay-columns-001/run_artifact.json`
4. G4 `RichArgAccepted` reservation:
   `docs/test-runs/core-engine/w050-g4-richargaccepted-reservation-001/reservation_artifact.json`
5. Supporting OxFunc note:
   `docs/handoffs/HANDOFF_CALC_004_OXFUNC_RICH_ARG_ACCEPTED_NOTE.md`

Validation commands already exercised for the supporting evidence include:

1. `cargo test -p oxcalc-core rich_value_capability -- --nocapture`
2. `cargo test -p oxcalc-core rich_value_hole -- --nocapture`
3. `cargo test -p oxcalc-core capability_set_trace_replay -- --nocapture`
4. `cargo test -p oxcalc-core rich_argaccepted_reservation -- --nocapture`
5. `cargo test -p oxcalc-core formula_identity -- --nocapture`
6. `cargo test -p oxcalc-core derivation_trace -- --nocapture`
7. `cargo test -p oxcalc-core`
8. `cargo clippy -p oxcalc-core --all-targets -- -D warnings`
9. `scripts/check-worksets.ps1`
10. `br dep cycles`

## Open Questions For OxFml And OxFunc
1. Should OxFml use the exact hole names in this packet or adapt them to
   OxFml-owned naming while preserving stable identity semantics?
2. Which fields should carry `RichValueHole` required capability sets in the
   public consumer-runtime and replay projections?
3. Should capability mismatch be a bind-time diagnostic, evaluation reject, or
   both depending on when producer capability facts are known?
4. What metadata/version signal should represent `ArgPreparationProfile`
   changes for `RichArgAccepted` and future sparse-reader profiles?
5. Which successor workset should own the first concrete sparse/rich producer
   activation in OxFunc?

## Requested Next Step
Please review this packet against the current OxFml semantic-plan,
consumer-runtime, evaluation, scheduler, and OxFunc function-metadata surfaces
and determine:

1. which hole-taxonomy and capability clauses should become canonical OxFml
   text,
2. which profile/metadata pieces belong in OxFunc,
3. which field names should replace OxCalc's local compatibility projections,
4. which parts should be routed to W051/W052 successor work,
5. whether a narrower replacement plan should supersede this packet.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - OxFml-side acknowledgment and canonical integration
  - OxFunc-side `RichArgAccepted` or equivalent metadata acknowledgment
  - canonical trace/replay field naming
  - sparse/rich producer activation in successor work
  - OxCalc migration after receiving-side acknowledgment
