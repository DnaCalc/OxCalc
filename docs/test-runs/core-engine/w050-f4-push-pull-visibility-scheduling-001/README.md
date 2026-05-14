# W050 F4 Push/Pull Visibility Scheduling

Run id: `w050-f4-push-pull-visibility-scheduling-001`

Purpose: pin selectable TreeCalc scheduling over the same dependency graph and
prepared-callable identity surface.

The fixture starts with two formula observers over the same constant input:
node 3 computes `A + 3`, and node 4 computes `A * 10`. After the input changes,
`PushVisibilityBounded` selects only node 3 as the visible observer while
`PullFullClosure` sweeps both formula owners.

Primary validation command:

```powershell
cargo test -p oxcalc-core push_pull_scheduling -- --nocapture
```

The checked result records the semantic-equivalence boundary: visible observer
values match full closure under the same graph and prepared-callable identities,
while non-visible dirty formulas require periodic full-closure sweeps or
observer aging to avoid indefinite deferral.
