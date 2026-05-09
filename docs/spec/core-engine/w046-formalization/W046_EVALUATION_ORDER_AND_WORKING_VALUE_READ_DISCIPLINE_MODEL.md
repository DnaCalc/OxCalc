# W046 Evaluation Order And Working-Value Read-Discipline Model

Status: `calc-gucd.5_evaluation_order_read_discipline_model_validated`

Workset: `W046`

Parent epic: `calc-gucd`

Bead: `calc-gucd.5`

## 1. Purpose

This packet models the semantic slice that starts after graph build, invalidation, rebind, and recalc tracker entry, and ends before the broader TraceCalc refinement relation.

It covers:

1. topological formula order or cycle rejection;
2. the working-value read discipline used by sequential TreeCalc evaluation;
3. the rule that formula reads come from stable seeded values or prior ordered computed values;
4. diagnostic short-circuit behavior before publication;
5. verified-clean no-publication behavior as it appears inside the evaluation loop;
6. no-torn-candidate bundle targets for the successful changed-value path;
7. exact limits left to `calc-gucd.6` and `calc-gucd.7`.

The packet is scoped to the W046 semantic proof-spine target. It does not claim a full mechanized semantic proof of the Rust engine, arbitrary finite graph topological correctness, full TraceCalc refinement, or release-grade readiness.

## 2. Implementation Crosswalk

| Semantic object or transition | Rust surface | Model surface | Evidence root |
| --- | --- | --- | --- |
| working-value seed | `treecalc.rs:277`; `seed_working_values` at `treecalc.rs:1099` | Lean `stableInputs`; TLA `StableInputs`, `workingValues` | TreeCalc local published-value and result artifacts |
| evaluation-order selection | `topological_formula_order` call at `treecalc.rs:305`; implementation at `treecalc.rs:1112` | Lean `BeforeInOrder`, `FormulaDependencyBeforeDependent`; TLA `SelectAcyclicOrder`, `SelectedOrderIsTopological` | `tc_local_recalc_chain_after_constant_edit_001` expected and observed evaluation order |
| cycle rejection | cycle error branch at `treecalc.rs:307` | Lean `CycleRejectNoEvaluation`; TLA `CycleReject`, `CycleRejectNoEvaluation` | TraceCalc `tc_cycle_region_reject_001`; local cycle reject tests |
| rebind gate before evaluation | scan at `treecalc.rs:329` and `requires_rebind` check at `treecalc.rs:333` | precondition inherited from `calc-gucd.3`; TLA keeps rebind out of this slice | `tc_local_rebind_after_rename_001` |
| evaluation loop | `for node_id in &evaluation_order` at `treecalc.rs:394` | TLA `EvaluateY*`, `EvaluateZ*`; Lean `EvaluationOrderSemanticModel` | TreeCalc trace `evaluate_node` events from `treecalc_runner.rs:883` |
| OxFml call with current working values | `evaluate_via_oxfml(prepared, &working_values)` at `treecalc.rs:400`; host packet at `treecalc.rs:1594` | Lean `StableOrPriorRead`; TLA `StableOrPriorReadDiscipline` | dynamic, host-sensitive, direct-formula, and chain cases |
| diagnostic/evaluation failure | failure branch around `treecalc.rs:427` | Lean `DiagnosticShortCircuit`; TLA `DiagnosticFailure`, `DiagnosticShortCircuitNoPublish` | `tc_local_dynamic_reject_001`; `tc_local_host_sensitive_reject_001` |
| verified clean | equality branch and tracker write at `treecalc.rs:448` | Lean `VerifiedCleanNoPublication`; TLA `FinalizeVerifiedClean`, `VerifiedCleanNoPublication` | `tc_local_verified_clean_001` |
| candidate result and working-value update | candidate write at `treecalc.rs:452`; `working_values.insert` at `treecalc.rs:457` | Lean `NoTornCandidateBundle`; TLA `PublishCandidate`, `NoTornCandidateBundle` | `tc_local_recalc_chain_after_constant_edit_001/post_edit` |
| local candidate bundle | `LocalEvaluatorCandidate` at `treecalc.rs:497` | Lean `candidateTargetSet`; TLA `candidateTargetSet` | candidate/publication result artifacts |
| publication handoff | `accept_and_publish` at `treecalc.rs:509`; tracker clear at `treecalc.rs:511` | TLA terminal publish path; recalc/coordinator model from `calc-gucd.4` | publication bundle and published-values artifacts |
| expected order assertion | `treecalc_runner.rs:429` | replay evidence link, not a formal proof object | TreeCalc conformance cases |

## 3. Phase Contracts

`T09.SelectEvaluationOrder`:

1. input: a built dependency graph and the formula owner set;
2. acyclic postcondition: every formula-to-formula dependency is ordered dependency-before-dependent;
3. stable-input postcondition: non-formula dependencies are read from seeded published or constant working values;
4. cycle postcondition: formula cycle detection routes to rejection before evaluation and before publication;
5. model limit: the current Lean/TLA slice checks a small explicit graph shape and theorem vocabulary, not the Rust queue algorithm for arbitrary finite graphs.

`T10.RebindGate`:

1. input: invalidation closure records and the selected evaluation order;
2. postcondition: an evaluated node with `requires_rebind` rejects before value publication;
3. this bead treats the rebind gate as a precondition for the evaluation-order/read-discipline slice because `calc-gucd.3` already modeled stale-binding no-publish;
4. rebind evidence remains listed here because the Rust call path scans `evaluation_order` before the first formula evaluation.

`T12.ReadWorkingValue`:

1. input: `working_values`, formula dependency references, and current formula owner;
2. read-source rule: a reference may be read only from a stable seeded value or from a prior node in the selected evaluation order;
3. future candidate values are not readable;
4. after a successful changed computation, the node value is inserted into `working_values` so later dependents may read it;
5. verified-clean computations do not insert an updated value because the seeded published value already equals the computed value.

`T13.EvaluateFormula`:

1. precondition: the tracker has begun evaluation for the node, and the rebind gate did not reject it;
2. evaluation delegates to OxFml using a host packet constructed from current `working_values`;
3. success returns a computed value plus diagnostics and runtime effects;
4. failure routes through typed reject detail and does not publish;
5. OxFml may surface runtime/effect facts, but publication remains OxCalc-owned.

`T14.VerifyClean`:

1. condition: computed value equals the seeded published value;
2. postcondition: tracker state becomes verified-clean and demand clears;
3. postcondition: no candidate result or publication bundle is emitted for that node;
4. model note: this bead only models the visible no-publication effect, while `calc-gucd.4` owns the tracker pre/post detail.

Candidate bundle and no-torn target:

1. changed successful values are accumulated into `value_updates`;
2. the local candidate target set is the selected `evaluation_order`;
3. publication is modeled as one terminal candidate decision after the full order is evaluated;
4. no-torn target: any published changed-value bundle has candidate targets equal to the selected order, and every updated node is inside that target set.

## 4. Checked Lean Artifact

Artifact: `formal/lean/OxCalc/CoreEngine/W046EvaluationOrderReadDiscipline.lean`

Checked command:

```powershell
lean formal\lean\OxCalc\CoreEngine\W046EvaluationOrderReadDiscipline.lean
```

Result: passed.

Lean definitions:

1. `DependencyEdge`: owner-target dependency relation.
2. `BeforeInOrder`: list-order relation where a target precedes its owner.
3. `FormulaDependencyBeforeDependent`: formula-to-formula edges must respect order.
4. `StableOrPriorRead`: each dependency read must be from stable inputs or a prior ordered formula result.
5. `EvaluationOrderSemanticModel`: proof-carrier envelope for dependency-before-dependent and stable/prior reads.
6. `EvaluationDecision`: bounded decision envelope for computed nodes, updated nodes, candidate targets, rejection, publication, verified-clean, and cycle rejection.
7. `DiagnosticShortCircuit`, `CycleRejectNoEvaluation`, `VerifiedCleanNoPublication`, and `NoTornCandidateBundle`: terminal decision contracts.

Checked theorem targets:

| Theorem | Contract |
| --- | --- |
| `model_dependency_before_dependent` | semantic model exposes dependency-before-dependent proof target |
| `model_stable_or_prior_reads` | semantic model exposes stable/prior read-discipline proof target |
| `model_diagnostic_short_circuit` | diagnostic failure implies rejection and no publication |
| `model_cycle_reject_no_evaluation` | cycle rejection implies rejection, no publication, and no computed nodes |
| `model_verified_clean_no_publication` | verified clean has no updates and no publication |
| `model_no_torn_candidate_bundle` | publication target set equals the selected order and covers updated nodes |
| `sample_dependency_before_dependent` | sample `X:=2; Y:=X*20; Z:=X+Y` order has `Y` before `Z` |
| `sample_stable_or_prior_reads` | sample reads use stable `X` or prior computed `Y` |
| `sample_*` decision theorems | sample publish, diagnostic failure, verified-clean, and cycle decisions satisfy terminal contracts |

## 5. Checked TLA Artifact

Artifacts:

1. `formal/tla/CoreEngineW046EvaluationOrder.tla`
2. `formal/tla/CoreEngineW046EvaluationOrder.smoke.cfg`
3. `docs/test-runs/core-engine/tla/w046-evaluation-order-001/`

Checked command:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run-tlc.ps1 formal\tla\CoreEngineW046EvaluationOrder.tla formal\tla\CoreEngineW046EvaluationOrder.smoke.cfg
```

Result: passed.

TLC summary:

| Field | Value |
| --- | --- |
| TLC version | `2.19 of 08 August 2024` |
| states generated | `24` |
| distinct states | `16` |
| queue left | `0` |
| complete-state depth | `5` |
| result | no error found |

Checked invariants:

1. `TypeInvariant`
2. `SelectedOrderIsTopological`
3. `EvaluationOrderPrefix`
4. `WorkingValuesContainStableInputs`
5. `WorkingValuesCoverComputedNodes`
6. `StableOrPriorReadDiscipline`
7. `NoFutureReadEvents`
8. `DiagnosticShortCircuitNoPublish`
9. `CycleRejectNoEvaluation`
10. `VerifiedCleanNoPublication`
11. `NoTornCandidateBundle`
12. `RejectNoPublication`
13. `TerminalDecisionIsExclusive`

Smoke model shape:

1. three-node sample vocabulary: stable input `X`, formula owners `Y` and `Z`;
2. acyclic selected order `Y, Z`;
3. evaluation actions that either produce changed values or verified-clean values;
4. read events that classify `X` as stable and `Y` as prior when `Z` evaluates;
5. terminal branches for changed-value publication, verified-clean finalization, diagnostic rejection, and cycle rejection;
6. bounded transition depth of five actions.

## 6. Replay Roots

| Root | Use in this bead |
| --- | --- |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_recalc_chain_after_constant_edit_001/result.json` | initial `X=2`, `Y=X*20`, `Z=X+Y` order and value evidence |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_recalc_chain_after_constant_edit_001/post_edit/result.json` | post-edit `X=3` recomputation, order, candidate, and publication evidence |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_recalc_chain_after_constant_edit_001/post_edit/trace.json` | ordered `evaluate_node` trace events |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_verified_clean_001/result.json` | verified-clean no-candidate/no-publication path |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_dynamic_reject_001/result.json` | diagnostic/evaluation failure short-circuit evidence |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_host_sensitive_reject_001/result.json` | host-sensitive residual rejection evidence |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/scenarios/tc_cycle_region_reject_001/result.json` | cycle reject no-publication evidence from the TraceCalc reference machine |

## 7. Assumptions And Limits

1. The Lean artifact states and checks theorem targets over a bounded semantic vocabulary; it does not prove the Rust `topological_formula_order` queue implementation line-by-line.
2. The TLA artifact is a bounded smoke model over a three-node graph shape, not full TLA verification.
3. The sample order deliberately matches the hand-auditable formula chain `X`, `Y`, `Z`; larger DAGs remain replay-covered but not mechanized in this bead.
4. Rebind soundness is consumed from `calc-gucd.3`; this bead does not reopen dynamic reference semantics or full `INDIRECT` behavior.
5. Tracker state transition detail is consumed from `calc-gucd.4`; this bead models the evaluation-level consequences.
6. The no-torn-candidate property is modeled at bundle shape level. Full coordinator atomic publication refinement remains in `calc-gucd.6`.
7. OxFml formula semantics, LET/LAMBDA carrier behavior, formatting/display deltas, and callable boundary behavior remain `calc-gucd.7`.
8. TraceCalc observable refinement remains the next bead.

## 8. Successor Obligations

| Successor bead | Consumes from this packet |
| --- | --- |
| `calc-gucd.6` | evaluation-order events, stable/prior read discipline, terminal decision vocabulary, and no-torn-candidate target for TraceCalc refinement |
| `calc-gucd.7` | read-value effect handler law and OxFml runtime call boundary constraints |
| `calc-gucd.8` | proof-service/evidence classifier rows mapped to concrete evaluation-order/read artifacts |
| `calc-gucd.9` | phase-timing signatures for order selection, read resolution, evaluation, diagnostic short-circuit, and candidate publication |
| `calc-gucd.11` | semantic-spine closure audit over graph, invalidation, recalc, order/read, refinement, and OxFml seam lanes |

## 9. Validation

| Command | Result |
| --- | --- |
| `lean formal\lean\OxCalc\CoreEngine\W046EvaluationOrderReadDiscipline.lean` | passed |
| `powershell -ExecutionPolicy Bypass -File scripts\run-tlc.ps1 formal\tla\CoreEngineW046EvaluationOrder.tla formal\tla\CoreEngineW046EvaluationOrder.smoke.cfg` | passed |
| `cargo test -p oxcalc-core local_treecalc_engine_` | passed; 9 tests passed, covering recalc chain, verified clean, cycle rejection, rebind rejection, missing target rejection, host-sensitive and dynamic runtime effects, dynamic resolved publication, and local formula publication |
| `powershell -ExecutionPolicy Bypass -File scripts\check-worksets.ps1` | passed |
| `br dep cycles --json` | passed with `count: 0` |
| JSON parse check for `w046-evaluation-order-001` summary/validation | passed |
| TLC log non-empty check for `w046-evaluation-order-001` | passed |
| `rg -n "\b(sorry|admit|axiom)\b" formal\lean\OxCalc\CoreEngine\W046EvaluationOrderReadDiscipline.lean` | passed; no matches |
| `git diff --check` | passed; emitted line-ending normalization warnings only |

## 10. Semantic-Equivalence Statement

This bead changes formal artifacts, evidence metadata, spec/status documents, and bead state only.

Observable OxCalc behavior is invariant under this bead. It does not change dependency graph construction, invalidation closure, soft-reference or dynamic-reference rebind behavior, recalc tracker behavior, evaluation order implementation, working-value reads, TraceCalc execution, TreeCalc/CoreEngine execution, OxFml/OxFunc behavior, rejection, publication, pack policy, or service readiness.

## 11. Reviewed Inbound Observations

Reviewed `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`.

Relevant intake for this bead:

1. OxFml remains authoritative for evaluator artifact meaning, reject semantics, replay-safe identity, and fence meaning.
2. Candidate and commit remain distinct artifact stages; reject remains no-publish.
3. OxFml now exposes stronger consumer/runtime, candidate, commit, reject, trace, capability-sensitive, execution-restriction, and topology/effect facts, but these do not transfer publication authority away from OxCalc.
4. Current host/runtime and stand-in fixture packet notes reinforce the W046 model choice: formula evaluation requests reads/effects from current working values, while graph, invalidation, rejection, and publication authority stay OxCalc-owned.
5. No new OxFml handoff is needed because this bead does not change shared FEC/F3E text or evaluator-facing clauses.

## 12. Pre-Closure Verification Checklist

| # | Check | Result |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W046 status surfaces, transition catalog, fragment ledger, and formal layout note the evaluation-order/read-discipline model |
| 2 | Pack expectations updated for affected packs? | yes; no pack expectation changed by this model bead |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; replay roots are listed in Section 6 and the TLA run artifact is checked in |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no policy/strategy change, invariant statement in Section 10 |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no normative FEC/F3E seam change and no OxFml handoff needed |
| 6 | All required tests pass? | yes; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for declared `calc-gucd.5` target; arbitrary finite topo proof, TraceCalc refinement, OxFml seam closure, full Rust proof, and unbounded TLA remain explicit successor or limit lanes |
| 8 | Completion language audit passed? | yes; no full Rust, full evaluation-engine, full TLA, TraceCalc-refinement, OxFml-seam, or release-grade proof is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; no ordered workset register change required |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; current semantic bead moved to `calc-gucd.6` |
| 11 | execution-state blocker surface updated? | yes; `.beads/` owns `calc-gucd.5` state |

## 13. Completion Claim Self-Audit

| Step | Result |
| --- | --- |
| Scope re-read | pass; bead asks for dependency-before-dependent, stable/prior read, diagnostic short-circuit, and no-torn-candidate targets, and this packet adds checked Lean/TLA artifacts plus replay roots for those targets |
| Gate criteria re-read | pass; model artifacts, checked commands, replay roots, exact assumptions, and successor limits are recorded |
| Silent scope reduction check | pass; arbitrary finite graph proof, Rust line proof, TraceCalc refinement, OxFml seam closure, and unbounded TLA verification are explicitly not hidden |
| "Looks done but is not" pattern check | pass; the packet says the model is checked and scoped, not a full mechanized semantic proof of all evaluation behavior |
| Include result | pass; checklist, self-audit, semantic-equivalence statement, and three-axis report are included |

## 14. Current Status

- execution_state: `calc-gucd.5_closed`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
