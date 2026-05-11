# W048 Engine And Formalization Roadmap

Status: `active_execution_workset`

## 1. Purpose

This roadmap turns W048 cycle design into executable lanes. It intentionally separates:

1. evidence collection from Excel;
2. graph materialization;
3. non-iterative cycle policy;
4. iterative-profile decision work;
5. TraceCalc reference implementation;
6. TreeCalc optimized/core implementation;
7. circular-reference test corpus and conformance execution;
8. W048 formal definitions, Lean/TLA/checker artifacts, and later W049 successor consumption;
9. innovation opportunities and experimental profiles.

## 2. Bead Rollout

Parent epic: `calc-zci1`

| Bead | Lane | Reviewable outcome |
| --- | --- | --- |
| `calc-zci1.1` | Excel probes | black-box observation packet and normalized ledger |
| `calc-zci1.2` | graph materialization | structural/effective/candidate graph sidecar contract and engine artifact widening |
| `calc-zci1.3` | TraceCalc reference implementation | reference behavior, fixtures, and run artifacts for structural cycles, CTRO-created cycles, reject, downstream state, release/re-entry |
| `calc-zci1.4` | iterative profile | algorithm decision ledger for root/order, initial vector, update model, stop metric, publication, and Excel disposition |
| `calc-zci1.5` | W048 formal artifacts | cycle definitions, Lean/TLA/checker targets, and evidence bindings introduced under W048 |
| `calc-zci1.6` | TreeCalc optimized implementation | optimized/core behavior and graph artifacts conforming to TraceCalc for declared cases |
| `calc-zci1.7` | test corpus | deterministic fixture corpus, runners, and differential/conformance result roots |
| `calc-zci1.8` | innovation ledger | OxCalc-specific cycle handling opportunities and opt-in experimental profiles |

## 3. TraceCalc Targets

TraceCalc is the reference behavior lane. It should grow before TreeCalc optimization work when a semantic choice is still unsettled, and it remains the first executable expression of W048's cycle semantics.

Required TraceCalc artifacts:

1. materialized graph output for direct and reverse dependencies;
2. graph layer marker for structural, published-effective, and candidate-effective scenarios;
3. cycle-region records with `cycle_source`, members, root/order, and terminal policy;
4. non-iterative cycle reject fixture for structural SCC;
5. CTRO-created cycle reject fixture;
6. CTRO cycle release/re-entry fixture;
7. downstream dependent fixture for blocked/released cycle;
8. iterative fixture only after `calc-zci1.4` chooses a profile.

First W048 TraceCalc evidence now lives in `W048_TRACECALC_REFERENCE_CYCLE_BEHAVIOR.md` and run root `docs/test-runs/core-engine/tracecalc-reference-machine/w048-tracecalc-cycles-003/`. That run passes 34 scenarios, including W048 structural self-cycle reject, CTRO candidate-cycle reject/no-overlay-commit, and CTRO release/re-entry/downstream recomputation fixtures.

Reference fixture families:

1. `tc_w048_structural_self_cycle_reject_001`;
2. `tc_cycle_region_reject_001` for prior two-node SCC coverage;
3. `tc_w048_ctro_candidate_cycle_reject_001`;
4. `tc_w048_ctro_release_reentry_downstream_001`;
5. `tc_cycle_iter_order_probe_001` after iterative policy selection.

## 4. TreeCalc Targets

TreeCalc is the optimized/core engine lane. It should match TraceCalc for declared behavior and emit enough artifacts for replay/checker work.

Required TreeCalc changes:

1. emit native reverse-edge sidecar facts, not only forward edges;
2. emit graph layer/basis metadata;
3. emit effective and candidate graph facts for CTRO waves;
4. emit overlay delta records;
5. emit cycle-region records with provenance;
6. route CTRO-created cycles through the shared cycle policy;
7. preserve no-publication/no-overlay-commit on rejected candidate cycles;
8. define release/re-entry invalidation seeds and downstream state;
9. add run artifacts for the same semantic families as TraceCalc.

Current useful floor:

1. in-memory reverse edges already exist;
2. structural cycle groups already exist;
3. invalidation can mark cycle members;
4. local cycle detection already maps to a reject path.

W048 should widen artifacts and policy while also adding the TreeCalc behavior needed for declared cycle cases. Performance-oriented dynamic topology remains a possible later optimization only after semantic behavior and artifacts are stable enough.

## 5. Non-Iterative Stage 1 Policy

Provisional Stage 1 target:

1. classify SCCs over the materialized graph layer;
2. for `G_struct` or `G_eff` cycle regions, mark affected members `cycle_blocked` and reject formula-family publication under the current profile;
3. for `G_eff_candidate` CTRO-created cycle regions, reject the whole candidate bundle unless W048 later adopts a narrower partition publication rule with evidence;
4. publish no new cycle-region values;
5. commit no candidate overlay that created the rejected cycle;
6. retain previous published values as prior state, not as new publication;
7. emit diagnostics that distinguish source/provenance and terminal policy;
8. emit downstream state using reverse-reachability from the cycle region and changed/released edges.

This is intentionally conservative. It prevents a dynamic edge discovered during evaluation from partially mutating published graph state after a failed candidate.

## 6. Iterative Profile Work

W048 should not add hidden iteration behavior. Iteration requires a profile record:

```json
{
  "profile_id": "excel_iterative_candidate_v1",
  "root_policy": "calculation_chain_root",
  "member_order_policy": "calculation_chain_order",
  "initial_value_policy": "published_prior_value",
  "update_model": "excel_chain_ordered",
  "max_iterations": 100,
  "max_change": 0.001,
  "change_metric": "max_abs_numeric_delta",
  "terminal_publication_policy": "publish_last_iterate",
  "divergence_policy": "stop_by_bound_with_diagnostic",
  "oscillation_policy": "stop_by_bound_or_change_metric"
}
```

The values above are placeholders for decision structure, not a selected profile. Excel probes must drive the Excel-compatible candidate fields.

## 7. W048 Formal Artifact Targets

W048 owns the first formal cycle artifacts. W049 should later receive a packet, not an intuition, but W048 is responsible for introducing the definitions and initial checks tied to its implementations.

Required W048 formal inputs:

1. graph materialization schema;
2. representative graph artifacts from TraceCalc and TreeCalc;
3. reverse-edge converse invariant evidence;
4. SCC/cycle-region classification fixtures;
5. no-publication/no-overlay-commit reject fixture;
6. release/re-entry fixture;
7. Excel disposition note: matched, intentionally different, or deferred;
8. iterative profile decision or explicit deferral.

Candidate W048 theorem/model targets:

1. `graphLayerCompositionDeterministic`;
2. `reverseEdgesAreForwardConverse`;
3. `cycleRegionsAreSccsOrSelfLoops`;
4. `candidateCycleRejectCommitsNoOverlay`;
5. `cycleRejectPublishesNoNewCycleValues`;
6. `cycleReleaseInvalidatesMembersAndDependents`;
7. `iterativeProfileDeterministicUnderFixedInputs`;
8. `schedulerChangePreservesObservableCycleProfile`.

The W048 formal lane may produce Lean definitions/theorem targets, TLA models, and checker rules. The target is useful executable/formal pressure over the W048 behavior, not a decorative proof inventory.

## 8. Test Corpus And Conformance Outputs

W048 owns a deterministic circular-reference corpus:

1. Excel probe definitions and observation packets;
2. TraceCalc reference fixtures;
3. TreeCalc local fixtures;
4. cross-engine comparison results;
5. graph sidecar and cycle-region artifact checks;
6. no-publication/no-overlay-commit assertions;
7. cycle release/re-entry assertions;
8. iterative-profile tests after the profile decision exists.

The corpus should distinguish:

1. `excel_observation`;
2. `tracecalc_reference`;
3. `treecalc_core`;
4. `formal_model`;
5. `checker_projection`.

## 9. Excel Disposition Outputs

For each Excel probe family W048 should classify:

1. `matched_by_profile`;
2. `matched_visible_surface_only`;
3. `intentionally_different_with_reason`;
4. `deferred_unobserved`;
5. `out_of_scope_data_table_or_host_specific`.

No Excel-match statement should be made without:

1. observation packet path;
2. Excel version/build;
3. calculation settings;
4. command sequence;
5. cell snapshots;
6. mapping to OxCalc profile fields.

## 10. Innovation Lane

W048 should capture opportunities for OxCalc to handle circular dependencies better than ordinary spreadsheet behavior, while keeping Excel compatibility explicit.

Candidate innovation themes:

1. clear opt-in profiles for monotone fixed-point cycles;
2. diagnostic cycle-region views that show entry/exit edges and root/order policy;
3. stable materialized graph exports for model audit;
4. safer non-iterative retained-prior-value handling;
5. explicit recurrence constructs that avoid accidental circular references;
6. bounded iterative profiles with divergence/oscillation diagnostics;
7. local frontier repair after cycle release;
8. profile-checked semantic equivalence under scheduler changes.

Any innovation must be profile-gated. The default Excel-match lane cannot silently adopt behavior that contradicts observed Excel behavior.

## 11. Exit Gate

W048 can route successor formalization or optimization work only when:

1. W048 docs state the selected non-iterative cycle policy;
2. materialized graph requirements are bound to TraceCalc/TreeCalc artifacts or exact blockers;
3. structural and CTRO-created cycle fixtures exist or exact blockers are recorded;
4. cycle release/re-entry behavior is specified and exercised or exact-blocked;
5. Excel observation disposition is recorded for the core probe set;
6. iterative profile is either selected for an initial lane or explicitly deferred with decision axes preserved;
7. W048 formal definitions/models/checker targets cite concrete W048 artifacts;
8. test corpus and run artifacts exist for the declared scope;
9. innovation opportunities are recorded separately from default Excel-match behavior.

This is an exit gate for successor routing, not a claim that every possible Excel cycle behavior has been reproduced.

## 12. Status Surface

- execution_state: `in_progress`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - `calc-zci1.1`
  - `calc-zci1.2`
  - `calc-zci1.3`
  - `calc-zci1.4`
  - `calc-zci1.5`
  - `calc-zci1.6`
  - `calc-zci1.7`
  - `calc-zci1.8`
