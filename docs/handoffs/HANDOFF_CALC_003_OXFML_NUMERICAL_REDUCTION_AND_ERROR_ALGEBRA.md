*Posted by Codex agent on behalf of @govert*

# HANDOFF-CALC-003: Numerical Reduction And Error Algebra For W050

## Purpose
This handoff packet requests canonical OxFml and OxFunc support for the W050
correctness-floor profile selectors:

1. `NumericalReductionPolicy`
2. `ErrorAlgebra`

These selectors are semantic state. They are not optimization hints. OxCalc
can record and replay-validate the active selectors locally, but OxFml must
thread them through semantic plan and evaluation context, and OxFunc kernels
must honor them anywhere reduction order or worksheet-error collapse is
observable.

## Source Scope
- Source workset: `W050_OXCALC_OXFML_FORMULA_AUTHORITY_REWORK`
- Source bead: `calc-cwpl.H2`
- Driving local evidence:
  - `docs/spec/core-engine/CORE_ENGINE_PROFILE_SELECTORS.md`
  - `src/oxcalc-core/src/numerical_reduction.rs`
  - `src/oxcalc-core/src/error_algebra.rs`
  - `src/oxcalc-core/src/correctness_floor.rs`
  - `docs/test-runs/core-engine/w050-e1-numerical-reduction-policy-selector-001/selector_artifact.json`
  - `docs/test-runs/core-engine/w050-e2-error-algebra-selector-001/selector_artifact.json`
  - `docs/test-runs/core-engine/w050-e3-correctness-floor-replay-hooks-001/run_artifact.json`
- OxFml / OxFunc references reviewed read-only:
  - `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`
  - `../OxFml/docs/spec/OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`
  - `../OxFml/crates/oxfml_core/src/semantics/mod.rs`
  - `../OxFml/crates/oxfml_core/src/eval/mod.rs`
  - `../OxFml/crates/oxfml_core/src/binding/mod.rs`
  - `../OxFunc/crates/oxfunc_core/src/function.rs`

## Current Compatibility Position
OxCalc now has local selector and replay-validation evidence:

1. `CorrectnessFloorProfile` carries `profile_version`,
   `numerical_reduction_policy`, and `error_algebra`.
2. `CorrectnessFloorReplayRecord` records those selector values in replay
   surfaces.
3. Replay validation accepts matching records and rejects mismatched
   `numerical_reduction_policy` or `error_algebra` records.
4. Exact clause text for all initial selector values is checked against the
   Rust selector surfaces.

Read-only sibling inspection found no current `NumericalReductionPolicy`,
`ErrorAlgebra`, or `correctness-floor` symbols in OxFml or OxFunc. Current
OxFml semantic-plan and evaluation surfaces carry function metadata,
prepared-call traces, and runtime result families, but do not yet carry these
correctness-floor selectors as canonical plan/evaluation/replay fields. Current
OxFunc `FunctionMeta` does not expose reduction-policy or error-algebra
metadata.

## Requested Canonical OxFml Clauses

### 1. Correctness-Floor Profile Fields
OxFml should admit a replay-visible correctness-floor profile record, or an
equivalent canonical object, with:

1. `profile_version`
2. `numerical_reduction_policy`
3. `error_algebra`

Requested clause direction:

1. The active correctness-floor profile is part of semantic evaluation
   context.
2. A semantic plan or runtime request that can reach reduction-sensitive or
   error-collapse-sensitive kernels must carry the active profile, either
   directly or through a stable linked context.
3. Replay recorded under one selector value is not valid under another
   selector value unless an explicit migration proof is attached.

### 2. Semantic Plan And Evaluation Threading
OxFml should thread the active correctness-floor profile through:

1. semantic plan compile inputs or linked profile context,
2. function-plan binding or equivalent per-call evaluation context where the
   selector affects kernel behavior,
3. runtime formula request/session execution context,
4. prepared-call trace and replay projection surfaces,
5. candidate/reject diagnostic surfaces when selector mismatch causes replay
   rejection or compatibility failure.

Requested clause direction:

1. OxCalc should not infer reduction policy or error precedence from source
   text, function name, scheduler choice, or host runtime behavior.
2. OxFml remains the canonical owner of formula-language plan/evaluation
   context and must expose the selector values in public consumer-runtime or
   replay-facing fields.
3. Runtime scheduling, parallelism, or local optimization cannot change the
   observable policy selected by the active profile.

### 3. Replay Validation Columns
OxFml should expose replay fields equivalent to:

```text
profile_version
numerical_reduction_policy
error_algebra
```

Requested clause direction:

1. Replay must reject a trace when recorded selector fields do not match the
   active profile fields.
2. A `NumericalReductionPolicy=PairwiseTree` trace must include enough
   tree-shape identity to prove deterministic replay over the recorded logical
   input order.
3. A `NumericalReductionPolicy=KahanCompensated` trace must identify the
   compensation policy as semantic state, not as a removable optimization.
4. An `ErrorAlgebra` trace must identify the total precedence order used for
   worksheet-error collapse.

## Requested OxFunc-Owned Kernel Obligations
OxFunc should own the kernel-side metadata and execution discipline needed for
the selectors to be meaningful:

1. kernels that reduce numeric sequences must receive or resolve the active
   `NumericalReductionPolicy`,
2. kernels that collapse multiple worksheet-error candidates into one
   observable result must receive or resolve the active `ErrorAlgebra`,
3. reduction-sensitive function metadata should be bind-visible when it
   affects semantic plan identity, cache identity, or replay requirements,
4. changes to reduction/error selector handling must be paired with a
   metadata/version signal that lets OxFml and OxCalc invalidate stale prepared
   callables conservatively if narrower invalidation is unavailable.

The supporting OxFunc note is:

`docs/handoffs/HANDOFF_CALC_003_OXFUNC_NUMERICAL_REDUCTION_AND_ERROR_ALGEBRA_NOTE.md`

## Exact Selector Clauses

### NumericalReductionPolicy

`CALC-003.NRP.SequentialLeftFold`:

When a profile declares NumericalReductionPolicy=SequentialLeftFold, OxFml/OxFunc reduction kernels MUST reduce numeric sequences in the recorded logical input order, applying each operand to the accumulator exactly once from left to right; kernels MUST NOT rebalance, parallelize, or compensate the order unless the active profile changes.

`CALC-003.NRP.PairwiseTree`:

When a profile declares NumericalReductionPolicy=PairwiseTree, OxFml/OxFunc reduction kernels MUST reduce numeric sequences using a deterministic pairwise tree whose leaf order is the recorded logical input order and whose tree-shape identity is replay-visible; kernels MUST NOT choose runtime-dependent partitioning.

`CALC-003.NRP.KahanCompensated`:

When a profile declares NumericalReductionPolicy=KahanCompensated, OxFml/OxFunc reduction kernels MUST reduce numeric sequences in the recorded logical input order using Kahan-style compensation state that is part of the semantic algorithm; kernels MUST surface the selector in replay so a non-compensated result cannot satisfy this profile.

### ErrorAlgebra

`CALC-003.ERR.CanonicalExcelLegacy`:

When a profile declares ErrorAlgebra=CanonicalExcelLegacy, OxFml/OxFunc kernels that must collapse multiple worksheet-error candidates into one observable result MUST select the earliest error in the precedence order #NULL!, #DIV/0!, #VALUE!, #REF!, #NAME?, #NUM!, #N/A; kernels MUST record the active error algebra in replay and MUST NOT substitute function-local or runtime-dependent precedence unless the active profile declares a different ErrorAlgebra selector.

`CALC-003.ERR.ExtensionRule`:

Any non-canonical ErrorAlgebra profile MUST use a new selector key and profile_version, MUST list a total precedence order over every admitted worksheet-error code plus explicit placement for newly admitted codes, and MUST be replay-invalid against traces recorded under CanonicalExcelLegacy unless an explicit migration proof is attached.

## Migration And Fallback Impact

### If Accepted
1. OxCalc can treat `CorrectnessFloorProfile` selector values as imported
   OxFml/OxFunc semantics rather than OxCalc-local replay hints.
2. OxCalc replay artifacts can validate against canonical OxFml replay
   fields instead of local compatibility records.
3. Reduction-sensitive and error-collapse-sensitive prepared callables can be
   invalidated when selector metadata changes.
4. Future profile variants can be admitted additively through explicit
   `profile_version` and selector keys.

### If Deferred
1. OxCalc can continue recording local correctness-floor replay records.
2. OxCalc cannot claim OxFml/OxFunc kernel enforcement for the selectors.
3. W050 aggregate closure remains blocked on receiving-side acknowledgment or
   an explicit replacement plan for the selector-threading contract.
4. No OxCalc adapter workaround should be added to simulate kernel-side
   selector compliance outside OxFml/OxFunc.

## Evidence And References
Current OxCalc evidence:

1. E1 `NumericalReductionPolicy` selector artifact:
   `docs/test-runs/core-engine/w050-e1-numerical-reduction-policy-selector-001/selector_artifact.json`
2. E2 `ErrorAlgebra` selector artifact:
   `docs/test-runs/core-engine/w050-e2-error-algebra-selector-001/selector_artifact.json`
3. E3 correctness-floor replay hooks:
   `docs/test-runs/core-engine/w050-e3-correctness-floor-replay-hooks-001/run_artifact.json`

Validation commands already exercised for the supporting evidence include:

1. `cargo test -p oxcalc-core numerical_reduction -- --nocapture`
2. `cargo test -p oxcalc-core error_algebra -- --nocapture`
3. `cargo test -p oxcalc-core correctness_floor_replay -- --nocapture`
4. `cargo test -p oxcalc-core`
5. `cargo clippy -p oxcalc-core --all-targets -- -D warnings`
6. `scripts/check-worksets.ps1`
7. `br dep cycles`

## Open Questions For OxFml And OxFunc
1. Should the profile object live in the runtime request/session API, the
   semantic-plan API, or a shared linked profile context used by both?
2. Which OxFunc metadata field should identify reduction-sensitive kernels?
3. Should `ErrorAlgebra` apply through a shared helper for all worksheet-error
   collapse sites, or through per-kernel metadata and explicit calls?
4. What replay field names should replace OxCalc's local
   `CorrectnessFloorReplayRecord`?
5. What metadata/version signal should trigger prepared-callable invalidation
   when reduction/error selector behavior changes?

## Requested Next Step
Please review this packet against the current OxFml consumer-runtime,
semantic-plan, evaluation, and OxFunc function-metadata surfaces and determine:

1. which clauses should be promoted directly into OxFml canonical text,
2. which fields belong in OxFml versus OxFunc,
3. which function families are first affected by the kernel obligations,
4. which replay-field names should be canonical,
5. whether a narrower replacement plan should supersede this packet.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - OxFml-side acknowledgment and canonical integration
  - OxFunc-side metadata and kernel obligation acknowledgment
  - canonical replay field naming
  - prepared-callable invalidation/version signal for selector behavior
  - OxCalc migration after receiving-side acknowledgment
