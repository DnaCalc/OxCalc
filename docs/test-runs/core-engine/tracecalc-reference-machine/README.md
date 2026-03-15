# TraceCalc Reference-Machine Runs

This directory holds emitted self-contained harness and oracle runs for the `TraceCalc` Stage 1 corpus.

## Purpose
1. preserve deterministic emitted artifacts for the first harness and oracle implementation slice,
2. provide a checked-in conformance baseline for the current Stage 1 engine floor,
3. give W009, W011, W012, and W013 concrete evidence paths for replay, pinned-view, publish, reject, and overlay behavior.

## Baseline Run
The current checked-in baseline is:
1. `w013-sequence-a-baseline`

It was emitted by:
1. `dotnet run --project src/OxCalc.TraceCalc.Tool -- w013-sequence-a-baseline`

The baseline run currently covers:
1. candidate-result versus publication separation,
2. reject-is-no-publish,
3. pinned-view stability,
4. dynamic dependency shape updates,
5. overlay retention,
6. verification-clean without publication,
7. a first scale-seed scenario.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - this directory currently holds the first deterministic baseline run only
  - richer generated corpus, replay-pack export, and later engine-versus-oracle series remain later lanes
