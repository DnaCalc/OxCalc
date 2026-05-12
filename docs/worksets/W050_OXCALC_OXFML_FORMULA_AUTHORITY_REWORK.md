# W050 OxCalc/OxFml Formula Authority Rework

Status: `open_planning`

Parent predecessor: `W048` single-host scoped circular-reference closure

Parent epic: `calc-cwpl`

## 1. Purpose

W050 re-establishes and hardens the relationship between OxCalc and OxFml for formula handling.

Working principle:

1. OxFml owns formula semantics: parsing, binding, function/operator evaluation, array/spill values, coercion, volatile behavior, and returned value surfaces.
2. OxCalc owns calculation-engine semantics around those formula results: structural graph state, dependency descriptors, invalidation, CTRO overlays, cycle policy, candidate/publication state, replay evidence, and host/evaluator fact plumbing.
3. TreeCalc should not expose a local formula AST for spreadsheet expressions. Formula bindings should be either empty/no-formula structural records or OxFml-deferred formula source plus dependency/evaluator-fact carriers. Existing `TreeFormula::Literal`, `TreeFormula::Binary`, and `TreeFormula::FunctionCall` are rework targets, not desired product surfaces.

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
| `TreeFormula::Literal` | legacy fixture/source convenience | Remove or quarantine; literals belong in OxFml source or structural constants, not a formula AST. |
| `TreeFormula::Binary` with `Add/Subtract/Multiply/Divide` | legacy source-string construction for OxFml | Remove or convert to OxFml-deferred source; OxCalc should not publish operator syntax as an owned AST. |
| `TreeFormula::FunctionCall` | legacy arbitrary function-name source construction for OxFml | Remove or convert to OxFml-deferred source; OxCalc should not publish function syntax as an owned AST. |
| `TreeFormula::RawOxfml` | direct OxFml source carriage with reference carriers | Keep as the preferred formula-bearing surface, or rename to make delegation explicit. |
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
| `calc-cwpl.5` | remove/quarantine the local `TreeFormula` semantic AST surface |

## 5. Required Work

1. Inventory every place OxCalc appears to implement formula/operator/function semantics.
2. Separate three concepts in docs and code comments:
   - formula source carriage;
   - dependency/evaluator-fact projection;
   - formula semantic evaluation.
3. Replace the local formula AST direction with an empty/no-formula or OxFml-deferred model.
4. Update TreeCalc fixture guidance to prefer OxFml-deferred formula source for user-facing formula examples, especially arrays/spills and modern Excel functions.
5. Add tests proving TreeCalc routes formula evaluation through OxFml and treats returned values opaquely at the OxCalc layer.
6. Audit `ShapeTopology`, `DynamicPotential`, `HostSensitive`, and `CapabilitySensitive` carriers so they represent evaluator/host facts, not OxCalc formula implementations.
7. Remove or quarantine any test that implies OxCalc computes dynamic arrays locally.
8. Update W047/W048 showcase wording if it implies TreeCalc implements spreadsheet function semantics rather than graph/publication semantics.

## 6. Exit Gate

W050 may close only when:

1. the OxCalc/OxFml formula authority boundary is explicitly documented;
2. local formula AST variants for literals, operators, and function calls are removed or quarantined behind an explicitly non-product fixture adapter;
3. all code paths that look like formula semantic evaluation are either removed or routed to OxFml;
4. tests prove `TreeCalc` delegates formula evaluation to OxFml for representative formulas;
5. dynamic-array examples are represented as OxFml result surfaces or explicit future work, not OxCalc-local array plumbing;
6. W047/W048 showcase and workset docs use boundary-accurate language.

## 7. Three-Axis Status

- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - code inventory;
  - removal/quarantine of local formula AST variants;
  - fixture policy;
  - docs/showcase wording repair;
  - OxFml opaque-result tests.
