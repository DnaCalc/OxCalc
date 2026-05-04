# W034 Residual Obligation And Authority Ledger

Status: `calc-e77.1_residual_ledger`
Workset: `W034`
Parent epic: `calc-e77`
Bead: `calc-e77.1`

## 1. Objective Restatement

The active formalization objective is broader than W033's first-pass scope. It asks OxCalc to keep running post-W033 successor work, add follow-up beads where needed, and continue until the core engine has full formalization, formal proofs/checks, TLA+ verification, Lean verification, TraceCalc oracle coverage, and optimized/core-engine conformance evidence.

Current evidence does not satisfy that full objective. W033 and its post-W033 successor beads created a strong first tranche, but several lanes remain partial. W034 is the next ordered tranche that turns those residuals into explicit executable work rather than leaving them as prose memory.

## 2. Authority Inputs Reviewed

| Input | Role in W034 |
|---|---|
| `CHARTER.md` and `OPERATIONS.md` | OxCalc-local scope, completion doctrine, staged realization, handoff and promotion rules. |
| `../Foundation/CHARTER.md`, `../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md`, `../Foundation/OPERATIONS.md` | Higher-precedence doctrine for proof/model/replay coupling, pack gates, formatting semantics, Stage 2, and no hidden mutation pathways. |
| `docs/WORKSET_REGISTER.md` | Ordered workset truth, now including W034. |
| `docs/BEADS.md` and `.beads/issues.jsonl` | Bead mutation and execution-state truth. |
| `docs/worksets/W033_OXCALC_OXFML_CORE_FORMALIZATION_PASS.md` | W033 scope and closure packetization model. |
| `docs/worksets/W034_CORE_FORMALIZATION_DEEPENING_AND_IMPLEMENTATION_VERIFICATION.md` | W034 scope and gate model. |
| `docs/spec/core-engine/w033-formalization/*` | W033 and post-W033 evidence packets, residuals, handoff/watch rows, and no-promotion decisions. |
| `docs/spec/core-engine/CORE_ENGINE_OXCALC_OXFML_FORMALIZATION_PASS_PLAN.md` | Formalization intent and TraceCalc-as-oracle framing. |
| `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md` | Core formalization and assurance positioning. |
| `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md` | TraceCalc reference-machine authority boundary. |
| `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md` | OxFml/FEC/F3E seam contract as consumed by OxCalc. |
| `formal/lean/OxCalc/CoreEngine/*.lean` | Current checked Lean surface: Stage 1, W033 first slice, and W033 post slice. |
| `formal/tla/CoreEngineStage1.tla`, `formal/tla/CoreEnginePostW033.tla` | Current checked TLA+ surface. |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | Current inbound OxFml observation ledger, including formatting/display seam updates. |
| `../OxFml/docs/spec/` | Read-only OxFml canonical spec inputs for FEC/F3E, formatting, replay, callable, and consumer surfaces. |

Reviewed inbound observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`.

## 3. Prompt-To-Artifact Checklist

| Explicit requirement | Current evidence | W034 disposition |
|---|---|---|
| Run post-W033 successor beads sequentially | `calc-8pe`, `calc-688`, `calc-y0r`, `calc-lwh`, `calc-rcr`, and `calc-8lg` are closed with evidence packets under `docs/spec/core-engine/w033-formalization/`. | Satisfied for the W033 successor set; W034 is the follow-up tranche. |
| Add follow-up beads until the formalization goal is reached | `calc-e77` and child beads `calc-e77.1` through `calc-e77.7` exist in `.beads/`. | Active W034 path. |
| Full formalization of the core engine | Current Lean/TLA/proof/replay packets are bounded first slices and post-W033 slices. | Not reached; owned by `calc-e77.2` through `calc-e77.7`. |
| TraceCalc verified as correctness oracle | W033 oracle self-check and LET/LAMBDA witness runs exist; coverage is explicitly bounded. | Partial; `calc-e77.2` widens oracle coverage. |
| Optimized/core-engine implementation verified | W033 independent conformance emits first comparison rows and declared gaps; `calc-e77.3` widens this to W034 with 15 comparison rows, 0 missing artifacts, and 0 unexpected mismatches. | Partial at W034 scope; Lean/TLA, pack/capability, continuous scale, and full independent evaluator diversity remain open. |
| Full TLA verification | `CoreEngineStage1` and `CoreEnginePostW033` smoke models pass, with Stage 2 unpromoted. | Partial; `calc-e77.5` owns deeper TLA and contention preconditions. |
| Full Lean verification | `Stage1State`, `W033FirstSlice`, and `W033PostSlice` check; broader module split remains open. | Partial; `calc-e77.4` owns deeper Lean proof family. |
| Pack-grade replay and capability claims | `W033_PACK_CAPABILITY_POST_W033_DECISION.md` records `cap.C5.pack_valid` not promoted; W034 gate binding adds `w034-pack-capability-gate-binding-001` with 7 satisfied inputs, 12 blockers, and no missing artifacts. | Partial/no-promotion; C5 remains unpromoted and closure audit remains in `calc-e77.7`. |
| Scaling and performance as correctness evidence | W034 gate binding adds `w034-continuous-scale-gate-binding-001` with 7 validated scale rows, 5 scale signature rows, 4 replay/conformance/pack binding rows, 0 missing artifacts, and 0 unexpected mismatches. | Partial/no-promotion; continuous scale assurance remains unpromoted because scheduled regression and continuous cross-engine differential criteria are missing. |
| OxFml updates related to formatting incorporated if needed | W034 plan, this ledger, and the gate-binding packet record `format_delta`/`display_delta`, formatting-sensitive facts, and W073 typed-only conditional-formatting input direction as seam-watch/input-contract evidence. | Watch lane; no OxCalc patch or handoff now because no W034 artifact constructs those payloads. |

## 4. Residual Obligation Matrix

| Obligation id | Area | Current floor | Open risk or residual | W034 owner | Closure evidence expected |
|---|---|---|---|---|---|
| `W034-OBL-001` | TraceCalc stale-fence and reject oracle depth | `tc_publication_fence_reject_001` and W033 self-checks pass. | Matrix breadth for stale tokens, compatibility basis, capability tokens, and no-publish combinations remains thin. | `calc-e77.2` | New TraceCalc scenarios and run artifacts with explicit assertion coverage. |
| `W034-OBL-002` | TraceCalc dynamic dependency negative cases | W033 seed dynamic cases and scale semantic binding exist. | Need negative cases for missing runtime deps, over/under-invalidation classification, and soft-reference update boundaries. | `calc-e77.2` | Widened oracle run with dependency fact assertions. |
| `W034-OBL-003` | TraceCalc overlay retention and eviction pressure | W033 overlay retention and protected overlay smoke model exist. | Need broader eviction/retention pressure beyond reflexive/smoke facts. | `calc-e77.2`, `calc-e77.5` | TraceCalc cases plus TLA overlay/pinned-reader checks. |
| `W034-OBL-004` | `LET`/`LAMBDA` carrier breadth | Post-W033 carrier witnesses cover origin, capture, arity, dependency visibility, runtime-effect visibility, and replay identity. | Higher-order callable families, callable publication policy, and structured TreeFormula coverage remain limited. | `calc-e77.2`, `calc-e77.3`, `calc-e77.4` | Additional oracle/conformance/proof rows or explicit deferral. |
| `W034-OBL-005` | Optimized/CoreEngine conformance | W033 independent conformance has 3 exact value matches, 2 no-publication matches, 2 declared gaps, and 0 unexpected mismatches. | Dynamic dependency and host-sensitive lambda projection gaps are retained; continuous differential coverage is absent. | `calc-e77.3` | Comparison artifacts classifying matches, declared gaps, and any mismatch. |
| `W034-OBL-006` | Fully independent evaluator diversity | W033 explicitly says current evidence is not fully independent evaluator implementation diversity. | TraceCalc and TreeCalc are distinct surfaces but not fully independent evaluator implementations. | `calc-e77.3`, `calc-e77.6` | Either a concrete wider independent surface or a no-promotion blocker. |
| `W034-OBL-007` | Lean proof-family depth | `W033PostSlice.lean` checks bridge/fence/dependency/overlay/carrier/replay facts. | Broader module split, refinement obligations, and proof depth remain open. | `calc-e77.4` | Checked Lean files plus proof-obligation map. |
| `W034-OBL-008` | TLA+ model-family depth | `CoreEngineStage1` and `CoreEnginePostW033` smoke configs pass. | Stale-fence interleavings, overlay GC, dependency update interleavings, and Stage 2 contention preconditions remain open. | `calc-e77.5` | Checked TLA models/configs, TLC output summary, and explicit model limits. |
| `W034-OBL-009` | Stage 2 contention | W033 records no Stage 2 contention promotion. | Need precondition and failure-mode modeling before any concurrency policy promotion. | `calc-e77.5`, `calc-e77.6` | TLA/replay/precondition packet; no promotion unless gates satisfy OPERATIONS Section 3/6. |
| `W034-OBL-010` | Pack-grade replay capability | `cap.C5.pack_valid` remains unpromoted with eight blockers. | Program-grade governance, continuous diff, direct evaluator re-execution, and TreeCalc C4/C5 remain unresolved. | `calc-e77.6` | Machine-readable pack decision with exact blockers or promotion evidence. |
| `W034-OBL-011` | Continuous scale assurance | Scale semantic binding validates seven scale rows and five metamorphic rows. | Single-baseline scale evidence is not continuous assurance or performance correctness proof. | `calc-e77.6` | Continuous scale criteria and no-promotion/promotion decision tied to semantic evidence. |
| `W034-OBL-012` | Direct OxFml fixture depth | OxFml bridge projects 45 fixture cases with 6 current matches and 39 deferred current counterparts. | Prepared-call, session lifecycle, and higher-order callable counterparts remain broad. | `calc-e77.2`, `calc-e77.3` | More exact counterpart rows or deferred/handoff classification. |
| `W034-OBL-013` | OxFml formatting/display seam | OxFml ledger says `format_delta` and `display_delta` are distinct and display breadth is narrower. | Risk of collapsing semantic format, display, and renderer-only state in later conformance/pack work. | `calc-e77.1`, later bead where exercised | Watch row; handoff only on concrete mismatch. |
| `W034-OBL-014` | OxFml W073 typed conditional-formatting payload | OxFml W073 changes aggregate/visualization CF option metadata to typed input; OxCalc has no current local request-construction path. | Future OxCalc fixture or conformance work could accidentally rely on W072 string-threshold metadata. | `calc-e77.1`, later bead where exercised | Watch row; if exercised, update local request construction or file handoff based on concrete evidence. |
| `W034-OBL-015` | Spec evolution discipline | W033 established decision taxonomy and historical no-loss review. | W034 proof/model work may expose stale or vague OxCalc specs. | every W034 child bead; audit in `calc-e77.7` | Spec patch, implementation bead, handoff/watch row, or deferred rationale. |
| `W034-OBL-016` | Evidence non-mutation | W033 relied on checked-in run roots and validation non-mutation doctrine. | W034 execution must not mutate baselines accidentally. | every W034 child bead | Declared run ids and validation commands; no accidental baseline rewrite. |

## 5. OxFml Formatting And Display Watch Rows

| Watch id | Source | Current read | W034 action |
|---|---|---|---|
| `W034-HW-001` | `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` Sections 10.5, 16B.4, 22 | `format_delta` and `display_delta` are distinct canonical seam categories; broader display closure is not assumed. | Keep as guardrail for conformance, replay, pack, and publication-surface artifacts. |
| `W034-HW-002` | `../OxFml/docs/spec/OXFML_DELTA_EFFECT_TRACE_AND_REJECT_TAXONOMIES.md` | `format_delta` carries semantic formatting consequences; `display_delta` is optional and only for explicit seam obligations. | Do not invent renderer-only display obligations in OxCalc. |
| `W034-HW-003` | `../OxFml/docs/spec/formatting/EXCEL_FORMATTING_HIERARCHY_AND_VISIBILITY_MODEL.md` | Formula-visible formatting and conditional-format observability remain bounded/provisional. | Treat formatting-sensitive inputs as evidence obligations only when surfaced by OxFml or exercised by W034 artifacts. |
| `W034-HW-004` | OxFml W073 working state and handoff text | Aggregate and visualization conditional-formatting metadata is typed-only for current W073 families. | No current OxCalc patch; if later W034 code constructs those payloads, require typed metadata and do not rely on bounded threshold strings. |

Current handoff state: no new OxFml handoff is filed by `calc-e77.1`, because no W034 artifact has exercised a concrete contradiction. The watch rows become handoff candidates only if later W034 evidence exposes a specific upstream insufficiency.

## 5A. Post-`calc-e77.3` Conformance Update

The W034 independent conformance widening packet is now recorded at `docs/spec/core-engine/w034-formalization/W034_INDEPENDENT_CONFORMANCE_WIDENING.md`.

Relevant residual effects:

1. `W034-OBL-004`: W034 higher-order `LET`/`LAMBDA` has a TreeCalc-local value counterpart; returned callable identity metadata remains a declared local projection gap.
2. `W034-OBL-005`: W034 conformance artifacts classify 5 exact value matches, 3 no-publication matches, 1 lifecycle match, 6 declared local gaps, 0 missing artifacts, and 0 unexpected mismatches.
3. `W034-OBL-006`: the widened comparison still does not promote fully independent evaluator implementation diversity.
4. `W034-OBL-012`: a W034 Raw OxFml higher-order value fixture exists locally; broader direct OxFml fixture bridge depth remains open.

## 5B. Post-`calc-e77.4` Lean Proof-Family Update

The W034 Lean proof-family packet is now recorded at `docs/spec/core-engine/w034-formalization/W034_LEAN_PROOF_FAMILY_DEEPENING.md`.

Relevant residual effects:

1. `W034-OBL-004`: the Lean surface now separates value-surface refinement from full callable metadata refinement for the higher-order `LET`/`LAMBDA` carrier lane.
2. `W034-OBL-007`: checked adjacent Lean files now cover publication fences, dependency closure, overlay safety, LET/LAMBDA carrier/replay facts, and refinement classification.
3. `W034-OBL-013` and `W034-OBL-014`: OxFml formatting/display and W073 typed-only conditional-formatting input direction remain watch rows, not local OxCalc code patches in this bead.
4. Full Lean verification, imported OxFml formal linkage, TLA contention modeling, pack-grade replay, and Stage 2 promotion remain open lanes.

## 5C. Post-`calc-e77.5` TLA Model-Family Update

The W034 TLA model-family packet is now recorded at `docs/spec/core-engine/w034-formalization/W034_TLA_MODEL_FAMILY_AND_CONTENTION_PRECONDITIONS.md`.

Relevant residual effects:

1. `W034-OBL-003`: protected overlay retention and release/eviction safety now has a W034 checked TLA interleaving slice.
2. `W034-OBL-008`: checked W034 TLA artifacts now cover stale-fence, dependency-update, overlay, and contention-precondition invariants.
3. `W034-OBL-009`: Stage 2 contention remains explicitly blocked by missing evidence preconditions and is not promoted.
4. Full TLA+ verification, pack-grade replay, continuous scale assurance, production scheduler equivalence, and any Stage 2 policy promotion remain open lanes.

## 5D. Post-`calc-e77.6` Pack And Scale Gate-Binding Update

The W034 pack/scale gate-binding packet is now recorded at `docs/spec/core-engine/w034-formalization/W034_PACK_CAPABILITY_AND_CONTINUOUS_SCALE_GATE_BINDING.md`.

Generated evidence roots:

1. `docs/test-runs/core-engine/pack-capability/w034-pack-capability-gate-binding-001/`
2. `docs/test-runs/core-engine/metamorphic-scale-semantic-binding/w034-continuous-scale-gate-binding-001/`

Relevant residual effects:

1. `W034-OBL-006`: fully independent evaluator diversity remains a no-promotion blocker.
2. `W034-OBL-009`: Stage 2 contention remains blocked by bounded precondition evidence and is not promoted.
3. `W034-OBL-010`: W034 pack gate emits `capability_not_promoted`, highest honest capability `cap.C4.distill_valid`, 7 satisfied inputs, 12 blockers, and 0 missing artifacts.
4. `W034-OBL-011`: W034 scale gate records 7 validated scale rows, 5 metamorphic signature rows, 4 replay/conformance/pack binding rows, and 0 unexpected mismatches, while scheduled regression and continuous cross-engine differential criteria remain missing.
5. `W034-OBL-013` and `W034-OBL-014`: OxFml formatting/display and W073 typed-only conditional-formatting input direction are carried into the gate packet as watch/input-contract evidence; no OxCalc request-construction path or handoff trigger is exercised.
6. Pack-grade replay, continuous scale assurance, full Lean/TLA verification, production scheduler equivalence, and Stage 2 policy promotion remain open lanes for audit/successor packetization.

## 6. Bead Mapping

| Bead | Obligations owned |
|---|---|
| `calc-e77.2` | `W034-OBL-001`, `W034-OBL-002`, `W034-OBL-003`, `W034-OBL-004`, `W034-OBL-012` |
| `calc-e77.3` | `W034-OBL-004`, `W034-OBL-005`, `W034-OBL-006`, `W034-OBL-012` |
| `calc-e77.4` | `W034-OBL-004`, `W034-OBL-007` |
| `calc-e77.5` | `W034-OBL-003`, `W034-OBL-008`, `W034-OBL-009` |
| `calc-e77.6` | `W034-OBL-006`, `W034-OBL-009`, `W034-OBL-010`, `W034-OBL-011` |
| `calc-e77.7` | `W034-OBL-015`, `W034-OBL-016`, all open-lane audit rows |

No additional W034 child bead is required by this ledger yet. The existing W034 child path covers every identified residual. Later beads may split only after a child bead finds disjoint work that cannot be responsibly closed inside its declared target.

## 7. Semantic-Equivalence Statement

This bead adds a residual/authority ledger and updates W034 planning documentation only. It does not change coordinator scheduling, invalidation strategy, publication semantics, reject policy, TraceCalc behavior, TreeCalc behavior, OxFml fixture content, formal model semantics, or pack decision logic.

Observable runtime behavior is invariant under this bead because it introduces no runtime producer, evaluator, coordinator transition, replay runner, formal theorem, TLA action, or fixture expectation change.

## 8. Verification

Commands run:

| Command | Result |
|---|---|
| `scripts/check-worksets.ps1` | passed |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

No cargo, Lean, or TLC command is required for this documentation/authority-ledger bead because it emits no code, formal model, fixture, or replay artifact. Later W034 execution beads must run their own scoped validation.

## 9. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for in-scope items? | yes; this ledger records W034 authority inputs, residuals, handoff/watch rows, bead ownership, and limits |
| 2 | Pack expectations updated for affected packs? | yes; pack-grade and continuous scale obligations are mapped to `calc-e77.6` without promotion |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for carried W033/post-W033 inputs cited by this ledger; this bead emits no new behavior |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 7 records that no runtime strategy or policy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; formatting/display and W073 payload rows are watch rows, and no concrete handoff trigger is present |
| 6 | All required tests pass? | yes for this ledger scope; see Section 8 |
| 7 | No known semantic gaps remain in declared target? | yes for this ledger target; all identified residuals are mapped to W034 beads or watch rows |
| 8 | Completion language audit passed? | yes; this ledger keeps full formalization, pack-grade replay, continuous scale assurance, and Stage 2 promotion partial/unpromoted |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; W034 was registered before this bead started |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; W034 is recorded as the active formalization successor tranche |
| 11 | execution-state blocker surface updated? | yes; `calc-e77.1` through `calc-e77.7` exist in `.beads/` |

## 10. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-e77.1` asks for a residual obligation ledger from W033 successor packets, current specs, inbound OxFml notes, formatting/display updates, and the active objective |
| Gate criteria re-read | pass; every identified formalization/proof/conformance/Stage 2/pack/formatting seam pressure maps to a W034 child bead or handoff/watch row |
| Silent scope reduction check | pass; this ledger explicitly states that full formalization, full Lean/TLA verification, full TraceCalc oracle coverage, optimized/core-engine verification, pack-grade replay, continuous scale assurance, and Stage 2 promotion remain partial |
| "Looks done but is not" pattern check | pass; W033/post-W033 evidence is treated as first-tranche input, not as proof of full formalization |
| Result | pass for the `calc-e77.1` residual-obligation ledger target |

## 11. Three-Axis Report

- execution_state: `calc-e77.1_residual_ledger_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-e77.2` TraceCalc oracle deepening
  - `calc-e77.3` optimized/core-engine conformance widening
  - `calc-e77.4` Lean proof-family deepening
  - `calc-e77.5` TLA model-family and contention precondition slice
  - `calc-e77.6` pack capability and continuous scale gate binding
  - `calc-e77.7` W034 closure audit and successor packetization
