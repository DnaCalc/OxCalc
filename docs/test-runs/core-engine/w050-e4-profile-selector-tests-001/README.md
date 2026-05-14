# W050 E4 Profile Selector Tests

This evidence root records the combined OxCalc-local selector checks for
`CorrectnessFloorProfile`.

Validation command:

```powershell
cargo test -p oxcalc-core correctness_floor_profile_selector -- --nocapture
```

The checked manifest is compared against runtime-generated selector defaults,
explicit selector selection, and mismatch diagnostics.
