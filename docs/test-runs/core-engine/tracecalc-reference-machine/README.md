# TraceCalc Reference-Machine Runs

This directory holds emitted self-contained harness and oracle runs for the `TraceCalc` Stage 1 corpus.

## Purpose
1. preserve deterministic emitted artifacts for the first harness and oracle implementation slice,
2. provide a checked-in conformance baseline for the current Stage 1 engine floor,
3. give W009, W011, W012, and W013 concrete evidence paths for replay, pinned-view, publish, reject, and overlay behavior.

## Baseline Runs
Historical checked-in baseline:
1. `w013-sequence-a-baseline`

Current active checked-in baseline:
1. `w014-stage1-widening-baseline`

Rust parity evidence run:
1. `w017-rust-parity-baseline`

They were emitted by:
1. a historical pre-Rust implementation path for `w013-sequence-a-baseline`
2. a historical pre-Rust implementation path for `w014-stage1-widening-baseline`
3. `cargo run -p oxcalc-tracecalc-cli -- w017-rust-parity-baseline`

The active widened baseline currently covers:
1. candidate-result versus publication separation,
2. reject-is-no-publish,
3. pinned-view stability,
4. dynamic dependency shape updates,
5. overlay retention,
6. verification-clean without publication,
7. multi-node DAG publication,
8. publication-fence reject,
9. artifact-token reject,
10. fallback and re-entry,
11. first SCC-oriented rejection,
12. a first scale-seed scenario.

## Rust Parity Direction
During `W017`, Rust-specific parity runs must use distinct run ids.
They must not silently rewrite `w014-stage1-widening-baseline`.

The carried baseline comparison contract is enforced by:
1. `scripts/compare-tracecalc-run.ps1`

The current carried parity proof is:
1. `pwsh ./scripts/compare-tracecalc-run.ps1 -CandidateRunId w017-rust-parity-baseline -BaselineRunId w014-stage1-widening-baseline`

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - this directory holds carried deterministic baselines plus one Rust parity evidence run
  - richer generated corpus, replay-pack export, and later engine-versus-oracle series remain later lanes
