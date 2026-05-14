# W050 F5 O(k) Differential Evidence

Run id: `w050-f5-ok-differential-evidence-001`

Purpose: pin deterministic trace-count evidence for O(k)-bounded TreeCalc
work after a single input changes in a hundred-formula model.

The fixture has 100 formula owners. Eight formulas depend on node 2, the
changed input, and ninety-two formulas depend on node 3, the stable input.
After the initial publication, node 2 changes from `2` to `5`.

Primary validation command:

```powershell
cargo test -p oxcalc-core o_k_differential_evidence -- --nocapture
```

The checked artifact records two post-change paths: pull full-closure visits
all 100 formula owners but invokes OxFml only for the changed fan-out while
reusing 92 cached edge values; push visibility-bounded scheduling selects only
the eight visible dirty observers over the same dependency graph and
prepared-callable identities.
