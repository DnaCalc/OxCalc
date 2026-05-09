# W046 TraceCalc Refinement Kernel And Replay Binding

Status: `calc-gucd.6_tracecalc_refinement_kernel_validated`

Workset: `W046`

Parent epic: `calc-gucd`

Bead: `calc-gucd.6`

## 1. Purpose

This packet defines the first W046 TraceCalc refinement kernel and binds current TreeCalc/CoreEngine replay evidence against it.

The scoped goal is to make the oracle relation explicit before W046 returns to proof-service, release-readiness, or performance classification:

1. TraceCalc is the selected-kernel reference handler for covered scenarios.
2. TreeCalc/CoreEngine is an optimized/production handler that must match or refine the TraceCalc observable packet.
3. The comparison surface includes values, diagnostics, dependency effects, invalidation records, rejection, publication decisions, and required trace families.
4. Rows outside the currently comparable surface are exact blockers, not conformance matches.

This packet is intentionally narrower than a full mechanized semantic proof of the Rust calculation engine. It creates a checked refinement vocabulary, a bounded TLA comparison model, and a current evidence binding register for the selected kernel.

## 2. Implementation Crosswalk

| Semantic object or transition | Rust surface | Model surface | Evidence root |
| --- | --- | --- | --- |
| execution artifact packet | `TraceCalcExecutionArtifacts` at `src/oxcalc-tracecalc/src/contracts.rs:308` | Lean `ObservablePacket`; TLA oracle/engine packet variables | TraceCalc scenario `result.json`, `oracle_baseline.json`, `engine_diff.json` |
| mismatch classification | `TraceCalcConformanceMismatchKind` at `contracts.rs:35` | exact equality or subset-preservation obligations | `docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/conformance/engine_diff.json` |
| scenario validation | `validate_scenario` at `contracts.rs:380` | oracle coverage precondition | TraceCalc run summary and oracle matrix |
| artifact comparator | `compare_artifacts` at `assertions.rs:119` | result/value/pinned/reject/trace/counter comparison input | W037 TraceCalc closure run |
| shared reference/engine execution loop | `execute_shared` at `machine.rs:61`; `execute_step` at `machine.rs:102` | TraceCalc reference-machine transition vocabulary | W037 scenario artifacts |
| candidate admission | `admit_work` at `machine.rs:256` | candidate/publication boundary | accept/publish, verified-clean, and reject rows |
| candidate result | `emit_candidate_result` at `machine.rs:401` | `publicationDecision`, `publishedValues`, dependency effects | accept, DAG, dynamic dependency rows |
| rejection | `emit_reject` at `machine.rs:497` | exact reject set and no-publication rule | reject and capability-fence rows |
| publication | `publish_candidate` at `machine.rs:578` | atomic publication decision in the observable packet | accept/publish and DAG rows |
| verified clean | `verify_clean` at `machine.rs:636` | no-publication terminal decision | verified-clean row |
| runner emission | `TraceCalcRunner::execute_manifest` at `runner.rs:67`; `oracle_baseline_object` at `runner.rs:1109`; `engine_diff_object` at `runner.rs:1131` | artifact binding and replay root | W037 TraceCalc run root |
| workset planning and SCC/order | `TraceCalcScenarioPlanner::plan_workset` at `planner.rs:58`; `compute_components` at `planner.rs:170`; `is_cycle_group` at `planner.rs:259` | graph/order/invalidation facts consumed by the refinement surface | graph, order, and invalidation scenarios |

## 3. Refinement Relation

The relation is observable, not implementation-structural.

For a covered row, the engine packet refines the TraceCalc oracle packet when:

1. the oracle scenario result passed;
2. scenario identity is the same;
3. engine result state equals oracle result state;
4. published values match exactly;
5. diagnostics match exactly;
6. dependency effects required by the oracle are present in the engine packet;
7. invalidation records required by the oracle are present in the engine packet;
8. reject facts match exactly;
9. publication decision matches exactly;
10. trace families required by the oracle are present in the engine packet.

The subset clauses are deliberate. They allow TreeCalc/CoreEngine to expose extra instrumentation or conservative invalidation facts while preserving required semantic facts. They do not permit missing oracle dependency effects, missing oracle invalidation records, value drift, diagnostic drift, reject drift, or publication-decision drift.

Rows that cannot currently be compared on the required surface must be registered as exact blockers. They are not promoted as matches.

## 4. Checked Lean Artifact

Artifact: `formal/lean/OxCalc/CoreEngine/W046TraceCalcRefinement.lean`

Checked command:

```powershell
lean formal\lean\OxCalc\CoreEngine\W046TraceCalcRefinement.lean
```

Result: passed.

Lean definitions:

1. `ObservablePacket`: scenario id, result state, published values, diagnostics, dependency effects, invalidation records, rejects, publication decision, and trace families.
2. `ObservableRefinement`: exact result/value/diagnostic/reject/publication matching plus dependency, invalidation, and trace-family subset preservation.
3. `RefinementBinding`: a covered-or-blocked row wrapper with exact blocker text.
4. `BindingHasExactBlocker`: the non-covered exact-blocker classification.

Checked theorem targets:

| Theorem | Contract |
| --- | --- |
| `refinement_preserves_result_state` | covered refinement preserves result state |
| `refinement_preserves_published_values` | covered refinement preserves exact published values |
| `refinement_preserves_diagnostics` | covered refinement preserves exact diagnostics |
| `refinement_preserves_dependency_effects` | covered refinement preserves required dependency effects |
| `refinement_preserves_invalidation_records` | covered refinement preserves required invalidation records |
| `refinement_preserves_rejects` | covered refinement preserves exact reject facts |
| `refinement_preserves_publication_decision` | covered refinement preserves publication decision |
| `refinement_preserves_required_trace_families` | covered refinement preserves required trace families |
| `refined_reject_no_publish` | a refined oracle reject/no-publish row is engine no-publish |
| `sample_accept_refines` | accept/publish sample satisfies the relation |
| `sample_reject_refines` | reject/no-publish sample satisfies the relation |
| `sample_dynamic_refines` | dynamic dependency sample permits extra instrumentation while preserving oracle facts |
| `sample_blocked_binding_has_exact_blocker` | uncovered dynamic projection row is classified as an exact blocker |

## 5. Checked TLA Artifact

Artifacts:

1. `formal/tla/CoreEngineW046TraceCalcRefinement.tla`
2. `formal/tla/CoreEngineW046TraceCalcRefinement.smoke.cfg`
3. `docs/test-runs/core-engine/tla/w046-tracecalc-refinement-001/`

Checked command:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run-tlc.ps1 formal\tla\CoreEngineW046TraceCalcRefinement.tla formal\tla\CoreEngineW046TraceCalcRefinement.smoke.cfg
```

Result: passed.

TLC summary:

| Field | Value |
| --- | --- |
| TLC version | `2.19 of 08 August 2024` |
| states generated | `11` |
| distinct states | `6` |
| queue left | `0` |
| complete-state depth | `2` |
| result | no error found |

Checked invariants:

1. `TypeInvariant`
2. `CoveredRowsHaveNoExactBlockers`
3. `ResultStateMatches`
4. `PublishedValuesMatch`
5. `DiagnosticsMatch`
6. `RejectsMatch`
7. `DependencyEffectsPreserved`
8. `InvalidationRecordsPreserved`
9. `PublicationDecisionMatches`
10. `RequiredTraceFamiliesPreserved`
11. `RejectIsNoPublish`
12. `NoSemanticMismatchForCoveredRows`

Smoke model shape:

1. accept/publish value row;
2. verified-clean no-publication row;
3. reject/no-publication row;
4. dynamic-dependency row with preserved required dependency effects and extra engine instrumentation;
5. invalidation-closure row with preserved required invalidation records and extra conservative engine record.

## 6. Replay Binding Root

Canonical binding root: `docs/test-runs/core-engine/refinement/w046-tracecalc-refinement-kernel-001/`

Checked-in artifacts:

1. `run_summary.json`
2. `kernel_binding_register.json`
3. `validation.json`

Source evidence:

| Source | Role |
| --- | --- |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/run_summary.json` | TraceCalc reference run: 31 scenarios passed |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/conformance/engine_diff.json` | TraceCalc engine-vs-oracle diff: 31 rows, 0 mismatches |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/oracle-matrix/run_summary.json` | oracle matrix: 32 rows, 31 covered, 1 excluded |
| `docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/comparisons/treecalc_tracecalc_differential.json` | TreeCalc/TraceCalc differential rows |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/` | TreeCalc local cases, including post-edit invalidation/recalc evidence |
| `docs/test-runs/core-engine/tla/w046-tracecalc-refinement-001/` | bounded TLA refinement check |

Binding summary:

| Metric | Value |
| --- | --- |
| binding rows | `12` |
| matched selected-kernel refinement rows | `8` |
| TraceCalc oracle self-check rows | `1` |
| exact blocker rows | `3` |
| unexpected mismatches | `0` |
| missing referenced artifacts | `0` |

Matched selected-kernel rows:

1. accept/publish value surface;
2. multi-node DAG value and order surface;
3. narrow LET/LAMBDA carrier value surface;
4. replay-equivalent independent-order value surface;
5. verified-clean no-publication surface;
6. typed reject no-publication surface;
7. capability-fence reject no-publication surface;
8. overlay lifecycle and semantic-counter surface.

Exact blockers:

| Blocker | Meaning |
| --- | --- |
| `treecalc_local_dynamic_dependency_projection_gap` | current local TreeCalc projection does not yet expose enough comparable dynamic dependency facts for this row |
| `treecalc_local_dynamic_dependency_shape_update_projection_gap` | current local TreeCalc projection does not yet expose enough comparable dynamic negative shape-update facts for this row |
| `tracecalc_to_treecalc_normalized_invalidation_record_comparator_missing` | current evidence lacks a normalized TraceCalc-to-TreeCalc invalidation-record comparator for full invalidation-record comparison |

## 7. Assumptions And Limits

1. The Lean artifact proves preservation properties of the abstract observable relation; it does not prove Rust implementation totality or line-by-line refinement.
2. The TLA artifact is a bounded model over selected row shapes, not full TLA verification.
3. The binding register uses existing W036/W037 evidence; it does not regenerate TraceCalc or TreeCalc runs.
4. Dynamic dependency and invalidation-record rows remain exact blockers where the current comparator/projection surface is not strong enough.
5. The narrow LET/LAMBDA carrier row is included because it is a visible cross-boundary value case; broader OxFml/OxFunc seam formalization remains `calc-gucd.7`.
6. Performance, proof-service, C5, operated-service, pack-grade replay, Stage 2, independent-evaluator breadth, and release-grade consequences remain downstream lanes.

## 8. Successor Obligations

| Successor bead | Consumes from this packet |
| --- | --- |
| `calc-gucd.7` | observable formula/evaluator facts and the LET/LAMBDA carrier row for the OxFml seam model |
| `calc-gucd.8` | checked refinement relation and exact blockers for proof-service/evidence coverage recast |
| `calc-gucd.9` | phase/event vocabulary and binding register as correctness signatures for scale/performance runs |
| `calc-gucd.10` | honest downstream consequence matrix over matched rows and exact blockers |
| `calc-gucd.11` | semantic-spine coverage audit and successor routing |

## 9. Validation

| Command | Result |
| --- | --- |
| `lean formal\lean\OxCalc\CoreEngine\W046TraceCalcRefinement.lean` | passed |
| `powershell -ExecutionPolicy Bypass -File scripts\run-tlc.ps1 formal\tla\CoreEngineW046TraceCalcRefinement.tla formal\tla\CoreEngineW046TraceCalcRefinement.smoke.cfg` | passed |
| `cargo test -p oxcalc-tracecalc` | passed; 12 tests passed |
| `cargo test -p oxcalc-core local_treecalc_engine_` | passed; 9 tests passed |
| `powershell -ExecutionPolicy Bypass -File scripts\check-worksets.ps1` | passed |
| `br dep cycles --json` | passed with `count: 0` |
| JSON parse check for `w046-tracecalc-refinement-001` and `w046-tracecalc-refinement-kernel-001` | passed |
| TLC log non-empty check for `w046-tracecalc-refinement-001` | passed |
| `rg -n "\b(sorry|admit|axiom)\b" formal\lean\OxCalc\CoreEngine\W046TraceCalcRefinement.lean` | passed; no matches |
| `git diff --check` | passed; emitted line-ending normalization warnings only |

## 10. Semantic-Equivalence Statement

This bead changes formal artifacts, evidence metadata, spec/status documents, and bead state only.

Observable OxCalc behavior is invariant under this bead. It does not change dependency graph construction, invalidation closure, soft-reference or dynamic-reference rebind behavior, recalc tracker behavior, evaluation order implementation, working-value reads, TraceCalc execution, TreeCalc/CoreEngine execution, OxFml/OxFunc behavior, rejection, publication, pack policy, proof-service policy, performance behavior, or service readiness.

## 11. Reviewed Inbound Observations

Reviewed `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`.

Relevant intake for this bead:

1. OxFml remains authoritative for evaluator artifact meaning, reject semantics, replay-safe identity, and fence meaning.
2. Candidate and commit remain distinct artifact stages; reject remains no-publish.
3. Current consumer/runtime surfaces preserve candidate, commit, reject, trace, capability-sensitive, execution-restriction, topology/effect, and dependency-sensitive fact families.
4. Dynamic dependency additions/removals/reclassifications should remain surfaced evaluator/runtime facts rather than coordinator-inferred policy.
5. No new OxFml handoff is needed because this bead records an OxCalc-local refinement surface and does not change canonical FEC/F3E or OxFml evaluator-facing clauses.

## 12. Pre-Closure Verification Checklist

| # | Check | Result |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W046 status surfaces, transition catalog, fragment ledger, semantic plan, and formal layout note the TraceCalc refinement kernel |
| 2 | Pack expectations updated for affected packs? | yes; no pack expectation changed by this model/binding bead |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; the binding register points to deterministic TraceCalc, TreeCalc, and independent conformance artifacts, and the TLA evidence root is checked in |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no policy/strategy change, invariant statement in Section 10 |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no normative FEC/F3E seam change and no OxFml handoff needed |
| 6 | All required tests pass? | yes; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for the declared `calc-gucd.6` target; full Rust implementation proof, full TraceCalc oracle promotion, dynamic dependency projection, normalized invalidation comparator, OxFml seam closure, and unbounded TLA remain explicit successor or blocker lanes |
| 8 | Completion language audit passed? | yes; no full Rust, full TraceCalc, full TreeCalc/CoreEngine, full dynamic-dependency, full invalidation, OxFml-seam, or release-grade proof is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; no ordered workset register change required |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; current semantic bead moved to `calc-gucd.7` |
| 11 | execution-state blocker surface updated? | yes; `.beads/` owns `calc-gucd.6` state |

## 13. Completion Claim Self-Audit

| Step | Result |
| --- | --- |
| Scope re-read | pass; bead asks for selected formula/model kernels where TraceCalc is the reference handler and TreeCalc/CoreEngine replay against it |
| Gate criteria re-read | pass; values, diagnostics, dependency effects, invalidation records, rejection, and publication decisions are compared for matched rows or recorded as exact blockers |
| Silent scope reduction check | pass; dynamic dependency projection and invalidation comparator gaps are explicit blockers, not hidden narrowed scope |
| "Looks done but is not" pattern check | pass; the packet says the model and binding are checked for the selected declared surface, not a full mechanized semantic proof of all engine behavior |
| Include result | pass; checklist, self-audit, semantic-equivalence statement, and three-axis report are included |

## 14. Current Status

- execution_state: `calc-gucd.6_closed`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
