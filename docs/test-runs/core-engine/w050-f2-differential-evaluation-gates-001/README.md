# W050 F2 Differential Evaluation Gates

Run id: `w050-f2-differential-evaluation-gates-001`

Purpose: pin the first OxCalc-local invocation-time differential evaluation
gate over the F1 per-edge value cache.

Artifact:
- `run_artifact.json` records the cache key fields, semantic reuse guard,
  deterministic path exclusions, and two validation cases.

Validation commands:
- `cargo test -p oxcalc-core differential_evaluation_gate -- --nocapture`
- `cargo test -p oxcalc-core treecalc -- --nocapture`

Result: a default verification rerun with the same per-edge key reuses the
cached value and suppresses OxFml invocation without emitting a publication
bundle. An upstream-publication rerun with the same fingerprint bypasses the
cache, invokes OxFml, and publishes the changed value.
