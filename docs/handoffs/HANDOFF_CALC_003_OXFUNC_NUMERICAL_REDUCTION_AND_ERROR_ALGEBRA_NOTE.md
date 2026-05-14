*Posted by Codex agent on behalf of @govert*

# HANDOFF-CALC-003 OxFunc Note: Numerical Reduction And Error Algebra

## Purpose
This supporting note records the OxFunc-owned kernel and metadata movement
needed by `HANDOFF-CALC-003`.

W050 introduces two correctness-floor selectors:

1. `NumericalReductionPolicy`
2. `ErrorAlgebra`

The OxCalc evidence records selector identity and replay validation. It does
not claim that OxFunc kernels currently consume these selectors.

## Read-Only Sibling Observation
The current OxFunc metadata surface has:

- `../OxFunc/crates/oxfunc_core/src/function.rs`
- `FunctionMeta`
- `ArgPreparationProfile`
- `CoercionLiftProfile`
- `KernelSignatureClass`

No current OxFunc symbol named `NumericalReductionPolicy`, `ErrorAlgebra`, or
`correctness-floor` was found during this filing pass.

The current OxFml evaluation path prepares calls and records prepared-call
trace information in:

- `../OxFml/crates/oxfml_core/src/eval/mod.rs`
- `../OxFml/crates/oxfml_core/src/semantics/mod.rs`

Those surfaces currently carry function metadata and prepared-call trace
records, but not canonical correctness-floor selector fields.

## Requested OxFunc-Owned Shape
OxFunc should provide a canonical way for reduction-sensitive and
error-collapse-sensitive kernels to consume the active selectors.

Required shape or equivalent metadata:

```text
active NumericalReductionPolicy
active ErrorAlgebra
reduction-sensitive function metadata
error-collapse-sensitive function metadata
metadata/version signal for selector-behavior changes
```

Required semantics:

1. A numeric reduction kernel must not choose summation order from local
   runtime convenience when the active profile selects a different policy.
2. `SequentialLeftFold` uses recorded logical input order, left to right.
3. `PairwiseTree` uses a deterministic pairwise tree over recorded logical
   input order, with tree-shape identity visible to replay.
4. `KahanCompensated` treats compensation state as semantic algorithm state,
   not as an optional optimization.
5. `CanonicalExcelLegacy` worksheet-error collapse uses this precedence order:
   `#NULL!`, `#DIV/0!`, `#VALUE!`, `#REF!`, `#NAME?`, `#NUM!`, `#N/A`.
6. Any non-canonical error algebra must use a new selector key and
   `profile_version`, and must list a total precedence order over admitted
   worksheet-error codes.
7. If selector handling changes for existing functions, the change is
   bind-visible and must carry a metadata/version signal so stale prepared
   callables can be invalidated.

## First Affected Function Families
The receiving repo should decide the exact affected set. The OxCalc-side
expectation is that the first review includes any OxFunc function family that:

1. reduces a sequence of numeric values into one numeric result,
2. aggregates over ranges, arrays, grouped data, or helper-callable results,
3. chooses one worksheet error from multiple worksheet-error candidates,
4. delegates to a shared reduction or worksheet-error helper.

This note intentionally does not enumerate a final function list, because
OxFunc owns function metadata and kernel classification.

## Requested OxFml Threading
When OxFunc owns the kernel metadata and execution hooks, OxFml should thread
the active correctness-floor profile through:

1. semantic-plan request or linked evaluation profile context,
2. function-plan binding where selector behavior affects plan identity,
3. prepared-call evaluation,
4. trace/replay fields,
5. bind-visible function metadata versioning.

OxCalc may conservatively rebind all formulas when only a global selector
metadata version is available. Narrower invalidation requires an OxFml/OxFunc
affected-function or affected-callable surface.

## Evidence
OxCalc-local evidence for this request:

- `docs/spec/core-engine/CORE_ENGINE_PROFILE_SELECTORS.md`
- `docs/test-runs/core-engine/w050-e1-numerical-reduction-policy-selector-001/selector_artifact.json`
- `docs/test-runs/core-engine/w050-e2-error-algebra-selector-001/selector_artifact.json`
- `docs/test-runs/core-engine/w050-e3-correctness-floor-replay-hooks-001/run_artifact.json`
- `src/oxcalc-core/src/numerical_reduction.rs`
- `src/oxcalc-core/src/error_algebra.rs`
- `src/oxcalc-core/src/correctness_floor.rs`

Validation commands:

```powershell
cargo test -p oxcalc-core numerical_reduction -- --nocapture
cargo test -p oxcalc-core error_algebra -- --nocapture
cargo test -p oxcalc-core correctness_floor_replay -- --nocapture
```

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - `HANDOFF-CALC-003` receiving-repo acknowledgment
  - OxFunc-owned metadata and kernel selector consumption
  - OxFml-owned semantic-plan, prepared-call, and trace/replay threading
  - OxCalc migration after canonical receiving-side fields exist
