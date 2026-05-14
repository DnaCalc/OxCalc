# W050 OxCalc/OxFml Formula Authority Rework

Status: `open_bead_execution`

Parent predecessor: `W048` single-host scoped circular-reference closure

Parent epic: `calc-cwpl`

Activation review: 2026-05-14. W050 has been reviewed against the
current Rust tree, the current OxFml inbound observation ledger, and
Foundation doctrine. The live execution graph is now expanded in
`.beads/`; this workset packet remains the scope and gate authority, not
a second execution tracker.

## 1. Purpose

W050 is the umbrella workset for bringing the OxCalc + OxFml + LET/LAMBDA calculation model into alignment with current best thinking. It is the large, disruptive rework that encapsulates the full job of getting the core engine model in line with the design baseline in §10 and the improvement moves in §11.

The workset covers:

1. removal of the OxCalc-local formula AST and source-construction code surfaces (the original W050 first-call concern);
2. replacement of the per-formula host-packet seam with a session-shaped OxCalc/OxFml protocol built on prepared callables;
3. introduction of the layered identity discipline that lets structurally equivalent formulas share compiled plans (plan templates with `shape_key`, `dispatch_skeleton_key`, `plan_template_key`);
4. the hole-type taxonomy that admits sparse and rich-value holes from day one, so value virtualisation can land additively in successor work without retrofit;
5. wiring external invalidation (RTD, registered externals, host-supplied watchers) as a first-class subscription / topic-envelope discipline;
6. the correctness-floor decisions — numerical reduction policy and error algebra — as profile-governed semantic state;
7. identity scaffolding for differential evaluation, derivation trace, and push/pull duality as performance/observability layers.

W050 is intentionally large. It is planned and executed in lanes (§5) that can run partly in parallel and that all roll up to the comprehensive exit gate in §8. The workset is `open_planning` at lane level; individual lanes may advance independently under the bead structure in §6, but the workset itself closes only when every lane meets its share of the exit gate.

Working principle (made comprehensive):

1. OxFml owns every formula concern: parsing, text understanding, binding, function/operator evaluation, array/spill values, coercion, volatile behaviour, returned-value surfaces, callable values, LET/LAMBDA semantics.
2. OxCalc owns multi-node calculation-engine semantics around OxFml-produced single-node results: structural graph state, dependency derivation from bind output, invalidation closure, runtime overlays, cycle policy, candidate/publication state, replay evidence, host/provider plumbing, subscription registry, topic envelopes.
3. OxCalc must not parse, construct, rewrite, or semantically interpret Excel formula text. Formula sources are opaque strings tagged with stable identity; everything else is OxFml output.
4. The OxCalc/OxFml seam is a *session* whose lifecycle spans a recalc wave, not a sequence of unrelated per-formula calls. Compiled artefacts are reused across cells, across waves, and across structurally equivalent formulas via layered identity.
5. References are *outputs* of OxFml bind, not pre-resolved inputs OxCalc constructs. The dependency graph is derived from those bind outputs against OxCalc's structural truth.
6. Identity discipline is committed up front for plan templates, capability-set holes, topic envelopes, and rich-value capability vocabularies — even where the concrete implementation is deferred to successor work. Retrofitting identity is the failure mode the spec is designed to prevent.

## 2. Triggering Observations

The original W050 triggering observation (the dynamic-array example) was one of several catalysts. The full catalyst set:

**Catalyst 1 — Dynamic arrays the engine cannot honestly carry.** A simple dynamic-array example revealed that OxCalc would need to parse, construct, evaluate, scalarize, or understand the formula source text to handle it:

```text
A: =RANDARRAY(5,5)
B: =A+1
C: =SUM(A, B)
D: =INDEX(A, 2, 2)
```

The intended architecture is that OxFml owns formula text and construction; OxCalc consumes value/effect/dependency facts opaquely. A temporary boundary test representing `RANDARRAY(5,5)` as an OxCalc `ShapeTopology` carrier was removed because it implied product-level dynamic-array plumbing in OxCalc.

**Catalyst 2 — The call-trace red flag.** A walk through the execution path for a trivial two-formula model (`A: =SIN(0.5)`, `B: =ABS(A)`) revealed that OxCalc makes one independent `RuntimeEnvironment::new().execute(...)` call per formula, with the entire OxFml pipeline (parse → red → bind → semantic plan → evaluate) re-walked from scratch each time. OxCalc constructs the formula source text itself (from a local `TreeFormula` AST), then re-flattens its already-resolved references into a synthetic A1 cell fixture so OxFml's bind can re-resolve them. The seam shape is upside-down.

**Catalyst 3 — Lambda optimisations are stranded.** OxFml's prepared-call machinery for LAMBDA/LET — lexical capture, prepared callable carrier, helper-bound invocation, higher-order helpers `MAP` / `REDUCE` / `SCAN` / `BYROW` / `BYCOL` / `MAKEARRAY` — already implements "compile once, invoke many with different bindings". This is also the dominant pattern of ordinary cell recalc (filled-down columns of structurally identical formulas with different references). The optimisation lives in a special form rather than at the general consumer surface.

**Catalyst 4 — Structural formula equivalence.** Formulas that differ only at leaf positions (e.g. `=MyFunction(1)+OtherFunction(2)`, `=MyFunction(A)+OtherFunction(3)`, `=MyFunction(B)+OtherFunction(4)`) cannot share their compiled plans under the current keying. The expensive artefact — the semantic plan — is the one not being reused, even when every structural decision the compiler made is identical.

**Catalyst 5 — External / RTD invalidation is undermodelled.** Foundation names three invalidation classes (`Standard`, `Volatile`, `ExternallyInvalidated`) and a profile-governed `StreamSemanticsVersion`, but the current engine has no explicit Subscription Registry, no topic-envelope discipline, and therefore no replay-deterministic RTD path. External signals are handled per-formula at evaluation time rather than as typed dirty seeds.

**Catalyst 6 — Sparse and rich values are blocked by the value-universe shape.** Whole-column references force materialisation of millions of values when only thousands are defined. Virtual regions from `FILTER` / `INDEX` / `CHOOSEROWS` have no type-system home. External data sources have no admission seam. The path to all of these runs through a single architectural decision — capability-set hole identity — which must be admitted now even if exercised later.

**Catalyst 7 — Determinism floors are informal.** Numerical reduction policy (summation order: Kahan vs pairwise vs sequential) and error algebra (`#NULL!` > `#DIV/0!` > `#VALUE!` precedence) are observable Excel behaviour. They are currently implementation choices rather than profile-governed semantic selectors. Replay conformance cannot be honest without them.

**Catalyst 8 — Layered identity not yet committed.** Foundation's overlay-reuse key tuple `(snapshot_epoch, wave_id, formula_stable_id, formula_token, bind_hash, profile_version)` is the right vocabulary, but the current engine has no `shape_key`, `dispatch_skeleton_key`, `plan_template_key`, or capability-set member. Adding these after artefacts ship is the expensive failure mode.

## 3. Scope Inventory

W050 is the umbrella for the following changes. Each is detailed in §10 (design baseline) or §11 (improvement moves) and lands under the lane structure in §5.

**A. Seam Reshape (§§10.3–10.5).** OxCalc Calculation Repository as the persistent structural truth surface. OxFml Recalc Session as the only consumer entry surface during a wave. `ensure_prepared(formula_stable_id, …) → PreparedCallable` and `invoke(call_site, bindings) → InvocationOutcome` as the only operations. Six-phase wave lifecycle.

**B. Removal of Wrong-Shape Code (§10.10).** `TreeFormula` AST and `translate_formula` / `TranslationState`. `MinimalUpstreamHostPacket` and the `Minimal*` family. Synthetic A1 cell-fixture flattening. Per-formula `RuntimeEnvironment::new().execute(...)` pattern in `evaluate_via_oxfml`. `formula_allows_lazy_residual_publication` (subsumed by general lazy-control-form handling).

**C. Plan-Template Identity Layer (§10.6).** `shape_key`, `dispatch_skeleton_key`, `plan_template_key`. `PlanTemplate` as a first-class artefact; `PreparedCallable = (PlanTemplate, HoleBindings)`.

**D. Hole-Type Taxonomy (§10.7).** Default taxonomy: `ValueHole`, `RefOrValueHole`, `CallableHole`, `ShapeSensitiveHole`, `SparseRangeHole`, `RichValueHole`. Wide-by-default policy; narrowing reserved for evidence-gated optimisation. `ArgPreparationProfile` is the architectural line between widenable and non-widenable.

**E. External Invalidation Discipline (§10.9).** Persistent Subscription Registry on the Repository. Topic Envelopes per subscribed topic, governed by `StreamSemanticsVersion`. Push invalidation produces typed dirty seeds; pull evaluation consults the envelope. RTD-driven recalc reproduces deterministically under replay.

**F. Rich-Value Future Direction (§10.12).** `RichValueHole(required_capability_set)` admitted from day one with initial vocabulary `Indexable + Enumerable + Shaped + Materialisable`. Capability vocabulary additively extensible. Concrete kernel work deferred; identity discipline committed now.

**G. Correctness-Floor Additions (§11.1 Move C).** `NumericalReductionPolicy` and `ErrorAlgebra` as profile-governed semantic selectors. Profile claims must declare both; replay validates against them.

**H. Performance / Observability Layer (§11.1 Moves A+, B, F).** Push/pull duality with visibility-bounded scheduling. Differential evaluation through stable per-edge value cache. Derivation trace as first-class observable surface.

**I. Scaffolded-But-Not-Implemented (§11.2).** Sparse range readers as the first concrete rich-value class (Move D+). Sensitivity / `Differentiable` capability (Move H). Both admitted at the identity/vocabulary level inside W050; concrete kernel implementations are successor work.

**J. Explicitly Deferred (§11.3).** Speculative parallel evaluation behind single-publisher commit (Move E, Stage 2 work). Bounded-memory pinned-epoch GC (Move G, requires post-W050 measurement infrastructure).

## 4. Current Code Surfaces To Audit

| Surface | Current role | Rework lane |
| --- | --- | --- |
| `TreeFormula` | Opaque OxFml source text plus explicit reference/evaluator-fact carriers after A3; no semantic expression variants remain on this type | Lane A/B — keep as boundary carriage until a canonical prepared-callable input replaces current V1 compatibility inputs |
| `FixtureFormulaAst` / `FixtureFormulaBinaryOp` | Quarantined fixture/scale-runner AST used only to render legacy checked-in fixtures and procedural scale corpora into opaque `TreeFormula` values | Lane A — delete or archive when fixture corpora use authored opaque OxFml sources directly |
| `TreeReference::*` | Dependency-carrier projection and fixture binding | Lane B — replace with bind-output reference handles |
| `translate_formula` / `TranslationState` | Retired AST-to-source-text lowering path; A3 replaces it with `project_opaque_formula` over explicit carriers | Lane A — keep retired; remove any future regressions |
| `MinimalUpstreamHostPacket` and `Minimal*` family | Deterministic upstream-host fixture/scaffolding packet surface; no longer the TreeCalc production invocation path after B7 | Lane B/A — keep fixture-only until canonical session intake covers the same evidence surface, then delete or quarantine |
| `evaluate_with_oxfml_session` / `invoke_prepared_formula_via_session` | Current V1 TreeCalc production bridge into `OxfmlRecalcSessionDriver::invoke`; still carries synthetic A1 compatibility inputs | Lane B — replace compatibility inputs with canonical prepared-callable invocation transport |
| `synthetic_cell_target` / `synthetic_cell_row` | Synthetic A1 address generation for current V1 cell-value / defined-name compatibility | Lane B/A — delete once CALC-002 reference/input transport lands |
| `TreeFormula.lazy_residual_publication` | Current V1 compatibility flag set by fixture quarantine for lazy-control residual cases | Lane B — replace with OxFml callable/control-form outcome metadata |
| `Stage1RecalcTracker` | Per-node state machine | Lane B — preserve, recompose plumbing |
| `TreeCalcCoordinator` | Single-publisher publication authority | Lane B — preserve, recompose plumbing |
| Overlay lifecycle / `OverlayKey` / `OverlayEntry` | Runtime overlay state | Lane B — preserve, extend to Subscription Registry |
| `DependencyGraph` / `InvalidationClosure` | Static + dynamic dependency state | Lane B — preserve, drive from bind output |
| `RuntimeEffect` / `RuntimeEffectFamily` | Runtime-derived effects carriage | Lane D — extend with `ExternallyInvalidated` family |
| W047/W048 CTRO and cycle fixtures | Many use structured TreeFormula carriers | Lane A — convert or quarantine |
| Profile selector vocabulary | `CycleSemantics`, `StreamSemanticsVersion` (named in Foundation) | Lane E — add `NumericalReductionPolicy`, `ErrorAlgebra` |
| Trace event taxonomy | Per-invocation trace events | Lane F — add derivation-trace columns, template-key columns |
| Per-edge value cache | (does not exist) | Lane F — new (Move B) |
| Subscription Registry / Topic Envelopes | (does not exist) | Lane D — new |
| `PlanTemplate` / `shape_key` / `dispatch_skeleton_key` / `plan_template_key` | (do not exist) | Lane C — new identity layer |
| Capability-set vocabulary for `RichValueHole` | (does not exist) | Lane G — new (identity only at W050 commit) |

### 4.1 A1 Active Inventory Snapshot

A1 search scope covers active source, active fixture inputs, scale/demo
generators, active showcase/slides docs, and upstream-host scaffolding.
Generated replay/test-run archives under `docs/test-runs/**` are retained
artifact outputs rather than active input surfaces; A6 should record them
only as allowed archive hits if the final search gate sees them.

| Exact path | Surface found | Classification | Follow-on bead |
| --- | --- | --- | --- |
| `src/oxcalc-core/src/formula.rs` | `TreeFormula`, `TreeFormulaReferenceCarrier`, `TreeReference`, `FixtureFormulaAst`, `FixtureFormulaBinaryOp`, `TreeFormulaCatalog` | `TreeFormula` is opaque source carriage plus explicit carriers; `TreeReference` is dependency/evaluator-fact projection; `FixtureFormulaAst` is fixture/test scaffolding. | `calc-cwpl.2`, `calc-cwpl.8.1`, `calc-cwpl.8.2`, `calc-cwpl.9.7` / CALC-002 |
| `src/oxcalc-core/src/treecalc.rs` | `prepare_oxfml_formula`, `project_opaque_formula`, `evaluate_with_oxfml_session`, `invoke_prepared_formula_via_session`, `synthetic_cell_row`, `synthetic_cell_target` | Production OxFml session bridge. Source text is passed through opaquely; carriers are projected to current V1 compatibility bindings. Synthetic A1 helpers are architectural residue pending CALC-002 reference/input transport. | `calc-cwpl.8.2`, `calc-cwpl.9.7`, `calc-cwpl.15.1` |
| `src/oxcalc-core/src/treecalc_fixture.rs` | `TreeCalcFixtureFormulaBinding.expression: FixtureFormulaAst` and fixture-to-catalog lowering | Fixture loader scaffolding for active TreeCalc fixture JSON. | `calc-cwpl.2`, `calc-cwpl.8.2` |
| `src/oxcalc-core/src/treecalc_scale.rs` | Procedural `FixtureFormulaAst` builders for scale/demo corpora | Scale/demo generator scaffolding; not production formula semantics. | `calc-cwpl.2`, `calc-cwpl.8.2` |
| `src/oxcalc-core/src/consumer.rs` | Unit-test `FixtureFormulaAst` / `TreeReference` constructions | Test scaffolding around the consumer facade. | `calc-cwpl.2`, `calc-cwpl.8.2` |
| `src/oxcalc-core/src/upstream_host.rs` | `MinimalUpstreamHostPacket` and `Minimal*` facts | Upstream-host fixture/scaffolding packet surface; no longer the TreeCalc production invocation path. | `calc-cwpl.9.7`, `calc-cwpl.15.1` |
| `src/oxcalc-core/src/upstream_host_fixture.rs` | Fixture loader for `MinimalUpstreamHostPacket` JSON | Upstream-host fixture scaffolding. | `calc-cwpl.9.7`, `calc-cwpl.15.1` |
| `src/oxcalc-core/tests/upstream_host_scaffolding.rs` | Test-local `MinimalUpstreamHostPacket` construction | Upstream-host scaffolding tests. | `calc-cwpl.9.7`, `calc-cwpl.15.1` |
| `docs/test-fixtures/core-engine/treecalc/README.md` and `docs/test-fixtures/core-engine/treecalc/MANIFEST.json` | Active TreeCalc fixture policy and manifest | Fixture inventory surface. | `calc-cwpl.2`, `calc-cwpl.8.2` |
| `docs/test-fixtures/core-engine/upstream-host/README.md` and `docs/test-fixtures/core-engine/upstream-host/MANIFEST.json` | Active upstream-host fixture docs/manifest | Fixture/scaffolding inventory surface. | `calc-cwpl.9.7`, `calc-cwpl.15.1` |
| `docs/slides/oxcalc_xyz_call_trace.html` | A4-repaired session-path wording | Active slide/doc wording now uses the session path and treats the minimal packet as fixture scaffolding. | `calc-cwpl.4` |
| `docs/showcase/oxcalc_w047_w048_core_engine_showcase.html` | A4-repaired `TreeFormulaCatalog` wording | Active W047/W048 showcase wording preserves graph/publication claims only. | `calc-cwpl.4` |
| `docs/showcase/oxcalc_w033_w045_formalization_showcase.html` | A4-repaired `TreeReference` and evaluator-seam wording | Active showcase wording now presents carrier projection and the session path rather than retired production names. | `calc-cwpl.4` |
| `docs/showcase/oxcalc_w033_w045_engine_formalization_storyboard.md` | A4-repaired evaluator-seam wording | Active storyboard wording now names the current session helper and residual gate. | `calc-cwpl.4` |
| `docs/showcase/oxcalc_w033_w045_engine_formalization_review_catalog.md` | A4-repaired carrier/catalog/session wording | Active review-catalog wording now separates opaque source carriage, carrier lowering, and evaluator-session adaptation. | `calc-cwpl.4` |
| `docs/worksets/W050_OXCALC_OXFML_FORMULA_AUTHORITY_REWORK.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`, `docs/spec/core-engine/CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md`, `docs/upstream/NOTES_FOR_OXFML.md`, `docs/IN_PROGRESS_FEATURE_WORKLIST.md` | Current W050/status references to the TreeCalc/OxFml seam | Active status/spec surfaces; keep boundary-accurate as successor beads land. | `calc-cwpl.16.4`, `calc-cwpl.8.2` |
| `docs/worksets/W026_TREECALC_OXFML_BIND_REFERENCE_AND_SEAM_INTAKE.md`, `docs/spec/core-engine/w033-formalization/W033_LET_LAMBDA_CARRIER_WITNESS_WIDENING.md`, `docs/spec/core-engine/w034-formalization/W034_RESIDUAL_OBLIGATION_AND_AUTHORITY_LEDGER.md`, `docs/spec/core-engine/w037-formalization/W037_DIRECT_OXFML_EVALUATOR_AND_LET_LAMBDA_SEAM_EVIDENCE.md`, `docs/spec/core-engine/w037-formalization/W037_OPTIMIZED_CORE_ENGINE_CONFORMANCE_IMPLEMENTATION_CLOSURE.md`, `docs/spec/core-engine/w046-formalization/W046_EVALUATION_ORDER_AND_WORKING_VALUE_READ_DISCIPLINE_MODEL.md`, `docs/spec/core-engine/w046-formalization/W046_OXFML_SEAM_LET_LAMBDA_FORMATTING_PUBLICATION_AND_CALLABLE_BOUNDARY_MODEL.md`, `docs/spec/core-engine/w046-formalization/W046_SEMANTIC_FRAGMENT_REVIEW_LEDGER.md`, `docs/spec/core-engine/w047-ctro/W047_IMPLEMENTATION_ROADMAP_AND_SUCCESSOR_GATES.md` | Historical spec references to the pre-W050 TreeCalc/OxFml seam | Historical evidence/spec archive. A6 should list allowed historical hits rather than treating them as active production claims. | `calc-cwpl.8.2` |

Active TreeCalc fixture input files with structured `FixtureFormulaAst`
variants (`Binary`, `FunctionCall`, `RawOxfml`, `Reference`, or
`Literal`) are:

```text
docs/test-fixtures/core-engine/treecalc/cases/tc_local_capability_sensitive_reject_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_dynamic_addition_auto_post_edit_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_dynamic_mixed_add_release_auto_post_edit_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_dynamic_reject_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_dynamic_release_reclassification_auto_post_edit_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_dynamic_release_reclassification_post_edit_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_dynamic_resolved_publish_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_dynamic_target_switch_downstream_publish_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_host_sensitive_reject_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_lambda_host_sensitive_reject_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_let_lambda_capture_publish_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_mixed_publish_then_post_edit_overlay_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_move_direct_target_rebind_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_post_edit_capability_sensitive_overlay_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_post_edit_host_sensitive_overlay_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_post_edit_shape_topology_overlay_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_publish_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_rebind_after_rename_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_recalc_after_constant_edit_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_recalc_chain_after_constant_edit_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_relative_sum_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_remove_direct_target_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_shape_topology_reject_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_sibling_offset_publish_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_tracecalc_accept_publish_equiv_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_tracecalc_multinode_dag_equiv_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_verified_clean_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_w034_higher_order_let_lambda_publish_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_local_w034_independent_order_equiv_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_w048_ctro_dynamic_release_reentry_downstream_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_w048_ctro_dynamic_self_cycle_reject_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_w048_excel_ctro_indirect_iterative_self_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_w048_excel_iter_fraction_precision_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_w048_excel_iter_three_node_order_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_w048_excel_iter_two_node_order_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_w048_structural_self_cycle_reject_001.json
docs/test-fixtures/core-engine/treecalc/cases/tc_w048_structural_two_node_cycle_reject_001.json
```

Active upstream-host fixture input files carrying `formula_text`,
`formula_channel_kind`, `cell_fixture`, or `defined_name_bindings` through
the `MinimalUpstreamHostPacket` scaffolding surface are:

```text
docs/test-fixtures/core-engine/upstream-host/cases/uh_info_directory_capture_001.json
docs/test-fixtures/core-engine/upstream-host/cases/uh_info_unsupported_query_001.json
docs/test-fixtures/core-engine/upstream-host/cases/uh_let_lambda_lexical_capture_eval_001.json
docs/test-fixtures/core-engine/upstream-host/cases/uh_returned_lambda_invocation_eval_001.json
docs/test-fixtures/core-engine/upstream-host/cases/uh_rtd_provider_error_001.json
docs/test-fixtures/core-engine/upstream-host/cases/uh_structured_column_sum_eval_001.json
docs/test-fixtures/core-engine/upstream-host/cases/uh_structured_data_multicol_sum_eval_001.json
docs/test-fixtures/core-engine/upstream-host/cases/uh_structured_headers_section_eval_001.json
docs/test-fixtures/core-engine/upstream-host/cases/uh_structured_reference_eval_001.json
docs/test-fixtures/core-engine/upstream-host/cases/uh_sum_defined_name_bind_001.json
docs/test-fixtures/core-engine/upstream-host/cases/uh_table_context_bind_001.json
docs/test-fixtures/core-engine/upstream-host/cases/uh_typed_cf_average_guard_001.json
docs/test-fixtures/core-engine/upstream-host/cases/uh_typed_cf_color_scale_guard_001.json
docs/test-fixtures/core-engine/upstream-host/cases/uh_typed_cf_data_bar_guard_001.json
docs/test-fixtures/core-engine/upstream-host/cases/uh_typed_cf_icon_set_guard_001.json
docs/test-fixtures/core-engine/upstream-host/cases/uh_typed_cf_top_rank_guard_001.json
```

## 5. Execution Lanes and Phasing

W050 decomposes into seven lanes. Lanes are independent enough to run partly in parallel; the dependency relations are explicit.

**Lane A — Removal of Wrong-Shape Code.** Delete the OxCalc-local formula AST, source-text construction, synthetic A1 flattening, and per-formula host-packet machinery. *Depends on:* Lane B sufficient to provide the replacement seam. *Precedes:* most fixture moves.

**Lane B — New Seam Implementation.** Build the Calculation Repository, the OxFml Recalc Session over `oxfml_core::consumer::runtime`, `ensure_prepared`, `invoke`, and the six-phase wave lifecycle. Wire reference handles from bind output to structural targets. *Depends on:* nothing in W050; depends on the frozen `OxFml_V1` consumer-facade contract. *Precedes:* Lane A, Lane C.

**Lane C — Plan-Template Identity Layer.** Implement `shape_key`, `dispatch_skeleton_key`, `plan_template_key`. Move `PreparedCallable` to `(PlanTemplate, HoleBindings)` shape. Implement the default hole-type taxonomy including `SparseRangeHole` and `RichValueHole` (the latter with empty capability vocabulary at W050 commit). *Depends on:* Lane B. *Precedes:* Lane F observability.

**Lane D — External Invalidation Discipline.** Implement the Subscription Registry, Topic Envelopes, and the typed dirty-seed pathway from external signals through to invalidation closure. Wire `StreamSemanticsVersion` as a profile selector. *Depends on:* Lane B. *Independent of:* Lane C.

**Lane E — Correctness Floor.** Add `NumericalReductionPolicy` and `ErrorAlgebra` as profile-governed semantic selectors. Define profile claims and replay-validation hooks. *Independent of:* Lanes A–D, but needs OxFunc cooperation for kernel-side reduction discipline. Requires a cross-repo handoff packet.

**Lane F — Performance and Observability Layer.** Differential evaluation (per-edge value cache), derivation trace, push/pull duality with visibility-bounded scheduling. *Depends on:* Lane B, Lane C, Lane D.

**Lane G — Forward-Direction Scaffolding.** Capability-set vocabulary admission for `RichValueHole`. No kernel work; pure identity discipline. *Depends on:* Lane C. *Independent of:* everything else.

**Phasing recommendation.** Wave 1 lands Lanes B and C concurrently (the new seam plus the identity layer); Lane A follows as soon as the new seam covers the existing test corpus. Wave 2 lands Lanes D and E in parallel. Wave 3 lands Lane F. Lane G can land in any wave; the cheapest moment is alongside Lane C since both touch the hole-type taxonomy.

Each lane carries its own bead chain under the parent epic `calc-cwpl` (§6) and its own corpus of acceptance tests. Lane completion is gated on the exit-gate clauses in §8 that apply to that lane. The workset itself remains `open_planning` until all lanes meet their share of the gate.

## 6. Bead Path

Parent epic: `calc-cwpl`. The original six beads are preserved and expanded to cover the umbrella scope. Beads are organised by lane.

| Lane | Bead | Purpose |
| --- | --- | --- |
| A | `calc-cwpl.A1` | inventory OxCalc formula-looking code paths (was `calc-cwpl.1`) |
| A | `calc-cwpl.A2` | repair TreeCalc fixture policy (was `calc-cwpl.2`) |
| A | `calc-cwpl.A3` | remove/quarantine local `TreeFormula` semantic AST (was `calc-cwpl.5`) |
| A | `calc-cwpl.A4` | repair W047/W048 showcase formula-boundary wording (was `calc-cwpl.4`) |
| B | `calc-cwpl.B1` | design the first-call OxFml name/reference binding protocol (was `calc-cwpl.6`) |
| B | `calc-cwpl.B2` | implement Calculation Repository |
| B | `calc-cwpl.B3` | implement Recalc Session (`ensure_prepared`, `invoke`) |
| B | `calc-cwpl.B4` | implement six-phase wave lifecycle |
| B | `calc-cwpl.B5` | wire reference handles from bind output to structural targets |
| B | `calc-cwpl.B6` | OxFml delegation / opaque result tests (was `calc-cwpl.3`) |
| B | `calc-cwpl.B7` | replace `evaluate_via_oxfml` / packet production path |
| B | `calc-cwpl.B8` | session corpus and replay evidence packet |
| B | `calc-cwpl.B9` | OxFml V1 compatibility and gap ledger |
| C | `calc-cwpl.C1` | identity layer: `shape_key`, `dispatch_skeleton_key`, `plan_template_key` |
| C | `calc-cwpl.C2` | `PlanTemplate` as first-class artefact |
| C | `calc-cwpl.C3` | hole-type taxonomy (default variants including `SparseRangeHole`, `RichValueHole`) |
| C | `calc-cwpl.C4` | bind-visible name-world invalidation extends to `ArgPreparationProfile` changes |
| D | `calc-cwpl.D1` | Subscription Registry on the Repository |
| D | `calc-cwpl.D2` | Topic Envelopes per topic; replay-deterministic ordering |
| D | `calc-cwpl.D3` | `StreamSemanticsVersion` profile selector wiring |
| D | `calc-cwpl.D4` | RTD-driven recalc replay-determinism corpus |
| E | `calc-cwpl.E1` | `NumericalReductionPolicy` profile selector spec |
| E | `calc-cwpl.E2` | `ErrorAlgebra` profile selector spec |
| E | `calc-cwpl.E3` | replay-validation hooks against both selectors |
| F | `calc-cwpl.F1` | per-edge value cache (differential evaluation) |
| F | `calc-cwpl.F2` | derivation trace as first-class outcome |
| F | `calc-cwpl.F3` | push/pull duality with visibility-bounded scheduling |
| G | `calc-cwpl.G1` | capability-set vocabulary admission (`Indexable`, `Enumerable`, `Shaped`, `Materialisable`) |
| G | `calc-cwpl.G2` | `RichValueHole(required_capability_set)` admitted to hole-type taxonomy |
| G | `calc-cwpl.G3` | trace/replay schema columns for capability-set identity |

Cross-repo handoff beads (filed against `docs/handoffs/HANDOFF_REGISTER.csv`):

| Bead | Handoff packet | Drives |
| --- | --- | --- |
| `calc-cwpl.H1` | `HANDOFF_CALC_002_OXFML_RECALC_SESSION_AND_PLAN_TEMPLATES.md` | Lanes B, C |
| `calc-cwpl.H2` | `HANDOFF_CALC_003_OXFML_NUMERICAL_REDUCTION_AND_ERROR_ALGEBRA.md` | Lane E (OxFunc cooperation required) |
| `calc-cwpl.H3` | `HANDOFF_CALC_004_OXFML_CAPABILITY_SET_HOLE_ADMISSION.md` | Lane G |

The original W050 bead numbering is mapped into the new lane structure above. The historical identifiers are preserved as references; the new identifiers are authoritative going forward.

### 6.1 Live `br` Rollout Map

The canonical lane names above are carried in `br` as `external_ref`
values. The live `br` ids are:

| Canonical lane | `br` id | Role |
| --- | --- | --- |
| `calc-cwpl` | `calc-cwpl` | parent epic |
| `calc-cwpl.R0` | `calc-cwpl.7` | activation review and rollout control bead |
| `calc-cwpl.A` | `calc-cwpl.8` | Lane A epic |
| `calc-cwpl.B` | `calc-cwpl.9` | Lane B epic |
| `calc-cwpl.C` | `calc-cwpl.10` | Lane C epic |
| `calc-cwpl.D` | `calc-cwpl.11` | Lane D epic |
| `calc-cwpl.E` | `calc-cwpl.12` | Lane E epic |
| `calc-cwpl.F` | `calc-cwpl.13` | Lane F epic |
| `calc-cwpl.G` | `calc-cwpl.14` | Lane G epic |
| `calc-cwpl.H` | `calc-cwpl.15` | handoff epic |
| `calc-cwpl.X` | `calc-cwpl.16` | cross-cutting spec/evidence/audit epic |

Leaf bead mapping uses the canonical `external_ref` values in `br`; the
current expanded graph has 64 total issues under the W050 parent set.
`br dep cycles` reports no dependency cycles. Current leaf readiness is
owned by `.beads/`, not this document.

### 6.2 Activation Review Findings

1. Reviewed inbound observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`
   now says ordinary downstream use should target
   `consumer::runtime`, `consumer::editor`, and `consumer::replay`;
   public `substrate::...` access is gone from OxFml's library surface.
   W050 must therefore consume `oxfml_core::consumer::runtime` and route
   missing prepared-callable / plan-template identity support through
   handoff packets, not OxCalc-local adapters.
2. Current OxFml code confirms the available consumer surface:
   `RuntimeEnvironment`, `RuntimeFormulaRequest`, `RuntimeFormulaResult`,
   and `RuntimeSessionFacade`. `RuntimeSessionFacade` has managed-session
   operations (`open_managed_session`, `execute_managed`,
   `commit_managed`, termination, diagnostics), but it does not yet expose
   W050's exact `ensure_prepared` / `invoke` / `PlanTemplate` names. That
   is a CALC-002 handoff pressure, not a reason to build a long-term
   OxCalc wrapper around non-consumer internals.
3. Current OxCalc code still contains W050 target surfaces:
   `synthetic_cell_target`, `synthetic_cell_row`, the
   `FixtureFormulaAst` fixture quarantine, and fixture/scaffolding
   `Minimal*` packet surfaces. After A3, `TreeFormula` no longer has
   semantic variants, and `translate_formula`, `TranslationState`, and
   `formula_allows_lazy_residual_publication` are no longer source-code
   surfaces. After B7, `src/oxcalc-core/src/treecalc.rs` no longer imports
   or constructs `MinimalUpstreamHostPacket` and no longer carries the
   `evaluate_via_oxfml` / `build_upstream_host_packet` production path.
4. The expanded bead graph adds explicit work for the full W050 exit
   gate: lane epics A-G, handoff lane H, cross-cutting spec/evidence lane
   X, and leaf beads for repository/session substrate, identity keys,
   hole taxonomy, stream discipline, correctness-floor selectors,
   observability, rich-value admission, handoff registration, evidence
   roots, and final audit.
5. B3 live uptake: `src/oxcalc-core/src/oxfml_session.rs` now carries the
   OxCalc-side session driver over public
   `oxfml_core::consumer::runtime` types. `ensure_prepared` maps to
   `RuntimeSessionFacade::open_managed_session`; current full-result
   `invoke` maps to `RuntimeSessionFacade::execute`; current V1 managed
   commit is exercised as compatibility evidence but leaves a CALC-002
   gap because it does not carry every `RuntimeFormulaResult` surface
   consumed by the TreeCalc coordinator path.
6. B4 live uptake: `src/oxcalc-core/src/recalc_wave.rs` now carries the
   six-phase W050 lifecycle over the session driver. It enforces phase
   order, records session prepare/invoke phases, and records exactly one
   OxCalc coordinator decision before close/replay capture. This is an
   ordering and authority guard; B5 still owns replacement of synthetic
   A1/name flattening with OxFml bind-output reference handles.
7. B5 live uptake: `DependencyDescriptor` now carries an optional
   `source_reference_handle`; `oxfml_dependency_descriptors` derives
   handles from `BoundFormula.normalized_references`,
   `BoundFormula.unresolved_references`, and runtime residual facts. The
   graph preserves these handles while keeping OxCalc structural targets
   as the dependency edge truth. CALC-002 still owns the fully canonical
   formal-reference set that can retire migration carriers.
8. B7 live uptake: `LocalTreeCalcEngine` now invokes OxFml through
   `OxfmlRecalcSessionDriver` directly. It assembles
   `RuntimeEnvironment` and `RuntimeFormulaRequest` from
   `PreparedOxfmlFormula`, current working values, and TreeCalc-local
   deterministic provider shims for host-sensitive and dynamic-potential
   residuals. `MinimalUpstreamHostPacket` is now fixture/scaffolding only
   in `upstream_host*`, integration tests, and runner evidence. Synthetic
   A1 compatibility inputs remain a Lane B / CALC-002 cleanup target.
9. A3 live uptake: `TreeFormula` now carries opaque OxFml `source_text`,
   explicit `TreeFormulaReferenceCarrier` entries, and a current V1
   lazy-residual compatibility flag. The old semantic variants
   (`Literal`, `Binary`, `FunctionCall`, `Reference`, `RawOxfml`) are
   quarantined as `FixtureFormulaAst` for checked-in fixtures, unit tests,
   and the procedural scale runner. TreeCalc preparation now projects
   explicit carriers through `project_opaque_formula` and no longer lowers
   a local semantic AST.
10. C1 live uptake: `src/oxcalc-core/src/formula_identity.rs` now derives
    current V1 `shape_key`, `dispatch_skeleton_key`, and
    `plan_template_key` fingerprints from public OxFml
    `BoundFormula`/`SemanticPlan` artifacts. `PreparedOxfmlFormula` carries
    those keys, and TreeCalc run artifacts plus runner traces surface
    `prepared_formula_identity` records. This is a compatibility identity
    layer; canonical OxFml `PreparedCallable`, `PlanTemplate`,
    `HoleBindings`, and formal-reference identity fields remain CALC-002
    pressure.
11. C2 live uptake: `formula_identity.rs` now models
    `PreparedCallable = (PlanTemplate, HoleBindings)` for the current V1
    compatibility path. `PlanTemplate` carries the C1 keys plus ordered
    compatibility holes; `HoleBindings` carries the per-formula payload
    vector and `binding_fingerprint`; TreeCalc and runner traces surface
    `prepared_callable_key`, `hole_binding_fingerprint`, and
    `template_hole_count`. Reuse evidence and canonical OxFml artifact
    fields remain C5/CALC-002 pressure.
12. C3 live uptake: `formula_identity.rs` now maps the default W050 hole
    taxonomy into current V1 prepared-call identity. `ValuesOnlyPreAdapter`
    arguments emit `ValueHole(AnyValue)`, `RefsVisibleInAdapter` arguments
    emit `RefOrValueHole(ReferenceIdentityVisible)`, invocation callees can
    emit `CallableHole(AnyCallable)`, and stable serialization exists for
    `ShapeSensitiveHole`, `SparseRangeHole`, and `RichValueHole`.
    `SparseRangeHole` and `RichValueHole` are representable but not emitted
    by current production kernels; Lane G capability vocabulary,
    cache/reuse evidence, and canonical OxFml artifact fields remain open.
13. C4 live uptake: TreeCalc now carries
    `arg_preparation_profile_version` in `LocalTreeCalcEnvironmentContext`,
    threads it into OxFml `StructureContextVersion`, and derives
    `StructuralRebindRequired` seeds when the previous and next profile
    versions differ. Current V1 behavior conservatively marks all formula
    owners for rebind because targeted affected-callable metadata is
    sibling-owned by OxFml/OxFunc.
14. C5 live uptake: TreeCalc diagnostics now emit
    `oxfml_plan_template_reuse_count` records grouped by
    `plan_template_key`. The deterministic C5 test proves two `SUM` call
    sites share one template key while retaining two prepared-callable
    identities, two hole-binding fingerprints, and distinct published
    values. This is current V1 trace-count evidence, not a canonical OxFml
    cache claim.
15. C6 live uptake: current V1 records the compile-time folding boundary
    for plan-template identity. OxCalc does not fold formula source text or
    import OxFunc semantics to infer folded plans; the deterministic C6
    test keeps `=2+3*4` and `=14` distinct by `shape_key` and
    `plan_template_key`. Canonical folded-plan identity remains routed to
    CALC-002, and future evidence-gated narrowing producers remain routed
    through CALC-004.
16. A2 live uptake: TreeCalc fixture policy is documented in
    `docs/test-fixtures/core-engine/treecalc/README.md`. `RawOxfml` is the
    preferred opaque OxFml source path; `Literal`, `Reference`, `Binary`,
    and `FunctionCall` survive only as legacy structured quarantine.
    Representative manifest tags now distinguish
    `fixture-policy:opaque-oxfml-source` from
    `fixture-policy:legacy-structured-quarantine`, and
    `FixtureFormulaAst::policy_class()` plus
    `treecalc_fixture_policy_tags_match_representative_cases` make that
    distinction executable.
17. B6 live uptake: TreeCalc now records OxFml returned-value surface
    diagnostics at the adapter boundary. Deterministic tests exercise
    literal, function-call, LET/LAMBDA invocation, dynamic-array/spill-like
    `SEQUENCE(3)`, returned-callable current V1 fallback,
    `INDIRECT(RTD(...))` dynamic rejection, and direct RTD typed-provider
    rejection. Current V1 callable return publication remains partial:
    `=LAMBDA(x,x+1)` reaches TreeCalc as worksheet fallback `Calc` with
    returned surface `Error(Calc)`, not as a stable callable payload.
    Canonical callable value transport remains routed to CALC-002.
18. B8 live uptake: `TreeCalcRunner` now emits
    `session_path_evidence.json` at the TreeCalc local run root. The
    checked-in B8 root is
    `docs/test-runs/core-engine/treecalc-local/w050-b8-treecalc-session-corpus-001`.
    It records the artifact-root declaration, retention policy, run and
    validation commands, candidate/commit/reject correlation keys,
    returned-value-surface diagnostics, replay-facing diagnostics, and
    per-entry non-mutation checks. The replay-appliance bundle references
    the packet and validates the path in
    `replay-appliance/validation/bundle_validation.json`.
19. B9 live uptake: current W050 session use composes with the frozen public
    OxFml V1 consumer facade. The read-only inventory confirms OxCalc uses
    `RuntimeEnvironment`, `RuntimeFormulaRequest`, `RuntimeFormulaResult`,
    and `RuntimeSessionFacade` from `oxfml_core::consumer::runtime`, with
    `OxfmlRecalcSessionDriver::ensure_prepared` mapped to
    `open_managed_session`, full-result `invoke` mapped to `execute`, and
    managed-commit evidence mapped to `execute_and_commit_managed` /
    `commit_managed`. Remaining prepared-callable, plan-template,
    reference/input, full managed-result, replay-correlation, callable/rich
    value, folding/reuse trace, and metadata-invalidation fields remain
    CALC-002 pressure rather than OxCalc-private adapter work.
20. A5 live uptake: `TreeReference::carrier_class()` now classifies
    `HostSensitive`, `DynamicPotential`, `CapabilitySensitive`, and
    `ShapeTopology` as `RuntimeFactProjection` rather than formula
    references. Executable tests prove those carriers lower to diagnostics
    without dependency edges or source-reference tokens in the fixture
    catalog path. This keeps them as host/evaluator/runtime fact projections;
    no new follow-on bead was needed beyond the existing Lane D/G/H
    subscription, capability, and handoff lanes.
21. A4 live uptake: active showcase and slide wording now presents
    `TreeFormulaCatalog` as opaque OxFml source/carrier carriage,
    `TreeReference` as dependency/evaluator-fact carrier projection, and
    runtime calculation as the current `evaluate_with_oxfml_session` /
    `OxfmlRecalcSessionDriver::invoke` path. The repaired surfaces no longer
    present `evaluate_via_oxfml`, `build_upstream_host_packet`, or
    `formula_allows_lazy_residual_publication` as current production names.
22. A6 live uptake: the Lane A search gate is recorded at
    `docs/test-runs/core-engine/w050-a6-lane-a-search-gate-001/SEARCH_GATE_SUMMARY.md`.
    The Rust no-hit proof covers `translate_formula`, `TranslationState`,
    `evaluate_via_oxfml`, `build_upstream_host_packet`,
    `formula_allows_lazy_residual_publication`,
    `RuntimeEnvironment::new().execute`, and
    `TreeFormula::{Literal,Binary,FunctionCall,Reference}`. Remaining hits
    are classified as opaque source/carrier code, runtime-fact carrier
    projection, `#[cfg(test)]` scaffolding, fixture/scale quarantine,
    synthetic A1 compatibility residue, upstream-host fixture scaffolding, or
    historical/status text.
23. D1 live uptake: `CalculationRepository` now carries a persistent
    subscription registry keyed by `(SubscriptionTopicId, formula_stable_id)`
    with a `SubscriptionHandle` and topic descriptor. Registry entries persist
    across ordinary wave operations and release when the owning callable is
    invalidated by formula-slot replacement or removal.
24. D2 live uptake: `CalculationRepository` now carries replay-visible
    `TopicEnvelope` state keyed by `SubscriptionTopicId`, with
    `topic_sequence`, `last_observed_payload_ref`, `ordering_key`, and
    `dedupe_identity`. `TopicEnvelopeUpdate` batches are sorted before
    mutation and deduped by event identity, with schema and deterministic
    ordering tests pinning the current V1 repository behavior.
25. D3 live uptake: `StreamSemanticsVersion` is now an OxCalc profile
    selector with `ExternalInvalidationV0`, `TopicEnvelopeV1`, and
    `RtdLifecycleV2` variants. `StreamSemanticsProfile` carries
    `profile_version` plus selector value, exposes replay profile keys and
    behavior hooks, and dispatches topic updates through the selector. Tests
    cover selector serialization plus V0, V1, and V2 dispatch behavior.
26. D5 live uptake: external topic updates now dispatch through
    `StreamSemanticsProfile::dispatch_external_invalidation_updates`, producing
    replay-visible `ExternalInvalidationDirtySeed` records with `topic_id`,
    `topic_sequence`, `formula_stable_id`, and `node_id`. Those seeds feed the
    ordinary `DependencyGraph::derive_invalidation_closure` path with
    `InvalidationReasonKind::ExternallyInvalidated`; tests prove topic fanout,
    no side-channel publication, ordinary session invocation, and coordinator
    commit authority.
27. D4 live uptake: the deterministic RTD/external replay corpus root is
    `docs/test-runs/core-engine/w050-d4-rtd-external-replay-corpus-001`.
    The checked fixture records two `topic:rtd:price` updates and two
    subscribers; the run artifact records identical normalized publications
    under `ExternalInvalidationV0`, `TopicEnvelopeV1`, and `RtdLifecycleV2`.
    `rtd_external_replay_corpus_publishes_identical_values_across_stream_versions`
    proves the runtime behavior, and
    `checked_in_rtd_external_replay_corpus_artifact_matches_runtime_validation`
    binds the checked run artifact to the computed baseline.
28. E1 live uptake: `NumericalReductionPolicy` is now an OxCalc-local
    profile selector in `docs/spec/core-engine/CORE_ENGINE_PROFILE_SELECTORS.md`
    and `src/oxcalc-core/src/numerical_reduction.rs`. The checked selector
    artifact root is
    `docs/test-runs/core-engine/w050-e1-numerical-reduction-policy-selector-001`;
    it records the initial `SequentialLeftFold`, `PairwiseTree`, and
    `KahanCompensated` variants plus exact CALC-003 handoff clause text.
29. E2 live uptake: `ErrorAlgebra` is now an OxCalc-local profile selector in
    `docs/spec/core-engine/CORE_ENGINE_PROFILE_SELECTORS.md` and
    `src/oxcalc-core/src/error_algebra.rs`. The checked selector artifact root
    is `docs/test-runs/core-engine/w050-e2-error-algebra-selector-001`; it
    records `CanonicalExcelLegacy`, the canonical precedence order
    `#NULL!`, `#DIV/0!`, `#VALUE!`, `#REF!`, `#NAME?`, `#NUM!`, `#N/A`,
    the version/profile extension rule, and exact CALC-003 handoff clause text.
30. E3 live uptake: `CorrectnessFloorProfile` now records
    `profile_version`, `numerical_reduction_policy`, and `error_algebra` in
    `OxfmlRecalcWave` trace/replay surfaces and rejects replay selector
    mismatches. The checked replay-hook artifact root is
    `docs/test-runs/core-engine/w050-e3-correctness-floor-replay-hooks-001`.
31. F1 live uptake: `EdgeValueCache` now implements the per-edge cache keyed by
    `(call_site_id, hole_binding_fingerprint)` in
    `src/oxcalc-core/src/value_cache.rs`, with deterministic volatile/effectful
    exclusion, bounded oldest-first eviction, and retention class
    `W054PendingEphemeralPerEdgeValueCache`. The checked evidence root is
    `docs/test-runs/core-engine/w050-f1-per-edge-value-cache-001`.
32. F2 live uptake: TreeCalc invocation now gates OxFml evaluation through the
    per-edge cache when prior published values seed the cache. Matching keys
    reuse cached values only when no upstream/external/dynamic dependency delta
    or caller-supplied invalidation seed is present; the checked evidence root
    is `docs/test-runs/core-engine/w050-f2-differential-evaluation-gates-001`.
33. F3 live uptake: TreeCalc now exposes `DerivationTraceRecord` as an
    opt-in invoke outcome when `derivation_trace_enabled` selects OxFml
    `PreparedCalls` trace mode. Default value-only runs emit no derivation
    trace. Trace-mode runs record template selection, hole bindings, a root
    prepared-callable invocation with child OxFml prepared calls, per-call
    kernel-returned values, and OxFml seam trace events. The checked evidence
    root is
    `docs/test-runs/core-engine/w050-f3-derivation-trace-invoke-outcome-001`.
34. F4 live uptake: TreeCalc now exposes selectable
    `LocalTreeCalcSchedulingPolicy` values for pull-flavoured full closure and
    push-flavoured visibility-bounded scheduling. Both policies run over the
    same dependency graph and prepared-callable identity surface. The checked
    fixture updates one visible observer under `PushVisibilityBounded`, proves
    the visible value matches `PullFullClosure`, records deferred hidden work,
    and pins the fairness note that push mode requires periodic full-closure
    sweeps or observer aging. The checked evidence root is
    `docs/test-runs/core-engine/w050-f4-push-pull-visibility-scheduling-001`.

## 7. Required Work

The W050 work, organised by lane.

**Lane A — Removal.**

1. Delete the OxCalc-local `TreeFormula` AST variants (`Literal`, `Binary`, `FunctionCall`, `Reference`) and all helpers that construct them. A3 has moved the remaining legacy construction into `FixtureFormulaAst`; that quarantine must not re-enter production inputs.
2. Delete `translate_formula`, `TranslationState`, and all production source-text rendering paths in OxCalc. A3 replaced the TreeCalc production preparation path with opaque source plus explicit carrier projection; fixture rendering remains quarantined.
3. Delete `synthetic_cell_target`, `synthetic_cell_row`, and the synthetic A1 cell-fixture flattening.
   A6 finds only the current V1 compatibility uses in `treecalc.rs`; deletion
   remains blocked on CALC-002/H1 canonical reference/input transport.
4. Delete `MinimalUpstreamHostPacket`, `MinimalFormulaSlotFacts`, `MinimalBindingWorld`, `MinimalTypedQueryFacts`, `MinimalRuntimeCatalogFacts` once the session API covers the same intake surface.
   A6 classifies these hits as upstream-host fixture/scaffolding residue, not
   the TreeCalc production invocation path.
5. Delete any remaining per-formula `RuntimeEnvironment::new().execute(...)` production pattern. B7 removed the `evaluate_via_oxfml` / packet-builder bridge from `treecalc.rs`; remaining packet surfaces must stay fixture-only until deleted or quarantined by later Lane A/B cleanup.
6. Convert or quarantine W047/W048 CTRO and cycle fixtures that use `TreeFormula` structured carriers.
   A2 tags `tc_w048_excel_iter_two_node_order_001` as representative
   `fixture-policy:legacy-structured-quarantine` and keeps broader fixture
   conversion/deletion in later Lane A cleanup.
7. Audit `ShapeTopology`, `DynamicPotential`, `HostSensitive`, and `CapabilitySensitive` carriers; ensure they represent evaluator/host facts, not OxCalc formula implementations.
   A5 records the carrier classification in code and tests. These four
   carriers are `RuntimeFactProjection` values, not formula references; they
   surface dependency diagnostics/runtime effects without graph edges in the
   fixture catalog path. Existing Lane D/G/H beads own subscription,
   capability, and cross-repo follow-up, so no new A5 follow-on bead was
   created.
8. Repair W047/W048 showcase wording where it implies OxCalc-local function semantics.
   A4 repairs the active showcase/slide/storyboard/catalog wording. The
   remaining Lane A search gate must keep historical/pre-W050 hits separate
   from active production claims.
9. Retain opaque OxFml source carriage only as boundary input; fixture migration adapters belong under `FixtureFormulaAst`, not `TreeFormula`.

**Lane B — Seam.**

10. Design and document the full first-call protocol per §10 (drafted in §10.3–§10.5).
11. Implement Calculation Repository: persistent structural state, dependency graph, per-node `NodeCalcState`, overlays, pinned reader views.
12. Implement OxFml Recalc Session over `oxfml_core::consumer::runtime`: open/close, `ensure_prepared`, `invoke`. Verify the session shape composes with the frozen `OxFml_V1` consumer-facade contract without reopening it. B9 records that compatibility ledger and routes missing canonical fields to CALC-002 rather than private OxFml adapters.
13. Implement the six-phase wave lifecycle (§10.5).
14. Wire reference handles: OxFml bind returns a normalised reference set; OxCalc maps it to structural targets; the dependency graph is derived from that mapping. No address-string round-trips.
15. Keep TreeCalc production invocation on the session-driven path. B7 routes `LocalTreeCalcEngine` through `OxfmlRecalcSessionDriver::invoke`; remaining work is final fixture quarantine/deletion and the explicit V1 gap ledger.
16. Add opaque-result tests proving OxCalc treats single-node outcomes without parsing or reconstruction.
    B6 now covers current V1 scalar, function-call, LET/LAMBDA invocation,
    dynamic-array/spill-like, dynamic `INDIRECT`, and RTD typed-provider
    surfaces through the TreeCalc session path. Remaining exact gaps are
    canonical callable-result publication, broader registered-external
    lifecycle surfaces, and any richer spill/rich-value transport beyond
    the current `ValueDelta` text summary. B8 adds the checked-in
    deterministic session corpus root
    `docs/test-runs/core-engine/treecalc-local/w050-b8-treecalc-session-corpus-001`,
    generated by
    `cargo run -p oxcalc-tracecalc-cli -- treecalc w050-b8-treecalc-session-corpus-001`.
    Its `session_path_evidence.json` packet records the root policy,
    command manifest, candidate/commit/reject correlation keys,
    replay-facing diagnostics, checked-in retention policy, and replay
    non-mutation validation. B9 records the public-facade compatibility
    inventory and confirms that current full-result session invocation
    remains the TreeCalc path while managed commit stays compatibility
    evidence pending CALC-002 full managed-result fields.

**Lane C — Identity Layer.**

17. Implement `shape_key` derivation during parse and bind (content-fingerprint over the parse tree with leaves abstracted).
18. Implement `dispatch_skeleton_key` (shape_key + bind-time function dispatch).
19. Implement `plan_template_key` (dispatch_skeleton_key + semantic-plan structure).
    C1 now covers current V1 compatibility fingerprints and trace fields;
    canonical OxFml fields remain open for C2/CALC-002.
20. Move `PreparedCallable` to `(PlanTemplate, HoleBindings)` shape; verify reuse across cells sharing `plan_template_key` via a microbenchmark or trace-counting test.
    C2 now covers the artifact shape and deterministic separation tests;
    C5 now covers current V1 trace-count evidence for shared
    `plan_template_key` with distinct per-call-site bindings and values.
21. Implement the default hole-type taxonomy: `ValueHole`, `RefOrValueHole`, `CallableHole`, `ShapeSensitiveHole`, `SparseRangeHole`, `RichValueHole`.
    C3 now covers current V1 taxonomy representation, stable keying, and
    wide-by-default production mapping. Lane G capability vocabulary and
    OxFunc kernel activation for sparse/rich producers remain open.
22. Document the wide-by-default policy and reserve narrowing producers (`ConstNumericHole`, etc.) for opt-in evidence-gated implementation.
    C3 documents and tests the current V1 wide-by-default mapping; narrower
    producers remain evidence-gated successor work. C6 records the
    compile-time folding boundary: no current public OxFml surface exposes
    folded-plan identity, so current V1 keeps folded-equivalent source
    strings distinct until CALC-002/CALC-004 provide the producer and
    identity contract.
23. Treat changes to `ArgPreparationProfile` for any existing OxFunc function as bind-visible name-world events; verify the invalidation pathway in the test corpus.
    C4 covers the current V1 conservative invalidation path with
    structure-context-version and runtime rebind tests. Narrow
    affected-callable targeting remains an OxFml/OxFunc surface gap routed
    through the upstream notes.

**Lane D — External Invalidation.**

24. Implement Subscription Registry on the Repository: `(topic_id, formula_stable_id) → SubscriptionHandle`, persistent across waves.
    D1 adds the repository-owned registry plus persistence and callable
    invalidation release tests.
25. Implement Topic Envelopes: `(topic_id, topic_sequence, last_observed_payload_ref)`. Replay-visible.
    D2 adds repository-owned `TopicEnvelope` / `TopicEnvelopeUpdate` state
    with `ordering_key` and `dedupe_identity`, JSON schema round-trip tests,
    and deterministic ordering/dedupe tests for replay-facing batches.
26. Wire `StreamSemanticsVersion = ExternalInvalidationV0 | TopicEnvelopeV1 | RtdLifecycleV2` as a profile-governed selector with the three behaviours specified in Foundation.
    D3 adds `StreamSemanticsProfile`, selector serialization, behavior hooks,
    and dispatch tests. `ExternalInvalidationV0` records the dirty-seed hook
    without mutating topic envelopes; `TopicEnvelopeV1` dispatches through
    deterministic envelope ordering/dedupe; `RtdLifecycleV2` dispatches
    through the same envelope path and exposes an RTD lifecycle hook for the
    later lifecycle/replay corpus beads.
27. Implement the typed dirty-seed pathway: external signal → topic envelope update → all subscribing formula_stable_ids marked dirty for the next wave.
    D5 adds `ExternalInvalidationDirtySeed` fanout by topic subscription,
    routes those seeds into ordinary dependency closure, preserves existing
    published values until coordinator commit, and exercises ordinary
    `OxfmlRecalcWave` session prepare/dependency/invoke plus
    `TreeCalcCoordinator` publication authority.
28. RTD-driven recalc replay-determinism corpus: a fixture suite that verifies a recorded sequence of topic updates reproduces identical published values under each `StreamSemanticsVersion`.
    D4 adds the checked corpus root
    `docs/test-runs/core-engine/w050-d4-rtd-external-replay-corpus-001`
    with `fixture.json`, `run_artifact.json`, validation commands, and tests
    proving identical normalized coordinator publications for recorded topic
    sequences 1 and 2 under all three stream semantics settings.

**Lane E — Correctness Floor.**

29. Specify `NumericalReductionPolicy` (e.g., `SequentialLeftFold`, `PairwiseTree`, `KahanCompensated`) as a profile-governed selector.
    E1 specifies the selector in
    `docs/spec/core-engine/CORE_ENGINE_PROFILE_SELECTORS.md`, exposes the
    Rust selector/profile surface in `src/oxcalc-core/src/numerical_reduction.rs`,
    and checks the handoff-ready exact clause language through
    `docs/test-runs/core-engine/w050-e1-numerical-reduction-policy-selector-001/selector_artifact.json`.
30. Specify `ErrorAlgebra` as a profile-governed precedence specification (canonical Excel precedence: `#NULL!` > `#DIV/0!` > `#VALUE!` > `#REF!` > `#NAME?` > `#NUM!` > `#N/A`; profile may declare alternatives).
    E2 specifies the selector in
    `docs/spec/core-engine/CORE_ENGINE_PROFILE_SELECTORS.md`, exposes the
    Rust selector/profile surface in `src/oxcalc-core/src/error_algebra.rs`,
    and checks the handoff-ready exact clause language through
    `docs/test-runs/core-engine/w050-e2-error-algebra-selector-001/selector_artifact.json`.
31. File `HANDOFF_CALC_003_OXFML_NUMERICAL_REDUCTION_AND_ERROR_ALGEBRA.md` to OxFml / OxFunc: kernels must honour the active reduction policy; OxFml semantic plan threads the policy and algebra into evaluation context.
32. Add replay-validation hooks: every wave's trace records the active reduction policy and error algebra; replay verifies match.
    E3 adds `CorrectnessFloorProfile` and
    `CorrectnessFloorReplayRecord` in `src/oxcalc-core/src/correctness_floor.rs`,
    wires the profile into `OxfmlRecalcWave`, updates the trace artifact schema
    note, and checks accepted/mismatched replay records through
    `docs/test-runs/core-engine/w050-e3-correctness-floor-replay-hooks-001/run_artifact.json`.

**Lane F — Performance / Observability.**

33. Implement per-edge value cache keyed by `(call_site_id, hole_binding_fingerprint)`; bounded by an explicit eviction policy.
    F1 adds `EdgeValueCache` with `MaxEntriesOldestFirst` bounded eviction,
    hit/miss/exclusion tests, and checked artifact
    `docs/test-runs/core-engine/w050-f1-per-edge-value-cache-001/run_artifact.json`.
34. Implement differential-evaluation gates: at invocation time, check whether hole-binding fingerprints match cached subresults; on hit, skip subexpression re-evaluation.
    F2 wires the TreeCalc invocation loop to skip OxFml evaluation on a
    cache-hit default verification rerun while preserving publication behavior,
    and to bypass reuse on upstream-publication invalidation. Evidence:
    `docs/test-runs/core-engine/w050-f2-differential-evaluation-gates-001/run_artifact.json`.
35. Implement derivation trace as a first-class `invoke` outcome under trace-mode opt-in: template selection, hole bindings, sub-invocation tree, kernel-returned values.
    F3 adds `DerivationTraceRecord` to TreeCalc run artifacts and consumer
    results under the `derivation_trace_enabled` runtime policy flag. The
    default path remains value-only. The checked artifact
    `docs/test-runs/core-engine/w050-f3-derivation-trace-invoke-outcome-001/run_artifact.json`
    binds the opt-in LET/LAMBDA fixture output to the runtime validation
    tests.
36. Implement push/pull duality: a wave's scheduling policy (push-flavoured visibility-bounded vs pull-flavoured full-closure) is selectable on top of the same dependency graph and the same `PreparedCallable` cache.
    F4 adds `LocalTreeCalcSchedulingPolicy::{PullFullClosure,
    PushVisibilityBounded}` through TreeCalc runtime policy. The visibility
    fixture selects node 3 as the only visible observer after a shared upstream
    input changes; node 3 publishes the same value as full closure, node 4
    remains at its seeded published value and is left dirty/deferred. Evidence:
    `docs/test-runs/core-engine/w050-f4-push-pull-visibility-scheduling-001/run_artifact.json`.

**Lane G — Forward Scaffolding.**

37. Specify the initial capability-set vocabulary: `Indexable(rank, index_type, element_value_class)`, `Enumerable(element_value_class, order_guarantee)`, `Shaped(extent_class)`, `Materialisable(target_class)`.
38. Admit `RichValueHole(required_capability_set)` to the hole-type taxonomy with the initial vocabulary; verify capability-set composition is part of `plan_template_key`.
39. Reserve `ArgPreparationProfile::RichArgAccepted(capability_set)` as a kernel-side variant; document that no OxFunc kernel currently consumes it, but the seam admits the variant additively.
40. Add capability-set columns to trace/replay schemas so that future rich-value kernels emit identity-discipline-compliant evidence from day one.

**Cross-cutting.**

41. Update `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md` to reflect the session model.
42. Update `docs/spec/core-engine/CORE_ENGINE_TREECALC_OXFML_SEAM_NEGOTIATION_MATRIX.md` to mark resolved topics as closed and new topics (capability-set vocabulary, topic envelopes, plan templates) as W050-internal.
43. Update `docs/LOCAL_EXECUTION_DOCTRINE.md` Lesson 8 reference: this rework is a working example of engine-and-oracle widening moving together.
44. File the three cross-repo handoff packets to OxFml under `docs/handoffs/HANDOFF_REGISTER.csv`.

## 8. Exit Gate

W050 may close only when **all** of the following hold. The gate is comprehensive by intent; partial completion remains `in_progress`.

**Lane A — Removal.**

1. The OxCalc-local `TreeFormula` AST is deleted or quarantined so no production-path semantic expression constructor remains.
2. `translate_formula`, `TranslationState`, `synthetic_cell_target`, `synthetic_cell_row` are deleted from production paths; current A3 leaves only the synthetic A1 compatibility helpers needed by B7/CALC-002.
3. `MinimalUpstreamHostPacket` and the `Minimal*` family are deleted; the session API covers the same intake surface.
4. Per-formula `RuntimeEnvironment::new().execute(...)` patterns are deleted from OxCalc production code.
5. W047/W048 CTRO/cycle fixtures use either OxFml-authored formula sources or explicitly-opaque/quarantined migration adapters; no `TreeFormula::{Literal,Binary,FunctionCall}` references remain in fixture corpora.

**Lane B — Seam.**

6. The OxCalc/OxFml first-call protocol is explicitly documented in §10 and reflected in `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`.
7. Calculation Repository, OxFml Recalc Session, `ensure_prepared`, `invoke`, and the six-phase wave lifecycle are implemented and exercise the entire TreeCalc test corpus through the session path.
8. Reference handles are wired end-to-end: OxFml bind output drives dependency graph derivation; no synthetic-A1 round-trips remain.
9. Opaque-result tests pass: representative formulas (literal, function-call, lambda, dynamic-array, INDIRECT, RTD) are evaluated through the session and OxCalc treats outcomes opaquely.

**Lane C — Identity Layer.**

10. `shape_key`, `dispatch_skeleton_key`, `plan_template_key` are derived during parse/bind/plan and surfaced on every `PreparedCallable` and in every trace record.
11. `PlanTemplate` is a first-class artefact; reuse across cells sharing `plan_template_key` is demonstrated by a microbenchmark or trace-counting test.
12. The default hole-type taxonomy is implemented; widening policy is documented in §10.7 and enforced.
13. `ArgPreparationProfile` changes for existing functions trigger structure-context-version bumps in the test corpus.

**Lane D — External Invalidation.**

14. Subscription Registry and Topic Envelopes are implemented on the Repository.
15. `StreamSemanticsVersion` is a profile-governed selector; the three variants behave per Foundation specification.
16. RTD-driven recalc replay-determinism corpus passes under all three `StreamSemanticsVersion` settings.

**Lane E — Correctness Floor.**

17. `NumericalReductionPolicy` and `ErrorAlgebra` are profile-governed selectors documented in OxCalc-local profile specifications and threaded through the session API.
18. `HANDOFF_CALC_003_*` is acknowledged by OxFml/OxFunc; the receiving repo's receipt is filed.
19. Replay-validation hooks verify that the recorded reduction policy and error algebra match the wave-active values.

**Lane F — Performance / Observability.**

20. Per-edge value cache and differential-evaluation gates are implemented; a microbenchmark demonstrates O(k) recalc on a single-input change against a hundred-formula model where k is the changed-input fan-out.
21. Derivation trace is a first-class `invoke` outcome under trace-mode opt-in; a fixture exercises it for a representative call-template family.
22. Push/pull scheduling policies are selectable; the visibility-bounded mode is exercised by a fixture that updates only one observer.

**Lane G — Forward Scaffolding.**

23. `RichValueHole(required_capability_set)` is admitted to the hole-type taxonomy with the initial capability vocabulary.
24. `ArgPreparationProfile::RichArgAccepted` is reserved in the kernel-side vocabulary and documented.
25. Capability-set columns appear in trace and replay schemas.

**Cross-cutting.**

26. The three cross-repo handoff packets are filed and acknowledged.
27. Dynamic-array examples are represented as OxFml result surfaces or explicit future work, not OxCalc-local array plumbing.
28. W047/W048 showcase wording and active workset documents use boundary-accurate language.
29. The `WORKSET_REGISTER.md` entry for W050 records all lanes as complete.
30. The `docs/spec/core-engine/*` set is updated to reflect the new model; mirrored Foundation snapshots are refreshed.

## 9. Three-Axis Status

- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - Lane A — removal of wrong-shape OxCalc code (per §3.B and §7 items 1–9);
  - Lane C — plan-template identity layer (per §3.C, §3.D and §7 items 17–23);
  - Lane D — external invalidation discipline (per §3.E and §7 items 24–28);
  - Lane E — correctness floor (per §3.G and §7 items 29–32);
  - Lane F — performance / observability layer (per §3.H and §7 items 33–36);
  - Lane G — forward-direction rich-value scaffolding (per §3.F and §7 items 37–40);
  - cross-cutting documentation, handoffs, fixtures (per §7 items 41–44).

Lane B status note: the local OxCalc seam bead set through B9 now has
evidence for the first-call protocol, Calculation Repository, public OxFml
session driver, six-phase wave lifecycle, bind-output reference handles,
TreeCalc session invocation, opaque result coverage, session corpus packet,
and OxFml V1 compatibility ledger. Remaining canonical reference/input,
managed-result, plan-template, and structured replay/correlation fields are
carried by the cross-cutting `HANDOFF_CALC_002` lane rather than by private
OxFml adapters in OxCalc.

Lane A status note: A6 records the removal search gate in
`docs/test-runs/core-engine/w050-a6-lane-a-search-gate-001/SEARCH_GATE_SUMMARY.md`.
The retired production names are absent from Rust production/test search
scope, and active showcase wording no longer carries the targeted local
formula-semantics claims. Aggregate Lane A remains partial because current V1
synthetic A1 compatibility helpers, Minimal* upstream-host fixture
scaffolding, and legacy structured fixture/scale quarantine surfaces still
exist until CALC-002 and fixture-migration work provide replacement evidence.

W050 is `open_planning` at lane level; individual lanes may advance independently under the bead structure in §6. The aggregate workset remains `open_planning` until all lanes meet their share of the exit gate in §8.

## 10. Ideal-Design Target — Unified Recalc Session and Prepared-Callable Model

### 10.1 Framing

This section locks the *shape* of the design W050 commits the rework toward. It is the answer to the catalysts in §2 and the open lanes in §9. It is not a freeze of wire encodings, field names, or Rust types; it is a commitment to categories, contracts, and identity discipline. The full narrative form of this design lives as a separate brief (`PROPOSAL-CWPL-01`); this section is the workset-internal statement of intent. Authoritative source artefacts and supporting docs are indexed in §12.

The design is unconstrained by current OxCalc code shape. It draws on Foundation doctrine (the layered S/R/D/V/O model and the single-publisher coordinator), on the frozen OxFml `V1` consumer-facade contract, and on the OxFml/OxFunc LET/LAMBDA pin-down prep. It collapses three currently separate concerns — multi-formula recalc, repeated invocation of the same compiled artefact, and `LET`/`LAMBDA` body evaluation — into one mechanism.

### 10.2 Tenets

1. **OxFml is the first and only parser.** No part of OxCalc constructs, parses, or rewrites Excel formula source text. OxCalc holds formula text as opaque strings tagged with stable identity.
2. **Compilation is a one-time cost per artefact, not per invocation.** Parse, red projection, bind, and semantic plan are produced once for each compatible `(formula_text_version, structure_context_version, library_context_snapshot)` triple and reused across all cells and all waves that share it.
3. **References are bind outputs, not seam inputs.** OxCalc supplies caller context (anchor, structure-context version, name world, table catalog); OxFml returns the normalised reference set as part of bind; OxCalc maps that set onto its structural graph.
4. **Recalc is a session, not a sequence of unrelated calls.** A wave opens one OxFml session, pins one library-context snapshot, walks the dependency graph once, invokes callables with reference inputs, and closes.
5. **Single-publisher coordinator authority is non-negotiable.** OxFml produces `AcceptedCandidateResult`s; OxCalc decides accept-and-publish or reject-with-detail. Atomic publication and reject-is-no-publish are invariants.
6. **Cycles, dynamic dependencies, and rebinds are first-class outcomes, not error states.** They are typed result variants with explicit replay-visible semantics.
7. **What `LAMBDA` does for its body, the recalc engine does for every cell.** The execution machinery is the same at two scopes.
8. **Identity is layered.** Stable logical identity, version key, content fingerprint, runtime handle, and fence tuple are five distinct categories. No one token is asked to do the job of two. The same discipline extends to plan templates (§10.6) and hole types (§10.7).

### 10.3 The Unification: Every Formula Is a Prepared Callable

A cell formula `=A1 + B1*SIN(C1)` is, semantically, `λ(A1, B1, C1) → A1 + B1*SIN(C1)`. Its formal parameters are the references it closes over; its body is the compiled expression; its captured environment is locale, library-context snapshot, caller anchor, address mode, structure-context version. This is not analogy — it is what OxFml's existing pipeline produces when it binds and compiles a formula. The W050 design surfaces this at the seam.

A `PreparedCallable` carries everything OxFml needs to evaluate the formula given a fresh set of input values:

- `formula_stable_id` (logical formula slot, OxCalc-owned),
- `formula_text_version` (declared text revision),
- `formula_token` (FEC/F3E fence token),
- `green_tree_key`, `red_view_key`, `bind_hash`, `semantic_plan_key` (content-derived identities for parse/red/bind/plan reuse),
- `formal_reference_set` (the normalised ADT references this formula reads — `CellRef`, `NameRef`, callable parameter, structured-reference atom — returned by bind, opaque to OxCalc except for the structural-target field),
- `capability_requirements` (typed query families needed: host info, RTD, locale-format-context, now-serial, random-value),
- `runtime_effect_classification` (statically classified families that may appear at runtime: dynamic-dependency, execution-restriction, capability-sensitive, shape/topology).

A `PreparedCallable` is immutable; its identity is derived from the artefacts that fed its construction. A *call site* is the live thing: `CallSite = { prepared_callable_handle, caller_anchor, formal_reference_set_binding }`. Invoking a call site means handing OxFml a set of `(formal_reference → EvalValue | ReferenceLike)` bindings; the engine evaluates and returns a typed result.

This framing is not academic. It is the level at which `LAMBDA` invocation already operates inside OxFml today. Exposing it as the *general* surface yields: parse/bind/plan reuse across cells, the same prepared-call optimisation `MAP`/`REDUCE`/`SCAN` already use, and a clean home for replay identity.

### 10.4 Architecture: Repository, Session, Coordinator

The design partitions live state across three named surfaces. Each surface has one owner.

**Calculation Repository (OxCalc-owned).** Persistent structural truth that survives across recalc waves: the `StructuralSnapshot`; the mapping `formula_stable_id → (formula_text, formula_text_version)`; the dependency graph derived during the most recent recalc from each callable's `formal_reference_set`; the per-node `NodeCalcState`; the published value view (last-accepted observer-visible derived state); runtime overlays keyed by `(snapshot_epoch, wave_id, formula_stable_id, formula_token, bind_hash, profile_version)`; pinned reader views. The Repository holds typed *handles* into OxFml artefacts, never the artefacts themselves.

**OxFml Recalc Session (OxFml-owned, OxCalc-driven).** The only consumer surface OxCalc uses during a wave. Opened with environment-scoped inputs that do not change during the wave: `library_context_snapshot` (or a provider plus a pinned snapshot ref), `locale_format_context`, `host_info_provider`, `rtd_provider`, `registered_external_provider`, `capability_view`, `structure_context_version`. The session exposes exactly two operations beyond open/close:

- `ensure_prepared(formula_stable_id, formula_text_version, source, caller_context) → PreparedCallable | BindResult` — idempotent, content-keyed. Walks parse → red → bind → compile_semantic_plan when the cache misses; returns the cached artefact otherwise.
- `invoke(call_site, reference_bindings) → InvocationOutcome` — runs evaluate. Returns one of: `Accepted(candidate_result)`, `Rejected(reject_record)`, `DependencyShapeUpdate(topology_delta)`, `CallableValue(captured_callable)`.

There is no concept of a "host packet". There is no flattened cell fixture.

**Coordinator (OxCalc-owned).** Single publication authority. Accepts `Accepted` outcomes, performs fence and snapshot checks, and either publishes one atomic derived bundle for the node or rejects with typed detail. Rejection is no-publish; rejected work is replay-visible. Pinned reader views remain consistent with the last published epoch.

### 10.5 Recalc Wave Protocol

A wave is the smallest atomic unit of recalculation. Its lifecycle is exactly six phases.

1. **Wave preparation.** OxCalc opens a session over the current `library_context_snapshot`, locale, and provider set. It computes the dirty closure from invalidation seeds (text changes, structural edits, external invalidations, RTD updates, prior-wave cycle frontier).
2. **Ensure prepared.** For each dirty `formula_stable_id`, OxCalc calls `ensure_prepared` with the current source text and caller context. The session returns a `PreparedCallable` (or a `BindResult` with diagnostics on failure). The reference set on each callable is harvested.
3. **Dependency derivation.** OxCalc translates each callable's `formal_reference_set` into structural targets, consulting the Repository's structural map (for `NameRef`, table catalog, structured-reference resolution) and direct node lookups (for `CellRef`, `DirectNode`). The result is a dependency graph keyed by `formula_stable_id`. Static analysis is complete; dynamic edges may appear later via `TopologyDelta`.
4. **Scheduling.** OxCalc topologically orders the dirty graph. SCCs are isolated; each SCC carries the profile's `CycleSemantics` (§10.9).
5. **Invocation.** For each node ready to evaluate, OxCalc builds `reference_bindings` from the dependency graph — looking up the *current* in-wave value of each upstream node — and invokes the call site. The session returns an `InvocationOutcome`; OxCalc advances the `NodeCalcState` machine and accumulates candidate results.
6. **Atomic publication.** Once the candidate set is complete, the Coordinator commits one atomic derived bundle per node (`value_delta`, optional `shape_delta`, optional `topology_delta`, optional `format_delta`, optional `display_delta`). The published epoch advances; pinned readers either see the new bundle or remain on a prior pinned epoch. The session closes. Compiled artefacts remain in the cache, keyed by content, available for the next wave.

### 10.6 Structural Formula Equivalence and Compiled Plan Templates

§10.3 keys a `PreparedCallable` on its content fingerprint. That captures one reuse class: cells that share *identical* source. But the stronger class is structural equivalence. The example

```text
A: =MyFunction(1) + OtherFunction(2)
B: =MyFunction(A) + OtherFunction(3)
C: =MyFunction(B) + OtherFunction(4)
```

shows three formulas that share *shape* — a sum of two single-argument function calls — and differ only at the leaves. The parse-tree skeleton, function-dispatch decisions, arg-preparation profiles, coercion lifts, and call-site arities are identical. The expensive artefact is the *plan*, not the source string; sharing the plan across structurally equivalent formulas is the dominant performance opportunity at workbook scale.

The design introduces a layer of identity *below* `green_tree_key`:

- `shape_key` — fingerprint over the parse tree with leaf-value content abstracted into typed holes. Two formulas with the same `shape_key` agree on operator/function nesting, arity, lazy-control-form posture, and lambda-parameter binding sites. They differ only at hole positions.
- `dispatch_skeleton_key` — `shape_key` extended with bind-time function-dispatch decisions (which OxFunc surface each call name resolved to, under the wave's library-context snapshot).
- `plan_template_key` — `dispatch_skeleton_key` extended with the semantic-plan structure (coercion lifts, capability requirements, call-preparation profiles). Two formulas with the same `plan_template_key` can share the compiled plan in full; each instance just supplies its holes.

A `PlanTemplate` is the natural artefact: it carries the compiled plan with explicit hole positions, each typed by what it accepts (`ValueHole`, `RefOrValueHole`, `CallableHole`, `ShapeSensitiveHole`, `SparseRangeHole`, `RichValueHole` — see §10.7). A concrete `PreparedCallable` is then `(PlanTemplate, HoleBindings)` — the template plus the leaf payloads. Two cells with the same template share the heavy artefact; their per-cell difference is a thin hole vector.

This is not coincidentally the structure of a `LAMBDA`. A lambda body with `n` parameters is exactly a plan template with `n` reference holes, and a lambda call site supplies the hole bindings. Every cell formula is a degenerate lambda whose hole vector is fixed in advance from the cell's literal-and-reference leaves.

**Three levels of ambition.**

- *Minimum.* Compute `shape_key` and `dispatch_skeleton_key` as content-derived fingerprints during parse and bind; store them on every `PreparedCallable`; surface them in the trace. Use as cache-collision detectors only — callables sharing `dispatch_skeleton_key` share lowered dispatch decisions and skip parts of semantic planning. Yields most of the win for filled-down columns. **Hard requirement for W050.**
- *Moderate.* Materialise `PlanTemplate` as a first-class artefact. The session cache is keyed by `plan_template_key`; each `PreparedCallable` becomes a thin handle plus hole bindings. Parse and bind still run per text instance (cheap); the compiled plan is built once per template and invoked many times. **W050 implementation target.**
- *Maximum.* Cross-workbook anti-unification: detect formulas with different surface text that reduce to the same template after normalisation. Deferred to evidence that workbook-scale recompilation cost matters beyond what *moderate* already saves.

**Replay safety.** All three keys are content-derived from deterministic walks. Per-cell identities (`formula_stable_id`, `formula_token`, hole bindings) remain unique. Replay reproduces per-cell evaluation by replaying hole bindings against the same template; nothing about template sharing is observable at the seam beyond a faster prepare step and the additional identity columns in the trace.

### 10.7 Hole-Type Widening Policy

§10.6 left open *how precise* hole types should be. Two stances bracket the design space.

**Narrow stance.** A hole's type is precise to what the call site supplies. A constant-numeric hole and a sheet-reference hole are different hole types even at the same syntactic position. A and B in the §10.6 example produce two distinct templates. Narrow templates are fast — they can drop the value-coercion lift, hard-fold literals into the plan, and specialise array-handling paths to scalar — but they fragment the cache: 50 cells doing `=MyFn(constant)` and 50 doing `=MyFn(ref)` produce two templates, and any axis of variation multiplies.

**Wide stance.** A hole's type is the semantically broadest shape the function's `ArgPreparationProfile` accepts. A function with `ValuesOnlyPreAdapter` sees an `EvalValue` regardless of whether the leaf was a literal or a reference — the dereference has already happened. Under the wide stance, both leaves bind into the same `ValueHole`, A and B share a template, and the literal-vs-reference distinction lives only in the hole bindings, not in the template identity. The cost is per-invocation: each call walks the general value-preparation path even when the leaf is a constant.

OxFunc's `ArgPreparationProfile` already collapses most of this distinction. The vast majority of numeric, statistical, financial, and text functions are `ValuesOnlyPreAdapter` — the kernel cannot tell whether an argument was a literal or a reference, only what its value is. Narrow specialisation buys speed only when the generic path does materially more work than a specialised path *and* the leaf is constant enough that fingerprinting the constant into the plan key is justified.

**Working policy: widen by default; narrow as an opt-in optimisation backed by evidence.**

The default hole-type taxonomy at template granularity is:

- `ValueHole(value_class_bound)` — accepts any expression producing a value of (or coercible to) the bound class. Default for arguments under `ValuesOnlyPreAdapter`. Literal, reference, and sub-expression leaves all bind here.
- `RefOrValueHole(ref_observability)` — required for arguments under `RefsVisibleInAdapter` (`COLUMN`, `ROW`, `CELL`, `OFFSET`-class). Reference identity is observable at the kernel; the hole cannot widen across the literal-vs-reference axis without changing function semantics.
- `CallableHole(callable_signature)` — accepts a callable value (lambda, defined-name callable, prepared external).
- `ShapeSensitiveHole(extent_class)` — accepts an expression whose array shape participates in the call semantics (e.g., `MAP`'s data argument). Shape becomes part of template identity.
- `SparseRangeHole(extent_class, cardinality_class)` — accepts a `SparseRangeReader` binding for reference ranges that may be large but mostly blank. The reader exposes typed `declared_extent`, `defined_cardinality`, `defined_iter`, `read_at(coord) → Defined(EvalValue) | Blank`, and `contains(coord)`. Aggregation kernels (`SUM`, `COUNT`, `AVERAGE`, `MIN`, `MAX`, criteria-family functions) consume this hole through a corresponding `SparseIteratorOk` argument-preparation profile; kernels that require dense access request materialisation explicitly. The two-state `EntryStatus` distinction (`Defined` / `Blank`) reflects the cell-value-level observable in Excel: `ISBLANK`, `COUNTBLANK`, `COUNTA`, value-coercion in arithmetic, and equality comparisons all treat never-assigned and assigned-then-cleared identically as `Blank`. Empty-string `""` is a `Defined(EvalValue::Text(""))` value, distinct from `Blank` — that distinction *is* observable (`ISBLANK("")` returns FALSE; `COUNTA` counts an empty-string cell). Sheet-structural state that persists across clear operations (used range, cell formatting, conditional-format ranges, data-validation rules, comments) is owned by other surfaces in the Repository, not by the cell-value layer the `SparseRangeReader` returns.
- `RichValueHole(required_capability_set)` — accepts any value handle whose published capability set is a superset of `required`. Capabilities are drawn from the rich-value vocabulary (initially `Indexable`, `Enumerable`, `Shaped`, `Materialisable`; extensible to `Queryable`, `Differentiable`, custom host capabilities). The hole's template identity is the *required capability set*, not the concrete rich-value class — virtual regions from `FILTER`/`INDEX`/`CHOOSEROWS`, sparse range readers, external query handles, and materialised arrays all bind into the same hole when they declare compatible capabilities. See §10.12 for the future-direction framing and the rationale for admitting this hole-type variant from day one even before the capability vocabulary is exercised by kernels.

Narrowing producers are *not* in the default taxonomy; they are reserved for a later, evidence-gated layer:

- `ConstNumericHole(value)`, `ConstTextHole(value)`, `ConstBoolHole(value)` — narrowings of `ValueHole` that fold the constant into the plan. May appear when profile evidence shows a kernel has a materially faster constant-folded path and the constant is observed stable across many call sites. The narrowing input enters `plan_template_key`.
- `MonomorphicReferenceHole(target_class)` — a narrowing of `RefOrValueHole` for kernels that branch on whether the target is a single cell, an area, or a whole row/column. Required (not optional) when the branching changes the semantic plan itself (e.g., `OFFSET`'s reachable-reference set differs when its size arguments are constant versus reference).

**Why widen by default.** Sharing: filled-down columns collapse to one template regardless of whether some rows used literals. Replay simplicity: fewer template variants means fewer trace columns. OxFunc has already done the work: `ArgPreparationProfile` is the architectural line between "this hole can widen" and "this hole cannot"; the taxonomy mirrors that boundary instead of inventing a parallel one. **The W050 spec treats changes to `ArgPreparationProfile` for any existing function as bind-visible name-world events**, requiring the same invalidation discipline as a name registration change.

**Why ever narrow.** Hot paths where constant-folded kernels measurably dominate recalc time; or kernels where reference observability changes the semantic plan itself (where narrowing is forced by correctness, not optimisation).

**Tiering as a deferred direction.** A future engine may run the wide template first, observe execution counts, and promote frequently-fired call sites to narrow templates (the standard JIT tiering pattern). The spec does not commit to tiering; it ensures the identity discipline cannot prevent it. Adding narrow `plan_template_key` variants is additive over the wide baseline; no in-flight artefact is invalidated to introduce tiering later.

### 10.8 LET / LAMBDA As the General Case

Once §§10.3, 10.6, 10.7 are in place, `LET` and `LAMBDA` are not special. `LET(x, expr, body)` is structurally a hidden local node whose formula is `expr` and `body` references it; the same recalc engine that evaluates a dependency-ordered set of expressions over a shared environment runs inside one node's evaluation frame. `LAMBDA(p1, p2, …) body` is exactly a `PreparedCallable` whose `formal_reference_set` is the parameter list rather than free cell references; calling a `LAMBDA` is `invoke(call_site, bindings)` — the same operation cells use.

Three consequences. First, `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, `MAKEARRAY` reduce to "invoke a `PreparedCallable` once per element"; no special-case fast path needed beyond the general invoke. Second, the prepared-call cache that powers `LAMBDA` is the same cache that powers a column of identical cell formulas. Third, OxFml's formal model gains a single uniform statement: every evaluation is a callable invocation; cells are degenerate callables whose only captured environment comes from the session.

OxFml retains canonical authority over body parse, bind, semantic plan, and evaluation. The LET/LAMBDA pin-down prep's truths (lexical not dynamic, exact capture not approximate, callable values as first-class) carry through unchanged. OxCalc gains no new opinion about lambda semantics; it just calls invoke.

### 10.9 Cycles, Dynamic Dependencies, External Invalidation, Rebinds, Rejects

**Cycles.** When a dirty SCC has more than one node, the profile's `CycleSemantics` decides. `PriorValueFallback`: invoke each node once with previous-published values of cyclic upstreams; emit non-fatal cycle diagnostic; publish. `CycleError`: emit typed reject for each node in the SCC; no publish for the SCC. `Iterative`: invoke nodes in stable order, snapshot SCC outputs after each pass, repeat until fixed-point under tolerance or bounded iteration cap; on cap-hit emit deterministic non-convergence outcome. SCC behaviour is observable as a `CycleIterationTrace` in the trace stream.

**Dynamic dependencies.** When evaluating a node, OxFml may observe a new dependency (e.g., `INDIRECT("A1")` or a spill-extent reach). It surfaces this as a `TopologyDelta` on the invocation outcome. The Coordinator's dependency overlay absorbs the new edge; the dirty closure recomputes to include any newly-reachable upstream; nodes whose targets actually changed are re-needed. Over-invalidation is allowed (correctness); under-invalidation is a fault. Dynamic-dependency overlays are epoch/fence-scoped and evict on snapshot or token mismatch.

**External invalidation.** Foundation distinguishes three invalidation classes — `Standard` (upstream value change), `Volatile` (host recalc cycle), `ExternallyInvalidated` (explicit external signal). The third class is the one the per-formula model handles least cleanly today and must be wired explicitly under §10 design:

- The Repository carries a persistent **Subscription Registry** mapping `(topic_id, formula_stable_id) → SubscriptionHandle`. Subscriptions are created at bind time when a `PreparedCallable`'s `runtime_effect_classification` declares `ExternallyInvalidated(topic_descriptor)` (RTD calls, registered-external streams, host-watcher hooks). Subscriptions release when the callable is invalidated by a text or name-world change.
- The Repository carries a **Topic Envelope** per subscribed topic: `(topic_id, topic_sequence, last_observed_payload_ref, ordering_key, dedupe_identity)`. The envelope is the replay-visible identity of "the most recent state of this topic"; ordering and dedupe semantics are profile-governed by `StreamSemanticsVersion = ExternalInvalidationV0 | TopicEnvelopeV1 | RtdLifecycleV2`.
- The active `StreamSemanticsProfile` is replay-visible as `(profile_version, stream_semantics_version)`. `ExternalInvalidationV0` is the pathfinder dirty-seed hook without envelope mutation; `TopicEnvelopeV1` records topic envelopes with deterministic ordering and event-identity dedupe; `RtdLifecycleV2` uses the same envelope path and additionally exposes RTD lifecycle tracking hooks for the later corpus/lifecycle lane.
- An external invalidation signal routes through the selected behavior hook and stamps every subscribing `formula_stable_id` as a dirty seed with reason `ExternallyInvalidated(topic_id, topic_sequence)`. Under `TopicEnvelopeV1` and `RtdLifecycleV2`, the signal also updates the topic envelope. The next wave's dirty closure includes subscribing formulas. The wave evaluates subscribed callables by ordinary `invoke`; the `RtdProvider` / registered-external provider consults the topic envelope when the active selector records one. **The push is invalidation only; evaluation remains pull.**
- The dirty-seed record is replay-visible as `(topic_id, topic_sequence, formula_stable_id, node_id)` and enters the same dependency closure as structural or upstream-publication seeds. It does not publish values or runtime effects; only the coordinator can accept and publish a later candidate result.
- When the active selector records topic envelopes, they are recorded as wave inputs alongside formula text and structural edits. Replay reconstructs them by replaying topic-update events in the recorded order under the active `StreamSemanticsVersion`. RTD-driven recalc therefore reproduces deterministically — the engine reads the same envelope, the provider returns the same payload, the invoke produces the same result.
- The first checked replay corpus for this path is `w050-d4-rtd-external-replay-corpus-001`; it records the fixture, run artifact, and validation commands for the three stream selectors.

The same wiring applies to host-supplied invalidation hooks (custom UDFs that signal "my output changed") under the same envelope discipline. External invalidation is a first-class seed source, not a side channel.

**Rebinds.** When the bind-visible name world changes (defined name added/removed, registered external installed, `ArgPreparationProfile` change for an existing function, `structure_context_version` bump), every `PreparedCallable` whose `bind_hash` could shift is invalidated. The next wave's prepare phase re-runs bind for those callables and may produce different `formal_reference_set`s; the dependency-derivation phase consumes the change. Static rebinds do not require text changes; they propagate through `structure_context_version`.

**Rejects.** A `Rejected` invocation outcome carries typed detail sufficient for replay. The taxonomy is fixed: `SnapshotMismatch`, `ArtifactTokenMismatch`, `ProfileVersionMismatch`, `CapabilityMismatch`, `BindMismatch`, `CycleBlocked`, `DynamicDependencyFailure`, `HostProviderFailure`. The Coordinator never publishes from a rejected candidate. Replay can reproduce the reject deterministically from session inputs.

### 10.10 What Gets Deleted

The audit table in §4 lists rework targets at the surface level. This subsection states what is removed outright when the §10 design lands:

- The OxCalc-local `TreeFormula` AST (`Literal`, `Binary`, `FunctionCall`, `Reference`) and every helper that constructs Excel source text from it.
- `MinimalUpstreamHostPacket`, `MinimalFormulaSlotFacts`, `MinimalBindingWorld`, the synthetic `A1` cell-fixture flattening, and the `synthetic_cell_target`/`synthetic_cell_row` helpers.
- `translate_formula` and `TranslationState` — they exist only to lower the local AST to source text for OxFml.
- The per-formula `RuntimeEnvironment::new().execute(...)` pattern in `evaluate_via_oxfml`. Invocation goes through the session.
- `formula_allows_lazy_residual_publication` — a one-off special case for `IF`; under the unified model this is just how the lazy control form's compiled plan already behaves.
- `TreeFormula::RawOxfml`, *unless* the new session API admits opaque OxFml-owned formula handles for fixture purposes. If it does not, `RawOxfml` survives only as the explicitly-opaque migration adapter and nothing else.

What survives, unchanged in meaning but recomposed in plumbing: `StructuralSnapshot`, `TreeNodeId`, `Stage1RecalcTracker` and its state machine, `TreeCalcCoordinator` and its accept/publish/reject discipline, the overlay lifecycle, `DependencyGraph`, `InvalidationClosure`, and the entire replay/witness artefact family.

### 10.11 Status Of This Section

This is the W050 design target. It is not yet:

- a frozen Rust API surface for the OxFml-side session,
- a handoff packet to OxFml for the names/types it must expose,
- code.

Promotion to those states follows the normal handoff path. The next concrete moves are: (a) draft `HANDOFF_CALC_002_OXFML_RECALC_SESSION_AND_PLAN_TEMPLATES.md` against this section (bead `calc-cwpl.H1` in §6); (b) inventory the OxCalc code surfaces named in §10.10 and §4; (c) prototype `ensure_prepared` and `invoke` against the existing `oxfml_core::consumer::runtime` types to validate the session shape can be expressed without reopening the frozen `OxFml_V1` consumer contract.

### 10.12 Future Direction — Rich Values and Value Virtualisation

§§10.3–10.10 treat values at the seam as either materialised `EvalValue` payloads or one of a small set of typed accessors (range readers, callables). The next architectural direction generalises this to **rich values**: immutable handles with published *capability sets* that downstream kernels and call sites consume through declared capability requirements.

A rich value is an immutable observation handle. Its identity at the type level is its **capability set**, not its concrete backing class. Initial capability vocabulary:

- `Indexable(rank, index_type, element_value_class)` — supports `index(coord) → RichValue`.
- `Enumerable(element_value_class, order_guarantee)` — supports a deterministic iterator over elements; may be sparse.
- `Shaped(extent_class)` — supports `shape() → Extent`, `cardinality() → Count`, `is_defined(coord) → bool`.
- `Materialisable(target_class)` — supports `materialise() → EvalValue | EvalArray`. Always implementable, may be expensive; kernels that need bytes call this.

Extension capabilities reserved for later vocabularies:

- `Queryable(query_grammar_class)` — supports `query(q) → RichValue` for a declared grammar; the seam for database connections, in-memory query stores, and external data sources.
- `Differentiable(parameter_set)` — supports `partial(parameter) → RichValue`; the seam for sensitivity / derivative values feeding Goal Seek, Solver, and what-if analyses.
- Custom host-declared capabilities — additive vocabulary extension is permitted; capability names enter the trace and replay schemas as typed columns.

The hole-type taxonomy in §10.7 admits `RichValueHole(required_capability_set)` from day one. Kernels declare capability requirements via a corresponding `ArgPreparationProfile::RichArgAccepted(capability_set)` variant, reserved but unused until the first rich-value-aware kernel ships. The discipline is:

- Capabilities are *published*, not inferred — a rich value declares its capability set at construction.
- The semantic plan verifies capability availability at compile (template binding fails fast on capability mismatch); the engine never introspects a value's concrete class to know what it supports.
- Rich values are immutable observation handles; capability invocations are deterministic functions of state; replay records *capabilities exercised* and *responses received* and reproduces by replaying operations against the recorded backing.

**The five concerns this single commitment unifies.**

1. Sparse range representation (§10.7 `SparseRangeHole`) is the `Indexable + Enumerable + Shaped` capability set with sparse-iteration semantics. `SparseRangeReader` is the first concrete rich-value class.
2. Virtual regions from `INDEX`, `OFFSET`, `FILTER`, `CHOOSEROWS`, `CHOOSECOLS` become deferred-materialisation rich values; the engine never flattens the source range to take a sub-region.
3. External data sources (databases, in-memory query stores, remote APIs) attach as `Queryable` rich values; the seam to "spreadsheet with a database column" becomes a single OxFunc kernel returning a `Queryable`, with everything downstream composing via capability declarations.
4. Sensitivity / derivative values are the `Differentiable` capability layered onto a numeric rich value; Goal Seek and Solver become capability queries against a graph of differentiable rich values.
5. Dynamic-array spill anchors and extents become first-class rich values with `Shaped + Indexable` capabilities; Excel spill semantics get a clean type-system home instead of bolt-on machinery.

**Why admit `RichValueHole` from day one even though kernels do not yet consume it.** Retrofitting a capability-set column into the hole-type taxonomy after thousands of plan templates exist is the expensive failure mode the layered-identity tenet was written to prevent (§10.2 Tenet 8). Admitting the hole-type variant now, with an empty capability vocabulary, costs nothing and preserves the option. **The single decision that matters at W050 commit time is the type-system commitment, not the kernel implementation.**

**What is deferred to later worksets.**

- Concrete rich-value classes beyond `SparseRangeReader`.
- `RichArgAccepted` argument-preparation profile activation in OxFunc kernels.
- The `Queryable` and `Differentiable` capability vocabularies (these are application-shaped capabilities and depend on use-case pressure).
- The integration of differential evaluation (Move B in the Additional Excellent Moves discussion) with rich-value capability invocations.

## 11. Improvement Moves Beyond The Baseline

The §10 design baseline is the *unified prepared-callable model* — the minimum architectural commitment that fixes the per-formula seam shape. This section catalogues the additional moves that turn the baseline into a competitive calculation engine. It records which moves land within W050 (mapped to the lane structure in §5), which are scaffolded inside W050 with concrete work deferred, and which are deferred entirely to successor worksets.

### 11.1 Moves Landing In W050

**Move A+ — Push/Pull Duality with External Invalidation.** Push and pull are two scheduling policies over the same dependency graph and the same `PreparedCallable` cache. *Internal push* (an editor changes A1, propagate forward to visible dependents) and *external push* (an RTD topic invalidates, a registered external signals, a host watcher fires) both produce typed dirty seeds; evaluation remains pull. External invalidation is wired through the persistent Subscription Registry, per-topic envelopes governed by `StreamSemanticsVersion`, and replay-deterministic ordering and dedupe.

*Lands in:* Lane D (external invalidation), Lane F (push/pull duality). See §7 items 24–28, 36.

**Move B — Differential Evaluation Through Per-Edge Value Cache.** Each invocation's subresult is cached keyed by `(call_site_id, hole_binding_fingerprint)`. On the next wave, subexpressions whose hole bindings match the cached fingerprint reuse the cached subresult; volatile subexpressions are excluded from the cache. Reduces O(n) recalc to O(k) where k is the number of subexpressions whose inputs actually changed — the dominant win for interactive single-input changes against a large fan-out.

*Lands in:* Lane F. See §7 items 33–34.

**Move C — Numerical Reduction Policy and Error Algebra as Profile State.** Two new profile-governed semantic selectors: `NumericalReductionPolicy` (summation order — `SequentialLeftFold`, `PairwiseTree`, `KahanCompensated`) and `ErrorAlgebra` (canonical Excel precedence `#NULL!` > `#DIV/0!` > `#VALUE!` > `#REF!` > `#NAME?` > `#NUM!` > `#N/A`, or a profile-declared alternative). Both are replay-validated. Filed as `HANDOFF_CALC_003_*` to OxFml / OxFunc because kernels must honour the active reduction policy and OxFml semantic plan must thread the policy and algebra into evaluation context.

*Lands in:* Lane E. See §7 items 29–32.

**Move F — Derivation Trace as First-Class Observable Surface.** Under trace-mode opt-in, every `invoke` emits the template selected, hole bindings, sub-invocation tree, kernel returns. Template sharing means explanation sharing: 247 cells using one template share an explanation skeleton; the per-cell view differs only in hole bindings. Turns "Why is this cell `#VALUE!`?" from a manual stare into a browse.

*Lands in:* Lane F. See §7 item 35.

**Move I (identity layer) — Rich Values and Capability-Set Hole Admission.** `RichValueHole(required_capability_set)` enters the default hole-type taxonomy from day one with the initial capability vocabulary `Indexable + Enumerable + Shaped + Materialisable`. The kernel-side `ArgPreparationProfile::RichArgAccepted(capability_set)` variant is reserved but not yet exercised. No concrete rich-value class beyond `SparseRangeHole` ships in W050; the architectural commitment is the identity discipline that admits rich-value kernels additively later.

*Lands in:* Lane C (hole taxonomy), Lane G (capability vocabulary), §10.12 (future-direction framing). See §7 items 21–23, 37–40.

### 11.2 Moves Scaffolded In W050, Concrete Work Deferred

**Move D+ — Sparse Range Readers as the First Concrete Rich-Value Class.** `SparseRangeHole(extent_class, cardinality_class)` is admitted to the hole-type taxonomy in W050 (Lane C). The hole's typed protocol — `declared_extent`, `defined_cardinality`, `defined_iter`, `read_at(coord) → Defined(EvalValue) | Blank`, `contains(coord)` — is specified. The two-state cell-value model (`Defined` covers all assigned values including empty-string `""`; `Blank` covers both never-assigned and cleared cells, which Excel treats identically) preserves `ISBLANK` / `COUNTBLANK` / `COUNTA` semantics without claiming a non-observable distinction at the cell-value level. **The kernel-side `SparseIteratorOk` argument-preparation profile is reserved in OxFunc but not yet exercised by aggregation kernels.** Concrete implementation requires touching each aggregation function (`SUM`, `COUNT`, `AVERAGE`, `MIN`, `MAX`, criteria family) and is owned by OxFunc; a successor workset `W051_SPARSE_RANGE_READERS_AND_DEFINED_ENTRY_SEMANTICS` (or equivalent) tracks this. W050's commitment ensures the seam admits sparse readers without retrofit.

**Move H — Sensitivity / Derivative Seam.** The `Differentiable(parameter_set)` capability is named in W050 Lane G as a reserved extension of the rich-value capability vocabulary. No concrete kernel carries derivative metadata yet. Goal Seek / Solver / what-if all sit on top of this capability when it lands. Successor workset: `W052_SENSITIVITY_AND_DERIVATIVE_SEAM` (or equivalent), filed against OxFunc as the primary owner. **The W050 commitment is that capability vocabulary admission is additive, so retrofit is not required.**

### 11.3 Moves Deferred Entirely (Not W050)

**Move E — Speculative Parallel Evaluation Behind Single-Publisher Commit.** Stage 2 concurrency work per Foundation `CORE_ENGINE_FORMAL_MODEL` §6.8 staged-realization contract. Prerequisite: W050 lands the Stage 1 sequential coordinator on the new session model, and FEC/F3E concurrency-hardening gates per Foundation Wave B are met. Successor workset: `W053_STAGED_CONCURRENCY_STAGE_2` (or equivalent).

**Move G — Bounded-Memory Pinned-Epoch GC.** Operational discipline requiring measurement infrastructure (artifact retention costs, overlay residency, pin-epoch distance histograms). Prerequisite: W050 lands the new artifact set so retention costs are measurable. Successor workset: `W054_BOUNDED_MEMORY_AND_PINNED_EPOCH_GC` (or equivalent).

### 11.4 Cross-Cutting Discipline (Landing In W050)

Three smaller catches affect everything above and land within W050:

- **Compile-time constant folding at bind.** Intended as OxFml/OxFunc-owned compiler hygiene: if `=2 + 3*4` folds to `14` in the plan, the folded form must enter canonical `plan_template_key` identity through public OxFml surfaces. Current V1 records the boundary instead of inferring it in OxCalc; C6 keeps `=2+3*4` and `=14` distinct until CALC-002/CALC-004 expose the folded-plan and narrowing-producer contracts.
- **Common-subexpression elimination across plan templates.** When `=SUM(A1:A100)/COUNT(A1:A100)` appears in many cells, the two aggregates over the same range share a materialisation pass. Restricted to deterministic-pure kernels; not applied to volatile or side-effecting paths. Lane F.
- **Compilation as observable phase.** "This template was compiled at epoch X with these inputs" enters the trace stream alongside evaluation events. Replay validates compilation determinism, not only evaluation determinism. Lane F.

### 11.5 Wave Ordering Reference

For implementation prioritisation see §5. The wave order it recommends is: Wave 1 (Lanes B + C concurrent, A following); Wave 2 (Lanes D + E parallel); Wave 3 (Lane F). Lane G lands cheapest alongside Lane C. The deferred moves D+, E, G, H are not staged inside W050; their identity scaffolding (where applicable) is part of Lanes C and G and lands in Wave 1.

## 12. Structured References

This section indexes every doctrinal, spec, handoff, and comparator artifact that bears on the W050 redesign. Scope is **strictly** the core-engine model, the calculation abstraction, and the decomposition of work across `OxCalc`, `OxFml`, and `OxFunc`. Summaries are focused on the W050 redesign questions:

1. what does OxCalc hand to OxFml on the first call for a node formula,
2. who parses raw formula text (locked: OxFml is first and only parser),
3. where do references live and when are they resolved,
4. how is OxFml asked to evaluate the same formula many times with different inputs (Lambda-style optimisation generalised to general formula reuse),
5. what shape of single-node result does OxCalc consume.

Creation dates are the git-recorded add date of the file in its current repo, reported as `YYYY-MM-DD`. Relevance to W050 is annotated as **HIGH / MED / LOW**.

### 12.1 Foundation Doctrine (authority precedence: highest)

`../Foundation/CHARTER.md` — DNA Calc Charter — 2026-02-22 — **HIGH**
- Locks the lane ownership split: OxFml owns formula language, parse/bind, lexical slots, LET/LAMBDA binding, child evaluation order, lazy control forms, compiled formula plans, FEC/F3E seam spec, trace publication policy. OxCalc owns multi-node core engine, workbook-level scheduling, invalidation, publication, caching, concurrency, graph/backend execution strategy. W050 must not violate this split.

`../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md` — Architecture and Requirements — 2026-02-22 — **HIGH**
- Section 3.4 calculation pipeline (parse / bind / dependency / invalidation / schedule / evaluate / commit) and 3.18 FEC/F3E lane boundary are the doctrinal frame W050 redesigns within. Section 3.18.1 already states the canonical data-flow: "Formula text enters OxFml — parse and bind produce a normalized formula AST with resolved references" — i.e. OxFml is the first parser by doctrine. Optimisation rule (3.4): optimised evaluators should consume resolved OxFunc call-site handles, not re-dispatch through string-keyed paths. CONSTR-027: the OxFunc / OxFml / OxCalc / OxVba lane boundaries cannot be collapsed without synthesis-approved architecture edits.

`../Foundation/OPERATIONS.md` — Foundation Operations — 2026-02-22 — **HIGH**
- Section governing FEC/F3E protocol definition authority: OxFml defines the evaluator-side contract (session lifecycle, commit deltas, trace schema); OxCalc co-defines coordinator-facing parts (publication fences, scheduling interaction, rejection policy). Forbids OxFml/hosts duplicating OxFunc-owned function/operator semantics for speed shortcuts — directly relevant to the W050 sub-question of generalising Lambda-style optimisation: any such optimisation must live behind metadata contracts, not by re-implementing semantics in OxCalc.

`../Foundation/CORE_ENGINE_FORMAL_MODEL.md` — Core Engine Formal Model (Foundation mirror) — 2026-02-28 — **HIGH**
- Read-only mirror of `OxCalc/docs/spec/core-engine/CORE_ENGINE_FORMAL_MODEL.md`. Defines the layered model: Structure (S), References (R derived from S + bind context), Dependencies (D derived from R), Values (V), and Operations (O) as the exclusive persistent-mutation pathway. Locks calc-time dependency/reference overlays as derived overlays over R/D, never canonical-layer mutation. Sections 5.4 (calculation semantics) and 6.3 (reference model) are the formal substrate W050's first-call protocol must satisfy. Overlay reuse key tuple `(snapshot_epoch, wave_id, formula_stable_id, formula_token, bind_hash, profile_version)` is the canonical identity vocabulary the first-call protocol should adopt.

`../Foundation/CORE_ENGINE_THEORY_AND_ALTERNATIVE_PATHS.md` — Core Engine Theory and Alternative Paths — 2026-03-09 — **MED**
- Theory companion to the formal model. Relevant subsections: 3.1 fixed-point/stabilization framing, 3.4 dynamic-dependency overlay theory (structural deps ∪ runtime-observed deps), 3.6 FEC/F3E transactional seam theory (prepare → open_session/capability_view → execute → commit), 3.7 single-publisher concurrency discipline. Useful for justifying any W050 design choice that departs from naive per-formula host calls.

### 12.2 OxCalc Canonical Core-Engine Spec Set

`docs/spec/core-engine/CORE_ENGINE_ARCHITECTURE.md` — Top-Level Core-Engine Architecture — 2026-03-15 — **HIGH**
- Establishes immutable structural truth, MVCC-style versioned runtime state, single-publisher coordinator authority, and TreeCalc-first scope (no grid, no spill, no structural rewrites). Architectural pillars all W050 work must respect: OxCalc owns the structural substrate; formula meaning is an attribute attached to it via OxFml.

`docs/spec/core-engine/CORE_ENGINE_STATE_AND_SNAPSHOTS.md` — State and Snapshots — 2026-03-15 — **MED**
- Defines structural snapshots, versioned runtime views, and the invariant that derived facts (dependency, reference resolution, value) are overlays over an immutable structural identity. Relevant to W050 because the first-call protocol must thread snapshot/structure-context identity, not implicit globals.

`docs/spec/core-engine/CORE_ENGINE_OVERLAY_AND_DERIVED_RUNTIME.md` — Overlay and Derived Runtime — 2026-03-15 — **MED**
- Specifies the overlay key, protection, eviction, and reuse rules for derived runtime state (invalidation/execution, dynamic dependency, capability fence, etc.). Any OxFml artifact reuse OxCalc relies on for repeated-evaluation optimisation must be representable as a typed overlay under this model, not as opaque mutable state inside OxFml.

`docs/spec/core-engine/CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md` — Recalc and Incremental Model — 2026-03-15 — **HIGH**
- Invalidation-state-machine semantics, deterministic recalc baseline, scheduling-as-non-semantic principle. Required to keep the W050 first-call protocol invalidation-correct: when a formula text changes, when a reference rebinds, when a dependency is added at runtime, OxCalc's invalidation closure must remain the authority.

`docs/spec/core-engine/CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md` — Coordinator and Publication — 2026-03-15 — **HIGH**
- Single-publisher authority, candidate-vs-publication separation, atomic publication, observer-stable transitions. Locks the post-evaluation half of the seam: whatever the first-call protocol returns, OxCalc — not OxFml — decides whether it publishes.

`docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md` — Core Engine OxFml Seam (OxCalc local view) — 2026-03-15 — **HIGH**
- The canonical OxCalc-local seam companion. Section 4 ("OxCalc Expectations Of Evaluator Artifacts"): evaluator artifacts are immutable versioned inputs identified by token/version discipline. Section 5 (Candidate Work Boundary): mandates separation of structural/evaluator inputs from candidate evaluation work from `AcceptedCandidateResult` from published consequences. Section 6 (Snapshot and Fence Requirements). This document is the OxCalc-side requirement surface that W050's first-call protocol must satisfy.

`docs/spec/core-engine/CORE_ENGINE_OXFML_MINIMAL_UPSTREAM_HOST_INTERFACES.md` — First OxCalc-Owned Upstream Host Packet — 2026-03-26 — **HIGH**
- Documents `MinimalUpstreamHostPacket` (the present `formula_slot` / `binding_world` / `typed_query_facts` / `runtime_catalog` / `verification_publication_context` set). **This is the surface W050 is reworking.** Today the packet is per-formula and ships flattened `cell_fixture` + `defined_name_bindings` synthesised from already-resolved OxCalc state — the very shape the user flagged as wrong. Section 7 explicitly names `src/oxcalc-core/src/upstream_host*.rs` as the code surface.

`docs/spec/core-engine/CORE_ENGINE_DOWNSTREAM_HOST_SEAM_REFERENCE.md` — Downstream Host Seam Reference — 2026-03-29 — **HIGH**
- Authority filter and entry point for downstream hosts. Forbids downstream hosts from inventing private evaluator contracts. Useful for W050 because any new first-call protocol must be expressible inside this authority filter (and DNA OneCalc / DNA TreeCalc must be able to read it without re-interpretation).

`docs/spec/core-engine/CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md` — OxCalcTree Consumer Contract V1 — 2026-04-02 — **HIGH**
- The OxCalc-facing tree-host object set (the analogue of OxFml V1's consumer facade, from the OxCalc side). Hard boundaries: OxFml owns formula-language meaning; OxCalc owns coordinator/publication; consumer packaging does not close TreeCalc residuals. The W050 first-call protocol must compose under this contract (it lives **below** the consumer surface).

`docs/spec/core-engine/CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md` — TreeCalc Semantic Completion Plan — 2026-03-20 — **HIGH**
- Names the target state: hold tree-structured calculation substrate of named nodes; attach real formula artifacts; **consume OxFml-owned parse and bind products rather than test-only scripted evaluation steps**; resolve direct and relative references through OxFml seam plus OxCalc structural truth. Section 4.2 explicitly lists what does not exist yet — exactly the gap W050 must close.

`docs/spec/core-engine/CORE_ENGINE_TREECALC_OXFML_SEAM_NEGOTIATION_MATRIX.md` — TreeCalc / OxFml Seam Negotiation Matrix — 2026-03-21 — **MED**
- Active note-exchange matrix. Topics 4.1 (formula and bind identity), 4.2 (direct and relative reference descriptors), 4.3 (dependency fact carriage), 4.4 (candidate consequences), 4.5 (reject contexts), and the "bind-time-fixed versus context-sensitive" decision are precisely the open W050 sub-questions. Marked temporary-planning, not seam authority — useful as a question list, not a contract.

`docs/spec/core-engine/CORE_ENGINE_FORMAL_MODEL.md` — Core Engine Formal Model (canonical OxCalc copy) — 2026-03-09 — **HIGH**
- Canonical editable copy of the Foundation mirror; same layered S/R/D/V/O semantics. Authoritative inside OxCalc for layer dependencies and overlay identity.

`docs/spec/core-engine/CORE_ENGINE_THEORY_AND_ALTERNATIVE_PATHS.md` — Core Engine Theory (canonical OxCalc copy) — 2026-03-09 — **MED**
- Canonical editable copy of the Foundation theory companion.

`docs/spec/core-engine/CORE_ENGINE_OXCALC_OXFML_FORMALIZATION_PASS_PLAN.md` — OxCalc + OxFml Formalization Pass Plan — 2026-05-04 — **HIGH**
- Most recent canonical OxCalc-led plan covering the seam from the formalisation angle. Section 1A names LET/LAMBDA as the explicit carrier-level boundary exception where OxCalc must reason about OxFml/OxFunc interaction. Relevant to W050 because the user-asked "lift Lambda optimisations into general formula optimisations" is the inverse direction — it makes the W050 generalisation a precondition for the formalisation pass to model uniformly.

`docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md` — Formalization and Assurance — 2026-03-15 — **MED**
- Assurance-side companion: TraceCalc as oracle, refinement statement, pack mapping. Indirectly relevant — W050's protocol must remain testable under this assurance stack.

`docs/spec/core-engine/CORE_ENGINE_REALIZATION_ROADMAP.md` — Realization Roadmap — 2026-03-15 — **MED**
- Sequencing of stages from TraceCalc through TreeCalc to grid. W050 deliverables must be slotted into this roadmap; nothing else here is normative for the seam shape.

`docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md` — TraceCalc Reference Machine — 2026-03-15 — **LOW**
- Defines TraceCalc as the executable correctness oracle. Relevant only as the surface that must keep passing as the W050 rework lands.

`docs/spec/core-engine/CORE_ENGINE_TREECALC_ASSURANCE_AUTHORITY_MAP.md` — TreeCalc Assurance Authority Map — 2026-04-29 — **LOW**
- Authority map across the assurance lanes; not normative for protocol shape.

`docs/LOCAL_EXECUTION_DOCTRINE.md` — Local Execution Doctrine — 2026-03-15 — **MED**
- Lesson 8 ("Engine and Oracle Widening Must Move Together") applies: the W050 protocol change must be exercised by oracle/replay artefacts in the same pass, not as code-only.

### 12.3 Foundation Snapshots Held Inside OxCalc

`docs/spec/core-engine/FOUNDATION_ARCHITECTURE_SNAPSHOT.md` — Foundation Architecture Snapshot — 2026-03-09 — **LOW**
- Snapshot of `../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md` for OxCalc-local reading. Read the Foundation original (8.1) when current text matters.

`docs/spec/core-engine/FOUNDATION_OPERATIONS_SNAPSHOT.md` — Foundation Operations Snapshot — 2026-03-09 — **LOW**
- Snapshot of `../Foundation/OPERATIONS.md`. Same rule: prefer Foundation original.

### 12.4 OxFml Canonical Seam, Runtime, and Consumer Contract

`../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` — Notes For OxCalc (OxFml outbound observation ledger) — 2026-03-19 — **HIGH**
- The current OxFml-side message to OxCalc. Confirms the consumer-facing entry surface OxCalc must build on is `oxfml_core::consumer::{runtime, editor, replay}`. Says `RuntimeEnvironment`, `RuntimeFormulaRequest`, `RuntimeFormulaResult`, `RuntimeSessionFacade` are the intended runtime-facing object set. Confirms provider-plus-pin library-context selection remains first-class. Critically for W050: tells OxCalc not to build new long-term wrappers around the flat OxFml root and to plan against the runtime facade as the intended consumer contract.

`../OxFml/docs/spec/OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md` — Consumer Interface and Facade Contract V1 — 2026-04-01 — **HIGH**
- Frozen `OxFml_V1` consumer contract. Names OxCalc as the "expected most important downstream runtime consumer" and DnaOneCalc as the present-day reference experiential consumer. The W050 first-call protocol is the OxCalc-side complement of this contract — it must reuse rather than wrap the runtime facade.

`../OxFml/docs/spec/OXFML_CONSUMER_INTERFACE_REARCHITECTURE_PLAN.md` — Consumer Interface Rearchitecture Plan — 2026-04-01 — **MED**
- Companion planning doc to the V1 contract. Useful for understanding why the facade shape landed where it did.

`../OxFml/docs/spec/OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md` — Host Runtime and External Requirements — 2026-03-23 — **HIGH**
- The canonical OxFml-owned host/runtime contract. Section 3 names two host modes: "Direct Host Mode" (single-formula, no coordinator — DnaOneCalc fits here) and "OxCalc-Integrated Host Mode" (broader workbook/graph with coordinator). Section 4 lists the required host-supplied inputs: `FormulaSourceRecord`, formula-stable identity, structure-context identity, caller anchor and address-mode context, direct cell bindings where semantic truth depends on concrete resolution, defined-name bindings. Section 4.1A adds host-owned table context. Section 4.2 covers `LibraryContextProvider`/`LibraryContextSnapshot`. Section 5 names the operation chain: parse → project_red_view → bind → compile_semantic_plan → evaluate → commit. **This is the doc against which W050 must redesign**: the current OxCalc per-formula packet is one (verbose) realisation of these required inputs.

`../OxFml/docs/spec/OXFML_PUBLIC_API_AND_RUNTIME_SERVICE_SKETCH.md` — Public API and Runtime Service Sketch — 2026-03-16 — **HIGH**
- Code-facing sketch of the canonical transforms and consumer-facade layers. The five-stage transform chain (parse → red → bind → semantic plan → evaluate → commit) is what a multi-formula OxCalc seam must address holistically rather than re-driving from scratch per formula.

`../OxFml/docs/spec/OXFML_MINIMUM_SEAM_SCHEMAS.md` — Minimum Seam Schemas — 2026-03-16 — **HIGH**
- Minimum typed payloads for `ValueDelta`, `ShapeDelta`, `TopologyDelta`, and reject-context families. Defines what the OxCalc coordinator gets back from a single-node evaluation. The W050 first-call result shape must compose over these.

`../OxFml/docs/spec/OXFML_DELTA_EFFECT_TRACE_AND_REJECT_TAXONOMIES.md` — Delta, Effect, Trace, and Reject Taxonomies — 2026-03-16 — **HIGH**
- Typed taxonomies for value/shape/topology deltas, evaluator-fact families, reject classes, trace events. Read with §12.4 minimum schemas; together they define the per-node result vocabulary OxCalc consumes.

`../OxFml/docs/spec/OXFML_CANONICAL_ARTIFACT_SHAPES.md` — Canonical Artifact Shapes — 2026-03-16 — **HIGH**
- Field surfaces for `FormulaSourceRecord`, green tree root, `BoundFormula`, `SemanticPlan`, and evaluation result. Separates identity/version metadata from semantic payload from diagnostics. The W050 first-call protocol cannot drop or reinterpret these.

`../OxFml/docs/spec/OXFML_ARTIFACT_IDENTITIES_AND_VERSION_KEYS.md` — Artifact Identities and Version Keys — 2026-03-16 — **HIGH**
- Defines the identity vocabulary: `formula_stable_id` (logical identity, survives text edits), `formula_text_version` (declared text revision), `formula_token` (FEC/F3E fence token derived from publish-relevant payload changes), `green_tree_key` (interning/version of an immutable green-tree root), `red_view_key`, `bind_hash`, `semantic_plan_key`. **Directly relevant to the user's "Lambda optimisations as general optimisations" sub-question**: the path to reusing parse/bind/semantic-plan across many evaluations of the same formula is keyed on these identities. The first-call protocol must let OxCalc address artifacts by these keys rather than re-supplying source text each time.

`../OxFml/docs/spec/OXFML_IMPLEMENTATION_SURFACES_AND_STATE_OPTIONS.md` — Implementation Surfaces and State Options — 2026-03-16 — **MED**
- Tradeoff analysis between stateless transform APIs and service-oriented session APIs. The doc allows both shapes as long as artifact identity remains explicit. Useful for W050 because the right OxCalc-side answer is likely a session/repo facade rather than a per-call request.

`../OxFml/docs/spec/OXFML_IMPLEMENTATION_BASELINE.md` — Implementation Baseline — 2026-03-16 — **LOW**
- Code-start baseline. Not normative for seam shape.

`../OxFml/docs/spec/OXFML_SYSTEM_DESIGN.md` — System Design — 2026-03-16 — **LOW**
- Subsystem boundaries; covered more sharply in the seam/runtime docs above.

`../OxFml/docs/spec/OXFML_REPLAY_APPLIANCE_ADAPTER_V1.md` — Replay Appliance Adapter V1 — 2026-03-16 — **MED**
- Defines how OxFml artefacts project into Replay-appliance bundles. Anything W050 changes must remain replay-projectable through this adapter.

`../OxFml/docs/spec/OXFML_FIXTURE_HOST_AND_COORDINATOR_STANDIN_PACKET.md` — Fixture Host and Coordinator Stand-In Packet — 2026-03-16 — **LOW**
- Test-fixture packaging. Relevant only because some current OxCalc TreeCalc fixtures must move to this side under W050 Required Work item 5.

### 12.5 OxFml Formula-Language Realization

`../OxFml/docs/spec/formula-language/OXFML_FORMULA_ENGINE_ARCHITECTURE.md` — Formula Engine Architecture — 2026-03-16 — **HIGH**
- Canonical OxFml architecture: parse text into full-fidelity immutable syntax trees, project versioned contextual views, bind names and references against workbook structure, compile semantic plans, execute through FEC/F3E. Section 4: canonical formula/bind/commit truth must be representable as explicit immutable artifacts; hidden mutable state is optional optimisation state only. **This is the doc that locks "OxFml is the first and only parser"** and tells OxCalc it must supply raw text plus binding context, not pre-parsed AST.

`../OxFml/docs/spec/formula-language/OXFML_PARSER_AND_BINDER_REALIZATION.md` — Parser and Binder Realization — 2026-03-16 — **HIGH**
- Realisation note for parse/bind. Defines green/red boundary, incremental reparse/rebind story, and how immutable formula artifacts fit into a larger immutable workbook/document structure in OxCalc-integrated mode. Says unchanged subtrees should be structurally reusable across formula-text edits — direct support for the W050 multi-formula sharing direction.

`../OxFml/docs/spec/formula-language/OXFML_NORMALIZED_REFERENCE_ADTS.md` — Normalized Reference ADTs — 2026-03-16 — **HIGH**
- Canonical reference normalisation: CellRef, AreaRef, WholeRowRef atoms; reference-expression composition; unresolved and runtime-discovered reference records. **Directly addresses the user's red flag** that OxCalc "already knew about the References before calling OxFml": those references belong on the OxFml side of the boundary as the output of bind, not as inputs constructed by OxCalc.

`../OxFml/docs/spec/formula-language/OXFML_HOST_MANAGED_NAME_AND_EXTERNAL_NAME_BOUNDARY.md` — Host-Managed Name / External-Name Boundary — 2026-03-24 — **HIGH**
- Splits ownership: host owns name objects (lifecycle, scope, storage); OxFml owns parse/bind/semantic-plan/FEC consequences once the request is presented. Defines unresolved-name classification and the rule that bind-visible name-world changes require a `structure_context_version` bump. The W050 protocol must thread the name world through this contract instead of OxCalc pre-resolving references into synthetic A1 cells.

`../OxFml/docs/spec/formula-language/OXFML_NAME_WORLD_AND_RUNTIME_REGISTRATION_INVALIDATION.md` — Name World and Runtime Registration Invalidation — 2026-03-26 — **MED**
- Invariant: changes to bind-visible name world (function catalog, defined names, registered externals) trigger structural rebind. Tells OxCalc how to invalidate when names or registrations move.

`../OxFml/docs/spec/formula-language/OXFML_STRUCTURED_REFERENCE_AND_TABLE_BOUNDARY.md` — Structured Reference and Table Boundary — 2026-03-24 — **MED**
- Host owns tables (catalog, range, columns, header/totals presence); OxFml owns grammar and bind consequences. Minimum first packet: `table_catalog`, `enclosing_table_ref`, `caller_table_region`. Relevant when W050 widens beyond TreeCalc-only nodes.

`../OxFml/docs/spec/formula-language/OXFML_R1C1_FORMULA_CHANNEL.md` — R1C1 Formula Channel — 2026-03-23 — **LOW**
- Channel-sensitive translation during bind. Not currently load-bearing for W050.

`../OxFml/docs/spec/formula-language/OXFML_REGISTERED_EXTERNAL_PROVIDER_AND_CALL_REGISTER_ID_BOUNDARY.md` — Registered External Provider Boundary — 2026-03-26 — **LOW**
- CALL / REGISTER.ID surface. Relevant only when W050 widens to user-defined registered externals.

`../OxFml/docs/spec/formula-language/OXFML_CF_DV_RESTRICTED_SUBLANGUAGES.md` — CF/DV Restricted Sublanguages — n/a — **LOW**
- Conditional-formatting and data-validation sublanguages. Out of W050 scope but listed so it is not re-investigated.

`../OxFml/docs/spec/formula-language/OXFML_EDITOR_LANGUAGE_SERVICE_AND_HOST_INTEGRATION_PLAN.md` — Editor Language Service Plan — 2026-03-24 — **LOW**
- Editor facade. Relevant only insofar as DnaOneCalc consumes it (see §12.9 comparator).

### 12.6 OxFml / OxFunc Semantic and Library-Context Boundary

`../OxFml/docs/spec/formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md` — OxFml / OxFunc Semantic Boundary — 2026-03-16 — **HIGH**
- Canonical semantic distinctions OxFml must preserve when calling OxFunc kernels: scalar vs array, value-only vs reference-observable, caller-context-sensitive scalarisation, locale/format dependencies, typed host-query capabilities. Bounds what OxFml needs from OxCalc as inputs and what comes back. W050 must keep OxCalc on the "preserve these distinctions" side, not collapse them in flattening.

`../OxFml/docs/spec/formula-language/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md` — OxFml/OxFunc Shared Interface Freeze Candidate V1 — n/a — **MED**
- Freeze candidate for the OxFml/OxFunc shared interface. The W050 rework must not perturb this seam — it is frozen.

`../OxFml/docs/spec/formula-language/OXFML_OXFUNC_LIBRARY_CONTEXT_RUNTIME_INTERFACE.md` — Library Context Runtime Interface — 2026-03-22 — **HIGH**
- Defines `LibraryContextProvider` and immutable versioned `LibraryContextSnapshot`. The preferred shape: OxFunc owns catalog truth; OxFml consumes immutable snapshots through a runtime interface; OxCalc plumbs the provider in. Directly relevant to W050 because the same provider can serve many formulas — it is a session-shaped seam, not a per-formula one. Reinforces the move away from per-call flattening.

`../OxFml/docs/spec/formula-language/OXFML_OXFUNC_EVALUATION_ADAPTER_AND_TEST_ARTIFACTS.md` — Evaluation Adapter and Test Artifacts — 2026-03-26 — **MED**
- Adapter test surface (W049). Useful as a worked example of an end-to-end formula text → preparation artifact → evaluation artifact flow.

`../OxFml/docs/spec/formula-language/OXFML_OXFUNC_LET_LAMBDA_PIN_DOWN_PREP.md` — LET / LAMBDA Pin-Down Prep — 2026-03-19 — **HIGH**
- **Directly addresses the user's "lift Lambda optimisations into general formula optimisations" goal.** Documents the OxFml-exercised floor for `LET` sequential binding, `LAMBDA` literal formation, immediate and helper-bound invocation, lexical (not dynamic) capture, callable values as semantic first-class, and higher-order helpers (`MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, `MAKEARRAY`). The semantic machinery for "same compiled formula evaluated many times with different inputs" already exists inside OxFml as lambda invocation; W050 needs to expose that machinery (or its prepared-call equivalent) at the OxFml consumer surface so OxCalc can drive repeated evaluation of an ordinary node formula without re-parsing/binding.

### 12.7 FEC/F3E Evaluator Seam Specification

`../OxFml/docs/spec/fec-f3e/FEC_F3E_DESIGN_SPEC.md` — FEC/F3E Design Specification — 2026-03-16 — **HIGH**
- Canonical OxFml-owned evaluator seam contract with OxCalc. Session lifecycle: prepare → open_session → capability_view → execute → commit. Defines capability requirements, commit bundle shape, typed trace/reject schemas, dynamic-reference/spill/format overlay participation. OxCalc owns publication fencing and global recalc policy. The session shape is the natural place for multi-formula reuse — W050's protocol should be expressible as a sequence of session operations over the same underlying repository.

`../OxFml/docs/spec/fec-f3e/FEC_F3E_FORMAL_AND_ASSURANCE_MAP.md` — FEC/F3E Formal and Assurance Map — 2026-03-16 — **MED**
- Maps the seam to formal/assurance evidence families.

`../OxFml/docs/spec/fec-f3e/FEC_F3E_TESTING_AND_REPLAY.md` — FEC/F3E Testing and Replay — 2026-03-16 — **MED**
- Testing strategy and replay pack planning for the seam.

`../OxFml/docs/spec/fec-f3e/FEC_F3E_SCHEMA_REPLAY_FIXTURE_PLAN.md` — FEC/F3E Schema Replay Fixture Plan — 2026-03-16 — **LOW**
- Fixture planning detail.

`docs/spec/fec-f3e/FEC_F3E_REDESIGN_SPEC_MIRROR.md` — FEC/F3E Redesign Spec (OxCalc mirror) — n/a — **LOW**
- Mirror of OxFml canonical FEC/F3E spec inside OxCalc. Read the OxFml canonical instead.

### 12.8 Cross-Repo Handoffs

`docs/handoffs/HANDOFF_CALC_001_OXFML_COORDINATOR_SEAM_HARDENING.md` — CALC-001: Coordinator-Facing Seam Hardening — 2026-03-15 — **HIGH**
- The active handoff requesting canonical OxFml-side seam tightening for accepted-result payload structure, structured reject detail, and snapshot/token/capability fence consequences. The shape the OxCalc coordinator needs back from a single-node evaluation is described here. W050 should re-read this packet and decide whether the requested clauses change once the per-formula model is replaced by a session/repo model.

`docs/handoffs/HANDOFF_CALC_001_OXFML_RECEIPT.md` — CALC-001 Receipt — 2026-03-15 — **MED**
- OxFml-side receipt acknowledging the CALC-001 ask.

`docs/handoffs/HANDOFF_FML_001_OXCALC_RECEIPT.md` — FML-001 Receipt — 2026-03-18 — **MED**
- OxCalc-side receipt of an OxFml-originated handoff. Status checkpoint.

`docs/handoffs/HANDOFF_REGISTER.csv` — Handoff Register — n/a — **MED**
- Tracking register; W050 must register a new handoff once the new protocol moves canonical seam text.

### 12.9 Comparator — DnaOneCalc OxFml Integration Pattern

DnaOneCalc is the present-day reference consumer for the OxFml `consumer::runtime` facade, and the OxFml V1 contract names it as the highest-weighted experiential consumer. Documenting its pattern here makes the W050 design choices concrete.

Code reference points (in `../DnaOneCalc/src/dnaonecalc-host/`):

- `src/adapters/oxfml/live_bridge.rs` is the canonical bridge. The per-formula run sits at line 232: `RuntimeEnvironment::new().execute(runtime_request).ok()` — DnaOneCalc uses the same OxFml entry point OxCalc uses, but with a different surrounding shape.
- `LiveOxfmlBridge` keeps `last_result` and a `cached_documents` map keyed on `FormulaEditFingerprint`, so that the same formula text + cursor + formatting + scenario policy hits the cache without re-running parse / bind / runtime.
- `FormulaSourceRecord` is constructed from the entered text plus a `formula_stable_id`. No pre-resolved cell values are flattened into the request: DnaOneCalc passes `BindContext::default()` and lets OxFml drive reference meaning through its editor environment.
- Library context and locale/scenario inputs go in through the `TypedContextQueryBundle` and the runtime-locale builder.

Contrast against OxCalc today (`src/oxcalc-core/src/upstream_host.rs` and `src/oxcalc-core/src/treecalc.rs`):

- OxCalc builds one `MinimalUpstreamHostPacket` per formula, flattens OxCalc-known cell values into a synthetic A1 `cell_fixture` and synthesises `ReferenceLike` defined-name bindings, then makes a fresh `RuntimeEnvironment::new().execute(...)` call per node.
- There is no per-formula artifact reuse across nodes or across recalc waves. Even formulas with identical text but different caller anchors re-walk parse → red → bind → compile_semantic_plan from scratch on each call.
- OxCalc additionally translates its own `TreeFormula` AST to source text immediately before the call (`prepare_oxfml_formula` → `translate_formula`), which is the surface W050 deletes outright.

W050 design implications (drawn from the comparator):

1. The OxFml entry point (`RuntimeEnvironment` / `RuntimeFormulaRequest` / `RuntimeSessionFacade`) is correct; what is wrong is the OxCalc framing around it.
2. The "first-call" should treat OxFml as a stateful (or repo-shaped) consumer surface across the whole recalc wave, not as a one-shot per cell. Artifact-identity keys (`formula_stable_id`, `formula_text_version`, `formula_token`, `green_tree_key`, `bind_hash`, `semantic_plan_key`) are the reuse currency.
3. References should be carried as the inputs to OxFml's bind step (caller anchor, name world, table catalog, library-context provider), **not** as values OxCalc has already resolved and re-flattened. Resolution belongs inside OxFml; runtime-observed dependencies come back to OxCalc as typed `topology_delta` facts.
4. Repeated evaluation of the same formula with changing inputs is structurally the same problem OxFml already solves inside lambda invocation. The W050 design should either generalise lambda invocation's prepared-call reuse to the consumer surface, or expose a session API that pins parse/red/bind/semantic-plan and re-runs `evaluate` per input. The OxFml side documents that allow this generalisation are §12.6 `OXFML_OXFUNC_LET_LAMBDA_PIN_DOWN_PREP.md` and §12.5 `OXFML_PARSER_AND_BINDER_REALIZATION.md` (incremental parse/bind reuse).

### 12.10 Open Cross-References

The following docs are referenced from this section but are tracked elsewhere as their own active surfaces. Read them in their canonical home, not as part of W050:

- `docs/WORKSET_REGISTER.md` — current workset ledger; W050 is listed here.
- `docs/BEADS.md` — local bead surface; the `calc-cwpl.*` beads in §6 above are tracked here.
- `docs/IN_PROGRESS_FEATURE_WORKLIST.md` — wider in-progress feature view.
- `docs/upstream/` (if present) — inbound observation ledgers from downstream consumer repos to OxCalc.
