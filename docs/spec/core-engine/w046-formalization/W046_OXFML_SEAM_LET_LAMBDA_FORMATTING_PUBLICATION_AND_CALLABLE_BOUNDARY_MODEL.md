# W046 OxFml Seam LET LAMBDA Formatting Publication And Callable-Boundary Model

Status: `calc-gucd.7_oxfml_effect_boundary_validated`

Workset: `W046`

Parent epic: `calc-gucd`

Bead: `calc-gucd.7`

## 1. Purpose

This packet models the OxCalc/OxFml boundary as engine-phase effects and handlers.

The scoped goal is not a broad proof of OxFml or OxFunc. The scoped goal is to state the OxCalc-owned laws that make formula evaluation safe to consume inside the W046 semantic spine:

1. OxFml formula evaluation may request reads, reference resolution, runtime effects, diagnostics, candidate creation, typed rejection, formatting/display deltas, and narrow `LET`/`LAMBDA` carrier facts.
2. OxCalc-owned phases handle graph build, invalidation, rebind, recalc state, rejection, and publication consequences.
3. Formula evaluation does not directly mutate the published model.
4. Candidates remain separate from committed publication.
5. Reject remains no-publish.
6. `format_delta` and `display_delta` remain distinct seam categories, even where the current TreeCalc floor records them only as explicit absences.
7. Registered-external and callable-publication consequences remain watch/blocker rows unless direct evidence later makes them coordinator-visible.

## 2. Source Surfaces Reviewed

| Surface | Intake |
| --- | --- |
| `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md` | candidate/commit/reject split, Stage 1 local seam packet, W026 executed residual floor, public-entry read |
| `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md` | TraceCalc as selected-kernel reference machine and no-publish/candidate/publication oracle surface |
| W046 `.1-.6` packets | phase transition catalog, graph/invalidation/recalc/order/refinement laws consumed here |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | current consumer-runtime facade, public surface, W026 residual notes, registered-external note-level convergence |
| `../OxFml/docs/spec/OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md` | ordinary consumption through `consumer::runtime`, `consumer::editor`, `consumer::replay`; `RuntimeFormulaResult` candidate/commit/reject/fact families |
| `../OxFml/docs/spec/OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md` | host/coordinator split, required inputs/outputs, candidate/commit/reject rules, LET/LAMBDA current floor |
| `../OxFml/docs/spec/OXFML_PUBLIC_API_AND_RUNTIME_SERVICE_SKETCH.md` | transform chain and runtime facade shape |
| `../OxFml/docs/spec/formula-language/OXFML_REGISTERED_EXTERNAL_PROVIDER_AND_CALL_REGISTER_ID_BOUNDARY.md` | registered-external callable packet convergence and current non-claims |
| `src/oxcalc-core/src/treecalc.rs` | formula preparation, descriptor lowering, OxFml-backed evaluation, runtime effects, candidate/reject/publication handoff |

## 3. Implementation Crosswalk

| Semantic object or transition | Rust surface | Model surface | Evidence root |
| --- | --- | --- | --- |
| formula source and bind context | `prepare_oxfml_formula`, `FormulaSourceRecord`, `BindContext` | Lean `Phase.prepareFormula`; TLA `prepare_formula` | TreeCalc local result diagnostics with `oxfml_*` ids |
| static reference lowering | `translate_reference`, `oxfml_dependency_descriptors` | `ResolveStatic`, `EmitDependency` | `tc_local_relative_sum_001/dependency_graph.json`; W046 graph model |
| dynamic and residual carriers | `ResidualCarrierKind`, `residual_runtime_effect`, `evaluate_via_oxfml` residual rejection | `ResolveDynamic`, `SurfaceRuntimeFact`, `RejectCandidate` | `tc_local_dynamic_reject_001`, `tc_local_dynamic_resolved_publish_001` |
| working-value reads | `build_upstream_host_packet`, `evaluate_via_oxfml(prepared, &working_values)` | `ReadValue` | W046 evaluation-order/read-discipline model |
| OxFml runtime facade intake | `RuntimeFormulaResult`, candidate diagnostics, commit diagnostics, reject diagnostics | `candidateResult`, `commitBundle`, `rejectRecord` boundary facts | `tc_local_publish_001`, `tc_local_let_lambda_capture_publish_001` |
| commit-bundle validation | `validate_oxfml_commit_bundle` | candidate/commit correlation law | local candidate diagnostics and publication bundles |
| candidate adaptation | `LocalEvaluatorCandidate`, `adapt_local_candidate` | `ProduceCandidate`; candidate is not publication | W046 recalc and TraceCalc refinement packets |
| coordinator rejection | `reject_run`, `map_local_error_to_reject_kind`, `reject_candidate_work` | `RejectCandidate`; reject no-publish | dynamic, host-sensitive, rebind, cycle reject roots |
| coordinator publication | `accept_and_publish`, `PublicationBundle` | `Publish` only in publication phase | local publish roots |
| formatting/display boundary | publication-bundle `explicit_current_absence_categories` | `EmitFormatDelta`, `EmitDisplayDelta`, distinct boundary facts | `tc_local_publish_001/result.json` explicit current absence; positive projection remains blocker |
| LET/LAMBDA carrier | raw OxFml formula path and TraceCalc/TreeCalc LET/LAMBDA rows | `BindLocal`, `EnterLambda`, narrow carrier fact | TraceCalc and TreeCalc LET/LAMBDA roots |
| registered-external callable boundary | OxFml note-level packet; no OxCalc production path yet | `registeredExternalPacket` watch row | exact blocker in W046 binding register |

## 4. Effect Signature

Effect names are specification handles, not Rust API commitments.

| Effect | Handler authority | Phase binding | First law |
| --- | --- | --- | --- |
| `ReadValue(reference, context)` | recalc/evaluation handler | `T12.ReadWorkingValue`, `T13.EvaluateFormula` | stable or prior reads only |
| `ResolveStatic(reference, context)` | prepare/dependency handler | `T01.PrepareFormula`, `T02.LowerDescriptors` | deterministic descriptor lowering |
| `ResolveDynamic(owner, text, context)` | rebind/dynamic handler | `T06.SeedInvalidation`, `T10.RebindGate`, `T13.EvaluateFormula` | dynamic changes route through rebind/reject before publication |
| `EmitDependency(owner, target, kind)` | graph handler | `T02`, `T03`, `T04` | graph/reverse-edge converse remains OxCalc-owned |
| `EmitDiagnostic(owner, kind)` | prepare/graph/evaluation/reject handlers | `T01`, `T03`, `T13`, `T16` | diagnostics are evidence or reject detail, not publication |
| `CallFunction(function_id, args)` | OxFml/OxFunc evaluator boundary | `T13` | general OxFunc kernels remain external |
| `BindLocal(name, value)` | narrow LET carrier | `T13` | LET carrier visible only where engine state/trace/publication cares |
| `EnterLambda(params, body, closure_env)` | narrow LAMBDA carrier | `T13` | LAMBDA carrier identity is not broad callable-publication closure |
| `ProduceCandidate(owner, value, diagnostics, effects)` | candidate handler | `T15` | candidate is not publication |
| `RejectCandidate(owner, reason)` | rejection handler | `T10`, `T16` | reject preserves published view |
| `Publish(bundle)` | coordinator publication handler | `T17` | single OxCalc publisher, atomic bundle |
| `EmitFormatDelta(owner, detail)` | evaluator fact, coordinator-carried if present | `T13`, `T15`, `T17` | distinct from display delta |
| `EmitDisplayDelta(owner, detail)` | evaluator fact, coordinator-carried if present | `T13`, `T15`, `T17` | distinct from format delta |
| `SurfaceRuntimeFact(owner, family)` | evaluator/runtime fact surfaced to OxCalc | `T13`, `T18` | capability/execution-restriction/topology facts are not scheduler-inferred policy |

## 5. Phase Authority Table For `calc-gucd.15`

| Phase | Allowed effect families | Forbidden promotion |
| --- | --- | --- |
| `prepare_formula` | `ResolveStatic`, `EmitDiagnostic`, narrow `BindLocal`/`EnterLambda` carrier discovery | no graph mutation, no candidate, no publication |
| `lower_descriptors` | `EmitDependency`, `EmitDiagnostic` | no value read, no publication |
| `graph_build` | `EmitDependency`, `EmitDiagnostic` | no candidate, no publication |
| `rebind_gate` | `ResolveDynamic`, dependency-shape effect, `RejectCandidate` | no stale-binding publication |
| `evaluate_formula` | `ReadValue`, `CallFunction`, `BindLocal`, `EnterLambda`, `ProduceCandidate`, `RejectCandidate`, `EmitDiagnostic`, `EmitFormatDelta`, `EmitDisplayDelta`, `SurfaceRuntimeFact` | no direct published-model mutation |
| `publication` | `Publish`, final no-publish rejection | no evaluator-owned publication bypass |
| `trace_projection` | replay/trace runtime facts | no semantic reinterpretation of lane-native artifacts |

## 6. Handler Laws

| Law | Statement | Current evidence |
| --- | --- | --- |
| evaluator purity boundary | OxFml/OxFunc evaluation requests effects and returns candidate/reject facts; it does not mutate OxCalc publication state directly | Lean/TLA effect boundary; `treecalc.rs` routes through coordinator |
| candidate/publication separation | `AcceptedCandidateResult` and local candidates precede `PublicationBundle`; coordinator commit is separate | W046 recalc/refinement packets; local candidate/publication roots |
| reject no-publish | reject paths preserve the previous published view and produce typed reject detail | W046 recalc/refinement packets; dynamic/host/rebind roots |
| rebind before publish | dynamic/reference-shape changes that require rebind reject before publication | W046 invalidation/rebind packet |
| stable/prior reads | formula evaluation receives current `working_values`, which are seeded published values plus prior ordered computed values | W046 evaluation-order/read-discipline packet |
| LET/LAMBDA narrow carrier | `LET`/`LAMBDA` facts are visible only where they affect OxCalc state, traces, dependencies, rejection, or publication; general OxFunc kernels remain external | TraceCalc and TreeCalc LET/LAMBDA roots; Lean carrier law |
| format/display separation | `format_delta` and `display_delta` are distinct categories; current TreeCalc local floor records positive deltas as explicit absences | local publication roots and exact blocker |
| registered-external watch | registered-external request/descriptor/call packets are converged at note level, but publication/topology consequences are not promoted by W046.7 | OxFml registered-external note; W046 exact blocker |

## 7. Checked Lean Artifact

Artifact: `formal/lean/OxCalc/CoreEngine/W046OxfmlEffectBoundary.lean`

Checked command:

```powershell
lean formal\lean\OxCalc\CoreEngine\W046OxfmlEffectBoundary.lean
```

Result: passed.

Lean definitions:

1. `Phase`: prepare, lower-descriptor, graph-build, rebind, evaluate, publication, and trace-projection phases.
2. `Effect`: read, static/dynamic resolution, dependency/diagnostic emission, function call, LET binding, LAMBDA carrier entry, candidate, reject, publish, format/display delta, and runtime fact effects.
3. `BoundaryFact`: candidate, commit, reject, runtime-effect, format/display, LET/LAMBDA, and registered-external facts.
4. `EffectAllowed`: phase-authority relation.
5. `SeamRun`: bounded run envelope carrying effects, boundary facts, publication state, and opacity of general OxFunc kernels.
6. `HandlerLawModel`: conjunction of no-direct-publication, reject no-publish, candidate-not-publication, LET/LAMBDA narrow carrier, and format/display separation.

Checked theorem targets:

| Theorem | Contract |
| --- | --- |
| `sample_formula_no_direct_publish` | a formula-evaluation LET/LAMBDA candidate run has no publish effect and no publication change |
| `sample_formula_candidate_not_publication` | candidate production in formula evaluation is not publication |
| `sample_formula_let_lambda_narrow` | LET/LAMBDA carrier facts imply general OxFunc kernels stay opaque to OxCalc |
| `sample_reject_preserves_publication` | runtime-effect rejection preserves publication |
| `sample_publication_allows_publish` | publish is admitted only in the publication-phase sample |
| `sample_format_display_distinct` | format and display boundary facts are distinct |
| `sample_handler_law_model` | the sample formula run satisfies the handler-law model |

## 8. Checked TLA Artifact

Artifacts:

1. `formal/tla/CoreEngineW046OxfmlEffectBoundary.tla`
2. `formal/tla/CoreEngineW046OxfmlEffectBoundary.smoke.cfg`
3. `docs/test-runs/core-engine/tla/w046-oxfml-effect-boundary-001/`

Checked command:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run-tlc.ps1 formal\tla\CoreEngineW046OxfmlEffectBoundary.tla formal\tla\CoreEngineW046OxfmlEffectBoundary.smoke.cfg
```

Result: passed.

TLC summary:

| Field | Value |
| --- | --- |
| TLC version | `2.19 of 08 August 2024` |
| states generated | `13` |
| distinct states | `7` |
| queue left | `0` |
| complete-state depth | `2` |
| result | no error found |

Checked invariants:

1. `TypeInvariant`
2. `PhaseAuthority`
3. `NoDirectFormulaPublication`
4. `RejectPreservesPublication`
5. `CandidateIsNotPublication`
6. `PublicationOnlyByCoordinator`
7. `LetLambdaCarrierKeepsOxFuncOpaque`
8. `FormatDisplayBoundaryDistinct`
9. `RuntimeEffectIsNotPublication`
10. `WatchRowsHaveExactBlockers`

Smoke model shapes:

1. LET/LAMBDA formula candidate without publication,
2. formula runtime-effect rejection,
3. format/display candidate with distinct boundary facts,
4. dynamic rebind rejection,
5. coordinator publication,
6. registered-external watch row with an exact blocker.

## 9. Replay And Binding Roots

Canonical binding root: `docs/test-runs/core-engine/refinement/w046-oxfml-effect-boundary-001/`

Checked-in artifacts:

1. `run_summary.json`
2. `effect_boundary_binding_register.json`

Binding summary:

| Metric | Value |
| --- | --- |
| rows | `8` |
| existing evidence rows | `7` |
| checked model rows | `1` |
| exact blockers | `3` |
| unexpected mismatches | `0` |
| missing referenced artifacts | `0` |

Replay/evidence roots:

| Root | Use |
| --- | --- |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/scenarios/tc_let_lambda_carrier_publish_001/result.json` | TraceCalc LET/LAMBDA carrier publication row |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_let_lambda_capture_publish_001/result.json` | TreeCalc LET/LAMBDA value publication through OxFml candidate and OxCalc publication |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/scenarios/tc_let_lambda_invocation_reject_001/result.json` | LET/LAMBDA invocation-contract reject/no-publish row |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_lambda_host_sensitive_reject_001/result.json` | LAMBDA-adjacent host-sensitive runtime-effect rejection |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_dynamic_resolved_publish_001/result.json` | dynamic dependency resolved publication and dependency-shape update |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_dynamic_reject_001/result.json` | dynamic unresolved runtime effect and no-publish rejection |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_publish_001/result.json` | explicit current absences for shape/topology/format/display deltas in publication bundle |
| `docs/test-runs/core-engine/tla/w046-oxfml-effect-boundary-001/run_summary.json` | checked phase-authority and handler-law smoke model |

## 10. Exact Blockers And Limits

| Blocker | Meaning | Successor route |
| --- | --- | --- |
| `format_display_delta_positive_projection_missing` | Current TreeCalc local evidence records `format_delta` and `display_delta` as explicit absences; W046.7 models distinct authority but does not prove positive format/display publication projection. | later OxFml formatting/display projection or TreeCalc host evidence |
| `registered_external_publication_consequence_breadth_not_frozen` | Registered-external packets are converged at note level and remain callable-boundary watch rows; publication/topology consequences are not promoted. | W046.15 carries watch row; later handoff only on concrete coordinator-visible mismatch |
| `broad_general_oxfunc_kernel_excluded` | LET/LAMBDA carrier facts are in scope only where they affect OxCalc state, trace, dependency, rejection, or publication; general OxFunc function semantics remain external. | OxFunc/OxFml-owned semantic work, not OxCalc W046 |

Additional limits:

1. This packet does not prove Rust `treecalc.rs` line-by-line.
2. This packet does not prove full OxFml evaluator semantics.
3. This packet does not claim all callable-publication or registered-external outcomes are settled.
4. This packet does not widen W046 release, C5, pack-grade replay, Stage 2, operated-service, or scale readiness.

## 11. Reviewed Inbound Observations

Reviewed `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`.

Relevant intake for this bead:

1. Ordinary OxFml downstream use should target `consumer::runtime`, `consumer::editor`, and `consumer::replay`; public `substrate::...` access is not ordinary integration contract.
2. `RuntimeFormulaResult` preserves candidate, commit, reject, trace, runtime-effect, capability-sensitive, topology/effect, and dependency-sensitive facts in consumer form.
3. Candidate and commit remain distinct artifact stages; reject remains no-publish.
4. `format_delta` and `display_delta` remain distinct categories.
5. Host/runtime, stand-in packet, table-context, immutable-edit, and registered-external notes are converged enough for first-slice planning but are not broad coordinator API freeze.
6. Registered-external direct packet names and the seven-field descriptor are converged at note level; bind-visible register/unregister implies snapshot generation plus bind invalidation, while `CALL`/`REGISTER.ID`-only mutation may remain targeted reevaluation by default.
7. Provider-failure, callable-publication, broader publication/topology breadth, execution-restriction transport shape, and caller-anchor/address-mode breadth remain watch or narrower lanes.

## 12. Handoff Assessment

No new OxFml handoff is filed by this bead.

Reason:

1. The packet states OxCalc-local phase-authority and handler-law modeling over already consumed seam facts.
2. It does not propose new canonical FEC/F3E or OxFml evaluator-facing text.
3. Registered-external and callable-publication rows are explicitly retained as watch/blocker rows rather than promoted.
4. Positive format/display projection remains blocked by missing current OxCalc evidence rather than silently requiring shared seam text changes.

## 13. Successor Obligations

| Successor bead | Consumes from this packet |
| --- | --- |
| `calc-gucd.15` | phase-authority table, effect signature, handler laws, exact blockers, and binding roots for the integrated semantic kernel |
| `calc-gucd.16` | no direct dependency unless finite proof strengthening adds formula-effect rows to graph/order rules |
| `calc-gucd.17` | effect facts, candidate/reject/publication boundary facts, and exact-blocker vocabulary for proof-carrying traces |
| `calc-gucd.18` | Rust-to-semantic mapping for `prepare_oxfml_formula`, `evaluate_via_oxfml`, runtime effects, candidates, rejects, and publications |
| `calc-gucd.8` | evidence classifier recast over direct seam laws and blockers, not readiness rows |
| `calc-gucd.10` | downstream consequence reassessment with registered-external/callable/format-display rows still unpromoted unless direct evidence appears |

## 14. Validation

| Command | Result |
| --- | --- |
| `lean formal\lean\OxCalc\CoreEngine\W046OxfmlEffectBoundary.lean` | passed |
| `powershell -ExecutionPolicy Bypass -File scripts\run-tlc.ps1 formal\tla\CoreEngineW046OxfmlEffectBoundary.tla formal\tla\CoreEngineW046OxfmlEffectBoundary.smoke.cfg` | passed |
| JSON parse check for `w046-oxfml-effect-boundary-001` TLA and binding roots | passed |
| `rg -n "\b(sorry|admit|axiom)\b" formal\lean\OxCalc\CoreEngine\W046OxfmlEffectBoundary.lean` | passed; no matches |
| `cargo test -p oxcalc-core local_treecalc_engine_` | passed; 9 tests passed |

## 15. Semantic-Equivalence Statement

This bead changes formal artifacts, evidence metadata, spec/status documents, and bead state only.

Observable OxCalc behavior is invariant under this bead. It does not change formula parsing/binding, dependency descriptor lowering, dependency graph construction, invalidation closure, dynamic-reference handling, recalc tracker behavior, evaluation order, working-value reads, OxFml/OxFunc runtime behavior, candidate adaptation, rejection, publication, pack policy, proof-service policy, performance behavior, or service readiness.

## 16. Pre-Closure Verification Checklist

| # | Check | Result |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W046 README/status surfaces, Lean/TLA artifacts, and binding register record the seam model |
| 2 | Pack expectations updated for affected packs? | yes; no pack expectation changed by this model/binding bead |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; Section 9 lists existing replay roots plus the checked W046 TLA root |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no policy/strategy change, invariant statement in Section 15 |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no normative FEC/F3E change and no OxFml handoff needed |
| 6 | All required tests pass? | yes; see Section 14 |
| 7 | No known semantic gaps remain in declared scope? | yes for the declared `calc-gucd.7` model target; positive format/display projection, registered-external publication breadth, and broad OxFunc kernels remain explicit blockers/limits |
| 8 | Completion language audit passed? | yes; no full OxFml proof, full callable closure, full Rust proof, full TLA, or release-grade proof is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; no ordered workset register change required |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; current semantic bead moved to `calc-gucd.15` |
| 11 | execution-state blocker surface updated? | yes; `.beads/` owns `calc-gucd.7` state |

## 17. Completion Claim Self-Audit

| Step | Result |
| --- | --- |
| Scope re-read | pass; bead asks for OxFml effects and handler-law model, LET/LAMBDA carrier, formatting/display/publication, callable-boundary model, inbound-observation intake, replay roots, and phase-authority table |
| Gate criteria re-read | pass; model artifacts, checked commands, binding roots, current intake, exact blockers, and phase-authority table are recorded |
| Silent scope reduction check | pass; broad OxFml proof, broad OxFunc semantics, positive format/display projection, registered-external publication breadth, and callable-publication closure are explicitly not hidden |
| "Looks done but is not" pattern check | pass; the packet says the model is checked and scoped, not a full mechanized proof of all formula/evaluator or callable behavior |
| Include result | pass; checklist, self-audit, semantic-equivalence statement, reviewed inbound observations, and three-axis report are included |

## 18. Current Status

- execution_state: `calc-gucd.7_closed`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
