# CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md

## 1. Purpose and Status
This document defines the `TraceCalc Reference Machine` for OxCalc.

The `TraceCalc Reference Machine` is the first intended executable conformance oracle for the core engine.
It is not the production engine.
It is the deterministic semantic reference that later implementations must match.

Status:
1. supporting realization companion,
2. realized in a first deterministic implementation slice,
3. intended to bridge spec, formalization, corpus, and implementation,
4. limited initially to the self-contained Stage 1 harness and `TraceCalc` corpus.

## 2. Why This Exists
OxCalc now has:
1. architectural semantics,
2. Lean-facing state vocabulary,
3. TLA+-facing coordinator actions,
4. replay and pack planning,
5. a self-contained `TraceCalc` corpus.

What is still missing is a single executable semantic source of truth.

The `TraceCalc Reference Machine` exists to provide that source of truth.
Its purpose is to:
1. execute the canonical `TraceCalc` scenarios,
2. produce canonical observed artifacts,
3. expose semantic mismatches before production optimization work begins,
4. provide the comparison baseline for later production-engine conformance.

## 3. Role in the Spec Set
This document refines and connects:
1. `CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`,
2. `CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`,
3. `CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`,
4. `CORE_ENGINE_TEST_HARNESS_AND_FIXTURES.md`,
5. `CORE_ENGINE_TEST_SCENARIO_SCHEMA_AND_TRACECALC.md`,
6. `CORE_ENGINE_TEST_VALIDATOR_AND_RUNNER_CONTRACT.md`,
7. `W007` through `W012`.

Its role is to define:
1. the reference state model,
2. the reference transition set,
3. the canonical observed artifact surface,
4. the conformance-comparison rules later engines must satisfy.

## 4. Scope and Non-Goals
### 4.1 In Scope
The initial `TraceCalc Reference Machine` should cover:
1. self-contained `TraceCalc` scenarios,
2. deterministic single-threaded execution,
3. candidate-result versus publication separation,
4. reject-is-no-publish behavior,
5. pinned-view stability,
6. runtime overlay and dynamic-dependency semantics at the harness level,
7. canonical trace and counter output for comparison purposes.

### 4.2 Out of Scope
The initial `TraceCalc Reference Machine` does not need:
1. production performance,
2. staged concurrency or async throughput,
3. full OxFml-integrated evaluator execution,
4. full replay-pack export,
5. grid semantics,
6. optimized storage layout.

## 5. Oracle Principle
The `TraceCalc Reference Machine` must be treated as the canonical semantic oracle for the surfaces it realizes.

That means:
1. production strategies may differ internally,
2. scheduling strategies may differ internally,
3. storage representation may differ internally,
4. observable results must still match the oracle on the covered corpus.

No strategy lane should be promoted if it changes covered observable semantics without an explicit spec change and semantic-equivalence statement.

## 6. Inputs and Execution Surface
The initial machine should consume:
1. `docs/test-corpus/core-engine/tracecalc/MANIFEST.json`,
2. one or more `TraceCalc` scenario files,
3. deterministic run settings,
4. the self-contained harness implementation surface.

The initial machine should support:
1. single-scenario execution,
2. manifest-wide execution,
3. tag-filtered execution.

## 7. Reference State Model
The reference machine should use a small explicit state model aligned to the W007 and W008 vocabulary.

### 7.1 Core State Objects
The initial reference state should contain at least:
1. `StructSnapshotRef`
2. `PublishedViewRef`
3. `PinnedViewSet`
4. `RuntimeOverlayStateRef`
5. `CandidateResultStore`
6. `RejectLogRef`
7. `TraceLogRef`
8. `CounterStateRef`
9. `RunContextRef`

### 7.2 State Intent
- `StructSnapshotRef`: immutable structural truth used for the run.
- `PublishedViewRef`: currently published values and publication-visible effects.
- `PinnedViewSet`: reader-visible pinned projections that must remain stable.
- `RuntimeOverlayStateRef`: dynamic dependency and related runtime-derived state.
- `CandidateResultStore`: accepted-but-not-yet-published candidate artifacts.
- `RejectLogRef`: typed reject outcomes and details.
- `TraceLogRef`: ordered event stream emitted by the machine.
- `CounterStateRef`: deterministic counters for overlay, fallback, and related activity.
- `RunContextRef`: run identifier, scenario identifier, and mode metadata.

### 7.3 Invariants
The initial machine should preserve at least these invariants:
1. structural truth is immutable during a scenario run,
2. publication is atomic with respect to the observed published view,
3. reject does not modify the previously published view,
4. pinned views remain stable until explicitly unpinned,
5. candidate artifacts exist independently of publication,
6. trace order is deterministic.

## 8. Reference Transition Set
The reference machine should realize an explicit transition set.

### 8.1 Run-Level Transitions
1. `RM0_LoadManifest`
2. `RM1_LoadScenario`
3. `RM2_ValidateScenario`
4. `RM3_InitializeRunState`

### 8.2 Scenario-Step Transitions
1. `RM4_PinView`
2. `RM5_UnpinView`
3. `RM6_MarkStale`
4. `RM7_AdmitWork`
5. `RM8_EmitCandidateResult`
6. `RM9_EmitReject`
7. `RM10_PublishCandidate`
8. `RM11_SeedOverlay`
9. `RM12_ResetFixture`

### 8.3 Completion Transitions
1. `RM13_EvaluateExpectations`
2. `RM14_EmitArtifacts`
3. `RM15_CloseRun`

## 9. Transition Semantics
### 9.1 `RM7_AdmitWork`
This transition records a candidate admission boundary.
It must not itself publish values.
It may allocate candidate-work identity and trace metadata.

### 9.2 `RM8_EmitCandidateResult`
This transition records an accepted candidate result in `CandidateResultStore`.
It must not change `PublishedViewRef`.
It may change candidate-local observed state and trace metadata.

### 9.3 `RM9_EmitReject`
This transition appends a typed reject to `RejectLogRef`.
It must not change `PublishedViewRef`.
It must emit deterministic reject trace metadata.

### 9.4 `RM10_PublishCandidate`
This transition consumes a previously emitted candidate result and atomically updates:
1. `PublishedViewRef`,
2. publication-visible runtime effects,
3. relevant counters,
4. publication trace events.

This is the only transition that changes the published view in the initial machine.

### 9.5 `RM4_PinView` and `RM5_UnpinView`
These transitions define the reader-stability surface.
A pinned view must preserve its visible state even after later publication occurs.

## 10. Canonical Observed Artifact Surface
The machine must emit the canonical observed artifacts later engines are compared against.

### 10.1 Required Artifact Classes
1. run summary,
2. per-scenario result,
3. canonical published-view artifact,
4. canonical pinned-view artifact where relevant,
5. canonical trace artifact,
6. canonical counter artifact,
7. canonical reject artifact where relevant,
8. conformance-ready metadata.

### 10.2 Canonical Artifact Root
The first canonical artifact root for oracle and conformance outputs is:
1. `docs/test-runs/core-engine/tracecalc-reference-machine/`

Each oracle or comparison run should emit into:
1. `docs/test-runs/core-engine/tracecalc-reference-machine/<run_id>/`

The expected normalized layout under `<run_id>` is:
1. `run_summary.json`
2. `manifest_selection.json`
3. `scenarios/<scenario_id>/result.json`
4. `scenarios/<scenario_id>/trace.json`
5. `scenarios/<scenario_id>/counters.json`
6. `scenarios/<scenario_id>/published_view.json`
7. `scenarios/<scenario_id>/pinned_views.json`
8. `scenarios/<scenario_id>/rejects.json`
9. `conformance/oracle_baseline.json` for oracle-only normalized comparison surfaces
10. `conformance/engine_diff.json` for engine-versus-oracle comparison results

### 10.3 Artifact Intent
These artifacts are not only diagnostics.
They are the observable semantic surface used for conformance comparison.

## 11. Conformance Contract
A later engine implementation should be considered semantically conformant for covered scenarios only if it matches the reference machine on the declared comparison surfaces.

### 11.1 Required Equality Surfaces
The first conformance comparison should require equality of at least:
1. final published view,
2. pinned-view observations,
3. typed reject outcomes,
4. trace label counts,
5. declared counters,
6. candidate-result versus publication boundary preservation.

### 11.2 Allowed Non-Equality Surfaces
The first conformance comparison may allow differences in:
1. internal storage shape,
2. internal scheduling path,
3. performance metrics,
4. richer trace payload fields not yet promoted to required comparison surfaces.

### 11.3 First Diff Policy
The first oracle-to-engine diff policy should compare surfaces in this order:
1. scenario presence and result state,
2. published view,
3. pinned views,
4. typed rejects,
5. trace label counts,
6. counters,
7. optional richer trace payloads only when explicitly enabled.

The first diff policy should emit typed mismatch kinds at least for:
1. `missing_scenario_result`,
2. `result_state_mismatch`,
3. `published_view_mismatch`,
4. `pinned_view_mismatch`,
5. `reject_mismatch`,
6. `trace_count_mismatch`,
7. `counter_mismatch`,
8. `unexpected_extra_artifact`.

Richer trace payload mismatches should be treated as informational unless the compared field has been promoted into the required equality surface.

### 11.4 Failure Meaning
A mismatch against the reference machine should be treated as:
1. a semantic regression,
2. an incomplete implementation,
3. or a proof that the spec surface itself needs correction.

It must not be dismissed as an implementation detail if it affects a covered comparison surface.

## 12. Relationship To Formalization
The reference machine should be the executable bridge between prose spec and formal models.

### 12.1 Lean Relationship
The machine state should align with the Lean-facing object inventory from W007.
The machine can later act as the practical witness for theorem targets such as:
1. reject-is-no-publish,
2. pinned-view stability,
3. publication atomicity,
4. candidate-result versus publication separation.

### 12.2 TLA+ Relationship
The machine transition set should align with the coordinator actions and safety obligations from W008.
The machine does not replace TLA+.
It provides an executable surface whose transitions can be mapped to the TLA+ action vocabulary.

## 13. Relationship To Validator and Runner
The first realized validator and runner should likely be implemented as the first operational shell around the reference machine.

That means:
1. the validator prepares admissible scenario input,
2. the runner hosts execution,
3. the reference machine provides the canonical semantics and observed artifacts.

The validator-runner contract remains the external execution boundary.
The reference machine is the semantic core inside that boundary.

## 14. Realization Direction
The first realized `TraceCalc Reference Machine` slice should likely include:
1. deterministic state object materialization,
2. explicit scenario-step interpreter,
3. canonical published-view projection,
4. canonical trace and counter emission,
5. canonical conformance artifact emission.

It does not need yet:
1. performance tuning,
2. multithreaded execution,
3. OxFml-integrated evaluator execution,
4. generated large-graph expansion,
5. replay-pack export beyond conformance-ready local artifacts.

## 15. Promotion Rule
No later production-engine optimization lane should be treated as semantically trustworthy for covered behaviors until it has been compared against the `TraceCalc Reference Machine` on the covered corpus.

This is the intended anti-regression and anti-premature-optimization gate.

## 16. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - the first reference-machine implementation now exists under `src/OxCalc.Core/TraceCalc/` and emits the checked-in baseline run
  - production-engine conformance workflow remains at the first diff policy and first corpus only
  - richer trace payload promotion remains a later tightening lane
  - engine-versus-oracle comparison now exists for the initial corpus, but only at the first Stage 1 surface
- claim_confidence: provisional
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
