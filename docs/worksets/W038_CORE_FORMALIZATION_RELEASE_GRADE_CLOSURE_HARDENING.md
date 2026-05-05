# W038: Core Formalization Release-Grade Closure Hardening

## Purpose

W038 continues the formalization path after W037.

W037 reached a non-promoting release decision: it improved direct evidence and removed the direct OxFml evaluator absence blocker for the exercised upstream-host slice, but it kept release-grade full verification, C5, pack-grade replay, Stage 2 policy, operated services, and fully independent evaluator diversity unpromoted. W038 turns those exact residuals into the next direct-evidence tranche.

W038 remains a spec-evolution and implementation-improvement workset. It does not treat current specs or current implementation behavior as immutable targets. New evidence may identify implementation faults, spec corrections, OxFml seam watch items, proof/model blockers, or successor scope.

## Position And Dependencies

- depends_on: `W037`
- parent epic: `calc-zsr`
- predecessor epic: `calc-ubd`
- upstream dependencies: `OxFml`
- canonical predecessor packet: `docs/spec/core-engine/w037-formalization/W037_CLOSURE_AUDIT_AND_FULL_VERIFICATION_RELEASE_DECISION.md`

## Scope

### In Scope

1. Convert W037 residual blockers and no-promotion reasons into W038 release-grade obligations with owners, evidence roots, promotion consequences, and spec-evolution hooks.
2. Discharge TraceCalc oracle authority exclusions with deterministic replay, accepted authority exclusions, spec corrections, or exact blockers.
3. Close optimized/core-engine conformance blockers through implementation fixes, direct differential matches, accepted spec evolution, or explicit residual blockers.
4. Deepen Lean/TLA artifacts from inventory and criteria into assumption discharge, totality-boundary hardening, stronger checked models, or exact proof/model blockers.
5. Execute deterministic Stage 2 partition replay and observable-result invariance evidence where feasible.
6. Move service-readiness from pilot criteria toward operated multi-run continuous assurance, cross-engine differential service, alert/quarantine enforcement, and service-readable history.
7. Strengthen independent evaluator diversity without counting shared implementation, shared fixtures, or declared gaps as independent matches.
8. Preserve OxFml seam and formatting watch rows, including W073 typed conditional-formatting metadata, distinct `format_delta`/`display_delta` consequences, and the narrow `LET`/`LAMBDA` carrier boundary.
9. Reassess pack-grade replay, C5, and Stage 2 policy only after direct W038 evidence is bound.

### Out Of Scope

1. General OxFunc semantic kernels beyond the narrow `LET`/`LAMBDA` carrier boundary consumed by OxCalc.
2. Direct edits to OxFml from this repo.
3. C5, pack-grade replay, Stage 2 policy, operated service, or full-verification promotion from proxy evidence.
4. Treating performance/scaling timing as a semantic correctness proof.
5. UI, host, or file-adapter work unless directly required by a W038 evidence artifact.

## Gate Model

### Entry Gate

1. `calc-ubd` W037 parent epic has closed.
2. W037 closure audit and full-verification release decision exists.
3. W038 successor beads exist in `.beads/`.

### Exit Gate

1. Every W037 open lane is mapped to W038 evidence, implementation work, handoff/watch rows, explicit deferral, or successor scope.
2. TraceCalc oracle authority claims distinguish replay-covered reference behavior, accepted authority exclusions, and remaining external/non-oracle rows.
3. Optimized/core-engine conformance gaps are fixed, replay/diff-promoted, spec-evolved, or carried as exact blockers without declared-gap match promotion.
4. Lean/TLA claims distinguish local proof, checked bounded model, external assumption, totality boundary, and blocked row.
5. Stage 2 promotion claims require deterministic partition replay and observable-result invariance evidence across promoted profiles.
6. Operated assurance and cross-engine differential claims require operated multi-run service artifacts and alert/quarantine enforcement evidence.
7. Independent evaluator diversity claims require independently implemented evaluator rows rather than projections over the same implementation.
8. OxFml seam and formatting watch rows remain current for exercised artifacts; any OxFml-owned contract pressure is packetized as handoff or watch-lane work.
9. Pack/C5 decisions state exact evidence and no-promotion blockers or direct promotion rationale.
10. Closure audit includes a prompt-to-artifact objective checklist, OPERATIONS Section 7 checklist, Section 9 self-audit, semantic-equivalence statement, and three-axis report.

## Bead Rollout

Parent:

1. `calc-zsr` - W038 core formalization release-grade closure hardening.

Child path:

1. `calc-zsr.1` - W038 residual release-grade obligation ledger and objective map.
2. `calc-zsr.2` - W038 TraceCalc oracle authority and authority-exclusion discharge.
3. `calc-zsr.3` - W038 optimized core-engine conformance blocker closure and fixes.
4. `calc-zsr.4` - W038 proof-model assumption discharge and totality boundary hardening.
5. `calc-zsr.5` - W038 Stage 2 partition replay and semantic-equivalence execution.
6. `calc-zsr.6` - W038 operated assurance alert-quarantine and cross-engine service.
7. `calc-zsr.7` - W038 independent evaluator diversity and OxFml seam watch closure.
8. `calc-zsr.8` - W038 pack-grade replay governance and C5 release decision.
9. `calc-zsr.9` - W038 closure audit and release-grade verification decision.

## Initial Guardrails

1. Release-grade claims require direct artifacts, not aggregate confidence.
2. TraceCalc remains the correctness oracle only for covered reference behavior.
3. A declared gap is not a match.
4. Bounded Lean/TLA artifacts are not total verification unless their assumptions and bounds are discharged for the claimed scope.
5. Stage 2 scheduler or partition changes require semantic-equivalence evidence that observable results are invariant for promoted profiles.
6. Operated service claims require operated evidence, not simulated history.
7. Fully independent evaluator diversity requires independent implementation authority.
8. W073 aggregate or visualization conditional-formatting metadata remains `typed_rule`-only; `thresholds` remains only for scalar/operator/expression rule families where threshold text is the actual input.
9. `LET`/`LAMBDA` remains a narrow OxCalc/OxFml/OxFunc carrier seam, not general OxFunc kernel formalization inside OxCalc.

## Current Status

- execution_state: `calc-zsr.3_optimized_core_conformance_disposition_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - TraceCalc authority is discharged for the current OxCalc-owned observable profile, but release-grade verification remains unpromoted
  - optimized/core-engine conformance blockers have W038 dispositions, but exact remaining blockers remain routed to later lanes
  - `calc-zsr.4` proof/model assumption discharge and totality boundary hardening is the next W038 bead
  - Stage 2 partition replay and semantic-equivalence execution remain open
  - operated assurance, alert/quarantine, and cross-engine service remain open
  - independent evaluator diversity and OxFml seam watch closure remain open
  - pack-grade replay governance, C5, and W038 release decision remain open

Latest W038 evidence:

1. `docs/spec/core-engine/w038-formalization/W038_RESIDUAL_RELEASE_GRADE_OBLIGATION_LEDGER_AND_OBJECTIVE_MAP.md` records the `calc-zsr.1` residual release-grade obligation ledger and objective map. The ledger packet `w038-residual-release-grade-obligation-ledger-001` maps 20 W038 obligations to owner beads, evidence roots, promotion consequences, and spec-evolution hooks without promoting release-grade verification, C5, pack-grade replay, Stage 2 policy, operated services, or fully independent evaluator diversity.
2. `docs/spec/core-engine/w038-formalization/W038_TRACECALC_ORACLE_AUTHORITY_AND_AUTHORITY_EXCLUSION_DISCHARGE.md` records the `calc-zsr.2` TraceCalc authority-discharge slice. The packet `w038-tracecalc-authority-discharge-001` accepts the single W037 authority-excluded row as an external OxFunc-owned semantic-kernel exclusion, leaves zero uncovered TraceCalc rows for the current OxCalc-owned observable profile, and keeps non-TraceCalc release gates open.
3. `docs/spec/core-engine/w038-formalization/W038_OPTIMIZED_CORE_ENGINE_CONFORMANCE_BLOCKER_CLOSURE_AND_FIXES.md` records the `calc-zsr.3` optimized/core-engine conformance-disposition slice. The packet `w038-optimized-core-conformance-disposition-001` rechecks 5 W037 residual blockers, binds 3 direct-evidence rows, accepts 1 boundary row, preserves 4 exact remaining blockers, promotes 0 declared gaps as matches, and keeps release-grade optimized/core-engine verification unpromoted.
