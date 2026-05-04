# W033 Metamorphic And Differential Test Families

Status: `calc-uri.10_entry_packet`
Workset: `W033`
Parent epic: `calc-uri`
Bead: `calc-uri.10`
Created: 2026-05-04

## 1. Purpose

This packet defines the first W033 metamorphic and differential test families and assigns each family to TraceCalc, TreeCalc, OxFml replay, or deferred evidence lanes.

These families are leverage tools. They do not promote correctness claims by being named here. A family becomes evidence only when it has a scenario, generator, replay run, comparison artifact, or explicit deferred rationale.

## 2. Assignment Vocabulary

| Lane | Meaning |
|---|---|
| `TraceCalc` | Add or transform scenarios under the TraceCalc corpus and compare observed artifacts through the TraceCalc refinement surface. |
| `TreeCalc-local` | Use checked-in TreeCalc fixtures and local sequential runtime artifacts. |
| `TreeCalc-scale` | Use generated scale/stress profiles and phase timing artifacts; semantic claims still require conformance binding. |
| `OxFml-replay` | Use read-only OxFml fixtures/witnesses as upstream evidence or future handoff pressure. |
| `Cross-engine` | Compare two engines or surfaces, such as TraceCalc oracle vs core-engine-backed engine or TreeCalc fixture vs TraceCalc projection. |
| `Deferred` | Keep as planned leverage until required model, fixture, or ownership surfaces exist. |

## 3. First Family Matrix

| Family ID | Family | Transformation | Expected invariant | First lane | Current seed surface | Evidence status |
|---|---|---|---|---|---|---|
| `W033-META-001` | From-scratch vs incremental recalc | Recompute a scenario from initial structure and compare with post-edit incremental rerun. | Stabilized published values, typed rejects, and required runtime-effect facts match. | `TreeCalc-local`; later `TraceCalc` | TreeCalc cases tagged `structural-edit`, `recalc-only`, `dependency-chain` | `planned` |
| `W033-META-002` | Independent-node reordering | Reorder independent nodes or independent work groups while preserving dependencies. | Published values and rejects match; trace order may differ only where non-semantic. | `TraceCalc`; `TreeCalc-local` | `tc_multinode_dag_publish_001`; local relative/sibling cases | `planned` |
| `W033-META-003` | LET inlining | Replace `LET(x,v,body)` with body where `x` is safely substituted and no shadowing/capture changes meaning. | Published value, dependency/runtime-effect facts, and trace/replay identity remain compatible. | `OxFml-replay`; later `TraceCalc` | OxFml `prepared_005_let_helper_trace`; LET/LAMBDA boundary packet | `deferred` until OxCalc has LET carrier scenarios |
| `W033-META-004` | LAMBDA call refactoring | Transform inline lambda calls to helper-bound or defined-name callable forms when lexical capture is preserved. | Value, callable carrier facts, typed rejects, and replay provenance remain compatible. | `OxFml-replay`; later `TraceCalc` | OxFml `higher_order_callable_cases.json`; prepared-call lambda cases | `deferred` until OxCalc carrier bridge exists |
| `W033-META-005` | Scheduling-policy variation | Change legal scheduling/tie-break order without changing dependency order requirements. | Stabilized observable surface matches; semantic counters match or declared strategy counters are bounded. | `TraceCalc`; later `TLA` | TraceCalc DAG and cycle-region scenarios | `planned` |
| `W033-META-006` | Dynamic reference retargeting | Change selector/input that retargets a dynamic reference and then recompute. | No under-invalidation; new runtime dependency is visible; old dependency release is conservative. | `TreeCalc-local`; `TreeCalc-scale` | `tc_dynamic_dep_switch_001`; TreeCalc dynamic reject; scale `dynamic-indirect-stripes` | `planned` |
| `W033-META-007` | Conservative invalidation widening | Add extra invalidated nodes that do not affect required observable results. | Published values/rejects match; work-volume counters may increase within declared measurement class. | `TraceCalc`; `TreeCalc-local`; `TreeCalc-scale` | invalidation/scale profiles | `planned` |
| `W033-META-008` | Reject injection invariance | Replace candidate success with a typed reject at the seam boundary. | Rejected candidate does not publish; published view remains pre-reject compatible; reject class/detail is stable. | `TraceCalc`; `OxFml-replay` | reject/no-publish, artifact-token, fence-reject cases | `existing_floor` through TraceCalc run, not transformed family yet |
| `W033-META-009` | Publication fence perturbation | Perturb compatibility basis or artifact token while keeping candidate payload otherwise valid. | Stale/incompatible candidate is rejected with no publication. | `TraceCalc` | `tc_publication_fence_reject_001`; `tc_artifact_token_reject_001` | `existing_floor` through fixed cases; generator planned |
| `W033-META-010` | Pinned-reader/overlay lifecycle variation | Pin/unpin around publication and overlay retention/release points. | Pinned views stay compatible; protected overlays are retained; eligible overlays can be released after protection ends. | `TraceCalc`; `TreeCalc-local`; `TLA` | pinned view and overlay retention scenarios | `planned` |
| `W033-META-011` | Cycle-region normalization | Rename or reorder nodes within an SCC without changing the cycle relation. | Cycle-region reject/fixed-point classification is stable. | `TraceCalc`; later `TreeCalc-local` | `tc_cycle_region_reject_001` | `planned` |
| `W033-META-012` | Cross-engine oracle differential | Compare TraceCalc oracle/engine artifacts to TreeCalc local artifacts where a fixture can be projected to the same observable surface. | Surface equality or classified mismatch. | `Cross-engine` | TraceCalc W033 run; TreeCalc local manifest | `deferred` until projection mapping exists |
| `W033-META-013` | Scale signature differential | Run scale profiles with controlled parameter changes and compare phase timing/counter shapes. | Measurement signature changes are explainable; semantic promotion requires conformance binding. | `TreeCalc-scale` | scale profiles `grid-cross-sum`, `fanout-bands`, `dynamic-indirect-stripes`, `relative-rebind-churn` | `planned`; semantic claim deferred |
| `W033-META-014` | OxFml fixture replay distillation | Select OxFml candidate/commit/reject/callable fixtures and map them to OxCalc seam outcomes. | Imported facts preserve identity and no-publish/publication boundaries. | `OxFml-replay`; `Cross-engine` | OxFml FEC/replay/witness fixtures | `planned` for replay bridge |

## 4. First Concrete Test Directions

### 4.1 TraceCalc Corpus Directions

1. Add generated variants for `tc_multinode_dag_publish_001` that reorder independent nodes and assert the same `published_view`, `reject_set`, and required trace families.
2. Add fence perturbation variants where candidate value updates remain valid but the compatibility basis changes.
3. Add pinned-reader lifecycle variants that move pin/unpin boundaries across publish and overlay-retention events.
4. Add cycle-region rename/reorder variants with stable cycle-region reject semantics.

### 4.2 TreeCalc Local Directions

1. Pair every post-edit recalc-only case with a from-scratch recompute expectation over the final structure.
2. Add dependency-chain variants that over-invalidate harmless siblings and confirm published values remain stable.
3. Add dynamic-reference variants that switch selectors across two or more targets, then verify affected-set coverage.
4. Add local fixture tags for cases that can be projected into the W033 TraceCalc observable surface.

### 4.3 TreeCalc Scale Directions

1. Use `grid-cross-sum` for broad fan-in/fan-out and phase timing split.
2. Use `fanout-bands` to stress dependency propagation and dirty/needed volume.
3. Use `dynamic-indirect-stripes` to stress soft/runtime-derived dependency resolution.
4. Use `relative-rebind-churn` to stress structural edit, rebind, dependency rebuild, and recalculation separation.

Scale directions are measurement lanes unless paired with deterministic expected values and conformance checks.

### 4.4 OxFml Replay Directions

1. Use prepared-call LET/LAMBDA cases to identify carrier facts needed by OxCalc replay/conformance.
2. Use FEC commit/reject fixtures to map candidate/commit/reject facts to OxCalc publication/no-publication outcomes.
3. Use retained witness indexes to select minimal candidate, reject, and callable watch cases.
4. Avoid treating OxFml fixture truth as OxCalc-owned authority; use handoff/watch when normative upstream text is required.

## 5. Checkability Rules

Every promoted metamorphic/differential test must state:

1. source scenario or fixture,
2. transformation,
3. required observable surface,
4. allowed internal differences,
5. expected equality or compatibility relation,
6. artifact root and run id,
7. mismatch classification policy,
8. whether the result is semantic evidence or measurement-only evidence.

## 6. Status

- execution_state: `metamorphic_and_differential_family_packet_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - no new generated metamorphic scenarios are emitted by this packet
  - no TreeCalc-scale semantic promotion is made by this packet
  - LET/LAMBDA metamorphic families are deferred until the replay/witness bridge maps carrier facts
  - cross-engine TraceCalc-to-TreeCalc projection remains deferred until a shared observable projection exists
