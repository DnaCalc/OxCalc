# W046 Proof-Carrying Trace And Semantic Replay Checker

Status: `calc-gucd.17_proof_carrying_trace_validated`

Workset: `W046`

Parent epic: `calc-gucd`

Bead: `calc-gucd.17`

## 1. Purpose

This packet defines the W046 proof-carrying trace lane and adds a deterministic semantic replay checker over existing TreeCalc and TraceCalc artifacts.

The scoped goal is not to replace engine execution. The goal is to make selected emitted artifacts independently checkable for semantic facts that W046 already models:

1. graph edge and descriptor facts,
2. derived reverse-edge converse facts,
3. invalidation closure presence and evaluation-order coverage,
4. dependency-before-dependent order facts,
5. stable/prior read facts,
6. candidate/publication consistency,
7. reject/no-publication consistency,
8. dynamic dependency reject facts,
9. cycle-region reject/no-publication TraceCalc facts,
10. replay projection equality-surface presence.

## 2. Source Surfaces Reviewed

| Surface | Intake |
| --- | --- |
| `src/oxcalc-core/src/treecalc_runner.rs` | existing TreeCalc result, graph, invalidation, publication, reject, and sidecar emission |
| `src/oxcalc-core/src/treecalc.rs` | source of evaluation order, candidate, publication, reject, and working-value behavior |
| `src/oxcalc-tracecalc/src/contracts.rs` | TraceCalc scenario/result/replay projection contracts |
| `src/oxcalc-tracecalc/src/assertions.rs` | assertion and conformance helper behavior |
| `src/oxcalc-tracecalc/src/replay_mappings.rs` | normalized event-family/equality-surface mapping vocabulary |
| `src/oxcalc-tracecalc/src/runner.rs` | TraceCalc result/sidecar/replay-appliance artifact emission |
| W046 `.2-.7`, `.15`, and `.16` packets | semantic fact families consumed by the checker |

## 3. Proof-Carrying Trace Schema

The checker consumes existing native artifacts plus derived replay facts. It does not mutate baseline roots.

### 3.1 TreeCalc Result Input

Required fields:

| Field | Meaning |
| --- | --- |
| `case_id` | checked case identity |
| `result_state` | `published`, `rejected`, or other state routed to blocker/failure |
| `dependency_graph_path` | native graph sidecar with `edges`, `descriptors`, `cycle_groups`, and diagnostics |
| `invalidation_closure_path` | native closure sidecar with node records and rebind flags |
| `evaluation_order` | native ordered formula node list |
| `candidate_result` | candidate target/value facts when published |
| `publication_bundle` | publication identity and published-view delta when published |
| `reject_detail` | reject kind/detail when rejected |

Derived checker facts:

| Fact | Check |
| --- | --- |
| `graph_artifact_parseable` | dependency graph sidecar is present and valid JSON |
| `forward_edges_well_formed` | every edge has owner, target, and edge id |
| `derived_reverse_index_converse` | checker derives target-to-owner reverse index from all forward edges |
| `edge_descriptor_links_checked` | every edge descriptor id is present in descriptors |
| `invalidation_closure_artifact_parseable` | closure sidecar is present and valid JSON |
| `evaluation_order_covered_by_invalidation_closure_or_reject` | published order nodes are covered by invalidation closure, or the run rejected |
| `formula_edges_respect_evaluation_order` | when owner and target are both in the order, target precedes owner |
| `stable_input_read_observed` | at least one ordered formula reads a target outside the order, i.e. stable input |
| `prior_formula_read_observed` | at least one ordered formula reads a prior ordered formula |
| `candidate_publication_bundle_consistent` | published candidate targets match order and candidate value keys match publication value keys |
| `reject_has_no_publication_bundle` | rejected run has no publication bundle |
| `dynamic_potential_descriptor_rejects` | dynamic-potential descriptor runs reject |
| `rebind_nodes_reject_no_publish` | closure records requiring rebind reject rather than publish when present |

### 3.2 TraceCalc Result Input

Required fields:

| Field | Meaning |
| --- | --- |
| `scenario_id` | checked scenario identity |
| `artifact_paths.trace` | event stream sidecar |
| `artifact_paths.rejects` | reject-set sidecar |
| `artifact_paths.published_view` | published-view sidecar |
| `artifact_paths.counters` | counter sidecar |
| `replay_projection` | replay class and equality surface requirements |
| `assertion_failures` | reference assertion result |
| `conformance_mismatches` | oracle/engine conformance result |
| `validation_failures` | scenario validation result |

Derived checker facts:

| Fact | Check |
| --- | --- |
| `tracecalc_sidecars_parseable` | referenced sidecars exist and parse |
| `normalized_event_families_present` | every trace event has a normalized family |
| `required_equality_surfaces_have_artifacts` | projection equality surfaces have concrete sidecars |
| `cycle_region_reject_no_publication` | cycle traces contain rejection and no publication event |
| `tracecalc_assertions_and_conformance_clean` | assertions and conformance mismatches are empty |
| `tracecalc_validation_clean` | scenario validation failures are empty |

## 4. Checker Implementation

Checker: `scripts/check-w046-proof-carrying-trace.py`

The checker is deterministic, local, and read-only with respect to baseline artifacts. It writes only the requested validation output path.

Invocation used for this packet:

```powershell
python scripts/check-w046-proof-carrying-trace.py `
  --treecalc-result docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_recalc_chain_after_constant_edit_001/result.json `
  --treecalc-result docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_let_lambda_capture_publish_001/result.json `
  --treecalc-result docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_dynamic_reject_001/result.json `
  --treecalc-result docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_rebind_after_rename_001/result.json `
  --tracecalc-result docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/scenarios/tc_cycle_region_reject_001/result.json `
  --output docs/test-runs/core-engine/refinement/w046-proof-carrying-trace-001/checker_output.json
```

Result: passed.

## 5. Validation Artifacts

Evidence root: `docs/test-runs/core-engine/refinement/w046-proof-carrying-trace-001/`

| Artifact | Meaning |
| --- | --- |
| `checker_output.json` | machine-readable checker output, per-artifact fact lists, failures, and blockers |
| `run_summary.json` | stable run summary for W046 evidence indexing |

Summary:

| Metric | Value |
| --- | --- |
| checked artifacts | `5` |
| TreeCalc cases checked | `4` |
| TraceCalc scenarios checked | `1` |
| failure count | `0` |
| blocker count in validation output | `0` |

Checked TreeCalc cases:

1. `tc_local_recalc_chain_after_constant_edit_001`
2. `tc_local_let_lambda_capture_publish_001`
3. `tc_local_dynamic_reject_001`
4. `tc_local_rebind_after_rename_001`

Checked TraceCalc scenario:

1. `tc_cycle_region_reject_001`

## 6. Evidence Policy: Native Authority Versus Replay Projection Sidecars

W046 treats native engine artifacts as the authority for facts they directly emit:

| Native authority | Fact authority |
| --- | --- |
| TreeCalc `dependency_graph.json` | forward edges, descriptors, diagnostics, cycle groups currently emitted by TreeCalc |
| TreeCalc `invalidation_closure.json` | closure records, calc states, reasons, and rebind flags |
| TreeCalc `result.json` | result state, evaluation order, candidate result, publication bundle, reject detail, and paths |
| TraceCalc `trace.json` | event labels, normalized event families, cycle-region/reject/publication observations |
| TraceCalc result sidecars | published view, reject set, counters, validation, assertion, and conformance outcomes |

Replay projection facts are checker-derived and must name their derivation:

| Projection fact | Derivation |
| --- | --- |
| derived reverse index | recomputed from TreeCalc forward edges by target node |
| topological order consistency | checked from TreeCalc forward edges and `evaluation_order` |
| stable/prior read classification | checked from TreeCalc forward edges and `evaluation_order` membership/order |
| candidate/publication consistency | checked by comparing native candidate targets/value keys to native publication value keys |
| cycle reject/no-publication | checked from TraceCalc event labels and sidecar presence |

Projection facts cannot override native artifacts. If native artifacts are missing or contradictory, the checker reports failure.

## 7. Failure Modes

| Failure mode | Checker response |
| --- | --- |
| missing result/sidecar JSON | failure |
| invalid JSON | failure |
| missing required result path | failure |
| malformed dependency edge | failure |
| edge references missing descriptor | failure |
| ordered dependency target appears after owner | failure |
| published candidate targets differ from evaluation order | failure |
| candidate value keys differ from publication value keys | failure |
| published run carries reject detail | failure |
| rejected run carries publication bundle | failure |
| dynamic-potential descriptor publishes | failure |
| rebind-required closure record publishes | failure |
| cycle trace lacks reject event | failure |
| cycle trace contains publication event | failure |
| TraceCalc replay projection equality surface has no sidecar | failure |
| TraceCalc assertion, conformance, or validation failures are present | failure |
| unclassified TreeCalc result state | blocker in checker output |

## 8. Limits And Residuals

This bead validates proof-carrying facts over selected existing W046/W037 roots. It does not claim:

1. native reverse-edge sidecar emission from TreeCalc; the checker derives reverse-index converse from forward edges,
2. native per-read event traces for every formula read; stable/prior facts are graph/order-derived,
3. arbitrary finite graph proof beyond the `.16` shape model,
4. line-by-line Rust proof of TreeCalc or TraceCalc implementations,
5. full Rust-to-semantic-kernel refinement bridge; that remains `calc-gucd.18`,
6. broad OxFml evaluator proof or broad OxFunc semantics,
7. mutation or regeneration of historical W037/W046 baseline roots.

## 9. Successor Obligations

| Successor bead | Consumes from this packet |
| --- | --- |
| `calc-gucd.18` | checker fact vocabulary, exact native/projection authority split, and validated artifact set for Rust refinement bridge |
| `calc-gucd.8` | checker output as proof-service/evidence-classifier input |
| `calc-gucd.9` | fact-family names for semantic-regression signatures |
| `calc-gucd.10` | proof-carrying trace validation status for downstream consequence reassessment |
| `calc-gucd.11` | closure audit over checker limits and successor routes |

## 10. Validation

| Command | Result |
| --- | --- |
| `python scripts/check-w046-proof-carrying-trace.py ... --output docs/test-runs/core-engine/refinement/w046-proof-carrying-trace-001/checker_output.json` | passed; 5 artifacts, 0 failures |
| `python -m py_compile scripts/check-w046-proof-carrying-trace.py` | passed |
| JSON parse/reference check for `w046-proof-carrying-trace-001` root | passed |

## 11. Semantic-Equivalence Statement

This bead adds a read-only checker, proof-carrying trace schema/spec text, evidence metadata, and bead/status updates only.

Observable OxCalc behavior is invariant under this bead. It does not change dependency graph construction, invalidation closure, topological order, formula evaluation, candidate construction, rejection, publication, TraceCalc execution, TreeCalc execution, OxFml/OxFunc behavior, proof-service behavior, pack policy, performance behavior, or service readiness.

## 12. Pre-Closure Verification Checklist

| # | Check | Result |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet defines schema, checker, evidence policy, failure modes, and limits |
| 2 | Pack expectations updated for affected packs? | yes; no pack expectation changed by this checker bead |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; checker output validates four TreeCalc results and one TraceCalc result |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no policy/strategy change, invariant statement in Section 11 |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no normative FEC/F3E or OxFml-facing change and no handoff needed |
| 6 | All required tests pass? | yes; see Section 10 |
| 7 | No known semantic gaps remain in declared scope? | yes for the declared checker/schema target; native reverse sidecar, per-read traces, arbitrary finite proof, and Rust refinement bridge limits are explicit residuals |
| 8 | Completion language audit passed? | yes; no full line-proof, release-grade proof, or broad evaluator proof is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; no ordered workset register change required |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; current semantic bead moves to `calc-gucd.18` after this bead |
| 11 | execution-state blocker surface updated? | yes; `.beads/` owns `calc-gucd.17` state |

## 13. Completion Claim Self-Audit

| Step | Result |
| --- | --- |
| Scope re-read | pass; bead asks for schema/spec packet, validator implementation or exact blocker, deterministic validation over existing runs, failure modes, and evidence policy |
| Gate criteria re-read | pass; checker implementation, output root, run summary, schema, failure modes, and native/projection authority split are recorded |
| Silent scope reduction check | pass; limitations are explicit and routed to `.18` or later beads |
| "Looks done but is not" pattern check | pass; the checker validates selected artifacts and does not claim full implementation proof |
| Include result | pass; checklist, self-audit, semantic-equivalence statement, and three-axis report are included |

## 14. Current Status

- execution_state: `calc-gucd.17_closed`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
