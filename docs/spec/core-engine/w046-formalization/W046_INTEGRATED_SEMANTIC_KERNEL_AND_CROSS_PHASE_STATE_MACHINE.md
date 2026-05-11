# W046 Integrated Semantic Kernel And Cross-Phase State Machine

Status: `calc-gucd.15_integrated_kernel_validated`

Workset: `W046`

Parent epic: `calc-gucd`

Bead: `calc-gucd.15`

## 1. Purpose

This packet connects the W046 phase-local models into one cross-phase semantic kernel.

The target is not another readiness classifier. The target is an integrated engine state machine that threads the semantic facts from:

1. formula preparation and descriptor lowering,
2. graph build, reverse-edge converse, diagnostics, and SCC/cycle facts,
3. invalidation closure and rebind requirements,
4. recalc tracker dirty/needed/evaluating/candidate/reject/verified-clean states,
5. evaluation order and stable/prior working-value reads,
6. OxFml effect-boundary laws,
7. TraceCalc observable refinement,
8. rejection, publication, and trace emission.

This packet composes the prior W046 `.2-.7` artifacts. It does not claim a line-by-line Rust proof, arbitrary finite graph proof, proof-carrying trace validator, or full Rust-to-kernel refinement bridge.

## 2. Source Surfaces Reviewed

| Surface | Intake |
| --- | --- |
| `W046_ENGINE_STATE_TRANSITION_CATALOG.md` | state vocabulary and transitions `T01` through `T18` |
| `W046_DEPENDENCY_GRAPH_REVERSE_EDGE_AND_SCC_MODEL.md` | `graphBuilt`, `reverseConverse`, `diagnosticsPreserved` facts |
| `W046_INVALIDATION_SOFT_REFERENCE_DYNAMIC_REFERENCE_AND_REBIND_MODEL.md` | `invalidationClosed`, `noUnderInvalidation`, `rebindRequired`, rebind no-publish |
| `W046_RECALC_TRACKER_TRANSITION_PRE_POST_MODEL.md` | dirty/needed, candidate-not-publication, reject no-publish, verified-clean no-publication |
| `W046_EVALUATION_ORDER_AND_WORKING_VALUE_READ_DISCIPLINE_MODEL.md` | `orderSelected`, `stablePriorReads`, no future reads |
| `W046_TRACECALC_REFINEMENT_KERNEL_AND_REPLAY_BINDING.md` | observable refinement and selected-kernel exact blockers |
| `W046_OXFML_SEAM_LET_LAMBDA_FORMATTING_PUBLICATION_AND_CALLABLE_BOUNDARY_MODEL.md` | effect-boundary phase authority and no direct publication from OxFml evaluation |
| `src/oxcalc-core/src/dependency.rs` | graph build, reverse edges, invalidation closure |
| `src/oxcalc-core/src/recalc.rs` | Stage 1 recalc tracker transitions and overlays |
| `src/oxcalc-core/src/coordinator.rs` | candidate admission, accepted candidate, reject, publication, pinned view |
| `src/oxcalc-core/src/treecalc.rs` | actual local sequential phase order and OxFml runtime adaptation |
| `src/oxcalc-tracecalc/src/machine.rs` | TraceCalc transition sequence for candidate/reject/publish/verify/trace |
| `src/oxcalc-tracecalc/src/planner.rs` | TraceCalc planning, reverse closure, SCC/order substrate |
| `src/oxcalc-tracecalc/src/runner.rs` | emitted artifact and replay projection surface |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | candidate/commit/reject, runtime-effect, consumer facade, and watch-lane intake |

## 3. Integrated State Vocabulary

| Integrated fact | Source packet | Meaning in the cross-phase kernel |
| --- | --- | --- |
| `prepared` | `.7` | formula source/bind context exists; OxFml preparation cannot publish |
| `descriptorsLowered` | `.2`, `.7` | dependency descriptors are lowered from prepared formula/reference/effect facts |
| `graphBuilt` | `.2` | dependency graph build has run over descriptors |
| `reverseConverse` | `.2` | every forward edge has a matching reverse edge |
| `diagnosticsPreserved` | `.2` | unresolved/invalid descriptors produce diagnostics, not silent drops |
| `invalidationClosed` | `.3` | invalidation seed and reverse-reachability closure has run |
| `noUnderInvalidation` | `.3` | seed nodes and reverse-reachable dependents are covered |
| `rebindRequired` | `.3`, `.7` | a stale binding/dynamic shape condition blocks publication |
| `dirtyNeeded` | `.4` | recalc tracker has marked dirty/needed work before evaluation |
| `orderSelected` | `.5` | acyclic work has selected dependency-before-dependent order |
| `stablePriorReads` | `.5`, `.7` | evaluation reads only stable published or prior ordered computed values |
| `candidateProduced` | `.4`, `.6`, `.7` | accepted/local candidate exists but is not publication |
| `oxfmlNoDirectPublish` | `.7` | formula evaluation effects cannot mutate OxCalc publication directly |
| `traceCalcObservableRefinement` | `.6` | selected rows match/refine TraceCalc observable packet or are exact blockers |
| terminal decision | `.4`, `.6`, `.7` | one of verified-clean, rejected, or published |
| `traceEmitted` | `.6`, `.17 later` | trace/evidence emission occurs only after a terminal decision in this kernel |

## 4. Cross-Phase Transition Order

The integrated model constrains the phase order to:

1. `T01.PrepareFormula`
2. `T02.LowerDescriptors`
3. `T03-T05.Graph`
4. `T06-T07.CloseInvalidation`
5. `T08.MarkDirtyNeeded`
6. either:
   - `T10.RebindGateReject`, or
   - `T09.SelectEvaluationOrder` -> `T11-T13.BeginEvaluateRead` -> terminal path
7. terminal paths:
   - `T14.VerifyClean`,
   - `T16.RejectCandidate`,
   - `T15.ProduceCandidate` -> `T17.PublishCandidate`
8. `T18.EmitTraceAndEvidence`

## 5. Integrated Invariants

| Invariant | Statement | Consumed source |
| --- | --- | --- |
| graph before invalidation | invalidation closure cannot be considered valid until graph facts and reverse converse exist | `.2`, `.3` |
| dirty/needed after invalidation | dirty/needed tracker work follows invalidation closure and no-under-invalidation | `.3`, `.4` |
| order after dirty and no rebind | evaluation order is selected only after dirty/needed and only when no rebind-required gate blocks it | `.3`, `.5` |
| evaluation reads after order | evaluation reads require selected order and stable/prior read discipline | `.5`, `.7` |
| candidate after evaluation | candidate production requires evaluation, stable/prior reads, and OxFml no-direct-publication law | `.4`, `.5`, `.7` |
| published requires integrated spine | publication requires prepared formula, descriptor lowering, graph facts, invalidation closure, dirty/needed, selected order, stable/prior reads, candidate, OxFml no-direct-publication, and TraceCalc refinement | `.2-.7` |
| reject no-publish | rejected work does not publish | `.4`, `.6`, `.7` |
| verified-clean no-publish | verified-clean terminal state does not produce a candidate or publication | `.4`, `.5`, `.6` |
| rebind no-publish | rebind-required state cannot publish | `.3`, `.7` |
| trace after terminal | trace emission occurs after publish, reject, or verified-clean in this kernel | `.6`, `.17 later` |

## 6. Checked Lean Artifact

Artifact: `formal/lean/OxCalc/CoreEngine/W046IntegratedSemanticKernel.lean`

Checked command:

```powershell
lean formal\lean\OxCalc\CoreEngine\W046IntegratedSemanticKernel.lean
```

Result: passed.

Lean definitions:

1. `KernelState`: integrated boolean/fact state over graph, invalidation, recalc, order/read, OxFml, TraceCalc, terminal decision, and trace emission.
2. `IntegratedKernelInvariant`: publication/rejection/verified-clean/rebind/trace cross-phase law bundle.
3. `KernelStep`, `StepPre`, `StepPost`, and `LegalStep`: cross-phase transition vocabulary for `T01` through `T18`.
4. `SamplePublishedState`, `SampleRebindRejectState`, and `SampleVerifiedCleanState`: representative terminal states.

Checked theorem targets:

| Theorem | Contract |
| --- | --- |
| `sample_published_integrated_kernel` | sample publication state satisfies integrated kernel invariant |
| `published_requires_graph_and_invalidation` | publication implies graph build, reverse converse, invalidation closure, and no-under-invalidation |
| `published_requires_order_reads_candidate_and_refinement` | publication implies order, stable/prior reads, candidate, and TraceCalc refinement |
| `published_requires_oxfml_no_direct_publish` | publication implies the OxFml no-direct-publication law was consumed |
| `rebind_required_blocks_publication` | rebind-required state cannot publish |
| `trace_emitted_after_terminal` | trace emission implies terminal decision |
| `sample_rebind_reject_integrated_kernel` | rebind reject sample satisfies integrated kernel invariant |
| `sample_verified_clean_integrated_kernel` | verified-clean sample satisfies integrated kernel invariant |

## 7. Checked TLA Artifact

Artifacts:

1. `formal/tla/CoreEngineW046IntegratedKernel.tla`
2. `formal/tla/CoreEngineW046IntegratedKernel.smoke.cfg`
3. `docs/test-runs/core-engine/tla/w046-integrated-kernel-001/`

Checked command:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run-tlc.ps1 formal\tla\CoreEngineW046IntegratedKernel.tla formal\tla\CoreEngineW046IntegratedKernel.smoke.cfg
```

Result: passed.

TLC summary:

| Field | Value |
| --- | --- |
| TLC version | `2.19 of 08 August 2024` |
| states generated | `23` |
| distinct states | `19` |
| queue left | `0` |
| complete-state depth | `11` |
| result | no error found |

Checked invariants:

1. `TypeInvariant`
2. `GraphFactsBeforeInvalidation`
3. `DirtyNeededAfterInvalidation`
4. `OrderAfterDirtyNoRebind`
5. `EvaluationReadsAfterOrder`
6. `CandidateAfterEvaluation`
7. `PublishedRequiresIntegratedSpine`
8. `RejectNoPublish`
9. `VerifiedCleanNoPublish`
10. `RebindNoPublish`
11. `TraceAfterTerminal`
12. `NoPrematureTrace`

Model paths:

1. normal publish path through graph -> invalidation -> order -> evaluation -> candidate -> publication -> trace,
2. rebind-required path through invalidation -> dirty/needed -> rejection,
3. formula reject path during evaluation,
4. verified-clean path during evaluation.

## 8. Binding Root

Canonical binding root: `docs/test-runs/core-engine/refinement/w046-integrated-kernel-001/`

Checked-in artifacts:

1. `run_summary.json`
2. `integrated_kernel_binding_register.json`

Binding summary:

| Metric | Value |
| --- | --- |
| phase packets | `7` |
| integrated invariants | `4` |
| exact blockers | `6` |
| unexpected mismatches | `0` |
| missing referenced artifacts | `0` |

## 9. Exact Blockers And Limits

| Blocker | Meaning | Successor route |
| --- | --- | --- |
| `phase_local_models_not_line_by_line_rust_proofs` | The integrated kernel composes W046 phase-local semantic models; it does not prove each Rust function line-by-line. | `calc-gucd.18` for Rust refinement bridge |
| `dynamic_dependency_projection_gap_inherited_from_calc_gucd_6` | Full dynamic dependency projection remains blocked where TraceCalc-to-TreeCalc comparable facts are missing. | `calc-gucd.17`/`.18` checker and bridge work |
| `format_display_positive_projection_missing_inherited_from_calc_gucd_7` | Integrated kernel carries format/display separation and current absences, not positive projection. | later formatting/display evidence or handoff if concrete mismatch appears |
| `unbounded_finite_graph_proof_strengthening_deferred_to_calc_gucd_16` | Current integrated model is bounded and phase-compositional, not arbitrary finite graph proof. | `calc-gucd.16` |
| `proof_carrying_trace_validation_deferred_to_calc_gucd_17` | Kernel names trace facts but does not validate emitted traces independently. | `calc-gucd.17` |
| `rust_refinement_bridge_deferred_to_calc_gucd_18` | Real Rust artifacts are not yet mapped into the kernel by an implementation checker. | `calc-gucd.18` |

## 10. Reviewed Inbound Observations

Reviewed `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`.

Relevant intake for this bead:

1. Candidate and commit are distinct artifact stages; reject remains no-publish.
2. OxFml remains authoritative for evaluator artifact meaning, typed reject outcomes, fence meaning, replay-safe identity, and consumer runtime facade facts.
3. OxCalc retains graph, invalidation, scheduler, rejection integration, and publication authority above OxFml runtime results.
4. Execution-restriction, capability-sensitive, topology/effect, dependency-sensitive, and formatting/display fact families are consumed as surfaced evaluator/runtime facts when present, not inferred scheduler policy.
5. Provider-failure, callable-publication, registered-external publication breadth, and broader publication/topology consequence breadth remain watch or blocker lanes rather than promoted integrated-kernel facts.

## 11. Handoff Assessment

No new OxFml handoff is filed by this bead.

Reason:

1. This packet composes existing OxCalc-local W046 models and consumed OxFml note-level/current-contract facts.
2. It does not propose new canonical FEC/F3E or OxFml evaluator-facing text.
3. Watch lanes from `.7` remain blockers and are not promoted by the integrated kernel.

## 12. Successor Obligations

| Successor bead | Consumes from this packet |
| --- | --- |
| `calc-gucd.16` | integrated graph/invalidation/order/read invariants and explicit arbitrary-finite proof strengthening blocker |
| `calc-gucd.17` | terminal trace law, integrated transition vocabulary, and proof-carrying trace blocker |
| `calc-gucd.18` | `KernelState` vocabulary and Rust refinement blocker for real artifacts |
| `calc-gucd.8` | direct semantic-spine coverage rows rather than readiness taxonomy |
| `calc-gucd.9` | phase timing signatures mapped to integrated transition names |
| `calc-gucd.10` | downstream consequence reassessment using integrated direct evidence and exact blockers |
| `calc-gucd.11` | closure audit over semantic-spine coverage and successor routing |

## 13. Validation

| Command | Result |
| --- | --- |
| `lean formal\lean\OxCalc\CoreEngine\W046IntegratedSemanticKernel.lean` | passed |
| `powershell -ExecutionPolicy Bypass -File scripts\run-tlc.ps1 formal\tla\CoreEngineW046IntegratedKernel.tla formal\tla\CoreEngineW046IntegratedKernel.smoke.cfg` | passed |
| JSON parse check for `w046-integrated-kernel-001` TLA and binding roots | passed |
| `rg -n "\b(sorry|admit|axiom)\b" formal\lean\OxCalc\CoreEngine\W046IntegratedSemanticKernel.lean` | passed; no matches |

## 14. Semantic-Equivalence Statement

This bead changes formal artifacts, evidence metadata, spec/status documents, and bead state only.

Observable OxCalc behavior is invariant under this bead. It does not change formula preparation, dependency graph construction, invalidation closure, recalc tracker behavior, evaluation order, working-value reads, OxFml/OxFunc runtime behavior, TraceCalc execution, TreeCalc/CoreEngine execution, rejection, publication, pack policy, proof-service policy, performance behavior, or service readiness.

## 15. Pre-Closure Verification Checklist

| # | Check | Result |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W046 README/status surfaces, Lean/TLA artifacts, and binding register record the integrated kernel |
| 2 | Pack expectations updated for affected packs? | yes; no pack expectation changed by this model/binding bead |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; prior `.2-.7` roots are bound in Section 8 and the new TLA root is checked in |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no policy/strategy change, invariant statement in Section 14 |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no normative FEC/F3E change and no OxFml handoff needed |
| 6 | All required tests pass? | yes; see Section 13 |
| 7 | No known semantic gaps remain in declared scope? | yes for the declared `calc-gucd.15` model target; finite proof strengthening, proof-carrying trace validation, and Rust refinement bridge remain explicit successor blockers |
| 8 | Completion language audit passed? | yes; no full Rust proof, arbitrary finite graph proof, proof-carrying trace checker, Rust refinement bridge, full TLA, or release-grade proof is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; no ordered workset register change required |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; current semantic bead moved to `calc-gucd.16` |
| 11 | execution-state blocker surface updated? | yes; `.beads/` owns `calc-gucd.15` state |

## 16. Completion Claim Self-Audit

| Step | Result |
| --- | --- |
| Scope re-read | pass; bead asks for an integrated semantic kernel over W046 graph, invalidation/rebind, recalc, evaluation-order/read, OxFml-effect, and TraceCalc-refinement slices |
| Gate criteria re-read | pass; integrated packet, Lean/TLA artifacts, evidence root, blocker register, inbound-observation intake, and successor routes are recorded |
| Silent scope reduction check | pass; finite proof strengthening, proof-carrying trace checker, and Rust refinement bridge are explicitly successor-scoped rather than hidden |
| "Looks done but is not" pattern check | pass; the packet says the model is checked and scoped, not a full mechanized proof of all engine behavior |
| Include result | pass; checklist, self-audit, semantic-equivalence statement, reviewed inbound observations, and three-axis report are included |

## 17. Current Status

- execution_state: `calc-gucd.15_closed`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
