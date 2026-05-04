# W033 TraceCalc Refinement Packet

Status: `calc-uri.7_entry_packet`
Workset: `W033`
Parent epic: `calc-uri`
Bead: `calc-uri.7`
Created: 2026-05-04

## 1. Purpose

This packet defines the first W033 observable surface and refinement relation between TraceCalc oracle behavior and production/core-engine behavior.

TraceCalc is the executable correctness oracle only for covered behavior. Production/core-engine implementations refine TraceCalc when they match the declared observable surface for a covered scenario, even if they differ internally in scheduling, storage, batching, caching, or non-semantic artifact ordering.

## 2. Coverage Rule

The refinement relation applies only when all of these hold:

1. The scenario is in the declared TraceCalc coverage set.
2. TraceCalc has a passing oracle self-check for that scenario or behavior family.
3. Production/core-engine emits a comparable observation packet for the same scenario, structural input, and declared capability mode.
4. Any OxFml-owned facts used by the scenario are either fixed fixture inputs or cited upstream artifacts from the W033 source freeze.
5. The comparison tool or packet states which surfaces are exact equality, normalized equality, compatibility, or excluded internal detail.

Uncovered behavior is not automatically wrong. It is an oracle gap, a conformance gap, a spec gap, or a deferred lane until classified.

## 3. Observable Surface

| Surface | TraceCalc artifact source | Production/core-engine counterpart | Relation | Notes |
|---|---|---|---|---|
| `published_view` | `published_view.json` | production published values/snapshot projection | normalized equality | Node identity, value payload, and visible snapshot/epoch must match after declared normalization. |
| `reject_set` | `rejects.json` | production reject projection | normalized equality for reject class and required detail | Reject ids may be normalized; reject kind and semantic detail must match where required by the scenario. |
| `candidate_outcome` | `trace.json`, `counters.json`, result metadata | production candidate/commit/reject events | compatibility | Candidate admission/rejection must preserve candidate-is-not-publication and reject-is-no-publish. |
| `publication_epoch` | published view snapshot id and trace compatibility basis | production epoch/snapshot id | compatibility | Exact id strings may differ; monotonicity and compatibility basis must be preserved. |
| `pinned_views` | `pinned_views.json` where present | production pinned-reader view projection | normalized equality or declared compatibility | Pinned view must remain compatible across publication and overlay lifecycle events. |
| `trace_labels` | `trace.json` label and normalized event family | production trace event labels/families | normalized compatibility | Event ids and non-semantic ordering may differ; required transition families must be present. |
| `replay_identity` | `result.json`, replay appliance paths, witness/reduction lifecycle | production run/replay metadata | compatibility | Scenario id, run id, witness id, and reduction identity must remain traceable. |
| `dependency_facts` | future TraceCalc/TreeCalc projection | production dependency projection | conservative compatibility | Over-invalidation may be admissible; missing required dependency is a mismatch. |
| `runtime_effect_facts` | future TraceCalc/OxFml projection | production runtime-effect projection | normalized compatibility | Effect facts needed for replay, invalidation, reject, trace, or scheduling must not silently disappear. |
| `semantic_counters` | `counters.json` | production counter projection | exact or bounded equality by counter class | Semantic counters must match; timing/scale counters are measurement-only unless tied to a claim. |
| `assertion_result_set` | `result.json` assertion/validation/conformance fields | production comparison result | exact equality for pass/fail meaning | Failures must be classified, not ignored. |

## 4. Refinement Relation

For a covered scenario `s`, production observation `P` refines TraceCalc observation `T` when:

1. `T.result_state = passed`.
2. `P` is produced from the same scenario intent, input structure, and declared capability mode.
3. `normalize_published(P) = normalize_published(T)`.
4. `normalize_rejects(P) = normalize_rejects(T)` for required reject classes and detail fields.
5. If `T` rejects a candidate, `P` does not publish the rejected candidate's target updates.
6. If `T` publishes, `P` publishes atomically over the declared target set.
7. `P` preserves every required trace family in `T`; additional trace events are allowed only when they are classified as internal detail or conservative instrumentation.
8. `P` preserves replay identity enough to map scenario, run, witness, reduction, and compared surfaces.
9. `P` has dependency and runtime-effect facts that are at least conservative relative to `T` and the declared upstream OxFml facts.
10. `P` matches semantic counters or satisfies the declared bound for counters that are strategy-dependent.

The relation is intentionally observational. It does not require identical algorithms, storage layouts, allocation behavior, work queues, topo tie-breakers, cache state, or timing.

## 5. Equality And Compatibility Rules

| Rule ID | Surface | Rule |
|---|---|---|
| `REF-EQ-001` | values | Value equality is exact for scalar string/number/error payloads unless the scenario declares a typed normalization. |
| `REF-EQ-002` | node ids | Node identity is exact for TraceCalc corpus ids; production aliases must normalize to the same ids. |
| `REF-EQ-003` | reject kinds | Reject kind equality is exact. |
| `REF-EQ-004` | reject detail | Reject detail must match required semantic fields after JSON canonicalization; extra diagnostic fields are allowed if non-semantic. |
| `REF-EQ-005` | trace event ids | Event ids are not semantic unless a scenario declares them as correlation keys. |
| `REF-EQ-006` | trace families | Required normalized event families must be present; independent-event ordering may differ. |
| `REF-EQ-007` | counters | Counters classified as semantic are exact; counters classified as work-volume/timing are compared only when a scenario declares a bound or pack obligation. |
| `REF-EQ-008` | publication epoch | Epoch names may differ; publication order and compatibility basis must match. |
| `REF-EQ-009` | dependency closure | Extra affected nodes may be allowed; missing required affected nodes are not allowed. |
| `REF-EQ-010` | runtime effects | Runtime-effect facts required for invalidation, replay, reject, or trace comparison must be preserved. |

## 6. Allowed Internal Differences

Production/core-engine behavior may differ from TraceCalc in:

1. node evaluation scheduling among independent nodes,
2. batching of candidate admission or publication work,
3. storage layout and cache implementation,
4. internal event ids and diagnostic event ordering,
5. non-semantic counters and timings,
6. artifact file order where JSON arrays are declared unordered,
7. over-invalidation when it does not change the stabilized observable result and is declared as conservative,
8. richer diagnostics that do not reinterpret OxFml-owned reject, fence, or trace meaning.

Any strategy change that can affect observable results, rejects, dependency/runtime-effect visibility, or publication epochs needs a semantic-equivalence statement before promotion.

## 7. Mismatch Classification

| Class | Meaning | Required next action |
|---|---|---|
| `implementation_fault` | Production violates a covered TraceCalc observable surface and TraceCalc coverage is valid. | File implementation/successor bead with repro artifact. |
| `tracecalc_oracle_gap` | TraceCalc cannot express or self-check the behavior needed for comparison. | Expand oracle or scenario corpus before using it as authority. |
| `spec_gap` | Neither TraceCalc nor production clearly contradicts current authority because the spec is vague, stale, or missing vocabulary. | Record in spec-evolution ledger; patch/defer/handoff. |
| `oxfml_handoff_gap` | Upstream OxFml facts are missing or ambiguous for a shared-seam claim. | Add handoff/watch row; no direct OxFml edit. |
| `intentional_strategy_difference` | Internal production behavior differs but should preserve the observable surface. | Provide semantic-equivalence statement and comparison evidence. |
| `fixture_or_adapter_defect` | Input fixture, projection, adapter, or comparator is wrong. | Correct artifact/tooling before semantic classification. |
| `deferred_uncovered_behavior` | Behavior is valid but outside first-pass W033 evidence. | Closure audit records successor packet or deferral rationale. |

## 8. First Existing TraceCalc Surface Read

The existing `tc_artifact_token_reject_001` run family already provides a minimal example of the observable surface:

| Artifact | Observed surface |
|---|---|
| `result.json` | pass/fail state, assertion/conformance mismatch lists, replay projection, artifact paths |
| `published_view.json` | published node values and snapshot id |
| `rejects.json` | reject kind and reject detail |
| `trace.json` | normalized event family sequence for scheduling, dirty/needed marking, evaluation start, candidate admission, and reject issue |
| `counters.json` | semantic counters for rejected candidate, abandoned candidate, fallback reason, dirty/needed markings |

This example is not broad enough to prove W033 conformance. It is sufficient to define the first comparison vocabulary used by `calc-uri.8` and `calc-uri.9`.

## 9. Downstream Obligations

1. `calc-uri.8` must self-check the TraceCalc oracle for at least the first covered slice before the slice is used as conformance authority.
2. `calc-uri.9` must define or add the first production/core-engine comparison against this observable surface.
3. `calc-uri.10` must define metamorphic/differential transformations against this surface.
4. `calc-uri.11` and `calc-uri.12` may encode selected parts of the relation as Lean/TLA obligations.
5. `calc-uri.13` must map replay/witness identity across OxCalc and OxFml fixtures.
6. `calc-uri.14` must keep pack/capability claims at or below actual evidence.

## 10. Status

- execution_state: `tracecalc_refinement_packet_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - TraceCalc oracle self-check first slice has not yet been authored
  - production/core-engine conformance first slice has not yet been authored
  - dependency/runtime-effect surfaces are defined here but not yet exercised by W033 evidence
  - no Lean, TLA+, replay bridge, pack, or handoff artifact is emitted by this packet
