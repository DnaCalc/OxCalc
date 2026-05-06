# W045 Residual Release-Grade Successor Obligation And Current OxFml Intake Map

Status: `calc-zkio.1_residual_successor_obligation_current_oxfml_intake_map_validated`
Workset: `W045`
Parent epic: `calc-zkio`
Bead: `calc-zkio.1`

## 1. Purpose

This packet converts the W044 non-promoting closure decision into the first W045 successor obligation map.

The target is not release-grade promotion. The target is to make the next tranche exact before `calc-zkio.2` starts: every W044 residual lane has a W045 owner bead, required direct evidence, promotion consequence, no-proxy guard, current OxFml public-surface and W073 formatting intake, and a spec-evolution route.

This packet also records the current OxFml formatting update as active W045 intake. W073 aggregate and visualization conditional-formatting metadata is typed-only. `VerificationConditionalFormattingRule.typed_rule` is the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`; W072 bounded `thresholds` strings are intentionally ignored for those families. DNA OneCalc typed-rule request construction remains required downstream uptake and is not verified by OxCalc.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W045_CORE_FORMALIZATION_RELEASE_GRADE_SERVICE_AND_CROSS_REPO_UPTAKE_VERIFICATION.md` | W045 scope, gate model, bead rollout, and release-grade guard |
| `docs/spec/core-engine/w044-formalization/W044_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md` | predecessor release-grade decision and W045 successor lanes |
| `docs/test-runs/core-engine/closure-audit/w044-closure-audit-release-grade-verification-decision-001/residual_lane_ledger.json` | 22 post-W044 residual lanes |
| `docs/test-runs/core-engine/closure-audit/w044-closure-audit-release-grade-verification-decision-001/release_decision.json` | W044 no-promotion release decision |
| `docs/test-runs/core-engine/release-grade-ledger/w044-residual-release-grade-blocker-reclassification-map-001/*` | predecessor residual-map and OxFml-intake shape |
| `docs/test-runs/core-engine/oxfml-seam/w044-oxfml-public-migration-typed-formatting-callable-registered-external-001/run_summary.json` | current consumed OxFml seam and W073 fixture-side evidence |
| `docs/test-runs/core-engine/pack-capability/w044-pack-grade-replay-governance-service-c5-reassessment-001/decision/pack_capability_decision.json` | W044 pack/C5 no-promotion decision |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | inbound OxFml observation ledger, including current public surface |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | W073 typed-only aggregate/visualization input contract |
| `../OxFml/docs/handoffs/HANDOFF-DNAONECALC-012_W073_TYPED_CF_PAYLOAD_FIRST_SLICE.md` | downstream typed-rule request-construction handoff |
| `../OxFml/crates/oxfml_core/src/publication/mod.rs` working-copy diff | current evaluator-side direct typed-only replacement |
| `../OxFml/crates/oxfml_core/tests/conditional_formatting_array_tests.rs` working-copy diff | typed-payload coverage and old bounded-string non-interpretation evidence |

OxFml was inspected read-only. Its local state was `main...origin/main [ahead 3]` with dirty formatting-related changes in the W073 publication/tests/docs lane.

## 3. Artifact Surface

Run id: `w045-residual-release-grade-successor-obligation-current-oxfml-intake-map-001`

| Artifact | Result |
|---|---|
| `docs/test-runs/core-engine/release-grade-ledger/w045-residual-release-grade-successor-obligation-current-oxfml-intake-map-001/run_summary.json` | records W045 ledger status, counts, W073 intake, public-surface intake, and no-promotion claims |
| `docs/test-runs/core-engine/release-grade-ledger/w045-residual-release-grade-successor-obligation-current-oxfml-intake-map-001/successor_obligation_map.json` | maps W044 residual lanes to W045 owner beads, 36 obligations, required disposition, and consequences |
| `docs/test-runs/core-engine/release-grade-ledger/w045-residual-release-grade-successor-obligation-current-oxfml-intake-map-001/promotion_contract_map.json` | maps 18 release-grade, service, seam, scaling, pack/C5, and OxFunc-boundary promotion contracts |
| `docs/test-runs/core-engine/release-grade-ledger/w045-residual-release-grade-successor-obligation-current-oxfml-intake-map-001/source_evidence_index.json` | names predecessor artifacts and inbound OxFml observation surfaces |
| `docs/test-runs/core-engine/release-grade-ledger/w045-residual-release-grade-successor-obligation-current-oxfml-intake-map-001/oxfml_inbound_observation_intake.json` | records W073, public-surface, and note-level watch/handoff trigger lanes |
| `docs/test-runs/core-engine/release-grade-ledger/w045-residual-release-grade-successor-obligation-current-oxfml-intake-map-001/validation.json` | records validation for this ledger packet |

## 4. Successor Obligation Map

| Residual lane | W045 owner | Required disposition |
|---|---|---|
| release-grade full verification | `calc-zkio.1`, `calc-zkio.11` | direct-evidence-only release decision with current OxFml intake and spec-evolution hooks |
| optimized/core dynamic dependency transitions | `calc-zkio.2` | broaden dynamic add/remove/reclassify, soft-reference, and `INDIRECT` coverage or retain exact blockers |
| snapshot/capability counterparts | `calc-zkio.2`, `calc-zkio.5` | prove snapshot-fence and capability-view counterparts for promoted profiles or retain blockers |
| callable metadata projection | `calc-zkio.2`, `calc-zkio.8` | implement and exercise metadata projection or retain exact blocker |
| callable carrier sufficiency | `calc-zkio.8` | prove narrow `LET`/`LAMBDA` carrier sufficiency without general OxFunc overclaim |
| Rust totality/refinement/panic surface | `calc-zkio.3` | discharge or retain totality, refinement, and panic-surface blockers |
| Lean/TLA full verification | `calc-zkio.4` | discharge proof/model, fairness, and totality boundaries or retain blockers |
| Stage 2 production policy and pack equivalence | `calc-zkio.5`, `calc-zkio.10` | prove production partition analyzer, scheduler equivalence, observable invariance, and pack equivalence or retain blockers |
| operated assurance service | `calc-zkio.6` | replace file-backed or envelope rows with operated service artifacts or retain exact blocker |
| retained history and witness lifecycle | `calc-zkio.6`, `calc-zkio.10` | bind retained-history query, witness lifecycle, and retention SLO to operated evidence or blockers |
| external alert dispatcher | `calc-zkio.6` | replace local alert rules with enforcing external dispatcher evidence or retain blocker |
| operated cross-engine differential | `calc-zkio.7` | provide operated cross-engine differential service evidence or retain blocker |
| fully independent evaluator | `calc-zkio.7` | broaden independent evaluator implementation authority and breadth or retain blocker |
| mismatch quarantine and retained-witness attachment | `calc-zkio.7` | bind mismatch authority, quarantine behavior, retained-witness attachment, and alert semantics |
| OxFml broad display/publication and public migration | `calc-zkio.8` | verify current consumer surfaces, public migration, and format/display boundary or retain blockers |
| W073 downstream typed-rule request construction | `calc-zkio.8` | verify downstream typed-rule construction where consumed or keep external public-migration blocker |
| registered-external/provider publication | `calc-zkio.8` | bind registered-external projection and provider/callable publication semantics or retain watch lane |
| release-scale replay performance/scaling | `calc-zkio.9` | turn scale models into continuous semantic assurance without treating timing as correctness proof |
| continuous scale assurance service | `calc-zkio.9` | create or block continuous scale-assurance semantic-regression service |
| pack-grade replay and C5 | `calc-zkio.10`, `calc-zkio.11` | reassess pack-grade replay governance and C5 only after direct W045 evidence |
| release-grade successor planning | `calc-zkio.1`, `calc-zkio.11` | keep W045 as improvement and spec-evolution scope with closure audit decision |
| general OxFunc kernels | `calc-zkio.8`, `calc-zkio.11` | keep external except narrow `LET`/`LAMBDA` carrier seam |

The machine-readable map records 36 obligations under `successor_obligation_map.json`.

## 5. Current OxFml Intake

Current W045 intake:

1. `VerificationConditionalFormattingRule.typed_rule` is the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are intentionally ignored for those aggregate and visualization families.
3. `thresholds` remains meaningful only for scalar/operator/expression rule families where threshold text is the actual rule input.
4. OxFml reports 21 focused conditional-formatting tests after the W073 direct-replacement update, including old bounded-string non-interpretation rows.
5. DNA OneCalc request construction for W073 families must emit `typed_rule`; OxCalc does not verify that downstream uptake in this bead.
6. Ordinary downstream OxFml use should target `oxfml_core::consumer::runtime`, `oxfml_core::consumer::editor`, and `oxfml_core::consumer::replay`.
7. Public `substrate::...` access is no longer an ordinary downstream integration contract; remaining host/session/adapter reach is `test_support` support surface.
8. `format_delta` and `display_delta` remain distinct canonical bundle categories.
9. Registered-external and provider-failure/callable-publication remain watch lanes until coordinator-visible evidence creates handoff pressure.
10. No OxFml handoff is filed by this bead because it records W045 obligations and does not expose a concrete exercised OxCalc/OxFml mismatch.

## 6. Promotion-Contract Map

| Promotion target | Contract state | Required direct evidence |
|---|---|---|
| release-grade full verification | blocked | all W045 obligation families, closure audit, direct-evidence coverage, and no-proxy release decision |
| full formalization and spec evolution | blocked | specs, TraceCalc authority, optimized/core, Rust, Lean/TLA, Stage 2, services, diversity, OxFml, scaling, pack/C5, and evolved scope consistency |
| full optimized/core verification | blocked | dynamic transition, soft-reference, snapshot, capability, callable metadata, and conformance rows |
| callable metadata projection | blocked | metadata projection implementation, fixture, publication, and OxFml/callable seam rows |
| callable carrier sufficiency | blocked | narrow `LET`/`LAMBDA` carrier proof and no general OxFunc overclaim |
| Rust totality/refinement and panic-free core | blocked | totality/refinement proof, panic-surface audit, and callable-carrier relation |
| full Lean/TLA verification and unbounded fairness | blocked | full proof/model discharge, placeholder audit, model-bound discharge, totality discharge, and fairness coverage |
| Stage 2 production policy | blocked | production analyzer soundness, scheduler equivalence, observable invariance, and operated differential service |
| pack-grade replay equivalence | blocked | baseline-versus-partition equivalence for values, rejects, dependencies, topology, overlays, witnesses, validation, and retained evidence |
| operated assurance and retained services | blocked | operated service artifacts, lifecycle service, retention SLO, replay query, and external dispatcher |
| operated cross-engine differential service | blocked | scheduled operated cross-engine service, differential authority, and retained-witness attachment |
| fully independent evaluator breadth | blocked | independent implementation authority and breadth beyond the current reference-model slice |
| mismatch quarantine and witness attachment service | blocked | service behavior, mismatch authority routing, retained-witness attachment, and alert/quarantine semantics |
| current OxFml public surface and W073 downstream uptake | blocked | current consumer surface, public migration, W073 downstream typed-rule uptake, and format/display boundary |
| registered-external and provider/callable publication | blocked | coordinator-visible registered-external projection and provider-failure/callable-publication semantics |
| continuous release-scale assurance | blocked | continuous semantic-regression service, validation oracle, phase timing, counters, and no-correctness-from-timing guard |
| pack-grade replay and `cap.C5.pack_valid` | blocked | W045 pack decision after direct services, proof/model, conformance, diversity, seam, scaling guard, and governance service evidence |
| general OxFunc kernels inside OxCalc | out of scope | external owner boundary; only narrow `LET`/`LAMBDA` carrier seam is in OxCalc formalization scope |

## 7. Spec-Evolution Hooks

| Hook | Applies to | Rule |
|---|---|---|
| `implementation_fault` | optimized/core, service, scaling, and seam rows | fix behavior and bind replay/differential evidence before promotion |
| `spec_correction` | reference machine, coordinator, proof, and seam clauses | update spec text and evidence artifacts together |
| `scope_evolution` | formalization workset scope | allow improved understanding to refine the spec rather than test against a fixed initial packet |
| `authority_exclusion` | external-owner or out-of-scope rows | name owner and prevent proxy promotion |
| `handoff_watch` | OxFml-owned seam clauses | file handoff only when exercised OxCalc evidence exposes a concrete insufficiency |
| `service_gap` | operated assurance, history, witness, alert, quarantine, and cross-engine rows | do not promote from file-backed or service-envelope evidence |
| `proof_gap` | Lean/TLA, Rust totality, refinement, and Stage 2 rows | distinguish proof, model bound, assumption, fairness boundary, and external seam |
| `scaling_gap` | release-scale replay and performance rows | do not treat throughput or phase timing as correctness proof |
| `promotion_gate` | pack/C5, release-grade, Stage 2, scaling, and service rows | require direct evidence before promotion |

## 8. Semantic-Equivalence Statement

This bead adds a successor obligation map, promotion-contract map, inbound OxFml observation-intake record, machine-readable evidence, status text, and bead graph state only.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc execution semantics, TreeCalc/CoreEngine runtime behavior, Lean/TLA model semantics, continuous-assurance runner semantics, pack runtime behavior, service behavior, alert dispatch behavior, retained-history behavior, release-scale replay/performance runner behavior, or OxFml/OxFunc evaluator behavior.

Observable runtime behavior is invariant under this bead. The packet defines evidence required before later release-grade, full formalization, C5, pack-grade replay, operated-service, retained-history, retained-witness, retention-SLO, independent-diversity, mismatch-quarantine, OxFml-breadth, W073 downstream uptake, callable-metadata, callable-carrier, registered-external, provider-publication, scaling, or Stage 2 claims.

## 9. Verification

| Command | Result |
|---|---|
| JSON parse for `docs/test-runs/core-engine/release-grade-ledger/w045-residual-release-grade-successor-obligation-current-oxfml-intake-map-001/*.json` | passed; all checked JSON files parsed |
| `scripts/check-worksets.ps1` | passed pre-close; `worksets=23; beads total=187; open=11; in_progress=1; ready=0; blocked=10; deferred=0; closed=175` |
| `br ready --json` | passed pre-close; no ready beads while `calc-zkio.1` is in progress |
| `br dep cycles --json` | passed pre-close; `count=0` |
| `git diff --check` | passed pre-close; CRLF normalization warning only for `.beads/issues.jsonl` |
| JSON parse for `docs/test-runs/core-engine/release-grade-ledger/w045-residual-release-grade-successor-obligation-current-oxfml-intake-map-001/*.json` | passed post-close; all checked JSON files parsed |
| `scripts/check-worksets.ps1` | passed post-close; `worksets=23; beads total=187; open=11; in_progress=0; ready=1; blocked=9; deferred=0; closed=176` |
| `br ready --json` | passed post-close; next ready bead is `calc-zkio.2` |
| `br dep cycles --json` | passed post-close; `count=0` |
| `git diff --check` | passed post-close; CRLF normalization warning only for `.beads/issues.jsonl` |

No cargo, Lean, TLC, or scaling command is required for this ledger bead because it emits no code, formal model, fixture, runner, replay behavior, scaling behavior, or runtime semantics.

## 10. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W045 artifact root, W045 workset/status surfaces, and machine-readable ledger artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-zkio.10` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for predecessor W044 behaviors cited by this ledger; this bead emits no new behavior |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 8 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; current OxFml W073 typed-only update and public-surface notes are carried as watch/handoff-trigger inputs and no handoff trigger exists |
| 6 | All required tests pass? | yes for this docs/evidence scope; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for this ledger target; broader semantic gaps are obligations with owners and promotion contracts |
| 8 | Completion language audit passed? | yes; no release-grade, full formalization, C5, pack-grade replay, Stage 2 policy, operated service, retained-history, retained-witness, retention SLO, independent-diversity, mismatch quarantine, broad OxFml, W073 downstream uptake, callable metadata, callable carrier, registered-external, provider-publication, scaling-correctness, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; W045 was registered by the predecessor closure packet and remains current |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W045 residual-map state |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-zkio.1` closed and `calc-zkio.2` ready |

## 11. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-zkio.1` asks for a residual release-grade successor obligation and current OxFml intake map |
| Gate criteria re-read | pass; W044 no-promotion blockers, owner beads, evidence targets, promotion contracts, inbound observation review, no-promotion blockers, and next-bead routing are explicit |
| Silent scope reduction check | pass; no release-grade, proof/model, optimized/core, service, pack, C5, OxFml breadth, W073 downstream uptake, callable metadata, independent evaluator, mismatch quarantine, scaling correctness, general OxFunc, or Stage 2 lane is promoted |
| "Looks done but is not" pattern check | pass; ledger text is not reported as implementation, proof, service, runtime, replay, scaling evidence, or downstream uptake verification |
| Result | pass for the `calc-zkio.1` ledger target after final post-close validation |

## 12. Three-Axis Report

- execution_state: `calc-zkio.1_residual_successor_obligation_current_oxfml_intake_map_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-zkio.2` optimized core counterpart coverage and callable metadata projection closure is ready next
  - Rust totality/refinement and panic-surface hardening remains open
  - Lean/TLA verification, fairness, and totality discharge remains open
  - Stage 2 production partition and pack-grade equivalence service evidence remains open
  - operated assurance, retained-history, retained-witness, SLO, and alert service implementation remains open
  - independent evaluator breadth, mismatch quarantine, and operated differential service remains open
  - OxFml public surface, W073 downstream typed formatting, callable, and registered-external uptake remains open
  - continuous release-scale assurance and semantic regression service remains open
  - pack-grade replay governance, C5 reassessment, and release-grade decision remain open
