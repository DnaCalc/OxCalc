# CORE_ENGINE_TEST_SCENARIO_SCHEMA_AND_TRACECALC.md

## 1. Purpose and Status
This document defines the first concrete scenario schema for the OxCalc self-contained core-engine harness.

It turns the `TraceCalc` direction from `CORE_ENGINE_TEST_HARNESS_AND_FIXTURES.md` into an authorable scenario format.

Status:
1. supporting realization companion,
2. spec_drafted rather than realized,
3. intended to unblock fixture, corpus, and replay-pack authoring,
4. limited to the self-contained engine-test harness rather than full OxFml-integrated scenarios.

## 2. Role in the Spec Set
This document refines:
1. `CORE_ENGINE_TEST_HARNESS_AND_FIXTURES.md`,
2. `CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`,
3. `CORE_ENGINE_OXFML_SEAM.md`,
4. `CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`,
5. `W009`, `W010`, and `W011`.

Its purpose is to make the first fixture and host implementation authorable without reopening basic schema choices.

## 3. Canonical Scenario Serialization
The canonical scenario serialization for the self-contained harness should be:
1. one JSON document per scenario,
2. UTF-8 text,
3. stable field names,
4. deterministic ordering in emitted golden artifacts where practical.

JSON is chosen because it is:
1. deterministic enough for replay-pack artifacts,
2. straightforward to diff and validate,
3. easy to generate for larger synthetic graphs,
4. neutral with respect to future host-language choice.

Authoring adapters may later support YAML or code-first builders, but the canonical persisted artifact should be JSON.

## 4. Top-Level Scenario Shape
Each scenario document should have the following top-level fields.

### 4.1 Required Fields
1. `schema_version`
2. `scenario_id`
3. `description`
4. `calc_space`
5. `initial_graph`
6. `initial_runtime`
7. `steps`
8. `expected`

### 4.2 Optional Fields
1. `tags`
2. `pack_tags`
3. `generator`
4. `notes`

### 4.3 Field Meanings
- `schema_version`: schema identifier for host compatibility.
- `scenario_id`: stable scenario identity used in traces, packs, and reports.
- `description`: short human-readable description.
- `calc_space`: should be `TraceCalc` for this document's scenarios.
- `initial_graph`: immutable structural declarations.
- `initial_runtime`: initial runtime, pinning, capability, and overlay seed state.
- `steps`: ordered host actions.
- `expected`: expected views, traces, counters, and reject or publish consequences.
- `tags`: authoring or classification tags.
- `pack_tags`: pack membership hints.
- `generator`: optional generator metadata for synthetic large scenarios.
- `notes`: optional explanatory prose.

## 5. Initial Graph Schema
`initial_graph` defines the immutable starting snapshot.

### 5.1 Required Fields
1. `snapshot_id`
2. `nodes`

### 5.2 Optional Fields
1. `regions`
2. `profiles`
3. `capabilities`

### 5.3 Node Shape
Each node should contain:
1. `node_id`
2. `kind`
3. `expr`

Optional node fields:
1. `profile`
2. `region_id`
3. `annotations`

### 5.4 TraceCalc Expression Shapes
The first supported `expr` variants should be object-shaped and explicit.

Examples:
1. `{ "op": "const", "value": 5 }`
2. `{ "op": "sum", "deps": ["A", "B"] }`
3. `{ "op": "concat", "deps": ["A", "B"] }`
4. `{ "op": "choose", "control": "Flag", "when_true": "A", "when_false": "B" }`
5. `{ "op": "dyn_select", "selector": "Mode", "candidates": { "left": "A", "right": "B" } }`
6. `{ "op": "cap_gate", "required_capability": "fmt", "dep": "A" }`
7. `{ "op": "delay", "dep": "A", "synthetic_stage": 1 }`
8. `{ "op": "cycle_member", "region_id": "R1", "seed_rule": "zero", "step_rule": "plus_dep" }`

### 5.5 Region Shape
Where used, each region should contain:
1. `region_id`
2. `kind`
3. `members`

The initial self-contained harness only needs regions where cycle or synthetic spill-shape scenarios depend on them.

## 6. Initial Runtime Schema
`initial_runtime` defines the non-structural starting state.

### 6.1 Optional Fields
1. `pinned_views`
2. `seed_overlays`
3. `capability_state`
4. `published_values`
5. `published_runtime_effects`

### 6.2 Pinned View Shape
Each pinned view should contain:
1. `view_id`
2. `snapshot_id`
3. `observed_nodes`

### 6.3 Seed Overlay Shape
Each seed overlay entry should contain:
1. `overlay_kind`
2. `owner_node_id`
3. `payload`

This is only for targeted retention or eviction scenarios.

## 7. Step Schema
`steps` is an ordered list of host actions.
Each step should have:
1. `step_id`
2. `kind`

Optional per-step fields:
1. `description`
2. `expect_trace_labels`
3. `expect_counter_deltas`

### 7.1 Structural and Host Steps
The first step kinds should include:
1. `pin_view`
2. `unpin_view`
3. `mark_stale`
4. `seed_overlay`
5. `reset_fixture`

### 7.2 Candidate Admission Step
`admit_work` should contain:
1. `compatibility_basis`
2. `targets`
3. `admission_id`

### 7.3 Accepted Candidate Result Step
`emit_candidate_result` should contain:
1. `candidate_result_id`
2. `compatibility_basis`
3. `value_updates`
4. `dependency_shape_updates`
5. `runtime_effects`
6. `diagnostic_events`

#### Value Update Shape
Each value update should contain:
1. `node_id`
2. `value`

Optional fields:
1. `value_state`
2. `signature`

#### Dependency Shape Update Shape
Each dependency shape update should contain:
1. `node_id`
2. `kind`

Initial kinds:
1. `none`
2. `activate_dynamic_dep`
3. `release_dynamic_dep`
4. `change_region_membership`
5. `synthetic_spill_shape`

#### Runtime Effect Shape
Each runtime effect should contain:
1. `effect_kind`
2. `owner_node_id`
3. `payload`

Initial effect kinds:
1. `dynamic_ref_activated`
2. `dynamic_ref_released`
3. `region_shape_activated`
4. `capability_observed`
5. `format_observed`

### 7.4 Reject Step
`emit_reject` should contain:
1. `reject_id`
2. `reject_kind`
3. `reject_detail`
4. `diagnostic_events`

Initial reject kinds:
1. `snapshot_mismatch`
2. `capability_mismatch`
3. `dynamic_dependency_failure`
4. `synthetic_cycle_reject`
5. `host_injected_failure`

### 7.5 Publish Step
`publish_candidate` should contain:
1. `candidate_result_id`
2. `publication_id`

This step exists so the scenario can model the candidate-result versus publication boundary explicitly.

## 8. Expected Outcome Schema
`expected` should be the assertion surface for the scenario.

### 8.1 Expected Fields
1. `published_view`
2. `pinned_views`
3. `trace_labels`
4. `counter_expectations`
5. `rejects`

All of these may be empty, but the fields should exist.

### 8.2 Published View Shape
`published_view` should contain:
1. `snapshot_id`
2. `node_values`

Each node value entry should contain:
1. `node_id`
2. `value`

### 8.3 Pinned View Expectation Shape
Each pinned view expectation should contain:
1. `view_id`
2. `snapshot_id`
3. `node_values`

### 8.4 Trace Label Shape
Each trace label expectation should contain:
1. `label`
2. `count`

Initial recommended labels include:
1. `candidate_admitted`
2. `candidate_emitted`
3. `candidate_published`
4. `candidate_rejected`
5. `reader_pinned`
6. `reader_unpinned`
7. `overlay_retained`
8. `overlay_released`

### 8.5 Counter Expectation Shape
Each counter expectation should contain:
1. `counter`
2. `comparison`
3. `value`

Examples:
1. `{ "counter": "overlay.retained", "comparison": "eq", "value": 1 }`
2. `{ "counter": "recalc.fallback", "comparison": "eq", "value": 0 }`

### 8.6 Reject Expectation Shape
Each reject expectation should contain:
1. `reject_id`
2. `reject_kind`

Optional fields:
1. `detail_contains`

## 9. Example Scenario Shapes
These examples are illustrative schema examples, not yet replay artifacts.

### 9.1 Accept and Publish
```json
{
  "schema_version": "tracecalc-s1",
  "scenario_id": "tc_accept_publish_001",
  "description": "accepted candidate result is distinct from later publication",
  "calc_space": "TraceCalc",
  "initial_graph": {
    "snapshot_id": "s0",
    "nodes": [
      { "node_id": "A", "kind": "value", "expr": { "op": "const", "value": 2 } },
      { "node_id": "B", "kind": "value", "expr": { "op": "sum", "deps": ["A"] } }
    ]
  },
  "initial_runtime": {
    "published_values": [
      { "node_id": "A", "value": 2 },
      { "node_id": "B", "value": 2 }
    ]
  },
  "steps": [
    { "step_id": "st1", "kind": "mark_stale", "targets": ["B"] },
    { "step_id": "st2", "kind": "admit_work", "admission_id": "adm1", "compatibility_basis": "s0", "targets": ["B"] },
    {
      "step_id": "st3",
      "kind": "emit_candidate_result",
      "candidate_result_id": "cand1",
      "compatibility_basis": "s0",
      "value_updates": [
        { "node_id": "B", "value": 2 }
      ],
      "dependency_shape_updates": [],
      "runtime_effects": [],
      "diagnostic_events": []
    },
    { "step_id": "st4", "kind": "publish_candidate", "candidate_result_id": "cand1", "publication_id": "pub1" }
  ],
  "expected": {
    "published_view": {
      "snapshot_id": "s0",
      "node_values": [
        { "node_id": "A", "value": 2 },
        { "node_id": "B", "value": 2 }
      ]
    },
    "pinned_views": [],
    "trace_labels": [
      { "label": "candidate_admitted", "count": 1 },
      { "label": "candidate_emitted", "count": 1 },
      { "label": "candidate_published", "count": 1 }
    ],
    "counter_expectations": [],
    "rejects": []
  }
}
```

### 9.2 Reject Is No Publish
```json
{
  "schema_version": "tracecalc-s1",
  "scenario_id": "tc_reject_no_publish_001",
  "description": "typed reject preserves prior published view",
  "calc_space": "TraceCalc",
  "initial_graph": {
    "snapshot_id": "s0",
    "nodes": [
      { "node_id": "A", "kind": "value", "expr": { "op": "const", "value": 2 } },
      { "node_id": "B", "kind": "value", "expr": { "op": "cap_gate", "required_capability": "fmt", "dep": "A" } }
    ]
  },
  "initial_runtime": {
    "published_values": [
      { "node_id": "A", "value": 2 },
      { "node_id": "B", "value": 2 }
    ]
  },
  "steps": [
    { "step_id": "st1", "kind": "admit_work", "admission_id": "adm1", "compatibility_basis": "s0", "targets": ["B"] },
    {
      "step_id": "st2",
      "kind": "emit_reject",
      "reject_id": "rej1",
      "reject_kind": "capability_mismatch",
      "reject_detail": { "required_capability": "fmt", "observed_capability": "none" },
      "diagnostic_events": []
    }
  ],
  "expected": {
    "published_view": {
      "snapshot_id": "s0",
      "node_values": [
        { "node_id": "A", "value": 2 },
        { "node_id": "B", "value": 2 }
      ]
    },
    "pinned_views": [],
    "trace_labels": [
      { "label": "candidate_admitted", "count": 1 },
      { "label": "candidate_rejected", "count": 1 }
    ],
    "counter_expectations": [],
    "rejects": [
      { "reject_id": "rej1", "reject_kind": "capability_mismatch" }
    ]
  }
}
```

## 10. Corpus Direction
The first authored self-contained corpus should include at least:
1. accept-and-publish,
2. reject-is-no-publish,
3. candidate-result versus publication separation,
4. pinned-view stability,
5. dynamic dependency activation and release,
6. overlay retention and release,
7. one synthetic scale scenario with generator metadata.

A later corpus may split into:
1. hand-auditable scenarios,
2. replay-pack scenarios,
3. generated economics scenarios.

## 11. Open Design Constraints
This schema intentionally leaves some details open for later realization work.

Open but bounded areas:
1. exact file location and naming convention for the first corpus,
2. exact validator implementation,
3. whether generated scale scenarios are checked in fully expanded or retained as generator inputs,
4. exact trace event body schema beyond the label-count assertion layer defined here.

These are no longer allowed to block fixture authoring.

## 12. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - no concrete corpus files or fixture runner exist yet
  - candidate-result and reject payloads still need final alignment with W003 and W004 realization packets
  - exact trace event body schema remains narrower than the future replay artifact schema
  - corpus-location and validator implementation details are still open
- claim_confidence: provisional
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
