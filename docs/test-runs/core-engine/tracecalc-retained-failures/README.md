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

It is emitted by:
1. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w016-sequence4-retained-failure-baseline`
2. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w018-retained-replay-appliance-bundle-baseline`

The current retained-failure baseline carries:
1. one replay-valid retained-local witness family,
2. one explanatory-only retained witness case,
3. one quarantined retained witness case.

The replay-appliance-aware retained-failure baseline additionally carries:
1. additive replay-appliance bundle roots,
2. bundle-validation artifacts,
3. per-case explain records bound to retained-failure lifecycle and mismatch targets.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - this directory now carries both the original retained-failure baseline and the replay-appliance-aware retained-failure baseline
  - broader retained-local mismatch coverage, pack-grade promotion, and later shared replay governance remain later lanes
