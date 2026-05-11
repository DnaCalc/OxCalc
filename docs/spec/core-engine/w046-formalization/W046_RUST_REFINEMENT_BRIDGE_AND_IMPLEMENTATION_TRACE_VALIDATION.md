# W046 Rust Refinement Bridge And Implementation Trace Validation

Status: `calc-gucd.18_rust_refinement_bridge_validated`

Workset: `W046`

Parent epic: `calc-gucd`

Bead: `calc-gucd.18`

## 1. Purpose

This packet maps selected real Rust TreeCalc/CoreEngine and TraceCalc execution artifacts into the W046 integrated semantic-kernel vocabulary.

The scoped goal is a refinement bridge for selected emitted traces, not a line-by-line proof of the whole implementation. The bridge consumes:

1. the integrated semantic kernel from `calc-gucd.15`,
2. finite graph/dataflow/order facts from `calc-gucd.16`,
3. proof-carrying trace checker facts from `calc-gucd.17`,
4. a focused Rust unit test over in-memory TreeCalc state,
5. a small Lean relation that states publication/reject refinement prerequisites.

## 2. Implementation-To-Semantics Mapping

Mapping register: `docs/test-runs/core-engine/refinement/w046-rust-refinement-bridge-001/implementation_semantic_mapping.json`

| Semantic fact | Rust/artifact authority | Validation route |
| --- | --- | --- |
| `graphBuilt` | `DependencyGraph::build`, `LocalTreeCalcRunArtifacts.dependency_graph`, `dependency_graph.json` | checker `graph_artifact_parseable` |
| `reverseConverse` | `DependencyGraph.reverse_edges`, forward edge artifact | Rust unit test checks native reverse edges; checker derives reverse index |
| `invalidationClosureCoversOrder` | `derive_invalidation_closure`, `invalidation_closure.json`, `evaluation_order` | checker `evaluation_order_covered_by_invalidation_closure_or_reject` |
| `orderRespectsDependencies` | `topological_formula_order`, `evaluation_order`, graph edges | Rust unit test and checker `formula_edges_respect_evaluation_order` |
| `stableOrPriorReads` | TreeCalc formula evaluation loop, graph/order artifacts | checker `stable_input_read_observed` and `prior_formula_read_observed` |
| `candidateBuilt/publicationBuilt` | `AcceptedCandidateResult`, `PublicationBundle`, result JSON | checker `candidate_publication_bundle_consistent` |
| `rejectBuilt/noPublicationBundle` | `RejectDetail`, result JSON | checker `reject_has_no_publication_bundle` |
| `dynamicReject` | `DependencyDescriptorKind::DynamicPotential`, `LocalTreeCalcError::DynamicReference` | checker `dynamic_potential_descriptor_rejects` |
| `cycleReject` | dependency cycle groups, TraceCalc planner/runner trace | checker `cycle_region_reject_no_publication` |
| `traceEmissionPresent` | TreeCalc/TraceCalc runner artifacts | checker sidecar/equality-surface facts |

## 3. Checked Code Bridge

Rust test added: `local_treecalc_engine_exposes_w046_refinement_bridge_facts` in `src/oxcalc-core/src/treecalc.rs`.

The test executes a two-formula chain and checks selected in-memory implementation facts before JSON projection:

1. every forward dependency edge has a reverse-edge bucket containing the same edge,
2. dependency targets precede dependent owners when both are in `evaluation_order`,
3. every evaluated node is present in `invalidation_closure.records`,
4. published runs carry candidate and publication bundles,
5. candidate target set equals `evaluation_order`,
6. candidate value updates equal publication value delta,
7. rejected runs carry no publication bundle and do carry reject detail,
8. verified-clean runs carry neither publication nor reject detail.

Checked command:

```powershell
cargo test -p oxcalc-core local_treecalc_engine_exposes_w046_refinement_bridge_facts
```

Result: passed; 1 test passed.

## 4. Checked Lean Relation

Lean artifact: `formal/lean/OxCalc/CoreEngine/W046RustRefinementBridge.lean`

The artifact defines `ImplementationTraceFacts` and refinement predicates:

1. `PublishedRefinesIntegratedKernel`,
2. `RejectedRefinesIntegratedKernel`,
3. `DynamicRejectRefines`,
4. `CycleRejectRefines`,
5. `RebindRejectRefines`.

Checked theorem targets:

| Theorem | Meaning |
| --- | --- |
| `chain_publish_refines_kernel` | checked publication trace fact row refines kernel prerequisites |
| `let_lambda_publish_refines_kernel` | LET/LAMBDA publication row reuses publication refinement prerequisites |
| `dynamic_reject_refines_kernel` | dynamic reject row refines reject/no-publication prerequisites |
| `cycle_reject_refines_kernel` | cycle reject row refines reject/no-publication prerequisites |
| `rebind_reject_refines_kernel` | rebind reject row refines reject/no-publication prerequisites |
| `published_trace_has_no_reject` | publication refinement implies no reject fact |
| `rejected_trace_has_no_publication` | rejection refinement implies no publication bundle fact |

Checked command:

```powershell
lean formal/lean/OxCalc/CoreEngine/W046RustRefinementBridge.lean
```

Result: passed.

## 5. Proof-Carrying Trace Checker Reuse

The bridge reuses the `calc-gucd.17` checker output:

`docs/test-runs/core-engine/refinement/w046-proof-carrying-trace-001/checker_output.json`

Summary:

| Metric | Value |
| --- | --- |
| checked artifacts | `5` |
| failures | `0` |
| blockers in checker output | `0` |
| TreeCalc cases | chain publish, LET/LAMBDA publish, dynamic reject, rebind fixture publish |
| TraceCalc scenarios | cycle reject |

This is the artifact-level evidence that selected implementation traces satisfy the W046 semantic fact families named in Section 2.

## 6. Dynamic-Dependency And Invalidation Blocker Reassessment

| Prior blocker/gap | Reassessment |
| --- | --- |
| dynamic-dependency projection gap from `.6` | sharpened: selected dynamic-potential TreeCalc artifact is independently checkable as reject/no-publication via descriptor plus reject detail |
| normalized invalidation comparator gap from `.6` | sharpened: selected artifacts check evaluation-order coverage by invalidation closure; full normalized comparator across regenerated runs remains outside this bead |
| reverse-edge artifact gap | sharpened: native Rust in-memory reverse edges are unit-tested, while JSON replay derives reverse index from forward edges until a native reverse-edge sidecar exists |
| per-read evidence gap | sharpened: stable/prior facts are graph/order-derived; native per-read event emission remains a future enhancement |

## 7. Exact Residuals

The bridge leaves these explicit residuals:

1. `rust_tarjan_line_proof_not_discharged`,
2. `rust_topological_queue_line_proof_not_discharged`,
3. `native_reverse_edge_json_sidecar_absent`,
4. `native_per_read_trace_events_absent`,
5. `positive_dynamic_dependency_publication_refinement_not_discharged`,
6. `arbitrary_finite_graph_refinement_not_discharged`.

These residuals do not block the declared `.18` scoped bridge because selected Rust in-memory facts, selected emitted artifacts, and the Lean relation are all checked. They do block any claim of full implementation proof or release-grade verification.

## 8. Evidence Root

Evidence root: `docs/test-runs/core-engine/refinement/w046-rust-refinement-bridge-001/`

| Artifact | Meaning |
| --- | --- |
| `implementation_semantic_mapping.json` | implementation-to-semantics mapping table and exact residuals |
| `run_summary.json` | stable evidence summary and command results |

## 9. Handoff Assessment

No OxFml handoff is filed by this bead.

Reason:

1. The bead maps OxCalc-local Rust/artifact facts into the existing W046 semantic kernel.
2. It does not change shared FEC/F3E clauses or OxFml evaluator-facing contracts.
3. Broad OxFml/OxFunc semantics remain outside this bridge.

## 10. Successor Obligations

| Successor bead | Consumes from this packet |
| --- | --- |
| `calc-gucd.8` | mapping rows and checker outputs for proof-service/evidence-classifier coverage |
| `calc-gucd.9` | mapping rows as semantic signatures for scale/performance regression classification |
| `calc-gucd.10` | residuals and bridge validation for downstream consequence reassessment |
| `calc-gucd.11` | closure audit over semantic-spine coverage, residuals, and successor routing |

## 11. Validation

| Command | Result |
| --- | --- |
| `lean formal/lean/OxCalc/CoreEngine/W046RustRefinementBridge.lean` | passed |
| `cargo test -p oxcalc-core local_treecalc_engine_exposes_w046_refinement_bridge_facts` | passed; 1 test passed |
| `python scripts/check-w046-proof-carrying-trace.py ... --output docs/test-runs/core-engine/refinement/w046-proof-carrying-trace-001/checker_output.json` | passed; 5 artifacts, 0 failures |
| JSON parse/reference check for `w046-rust-refinement-bridge-001` root | passed |

## 12. Semantic-Equivalence Statement

This bead adds a focused Rust test, a Lean relation artifact, mapping/evidence metadata, spec/status text, and bead-state updates. The Rust code change is test-only.

Observable OxCalc behavior is invariant under this bead. It does not change dependency graph construction, invalidation closure, evaluation order, formula evaluation, candidate construction, rejection, publication, TraceCalc execution, TreeCalc execution, OxFml/OxFunc behavior, proof-service behavior, pack policy, performance behavior, or service readiness.

## 13. Pre-Closure Verification Checklist

| # | Check | Result |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet plus mapping/run-summary artifacts record the bridge |
| 2 | Pack expectations updated for affected packs? | yes; no pack expectation changed by this bridge bead |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; `.17` checker output and `.18` mapping root validate selected TreeCalc/TraceCalc artifacts |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no behavior/policy change, invariant statement in Section 12 |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no normative seam change and no handoff needed |
| 6 | All required tests pass? | yes; see Section 11 |
| 7 | No known semantic gaps remain in declared scope? | yes for scoped selected-artifact bridge; exact residuals are explicit |
| 8 | Completion language audit passed? | yes; no full Rust proof, arbitrary graph refinement, or release-grade proof is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; no ordered workset register change required |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; current semantic bead moves to `calc-gucd.8` after this bridge |
| 11 | execution-state blocker surface updated? | yes; `.beads/` owns `calc-gucd.18` state |

## 14. Completion Claim Self-Audit

| Step | Result |
| --- | --- |
| Scope re-read | pass; bead asks for mapping table, checker output/kernel satisfaction or blockers, dynamic/invalidation blocker sharpening, focused Rust tests, Lean/TLA relation reuse, and no broad release promotion |
| Gate criteria re-read | pass; mapping root, checker reuse, Rust test, Lean relation, residuals, and validation commands are recorded |
| Silent scope reduction check | pass; line-proof and arbitrary-refinement gaps are explicit residuals |
| "Looks done but is not" pattern check | pass; this is a selected-artifact bridge, not a full implementation proof |
| Include result | pass; checklist, self-audit, semantic-equivalence statement, and three-axis report are included |

## 15. Current Status

- execution_state: `calc-gucd.18_closed`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
