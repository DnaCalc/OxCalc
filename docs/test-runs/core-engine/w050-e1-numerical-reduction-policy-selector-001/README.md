# W050 E1 Numerical Reduction Policy Selector

Run id: `w050-e1-numerical-reduction-policy-selector-001`

Purpose: pin the first OxCalc-local `NumericalReductionPolicy` selector record
and handoff-ready exact clause language for CALC-003.

Artifact:
- `selector_artifact.json` records the required profile fields, replay key
  format, initial selector variants, behavior flags, and exact clauses.

Validation commands:
- `cargo test -p oxcalc-core numerical_reduction -- --nocapture`
- `cargo test -p oxcalc-core`

Result: the checked artifact matches the Rust selector/profile surface and the
three exact CALC-003 numerical-reduction clauses.
