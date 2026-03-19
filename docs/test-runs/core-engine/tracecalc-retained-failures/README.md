# TraceCalc Retained-Failure Runs

This directory holds retained-failure and reduced-witness baseline artifacts for `TraceCalc`.

## Purpose
1. preserve deterministic retained-failure artifacts separate from the ordinary reference-machine run root,
2. keep replay-valid retained-local, explanatory-only, and quarantined witness outcomes explicit,
3. provide checked-in evidence for W016 without treating those artifacts as pack-grade replay evidence.

## Baseline Runs
Current retained-failure baseline:
1. `w016-sequence4-retained-failure-baseline`

Current replay-appliance-aware retained-failure baseline:
1. `w018-retained-replay-appliance-bundle-baseline`

Current distill-valid and pack-candidate rehearsal baseline:
1. `w019-distill-and-pack-candidate-baseline`

Current W021 semantic-only pack-contract baseline:
1. `w021-sequence1-pack-contract`

Current W021 gate baseline:
1. `w021-pack-grade-gate-baseline`

Current W022 direct-binding family baseline:
1. `w022-sequence1-direct-binding-family`

It is emitted by:
1. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w016-sequence4-retained-failure-baseline`
2. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w018-retained-replay-appliance-bundle-baseline`
3. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w019-distill-and-pack-candidate-baseline`
4. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w021-sequence1-pack-contract`
5. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w021-pack-grade-gate-baseline`
6. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w022-sequence1-direct-binding-family`

The current retained-failure baseline carries:
1. one replay-valid retained-local witness family,
2. one explanatory-only retained witness case,
3. one quarantined retained witness case.

The replay-appliance-aware retained-failure baseline additionally carries:
1. additive replay-appliance bundle roots,
2. bundle-validation artifacts,
3. per-case explain records bound to retained-failure lifecycle and mismatch targets.

The W019 retained-failure baseline additionally carries:
1. replay-valid reduced-scenario artifacts for retained-local witness families,
2. run-level `distill_validation.json` proving `cap.C4.distill_valid`,
3. dependency-projection-sensitive retained-local evidence,
4. rehearsal-only `pack_candidate_validation.json` proving non-pack separation without claiming `cap.C5.pack_valid`.

W021 Sequence 1 begins by adding:
1. `pack_grade_contract.json` to declare the current semantic-only `TraceCalc` pack scope,
2. an explicit emitted statement that the pack-grade validator remains a later proof step,
3. no change to the current highest honest capability claim.

The W021 gate baseline additionally carries:
1. `pack_grade_validation.json` with explicit bounded blockers,
2. no claim of `cap.C5.pack_valid`,
3. next evidence steps for retained-shared or pack-promoted witness families and direct-binding-sensitive pack evidence.

The W022 Sequence 1 baseline additionally carries:
1. one direct-binding-sensitive retained-local case,
2. pack-facing metadata proving that the direct-binding family is now exercised locally,
3. a narrowed pack-grade blocker set focused on missing shared-lifecycle evidence.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - this directory now carries the original retained-failure baseline, the replay-appliance-aware retained-failure baseline, the W019 distill-valid retained-failure baseline, the W021 gate baseline, and the W022 direct-binding family baseline
  - retained-shared or pack-promoted witness-family evidence and later shared replay governance remain later lanes
