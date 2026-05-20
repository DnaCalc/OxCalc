# W055 Circular References And Iterative Calculation Excel-Match Closure

Status: `pre_planning_doctrine_blocked`

Parent predecessor: `W048` (single-host scoped circular dependency calculation processing)

Parent epic: TBD (allocated only after explicit activation)

## 1. Purpose

W055 exists to turn circular references and iterative calculation from a narrow evidence slice into a product-grade Excel-match feature area.

The target answer for this workset is:

> Circular references and iterative calculation are implemented for the declared Excel-match scope. The implementation matches Excel across a broad tested scenario spread, including edge cases that Excel leaves implicit. Formal proof is not yet required for the product claim, but definitions, contracts, and formalization obligations are packetized for successor formal work.

This workset is intentionally not an implementation start. It is recorded now so the feature area has a clear future product target instead of remaining trapped between an over-broad impossible bar and a narrow fixture-slice claim.

## 2. Activation Rule

Do not create execution beads, change code, run Excel probes, or widen evidence under W055 until the user explicitly activates this workset after the planned doctrine/meta-work.

Activation requires:

1. a doctrine decision that separates product feature claims from full formal proof claims,
2. an explicit declared Excel-match scope,
3. an explicit exclusion policy for any behavior not included in that declared scope,
4. a concrete evidence plan,
5. a bead rollout that preserves product accountability rather than hiding behind partial-status language.

## 3. Scope

W055 includes ordinary and difficult Excel circular-reference behavior. The scope must not silently exclude the hard cases.

In scope:

1. direct self-references,
2. two-node and larger structural cycles,
3. guarded activation cycles,
4. iterative calculation with `MaxIterations` and `MaxChange`,
5. edit-order and calculation-chain sensitivity,
6. initial-vector behavior for blank, zero, numeric, text, logical, error, and prior calculated states,
7. manual versus automatic recalculation interactions,
8. full recalculation and workbook reopen behavior where calculation-chain state matters,
9. CTRO and dynamic-reference cycles such as `INDIRECT`-like references,
10. dynamic-array spill cycles, including spill growth, spill contraction, blocked spills, and spill-cycle release,
11. data-table circular-reference behavior and its special recalculation rules,
12. external workbook link cycles, including closed-source, stale-link, missing-link, and refresh-order variants,
13. volatile and externally invalidated functions inside cycle regions,
14. downstream dependent publication after convergence, max-iteration terminal states, rejection, release, or re-entry,
15. multi-threaded and cross-thread calculation variants, including nondeterministic-looking Excel behavior that must be reduced to explicit profile rules or documented compatibility limits,
16. least-significant-bit numeric parity for tested numeric surfaces.

Out of scope until explicitly added:

1. unsupported proprietary internals or reverse engineering,
2. claims not backed by reproducible public documentation or black-box observation,
3. formal proof as a prerequisite for the product feature claim.

## 4. Required Outcome

W055 should leave the repo with a clear feature-area answer, not a defensive status cloud.

Required product claim shape:

1. supported circular-reference modes are named by profile,
2. supported Excel-match scope is named directly,
3. exclusions are listed as exclusions, not hidden as "partial" generalities,
4. ordinary users and downstream hosts can tell whether circular references and iterative calculation are supported,
5. evidence links show the scenario spread tested against Excel,
6. implementation paths are general algorithms where possible, not fixture-keyed result tables,
7. broad compatibility claims are bounded by observed Excel versions and profile declarations,
8. formalization status is reported separately from product implementation status.

## 5. Evidence Plan

The future evidence plan should include:

1. an Excel observation matrix with scenario families, Excel version/build, calculation mode, thread mode, workbook save/open state, and exact observed values,
2. a scenario taxonomy for ordinary cycles, dynamic-reference cycles, spill cycles, data tables, external links, volatile/external invalidation, and cross-thread variants,
3. deterministic reproduction scripts using approved tooling only,
4. TraceCalc fixtures that model the observed semantics,
5. TreeCalc/core fixtures that exercise the actual implementation path,
6. cross-run conformance summaries that compare Excel observations, TraceCalc reference behavior, and TreeCalc/core behavior,
7. a failure ledger that distinguishes implementation bugs, Excel-version divergence, unsupported product scope, and formalization gaps.

## 6. Implementation Expectations

Implementation should move beyond the W048 fixture slice.

Required implementation direction:

1. general cycle-region construction and stable identity,
2. declared member ordering rules per profile,
3. declared initial-vector construction per profile,
4. iterative update engine with explicit update model, stop metric, iteration bound, and terminal-state behavior,
5. atomic cycle-region publication when the active profile publishes terminal values,
6. no-publication rejection when the active profile rejects the candidate,
7. downstream invalidation and recomputation after accepted cycle values or cycle release,
8. graph/materialized sidecars that expose cycle members, order, prior values, iteration summaries, terminal state, and publication decision,
9. explicit handling for dynamic-array spill and data-table cycle surfaces rather than treating them as generic cycles without evidence,
10. external-link and thread-mode behavior represented as profile dimensions or explicit compatibility limitations.

## 7. Formalization Boundary

W055 should not make full formal proof a blocker for the product feature claim.

It must, however, leave formalization material strong enough for W049 or a successor formal lane:

1. cycle-region definitions,
2. graph-layer definitions over structural, published-effective, and candidate-effective graphs,
3. profile definitions for non-iterative, Excel-match iterative, deterministic iterative, and any compatibility-limited variants,
4. transition contracts for reject, publish, release, and re-entry,
5. replay-visible trace/event contracts,
6. proof obligations for future Lean/TLA/model work.

Formal proof status must be reported separately from implementation status.

## 8. Dependencies And Coordination

Depends on:

1. `W048` for the existing single-host scoped evidence slice and cycle vocabulary,
2. `W050` for the current formula-authority and prepared-runtime seam,
3. `W051` where sparse/range surfaces affect data tables, dynamic-array spill behavior, or range-backed cycle fixtures.

Expected upstream coordination:

1. `OxFml` for formula/evaluator-facing behavior, dynamic arrays/spills, external references, and trace/replay surfaces,
2. `OxFunc` for function semantics inside cycle regions, volatility, external invalidation, numeric precision, and data-table-adjacent kernel behavior,
3. Foundation only for doctrine/profile changes and conformance-pack promotion.

## 9. Closure Gate

Closure criteria are intentionally product-facing.

W055 can claim the feature area is implemented for its declared scope only when:

1. declared Excel-match scope is explicit,
2. all in-scope scenario families have Excel observations or exact blockers accepted before activation,
3. TraceCalc and TreeCalc/core both match the accepted Excel observation set,
4. implementation uses general profile-driven behavior for the declared scope, with any fixture-specific behavior removed or justified as test-only,
5. numeric surfaces match to the declared precision, including least-significant-bit checks where claimed,
6. data-table, dynamic-array spill, external-link, and thread-mode behaviors are either implemented and evidenced or explicitly excluded from the declared product scope before the claim,
7. profile selectors and capability manifests state the supported cycle behavior,
8. spec text and replay contracts are updated,
9. formal definitions and future proof obligations are packetized,
10. final reporting separates product implementation status from formal proof status.

## 10. Status Surface

- execution_state: `planned`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - doctrine/meta-work before activation;
  - Excel observation matrix;
  - general implementation;
  - dynamic-array spill cycles;
  - data-table cycles;
  - external workbook link cycles;
  - cross-thread and multithread variants;
  - conformance comparison;
  - spec consolidation;
  - formalization packetization.
