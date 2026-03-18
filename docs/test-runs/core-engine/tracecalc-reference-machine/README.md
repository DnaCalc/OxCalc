# TraceCalc Reference-Machine Runs

This directory holds emitted self-contained harness and oracle runs for the `TraceCalc` Stage 1 corpus.

## Purpose
1. preserve deterministic emitted artifacts for the first harness and oracle implementation slice,
2. provide a checked-in conformance baseline for the current Stage 1 engine floor,
3. give W009, W011, W012, and W013 concrete evidence paths for replay, pinned-view, publish, reject, and overlay behavior.

## Baseline Runs
Historical checked-in baseline:
1. `w013-sequence-a-baseline`

Historical semantic-anchor baseline:
1. `w014-stage1-widening-baseline`

Current active regenerable Rust baseline:
1. `w017-rust-parity-baseline`

Current replay-appliance-aware baseline:
1. `w018-replay-appliance-bundle-baseline`

Current additive witness-seed run:
1. `w016-sequence1-witness-seeds`

Current additive witness-lifecycle run:
1. `w016-sequence2-lifecycle-outcomes`

They were emitted by:
1. a historical pre-Rust implementation path for `w013-sequence-a-baseline`
2. a historical pre-Rust implementation path for `w014-stage1-widening-baseline`
3. `cargo run -p oxcalc-tracecalc-cli -- w017-rust-parity-baseline`
4. `cargo run -p oxcalc-tracecalc-cli -- w018-replay-appliance-bundle-baseline`
5. `cargo run -p oxcalc-tracecalc-cli -- w016-sequence1-witness-seeds`
6. `cargo run -p oxcalc-tracecalc-cli -- w016-sequence2-lifecycle-outcomes`

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
Rust-specific parity runs must use distinct run ids.
They must not silently rewrite `w014-stage1-widening-baseline`.

Role split:
1. `w014-stage1-widening-baseline` remains the carried semantic anchor for Stage 1 parity comparison.
2. `w017-rust-parity-baseline` is the current regenerable implementation baseline produced by the active Rust workspace.
3. `w018-replay-appliance-bundle-baseline` is the current replay-appliance-aware ordinary-run baseline and must remain parity-equivalent to `w017-rust-parity-baseline` on the carried conformance surface.
4. `w016-sequence1-witness-seeds` is the first additive retained-witness seed run and must remain parity-equivalent to `w017-rust-parity-baseline` on the carried conformance surface.
5. `w016-sequence2-lifecycle-outcomes` is the current additive retained-witness lifecycle run and must remain parity-equivalent to `w017-rust-parity-baseline` on the carried conformance surface.

The carried baseline comparison contract is enforced by:
1. `scripts/compare-tracecalc-run.ps1`

The current carried parity proof is:
1. `pwsh ./scripts/compare-tracecalc-run.ps1 -CandidateRunId w017-rust-parity-baseline -BaselineRunId w014-stage1-widening-baseline`
2. `pwsh ./scripts/compare-tracecalc-run.ps1 -CandidateRunId w018-replay-appliance-bundle-baseline -BaselineRunId w017-rust-parity-baseline`
3. `pwsh ./scripts/compare-tracecalc-run.ps1 -CandidateRunId w016-sequence1-witness-seeds -BaselineRunId w017-rust-parity-baseline`
4. `pwsh ./scripts/compare-tracecalc-run.ps1 -CandidateRunId w016-sequence2-lifecycle-outcomes -BaselineRunId w017-rust-parity-baseline`

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - this directory now holds the carried historical baselines, one active regenerable Rust baseline, one checked-in replay-appliance-aware baseline, and additive W016 witness-seed and witness-lifecycle runs
  - richer generated corpus, broader mismatch-family explain coverage, replay-pack export, replay-valid reduced-witness widening, and later engine-versus-oracle series remain later lanes
