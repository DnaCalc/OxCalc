# TreeCalc Scale Runs

This directory is for procedural scale/performance artifacts emitted by:

```powershell
cargo run --release -p oxcalc-tracecalc-cli -- treecalc-scale <profile> <run-id> [options]
```

The runner does not check in million-node fixtures. It generates the model in memory, times the major phases, validates compact closed-form expectations, and writes:

- `run_summary.json`
- `phase_timings.json`
- `validation_summary.json`
- `model_profile.json`

Current release-mode baseline results are summarized in `BASELINE_2026-05-04.md`.

Use `--recalc-rounds N` to repeat the synthetic calculation/reference-visit phase while keeping the same generated dependency model. This is useful when descriptor and graph phases are already large enough, but calc-phase timing needs to be amplified into the seconds range.

## Profiles

### `grid-cross-sum`

Logical grid with left constants and top constants. Each interior formula is:

```text
left_row + top_col
```

The edit changes `left[0]` and `top[0]`. The final aggregate is checked by:

```text
cols * sum(left_after) + rows * sum(top_after)
```

Large run:

```powershell
cargo run --release -p oxcalc-tracecalc-cli -- treecalc-scale grid-cross-sum million_grid --rows 1000 --cols 1000 --left-delta 7 --top-delta 11
```

### `fanout-bands`

One anchor band feeds many calculation nodes. Each formula is:

```text
SUM(anchor_0, ..., anchor_n)
```

The edit changes `anchor[0]`. The final aggregate is checked by:

```text
formula_count * sum(anchor_after)
```

This profile increases descriptor lowering and dependency graph edge volume.

Large run:

```powershell
cargo run --release -p oxcalc-tracecalc-cli -- treecalc-scale fanout-bands million_fanout --nodes 1000000 --fanout 16 --left-delta 7
```

Calc-amplified run:

```powershell
cargo run --release -p oxcalc-tracecalc-cli -- treecalc-scale fanout-bands million_fanout_calc --nodes 1000000 --fanout 8 --left-delta 7 --recalc-rounds 128
```

### `dynamic-indirect-stripes`

Logical grid with the same static base as `grid-cross-sum`, plus an INDIRECT-shaped dynamic potential carrier:

```text
(left_row + top_col) + INDIRECT(dynamic_potential_selector)
```

The static base sum remains closed-form checkable. The dynamic carrier is expected to surface as `DynamicPotentialReference` diagnostics rather than static dependency edges.

Large run:

```powershell
cargo run --release -p oxcalc-tracecalc-cli -- treecalc-scale dynamic-indirect-stripes million_indirect --rows 1000 --cols 1000 --selector-period 64 --left-delta 7 --top-delta 11
```

### `relative-rebind-churn`

One anchor band feeds many formulas through relative paths rather than direct node ids:

```text
SUM(../A0, ..., ../An)
```

The scale runner times an extra soft-reference phase by renaming `A0` and deriving rebind seeds against the successor snapshot. The expected result is one rebind seed for every formula owner, while the static recalc remains closed-form checkable with the same anchor-sum equation as `fanout-bands`.

Large run:

```powershell
cargo run --release -p oxcalc-tracecalc-cli -- treecalc-scale relative-rebind-churn million_relative_rebind --nodes 1000000 --fanout 8 --left-delta 7
```

## Timed Phases

- `model_build_structural_snapshot_and_formula_catalog`
- `dependency_descriptor_lowering`
- `dependency_graph_build_and_cycle_scan`
- `soft_reference_update_rebind_seed_derivation`
- `invalidation_closure_derivation`
- `synthetic_closed_form_recalc`
- `validation_checks`

## Three Additional Test Directions

- `deep-chain-checkpoints`: a long affine chain with periodic checkpoint totals, for propagation depth and stack/queue behavior.
- `sparse-hotspot-edit-storm`: many independent regions plus a few shared hot constants, for hot/cold invalidation contrast.
- `relative-rebind-churn`: heavy `RelativePath` and `SiblingOffset` formulas followed by rename/move edits, for rebind-required scaling separate from recalc-only scaling.
