# W046 Invalidation Soft-Reference Dynamic-Reference And Rebind Model

Status: `calc-gucd.3_invalidation_rebind_model_validated`

Workset: `W046`

Parent epic: `calc-gucd`

Bead: `calc-gucd.3`

## 1. Purpose

This packet models the invalidation/rebind slice that follows graph construction.

It covers:

1. invalidation seeds and reverse-reachability closure,
2. rebind-required reason classification,
3. dependency-shape transition seeds for dynamic/soft references,
4. stale-binding rejection before publication,
5. replay roots for dynamic `INDIRECT`-style residuals, dynamic resolved references, and soft-reference rebind pressure.

The packet is scoped to W046 semantic proof-spine targets. It does not claim full dynamic-reference semantics, full Rust implementation proof, or release-grade readiness.

## 2. Implementation Crosswalk

| Semantic object | Rust surface | Model surface | Evidence root |
| --- | --- | --- | --- |
| invalidation seed | `InvalidationSeed`, `InvalidationReasonKind` in `dependency.rs` | Lean `InvalidationSeed`; TLA `SeedInput` | TreeCalc `invalidation_seeds.json` |
| reverse reachability | `DependencyGraph::derive_invalidation_closure` queue over `reverse_edges` | Lean `ReverseReachable`; TLA `ReverseReachableClosure` | closure artifacts and scale impacted counts |
| closure record | `NodeInvalidationRecord` | Lean `NodeInvalidationRecord`; TLA `ClosureRecord` | `invalidation_closure.json` |
| rebind reason | `reason_requires_rebind` | Lean `ReasonRequiresRebind`; TLA `RebindReasons` | Rust tests and post-edit artifacts |
| structural rebind seed | `derive_structural_context_invalidation_seeds` | model reason `StructuralRebindRequired` | `tc_local_rebind_after_rename_001/post_edit` |
| dependency transition seed | `dependency_descriptor_transition_seeds` | Lean `DependencyTransitionReasons`; TLA B-record `DependencyAdded`/`DependencyReclassified` | dynamic add/release Rust tests and TraceCalc dynamic scenarios |
| rebind gate | `treecalc.rs` `rebind_gate_scan` | Lean `RebindGateRejectsBeforePublish`; TLA `RebindNoPublish` | `tc_local_rebind_after_rename_001/post_edit/result.json` |
| dynamic resolved reference | `DynamicResolved` descriptor and `dynamic_dependency_shape_updates` | dynamic transition/rebind reason layer | `tc_local_dynamic_resolved_publish_001` |
| dynamic potential residual | `DynamicPotential` descriptor diagnostic/runtime effect | dynamic unresolved/reject evidence | `tc_local_dynamic_reject_001`; `tc_w034_dynamic_dependency_negative_001` |
| soft-reference scale update | `execute_soft_reference_update` | rebind seed derivation pressure | `million_relative_rebind_f8_r1` |

## 3. Phase Contracts

`T06.SeedInvalidation`:

1. default formula-owner seeds are `StructuralRecalcOnly`;
2. structural rename/removal pressure emits `StructuralRebindRequired` when the formula owner, bound target, or caller context can change binding meaning;
3. descriptor transitions emit `DependencyAdded`, `DependencyRemoved`, and/or `DependencyReclassified`;
4. rebind-required reasons are distinguished from ordinary recalc-only and upstream-publication reasons.

`T07.CloseInvalidation`:

1. seed nodes are recorded;
2. every dependent reachable through reverse edges from a seed is recorded;
3. dependents reached through propagation receive `UpstreamPublication`;
4. cycle members are marked `CycleBlocked`;
5. dependency-change and structural-rebind seed records retain `requires_rebind = true`.

`T10.RebindGate`:

1. evaluation order is scanned against invalidation closure records;
2. any evaluated node with `requires_rebind = true` rejects before evaluation publication;
3. stale-binding rejection preserves the previous published view.

## 4. Checked Lean Artifact

Artifact: `formal/lean/OxCalc/CoreEngine/W046InvalidationRebind.lean`

Checked command:

```powershell
lean formal\lean\OxCalc\CoreEngine\W046InvalidationRebind.lean
```

Result: passed.

Lean definitions:

1. `ReverseReachable`: inductive reverse-reachability over graph edges.
2. `NoUnderInvalidation`: every node reverse-reachable from any seed has a closure record.
3. `ReasonRequiresRebind`: exact reason classifier for structural rebind and dependency-shape changes.
4. `DependencyTransitionReasons`: target-add/remove/reclassify reason model.
5. `RebindGateRejectsBeforePublish`: rebind records force rejection and no publication.
6. `InvalidationRebindSemanticModel`: envelope tying closure and rebind-gate obligations together.

Checked theorem targets:

| Theorem | Contract |
| --- | --- |
| `seedRecord_rebind_flag_matches_reason` | seed record rebind flag follows reason classifier |
| `dependencyAdded_requiresRebind` | dependency addition requires rebind |
| `dependencyRemoved_requiresRebind` | dependency removal requires rebind |
| `dependencyReclassified_requiresRebind` | dependency reclassification requires rebind |
| `upstreamPublication_doesNotRequireRebind` | propagated upstream publication alone is not rebind |
| `structuralRecalcOnly_doesNotRequireRebind` | ordinary recalc-only seed is not rebind |
| `dynamicTargetAdded_emitsDependencyAdded` | unresolved-to-resolved dynamic target emits dependency added |
| `dynamicTargetRemoved_emitsDependencyRemoved` | resolved-to-unresolved dynamic target emits dependency removed |
| `dependencyReclassification_emitsDependencyReclassified` | changed kind/rebind/carrier/target emits reclassification |
| `semanticModel_noUnderInvalidation` | semantic model carries no-under-invalidation |
| `semanticModel_rebindGateRejectsBeforePublish` | semantic model carries stale-binding no-publish |
| `sampleReachable_A_to_B` | one-step reverse reachability witness |
| `sampleClosure_contains_rebind_seed` | sample closure records a rebind seed |
| `sampleRejectedDecision_noPublish` | sample rejected decision has no publication |

## 5. Checked TLA Artifact

Artifacts:

1. `formal/tla/CoreEngineW046InvalidationRebind.tla`
2. `formal/tla/CoreEngineW046InvalidationRebind.smoke.cfg`
3. `docs/test-runs/core-engine/tla/w046-invalidation-rebind-001/`

Checked command:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run-tlc.ps1 formal\tla\CoreEngineW046InvalidationRebind.tla formal\tla\CoreEngineW046InvalidationRebind.smoke.cfg
```

Result: passed.

TLC summary:

| Field | Value |
| --- | --- |
| TLC version | `2.19 of 08 August 2024` |
| states generated | `4` |
| distinct states | `3` |
| queue left | `0` |
| complete-state depth | `3` |
| result | no error found |

Checked invariants:

1. `TypeInvariant`
2. `NoUnderInvalidation`
3. `RebindReasonsSetFlag`
4. `NonRebindReasonsDoNotSetFlag`
5. `DynamicTransitionSeedsPresent`
6. `DependentPropagationRecordsUpstream`
7. `RebindNoPublish`
8. `RejectedDecisionIsNoPublish`

Smoke model shape:

1. nodes `A`, `B`, `C`;
2. forward edges `B -> A`, `C -> B`;
3. seed `A` has `UpstreamPublication`;
4. seed `B` has `DependencyAdded` and `DependencyReclassified`, modeling a dynamic target transition;
5. bounded reverse closure reaches `A`, `B`, and `C`;
6. closure records `C` via upstream propagation;
7. rebind gate rejects before publication because `B` has rebind-required dependency-transition reasons.

## 6. Replay Roots

| Root | Use in this bead |
| --- | --- |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_rebind_after_rename_001/post_edit/invalidation_seeds.json` | structural rename emits `StructuralRebindRequired` |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_rebind_after_rename_001/post_edit/result.json` | rebind gate rejects and preserves published value |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_rebind_after_rename_001/post_edit/explain.json` | phase timing and rejection explanation for rebind gate |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_dynamic_resolved_publish_001/result.json` | dynamic resolved reference publishes value plus dependency shape update |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_dynamic_resolved_publish_001/dependency_graph.json` | dynamic resolved descriptor has a target and graph edge |
| `docs/test-runs/core-engine/treecalc-local/w034-independent-conformance-treecalc-001/cases/tc_local_dynamic_reject_001/result.json` | dynamic potential residual rejects with runtime dynamic-reference effect |
| `docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_w034_dynamic_dependency_negative_001.json` | TraceCalc dynamic negative reference scenario |
| `docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_w035_dynamic_dependency_switch_publish_001.json` | TraceCalc dynamic dependency switch publish scenario |
| `docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_w035_dirty_seed_closure_no_under_invalidation_001.json` | no-under-invalidation closure evidence seed |
| `docs/test-runs/core-engine/treecalc-scale/million_relative_rebind_f8_r1/run_summary.json` | scale soft-reference update: 999,991 expected and observed rebind seeds |

## 7. Assumptions And Limits

1. The Lean artifact models no-under-invalidation and rebind-gate obligations over graph facts, but does not prove the Rust queue implementation line-by-line.
2. The TLA artifact is a bounded smoke model, not full TLA verification.
3. The model exercises `DependencyAdded` plus `DependencyReclassified`; `DependencyRemoved` is checked in Lean and Rust tests but not in the bounded TLA instance.
4. Dynamic `INDIRECT` remains split in current implementation evidence: unresolved `DynamicPotential` emits diagnostics/runtime effects and rejects; `DynamicResolved` emits graph edges, runtime effects, and dependency-shape updates.
5. General OxFunc semantics for `INDIRECT` remain outside this bead. The model covers the OxCalc-visible dependency/rebind consequences.
6. Evaluation-order, working-value reads, candidate/publication refinement, and TraceCalc equivalence are successor beads.

## 8. Successor Obligations

| Successor bead | Consumes from this packet |
| --- | --- |
| `calc-gucd.4` | rebind-gate rejection and cycle/dirty state records for recalc tracker pre/post conditions |
| `calc-gucd.5` | closure records and rebind gate as preconditions for evaluation-order/read discipline |
| `calc-gucd.6` | dynamic transition and rebind reject observations for TraceCalc refinement |
| `calc-gucd.7` | OxFml/OxFunc-visible `INDIRECT` and dynamic-reference effect boundary |
| `calc-gucd.9` | `invalidation_closure_derivation`, `soft_reference_update_rebind_seed_derivation`, and `rebind_gate_scan` phase signatures |

## 9. Validation

| Command | Result |
| --- | --- |
| `lean formal\lean\OxCalc\CoreEngine\W046InvalidationRebind.lean` | passed |
| `powershell -ExecutionPolicy Bypass -File scripts\run-tlc.ps1 formal\tla\CoreEngineW046InvalidationRebind.tla formal\tla\CoreEngineW046InvalidationRebind.smoke.cfg` | passed |
| `cargo test -p oxcalc-core invalidation` | passed |
| `cargo test -p oxcalc-core rebind` | passed |
| `cargo test -p oxcalc-core dynamic` | passed |
| `powershell -ExecutionPolicy Bypass -File scripts\check-worksets.ps1` | passed |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed |

## 10. Semantic-Equivalence Statement

This bead changes formal artifacts, evidence metadata, spec/status documents, and bead state only.

Observable OxCalc behavior is invariant under this bead. It does not change invalidation closure, dependency graph construction, dynamic-reference resolution, soft-reference rebind derivation, recalc tracker behavior, evaluation order, working-value reads, TraceCalc execution, TreeCalc/CoreEngine execution, OxFml evaluation, rejection, publication, pack policy, or service readiness.

## 11. Pre-Closure Verification Checklist

| # | Check | Result |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W046 status surfaces, transition catalog, fragment ledger, and formal layout note the invalidation/rebind model |
| 2 | Pack expectations updated for affected packs? | yes; no pack expectation changed by this model bead |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; replay roots are listed in Section 6 and the TLA run artifact is checked in |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no policy/strategy change, invariant statement in Section 10 |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no normative FEC/F3E seam change and no OxFml handoff needed |
| 6 | All required tests pass? | yes; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for declared `calc-gucd.3` target; broader dynamic/INDIRECT semantics and implementation proof limits are explicit |
| 8 | Completion language audit passed? | yes; no full Rust, full dynamic-reference, full TLA, or release-grade proof is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; no ordered workset register change required |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; current semantic bead moved to `calc-gucd.4` |
| 11 | execution-state blocker surface updated? | yes; `.beads/` owns `calc-gucd.3` state |

## 12. Completion Claim Self-Audit

| Step | Result |
| --- | --- |
| Scope re-read | pass; bead asks for invalidation seeds, reverse-reachability closure, dynamic dependency transitions, soft-reference update, `INDIRECT` behavior, rebind flags, assumptions, and blockers |
| Gate criteria re-read | pass; model artifacts, checked commands, replay roots, and explicit limits are recorded |
| Silent scope reduction check | pass; full `INDIRECT`/OxFunc semantics, Rust queue proof, unbounded TLA verification, evaluation-order proof, and refinement are explicitly not hidden |
| "Looks done but is not" pattern check | pass; the packet says the model is checked and scoped, not a full mechanized semantic proof of all dynamic/rebind behavior |
| Include result | pass; checklist, self-audit, semantic-equivalence statement, and three-axis report are included |

## 13. Current Status

- execution_state: `calc-gucd.3_closed`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
