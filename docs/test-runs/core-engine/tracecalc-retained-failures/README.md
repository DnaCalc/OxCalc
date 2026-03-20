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

Current W022 shared-lifecycle family baseline:
1. `w022-sequence2-shared-lifecycle-family`

Current W022 decision baseline:
1. `w022-sequence3-pack-decision`

Current W023 program-scope contract baseline:
1. `w023-sequence1-program-scope-contract`

Current W023 host-sensitive family baseline:
1. `w023-sequence2-host-sensitive-family`

Current W023 program-scope decision baseline:
1. `w023-sequence3-program-decision`

It is emitted by:
1. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w016-sequence4-retained-failure-baseline`
2. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w018-retained-replay-appliance-bundle-baseline`
3. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w019-distill-and-pack-candidate-baseline`
4. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w021-sequence1-pack-contract`
5. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w021-pack-grade-gate-baseline`
6. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w022-sequence1-direct-binding-family`
7. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w022-sequence2-shared-lifecycle-family`
8. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w022-sequence3-pack-decision`
9. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w023-sequence1-program-scope-contract`
10. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w023-sequence2-host-sensitive-family`
11. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w023-sequence3-program-decision`

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

The W022 Sequence 2 baseline additionally carries:
1. one replay-valid retained-shared witness family,
2. pack-facing validation that family coverage is now reached for the current semantic-only `TraceCalc` pack scope,
3. no claim of `cap.C5.pack_valid`, leaving that as the explicit next decision lane.

The W022 Sequence 3 baseline additionally carries:
1. an explicit `pack_grade_decision.json` artifact,
2. a bounded local decision to keep `cap.C5.pack_valid` unclaimed for the current semantic-only scope,
3. a packetized residual lane in `W023` rather than an implicit lingering blocker.

The W023 Sequence 1 baseline additionally carries:
1. an explicit `program_grade_contract.json` artifact,
2. an explicit `program_grade_validation.json` artifact,
3. a bounded emitted statement that current semantic-only family coverage is insufficient for broader program-grade pack promotion.

The W023 Sequence 2 baseline additionally carries:
1. one retained-shared direct-binding case with broader host identity references,
2. a narrowed `program_grade_validation.json` blocker set containing only `pack.grade.program_scope.unproven`,
3. no claim of `cap.C5.pack_valid`, leaving broader program-grade promotion as the remaining decision lane.

The W023 Sequence 3 baseline additionally carries:
1. an explicit `program_grade_decision.json` artifact,
2. a bounded local decision to keep `cap.C5.pack_valid` unclaimed,
3. a packetized residual lane in `W024` rather than an implicit lingering blocker.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - this directory now carries the original retained-failure baseline, the replay-appliance-aware retained-failure baseline, the W019 distill-valid retained-failure baseline, the W021 gate baseline, the W022 direct-binding, shared-lifecycle, and decision baselines, and the W023 program-scope contract, host-sensitive family, and program-scope decision baselines
  - the next live lane after W023 is broader program-scope widening in W024
