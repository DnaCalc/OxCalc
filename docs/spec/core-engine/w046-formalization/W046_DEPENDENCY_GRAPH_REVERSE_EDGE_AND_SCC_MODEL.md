# W046 Dependency Graph Reverse-Edge And SCC Model

Status: `calc-gucd.2_dependency_graph_model_validated`

Workset: `W046`

Parent epic: `calc-gucd`

Bead: `calc-gucd.2`

## 1. Purpose

This packet is the first W046 engine-semantic proof-spine model after the redirect bead.

It covers the graph-building slice of the calculation engine:

1. prepared dependency descriptors,
2. valid forward dependency edges,
3. reverse-edge converse,
4. diagnostics for untargeted descriptors,
5. non-trivial SCC and self-cycle classification shape.

This packet does not claim a full mechanized proof of the Rust Tarjan implementation. It establishes checked Lean/TLA model targets and replay roots for the semantic contracts that later W046 beads depend on.

## 2. Implementation Crosswalk

| Semantic object | Rust surface | Model surface | Evidence root |
| --- | --- | --- | --- |
| descriptor | `src/oxcalc-core/src/dependency.rs` `DependencyDescriptor` | Lean `Descriptor`; TLA `DescriptorIds`, `OwnerOf`, `TargetOf`, `KindOf` | TreeCalc dependency graph JSON descriptors |
| forward edge | `DependencyEdge`; `DependencyGraph::build` targetful valid-descriptor branch | Lean `Edge`; TLA `EdgeRecord` | TreeCalc `edges` arrays |
| reverse edge | `DependencyGraph::build` `reverse_edges.entry(edge.target_node_id)` | Lean `BuildReverseEdges`; TLA `ReverseRecord` | model-checked converse invariant |
| untargeted diagnostic | `DependencyDiagnosticKind::*Reference` branches for `target_node_id: None` | Lean `DiagnosticRequired`; TLA `DiagnosticRecord` | dynamic/host/capability/shape TreeCalc graph artifacts |
| cycle group | `find_cycle_groups` Tarjan scan over `edges_by_owner` | Lean `CycleGroupSupported`; TLA `CycleGroupClassification` | `tc_cycle_region_reject_001` |
| reference planner analogue | `src/oxcalc-tracecalc/src/planner.rs` direct/reverse dependency maps and SCC grouping | same relation vocabulary | TraceCalc cycle scenario |

## 3. Phase Contracts

`T03.BuildGraph`:

1. every descriptor is retained under its owner index;
2. a descriptor with an existing owner and existing target emits one forward edge;
3. a missing owner, missing target, or untargeted descriptor emits a diagnostic rather than a silent edge;
4. every emitted forward edge has an owner and target in the snapshot.

`T04.BuildReverseEdges`:

1. reverse entries are derived from forward edges;
2. each forward edge `(owner,target,descriptor)` appears under `target`;
3. every reverse entry points back to a forward edge.

`T05.ClassifySCC`:

1. non-trivial SCCs are cycle groups;
2. singleton components are cycle groups only when a self-loop exists;
3. cycle groups are snapshot-node groups and become inputs to cycle-blocked recalc handling.

## 4. Checked Lean Artifact

Artifact: `formal/lean/OxCalc/CoreEngine/W046DependencyGraph.lean`

Checked command:

```powershell
lean formal\lean\OxCalc\CoreEngine\W046DependencyGraph.lean
```

Result: passed.

Lean definitions:

1. `Graph`: snapshot nodes, descriptors, forward edges, reverse edges, diagnostics, and cycle groups.
2. `BuildReverseEdges`: executable list-level reverse-edge derivation.
3. `ReverseEdgeConverse`: forward/reverse relation contract.
4. `DiagnosticsPreserved`: required diagnostic descriptors have a diagnostic witness.
5. `CycleGroupsClassified`: cycle groups are snapshot groups with non-trivial SCC or self-loop support.
6. `BuildGraphSemanticModel`: envelope carrying the graph-build obligations.

Checked theorem targets:

| Theorem | Contract |
| --- | --- |
| `buildReverseEdges_converse` | list-level reverse construction is exactly the forward-edge converse |
| `graphModel_forwardEdges_have_snapshot_targets` | emitted forward edges target snapshot nodes |
| `graphModel_reverse_edges_are_converse` | model graph has reverse-edge converse |
| `graphModel_diagnostics_preserve_required_descriptors` | required diagnostics are preserved |
| `graphModel_cycle_groups_are_classified` | cycle groups satisfy classification shape |
| `sampleReverseConverse_AB`, `sampleReverseConverse_BA` | tiny A/B cycle witness for converse entries |

## 5. Checked TLA Artifact

Artifacts:

1. `formal/tla/CoreEngineW046DependencyGraph.tla`
2. `formal/tla/CoreEngineW046DependencyGraph.smoke.cfg`
3. `docs/test-runs/core-engine/tla/w046-dependency-graph-001/`

Checked command:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run-tlc.ps1 formal\tla\CoreEngineW046DependencyGraph.tla formal\tla\CoreEngineW046DependencyGraph.smoke.cfg
```

Result: passed.

TLC summary:

| Field | Value |
| --- | --- |
| TLC version | `2.19 of 08 August 2024` |
| states generated | `3` |
| distinct states | `2` |
| queue left | `0` |
| complete-state depth | `2` |
| result | no error found |

Checked invariants:

1. `TypeInvariant`
2. `ForwardEdgesHaveValidTargets`
3. `ReverseEdgeConverse`
4. `DiagnosticsPreserved`
5. `CycleGroupClassification`

Smoke model shape:

1. nodes `A`, `B`, `C`;
2. descriptors `dAB`, `dBA`, and `dDynamic`;
3. `A -> B` and `B -> A` form a non-trivial SCC;
4. `dDynamic` has no target and must emit a `DynamicPotentialReference` diagnostic;
5. `CycleGroupsInput = {{A, B}}`.

## 6. Replay Roots

| Root | Use in this bead |
| --- | --- |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_relative_sum_001/dependency_graph.json` | concrete descriptor-to-edge and acyclic reverse-target evidence |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_dynamic_reject_001/dependency_graph.json` | concrete untargeted `DynamicPotentialReference` diagnostic evidence |
| `docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_cycle_region_reject_001.json` | hand-auditable A/B cycle-region seed for non-trivial SCC handling |
| `docs/test-runs/core-engine/treecalc-scale/*/phase_timings.json` | existing phase name `dependency_graph_build_and_cycle_scan` for later W046 scale semantic-regression binding |

## 7. Assumptions And Limits

1. The Lean artifact proves the reverse-edge constructor theorem directly over lists and records the other graph obligations as a semantic model envelope.
2. The TLA artifact model-checks a bounded graph-build transition with one non-trivial SCC and one untargeted dynamic descriptor.
3. This bead does not prove the Rust Tarjan implementation line-by-line.
4. This bead does not prove SCC completeness over arbitrary finite graphs.
5. Missing-owner and missing-target diagnostics are represented in the contract and Rust crosswalk, but the bounded TLA smoke instance exercises untargeted dynamic diagnostics first.
6. Invalidation, rebind, recalc-state, evaluation-order, and TraceCalc refinement are successor beads.

## 8. Successor Obligations

| Successor bead | Consumes from this packet |
| --- | --- |
| `calc-gucd.3` | reverse-edge converse and diagnostic preservation as inputs to invalidation/rebind closure |
| `calc-gucd.4` | cycle-group classification shape as input to cycle-blocked recalc tracker transitions |
| `calc-gucd.5` | acyclic graph and cycle-group split as input to evaluation-order/read-discipline model |
| `calc-gucd.6` | TraceCalc cycle and dependency observations as first selected-kernel refinement events |
| `calc-gucd.9` | `dependency_graph_build_and_cycle_scan` phase as scale/performance semantic-regression signature |

## 9. Validation

| Command | Result |
| --- | --- |
| `lean formal\lean\OxCalc\CoreEngine\W046DependencyGraph.lean` | passed |
| `powershell -ExecutionPolicy Bypass -File scripts\run-tlc.ps1 formal\tla\CoreEngineW046DependencyGraph.tla formal\tla\CoreEngineW046DependencyGraph.smoke.cfg` | passed |
| `cargo test -p oxcalc-core dependency` | passed |
| `powershell -ExecutionPolicy Bypass -File scripts\check-worksets.ps1` | passed |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed |

## 10. Semantic-Equivalence Statement

This bead changes formal artifacts, evidence metadata, spec/status documents, and bead state only.

Observable OxCalc behavior is invariant under this bead. It does not change dependency graph construction, invalidation closure, dynamic-reference resolution, soft-reference rebind, recalc tracker behavior, evaluation order, working-value reads, TraceCalc execution, TreeCalc/CoreEngine execution, OxFml evaluation, rejection, publication, pack policy, or service readiness.

## 11. Pre-Closure Verification Checklist

| # | Check | Result |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W046 status surfaces, and formal layout note the graph/reverse-edge/SCC model |
| 2 | Pack expectations updated for affected packs? | yes; no pack expectation changed by this model bead |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; replay roots are listed in Section 6 and the TLA run artifact is checked in |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no policy/strategy change, invariant statement in Section 10 |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no FEC/F3E seam change and no OxFml handoff needed |
| 6 | All required tests pass? | yes; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for declared `calc-gucd.2` target; broader SCC completeness and Tarjan implementation proof remain explicit successor/open limits |
| 8 | Completion language audit passed? | yes; no full Rust, full SCC, full TLA, or release-grade proof is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; no ordered workset register change required |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; current semantic bead moved to `calc-gucd.3` |
| 11 | execution-state blocker surface updated? | yes; `.beads/` owns `calc-gucd.2` state |

## 12. Completion Claim Self-Audit

| Step | Result |
| --- | --- |
| Scope re-read | pass; bead asks for graph/reverse-edge/SCC model, Lean/TLA targets, replay roots, assumptions, and blockers |
| Gate criteria re-read | pass; model artifacts, checked commands, replay roots, and explicit limits are recorded |
| Silent scope reduction check | pass; Tarjan line proof, arbitrary SCC completeness, missing-owner/target smoke coverage, invalidation, rebind, recalc, evaluation, and refinement are explicitly not hidden |
| "Looks done but is not" pattern check | pass; the packet says the model is checked and scoped, not a full mechanized semantic proof of all Rust graph/SCC behavior |
| Include result | pass; checklist, self-audit, semantic-equivalence statement, and three-axis report are included |

## 13. Current Status

- execution_state: `calc-gucd.2_closed`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
