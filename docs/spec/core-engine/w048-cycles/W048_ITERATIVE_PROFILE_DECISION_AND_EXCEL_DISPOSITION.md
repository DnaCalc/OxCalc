# W048 Iterative Profile Decision And Excel Disposition

Status: `decision_recorded_for_future_profile`

Machine-readable decision: `W048_ITERATIVE_PROFILE_DECISION.json`

## 1. Decision Summary

W048 keeps the current Stage 1 default profile as non-iterative cycle rejection:

- profile: `cycle.non_iterative_stage1`
- terminal state: reject candidate
- publication: whole-wave reject with no new cycle values
- prior values: previously published values may remain visible, but are not a new publication
- root/order: canonical diagnostic-only ordering

W048 does **not** admit an Excel-match iterative profile yet. Public Excel documentation exposes iteration controls, but not enough algorithmic detail to make root/order, initial vector, update model, and terminal edge cases replayable. The first COM observation packet (`w048-excel-cycles-001`) captured 12 observations but did not yield a stable `Application.CircularReference` root, so Excel-match iterative semantics remain observation-gated.

## 2. Future Non-Default Iterative Profile

The future explicit opt-in OxCalc profile is:

- profile: `cycle.iterative_deterministic_v0`
- root policy: `canonical_node_id_root`
- member order: ascending stable node id with artifact-token tiebreak
- initial vector: last published numeric value, or zero if absent
- nonnumeric/error prior: reject before iteration
- update model: `jacobi_snapshot_region_function`
- stop metric: maximum absolute member delta
- default threshold: `0.001`
- default max iterations: `100`
- converged terminal state: accept candidate and publish the whole cycle region atomically
- non-converged/divergent/oscillating/non-numeric terminal state: reject candidate with no publication

## 3. Excel Disposition

Excel mapping for now:

| Surface | Disposition |
| --- | --- |
| non-iterative visible retained values | observed but not mapped to OxCalc publication |
| circular-reference root/report cell | inconclusive in first COM packet |
| iterative member order | deferred/unobserved |
| iterative initial vector | deferred/unobserved |
| iterative stop metric edge cases | deferred/unobserved |
| data-table cycles | out of W048 TreeCalc baseline |

This means `cycle.excel_match_iterative` is `not_admitted_yet`. Any future claim to match Excel must add probes for chain state, cold-open behavior, prior displayed values, order-sensitive SCCs, oscillation/divergence, and error/blank/nonnumeric values.

## 4. Semantic Equivalence Obligation

For `cycle.iterative_deterministic_v0`, scheduler or coordinator optimizations are allowed only if observable results are invariant:

1. same cycle-region membership;
2. same initial vector;
3. same Jacobi snapshot sequence up to terminal step;
4. same terminal state;
5. same accepted/rejected candidate decision;
6. same publication bundle when converged;
7. same diagnostics modulo timing-only fields.

## 5. Review Evidence

Validation command:

```powershell
python scripts/check-w048-iterative-profile-decision.py
```

The checker verifies that the decision JSON names the default non-iterative profile, Excel-match disposition, future profile root/order, initial vector, update model, stop metric, terminal states, publication rule, diagnostics, and semantic-equivalence obligation.

## 6. Three-Axis Status

- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - Excel-match iterative algorithm remains observation-gated.
  - No iterative engine implementation is admitted by this bead.
  - Formal profile determinism proof obligations route to `calc-zci1.5`.
  - Innovation profile comparison routes to `calc-zci1.8`.
