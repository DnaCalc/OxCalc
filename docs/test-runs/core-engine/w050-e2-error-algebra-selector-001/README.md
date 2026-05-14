# W050 E2 Error Algebra Selector

Run id: `w050-e2-error-algebra-selector-001`

Purpose: pin the first OxCalc-local `ErrorAlgebra` selector record, canonical
Excel legacy worksheet-error precedence order, extension rule, and
handoff-ready exact clause language for CALC-003.

Artifact:
- `selector_artifact.json` records the required profile fields, replay key
  format, canonical precedence order, and exact clauses.

Validation commands:
- `cargo test -p oxcalc-core error_algebra -- --nocapture`
- `cargo test -p oxcalc-core`

Result: the checked artifact matches the Rust selector/profile surface and the
canonical precedence resolver chooses the earliest error in the profile order.
