# W052 Sensitivity And Derivative Seam

Status: `queued_successor`

Parent predecessor: `W050` capability vocabulary admission

Parent epic: allocate when W052 starts.

## 1. Purpose

W052 adds a spreadsheet-recognizable sensitivity capability over the calculation
graph.

The core idea is `Differentiable(parameter_set)`: a value can expose
`partial(parameter) -> RichValue` for declared parameters. Goal Seek, Solver,
and what-if analysis can then be built as graph queries instead of hidden
side-path loops.

## 2. Product Scope

The first W052 scope should be the capability and graph-walk seam, not a full
Solver product.

In scope for the first tranche:

1. `Differentiable(parameter_set)` capability identity,
2. derivative metadata imported from OxFunc,
3. OxFml threading of the capability through semantic plan/runtime,
4. OxCalc graph walk for sensitivity propagation,
5. replay-visible sensitivity query inputs and outputs.

Out of scope unless explicitly added:

1. full Goal Seek UI behavior,
2. full Solver behavior,
3. nonlinear optimization policy,
4. undocumented Excel solver internals.

## 3. Ownership

OxFunc owns per-kernel derivative metadata:

1. `Analytical(kernel)`,
2. `Finite(epsilon)`,
3. `Discontinuous`.

OxFml owns semantic-plan and runtime carriage.

OxCalc owns graph-level sensitivity queries and replay-visible orchestration.

## 4. First Work

The first W052 beads should:

1. write the minimum capability contract,
2. ask OxFunc for the first derivative metadata slice,
3. decide how `Discontinuous` is surfaced,
4. thread the capability through OxFml runtime/replay,
5. implement one bounded OxCalc graph-walk scenario,
6. record replay evidence for the query.

## 5. Closure Gate

W052 can close its first scope when:

1. at least one derivative-capable function path is exercised,
2. discontinuous or unsupported paths return typed outcomes,
3. sensitivity query results are replay-visible,
4. graph walk behavior is deterministic for the declared scope,
5. Goal Seek/Solver product work is either implemented or explicitly routed to
   a successor.

## 6. Status

Product status: queued successor work. No user-facing sensitivity feature is
implemented by W052 yet.

Evidence: W050 reserves the capability vocabulary shape.

Still open: derivative metadata, runtime carriage, graph walk, replay evidence,
and product-scope decision for Goal Seek/Solver.

Formal status: no proof claim.
