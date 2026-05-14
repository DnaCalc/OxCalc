# W050 G1 Rich Capability Vocabulary

Run id: `w050-g1-rich-capability-vocabulary-001`

Purpose: pin the initial typed rich-value capability vocabulary and replay
identity rules for `RichValueHole` successor work.

Primary validation command:

```powershell
cargo test -p oxcalc-core rich_value_capability -- --nocapture
```

The artifact records the initial `Indexable`, `Enumerable`, `Shaped`, and
`Materialisable` selector keys, their typed parameters, the stable required-set
identity rule, and the additive extension rule. It does not activate any
rich-value-producing kernel.
