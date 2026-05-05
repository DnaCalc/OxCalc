# W041 Residual Release-Grade Successor Obligation Map

Status: `calc-sui.1_residual_successor_obligation_map_validated`
Workset: `W041`
Parent epic: `calc-sui`
Bead: `calc-sui.1`

## 1. Purpose

This packet converts the W040 non-promoting release-grade verification decision into W041 successor obligations.

The target is not to promote release-grade verification. The target is to make the W041 successor tranche exact before `calc-sui.2` starts: every post-W040 residual lane has an owner bead, required direct evidence, promotion consequence, and spec-evolution hook.

The packet also retains the current OxFml W073 formatting intake: aggregate and visualization conditional-formatting metadata is `typed_rule`-only for the named W073 families, and W072 bounded `thresholds` strings are not a fallback for those families.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W041_CORE_FORMALIZATION_RELEASE_GRADE_SUCCESSOR_VERIFICATION.md` | W041 scope, gate model, bead rollout, and successor-verification guard |
| `docs/spec/core-engine/w040-formalization/W040_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md` | predecessor release-grade decision and successor lanes |
| `docs/test-runs/core-engine/closure-audit/w040-closure-audit-release-grade-verification-decision-001/residual_lane_ledger.json` | 15 post-W040 residual lanes |
| `docs/test-runs/core-engine/closure-audit/w040-closure-audit-release-grade-verification-decision-001/direct_evidence_coverage_audit.json` | W040 direct-evidence coverage audit |
| `docs/test-runs/core-engine/pack-capability/w040-pack-grade-replay-governance-c5-promotion-decision-001/decision/pack_capability_decision.json` | W040 pack/C5 no-promotion decision |
| W040 evidence packet run summaries | optimized/core, Rust, Lean/TLA, Stage 2, service, diversity, OxFml seam, pack/C5, and closure-audit inputs |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | inbound OxFml observation ledger |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | W073 typed-only aggregate/visualization input contract |
| `../OxFml/docs/handoffs/HANDOFF-DNAONECALC-012_W073_TYPED_CF_PAYLOAD_FIRST_SLICE.md` | downstream typed request-construction handoff |

## 3. Artifact Surface

Run id: `w041-residual-release-grade-successor-obligation-map-001`

| Artifact | Result |
|---|---|
| `docs/test-runs/core-engine/release-grade-ledger/w041-residual-release-grade-successor-obligation-map-001/run_summary.json` | records W041 ledger status, counts, W073 intake, and no-promotion claims |
| `docs/test-runs/core-engine/release-grade-ledger/w041-residual-release-grade-successor-obligation-map-001/successor_obligation_map.json` | machine-readable W041 obligations, owners, source lanes, required evidence, consequences, and hooks |
| `docs/test-runs/core-engine/release-grade-ledger/w041-residual-release-grade-successor-obligation-map-001/promotion_target_gate_map.json` | maps release-grade, optimized/core, Rust, Lean/TLA, Stage 2, service, diversity, OxFml, callable, pack/C5, and OxFunc-boundary targets to blockers |
| `docs/test-runs/core-engine/release-grade-ledger/w041-residual-release-grade-successor-obligation-map-001/source_evidence_index.json` | names predecessor artifacts and inbound OxFml observation surfaces |
| `docs/test-runs/core-engine/release-grade-ledger/w041-residual-release-grade-successor-obligation-map-001/w073_formatting_intake.json` | records the current W073 typed-only formatting consequence for W041 |
| `docs/test-runs/core-engine/release-grade-ledger/w041-residual-release-grade-successor-obligation-map-001/validation.json` | records validation for this ledger packet |

## 4. Successor Obligation Map

| Obligation id | Area | Owner bead | Required W041 disposition |
|---|---|---|---|
| `W041-OBL-001` | successor-verification objective map and no-proxy guard | `calc-sui.1`, `calc-sui.10` | maintain machine-readable obligations and no-proxy promotion guard |
| `W041-OBL-002` | automatic dynamic dependency-set transition detection | `calc-sui.2` | prove automatic transition detection or retain exact optimized/core blocker |
| `W041-OBL-003` | snapshot-fence optimized/core counterpart | `calc-sui.2`, `calc-sui.5` | prove stale-candidate reject/no-publish counterpart for promoted profiles |
| `W041-OBL-004` | capability-fence optimized/core counterpart | `calc-sui.2`, `calc-sui.5` | prove capability-view mismatch reject/no-publish counterpart for promoted profiles |
| `W041-OBL-005` | callable metadata projection implementation | `calc-sui.2`, `calc-sui.8` | add projection fixture/implementation evidence or exact blocker |
| `W041-OBL-006` | declared-gap and proxy-promotion guard | `calc-sui.2`, `calc-sui.9`, `calc-sui.10` | keep declared gaps, file-backed-only artifacts, and bounded rows out of promotion counts |
| `W041-OBL-007` | Rust panic-free core domain | `calc-sui.3` | discharge or retain panic-surface blockers for promoted Rust paths |
| `W041-OBL-008` | Rust totality across dependency and publication transitions | `calc-sui.3` | bind totality evidence for dependency changes, publication fences, and callable carriers |
| `W041-OBL-009` | Rust refinement relation | `calc-sui.3`, `calc-sui.4` | relate TraceCalc-covered reference behavior to optimized/core behavior |
| `W041-OBL-010` | Lean proof discharge | `calc-sui.4` | discharge or retain proof blockers with checked Lean artifacts and axiom/sorry/admit audit |
| `W041-OBL-011` | TLA model coverage, fairness, and scheduler assumptions | `calc-sui.4`, `calc-sui.5` | state model bounds, unboundedness limits, fairness assumptions, and promotion predicates |
| `W041-OBL-012` | Stage 2 production partition-analyzer soundness | `calc-sui.5` | prove or block production analyzer soundness for promoted profiles |
| `W041-OBL-013` | Stage 2 observable invariance and pack replay equivalence | `calc-sui.5`, `calc-sui.9` | show baseline-versus-partition equivalence for values, rejects, dependencies, topology, overlays, and replay validation |
| `W041-OBL-014` | operated continuous-assurance service | `calc-sui.6` | replace file-backed runner evidence with operated scheduler/service evidence or exact blocker |
| `W041-OBL-015` | retained-history service lifecycle and query API | `calc-sui.6` | provide lifecycle, retention, query, and replay-correlation guarantees |
| `W041-OBL-016` | retained-witness lifecycle and retention SLO | `calc-sui.6`, `calc-sui.9` | connect retained witnesses to pack-grade replay governance |
| `W041-OBL-017` | external alert/quarantine dispatcher enforcement | `calc-sui.6` | wire quarantine policy to enforcing external dispatcher evidence or exact blocker |
| `W041-OBL-018` | operated cross-engine differential service | `calc-sui.6`, `calc-sui.7` | replace file-backed differential rows with operated differential service evidence |
| `W041-OBL-019` | independent evaluator breadth and authority | `calc-sui.7` | broaden independent implementation beyond bounded scalar rows or retain exact blocker |
| `W041-OBL-020` | diversity mismatch handling and authority routing | `calc-sui.7` | classify agreement, mismatch, authority, quarantine, and spec-evolution consequences |
| `W041-OBL-021` | OxFml W073 typed-only formatting seam | `calc-sui.8` | prevent W072 threshold fallback assumptions for W073 aggregate/visualization families |
| `W041-OBL-022` | OxFml broad display/publication and public migration | `calc-sui.8` | exercise broad consumed surfaces, display/publication deltas, and public migration evidence or exact blockers |
| `W041-OBL-023` | callable carrier sufficiency for `LET`/`LAMBDA` | `calc-sui.4`, `calc-sui.8` | prove narrow callable carrier sufficiency without general OxFunc kernel promotion |
| `W041-OBL-024` | registered-external callable and provider notes | `calc-sui.8` | classify registered-external notes, provider failure rows, and handoff triggers |
| `W041-OBL-025` | pack-grade replay governance | `calc-sui.9` | bind retained witnesses, services, proof/model, differential, and OxFml seam evidence before pack-grade promotion |
| `W041-OBL-026` | C5 promotion decision | `calc-sui.9`, `calc-sui.10` | reassess C5 from direct W041 evidence only |
| `W041-OBL-027` | release-grade full-verification decision | `calc-sui.10` | decide release-grade promotion or successor scope from direct evidence only |
| `W041-OBL-028` | general OxFunc owner boundary | `calc-sui.4`, `calc-sui.8`, `calc-sui.10` | keep general OxFunc kernels external except the narrow `LET`/`LAMBDA` carrier seam |

## 5. Promotion-Target Gate Map

| Promotion target | Current readiness | Blocking obligations |
|---|---|---|
| release-grade full verification | blocked | all `W041-OBL-*` rows |
| full optimized/core verification | blocked | `W041-OBL-002` through `W041-OBL-006`, `W041-OBL-009` |
| Rust totality and refinement | blocked | `W041-OBL-007` through `W041-OBL-009` |
| full Lean/TLA verification | blocked | `W041-OBL-009` through `W041-OBL-011`, `W041-OBL-023`, `W041-OBL-028` |
| Stage 2 production policy | blocked | `W041-OBL-003`, `W041-OBL-004`, `W041-OBL-011` through `W041-OBL-013`, `W041-OBL-018`, `W041-OBL-025` |
| operated assurance services | blocked | `W041-OBL-014` through `W041-OBL-018` |
| retained-history and retained-witness services | blocked | `W041-OBL-015`, `W041-OBL-016`, `W041-OBL-025` |
| fully independent evaluator breadth | blocked | `W041-OBL-018` through `W041-OBL-020` |
| broad OxFml display/publication and public migration | blocked | `W041-OBL-021` through `W041-OBL-024` |
| callable metadata and carrier sufficiency | blocked | `W041-OBL-005`, `W041-OBL-023`, `W041-OBL-024`, `W041-OBL-028` |
| pack-grade replay and `cap.C5.pack_valid` | blocked | `W041-OBL-001`, `W041-OBL-006`, `W041-OBL-013`, `W041-OBL-015`, `W041-OBL-016`, `W041-OBL-018` through `W041-OBL-020`, `W041-OBL-025`, `W041-OBL-026` |
| general OxFunc kernels inside OxCalc | out of scope, not promoted | `W041-OBL-028` |

## 6. OxFml Formatting Intake

Current W041 intake:

1. `VerificationConditionalFormattingRule.typed_rule` is the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are intentionally ignored for those aggregate and visualization families.
3. `thresholds` remains meaningful only for scalar/operator/expression rule families where threshold text is the actual rule input.
4. OxCalc W041 evidence must not assume threshold fallback for the typed-only W073 families.
5. If exercised OxCalc evidence depends on old W072 aggregate/visualization threshold strings, `calc-sui.8` must record a mismatch or handoff trigger.
6. No OxFml handoff is filed by this bead because it records obligations only and does not expose a concrete exercised OxCalc mismatch.

## 7. Spec-Evolution Hooks

| Hook | Applies to | Rule |
|---|---|---|
| `implementation_fault` | optimized/core and service rows | fix behavior and bind replay/diff evidence before promotion |
| `spec_correction` | reference-machine, coordinator, proof, and seam clauses | update spec text and evidence artifacts together |
| `authority_exclusion` | external-owner or out-of-scope rows | name owner and prevent proxy promotion |
| `handoff_watch` | OxFml-owned seam clauses | file handoff only when exercised OxCalc evidence exposes a concrete insufficiency |
| `service_gap` | operated assurance, history, witness, alert, and cross-engine rows | do not promote from file-backed pilot evidence |
| `proof_gap` | Lean/TLA, Rust totality, refinement, and Stage 2 rows | distinguish proof, model bound, assumption, fairness boundary, and external seam |
| `promotion_gate` | pack/C5, release-grade, Stage 2, and service rows | require direct evidence before promotion |

## 8. Semantic-Equivalence Statement

This bead adds a successor obligation map, promotion-target gate map, W073 formatting-intake record, machine-readable evidence, and status text only.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc execution semantics, TreeCalc/CoreEngine runtime behavior, Lean/TLA model semantics, continuous-assurance runner semantics, pack runtime behavior, service behavior, alert dispatch behavior, retained-history behavior, or OxFml/OxFunc evaluator behavior.

Observable runtime behavior is invariant under this bead. The packet defines evidence required before later release-grade, C5, pack-grade replay, operated-service, independent-diversity, OxFml-breadth, callable-metadata, callable-carrier, or Stage 2 claims.

## 9. Verification

| Command | Result |
|---|---|
| JSON parse for `docs/test-runs/core-engine/release-grade-ledger/w041-residual-release-grade-successor-obligation-map-001/*.json` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-sui.2` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

No cargo, Lean, or TLC command is required for this ledger bead because it emits no code, formal model, fixture, runner, replay behavior, or runtime semantics.

## 10. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W041 README/status surfaces, feature map, and machine-readable ledger artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-sui.9` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for predecessor W040 behaviors cited by this ledger; this bead emits no new behavior |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 8 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; current OxFml W073 update is carried as a typed-only guard and no handoff trigger exists |
| 6 | All required tests pass? | yes for this documentation/bead-graph audit scope; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for this ledger target; broader semantic gaps are obligations with owners and promotion consequences |
| 8 | Completion language audit passed? | yes; no release-grade, full formalization, C5, pack-grade replay, Stage 2 policy, operated service, independent-diversity, broad OxFml, callable metadata, callable carrier, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; W041 was registered in the bootstrap checkpoint |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W041 ledger state |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-sui.1` closure and `calc-sui.2` readiness |

## 11. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-sui.1` asks for a residual release-grade successor obligation map |
| Gate criteria re-read | pass; exact residual rows, owner beads, evidence targets, promotion consequences, and spec-evolution hooks are present |
| Silent scope reduction check | pass; no release-grade, proof/model, optimized/core, service, pack, C5, OxFml breadth, callable metadata, independent evaluator, general OxFunc, or Stage 2 lane is promoted |
| "Looks done but is not" pattern check | pass; ledger text is not reported as implementation, proof, service, runtime, or replay evidence |
| Result | pass for the `calc-sui.1` ledger target |

## 12. Three-Axis Report

- execution_state: `calc-sui.1_residual_successor_obligation_map_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-sui.2` optimized/core residual blocker implementation and differential closure is next
  - Rust totality/refinement and panic-boundary discharge remains open
  - Lean/TLA full-verification and fairness discharge remains open
  - Stage 2 production analyzer and pack-equivalence proof remains open
  - operated assurance, retained-history, retained-witness, and alert-dispatch service work remains open
  - independent evaluator breadth and operated differential service remains open
  - OxFml broad display/publication, public migration, callable metadata, and callable-carrier closure remains open
  - pack-grade replay governance, C5, and release-grade decision remain open
