# W050 E3 Correctness-Floor Replay Hooks

Run id: `w050-e3-correctness-floor-replay-hooks-001`

Purpose: pin the OxCalc-local wave trace/replay hook for active
`NumericalReductionPolicy` and `ErrorAlgebra` selectors.

Artifact:
- `run_artifact.json` records the active correctness-floor profile, trace
  replay record, accepted replay case, and rejected replay cases for
  selector mismatches.

Validation commands:
- `cargo test -p oxcalc-core correctness_floor_replay -- --nocapture`
- `cargo test -p oxcalc-core`

Result: active selectors are replay-visible on `OxfmlRecalcWave` traces, and
replay validation rejects recorded selector values that differ from the active
profile.
