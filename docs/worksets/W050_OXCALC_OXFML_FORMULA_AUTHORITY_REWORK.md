# W050 OxCalc/OxFml Formula Authority Rework

Status: `open_planning`

Parent predecessor: `W048` single-host scoped circular-reference closure

Parent epic: `calc-cwpl`

## 1. Purpose

W050 re-establishes and hardens the relationship between OxCalc and OxFml for formula handling.

Working principle:

1. OxFml owns formula semantics: parsing, binding, function/operator evaluation, array/spill values, coercion, volatile behavior, and returned value surfaces.
2. OxCalc owns calculation-engine semantics around those formula results: structural graph state, dependency descriptors, invalidation, CTRO overlays, cycle policy, candidate/publication state, replay evidence, and host/evaluator fact plumbing.
3. TreeCalc may carry fixture-facing formula syntax (`TreeFormula`) only as a source/fixture convenience or dependency-carrier projection. It must not become an independent spreadsheet formula evaluator.

## 2. Triggering Observation

A dynamic-array example was requested:

```text
A: =RANDARRAY(5,5)
B: =A+1
C: =SUM(A, B)
D: =INDEX(A, 2, 2)
```

The intended architecture is that TreeCalc calls OxFml for `=RANDARRAY(5,5)` and receives an opaque array-valued result surface. OxCalc should not locally evaluate the array, scalarize the spill, or implement `RANDARRAY`, `SUM`, `INDEX`, or array arithmetic. OxCalc should only consume the value/effect/dependency facts exposed through the OxFml seam and route them through invalidation, CTRO, publication, and replay.

A temporary boundary test that represented `RANDARRAY(5,5)` as an OxCalc `ShapeTopology` carrier was removed because it risked suggesting product-level dynamic-array plumbing in OxCalc rather than an OxFml-owned formula result.

## 3. Current Code Surfaces To Audit

| Surface | Current role | Rework question |
| --- | --- | --- |
| `TreeFormula::Literal` | fixture/source convenience | Should this remain only fixture syntax? |
| `TreeFormula::Binary` with `Add/Subtract/Multiply/Divide` | source-string construction for OxFml | Ensure no local semantic evaluation remains or reappears. |
| `TreeFormula::FunctionCall` | arbitrary function-name source construction for OxFml | Ensure function semantics are fully delegated to OxFml. |
| `TreeFormula::RawOxfml` | direct OxFml source carriage with reference carriers | Prefer this for formula examples whose syntax matters. |
| `TreeReference::*` | dependency-carrier projection and fixture binding | Clarify which carriers are source references vs host/evaluator facts. |
| `evaluate_via_oxfml` | current TreeCalc formula evaluation path | Treat as authoritative evaluation boundary. |
| `formula_allows_lazy_residual_publication` | special-case residual handling for `IF` | Audit for architectural leakage. |
| W047/W048 CTRO/cycle fixtures | many use structured TreeFormula carriers | Classify as fixture scaffolding or convert key cases to RawOxfml. |

## 4. Bead Path

Parent epic: `calc-cwpl`

| Bead | Purpose |
| --- | --- |
| `calc-cwpl.1` | inventory OxCalc formula-looking code paths and classify source carriage vs semantic leakage |
| `calc-cwpl.2` | repair TreeCalc fixture policy around `RawOxfml` and formula delegation |
| `calc-cwpl.3` | add OxFml delegation / opaque result tests |
| `calc-cwpl.4` | repair W047/W048 showcase formula-boundary wording |

## 5. Required Work

1. Inventory every place OxCalc appears to implement formula/operator/function semantics.
2. Separate three concepts in docs and code comments:
   - formula source carriage;
   - dependency/evaluator-fact projection;
   - formula semantic evaluation.
3. Update TreeCalc fixture guidance to prefer `RawOxfml` for user-facing formula examples, especially arrays/spills and modern Excel functions.
4. Add tests proving TreeCalc routes formula evaluation through OxFml and treats returned values opaquely at the OxCalc layer.
5. Audit `ShapeTopology`, `DynamicPotential`, `HostSensitive`, and `CapabilitySensitive` carriers so they represent evaluator/host facts, not OxCalc formula implementations.
6. Remove or quarantine any test that implies OxCalc computes dynamic arrays locally.
7. Update W047/W048 showcase wording if it implies TreeCalc implements spreadsheet function semantics rather than graph/publication semantics.

## 6. Exit Gate

W050 may close only when:

1. the OxCalc/OxFml formula authority boundary is explicitly documented;
2. all code paths that look like formula semantic evaluation are either removed, routed to OxFml, or documented as fixture-only syntax construction;
3. tests prove `TreeCalc` delegates formula evaluation to OxFml for representative formulas;
4. dynamic-array examples are represented as OxFml result surfaces or explicit future work, not OxCalc-local array plumbing;
5. W047/W048 showcase and workset docs use boundary-accurate language.

## 7. Three-Axis Status

- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - code inventory;
  - fixture policy;
  - docs/showcase wording repair;
  - OxFml opaque-result tests.
