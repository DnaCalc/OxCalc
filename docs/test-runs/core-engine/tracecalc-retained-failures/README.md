# TraceCalc Retained-Failure Runs

This directory holds retained-failure and reduced-witness baseline artifacts for `TraceCalc`.

## Purpose
1. preserve deterministic retained-failure artifacts separate from the ordinary reference-machine run root,
2. keep replay-valid retained-local, explanatory-only, and quarantined witness outcomes explicit,
3. provide checked-in evidence for W016 without treating those artifacts as pack-grade replay evidence.

## Baseline Runs
Current retained-failure baseline:
1. `w016-sequence4-retained-failure-baseline`

It is emitted by:
1. `cargo run -p oxcalc-tracecalc-cli -- retained-failures w016-sequence4-retained-failure-baseline`

The current retained-failure baseline carries:
1. one replay-valid retained-local witness family,
2. one explanatory-only retained witness case,
3. one quarantined retained witness case.

## Status
- execution_state: complete
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: integrated
- open_lanes: []
