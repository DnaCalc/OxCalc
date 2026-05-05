# W040 Residual Direct-Verification Obligation Map

Status: `calc-tv5.1_residual_direct_verification_map_validated`
Workset: `W040`
Parent epic: `calc-tv5`
Bead: `calc-tv5.1`

## 1. Purpose

This packet converts the W039 non-promoting release-grade decision into W040 direct-verification obligations.

The target is not to promote release-grade verification. The target is to make the direct-verification tranche exact before `calc-tv5.2` starts: every post-W039 residual lane has an owner bead, required direct evidence, promotion consequence, and spec-evolution hook.

The packet also retains the current OxFml W073 formatting intake: aggregate and visualization conditional-formatting metadata is `typed_rule`-only for the named W073 families, and W072 bounded `thresholds` strings are not a fallback for those families.

The current OxFml update is a direct-replacement input contract for those families, not the earlier additive/fallback reading. OxFml evidence now includes focused tests that typed payloads drive behavior and old bounded aggregate/visualization strings are not interpreted.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W040_CORE_FORMALIZATION_RELEASE_GRADE_DIRECT_VERIFICATION.md` | W040 scope, gate model, bead rollout, and direct-verification guard |
| `docs/spec/core-engine/w039-formalization/W039_CLOSURE_AUDIT_AND_RELEASE_GRADE_DECISION.md` | predecessor release decision and successor lanes |
| `docs/test-runs/core-engine/closure-audit/w039-closure-audit-release-grade-decision-001/residual_lane_ledger.json` | 12 post-W039 residual lanes |
| `docs/test-runs/core-engine/release-grade-ledger/w039-residual-successor-obligation-ledger-001/successor_obligation_ledger.json` | predecessor W039 obligation rows |
| W039 evidence packet run summaries | optimized/core, proof/model, Stage 2, service, diversity, OxFml seam, pack/C5, and closure-audit inputs |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | inbound OxFml observation ledger |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | W073 typed-only aggregate/visualization input contract |
| `../OxFml/docs/handoffs/HANDOFF-DNAONECALC-012_W073_TYPED_CF_PAYLOAD_FIRST_SLICE.md` | downstream typed request-construction handoff |
| `../OxFml/crates/oxfml_core/src/publication/mod.rs` | observed removal of bounded-threshold fallback for W073 families |
| `../OxFml/crates/oxfml_core/tests/conditional_formatting_array_tests.rs` | focused typed-payload and old-string non-interpretation evidence |

## 3. Artifact Surface

Run id: `w040-direct-verification-obligation-map-001`

| Artifact | Result |
|---|---|
| `docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/run_summary.json` | records W040 ledger status, counts, W073 intake, and no-promotion claims |
| `docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/direct_verification_obligation_map.json` | machine-readable W040 obligations, owners, source lanes, required evidence, consequences, and hooks |
| `docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/promotion_target_gate_map.json` | maps release-grade, optimized/core, Rust totality, Lean/TLA, Stage 2, service, diversity, OxFml, callable, pack/C5, and OxFunc-boundary targets to blockers |
| `docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/source_evidence_index.json` | names predecessor artifacts and inbound OxFml observation surfaces |
| `docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/w073_formatting_intake.json` | records the current W073 typed-only formatting consequence for W040 |
| `docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/validation.json` | records validation for this ledger packet |

## 4. Direct-Verification Obligation Map

| Obligation id | Area | Owner bead | Required W040 disposition |
|---|---|---|---|
| `W040-OBL-001` | direct-verification objective map and no-proxy guard | `calc-tv5.1`, `calc-tv5.10` | maintain machine-readable obligations and no-proxy promotion guard |
| `W040-OBL-002` | optimized/core dynamic release and reclassification | `calc-tv5.2` | produce direct optimized/core differential evidence or exact blocker |
| `W040-OBL-003` | snapshot fence and capability fence counterparts | `calc-tv5.2`, `calc-tv5.5` | prove promoted fence behavior across coordinator and optimized/core lanes or retain exact blocker |
| `W040-OBL-004` | callable metadata projection implementation | `calc-tv5.2`, `calc-tv5.8` | add projection fixture/implementation evidence or prove carrier sufficiency without general OxFunc kernel promotion |
| `W040-OBL-005` | declared-gap match-promotion guard | `calc-tv5.2`, `calc-tv5.9`, `calc-tv5.10` | keep declared gaps out of match and pack/C5 promotion counts |
| `W040-OBL-006` | Rust totality and panic-free core domain | `calc-tv5.3` | bind totality evidence for promoted Rust paths or exact totality blockers |
| `W040-OBL-007` | Rust refinement relation | `calc-tv5.3`, `calc-tv5.4` | relate TraceCalc-covered reference behavior to optimized/core behavior |
| `W040-OBL-008` | Lean proof discharge | `calc-tv5.4` | discharge or retain proof blockers with checked Lean artifacts and explicit axiom/sorry/admit audit |
| `W040-OBL-009` | TLA model coverage and fairness assumptions | `calc-tv5.4`, `calc-tv5.5` | state model bounds, unboundedness limits, fairness assumptions, and promotion predicates |
| `W040-OBL-010` | Stage 2 production partition analyzer soundness | `calc-tv5.5` | prove or block analyzer soundness for promoted partition profiles |
| `W040-OBL-011` | Stage 2 observable invariance and replay equivalence | `calc-tv5.5`, `calc-tv5.9` | show baseline-versus-partition invariance for observable results and replay validation |
| `W040-OBL-012` | operated continuous-assurance runner service | `calc-tv5.6` | produce runnable runner, scheduler, retention, thresholds, and service-readable run history artifacts |
| `W040-OBL-013` | retained history and witness service lifecycle | `calc-tv5.6` | produce lifecycle, retention, query, and replay-correlation guarantees |
| `W040-OBL-014` | alert/quarantine dispatcher enforcement | `calc-tv5.6` | connect quarantine policy to enforcing alert dispatcher evidence |
| `W040-OBL-015` | operated cross-engine differential service | `calc-tv5.6`, `calc-tv5.7` | replace file-backed rows with operated differential service evidence |
| `W040-OBL-016` | independent evaluator implementation authority | `calc-tv5.7` | identify independently implemented evaluator rows with distinct authority |
| `W040-OBL-017` | diversity mismatch handling | `calc-tv5.7` | classify agreement, mismatch, authority, quarantine, and spec-evolution consequences |
| `W040-OBL-018` | OxFml W073 typed-only conditional-formatting seam | `calc-tv5.8` | prevent W072 threshold-fallback assumptions for W073 aggregate/visualization families |
| `W040-OBL-019` | OxFml public consumer surfaces and display/publication breadth | `calc-tv5.8` | classify consumed public surfaces, `format_delta`/`display_delta`, and display/publication blockers |
| `W040-OBL-020` | `LET`/`LAMBDA` carrier boundary and general OxFunc external owner | `calc-tv5.4`, `calc-tv5.8`, `calc-tv5.10` | keep narrow carrier rows in OxCalc and general OxFunc kernels external |
| `W040-OBL-021` | pack-grade replay governance service | `calc-tv5.9` | bind pack-grade replay governance to retained history, service, proof/model, and differential evidence |
| `W040-OBL-022` | C5 promotion decision | `calc-tv5.9`, `calc-tv5.10` | reassess C5 only after direct W040 evidence is bound |
| `W040-OBL-023` | release-grade full-verification decision | `calc-tv5.10` | decide release-grade promotion or successor scope from direct evidence only |

## 5. Promotion-Target Gate Map

| Promotion target | Current readiness | Blocking obligations |
|---|---|---|
| release-grade full verification | blocked | all `W040-OBL-*` rows |
| full optimized/core verification | blocked | `W040-OBL-002` through `W040-OBL-005`, `W040-OBL-007` |
| Rust totality and refinement | blocked | `W040-OBL-006`, `W040-OBL-007` |
| full Lean/TLA verification | blocked | `W040-OBL-007` through `W040-OBL-009`, `W040-OBL-020` |
| Stage 2 production policy | blocked | `W040-OBL-003`, `W040-OBL-009` through `W040-OBL-011`, `W040-OBL-015`, `W040-OBL-021` |
| operated assurance services | blocked | `W040-OBL-012` through `W040-OBL-015` |
| fully independent evaluator diversity | blocked | `W040-OBL-015` through `W040-OBL-017` |
| broad OxFml seam and display/publication | blocked | `W040-OBL-004`, `W040-OBL-018` through `W040-OBL-020` |
| callable metadata projection | blocked | `W040-OBL-004`, `W040-OBL-020` |
| pack-grade replay and `cap.C5.pack_valid` | blocked | `W040-OBL-001`, `W040-OBL-005`, `W040-OBL-011`, `W040-OBL-013`, `W040-OBL-015` through `W040-OBL-017`, `W040-OBL-021`, `W040-OBL-022` |
| general OxFunc kernels inside OxCalc | out of scope, not promoted | `W040-OBL-020` |

## 6. OxFml Formatting Intake

Current W040 intake:

1. `VerificationConditionalFormattingRule.typed_rule` is the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are intentionally ignored for those aggregate and visualization families.
3. `thresholds` remains meaningful only for scalar/operator/expression rule families where threshold text is the actual rule input.
4. OxCalc W040 evidence must not assume threshold fallback for the typed-only W073 families.
5. If exercised OxCalc evidence depends on old W072 aggregate/visualization threshold strings, `calc-tv5.8` must record a mismatch or handoff trigger.
6. No OxFml handoff is filed by this bead because it records obligations only and does not expose a concrete exercised OxCalc mismatch.

Observed OxFml focused evidence for this intake names 21 conditional-formatting tests, including typed-payload behavior rows and two non-interpretation rows: `bounded_visualization_threshold_strings_are_not_interpreted` and `bounded_aggregate_option_strings_are_not_interpreted`.

## 7. Spec-Evolution Hooks

| Hook | Applies to | Rule |
|---|---|---|
| `implementation_fault` | optimized/core rows | fix behavior and bind replay/diff evidence before promotion |
| `spec_correction` | reference-machine, coordinator, and seam clauses | update spec text and evidence artifacts together |
| `authority_exclusion` | external-owner or out-of-scope rows | name owner and prevent proxy promotion |
| `handoff_watch` | OxFml-owned seam clauses | file handoff only when exercised OxCalc evidence exposes a concrete insufficiency |
| `service_gap` | operated assurance, history, alert, and cross-engine rows | do not promote from file-backed pilot evidence |
| `proof_gap` | Lean/TLA, Rust totality, and refinement rows | distinguish proof, model bound, assumption, and external seam |
| `promotion_gate` | pack/C5, release-grade, and Stage 2 rows | require direct evidence before promotion |

## 8. Semantic-Equivalence Statement

This bead adds a direct-verification obligation map, promotion-target gate map, W073 formatting-intake record, machine-readable evidence, and status text only.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc execution semantics, TreeCalc/CoreEngine runtime behavior, Lean/TLA model semantics, continuous-assurance runner semantics, pack runtime behavior, service behavior, alert dispatch behavior, retained-history behavior, or OxFml/OxFunc evaluator behavior.

Observable runtime behavior is invariant under this bead. The packet defines evidence required before later release-grade, C5, pack-grade replay, operated-service, independent-diversity, OxFml-breadth, callable-metadata, or Stage 2 claims.

## 9. Verification

| Command | Result |
|---|---|
| JSON parse for `docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/*.json` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-tv5.2` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

No cargo, Lean, or TLC command is required for this ledger bead because it emits no code, formal model, fixture, runner, replay behavior, or runtime semantics.

## 10. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W040 README/status surfaces, feature map, and machine-readable ledger artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-tv5.9` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for predecessor W039 behaviors cited by this ledger; this bead emits no new behavior |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 8 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; current OxFml W073 update is carried as a typed-only guard and no handoff trigger exists |
| 6 | All required tests pass? | yes for this documentation/bead-graph audit scope; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for this ledger target; broader semantic gaps are obligations with owners and promotion consequences |
| 8 | Completion language audit passed? | yes; no release-grade, full formalization, C5, pack-grade replay, Stage 2 policy, operated service, independent-diversity, broad OxFml, callable metadata, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; W040 was registered in the preceding bootstrap checkpoint |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W040 ledger state |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-tv5.1` closure and `calc-tv5.2` readiness |

## 11. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-tv5.1` asks for a residual direct-verification obligation map |
| Gate criteria re-read | pass; exact residual rows, owner beads, evidence targets, promotion consequences, and spec-evolution hooks are present |
| Silent scope reduction check | pass; no release-grade, proof/model, optimized/core, service, pack, C5, OxFml breadth, callable metadata, independent evaluator, general OxFunc, or Stage 2 lane is promoted |
| "Looks done but is not" pattern check | pass; ledger text is not reported as implementation, proof, service, runtime, or replay evidence |
| Result | pass for the `calc-tv5.1` ledger target |

## 12. Three-Axis Report

- execution_state: `calc-tv5.1_residual_direct_verification_map_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-tv5.2` optimized core exact blocker fixes and differentials is next
  - Rust totality and refinement proof remains open
  - Lean/TLA full-verification discharge remains open
  - Stage 2 production policy and equivalence remains open
  - operated assurance and retained-history service implementation remains open
  - independent evaluator implementation and operated differential remains open
  - OxFml seam breadth and callable metadata implementation remains open
  - pack-grade replay governance, C5, and release-grade decision remain open
