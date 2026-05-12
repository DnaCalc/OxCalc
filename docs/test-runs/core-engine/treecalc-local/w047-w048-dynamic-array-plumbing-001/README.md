# TreeCalc Dynamic Array Plumbing Example

Status: `passed_boundary_test`

Command:

```powershell
cargo test -p oxcalc-core --test treecalc_dynamic_array_plumbing -- --nocapture
```

Formula surface modeled:

```text
A: =RANDARRAY(5,5)
B: =A+1
C: =SUM(A, B)
D: =INDEX(A, 2, 2)
```

Current TreeCalc meaning:

- `A` is represented as a `ShapeTopology` runtime carrier because native dynamic-array/spill evaluation is not implemented in the local sequential TreeCalc engine.
- `B`, `C`, and `D` are represented with ordinary dependency edges to show the graph plumbing that would be needed after `A` has a spill value surface.
- The run rejects safely with `HostInjectedFailure` and emits `runtime_effect.shape_topology_reference` plus a `ShapeTopology` overlay.
- No publication bundle is emitted.

This is intentionally a boundary/plumbing test, not a claim that TreeCalc currently computes `RANDARRAY`, array addition, aggregate array sums, or `INDEX` over a spill.
