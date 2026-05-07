# W043 Residual Release-Grade Proof-Service Obligation Map

Status: `calc-2p3.1_residual_release_grade_proof_service_obligation_map_validated`
Workset: `W043`
Parent epic: `calc-2p3`
Bead: `calc-2p3.1`

## 1. Purpose

This packet converts the W042 non-promoting release-grade verification decision into W043 proof, operated-service, conformance, diversity, OxFml/callable, pack/C5, and release-grade obligations.

The target is not to promote release-grade verification. The target is to make the W043 tranche exact before `calc-2p3.2` starts: every post-W042 residual lane has an owner bead, required direct evidence, promotion consequence, no-proxy guard, OxFml watch or handoff trigger, and spec-evolution hook.

This packet also records the current inbound OxFml observations relevant to W043. W073 formatting remains typed-only for aggregate and visualization conditional-formatting families. Runtime-facade, structured-reference, stand-in fixture-host, registered-external, provider-failure, callable-publication, public-migration, and `format_delta`/`display_delta` notes are carried as watch or handoff-trigger inputs, not as shared seam-freeze text.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W043_CORE_FORMALIZATION_RELEASE_GRADE_PROOF_AND_OPERATED_SERVICE_INTEGRATION.md` | W043 scope, gate model, bead rollout, and proof/service guard |
| `docs/spec/core-engine/w042-formalization/W042_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md` | predecessor release-grade decision and successor lanes |
| `archive/test-runs-core-engine-w038-w045/closure-audit/w042-closure-audit-release-grade-verification-decision-001/residual_lane_ledger.json` | 18 post-W042 residual lanes |
| `archive/test-runs-core-engine-w038-w045/closure-audit/w042-closure-audit-release-grade-verification-decision-001/direct_evidence_coverage_audit.json` | W042 direct-evidence coverage audit |
| `archive/test-runs-core-engine-w038-w045/pack-capability/w042-pack-grade-replay-governance-c5-reassessment-001/decision/pack_capability_decision.json` | W042 pack/C5 no-promotion decision |
| W042 evidence packet run summaries | optimized/core, Rust, Lean/TLA, Stage 2, operated assurance, diversity, OxFml seam, upstream-host, pack/C5, and closure-audit inputs |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | inbound OxFml observation ledger |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | W073 typed-only aggregate/visualization input contract |
| `../OxFml/docs/handoffs/HANDOFF-DNAONECALC-012_W073_TYPED_CF_PAYLOAD_FIRST_SLICE.md` | downstream typed request-construction handoff |

## 3. Artifact Surface

Run id: `w043-residual-release-grade-proof-service-obligation-map-001`

| Artifact | Result |
|---|---|
| `archive/test-runs-core-engine-w038-w045/release-grade-ledger/w043-residual-release-grade-proof-service-obligation-map-001/run_summary.json` | records W043 ledger status, counts, W073 intake, inbound observation review, and no-promotion claims |
| `archive/test-runs-core-engine-w038-w045/release-grade-ledger/w043-residual-release-grade-proof-service-obligation-map-001/proof_service_obligation_map.json` | machine-readable W043 obligations, owners, source lanes, required evidence, consequences, and hooks |
| `archive/test-runs-core-engine-w038-w045/release-grade-ledger/w043-residual-release-grade-proof-service-obligation-map-001/promotion_target_gate_map.json` | maps release-grade, full formalization, optimized/core, callable, Rust, Lean/TLA, Stage 2, services, diversity, mismatch, OxFml, pack/C5, and OxFunc-boundary targets to blockers |
| `archive/test-runs-core-engine-w038-w045/release-grade-ledger/w043-residual-release-grade-proof-service-obligation-map-001/source_evidence_index.json` | names predecessor artifacts and inbound OxFml observation surfaces |
| `archive/test-runs-core-engine-w038-w045/release-grade-ledger/w043-residual-release-grade-proof-service-obligation-map-001/oxfml_inbound_observation_intake.json` | records W073 and current note-level OxFml watch/handoff trigger lanes |
| `archive/test-runs-core-engine-w038-w045/release-grade-ledger/w043-residual-release-grade-proof-service-obligation-map-001/validation.json` | records validation for this ledger packet |

## 4. Obligation Map

| Obligation id | Area | Owner bead | Required W043 disposition |
|---|---|---|---|
| `W043-OBL-001` | release-grade objective map and no-proxy guard | `calc-2p3.1`, `calc-2p3.10` | maintain obligations, target gates, no-proxy guard, successor consequences, and direct-evidence requirements |
| `W043-OBL-002` | full formalization proof-service closure checklist | `calc-2p3.1`, `calc-2p3.10` | map full-formalization prerequisites across specs, TraceCalc, optimized/core, Rust, Lean, TLA, Stage 2, services, diversity, seam, pack/C5, and successor scope |
| `W043-OBL-003` | broader dynamic dependency transition coverage | `calc-2p3.2` | broaden automatic addition/removal/reclassification/soft-reference coverage or retain exact blocker |
| `W043-OBL-004` | snapshot-fence optimized/core counterpart | `calc-2p3.2`, `calc-2p3.5` | prove stale-candidate reject/no-publish counterpart or retain exact blocker |
| `W043-OBL-005` | capability-view optimized/core counterpart | `calc-2p3.2`, `calc-2p3.5` | prove capability-view mismatch reject/no-publish counterpart or retain exact blocker |
| `W043-OBL-006` | callable metadata projection implementation | `calc-2p3.2`, `calc-2p3.8` | add metadata projection fixture, implementation, and publication evidence or retain exact blocker |
| `W043-OBL-007` | callable value carrier versus metadata separation | `calc-2p3.2`, `calc-2p3.8` | keep carrier sufficiency, metadata projection, and publication surface as separate promotion rows |
| `W043-OBL-008` | declared-gap and proxy-promotion guard | `calc-2p3.2`, `calc-2p3.9`, `calc-2p3.10` | keep gaps, file-backed artifacts, bounded rows, watch lanes, and aggregate confidence out of promotion counts |
| `W043-OBL-009` | Rust panic-free core domain | `calc-2p3.3` | discharge or retain panic-surface blockers with direct audit, tests, or checked proof/model artifacts |
| `W043-OBL-010` | Rust totality across dependency, publication, fence, and callable transitions | `calc-2p3.3` | bind totality evidence or retain exact blockers |
| `W043-OBL-011` | Rust refinement relation to TraceCalc and optimized/core | `calc-2p3.3`, `calc-2p3.4` | relate TraceCalc reference behavior to optimized/core behavior or retain refinement blockers |
| `W043-OBL-012` | Lean proof discharge and zero-placeholder audit | `calc-2p3.4` | discharge or retain proof blockers with checked Lean artifacts and axiom/sorry/admit audit |
| `W043-OBL-013` | TLA unbounded scheduler and fairness coverage | `calc-2p3.4`, `calc-2p3.5` | state model bounds, unboundedness limits, fairness assumptions, scheduler equivalence, and promotion predicates |
| `W043-OBL-014` | proof/model/refinement bridge across Rust, TLA, and Stage 2 | `calc-2p3.4`, `calc-2p3.5` | connect checked rows to runtime/replay evidence or keep them bounded and non-promoting |
| `W043-OBL-015` | narrow `LET`/`LAMBDA` callable carrier proof boundary | `calc-2p3.4`, `calc-2p3.8` | prove or block the narrow carrier seam without general OxFunc kernel promotion |
| `W043-OBL-016` | Stage 2 production partition-analyzer soundness | `calc-2p3.5` | prove or block production analyzer soundness |
| `W043-OBL-017` | Stage 2 scheduler equivalence and observable-result invariance | `calc-2p3.5` | show observable invariance under scheduler/partition strategy or retain exact blocker |
| `W043-OBL-018` | pack-grade replay equivalence | `calc-2p3.5`, `calc-2p3.9` | show baseline-versus-partition equivalence across values, rejects, dependencies, topology, overlays, witnesses, and validation |
| `W043-OBL-019` | operated continuous-assurance service | `calc-2p3.6` | replace service-envelope evidence with operated scheduler/service evidence or exact blocker |
| `W043-OBL-020` | retained-history lifecycle and query service | `calc-2p3.6` | provide lifecycle, retention, query, and replay-correlation guarantees or exact blocker |
| `W043-OBL-021` | retained-witness lifecycle and retention SLO | `calc-2p3.6`, `calc-2p3.9` | connect retained witnesses to pack-grade replay governance with lifecycle and retention SLO evidence |
| `W043-OBL-022` | external alert/quarantine dispatcher enforcement | `calc-2p3.6` | wire quarantine policy to enforcing dispatcher evidence or exact blocker |
| `W043-OBL-023` | operated cross-engine differential service | `calc-2p3.6`, `calc-2p3.7` | replace file-backed differential rows with operated differential service evidence or exact blocker |
| `W043-OBL-024` | mismatch quarantine service | `calc-2p3.7` | bind mismatch routing to service behavior, retained witness attachment, and alert/quarantine semantics |
| `W043-OBL-025` | independent evaluator breadth and authority | `calc-2p3.7` | broaden independent implementation or retain exact blocker |
| `W043-OBL-026` | diversity mismatch handling and spec-evolution routing | `calc-2p3.7`, `calc-2p3.10` | classify agreement, mismatch, authority, quarantine, spec correction, and implementation-fault consequences |
| `W043-OBL-027` | OxFml W073 typed-only formatting seam | `calc-2p3.8` | prevent W072 threshold fallback assumptions for W073 aggregate/visualization families |
| `W043-OBL-028` | `format_delta` and `display_delta` boundary | `calc-2p3.8` | preserve distinct semantic-format and display-facing publication consequences |
| `W043-OBL-029` | OxFml broad display/publication and public migration | `calc-2p3.8` | exercise broad consumed surfaces, public migration, and downstream request-construction rows or retain exact blockers |
| `W043-OBL-030` | callable carrier sufficiency for `LET`/`LAMBDA` | `calc-2p3.4`, `calc-2p3.8` | prove narrow callable carrier sufficiency or retain exact blocker without general OxFunc promotion |
| `W043-OBL-031` | registered-external callable projection | `calc-2p3.8` | classify projection rows, provider boundaries, registration identity, and handoff triggers |
| `W043-OBL-032` | provider-failure/callable-publication semantics | `calc-2p3.8` | bind coordinator-visible evidence or retain watch lane and exact blocker |
| `W043-OBL-033` | inbound OxFml observation-ledger packet families | `calc-2p3.8`, `calc-2p3.10` | track runtime facade, host/runtime, structured-reference, stand-in fixture-host, registered-external, provider-failure/callable-publication, and W073 notes without treating note convergence as seam-freeze text |
| `W043-OBL-034` | pack-grade replay governance | `calc-2p3.9` | bind witnesses, services, proof/model, differential, Stage 2, conformance, and OxFml seam evidence before pack-grade promotion |
| `W043-OBL-035` | C5 promotion decision | `calc-2p3.9`, `calc-2p3.10` | reassess C5 from direct W043 evidence only |
| `W043-OBL-036` | general OxFunc owner boundary | `calc-2p3.4`, `calc-2p3.8`, `calc-2p3.10` | keep general OxFunc kernels external except the narrow `LET`/`LAMBDA` carrier seam |

## 5. Promotion-Target Gate Map

| Promotion target | Current readiness | Blocking obligations |
|---|---|---|
| release-grade full verification | blocked | all `W043-OBL-*` rows |
| full formalization | blocked | `W043-OBL-001`, `W043-OBL-002`, `W043-OBL-008`, `W043-OBL-012` through `W043-OBL-014`, `W043-OBL-034`, `W043-OBL-035` |
| full optimized/core verification | blocked | `W043-OBL-003` through `W043-OBL-008`, `W043-OBL-011` |
| callable metadata projection | blocked | `W043-OBL-006`, `W043-OBL-007`, `W043-OBL-031`, `W043-OBL-032` |
| callable carrier sufficiency | blocked | `W043-OBL-007`, `W043-OBL-015`, `W043-OBL-030`, `W043-OBL-036` |
| Rust totality/refinement and panic-free core | blocked | `W043-OBL-009` through `W043-OBL-011`, `W043-OBL-015`, `W043-OBL-030` |
| full Lean/TLA verification and unbounded fairness | blocked | `W043-OBL-011` through `W043-OBL-015`, `W043-OBL-030`, `W043-OBL-036` |
| Stage 2 production policy | blocked | `W043-OBL-004`, `W043-OBL-005`, `W043-OBL-013`, `W043-OBL-014`, `W043-OBL-016` through `W043-OBL-018`, `W043-OBL-023`, `W043-OBL-034` |
| pack-grade replay equivalence | blocked | `W043-OBL-018`, `W043-OBL-021`, `W043-OBL-023`, `W043-OBL-024`, `W043-OBL-034` |
| operated assurance, retained-history, retained-witness, and alert services | blocked | `W043-OBL-019` through `W043-OBL-024` |
| retention SLO enforcement | blocked | `W043-OBL-020`, `W043-OBL-021`, `W043-OBL-034` |
| fully independent evaluator breadth | blocked | `W043-OBL-023` through `W043-OBL-026` |
| mismatch quarantine service | blocked | `W043-OBL-022` through `W043-OBL-024`, `W043-OBL-026` |
| broad OxFml display/publication and public migration | blocked | `W043-OBL-027` through `W043-OBL-033` |
| pack-grade replay and `cap.C5.pack_valid` | blocked | `W043-OBL-001`, `W043-OBL-002`, `W043-OBL-008`, `W043-OBL-018` through `W043-OBL-021`, `W043-OBL-023` through `W043-OBL-025`, `W043-OBL-034`, `W043-OBL-035` |
| general OxFunc kernels inside OxCalc | out of scope, not promoted | `W043-OBL-015`, `W043-OBL-030`, `W043-OBL-036` |

## 6. OxFml Observation Intake

Current W043 intake:

1. `VerificationConditionalFormattingRule.typed_rule` is the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are intentionally ignored for those aggregate and visualization families.
3. `thresholds` remains meaningful only for scalar/operator/expression rule families where threshold text is the actual rule input.
4. `format_delta` and `display_delta` remain distinct canonical bundle categories.
5. Runtime facade and host/runtime external requirements are settled enough for first implementation planning, not shared seam-freeze text.
6. `table_catalog + enclosing_table_ref + caller_table_region` is the current structured-reference packet read, with table context host/OxCalc-owned.
7. The stand-in fixture-host packet is useful for deterministic fixture-host and TreeCalc-facing integration reuse, not for freezing the production coordinator API.
8. Registered-external registration-channel identity and stable registration identity must be preserved where later TreeCalc-facing evidence uses that lane.
9. Provider-failure/callable-publication remains a likely future formal-handoff trigger, but still a watch lane until coordinator-visible evidence creates pressure.
10. No OxFml handoff is filed by this bead because it records W043 obligations and does not expose a concrete exercised OxCalc/OxFml mismatch.

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

This bead adds a proof-service obligation map, promotion-target gate map, inbound OxFml observation-intake record, machine-readable evidence, and status text only.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc execution semantics, TreeCalc/CoreEngine runtime behavior, Lean/TLA model semantics, continuous-assurance runner semantics, pack runtime behavior, service behavior, alert dispatch behavior, retained-history behavior, or OxFml/OxFunc evaluator behavior.

Observable runtime behavior is invariant under this bead. The packet defines evidence required before later release-grade, full formalization, C5, pack-grade replay, operated-service, retained-history, retained-witness, retention-SLO, independent-diversity, mismatch-quarantine, OxFml-breadth, callable-metadata, callable-carrier, registered-external, provider-publication, or Stage 2 claims.

## 9. Verification

| Command | Result |
|---|---|
| JSON parse for `archive/test-runs-core-engine-w038-w045/release-grade-ledger/w043-residual-release-grade-proof-service-obligation-map-001/*.json` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure: worksets=21; beads total=163, open=10, in_progress=0, ready=1, blocked=8, deferred=0, closed=153 |
| `br ready --json` | passed after bead closure; next ready bead is `calc-2p3.2` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

No cargo, Lean, or TLC command is required for this ledger bead because it emits no code, formal model, fixture, runner, replay behavior, or runtime semantics.

## 10. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W043 README/status surfaces, feature map, and machine-readable ledger artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-2p3.9` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for predecessor W042 behaviors cited by this ledger; this bead emits no new behavior |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 8 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; current OxFml W073 update and inbound notes are carried as watch/handoff-trigger inputs and no handoff trigger exists |
| 6 | All required tests pass? | yes for this documentation/bead-graph audit scope; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for this ledger target; broader semantic gaps are obligations with owners and promotion consequences |
| 8 | Completion language audit passed? | yes; no release-grade, full formalization, C5, pack-grade replay, Stage 2 policy, operated service, retained-history, retained-witness, retention SLO, independent-diversity, mismatch quarantine, broad OxFml, callable metadata, callable carrier, registered-external, provider-publication, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; W043 was registered in the bootstrap checkpoint |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W043.1 ledger state |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-2p3.1` closure and `calc-2p3.2` readiness |

## 11. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-2p3.1` asks for a residual release-grade proof-service obligation map |
| Gate criteria re-read | pass; exact residual rows, owner beads, evidence targets, promotion consequences, inbound observation watch lanes, and spec-evolution hooks are present |
| Silent scope reduction check | pass; no release-grade, proof/model, optimized/core, service, pack, C5, OxFml breadth, callable metadata, independent evaluator, mismatch quarantine, general OxFunc, or Stage 2 lane is promoted |
| "Looks done but is not" pattern check | pass; ledger text is not reported as implementation, proof, service, runtime, or replay evidence |
| Result | pass for the `calc-2p3.1` ledger target |

## 12. Three-Axis Report

- execution_state: `calc-2p3.1_residual_release_grade_proof_service_obligation_map_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-2p3.2` optimized core broad conformance and callable metadata closure is next
  - Rust totality/refinement and panic-free core proof frontier remains open
  - Lean/TLA full-verification and unbounded fairness discharge remains open
  - Stage 2 production partition analyzer and scheduler equivalence remains open
  - operated assurance, retained-history, witness, SLO, and alert service remains open
  - independent evaluator breadth, mismatch quarantine, and differential service remains open
  - OxFml public migration, formatting, callable, and registered-external seam remains open
  - pack-grade replay governance, C5, and release-grade decision remain open
