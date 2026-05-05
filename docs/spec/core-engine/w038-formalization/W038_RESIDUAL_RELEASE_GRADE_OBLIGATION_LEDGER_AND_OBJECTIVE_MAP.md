# W038 Residual Release-Grade Obligation Ledger And Objective Map

Status: `calc-zsr.1_residual_release_grade_ledger_validated`
Workset: `W038`
Parent epic: `calc-zsr`
Bead: `calc-zsr.1`

## 1. Purpose

This packet converts the W037 closure-audit residuals into W038 release-grade obligations.

The target is not to promote release-grade verification. The target is to make the next direct-evidence tranche exact before `calc-zsr.2` starts: every W037 residual lane, no-promotion reason, relevant OxFml watch item, and spec-evolution hook has an owner, an evidence root, a required disposition, and a promotion consequence.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W038_CORE_FORMALIZATION_RELEASE_GRADE_CLOSURE_HARDENING.md` | W038 scope, gate model, and bead rollout |
| `docs/spec/core-engine/w037-formalization/W037_CLOSURE_AUDIT_AND_FULL_VERIFICATION_RELEASE_DECISION.md` | predecessor release decision and W038 successor path |
| `docs/test-runs/core-engine/closure-audit/w037-closure-audit-full-verification-release-decision-001/residual_lane_ledger.json` | 10 W037 residual lanes routed to W038 |
| `docs/test-runs/core-engine/implementation-conformance/w037-implementation-conformance-closure-001/w037_residual_blocker_register.json` | 5 optimized/core-engine residual blockers |
| `docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/promotion_blockers.json` | 7 proof/model promotion blockers |
| `docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/promotion_decision.json` | 4 Stage 2 promotion blockers |
| `docs/test-runs/core-engine/continuous-assurance/w037-operated-assurance-service-pilot-001/service/service_readiness.json` | 4 blocked service-readiness criteria |
| `docs/test-runs/core-engine/pack-capability/w037-pack-c5-candidate-decision-001/decision/pack_capability_decision.json` | 22 pack/C5 no-promotion reasons |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | reviewed inbound observations, including W073 typed-formatting, `format_delta`/`display_delta`, direct-packet, structured-reference, host/runtime, stand-in, registered-external, and W026 residual note-level lanes |

## 3. Artifact Surface

Run id: `w038-residual-release-grade-obligation-ledger-001`

| Artifact | Result |
|---|---|
| `docs/test-runs/core-engine/release-grade-ledger/w038-residual-release-grade-obligation-ledger-001/run_summary.json` | records 20 W038 obligations and non-promoting ledger status |
| `docs/test-runs/core-engine/release-grade-ledger/w038-residual-release-grade-obligation-ledger-001/residual_obligation_ledger.json` | machine-readable obligation rows, owners, required evidence, blocker inputs, and promotion consequences |
| `docs/test-runs/core-engine/release-grade-ledger/w038-residual-release-grade-obligation-ledger-001/objective_map.json` | maps the active objective requirements to W038 obligations and evidence owners |
| `docs/test-runs/core-engine/release-grade-ledger/w038-residual-release-grade-obligation-ledger-001/source_evidence_index.json` | names predecessor artifacts and reviewed inbound observation surfaces |
| `docs/test-runs/core-engine/release-grade-ledger/w038-residual-release-grade-obligation-ledger-001/validation.json` | records validation for this ledger packet |

## 4. Release-Grade Obligation Ledger

| Obligation id | Area | Owner bead | Required W038 disposition |
|---|---|---|---|
| `W038-OBL-001` | TraceCalc oracle authority | `calc-zsr.2` | discharge authority exclusion, justify it as external owner scope, or carry an exact blocker |
| `W038-OBL-002` | TraceCalc no-loss objective map | `calc-zsr.2` | prove every in-scope observable row has replay coverage or an accepted authority-exclusion row |
| `W038-OBL-003` | dynamic dependency negative/release/reclassification optimized counterpart | `calc-zsr.3` | add optimized differential evidence, implementation fix, spec correction, or exact residual blocker |
| `W038-OBL-004` | callable host-effect conformance | `calc-zsr.3`, `calc-zsr.7` | bind direct OxFml and narrow `LET`/`LAMBDA` carrier evidence without promoting general OxFunc kernels |
| `W038-OBL-005` | snapshot-fence optimized counterpart | `calc-zsr.3`, `calc-zsr.5` | add coordinator/optimized replay counterpart or carry Stage 2 blocker |
| `W038-OBL-006` | capability-view fence optimized counterpart | `calc-zsr.3`, `calc-zsr.5` | add compatibility-fenced optimized counterpart or carry exact blocker |
| `W038-OBL-007` | callable metadata projection | `calc-zsr.3`, `calc-zsr.4`, `calc-zsr.7` | prove callable carrier sufficiency or add metadata projection fixture |
| `W038-OBL-008` | full Lean proof boundary | `calc-zsr.4` | separate local proof, assumed seams, external owners, and unproved Rust-engine totality rows |
| `W038-OBL-009` | full TLA/model boundary | `calc-zsr.4`, `calc-zsr.5` | separate checked bounded configs from unbounded/model-completeness claims |
| `W038-OBL-010` | general OxFunc kernel boundary | `calc-zsr.4`, `calc-zsr.7` | keep external OxFunc kernel semantics opaque unless a narrow shared carrier proof is exercised |
| `W038-OBL-011` | Stage 2 deterministic partition replay | `calc-zsr.5` | execute baseline-versus-partition replay or keep Stage 2 policy blocked |
| `W038-OBL-012` | production partition analyzer soundness | `calc-zsr.5` | prove analyzer soundness for promoted profiles or carry exact blocker |
| `W038-OBL-013` | Stage 2 observable-result invariance | `calc-zsr.5`, `calc-zsr.6` | show invariant published values, rejects, dependency/topology consequences, and replay validation across promoted profiles |
| `W038-OBL-014` | operated regression runner and retained history service | `calc-zsr.6` | produce operated multi-run service artifacts or keep service promotion blocked |
| `W038-OBL-015` | alert/quarantine enforcement | `calc-zsr.6` | connect quarantine policy to enforcing alert dispatcher evidence |
| `W038-OBL-016` | operated cross-engine differential service | `calc-zsr.6` | replace file-backed pilot evidence with operated differential service evidence or carry exact blocker |
| `W038-OBL-017` | fully independent evaluator diversity | `calc-zsr.7` | identify independent implementation authority and row set, not projection over the same implementation |
| `W038-OBL-018` | OxFml seam and formatting watch | `calc-zsr.7` | keep W073 typed formatting, `format_delta`/`display_delta`, host/runtime, structured-reference, stand-in, and registered-external note-level lanes current; file handoff only on concrete mismatch |
| `W038-OBL-019` | pack-grade replay governance and program scope | `calc-zsr.8` | bind replay governance, retained-witness policy, service evidence, and no-proxy promotion guard |
| `W038-OBL-020` | C5 and release decision | `calc-zsr.8`, `calc-zsr.9` | emit C5/release decision only after direct W038 evidence is bound |

## 5. Objective Map

| Objective requirement | W038 mapping | Current state |
|---|---|---|
| run post-W033 successor beads sequentially | W034, W035, W036, and W037 are closed; W038 starts at `calc-zsr.1` | active; `calc-zsr.2` follows this ledger |
| continue through all formalization slices | W038 obligations cover TraceCalc, optimized/core, Lean/TLA, Stage 2, operated assurance, OxFml seam/watch, diversity, pack, and C5 lanes | mapped, not promoted |
| full core-engine formalization | `W038-OBL-001` through `W038-OBL-020` | release-grade verification not promoted |
| verify TraceCalc | `W038-OBL-001`, `W038-OBL-002` | full oracle authority remains open |
| verify optimized implementation | `W038-OBL-003` through `W038-OBL-007` | five W037 residual conformance blockers remain |
| put formal proofs/checks in place | `W038-OBL-008` through `W038-OBL-010` | proof/model inventory exists; full proof remains open |
| put full TLA and Lean verification in place | `W038-OBL-008`, `W038-OBL-009`, `W038-OBL-011` through `W038-OBL-013` | full Lean/TLA verification remains open |
| preserve OxCalc plus OxFml scope, excluding general OxFunc kernels except the narrow carrier seam | `W038-OBL-004`, `W038-OBL-007`, `W038-OBL-010`, `W038-OBL-018` | direct OxFml slice exercised; general OxFunc kernels remain external |
| make performance/scaling confidence subordinate to correctness | `W038-OBL-014` through `W038-OBL-016` | timing is not correctness evidence |
| make pack/C5 decisions from direct evidence | `W038-OBL-019`, `W038-OBL-020` | highest honest capability remains `cap.C4.distill_valid` |

## 6. Spec-Evolution Hooks

W038 may change specs when evidence shows a mismatch, but every change must stay traceable to an obligation row.

| Hook | Applies to | Rule |
|---|---|---|
| `implementation_fault` | optimized/core conformance rows | fix Rust behavior and bind replay/diff evidence before promotion |
| `spec_correction` | TraceCalc/reference-machine or coordinator clauses | update spec text and replay artifacts together |
| `authority_exclusion` | external or out-of-scope rows | name owner and prevent proxy promotion |
| `handoff_watch` | OxFml-owned seam clauses | file OxFml handoff only when exercised OxCalc evidence exposes a concrete insufficiency |
| `service_gap` | operated assurance/cross-engine rows | do not promote from file-backed pilot evidence |
| `proof_gap` | Lean/TLA rows | distinguish local proof, bounded model, assumptions, and external seams |
| `promotion_gate` | pack/C5/Stage 2 rows | require direct evidence before capability or scheduler promotion |

## 7. Reviewed Inbound Observations

Reviewed `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`.

Current W038 intake:

1. W073 aggregate/visualization conditional-formatting metadata remains `typed_rule`-only; `thresholds` remains scalar/operator/expression input text.
2. `format_delta` and `display_delta` remain distinct consequence categories.
3. host/runtime, structured-reference, immutable-edit, stand-in fixture-host, and registered-external packet families are converged enough for first-slice planning but not broad coordinator API freeze.
4. W026 residual topics remain note-level: caller-anchor/address-mode carriage, execution-restriction transport breadth, and publication/topology consequence breadth.
5. no new OxFml handoff is filed by this bead because no exercised W038 artifact exposes an OxFml-owned contract defect.

## 8. Semantic-Equivalence Statement

This bead adds a release-grade obligation ledger, objective map, machine-readable evidence, and status text only.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc execution semantics, TreeCalc/CoreEngine runtime behavior, Lean/TLA model semantics, continuous-assurance runner semantics, pack runtime behavior, service behavior, alert dispatch behavior, or OxFml/OxFunc evaluator behavior.

Observable runtime behavior is invariant under this bead. The packet defines evidence required before later release-grade, C5, pack-grade replay, operated-service, or Stage 2 claims.

## 9. Verification

| Command | Result |
|---|---|
| JSON parse for `docs/test-runs/core-engine/release-grade-ledger/w038-residual-release-grade-obligation-ledger-001/*.json` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-zsr.2` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

No cargo, Lean, or TLC command is required for this ledger bead because it emits no code, formal model, fixture, runner, replay behavior, or runtime semantics.

## 10. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W038 README/status surfaces, and machine-readable ledger artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-zsr.8` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for predecessor W037 behaviors cited by this ledger; this bead emits no new behavior |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 8 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; reviewed inbound OxFml notes and no handoff trigger exists |
| 6 | All required tests pass? | yes for this documentation/bead-graph audit scope; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for this ledger target; broader semantic gaps are obligations with owners and promotion consequences |
| 8 | Completion language audit passed? | yes; no full verification, C5, pack-grade replay, Stage 2 policy, operated service, or independent-diversity promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W038 ledger state |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-zsr.1` closure and `calc-zsr.2` readiness |

## 11. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-zsr.1` asks for a residual release-grade obligation ledger and objective map |
| Gate criteria re-read | pass; exact residual rows, owner beads, evidence roots, promotion consequences, and spec-evolution hooks are present |
| Silent scope reduction check | pass; no release-grade, proof/model, TraceCalc, optimized/core, service, pack, C5, or Stage 2 lane is promoted |
| "Looks done but is not" pattern check | pass; ledger text is not reported as implementation, proof, or runtime evidence |
| Result | pass for the `calc-zsr.1` ledger target |

## 12. Three-Axis Report

- execution_state: `calc-zsr.1_residual_release_grade_ledger_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-zsr.2` TraceCalc oracle authority and authority-exclusion discharge is next
  - optimized/core-engine conformance blocker closure remains open
  - proof/model assumption discharge remains open
  - Stage 2 partition replay and semantic-equivalence execution remain open
  - operated assurance, alert/quarantine, and cross-engine service remain open
  - independent evaluator diversity and OxFml seam watch closure remain open
  - pack-grade replay governance, C5, and W038 release decision remain open

Post-`calc-zsr.2` note: `w038-tracecalc-authority-discharge-001` accepts the single W037 authority-excluded row, `w035_callable_full_oxfunc_semantics`, as an external OxFunc-owned semantic-kernel exclusion. It leaves zero uncovered TraceCalc rows for the current OxCalc-owned observable profile and keeps non-TraceCalc release gates open.

Post-`calc-zsr.3` note: `w038-optimized-core-conformance-disposition-001` rechecks the five W037 optimized/core residual blockers, binds three direct-evidence rows, accepts one boundary row, preserves four exact remaining blockers, promotes zero declared gaps as matches, and routes proof/model, Stage 2/coordinator, OxFml seam/watch, operated-service, pack/C5, and release-decision lanes to later W038 beads.

Post-`calc-zsr.4` note: `w038-proof-model-assumption-discharge-001` records eight proof/model assumption rows, three local-proof rows, two bounded-model rows, one external-seam row, three totality boundaries, six exact blockers, and zero failed rows. It binds the W038 Lean assumption-discharge file and routine TLC floor as bounded evidence while keeping full Lean/TLA verification, Stage 2 policy, pack-grade replay, C5, and general OxFunc kernels unpromoted.
