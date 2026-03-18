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

It is emitted by:
1. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w016-sequence4-retained-failure-baseline`
2. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w018-retained-replay-appliance-bundle-baseline`
3. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w019-distill-and-pack-candidate-baseline`

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

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - this directory now carries the original retained-failure baseline, the replay-appliance-aware retained-failure baseline, and the W019 distill-valid retained-failure baseline
  - broader retained-local mismatch coverage, pack-grade promotion, and later shared replay governance remain later lanes
