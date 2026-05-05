# W042 Residual Release-Grade Closure Obligation Ledger

Status: `calc-czd.1_residual_release_grade_closure_obligation_ledger_validated`
Workset: `W042`
Parent epic: `calc-czd`
Bead: `calc-czd.1`

## 1. Purpose

This packet converts the W041 non-promoting release-grade verification decision into W042 closure-expansion obligations.

The target is not to promote release-grade verification. The target is to make the W042 tranche exact before `calc-czd.2` starts: every post-W041 residual lane has an owner bead, required direct evidence, promotion consequence, and spec-evolution hook.

The packet also retains the current OxFml W073 formatting intake: aggregate and visualization conditional-formatting metadata is `typed_rule`-only for the named W073 families, and W072 bounded `thresholds` strings are not a fallback for those families.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W042_CORE_FORMALIZATION_RELEASE_GRADE_EVIDENCE_CLOSURE_EXPANSION.md` | W042 scope, gate model, bead rollout, and evidence-closure guard |
| `docs/spec/core-engine/w041-formalization/W041_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md` | predecessor release-grade decision and successor lanes |
| `docs/test-runs/core-engine/closure-audit/w041-closure-audit-release-grade-verification-decision-001/residual_lane_ledger.json` | 16 post-W041 residual lanes |
| `docs/test-runs/core-engine/closure-audit/w041-closure-audit-release-grade-verification-decision-001/direct_evidence_coverage_audit.json` | W041 direct-evidence coverage audit |
| `docs/test-runs/core-engine/pack-capability/w041-pack-grade-replay-governance-c5-reassessment-001/decision/pack_capability_decision.json` | W041 pack/C5 no-promotion decision |
| W041 evidence packet run summaries | optimized/core, Rust, Lean/TLA, Stage 2, service, diversity, OxFml seam, pack/C5, and closure-audit inputs |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | inbound OxFml observation ledger |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | W073 typed-only aggregate/visualization input contract |
| `../OxFml/docs/handoffs/HANDOFF-DNAONECALC-012_W073_TYPED_CF_PAYLOAD_FIRST_SLICE.md` | downstream typed request-construction handoff |

## 3. Artifact Surface

Run id: `w042-residual-release-grade-closure-obligation-ledger-001`

| Artifact | Result |
|---|---|
| `docs/test-runs/core-engine/release-grade-ledger/w042-residual-release-grade-closure-obligation-ledger-001/run_summary.json` | records W042 ledger status, counts, W073 intake, and no-promotion claims |
| `docs/test-runs/core-engine/release-grade-ledger/w042-residual-release-grade-closure-obligation-ledger-001/closure_obligation_map.json` | machine-readable W042 obligations, owners, source lanes, required evidence, consequences, and hooks |
| `docs/test-runs/core-engine/release-grade-ledger/w042-residual-release-grade-closure-obligation-ledger-001/promotion_target_gate_map.json` | maps release-grade, optimized/core, Rust, Lean/TLA, Stage 2, pack-equivalence, service, diversity, mismatch, OxFml, callable, pack/C5, and OxFunc-boundary targets to blockers |
| `docs/test-runs/core-engine/release-grade-ledger/w042-residual-release-grade-closure-obligation-ledger-001/source_evidence_index.json` | names predecessor artifacts and inbound OxFml observation surfaces |
| `docs/test-runs/core-engine/release-grade-ledger/w042-residual-release-grade-closure-obligation-ledger-001/w073_formatting_intake.json` | records the current W073 typed-only formatting consequence for W042 |
| `docs/test-runs/core-engine/release-grade-ledger/w042-residual-release-grade-closure-obligation-ledger-001/validation.json` | records validation for this ledger packet |

## 4. Closure Obligation Map

| Obligation id | Area | Owner bead | Required W042 disposition |
|---|---|---|---|
| `W042-OBL-001` | release-grade objective map and no-proxy guard | `calc-czd.1`, `calc-czd.10` | maintain machine-readable obligations, target gates, no-proxy promotion guard, and successor consequences |
| `W042-OBL-002` | broader automatic dynamic dependency-set transition coverage | `calc-czd.2` | expand beyond the W041 exercised resolved-to-potential pattern or retain exact blocker |
| `W042-OBL-003` | snapshot-fence optimized/core counterpart | `calc-czd.2`, `calc-czd.5` | prove stale-candidate reject/no-publish counterpart for promoted profiles |
| `W042-OBL-004` | capability-view optimized/core counterpart | `calc-czd.2`, `calc-czd.5` | prove capability-view mismatch reject/no-publish counterpart for promoted profiles |
| `W042-OBL-005` | callable metadata projection implementation | `calc-czd.2`, `calc-czd.8` | add projection fixture/implementation evidence or exact blocker |
| `W042-OBL-006` | declared-gap and proxy-promotion guard | `calc-czd.2`, `calc-czd.9`, `calc-czd.10` | keep declared gaps, file-backed-only artifacts, bounded rows, and aggregate confidence out of promotion counts |
| `W042-OBL-007` | Rust panic-free core domain | `calc-czd.3` | discharge or retain panic-surface blockers for promoted Rust paths |
| `W042-OBL-008` | Rust totality across dependency, publication, and callable transitions | `calc-czd.3` | bind totality evidence for dependency changes, publication fences, and callable carriers |
| `W042-OBL-009` | Rust refinement relation | `calc-czd.3`, `calc-czd.4` | relate TraceCalc-covered reference behavior to optimized/core behavior |
| `W042-OBL-010` | Lean proof discharge | `calc-czd.4` | discharge or retain proof blockers with checked Lean artifacts and axiom/sorry/admit audit |
| `W042-OBL-011` | TLA model coverage and unbounded scheduler/fairness assumptions | `calc-czd.4`, `calc-czd.5` | state model bounds, unboundedness limits, fairness assumptions, scheduler equivalence, and promotion predicates |
| `W042-OBL-012` | callable carrier proof boundary | `calc-czd.4`, `calc-czd.8` | prove or block the narrow `LET`/`LAMBDA` carrier seam without general OxFunc kernel promotion |
| `W042-OBL-013` | Stage 2 production partition-analyzer soundness | `calc-czd.5` | prove or block production analyzer soundness for promoted profiles |
| `W042-OBL-014` | Stage 2 scheduler equivalence | `calc-czd.5` | show observable-result invariance under scheduler/partition strategy for promoted profiles |
| `W042-OBL-015` | pack-grade replay equivalence | `calc-czd.5`, `calc-czd.9` | show baseline-versus-partition equivalence for values, rejects, dependencies, topology, overlays, retained witnesses, and replay validation |
| `W042-OBL-016` | operated continuous-assurance service | `calc-czd.6` | replace file-backed/service-envelope evidence with operated scheduler/service evidence or exact blocker |
| `W042-OBL-017` | retained-history service lifecycle and query API | `calc-czd.6` | provide lifecycle, retention, query, and replay-correlation guarantees |
| `W042-OBL-018` | retained-witness lifecycle and retention SLO | `calc-czd.6`, `calc-czd.9` | connect retained witnesses to pack-grade replay governance |
| `W042-OBL-019` | external alert/quarantine dispatcher enforcement | `calc-czd.6` | wire quarantine policy to enforcing external dispatcher evidence or exact blocker |
| `W042-OBL-020` | operated cross-engine differential service | `calc-czd.6`, `calc-czd.7` | replace file-backed differential rows with operated differential service evidence |
| `W042-OBL-021` | mismatch quarantine service | `calc-czd.7` | bind mismatch authority routing to service behavior, retained witness attachment, and alert/quarantine semantics |
| `W042-OBL-022` | independent evaluator breadth and authority | `calc-czd.7` | broaden independent implementation beyond current formula-fragment rows or retain exact blocker |
| `W042-OBL-023` | diversity mismatch handling and spec-evolution routing | `calc-czd.7`, `calc-czd.10` | classify agreement, mismatch, authority, quarantine, spec correction, and implementation-fault consequences |
| `W042-OBL-024` | OxFml W073 typed-only formatting seam | `calc-czd.8` | prevent W072 threshold fallback assumptions for W073 aggregate/visualization families |
| `W042-OBL-025` | OxFml broad display/publication and public migration | `calc-czd.8` | exercise broad consumed surfaces, display/publication deltas, and public migration evidence or exact blockers |
| `W042-OBL-026` | callable carrier sufficiency for `LET`/`LAMBDA` | `calc-czd.4`, `calc-czd.8` | prove narrow callable carrier sufficiency without general OxFunc kernel promotion |
| `W042-OBL-027` | callable metadata publication surface | `calc-czd.2`, `calc-czd.8` | separate callable value carrier sufficiency from callable metadata projection and publication surface |
| `W042-OBL-028` | registered-external callable projection | `calc-czd.8` | classify registered-external callable projection rows, provider boundaries, and handoff triggers |
| `W042-OBL-029` | provider-failure/callable-publication semantics | `calc-czd.8` | bind provider failure and callable-publication observable semantics or retain exact blocker |
| `W042-OBL-030` | pack-grade replay governance | `calc-czd.9` | bind retained witnesses, services, proof/model, differential, Stage 2, and OxFml seam evidence before pack-grade promotion |
| `W042-OBL-031` | C5 promotion decision | `calc-czd.9`, `calc-czd.10` | reassess C5 from direct W042 evidence only |
| `W042-OBL-032` | release-grade full-verification decision | `calc-czd.10` | decide release-grade promotion or successor scope from direct evidence only |
| `W042-OBL-033` | general OxFunc owner boundary | `calc-czd.4`, `calc-czd.8`, `calc-czd.10` | keep general OxFunc kernels external except the narrow `LET`/`LAMBDA` carrier seam |

## 5. Promotion-Target Gate Map

| Promotion target | Current readiness | Blocking obligations |
|---|---|---|
| release-grade full verification | blocked | all `W042-OBL-*` rows |
| full optimized/core verification | blocked | `W042-OBL-002` through `W042-OBL-006`, `W042-OBL-009`, `W042-OBL-027` |
| Rust totality and refinement | blocked | `W042-OBL-007` through `W042-OBL-009`, `W042-OBL-026` |
| full Lean/TLA verification | blocked | `W042-OBL-009` through `W042-OBL-012`, `W042-OBL-026`, `W042-OBL-033` |
| Stage 2 production policy | blocked | `W042-OBL-003`, `W042-OBL-004`, `W042-OBL-011`, `W042-OBL-013` through `W042-OBL-015`, `W042-OBL-020`, `W042-OBL-030` |
| pack-grade replay equivalence | blocked | `W042-OBL-015`, `W042-OBL-018`, `W042-OBL-020`, `W042-OBL-021`, `W042-OBL-030` |
| operated assurance services | blocked | `W042-OBL-016` through `W042-OBL-021` |
| retained-history and retained-witness services | blocked | `W042-OBL-017`, `W042-OBL-018`, `W042-OBL-030` |
| fully independent evaluator breadth | blocked | `W042-OBL-020` through `W042-OBL-023` |
| mismatch quarantine service | blocked | `W042-OBL-019` through `W042-OBL-021`, `W042-OBL-023` |
| broad OxFml display/publication and public migration | blocked | `W042-OBL-024`, `W042-OBL-025`, `W042-OBL-028`, `W042-OBL-029` |
| callable metadata and carrier sufficiency | blocked | `W042-OBL-005`, `W042-OBL-012`, `W042-OBL-026` through `W042-OBL-028`, `W042-OBL-033` |
| pack-grade replay and `cap.C5.pack_valid` | blocked | `W042-OBL-001`, `W042-OBL-006`, `W042-OBL-015` through `W042-OBL-018`, `W042-OBL-020` through `W042-OBL-022`, `W042-OBL-030`, `W042-OBL-031` |
| general OxFunc kernels inside OxCalc | out of scope, not promoted | `W042-OBL-033` |

## 6. OxFml Formatting Intake

Current W042 intake:

1. `VerificationConditionalFormattingRule.typed_rule` is the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are intentionally ignored for those aggregate and visualization families.
3. `thresholds` remains meaningful only for scalar/operator/expression rule families where threshold text is the actual rule input.
4. OxCalc W042 evidence must not assume threshold fallback for the typed-only W073 families.
5. If exercised OxCalc evidence depends on old W072 aggregate/visualization threshold strings, `calc-czd.8` must record a mismatch or handoff trigger.
6. No OxFml handoff is filed by this bead because it records obligations only and does not expose a concrete exercised OxCalc mismatch.

## 7. Spec-Evolution Hooks

| Hook | Applies to | Rule |
|---|---|---|
| `implementation_fault` | optimized/core and service rows | fix behavior and bind replay/diff evidence before promotion |
| `spec_correction` | reference-machine, coordinator, proof, and seam clauses | update spec text and evidence artifacts together |
| `authority_exclusion` | external-owner or out-of-scope rows | name owner and prevent proxy promotion |
| `handoff_watch` | OxFml-owned seam clauses | file handoff only when exercised OxCalc evidence exposes a concrete insufficiency |
| `service_gap` | operated assurance, history, witness, alert, quarantine, and cross-engine rows | do not promote from file-backed or service-envelope evidence |
| `proof_gap` | Lean/TLA, Rust totality, refinement, and Stage 2 rows | distinguish proof, model bound, assumption, fairness boundary, and external seam |
| `promotion_gate` | pack/C5, release-grade, Stage 2, and service rows | require direct evidence before promotion |

## 8. Semantic-Equivalence Statement

This bead adds a closure obligation map, promotion-target gate map, W073 formatting-intake record, machine-readable evidence, and status text only.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc execution semantics, TreeCalc/CoreEngine runtime behavior, Lean/TLA model semantics, continuous-assurance runner semantics, pack runtime behavior, service behavior, alert dispatch behavior, retained-history behavior, or OxFml/OxFunc evaluator behavior.

Observable runtime behavior is invariant under this bead. The packet defines evidence required before later release-grade, C5, pack-grade replay, operated-service, independent-diversity, mismatch-quarantine, OxFml-breadth, callable-metadata, callable-carrier, registered-external, provider-publication, or Stage 2 claims.

## 9. Verification

| Command | Result |
|---|---|
| JSON parse for `docs/test-runs/core-engine/release-grade-ledger/w042-residual-release-grade-closure-obligation-ledger-001/*.json` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-czd.2` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

No cargo, Lean, or TLC command is required for this ledger bead because it emits no code, formal model, fixture, runner, replay behavior, or runtime semantics.

## 10. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W042 README/status surfaces, feature map, and machine-readable ledger artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-czd.9` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for predecessor W041 behaviors cited by this ledger; this bead emits no new behavior |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 8 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; current OxFml W073 update is carried as a typed-only guard and no handoff trigger exists |
| 6 | All required tests pass? | yes for this documentation/bead-graph audit scope; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for this ledger target; broader semantic gaps are obligations with owners and promotion consequences |
| 8 | Completion language audit passed? | yes; no release-grade, full formalization, C5, pack-grade replay, Stage 2 policy, operated service, independent-diversity, broad OxFml, callable metadata, callable carrier, registered-external, provider-publication, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; W042 was registered in the bootstrap checkpoint |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W042 ledger state |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-czd.1` closure and `calc-czd.2` readiness |

## 11. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-czd.1` asks for a residual release-grade closure obligation ledger |
| Gate criteria re-read | pass; exact residual rows, owner beads, evidence targets, promotion consequences, and spec-evolution hooks are present |
| Silent scope reduction check | pass; no release-grade, proof/model, optimized/core, service, pack, C5, OxFml breadth, callable metadata, independent evaluator, mismatch quarantine, general OxFunc, or Stage 2 lane is promoted |
| "Looks done but is not" pattern check | pass; ledger text is not reported as implementation, proof, service, runtime, or replay evidence |
| Result | pass for the `calc-czd.1` ledger target |

## 12. Three-Axis Report

- execution_state: `calc-czd.1_residual_release_grade_closure_obligation_ledger_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-czd.2` optimized/core counterpart conformance and callable metadata projection is next
  - Rust totality/refinement and core panic-boundary closure remains open
  - Lean/TLA fairness and full-verification expansion remains open
  - Stage 2 production analyzer and pack-grade equivalence closure remains open
  - operated assurance, retained-history, retained-witness, and alert service closure remains open
  - independent evaluator breadth, mismatch quarantine, and operated differential service remains open
  - OxFml public migration, callable carrier, registered-external, and provider/callable publication closure remains open
  - pack-grade replay governance, C5, and release-grade decision remain open
