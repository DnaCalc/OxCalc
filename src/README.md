# OxCalc Workspace

This workspace is the active OxCalc implementation lane.

## Crates
- `oxcalc-core`
  - Rust home for the structural snapshot kernel, coordinator/publication baseline, and recalc/overlay baseline.
- `oxcalc-tracecalc`
  - Rust home for `TraceCalc` contracts, scenario loading, runner support, and engine/oracle comparison surfaces.
- `oxcalc-tracecalc-cli`
  - Rust CLI host for the self-contained `TraceCalc` execution lane.

## Boundary Rules
1. Crate boundaries follow OxCalc semantic ownership, not older non-Rust object layouts.
2. `oxcalc-core` does not depend on CLI or host concerns.
3. `oxcalc-tracecalc` depends on `oxcalc-core`, not the other way around.
4. `oxcalc-tracecalc-cli` is the outer host layer and should stay thin.
5. All crates must forbid `unsafe` and stay warning-clean under the declared validation commands.

## Current State
This workspace is the active implementation home for the current declared OxCalc scope.
