# CORE_ENGINE_REPLAY_APPLIANCE_ADAPTER.md

## 1. Purpose and Status
This document defines the OxCalc-local adapter contract for projecting `TraceCalc`, reference-machine, runner, and engine-diff artifacts into the Foundation Replay appliance rollout.

Status:
1. active OxCalc-local replay incorporation companion,
2. authoritative for OxCalc adapter semantics and projection rules,
3. aligned to the Foundation replay handoff run from `2026-03-15`,
4. intentionally narrower than a full `DNA ReCalc` implementation.

This document does not replace:
1. `TraceCalc` scenario authoring,
2. the OxCalc coordinator or publication model,
3. the `TraceCalc Reference Machine`,
4. Stage 1 replay classes `R1..R8`.

## 2. Scope and Non-Goals

### 2.1 In Scope
1. The OxCalc adapter boundary for projecting local artifacts into Replay appliance bundles.
2. The authority split between OxCalc semantics and Foundation replay governance.
3. Event-family normalization and label-drift handling for current OxCalc replay artifacts.
4. Required preserved view, reject, assertion, and diff surfaces.
5. Capability rollout, registry pinning, witness lifecycle, and quarantine rules as they apply to OxCalc.

### 2.2 Non-Goals
1. Replacing `TraceCalc` with a new scenario DSL.
2. Replacing local OxCalc run artifacts with generic bundle-only authoring.
3. Collapsing candidate-result and publication into one success event.
4. Flattening `engine_diff` into prose-only explanations.
5. Claiming pack-grade rollout before the repo has real pack-grade evidence.

## 3. Authority Split and Conflict Rule

### 3.1 OxCalc Authority
OxCalc remains authoritative for:
1. coordinator semantics,
2. `TraceCalc` scenario, validator, and runner meaning,
3. reference-machine behavior,
4. Stage 1 replay classes `R1..R8`,
5. engine-diff required equality surfaces and severity meaning for OxCalc-local runs.

### 3.2 Foundation Authority
Foundation remains authoritative for:
1. cross-lane replay bundle rollout policy,
2. normalized replay object families,
3. capability levels `cap.C0.ingest_valid` through `cap.C5.pack_valid`,
4. shared registry families for predicates, mismatches, severity, reduction outcomes, and witness lifecycle,
5. witness lifecycle and quarantine governance.

### 3.3 Explicit Adaptation Rule
If Foundation wording conflicts with OxCalc semantics:
1. preserve OxCalc semantics,
2. keep the local artifact vocabulary intact,
3. adapt the replay rollout wording rather than copying it verbatim,
4. record the conflict explicitly in this document.

### 3.4 Current Explicit Adaptations
1. Foundation treats OxCalc as the first target lane through `cap.C4.distill_valid`; OxCalc currently documents that rollout path but only claims the highest locally proven capability level in the manifest.
2. Foundation normalizes event families such as `candidate.built` and `publication.committed`; OxCalc currently emits source labels such as `candidate_recorded`, `candidate_emitted`, `candidate_published`, and `publication_committed`. The adapter preserves source labels and normalizes them explicitly.

## 4. Source and Projection Boundary

### 4.1 Canonical OxCalc Source Surfaces
The adapter projects from these OxCalc-owned source surfaces:
1. `TraceCalc` scenario manifests and scenario JSON files,
2. `TraceCalc` validator and runner emitted artifacts,
3. `TraceCalc Reference Machine` oracle artifacts,
4. `engine_diff.json`,
5. Stage 1 replay classes and pack bindings from `W009`.

### 4.2 Projection Rule
The Replay appliance bundle is a projection over the OxCalc source surfaces, not a replacement for them.

That means:
1. source scenario ids remain the authoritative scenario identity,
2. source run ids remain stable and visible,
3. source trace labels remain preserved even when normalized event families are added,
4. source views remain first-class artifacts rather than being inferred from trace labels alone.

## 5. TraceCalc Scenario and Artifact Projection

### 5.1 Scenario Projection
The adapter projects each `TraceCalc` scenario into Replay appliance bundle scope using:
1. `scenario_id` -> `ReplayScenarioManifest.scenario_id`
2. `description` -> `ReplayScenarioManifest.description`
3. `tags` -> `ReplayScenarioManifest.tags`
4. `pack_tags` -> `ReplayScenarioManifest.pack_tags`
5. replay-class metadata -> bundle tags and pack/equality metadata
6. witness anchors -> reduction-unit seeds and closure references

### 5.2 Run Projection
The current OxCalc artifact root:
1. `docs/test-runs/core-engine/tracecalc-reference-machine/<run_id>/`

remains the source run root.

The normalized replay projection for that run should emit under:
1. `docs/test-runs/core-engine/tracecalc-reference-machine/<run_id>/replay-appliance/`

This root is the OxCalc-local projection boundary for the wider Replay appliance bundle contract.

### 5.3 Artifact Projection Table
| OxCalc source artifact | Replay appliance projection |
|---|---|
| `run_summary.json` | `ReplayRunManifest` seed |
| `manifest_selection.json` | selection metadata |
| `scenarios/<scenario_id>/trace.json` | normalized `ReplayEvent` stream |
| `scenarios/<scenario_id>/counters.json` | `ReplayCounterSet` |
| `scenarios/<scenario_id>/published_view.json` | `ReplayView` family `published_view` |
| `scenarios/<scenario_id>/pinned_views.json` | `ReplayView` family `pinned_view` |
| `scenarios/<scenario_id>/rejects.json` | `ReplayView` family `reject_set` |
| `scenarios/<scenario_id>/result.json` | scenario result and assertion surface |
| `conformance/oracle_baseline.json` | oracle comparison baseline |
| `conformance/engine_diff.json` | `ReplayDiff` source |

### 5.4 Required Projection Preservation
The adapter must preserve:
1. candidate admission versus candidate result versus publication,
2. typed reject outcomes,
3. `published_view`,
4. `pinned_view`,
5. reject sets,
6. assertion results,
7. pack tags and replay-class metadata,
8. engine-diff mismatch kinds.

## 6. Normalized Event-Family Mapping and Label Drift

### 6.1 Mapping Rule
The adapter must preserve source label and normalized family side by side.

The normalized family is a projection axis.
The source label remains the OxCalc-local semantic spelling.

### 6.2 Current Label Drift Resolution
The current drift that must be normalized explicitly is:
1. `candidate_recorded` versus `candidate_emitted`
2. `publication_committed` versus `candidate_published`

OxCalc uses both because:
1. workset and TLA-facing coordinator transitions emphasize `candidate_recorded` and `publication_committed`,
2. the current `TraceCalc` runner and reference machine emit `candidate_emitted` and `candidate_published`.

The adapter may not silently pick one and erase the other.

### 6.3 Mapping Table V1
| Source label | Normalized family | Notes |
|---|---|---|
| `candidate_admitted` | `candidate.admitted` | source label preserved |
| `candidate_recorded` | `candidate.built` | coordinator-facing spelling |
| `candidate_emitted` | `candidate.built` | current TraceCalc runner/reference-machine spelling |
| `candidate_rejected` | `reject.issued` | no-publish consequence required |
| `publication_committed` | `publication.committed` | coordinator-facing spelling |
| `candidate_published` | `publication.committed` | current TraceCalc runner/reference-machine spelling |
| `reader_pinned` | `session.reader_pinned` | source label preserved |
| `reader_unpinned` | `session.reader_unpinned` | source label preserved |
| `overlay_retained` | `overlay.retained` | source label preserved |
| `overlay_released` | `overlay.released` | reserve label for later widened artifacts |
| `node_verified_clean` | `candidate.verified_clean` | no-publication verification path |
| `fallback_forced` | `candidate.fallback_forced` | Stage 1 reserve until emitted |
| `eviction_eligibility_opened` | `overlay.eviction_eligible` | overlay lifecycle projection |

### 6.4 Local-Only Id Rule
If OxCalc needs a projection id not present in the Foundation handoff, it must use the `oxcalc.local.*` prefix and mark the id as local-only.

## 7. Required Preserved View Surfaces
The adapter must preserve these view surfaces as first-class artifacts:
1. `published_view`
2. `pinned_view`
3. `reject_set`
4. `assertion_result_set`
5. `counter_set`

The adapter may not reduce replay to:
1. trace labels alone,
2. counters alone,
3. prose summaries alone.

## 8. Engine-Diff Severity and Mismatch Mapping

### 8.1 Current OxCalc Mismatch Kinds
Current `engine_diff` mismatches are:
1. `missing_scenario_result`
2. `result_state_mismatch`
3. `published_view_mismatch`
4. `pinned_view_mismatch`
5. `reject_mismatch`
6. `trace_count_mismatch`
7. `counter_mismatch`
8. `unexpected_extra_artifact`

### 8.2 Normalized Mismatch Mapping
| OxCalc mismatch kind | Foundation mismatch id |
|---|---|
| `missing_scenario_result` | `mm.scenario.presence` |
| `result_state_mismatch` | `mm.result.state` |
| `published_view_mismatch` | `mm.view.value` |
| `pinned_view_mismatch` | `mm.view.value` |
| `reject_mismatch` | `mm.reject.kind` |
| `trace_count_mismatch` | `mm.trace.event` |
| `counter_mismatch` | `mm.counter.value` |
| `unexpected_extra_artifact` | `mm.sidecar.payload` |

### 8.3 Severity Mapping Rule
The adapter must assign severity using the scenario's declared required-equality surfaces.

Default rule:
1. mismatches on `published_view`, `pinned_view`, `reject_set`, `candidate_publication_boundary`, or `result_state` are `sev.semantic`,
2. mismatches on declared optional capture or comparison surfaces are `sev.instrumentation`,
3. extra optional artifacts or optional sidecars are `sev.informational`,
4. evidence or clause binding mismatches, when later emitted, are `sev.coverage`.

### 8.4 Current Rollout Limitation
Current `engine_diff.json` does not yet emit normalized mismatch ids or `severity_class` ids.
This is a documented rollout gap, not a reason to blur the local mismatch meanings.

## 9. Adapter Capability Target and Known Limits

### 9.1 Rollout Target
OxCalc is the first proving lane for the Replay appliance and should document the path through:
1. `cap.C0.ingest_valid`
2. `cap.C1.replay_valid`
3. `cap.C2.diff_valid`
4. `cap.C3.explain_valid`
5. `cap.C4.distill_valid`

### 9.2 Highest Currently Claimed Capability
The OxCalc adapter manifest currently claims only the highest locally proven level.

For this pass, the highest honest claim is:
1. `cap.C1.replay_valid`

The path to `cap.C2.diff_valid`, `cap.C3.explain_valid`, and `cap.C4.distill_valid` is documented and workset-bound, but not yet proven by local conformance artifacts.

### 9.3 Known Limits
Known current limits include:
1. normalized severity ids are not yet emitted in `engine_diff.json`,
2. replay bundle projection is specified but not yet emitted by the current runner,
3. explain records are not yet emitted by the current OxCalc runner or oracle tooling,
4. witness distillation and reduced-witness bundle emission are not yet realized,
5. `cap.C5.pack_valid` is not claimed because the repo does not yet contain pack-grade Replay appliance evidence.

## 10. Registry Version Pins

### 10.1 Pinning Rule
Until Foundation externalizes canonical machine-readable registries, OxCalc pins to the `2026-03-15` authoritative replay handoff run.

### 10.2 Current Pin Set
The adapter uses these registry families from the Foundation handoff:
1. `predicate_kind`
2. `mismatch_kind`
3. `severity_class`
4. `reduction_status`
5. `witness_lifecycle_state`
6. `capability_level`

Current version pin:
1. `foundation.replay.authoritative-pass-01.2026-03-15`

This is an interim registry version ref, not a claim that Foundation has already published separate machine-readable registry snapshots.

## 11. Witness Lifecycle and Quarantine Usage Rules

### 11.1 Lifecycle Rule
Reduced witnesses projected from OxCalc runs must use Foundation lifecycle ids:
1. `wit.generated_local`
2. `wit.explanatory_only`
3. `wit.retained_local`
4. `wit.retained_shared`
5. `wit.pack_candidate`
6. `wit.pack_promoted`
7. `wit.superseded`
8. `wit.quarantined`
9. `wit.gc_eligible`
10. `wit.archived`

### 11.2 Quarantine Rule
Quarantine reasons for OxCalc reduced witnesses must use Foundation quarantine families where applicable:
1. `oracle_unstable`
2. `capture_insufficient`
3. `source_artifact_missing`
4. `schema_incompatible`
5. `adapter_bug_suspected`
6. `replay_invalid`
7. `policy_blocked`

### 11.3 Pack Eligibility Rule
Explanatory-only or quarantined witnesses are not pack-eligible.
OxCalc may retain them locally for triage, but the adapter must not present them as pack-valid.

### 11.4 Replay-Valid Default Rule
Reduced witnesses should remain replay-valid by default.
If a reduced artifact is explanatory-only rather than replay-valid, that status must be explicit and the witness must remain out of pack-eligible states.

## 12. Open Alignment Items and Follow-On Evidence
Open alignment items include:
1. closing the gap between current source trace labels and normalized event-family emission,
2. emitting normalized mismatch ids and `severity_class` ids in `engine_diff.json`,
3. emitting a normalized bundle root under each run's `replay-appliance/` directory,
4. realizing explain records over `engine_diff`, view mismatches, and reject sets,
5. realizing reduced-witness bundles and lifecycle records for OxCalc-local failures,
6. proving `cap.C2.diff_valid`, `cap.C3.explain_valid`, and `cap.C4.distill_valid` with local conformance artifacts.

## 13. Current Semantic Conflict Notes
The following conflicts remain explicitly adapted rather than silently resolved:
1. label drift between `candidate_recorded` and `candidate_emitted`,
2. label drift between `publication_committed` and `candidate_published`,
3. Foundation's rollout target through `cap.C4.distill_valid` versus OxCalc's current proven capability floor at `cap.C1.replay_valid`.

These conflicts do not justify weakening OxCalc semantics.
They justify explicit normalization and staged rollout.

## 14. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - normalized bundle emission is specified but not yet emitted by the current runner,
  - `engine_diff.json` does not yet emit normalized mismatch ids or severity ids,
  - explain and distillation flows remain planned rather than realized,
  - the adapter manifest intentionally stops short of `cap.C2.diff_valid`
- claim_confidence: provisional
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
