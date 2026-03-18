# CORE_ENGINE_TEST_VALIDATOR_AND_RUNNER_CONTRACT.md

## 1. Purpose and Status
This document defines the first validator and scenario-runner contract for the self-contained OxCalc `TraceCalc` harness.

It exists to turn the checked-in corpus and schema into a concrete execution boundary for later fixture and host implementation.

Status:
1. supporting realization companion,
2. realized in a first executable slice,
3. intended to unblock validator, fixture-runner, and replay-pack authoring,
4. limited to the self-contained harness rather than full OxFml-integrated execution.

## 2. Role in the Spec Set
This document refines:
1. `CORE_ENGINE_TEST_HARNESS_AND_FIXTURES.md`,
2. `CORE_ENGINE_TEST_SCENARIO_SCHEMA_AND_TRACECALC.md`,
3. `CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`,
4. `CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`,
5. `CORE_ENGINE_REPLAY_APPLIANCE_ADAPTER.md`,
6. `W009`, `W010`, `W011`, `W012`, `W015`, and `W016`.

Its purpose is to lock:
1. what must be validated before a scenario is runnable,
2. what a runner must do with a scenario,
3. what artifacts a run must emit,
4. what failure classes must be distinguishable.

The first realized runner is expected to host the `TraceCalc Reference Machine` defined in `CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`.

## 3. Corpus Consumption Boundary
The first validator and runner should consume:
1. `docs/test-corpus/core-engine/tracecalc/MANIFEST.json`,
2. one or more scenario files under `docs/test-corpus/core-engine/tracecalc/hand-auditable/`.

The validator is responsible for structural and schema admissibility.
The runner is responsible for deterministic scenario execution and artifact production.

## 4. Validator Contract
The validator should reject a scenario before execution if any required contract is violated.

### 4.1 Validation Inputs
The validator consumes:
1. the manifest,
2. one scenario document,
3. the active schema version expected by the runner.

### 4.2 Validation Outputs
The validator should emit one of:
1. `valid`, with normalized scenario metadata,
2. `invalid`, with one or more typed validation failures.

### 4.3 Required Validation Checks
The first validator must check at least:
1. required top-level fields are present,
2. `calc_space` is `TraceCalc`,
3. all referenced node identifiers exist,
4. all referenced view identifiers are well-formed within the scenario,
5. all step kinds are recognized,
6. `publish_candidate` references a prior emitted candidate result,
7. expected sections are present even when empty,
8. manifest scenario identity matches the file contents,
9. JSON is well-formed and schema version is supported.

### 4.4 Typed Validation Failure Kinds
The first validator should distinguish at least:
1. `json_parse_failure`,
2. `unsupported_schema_version`,
3. `missing_required_field`,
4. `unknown_step_kind`,
5. `unknown_node_reference`,
6. `unknown_candidate_reference`,
7. `manifest_mismatch`,
8. `invalid_expected_shape`.

### 4.5 Normalization Rules
The validator may normalize only where doing so preserves authored meaning.

Allowed normalization includes:
1. stable scenario metadata projection,
2. deterministic expansion of omitted empty collections where the schema permits them,
3. canonical step indexing for later runner diagnostics.

The validator must not rewrite authored semantics.

## 5. Runner Contract
The runner consumes validated scenarios and produces deterministic execution artifacts.
The semantic core inside that runner boundary is expected to be the `TraceCalc Reference Machine`.

### 5.1 Runner Inputs
The runner consumes:
1. normalized scenario input,
2. the self-contained harness implementation,
3. deterministic run settings,
4. optional scenario filters from the manifest.

### 5.2 Runner Outputs
The runner should emit:
1. run summary,
2. per-scenario result,
3. trace artifact,
4. counter snapshot,
5. assertion result set,
6. optional replay-pack fragment later when W009 binds it.

### 5.3 Scenario Result States
The first runner should distinguish at least:
1. `passed`,
2. `failed_assertion`,
3. `invalid_scenario`,
4. `execution_error`,
5. `unsupported_feature`.

### 5.4 Determinism Rules
The runner must:
1. process steps in declared order,
2. assign deterministic run and event identifiers,
3. preserve candidate-result versus publication separation in trace output,
4. record reject-is-no-publish outcomes explicitly,
5. preserve pinned-view observations separately from current published view.

## 6. Runner Lifecycle
The first runner lifecycle should have explicit phases.

### 6.1 Phase R1: Load and Validate
1. load manifest and selected scenarios,
2. validate each scenario,
3. stop invalid scenarios before execution.

### 6.2 Phase R2: Materialize Structural Fixture
1. build the immutable structural snapshot,
2. load initial runtime state,
3. pin any declared initial views.

### 6.3 Phase R3: Execute Steps
1. apply each host step in order,
2. record step-local trace labels,
3. capture candidate, publication, reject, and reader events.

### 6.4 Phase R4: Evaluate Expectations
1. compare expected published view,
2. compare expected pinned views,
3. compare expected trace-label counts,
4. compare expected counters,
5. compare expected rejects.

### 6.5 Phase R5: Emit Run Artifacts
1. emit per-scenario result status,
2. emit normalized trace artifact,
3. emit counter snapshot,
4. emit assertion details.

## 7. Artifact Root and Naming Policy
The first canonical artifact root for self-contained reference-machine and runner outputs is:
1. `docs/test-runs/core-engine/tracecalc-reference-machine/`

Each run should emit into:
1. `docs/test-runs/core-engine/tracecalc-reference-machine/<run_id>/`

The first canonical file layout under `<run_id>` is:
1. `run_summary.json`
2. `manifest_selection.json`
3. `scenarios/<scenario_id>/result.json`
4. `scenarios/<scenario_id>/trace.json`
5. `scenarios/<scenario_id>/counters.json`
6. `scenarios/<scenario_id>/published_view.json`
7. `scenarios/<scenario_id>/pinned_views.json`
8. `scenarios/<scenario_id>/rejects.json`
9. `conformance/oracle_baseline.json` when the run is an oracle run
10. `conformance/engine_diff.json` when the run is a comparison run

This layout is canonical for the first implementation slice.
Alternative transient local output locations may exist during development, but any checked-in or pack-bound artifact should normalize to this layout.

## 7.1 Normalized Replay Bundle Emission Path
The current runner artifact root remains canonical for OxCalc-local execution.

The current Rust runner now emits the normalized additive bundle projection for an ordinary run under:
1. `docs/test-runs/core-engine/tracecalc-reference-machine/<run_id>/replay-appliance/`

The retained-failure runner now emits the matching additive bundle projection under:
1. `docs/test-runs/core-engine/tracecalc-retained-failures/<run_id>/replay-appliance/`

The first expected normalized bundle layout under that root should follow the Foundation bundle direction and include at least:
1. `bundle_manifest.json`
2. `validation/bundle_validation.json`
3. `runs/<run_id>/run_manifest.json`
4. `runs/<run_id>/scenarios/<scenario_id>/events.jsonl`
5. `runs/<run_id>/scenarios/<scenario_id>/counters.json`
6. `runs/<run_id>/scenarios/<scenario_id>/views/*.json`
7. `runs/<run_id>/diff/*.json`
8. `adapter_capabilities/oxcalc.json`

This projection root is additive.
It does not replace the current OxCalc-native artifact layout.

The current realized Rust runner floor now also emits:
1. `replay-appliance/validation/bundle_validation.json` for ordinary and retained-failure runs,
2. `replay-appliance/runs/<run_id>/diff/explain_records.json` for ordinary runs,
3. `replay-appliance/runs/<run_id>/cases/<case_id>/explain.json` for retained-failure runs,
4. `replay-appliance/validation/distill_validation.json` for retained-failure distillation runs,
5. `replay-appliance/validation/pack_candidate_validation.json` for rehearsal-only retained-failure pack-candidate separation,
6. `replay-appliance/runs/<run_id>/cases/<case_id>/distill_validation.json` and `pack_candidate_assessment.json` for retained-failure case projections.

## 8. Artifact Shapes
The first realized runner should emit data-first artifacts rather than ad hoc console text only.

### 8.1 Run Summary
The run summary should contain:
1. `run_id`,
2. `schema_version`,
3. `scenario_count`,
4. `result_counts`,
5. `artifact_root`.

### 8.2 Per-Scenario Result
Each per-scenario result should contain:
1. `scenario_id`,
2. `result_state`,
3. `validation_failures`,
4. `assertion_failures`,
5. `artifact_paths`.

### 8.3 Trace Artifact
The first trace artifact should contain:
1. `scenario_id`,
2. `run_id`,
3. `events`.

Each event should contain at least:
1. `event_id`,
2. `step_id`,
3. `label`,
4. `payload`.

### 8.4 Counter Snapshot
The first counter artifact should contain:
1. `scenario_id`,
2. `counters`.

Each counter entry should contain:
1. `counter`,
2. `value`.

## 8.5 Capability Manifest Output Expectation
Replay-appliance-aware runner flows should either:
1. emit a run-local adapter capability snapshot at `replay-appliance/adapter_capabilities/oxcalc.json`, or
2. emit a stable reference to the canonical manifest `docs/spec/core-engine/CORE_ENGINE_REPLAY_ADAPTER_CAPABILITY_MANIFEST_V1.json`.

The canonical manifest remains the authority for claimed capability levels.
Run-local snapshots may only narrow or annotate that claim, never silently widen it.

## 8.6 Registry-Id Usage Expectation
Replay-appliance-aware emissions should use Foundation registry ids where a registry family exists, including:
1. `mismatch_kind`
2. `severity_class`
3. `predicate_kind`
4. `reduction_status`
5. `witness_lifecycle_state`
6. `capability_level`

If OxCalc needs a temporary id family before Foundation publishes a machine-readable registry artifact, it must use the `oxcalc.local.*` prefix and mark the field as local-only.

## 9. Failure Semantics
The runner must distinguish scenario invalidity from execution failure.

### 9.1 Invalid Scenario
This means validation failed before execution.
No scenario-execution trace should be emitted.

### 9.2 Failed Assertion
This means execution succeeded, but expected state did not match observed state.
Trace and counter artifacts must still be emitted.

### 9.3 Execution Error
This means the harness or runner could not complete deterministic execution.
Partial artifacts may be emitted, but the error must be typed.

### 9.4 Unsupported Feature
This means the scenario uses a feature that the current runner intentionally does not realize yet.
This should be explicit, not silently downgraded.

## 10. Manifest and Selection Contract
The first runner should support:
1. run all scenarios in the manifest,
2. run one named scenario,
3. run by tag subset.

Selection must not change scenario semantics.
It only changes which validated scenarios are executed.

## 11. Relationship To Replay Packs
The validator and runner contract should be replay-pack aware, even before replay-pack artifacts are finalized.

That means the runner should preserve:
1. stable scenario identity,
2. stable run identity,
3. deterministic event ordering,
4. candidate-versus-publication boundaries,
5. typed reject outcomes.

W009 remains the place where those artifacts are promoted into pack obligations.

## 11.1 Witness Lifecycle and Quarantine
Replay-aware runner flows must preserve the distinction between:
1. replay-valid artifacts,
2. explanatory-only reduced witnesses,
3. quarantined witnesses,
4. pack-eligible retained witnesses.

If a replay or distill flow cannot satisfy the required capture surface, it must not silently produce pack-eligible output.
It must instead surface:
1. an explicit unsupported or degraded-capture outcome,
2. the relevant known limit or projection gap,
3. where applicable, a witness lifecycle state such as `wit.explanatory_only` or `wit.quarantined`.

## 11.2 Unsupported or Degraded Capture
Unsupported or degraded capture must be explicit.

Allowed behavior:
1. emit `unsupported_feature` when the runner does not support a requested feature,
2. emit `execution_error` when deterministic execution fails,
3. emit explicit projection-gap or known-limit metadata when normalized bundle projection is incomplete,
4. quarantine reduced witnesses when required replay-valid capture is missing.

Not allowed:
1. silently omitting required candidate/publication boundaries,
2. silently dropping reject sets or pinned-view surfaces,
3. treating degraded capture as replay-valid pack evidence.

## 11.3 First Retained-Witness Seed Slice
The first realized W016 slice may emit deterministic witness-seed artifacts before full reduced-witness replay validity exists.

That first slice should:
1. emit `reduction_manifest.json` and `lifecycle.json` under the declared replay-appliance root,
2. use `wit.generated_local` for lifecycle state,
3. keep `pack_eligible` false,
4. mark any pre-registry reduction-status id as explicit OxCalc-local status rather than silently treating it as canonical Foundation registry state.

The next realized retained-witness slice may additionally:
1. transition lifecycle state to `wit.explanatory_only` when mismatch or assertion preservation remains informative but replay-valid retained-witness status is not yet justified,
2. transition lifecycle state to `wit.quarantined` with explicit quarantine reason when validation or capture is insufficient,
3. keep those outcomes non-pack-eligible.

## 12. Realization Direction
The first realized validator and runner slice should likely be:
1. manifest loader,
2. scenario validator,
3. deterministic single-scenario runner,
4. JSON artifact emitter,
5. summary reporter.

It does not need yet:
1. parallel execution,
2. OxFml-integrated evaluator execution,
3. full replay-pack export,
4. generated-corpus expansion.

## 13. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - the first validator, runner, and emitted artifact path now exist and are exercised by unit tests plus the checked-in `w013-sequence-a-baseline` and `w014-stage1-widening-baseline` runs
  - emitted runtime counter snapshots now exist per scenario in the checked-in widened baseline run
  - replay-pack fragment shape remains a W009 follow-on lane
  - generated-corpus execution policy remains a later lane
- claim_confidence: provisional
- reviewed_inbound_observations: OxFml's current downstream note is reviewed; this validator and runner contract remains OxCalc-owned and self-contained, but later OxFml-integrated execution lanes must preserve OxFml-owned candidate, reject, fence, and capability meanings
