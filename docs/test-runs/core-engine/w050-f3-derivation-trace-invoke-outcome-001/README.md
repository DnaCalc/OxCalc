# W050 F3 Derivation Trace Invoke Outcome

Run id: `w050-f3-derivation-trace-invoke-outcome-001`

This checked artifact records the current OxCalc trace-mode opt-in path for
TreeCalc invocation through OxFml `PreparedCalls`. The fixture uses
`=LET(base,2,LAMBDA(delta,base+delta)(5))` so the trace includes template
selection, hole bindings, a root prepared-callable invocation with child
OxFml prepared calls, and kernel-returned values.

Primary validation command:

```powershell
cargo test -p oxcalc-core derivation_trace -- --nocapture
```
