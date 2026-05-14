# CORE_ENGINE_PROFILE_SELECTORS.md

## 1. Purpose and Status
This document defines OxCalc-owned profile-governed semantic selectors that
affect evaluation and replay.

Status:
1. active W050 profile-selector companion,
2. canonical local source for selectors introduced by W050 Lane E,
3. handoff-preparation surface for shared OxFml/OxFunc evaluation context work.

This document locks the `NumericalReductionPolicy` and `ErrorAlgebra`
selectors.

## 2. Profile Record Rule
Every replay-visible correctness-floor evaluation profile must carry:

1. `profile_version`
2. `numerical_reduction_policy`
3. `error_algebra`

The individual replay key forms are:

```text
<profile_version>|numerical_reduction_policy:<selector_key>
<profile_version>|error_algebra:<selector_key>
```

Both selectors are semantic state. They are not optimization hints and not
scheduler-local choices.

## 3. NumericalReductionPolicy
`NumericalReductionPolicy` has the following initial selector values:

| selector value | order basis | replay obligation |
|---|---|---|
| `SequentialLeftFold` | recorded logical input order, left to right | record the selector and input-order identity |
| `PairwiseTree` | deterministic pairwise tree over recorded logical input order | record the selector and tree-shape identity |
| `KahanCompensated` | recorded logical input order with Kahan-style compensation state | record the selector and compensation-policy identity |

Profiles may later declare documented equivalents, but an equivalent must name
which of these replay obligations it preserves and must not silently alias a
different observable result policy.

## 4. ErrorAlgebra
`ErrorAlgebra` has the following initial selector value:

| selector value | precedence order | replay obligation |
|---|---|---|
| `CanonicalExcelLegacy` | `#NULL!`, `#DIV/0!`, `#VALUE!`, `#REF!`, `#NAME?`, `#NUM!`, `#N/A` | record the selector and validate the same precedence order on replay |

The order is highest precedence first. When multiple worksheet-error candidates
must collapse to one observable result, the earliest error in the active
algebra wins.

### 4.1 Extension Rule
Profiles may declare a non-canonical `ErrorAlgebra`, but only by using a new
selector key and `profile_version`. A non-canonical profile must list a total
precedence order over every admitted worksheet-error code and must explicitly
place any newly admitted code. Replay recorded under one error algebra is not
valid under another unless an explicit migration proof is attached.

## 5. Handoff-Ready Exact Clauses
The following clause language is ready to be copied into
`HANDOFF_CALC_003_OXFML_NUMERICAL_REDUCTION_AND_ERROR_ALGEBRA.md`.

### 5.1 NumericalReductionPolicy

`CALC-003.NRP.SequentialLeftFold`:

When a profile declares NumericalReductionPolicy=SequentialLeftFold, OxFml/OxFunc reduction kernels MUST reduce numeric sequences in the recorded logical input order, applying each operand to the accumulator exactly once from left to right; kernels MUST NOT rebalance, parallelize, or compensate the order unless the active profile changes.

`CALC-003.NRP.PairwiseTree`:

When a profile declares NumericalReductionPolicy=PairwiseTree, OxFml/OxFunc reduction kernels MUST reduce numeric sequences using a deterministic pairwise tree whose leaf order is the recorded logical input order and whose tree-shape identity is replay-visible; kernels MUST NOT choose runtime-dependent partitioning.

`CALC-003.NRP.KahanCompensated`:

When a profile declares NumericalReductionPolicy=KahanCompensated, OxFml/OxFunc reduction kernels MUST reduce numeric sequences in the recorded logical input order using Kahan-style compensation state that is part of the semantic algorithm; kernels MUST surface the selector in replay so a non-compensated result cannot satisfy this profile.

### 5.2 ErrorAlgebra

`CALC-003.ERR.CanonicalExcelLegacy`:

When a profile declares ErrorAlgebra=CanonicalExcelLegacy, OxFml/OxFunc kernels that must collapse multiple worksheet-error candidates into one observable result MUST select the earliest error in the precedence order #NULL!, #DIV/0!, #VALUE!, #REF!, #NAME?, #NUM!, #N/A; kernels MUST record the active error algebra in replay and MUST NOT substitute function-local or runtime-dependent precedence unless the active profile declares a different ErrorAlgebra selector.

`CALC-003.ERR.ExtensionRule`:

Any non-canonical ErrorAlgebra profile MUST use a new selector key and profile_version, MUST list a total precedence order over every admitted worksheet-error code plus explicit placement for newly admitted codes, and MUST be replay-invalid against traces recorded under CanonicalExcelLegacy unless an explicit migration proof is attached.

## 6. Replay Validation Evidence
The first checked OxCalc-local selector artifact is:

`docs/test-runs/core-engine/w050-e1-numerical-reduction-policy-selector-001`

It records the selector profile fields, initial variants, behavior flags, and
exact CALC-003 clause text. The validation test
`checked_in_numerical_reduction_policy_artifact_matches_runtime_clauses`
compares the checked artifact against the Rust selector surface.

The first checked OxCalc-local error-algebra selector artifact is:

`docs/test-runs/core-engine/w050-e2-error-algebra-selector-001`

It records the canonical Excel legacy precedence order, extension rule, and
exact CALC-003 clause text. The validation test
`checked_in_error_algebra_artifact_matches_runtime_clauses` compares the
checked artifact against the Rust selector surface.

## 7. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - OxFml/OxFunc threading and acknowledgement remain routed through H2/CALC-003,
  - wave trace replay hooks remain routed through the Lane E replay-validation item
