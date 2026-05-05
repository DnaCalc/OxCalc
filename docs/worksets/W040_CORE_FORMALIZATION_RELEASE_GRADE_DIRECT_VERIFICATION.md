# W040: Core Formalization Release-Grade Direct Verification

## Purpose

W040 continues after the W039 non-promoting release-grade successor tranche.

W039 improved the evidence floor and made the remaining blockers exact, but it did not promote release-grade full verification, full formalization, C5, pack-grade replay, Stage 2 policy, operated assurance services, retained-history service, fully independent evaluator diversity, broad OxFml display/publication closure, callable metadata projection, or general OxFunc kernels inside OxCalc.

W040 is the next direct-verification execution lane. Its job is to convert W039 residuals into direct implementation, proof, model, service, diversity, OxFml/callable, pack/C5, and release-grade evidence where possible, while preserving exact blockers where evidence is still insufficient.

W040 remains a formalization, spec-evolution, and engine-improvement workset. It is not a test against a fixed initial spec. Current specs, TraceCalc behavior, optimized/core behavior, proof artifacts, TLA models, OxFml seam notes, service artifacts, and pack decisions are evidence surfaces that can force spec correction, implementation work, exact blocker carry-forward, or promotion only when direct evidence warrants it.

## Position And Dependencies

- depends_on: `W039`
- parent epic: `calc-tv5`
- predecessor epic: `calc-f7o`
- upstream dependencies: `OxFml`
- canonical predecessor packet: `docs/spec/core-engine/w039-formalization/W039_CLOSURE_AUDIT_AND_RELEASE_GRADE_DECISION.md`

## Scope

### In Scope

1. Convert W039 residual lanes into W040 direct-verification obligations with concrete artifacts, promotion criteria, owners, and no-proxy guards.
2. Address optimized/core exact blockers for dynamic release/reclassification, snapshot-fence counterparts, capability-fence counterparts, and callable metadata projection with real fixes, direct differentials, proof obligations, or exact blockers.
3. Connect optimized/core behavior to Rust totality and refinement proof obligations.
4. Discharge or sharply retain Lean/TLA full-verification blockers with checked Lean artifacts and TLA/TLC model coverage.
5. Move Stage 2 evidence toward production partition-analyzer soundness and observable-result equivalence for promoted profiles.
6. Replace checked-in service substrate with operated runner, retained-history store/query API, replay-correlation evidence, and alert/quarantine dispatcher artifacts where feasible.
7. Establish an independently owned evaluator implementation surface and operated cross-engine differential evidence, or retain independence blockers exactly.
8. Exercise OxFml seam breadth where OxCalc consumes it, including W073 typed conditional-formatting metadata, public consumer surfaces, display/publication breadth, `LET`/`LAMBDA`, registered-external notes, and callable metadata projection.
9. Reassess pack-grade replay, C5, and release-grade verification only after direct W040 evidence is bound.

### Out Of Scope

1. General OxFunc semantic-kernel formalization beyond the narrow `LET`/`LAMBDA` carrier seam consumed by OxCalc.
2. Direct edits to OxFml from this repo.
3. Promotion from proxy evidence, file-backed-only evidence, local simulation, declared gaps, or aggregate confidence.
4. Treating performance/scaling measurements as semantic correctness proof.
5. Freezing a production coordinator API from fixture-host or note-level packet convergence alone.

## Gate Model

### Entry Gate

1. `calc-f7o` W039 parent epic has reached its recorded non-promoting release-grade successor target.
2. W039 closure audit and release-grade decision exists.
3. W040 successor beads exist in `.beads/`.
4. Latest OxFml W073 formatting intake remains carried as a seam-watch and verification obligation.

### Exit Gate

1. W039 residual lanes are mapped to W040 direct-verification evidence, implementation work, proof/model work, service work, OxFml watch/handoff rows, explicit deferral, or successor scope.
2. Optimized/core exact blockers are fixed, directly evidenced, spec-evolved, or retained with exact blocker ids and no declared-gap match promotion.
3. Rust totality/refinement claims are backed by direct tests, checked proof/model artifacts, or exact blockers.
4. Lean/TLA claims distinguish discharged proof, explicit assumption, model bound, totality boundary, and blocked row.
5. Stage 2 promotion claims require production-relevant partition soundness and observable-result invariance evidence for promoted profiles.
6. Operated service claims require runnable service artifacts, retained-history lifecycle, alert/quarantine dispatch behavior, and cross-engine differential operation where claimed.
7. Independent evaluator claims require independent implementation authority rather than projections over TraceCalc, optimized/core, or shared fixture rows.
8. OxFml seam claims distinguish current consumed surface, note-level watch lane, handoff trigger, and broad unexercised surface.
9. W073 typed conditional-formatting metadata is treated as typed-only for aggregate and visualization families; any OxCalc assumption of W072 threshold fallback for those families is recorded as a mismatch or blocker.
10. Pack/C5 and release-grade decisions state exact evidence consequence and no-promotion blockers or direct promotion rationale.
11. Closure audit includes a prompt-to-artifact checklist, OPERATIONS Section 7 checklist, Section 9 self-audit, semantic-equivalence statement, direct-evidence coverage audit, and three-axis report.

## Bead Rollout

Parent:

1. `calc-tv5` - W040 core formalization release-grade direct verification.

Child path:

1. `calc-tv5.1` - W040 residual direct-verification obligation map.
2. `calc-tv5.2` - W040 optimized core exact blocker fixes and differentials.
3. `calc-tv5.3` - W040 Rust totality and refinement proof tranche.
4. `calc-tv5.4` - W040 Lean TLA full-verification discharge tranche.
5. `calc-tv5.5` - W040 Stage 2 production policy and equivalence implementation.
6. `calc-tv5.6` - W040 operated assurance and retained history service implementation.
7. `calc-tv5.7` - W040 independent evaluator implementation and operated differential.
8. `calc-tv5.8` - W040 OxFml seam breadth and callable metadata implementation.
9. `calc-tv5.9` - W040 pack-grade replay governance and C5 promotion decision.
10. `calc-tv5.10` - W040 closure audit and release-grade verification decision.

## Initial Guardrails

1. Release-grade claims require direct artifacts and gate-specific evidence.
2. TraceCalc remains the correctness oracle for covered reference behavior, not a substitute for optimized/core, service, or OxFml breadth evidence.
3. A declared gap is not a match.
4. Bounded proof/model artifacts are not total verification unless their assumptions and bounds are discharged for the claimed scope.
5. Stage 2 strategy changes require semantic-equivalence evidence that observable results are invariant for the promoted profiles.
6. Operated service claims require operated evidence, not local file-backed simulation.
7. Fully independent evaluator diversity requires independent implementation authority.
8. W073 aggregate and visualization conditional-formatting metadata is `typed_rule`-only for the named families; W072 threshold strings are not a fallback there.
9. `LET`/`LAMBDA` remains a narrow OxCalc/OxFml/OxFunc carrier seam, not general OxFunc kernel formalization inside OxCalc.

## Current Status

- execution_state: `calc-tv5_bootstrapped_ready`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - `calc-tv5.1` residual direct-verification obligation map is next
  - all W039 non-promotion release-grade lanes remain open for direct W040 evidence
