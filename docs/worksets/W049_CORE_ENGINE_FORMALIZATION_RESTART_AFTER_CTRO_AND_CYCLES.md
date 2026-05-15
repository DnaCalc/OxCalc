# W049 Core Engine Formalization Restart After CTRO And Cycles

Status: `pre_planning`

Parent predecessors: `W047` (Calc-Time Rebinding Overlay implementation core) and `W048` (circular dependency calculation processing)

Parent epic: TBD (allocated when W049 is activated)

## 1. Purpose

W049 will resume formal verification work on the calculation engine **after** the CTRO phase has landed in the implementation core under W047 and circular-dependency behavior has been grounded under W048. The aim is to formalize **around** a single coherent implementation, not in parallel to it.

W046's failure mode was producing formal artifacts that did not constrain the implementation: record-projection Lean theorems, two-step `MaxTransitions` TLA smoke models, fact-list checkers that pass on empty inputs, evidence rows that point to predecessor-workset artifacts rather than re-validating, and unbound test-run roots. W049 inherits the explicit obligation to do better.

W049 is in a deliberate **pre-planning** state. The punch-list inputs in §2 set the constraints. The actual scope, beads, exit gates, and evidence policy are not yet decided and will be planned after W047 closes and W048 cycle semantics are stable enough to formalize. This document is pre-planning background only; do not infer a bead path or commit to artifacts from it.


## 2. Pre-Planning Background

W049 inherits a critical fresh-eyes review of the W046 formalization effort, conducted 2026-05-09 across four parallel agent passes covering: spec coherence over the nine new W046 packets and the workset doc; formal-artifact soundness across the four W046 Lean files, three TLA models with smoke configs, and the new Python checker; bead-status integrity vs. on-disk evidence for nineteen `calc-gucd*` beads; and impl/doc-edit coherence across the modified `treecalc.rs`, the two showcase HTMLs, and the modified W046 docs. The findings establish what W049 must avoid replicating and what it must actually formalize.

### 2.1 Severity 1 — real defects in W046 formalization (must not be replicated)

1. **Successor-bead label collision between W046 and W047.** W046 closure-audit packet, packet README, workset doc, `closure_audit.json`, and the spine showcase HTML all map `calc-aylq.1` → "Rust Tarjan and topological queue line proof", `.2` → sidecar enrichment, `.3` → dynamic publication, `.4` → readiness gate. W047 workset doc (and `IN_PROGRESS_FEATURE_WORKLIST.md`) say `calc-aylq.1` = historical no-loss CTRO sweep, with the four W046 obligations re-numbered to `.5`–`.8`. Under the post-2026-05-10 W047 scope reset, the W046 formal/checker/sidecar/readiness obligations transfer to **W049**, not W047. The W046 closure docs need re-pointing to W049 once W049's epic id is allocated.

2. **W046 closure-audit packet mislabels the W047 epic.** Names `calc-aylq` "W047 core engine semantic proof deepening successor"; actual W047 name is "Calc-Time Rebinding Overlay Design Sweep" — drafted before W047 scope was finalized as CTRO and never re-labeled.

3. **Proof-carrying trace checker exercises the rebind path only vacuously.** The fixture fed to `scripts/check-w046-proof-carrying-trace.py` is `tc_local_rebind_after_rename_001/result.json` (the **pre**-edit publish run, `requires_rebind: false`, `result_state: "published"`). The post-edit run, where the rebind actually fires and rejects, lives at `…/post_edit/result.json` and is not checked. Consequence: the `rebind_nodes_reject_no_publish` failure rule and the `RebindRejectRefines` Lean theorem have **zero** rebind-required-and-rejected evidence anywhere in the validated set. The bridge calls `rebind_reject_refines_kernel` "checked"; the closure audit reports "passed; 5 artifacts, 0 failures" without flagging this.

4. **`LetLambdaPublishFacts := ChainPublishFacts`.** `formal/lean/OxCalc/CoreEngine/W046RustRefinementBridge.lean:62` literally aliases the LET/LAMBDA fact record to the chain-publish record, then `let_lambda_publish_refines_kernel` (line 115) reuses the chain proof. The spec table lists this as a distinct theorem; in Lean it is `def x := y; theorem t_x := t_y`.

5. **Integrated-kernel state machine in Lean is dead code.** `W046IntegratedSemanticKernel.lean` lines 65–134 define `KernelStep`, `StepPre`, `StepPost`, `LegalStep`, and `InitialKernelState`. **No theorem references any of them.** The "cross-phase state machine" promised in the README is defined and abandoned. All theorems are projections of the conjunctively-defined `IntegratedKernelInvariant` — record-projection tautologies.

6. **`CycleGroupSupported` is satisfied by any 2+ element list.** `W046FiniteGraphDataflowOrder.lean:62-63`: `group.members.length > 1 ∨ ∃ node, group.members = [node] ∧ HasSelfLoop edges node`. Multi-member groups are not required to contain edges or to be SCCs. `[10, 11]` with no edges passes. `two_node_scc_group_supported` (line 263) is discharged by `simp` because `length = 2 > 1` is definitional. Spec section 4 advertises five Lean shapes; `fanout_rebind` is in the spec table but **zero matches** in the Lean file (TLA only).

7. **Python checker silently degrades on missing inputs and unconditionally records "facts".** Lines 61–73 fail-record then fall back to `graph = {"edges": [], …}` and `closure_records = []`. Lines 98/99/106/137 then `facts.append(...)` unconditionally after loops that did nothing. The `facts` list is "checks attempted", not "checks that constrain anything". Several headline facts are vacuously satisfiable.

8. **TLA models are degenerately small — most are smoke-only.** `CoreEngineW046DependencyGraph` reports 3 states / 1 distinct / depth 2. `FiniteGraphDataflowOrder` reports 11 / 6 / depth 2 across five named shapes (cannot be exploring the shapes). `OxfmlEffectBoundary` similar. `MaxTransitions = 2` in several configs. Several invariants (e.g., `ClosureCoversExpectedDependents`) literally restate the assignment the action just made — **cannot fail by construction**. The recalc-tracker model (77,096 states, depth 7) is the one substantive TLA model; everything else is a smoke check.

### 2.2 Severity 2 — scope inflation; titles oversell what artifacts deliver

9. **Rust "refinement bridge" change is purely test code.** `git diff HEAD -- src/oxcalc-core/src/treecalc.rs` is +119 lines, all inside `mod tests`. No production-code change. Helper `assert_w046_refinement_bridge_facts` has dead `Rejected`/`VerifiedClean` branches (no caller exercises them). The new test is byte-for-byte identical to `local_treecalc_engine_publishes_local_formula_results` except for a one-line append.

10. **OxFml seam packet and TraceCalc refinement packet bind, they don't validate.** `effect_boundary_binding_register.json`: 7 of 8 rows are `evidence_root_existing` — pointers to W035/W037 artifacts, no new validation. `kernel_binding_register.json`: every `matched_refinement_surface` row points at the same W036 differential JSON. W046 cites W036; it does not re-check.

11. **`finite_graph_binding_register.json` `two_node_scc` row points to a fixture, not a run.** `evidence_roots: ["docs/test-corpus/.../tc_cycle_region_reject_001.json"]` is the **input scenario JSON**, not a `result.json`. Other shapes cite real runs.

12. **`FormatDisplaySeparated` and `HandlerLawModel` are vacuous.** `W046OxfmlEffectBoundary.lean:110-112`: `formatDelta ≠ displayDelta` is a constructor-distinctness tautology, true for every `SeamRun`. `HandlerLawModel` is verified only on a sample whose antecedents are empty (no `formatDelta`, no reject effect) — `simp` discharges by absence.

13. **Indirect-explorer evidence root is unbound.** `docs/test-runs/core-engine/tracecalc-reference-machine/w046-indirect-explorer-tracecalc-001/` has 30+ scenarios plus the showcase HTML at `docs/showcase/oxcalc_w046_indirect_recalc_phase_explorer.html`. **No bead, binding register, or audit cites it.** Either supposed-to-be-bound and missed, or surplus.

14. **Coverage ledger row count is off.** `W046_PROOF_SERVICE_…COVERAGE_LEDGER.md` Section 2 says `rows with replay/checker artifacts = 11`, but the recalc-tracker row's own blocker text admits no Rust bridge **and** no checker artifact. It should not be in the count.

### 2.3 Severity 3 — state-tracking staleness (mostly mechanical fixes)

15. **`formal/README.md:160-162` lags the workset.** Says `execution_state: calc-gucd.11_closure_audit_ready` while everywhere else says `…_validated`. Also keeps `scope_partial`/`target_partial` while workset/packet README publish `scope_complete`/`target_complete`.

16. **`W046_ENGINE_STATE_TRANSITION_CATALOG.md:182` status footer is stale.** Says `execution_state: calc-gucd.1_closed`. Body sections 6.7–6.14 just added in this round document results through `.11`.

17. **Closure declared before final validations ran.** README `Current Status` set to `calc-gucd.11_closure_audit_validated` while the closure-audit packet still records that `check-worksets.ps1`, `br dep cycles --json`, and `git diff --check` are "to be rerun after final bead close" — i.e. status declared validated before final validations ran.

### 2.4 Severity 4 — terminology drift

18. **"candidate" / "accepted candidate" / "commit" / "publication"** used interchangeably across packets without a shared definition. `AcceptedCandidateResult` precedes `PublicationBundle` in the OxFml packet; `candidateProduced` in the integrated kernel; `candidate_result` schema field in the proof-carrying trace packet; "candidate/commit" pairing in the OxFml docs. Three or four near-synonyms in active use.

19. **"refinement" / "binding" / "bridge" / "kernel match"** all used loosely. None of the packets define refinement in a technical sense (forward simulation / observation refinement). The Lean predicate `PublishedRefinesIntegratedKernel` is just record-equality conjunction — material implication, not refinement. The word "refinement" is doing rhetorical work it cannot cash.

### 2.5 Honest positives to preserve

- No `sorry` / `admit` / `axiom` in any W046 Lean file — tactics are mechanical, not hand-waved.
- The recalc-tracker TLA model (77k states, depth 7) is genuine.
- The Python checker is a real 315-line program (defects above are about coverage and silent-degrade behaviour, not fakery).
- W046 closures consistently avoid "production ready" / Stage-2 / C5 / operated-service / release promotion.
- The W046 showcase HTMLs are honest about the INDIRECT-rebind boundary — no false animation of dynamic-target switch.
- All packets named in the W046 packet README index exist on disk.

### 2.6 Working theory for W049

W049 will be planned around the following operating principles, derived from the punch list above:

1. **Reject record-projection "proofs."** A Lean theorem must be defined separately from the predicate it proves and must constrain the predicate non-vacuously. Avoid the `Invariant := A ∧ B ∧ C` followed by `theorem invariant_implies_a (h : Invariant) : A := h.1` pattern.
2. **Reject smoke TLA models.** No `MaxTransitions ≤ 2` configurations. No invariants that restate the action's own assignments. State spaces must be large enough that a violation could plausibly surface; if that is impossible at the chosen modeling level, abandon the model rather than ship a tiny one.
3. **Reject silent-degrade checkers.** A checker fed a missing input must fail loudly, not fall back to empty graphs and then unconditionally append "fact" labels. Distinguish "tested and clean" from "absent and unchecked".
4. **Treat the W047 implementation core as authoritative.** Formal models refine the landed Rust behaviour, not an idealized parallel description. If a model and the implementation disagree, the implementation defines truth and the model is wrong, until proven otherwise.
5. **Pick a small number of properties that matter and discharge them fully.** W046 produced ~17 packets and four Lean files for limited net constraint. Wide shallow coverage is inferior to a focused set of properties whose proofs actually constrain the implementation.
6. **Ship a glossary first.** Define `candidate / accepted candidate / commit / publication` and `refinement / binding / bridge / kernel match` once, with citations into the implementation, before any new spec packet is written under W049.
7. **Bind every claimed evidence root.** No unbound test-run dirs. If something exists on disk and is not cited by a bead or binding register, either bind it or delete it. The `w046-indirect-explorer-*` roots are the cleanest existing example.
8. **Re-route W046 successor labels.** The four obligations W046 routed to `calc-aylq.1`–`.4` (Rust Tarjan / sidecars / dynamic publication / readiness gate) transfer to the W049 epic. W046 closure docs need re-pointing once W049's epic id is allocated.
9. **Distinguish "checked at this bounded scope" from "proved."** W046 packet authors usually did this in the JSON ledgers but inflated it in titles and packet headlines. W049 should hold the same standard in titles too.

### 2.7 Open scoping questions for later planning

These are deferred until W047 closes and we plan W049 in detail:

- Which W046 punch-list items get corrected, which are accepted as residuals, and which are dropped entirely?
- Which subset of the nine W047 formal obligations (W047 §8) does W049 actually take on, and at what depth?
- Does W049 retain Lean and TLA both, or pick one? Does it introduce a new tooling axis (property-based testing against the implementation core, executable TLA, model-based test generation, refinement-checker tooling)?
- What is the bead structure — depth-first on a few properties, or breadth-first across the spine?
- How are the unbound `w046-indirect-explorer-*` evidence roots re-bound or retired?
- Which terminology drifts get a glossary entry, and where does that glossary live?
- What gates does W049 administer for pack / C5 / operated-service / release promotion, and how do those gates interact with the W047 implementation core?
- Does W049 retire any of the W046 Lean/TLA artifacts outright (dead-code state machine, vacuous handler-law model, two-node-SCC-by-list-length lemma, etc.) rather than carrying them forward?

### 2.8 Inherited W046 Successor Obligations (intended W049 scope)

W046 routed four deep-proof obligations forward as successor beads `calc-aylq.1`–`.4`. The W046 closure-audit packet (`docs/spec/core-engine/w046-formalization/W046_CLOSURE_AUDIT_SEMANTIC_SPINE_COVERAGE_AND_SUCCESSOR_ROUTING.md` §5) labels the `calc-aylq` epic as a W047 successor; under the post-2026-05-10 W047 scope reset these obligations transfer to W049. To keep the carry-forward from dangling between worksets, the four obligations are recorded here as **intended W049 scope** rather than as a label-re-pointing footnote. This section is the authoritative record that the obligations belong to W049.

Authoritative obligation set (from the W046 closure audit §5):

| W046 successor bead | Obligation | What it actually requires |
| --- | --- | --- |
| `calc-aylq.1` | Rust Tarjan and topological queue line proof | a real proof — not a record-projection tautology (see §2.1 item 5) — that the implementation's Tarjan SCC decomposition and topological evaluation queue in `treecalc.rs` produce a correct, deterministic order; must constrain the landed Rust per §2.6 principle 4 |
| `calc-aylq.2` | native proof-carrying trace sidecar enrichment | enrich the proof-carrying trace sidecar so it carries the facts a checker needs, and exercise it on the rebind-required-and-rejected path that W046's checker missed entirely (§2.1 item 3) — not the vacuous pre-edit publish run |
| `calc-aylq.3` | dynamic dependency positive publication refinement | a non-vacuous refinement statement for dynamic-dependency positive publication (the CTRO positive-publication path landed under W047); must be a technical refinement (forward simulation / observation refinement), not material implication (§2.4 items 19, §2.6 principle 1) |
| `calc-aylq.4` | semantic pack and operated-service readiness gate | the readiness-gate assessment for pack-grade replay and operated-service promotion — explicitly **not** a promotion claim; W049 administers this gate and states exact evidence consequence rather than asserting readiness |

Intended-scope rule for W049 planning:

1. W049's eventual bead path **must** account for all four obligations — either by taking them on (re-numbered under the W049 epic when its id is allocated), or by explicitly dropping/deferring one with recorded rationale under the §2.6 admit/correct/drop discipline. None may be silently lost.
2. The §2.7 open question "which subset of the W047 formal obligations does W049 take on" governs *depth and sequencing* of these four, not whether they are tracked at all.
3. When W049 is activated and its epic id is allocated, the W046 closure docs (`W046_CLOSURE_AUDIT_SEMANTIC_SPINE_COVERAGE_AND_SUCCESSOR_ROUTING.md` §5, the W046 packet README, `closure_audit.json`, and the spine showcase HTML) are re-pointed from the mislabelled `calc-aylq` → W047 successor to the W049 epic. Until then, this section holds the truth.
4. `calc-aylq.1`–`.3` are formalization obligations and compose with the §2.6 working theory (no record-projection proofs, no smoke models, no silent-degrade checkers, treat the W047 implementation core as authoritative). `calc-aylq.4` is a gate-administration obligation and composes with the §2.7 open question on how W049 gates pack / C5 / operated-service / release promotion.

This section discharges the cross-workset carry-forward: the "remaining W046 bead" set is now contained in W049 as recorded intended scope. It does not change W049's `pre_planning` status, does not allocate an epic id, and does not commit a bead path — scope, beads, exit gates, and evidence policy remain deferred per §1 and §2.7.

## 3. Relationship To W048 Cycle Work

W049 deliberately depends on W048 rather than absorbing it. W048 owns circular dependency calculation processing: Excel probes, structural-cycle behavior, CTRO-created cycle behavior, cycle release/re-entry, downstream invalidation, no-publication policy, and trace facts for cycle provenance.

W049 should formalize cycle behavior only after W048 has produced an implementation-facing behavioral target. If W048 leaves a cycle question open, W049 must either exclude it explicitly or mark the corresponding formal/checker lane blocked.

## 4. Post-W050 Formalization Intake

The W050 closure pass routes formalization-shaped remainder here rather than
keeping it as W050 process debt. W049 should treat these as inputs to its
eventual planning pass, not as already allocated beads:

1. The post-W050 runtime seam vocabulary: candidate, accepted candidate,
   commit, publication, prepared formula package, plan template,
   hole-binding identity, formal reference, formal input binding, reject,
   runtime effect, and derivation trace.
2. Non-vacuous invariants over the Stage-1 sequential engine: dependency
   graph construction, SCC/cycle classification, invalidation closure,
   rebind requirement, evaluation order, working-value reads, reject/no-publish
   behavior, and single-publisher commit authority.
3. Refinement/equivalence statements tying TreeCalc optimized runs,
   TraceCalc/reference traces, OxFml runtime packages, and checked replay
   artifacts to the same observable calculation behavior.
4. Proof-carrying or checker sidecars for real paths, especially rebind
   required-and-rejected paths, dynamic dependency publication/release paths,
   and derivation trace package/template/hole identity.
5. Bounded-memory and cache-retention properties after W054: deterministic
   eviction order, pinned-epoch safety, replay-visible retention decisions,
   and the interaction with per-edge value cache and subscription/topic
   envelope state.

Non-workset future ideas such as virtualized arrays beyond Excel-compatible
sheet/reference behavior, generic external rich data producers, and queryable
rich objects are explicitly not W049 formalization intake. They are parked in
`docs/spec/core-engine/CORE_ENGINE_FUTURE_IDEAS_RICH_AND_VIRTUAL_DATA.md`
until promoted by an explicit product decision.

## 5. Status Surface

- execution_state: `pre_planning`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- prerequisites: W047 closure (CTRO phase landed in implementation core) and W048 cycle semantics stabilized enough to formalize
- bead_path: not yet specified - W049 epic id and bead structure to be allocated when W049 is activated
- exit_gate: not yet specified
- evidence_policy: not yet specified
- predecessor_obligation_inheritance: see §2.6 working theory for the rules under which W046 punch-list items will be admitted, corrected, or dropped at planning time; see §2.8 for the four inherited W046 successor obligations (`calc-aylq.1`–`.4`) recorded as intended W049 scope
- w048_dependency: circular dependency calculation processing remains W048-owned input material
