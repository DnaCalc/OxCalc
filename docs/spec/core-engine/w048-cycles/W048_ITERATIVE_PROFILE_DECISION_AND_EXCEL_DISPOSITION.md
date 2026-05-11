# W048 Iterative Profile Decision And Excel Disposition

Status: `reopened_active_profile_specified`

Machine-readable decision: `W048_ITERATIVE_PROFILE_DECISION.json`

Parent bead: `calc-zci1.12`

## 1. Decision Summary

W048 now has two explicitly separated iterative profiles:

1. `cycle.excel_match_iterative`
   - reopened W048 implementation target;
   - specified from black-box Excel observation packets;
   - carries exact blockers before any final broad Excel-match claim.
2. `cycle.iterative_deterministic_v0`
   - OxCalc opt-in deterministic profile;
   - intentionally clearer than Excel where Excel state is hidden or weakly observed;
   - remains useful for engine proof and scheduler-equivalence work.

The default W048 non-iterative behavior remains unchanged:

- profile: `cycle.non_iterative_stage1`
- terminal state: reject candidate
- publication: whole-wave reject with no new cycle values
- prior values: previously published values may remain visible, but are not a new publication
- root/order: canonical diagnostic-only ordering

## 2. Evidence Runs

| Run | Probe count | Role |
| --- | ---: | --- |
| `docs/test-runs/excel-cycles/w048-excel-cycles-001/observation.json` | 12 | first core packet |
| `docs/test-runs/excel-cycles/w048-excel-cycles-bitexact-001/observation.json` | 19 | expanded bit-exact packet |

Validation commands:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-excel-observation-packet.ps1 docs/test-runs/excel-cycles/w048-excel-cycles-001
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-excel-observation-packet.ps1 docs/test-runs/excel-cycles/w048-excel-cycles-bitexact-001 -MinimumProbeCount 19
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-iterative-profile-decision.ps1
```

## 3. Excel-Match Iterative Profile

Machine profile id: `cycle.excel_match_iterative`

Admission: `implementation_target_for_reopened_w048`

### 3.1 Region And Order

Observed candidate member-order policy:

1. use saved workbook calculation-chain order when available;
2. preserve workbook/edit-history sensitivity rather than canonicalizing by node id;
3. if no chain/order witness exists, report an exact blocker rather than inventing canonical Excel behavior.

Evidence:

| Probe | Chain/order | Visible result |
| --- | --- | --- |
| `excel_chain_edit_order_ab_001` | `B1,A1` | `A1=1`, `B1=2` |
| `excel_chain_edit_order_ba_001` | `A1,B1` | `A1=2`, `B1=1` |
| `excel_struct_three_node_001` | `C1,B1,A1` | `A1=2`, `B1=1`, `C1=3` |
| `excel_iter_three_node_order_001` | `C1,B1,A1` | `A1=102`, `B1=101`, `C1=103` |

### 3.2 Update Model

Observed candidate update model: `chain_ordered_sequential_region_update`.

The order probe `excel_iter_two_node_order_001` with `MaxIterations=1` yielded `A1=11`, `B1=22`, matching sequential A-then-B for that workbook/edit sequence and not the Jacobi snapshot alternative (`B1=2`).

### 3.3 Initial Vector

Current state: `partially_observed`.

Observed enough for implementation fixtures:

1. fresh self-cycle surfaces in these packets start from an empty/zero-like base;
2. chain-order probes produce deterministic terminal surfaces tied to workbook history;
3. fractional precision probe yields `0.33333333333333331` in `Value2`.

Not yet inferred strongly enough for final Excel-match claims:

1. numeric-prior self-cycle behavior is now observed for declared probes: prior numeric seeds do not survive formula assignment; assignment uses a zero/blank-derived base for tested formulas;
2. blank, text, and error prior states are now observed for declared probes: prior states do not survive formula assignment and assignment uses a zero/blank-derived base for tested formulas;
3. UI warning/report-cell interaction with initial vector remains blocked by root/report-cell evidence.

### 3.4 Stop Metric And Bound

Implementation target for falsification:

- stop metric: maximum absolute visible numeric delta across cycle members;
- default max change: `0.001`;
- numeric precision: IEEE-754 double `Value2` comparison unless a later Excel packet falsifies this;
- iteration bound: Excel `MaxIterations` parameter.

Observed `MaxIterations` values: `1`, `2`, `3`, `5`, `6`, `20`, `50`.

### 3.5 Terminal And Publication Policy

For `cycle.excel_match_iterative`:

1. convergence publishes the whole cycle region atomically, then downstream dependents recompute;
2. max-iteration terminal values are published when iteration is enabled and the fixture expected surface matches the Excel packet;
3. oscillation publishes the observed terminal value after the iteration bound for the Excel-match profile;
4. non-numeric/error cases remain blocked pending targeted probes;
5. dynamic release publishes the released acyclic candidate and downstream values.

The default non-iterative profile still rejects cycle candidates with no new cycle-value publication.

## 4. Falsification Fixtures

The profile is intentionally falsifiable. At minimum, TraceCalc and TreeCalc implementations must reproduce or explicitly block these Excel surfaces:

| Probe | Expected cells | Expected chain |
| --- | --- | --- |
| `excel_iter_two_node_order_001` | `A1=11`, `B1=22` | `B1,A1` |
| `excel_iter_three_node_order_001` | `A1=102`, `B1=101`, `C1=103` | `C1,B1,A1` |
| `excel_iter_fraction_precision_001` | `A1=0.33333333333333331` | `A1` |
| `excel_ctro_indirect_iterative_self_001` | `A1=1`, `B1="A1"` | `A1` |

## 5. Named Blockers Before Final Excel-Match Claim

These are not blockers for starting implementation; they are blockers for claiming broad final Excel compatibility.

| Blocker | Required unblock |
| --- | --- |
| `BLK-W048-EXCEL-ROOT` | capture report-cell/root behavior through UI warning or a public object-model surface that does not return null for circular references in current COM packets |
| `BLK-W048-EXCEL-VERSION` | repeat the falsification fixture set on a second Excel host/version before broad compatibility claims |
| `BLK-W048-EXCEL-MT` | run multi-threaded calculation variants or explicitly scope the Excel-match profile to single-threaded observations |

## 6. OxCalc Deterministic Iterative Profile

The opt-in deterministic profile remains:

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

## 7. Semantic Equivalence Obligation

For any iterative profile, scheduler or coordinator optimizations are allowed only if observable results are invariant:

1. same cycle-region membership;
2. same initial vector;
3. same update order/model sequence up to terminal step;
4. same terminal state;
5. same accepted/rejected candidate decision;
6. same publication bundle when values are published;
7. same diagnostics modulo timing-only fields.

## 8. Fresh-Eyes Review For `calc-zci1.12`

Review date: 2026-05-11

Review questions:

1. Does the spec overclaim final Excel compatibility from two COM packets?
2. Is every weakly observed behavior named as a blocker rather than hidden?
3. Is the profile falsifiable against concrete packet values?
4. Does the spec preserve the default non-iterative Stage 1 no-publication behavior?
5. Is the deterministic OxCalc profile separate from Excel-match behavior?

Findings:

1. The spec does not claim final broad Excel compatibility; it declares `cycle.excel_match_iterative` an implementation target with named blockers.
2. Root/report-cell, cross-version behavior, and multi-threaded variants are explicit blockers; numeric and nonnumeric prior-state behavior is observed for declared self-cycle probes.
3. The machine-readable JSON lists concrete falsification fixtures and expected cells/chains.
4. The default non-iterative profile remains candidate rejection with no new cycle-value publication.
5. `cycle.iterative_deterministic_v0` remains separate and uses canonical order plus Jacobi semantics, while Excel-match uses observed chain-ordered sequential behavior.

Fresh-eyes result: `calc-zci1.12` has a falsifiable profile specification sufficient to route implementation beads. The final Excel-match claim remains in-progress until implementation and blocker disposition.

## 9. Three-Axis Status

- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - TraceCalc implementation of `cycle.excel_match_iterative`;
  - TreeCalc implementation of `cycle.excel_match_iterative`;
  - conformance fixtures from falsification probes;
  - blocker disposition before any final broad Excel-match claim.
