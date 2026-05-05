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

- execution_state: `calc-tv5.7_bounded_independent_evaluator_diversity_validated_no_promotion`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-tv5.8` OxFml seam breadth and callable metadata implementation is next
  - full Lean verification, full TLA verification, fairness and unbounded scheduler coverage, Rust totality/refinement dependencies, automatic dynamic dependency-set transition differential, panic-free whole-engine Rust totality, full production partition analyzer soundness, operated recurring scheduler/service endpoint, external alert/quarantine dispatcher, operated cross-engine differential service, full independent-evaluator breadth, retention SLO, pack-grade replay governance, callable metadata projection, OxFml seam breadth, pack/C5, and release-grade decision remain open

Latest evidence:

1. `docs/spec/core-engine/w040-formalization/W040_RESIDUAL_DIRECT_VERIFICATION_OBLIGATION_MAP.md` records the `calc-tv5.1` direct-verification obligation map. The packet `w040-direct-verification-obligation-map-001` maps 12 post-W039 residual lanes into 23 W040 obligations, 11 promotion-target gates, current OxFml W073 direct typed-only conditional-formatting intake, owner beads, required evidence, promotion consequences, and spec-evolution hooks without promoting release-grade verification, C5, pack-grade replay, Stage 2 policy, operated services, independent evaluator diversity, broad OxFml display/publication, callable metadata, or general OxFunc kernels.
2. `docs/spec/core-engine/w040-formalization/W040_OPTIMIZED_CORE_EXACT_BLOCKER_FIXES_AND_DIFFERENTIALS.md` records the `calc-tv5.2` optimized/core exact-blocker slice. The packet `w040-optimized-core-exact-blocker-fixes-differentials-001` adds explicit dependency-change invalidation fixture support, emits `w040-optimized-core-dynamic-release-reclassification-001` with 25 TreeCalc cases and 0 expectation mismatches, binds direct evidence that `DependencyRemoved` and `DependencyReclassified` force rebind/no-publication behavior, and retains four exact blockers without match promotion.
3. `docs/spec/core-engine/w040-formalization/W040_RUST_TOTALITY_AND_REFINEMENT_PROOF_TRANCHE.md` records the `calc-tv5.3` Rust totality/refinement slice. The packet `w040-rust-totality-refinement-proof-tranche-001` adds a checked W040 Lean classification file and formal-assurance runner profile, records 10 Rust/proof rows, 7 local checked-proof classification rows, 5 totality boundaries, 5 refinement rows, 5 exact blockers, and 0 failed rows, while retaining whole-engine Rust totality, panic-free core domain, automatic dynamic transition detection, snapshot/capability counterparts, callable metadata projection, broad OxFml, general OxFunc, Stage 2, pack/C5, and release-grade promotion blockers.
4. `docs/spec/core-engine/w040-formalization/W040_LEAN_TLA_FULL_VERIFICATION_DISCHARGE_TRANCHE.md` records the `calc-tv5.4` Lean/TLA proof/model slice. The packet `w040-lean-tla-full-verification-discharge-001` adds a checked W040 Lean classification file and formal-assurance runner profile, records 11 proof/model rows, 6 local checked-proof rows, 3 bounded-model rows, 1 accepted external seam, 2 accepted boundaries, 5 totality boundaries, 5 exact blockers, and 0 failed rows. It also reruns the five W036 Stage 2 partition TLC configs locally for W040, while retaining full Lean verification, full TLA verification, unbounded fairness/scheduler coverage, Rust totality/refinement dependencies, Stage 2 production policy, broad OxFml, general OxFunc, pack/C5, and release-grade promotion blockers.
5. `docs/spec/core-engine/w040-formalization/W040_STAGE2_PRODUCTION_POLICY_AND_EQUIVALENCE_IMPLEMENTATION.md` records the `calc-tv5.5` Stage 2 policy/equivalence slice. The packet `w040-stage2-production-policy-equivalence-001` adds a checked W040 Lean predicate and Stage 2 runner profile, records 12 policy rows, 8 satisfied policy rows, 5 partition replay rows, 6 permutation rows, 5 observable-invariance rows, 1 formatting watch row, snapshot/capability fence counterparts evidenced, bounded analyzer evidence, 4 exact blockers, and 0 failed rows. Stage 2 policy remains unpromoted until full production partition analyzer soundness, fairness/unbounded scheduler coverage, operated cross-engine service evidence, and pack-grade replay governance are present.
6. `docs/spec/core-engine/w040-formalization/W040_OPERATED_ASSURANCE_AND_RETAINED_HISTORY_SERVICE_IMPLEMENTATION.md` records the `calc-tv5.6` operated-assurance and retained-history service-artifact slice. The packet `w040-operated-assurance-retained-history-service-001` adds a W040 operated-assurance runner branch, records 10 source rows, 6 runner rows, 21 retained-history store rows, 5 query rows, 3 replay-correlation rows, 14 alert/quarantine rows, 14 readiness criteria, 4 exact service blockers, and 0 failed rows. It evidences file-backed runner, retained-history store/query, replay-correlation, and local dispatcher artifacts while keeping operated continuous assurance, retained-history service, external alert/quarantine dispatcher, operated cross-engine differential, Stage 2, pack/C5, and release-grade promotion unpromoted.
7. `docs/spec/core-engine/w040-formalization/W040_INDEPENDENT_EVALUATOR_IMPLEMENTATION_AND_OPERATED_DIFFERENTIAL.md` records the `calc-tv5.7` independent-evaluator and operated-differential slice. The packet `w040-independent-evaluator-operated-differential-001` adds a bounded independent scalar arithmetic evaluator, records 10 source rows, 5 independent scalar cases, 5 scalar matches, 8 independent-evaluator rows, 8 cross-engine differential rows, 7 differential-authority rows, 5 exact blockers, and 0 failed rows. It narrows independent implementation evidence for the bounded scalar slice while keeping full independent-evaluator breadth, operated cross-engine differential service, Stage 2 service dependency, mismatch/quarantine service, pack/C5, and release-grade promotion unpromoted.
