# W050 G2 RichValueHole Capability Requirements

Run id: `w050-g2-rich-value-hole-capability-requirements-001`

Purpose: pin `RichValueHole(required_capability_set)` as a typed hole-taxonomy
member whose required capability set participates in plan-template identity.

Primary validation command:

```powershell
cargo test -p oxcalc-core rich_value_hole -- --nocapture
```

The artifact records typed required-set keys and proves that changing the
required capability set changes the plan-template key material. It does not
claim any current production path emits rich-value holes or that any rich-value
kernel is active.
