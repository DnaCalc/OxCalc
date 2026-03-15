# CORE_ENGINE_TEST_HARNESS_AND_FIXTURES.md

## 1. Purpose and Status
This document defines the intended self-contained test harness for the OxCalc core engine.

It exists so OxCalc can test coordinator, publication, invalidation, overlay, replay, and scale behavior without depending on:
1. full Excel formula-language semantics,
2. full OxFml parser or binder breadth,
3. full function-library completeness.

Status:
1. supporting realization companion,
2. spec_drafted rather than realized,
3. aligned to the TreeCalc-first Stage 1 target,
4. intended to guide fixture, replay, and stress-test authoring.

## 2. Role in the Spec Set
This document is not a replacement for the canonical architecture or seam specs.
It translates them into a testable harness shape.

It binds together:
1. `CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`,
2. `CORE_ENGINE_OXFML_SEAM.md`,
3. `CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`,
4. `CORE_ENGINE_REALIZATION_ROADMAP.md`,
5. `CORE_ENGINE_TEST_SCENARIO_SCHEMA_AND_TRACECALC.md`,
6. `W003`, `W004`, `W009`, `W010`, and `W011`.

## 3. Harness Goals
The self-contained harness should let OxCalc test:
1. accepted-candidate versus committed-publication separation,
2. reject-is-no-publish behavior,
3. pinned-view stability while later work occurs,
4. invalidation and overlay behavior under deterministic scenarios,
5. replay and trace capture obligations,
6. scalable synthetic workloads whose semantics remain traceable by hand.

The harness should not require a production evaluator implementation to exercise those properties.

## 4. Harness Layers
The intended harness has four layers.

### 4.1 Structural Fixture Layer
This layer builds immutable structural snapshots from compact scripted declarations.

It is responsible for:
1. stable synthetic node identifiers,
2. declared dependency shape,
3. optional region, profile, and capability annotations,
4. creation of snapshot roots suitable for pinning and publication tests.

### 4.2 Evaluator Test-Double Layer
This layer stands in for the OxFml-facing evaluator side.

It is responsible for:
1. opening candidate work against a declared compatibility basis,
2. emitting accepted candidate results,
3. emitting typed rejects,
4. surfacing runtime-derived effects that OxCalc coordinates on,
5. preserving deterministic identifiers for trace and replay capture.

### 4.3 Coordinator Host Layer
This layer drives the actual OxCalc coordinator and runtime state under test.

It is responsible for:
1. candidate admission,
2. publication or reject handling,
3. reader pin and unpin operations,
4. trace capture,
5. counter capture,
6. deterministic reset between scenarios.

### 4.4 Scenario Script Layer
This layer provides compact, replay-friendly scenario declarations.

It is responsible for:
1. graph declarations,
2. ordered actions,
3. expected outcomes,
4. expected trace classes and counter deltas.

## 5. Minimal Evaluator Test-Double Contract
The minimal contract must be small enough to keep engine tests self-contained, while still respecting the accepted OxFml seam direction.

### 5.1 Open Candidate Work
The host must be able to ask the test double to open evaluator work against:
1. a structural snapshot identity,
2. a compatibility token or basis,
3. a target node set or region set,
4. a deterministic scenario-run identifier.

### 5.2 Accepted Candidate Result
The test double must be able to emit an accepted candidate result that is not yet published.

The intended minimum payload is:
1. `candidate_result_id`
2. `scenario_run_id`
3. `struct_snapshot_id`
4. `compatibility_basis`
5. `value_updates`
6. `dependency_shape_updates`
7. `runtime_effects`
8. `diagnostic_events`

`value_updates` should be sufficient for core-engine tests to model per-node or per-region changed outputs without requiring Excel-compatible values.

`dependency_shape_updates` should be sufficient to represent:
1. no shape change,
2. dynamic-dependency activation or deactivation,
3. region membership or synthetic spill-shape change where a scenario intentionally models it.

`runtime_effects` should be a structured list rather than an untyped blob.

### 5.3 Typed Reject Result
The test double must be able to emit a typed reject result with machine-readable detail.

The intended minimum payload is:
1. `reject_id`
2. `scenario_run_id`
3. `struct_snapshot_id`
4. `reject_kind`
5. `reject_detail`
6. `diagnostic_events`

The initial reject kinds should include at least:
1. `snapshot_mismatch`
2. `capability_mismatch`
3. `dynamic_dependency_failure`
4. `synthetic_cycle_reject`
5. `host_injected_failure`

### 5.4 Runtime-Derived Effects
The harness should model only the effect families that matter to coordinator correctness in Stage 1.

The initial effect families are:
1. dynamic-reference activation
2. dynamic-reference release
3. synthetic spill or region-shape activation
4. format or visibility-sensitive observation when a scenario intentionally depends on it
5. capability-sensitive observation

This list is intentionally not treated as closed doctrine.

## 6. Fixture Lifecycle
The fixture lifecycle should be explicit and deterministic.

### 6.1 Phase F1: Build Structural Snapshot
1. declare nodes in the alternate calculation space,
2. declare dependency edges or conditional dependency rules,
3. assign stable synthetic identifiers,
4. produce the initial immutable structural snapshot.

### 6.2 Phase F2: Initialize Runtime and Readers
1. initialize coordinator state,
2. initialize runtime-derived state,
3. optionally seed retained overlays,
4. optionally pin one or more reader views.

### 6.3 Phase F3: Admit Work
1. select target nodes or affected regions,
2. open evaluator work against the current compatibility basis,
3. record candidate admission in the trace stream.

### 6.4 Phase F4: Inject Evaluator Outcome
1. inject an accepted candidate result, or
2. inject a typed reject.

This phase exists explicitly so tests can distinguish candidate creation from publication.

### 6.5 Phase F5: Apply Coordinator Consequence
1. accept and publish the candidate result atomically, or
2. reject and preserve prior published state,
3. record publication or no-publish consequences,
4. update counters and retention state.

### 6.6 Phase F6: Capture and Assert
1. capture trace events,
2. capture replay fragments,
3. capture counter snapshots,
4. assert published-view state,
5. assert pinned-view stability,
6. assert expected reject detail where relevant.

### 6.7 Phase F7: Reset
1. unpin readers,
2. release retained state according to the scenario,
3. reset deterministic host state,
4. preserve artifacts that belong in replay packs.

## 7. Scriptable Host Contract
The harness should expose a small scriptable host rather than force each test to encode coordinator transitions imperatively.

### 7.1 Required Host Operations
The first host contract should support operations equivalent to:
1. `declare_node`
2. `declare_edge`
3. `set_node_definition`
4. `seed_overlay`
5. `pin_view`
6. `admit_work`
7. `emit_candidate_result`
8. `emit_reject`
9. `publish_candidate`
10. `assert_view`
11. `assert_trace`
12. `assert_counters`
13. `unpin_view`
14. `reset_fixture`

### 7.2 Determinism Rules
The host must:
1. assign deterministic event identifiers,
2. preserve total order for scenario actions,
3. make publication versus rejection visible in the trace,
4. make pinned-reader observations queryable by scenario assertions.

### 7.3 Scenario Schema Direction
The first scenario schema should be data-first rather than code-first.

The intended top-level fields are:
1. `scenario_id`
2. `description`
3. `initial_graph`
4. `initial_runtime`
5. `steps`
6. `expected_traces`
7. `expected_counters`
8. `expected_views`
9. `pack_tags`

The concrete first schema and canonical JSON serialization are now defined in `CORE_ENGINE_TEST_SCENARIO_SCHEMA_AND_TRACECALC.md`.

## 8. Alternate Calculation Space
OxCalc should define a small alternate calculation space for engine-only tests.

This space is recommended, not optional.
Without it, core-engine testing remains too tightly coupled to Excel-language and evaluator completeness.

### 8.1 Name and Role
The alternate space should be treated as a synthetic test language.

The first concrete schema for `TraceCalc` scenarios is defined in `CORE_ENGINE_TEST_SCENARIO_SCHEMA_AND_TRACECALC.md`.

Working name:
1. `TraceCalc`

`TraceCalc` is not a user-facing language.
It is a deterministic semantic substrate for engine testing.

### 8.2 Design Requirements
`TraceCalc` should be:
1. deterministic,
2. compact to serialize,
3. easy to inspect in traces,
4. scalable to large synthetic graphs,
5. expressive enough for dynamic dependency and reject scenarios.

### 8.3 Initial Node Kinds
The first node kinds should include at least:
1. `const(value)`
2. `sum(dep_ids)`
3. `concat(dep_ids)`
4. `choose(control_dep, true_dep, false_dep)`
5. `dyn_select(selector_dep, candidate_dep_set)`
6. `cap_gate(required_capability, dep)`
7. `delay(dep, synthetic_stage)`
8. `cycle_member(region_id, seed_rule, step_rule)`

### 8.4 Semantics Guidance
The semantics should prioritize engine testability over richness.

That means:
1. values should be small and easy to diff,
2. dynamic dependency choices should be explicit in runtime effects,
3. reject conditions should be injectable and traceable,
4. cycle-region behavior should be synthetic and bounded rather than trying to mimic Excel iteration fully.

### 8.5 Scale Guidance
`TraceCalc` should support:
1. small hand-auditable graphs,
2. medium graphs for replay packs,
3. larger generated graphs for economics and retention measurements.

The same semantic core should serve all three sizes so scale tests remain traceable back to the small cases.

## 9. Test Families
The harness should support at least four test families.

### 9.1 Unit Fixture Tests
These test:
1. snapshot construction,
2. candidate-result payload handling,
3. reject typing,
4. host determinism,
5. pin and unpin behavior.

### 9.2 Coordinator Scenario Tests
These test:
1. accept-and-publish success,
2. reject-is-no-publish,
3. candidate-result versus publication separation,
4. pinned-view stability under later work,
5. overlay retention and release.

### 9.3 Replay and Pack Tests
These test:
1. trace completeness,
2. replay determinism,
3. candidate versus publication boundary capture,
4. typed fence and capability reject capture.

### 9.4 Scale and Experiment Tests
These test:
1. synthetic graph growth,
2. fallback rates,
3. overlay reuse and retention,
4. work-volume signatures,
5. later concurrency replay pressure once Stage 2 begins.

## 10. Relationship To OxFml-Integrated Tests
This harness does not replace tests that exercise the real OxFml seam.

Instead:
1. self-contained harness tests prove core-engine behavior in isolation,
2. OxFml-integrated tests prove seam compatibility and evaluator interaction,
3. replay packs should eventually include both classes.

The self-contained harness should be the first testing surface, because it makes engine regressions diagnosable before full evaluator breadth is in play.

## 11. Realization Direction
The first realized harness slice should likely be:
1. a unit-test fixture library,
2. a small scenario runner,
3. a `TraceCalc` graph builder,
4. deterministic trace capture,
5. golden replay-pack fragments for a small initial corpus.

The first realized harness does not need:
1. a production-grade DSL parser,
2. concurrency stress from day one,
3. full OxFml-integrated execution.

## 12. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - the initial hand-auditable corpus now exists, but no validator, fixture library, or host implementation exists yet
  - `TraceCalc` scenario schema and initial corpus are now defined, but no runner consumes them yet
  - replay-pack ownership and exact artifact locations still need binding through W009
  - OxFml-integrated harness coverage still remains a later companion lane
- claim_confidence: provisional
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
