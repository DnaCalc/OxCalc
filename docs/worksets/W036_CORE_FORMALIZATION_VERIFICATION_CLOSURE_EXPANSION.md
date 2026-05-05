# W036: Core Formalization Verification Closure Expansion

## Purpose

W036 continues the formalization path after W035.

W035 turned W034 residuals into stronger TraceCalc, implementation-conformance, Lean, TLA, continuous-assurance, and pack/Stage 2 gate evidence. It also made the remaining no-promotion blockers explicit. W036 converts those blockers into the next execution tranche for deeper coverage, stronger proof/model evidence, implementation gap closure, operated continuous assurance, and pack-grade readiness.

W036 is still not a promotion workset by default. It may produce a promotion candidate only where direct artifacts satisfy the relevant gate. Otherwise it must record exact blockers and carry them forward.

## Position And Dependencies

- depends_on: `W035`
- parent epic: `calc-rqq`
- predecessor epic: `calc-tkq`
- upstream dependencies: `OxFml`
- canonical predecessor packet: `docs/spec/core-engine/w035-formalization/W035_CLOSURE_AUDIT_AND_SUCCESSOR_PACKET.md`

## Scope

### In Scope

1. Convert W035 no-promotion blockers into W036 proof obligations, implementation work, replay/evidence work, handoff/watch rows, or explicit deferrals.
2. Expand TraceCalc coverage criteria from bounded matrices toward a current observable-semantics closure matrix.
3. Convert W035 implementation-conformance deferrals into concrete optimized/core-engine fixes, harness work, or authoritative blockers.
4. Expand Lean theorem coverage beyond W035 assumption classification rows while preserving external seam and OxFunc-opaque boundaries.
5. Replace the W035 abstract Stage 2 partition-soundness input with a concrete bounded partition/ownership TLA model where practical.
6. Define and exercise stronger independent-evaluator diversity and cross-engine differential evidence.
7. Evolve W035 continuous-assurance packet shape toward operated evidence with schedule, history-window criteria, regression thresholds, and quarantine/alert policy where practical.
8. Reassess pack-grade replay and capability promotion only after W036 evidence is bound.
9. Preserve OxFml W073 typed conditional-formatting direction as a watch/input-contract guardrail until a concrete OxCalc request path exercises it.

### Out Of Scope

1. General OxFunc semantic kernels beyond the narrow `LET`/`LAMBDA` carrier boundary consumed by OxCalc.
2. Direct edits to OxFml from this repo.
3. Stage 2 policy promotion without concrete partition, replay-equivalence, differential, and pack-grade gate evidence.
4. Pack-grade replay or continuous-scale promotion based on bounded smoke checks, single-run timing, generated matrices alone, or declared-gap proxy evidence.
5. UI, host, or file-adapter work unless directly required by a W036 proof/conformance artifact.

## Gate Model

### Entry Gate

1. `calc-tkq` W035 parent epic has closed.
2. W035 closure audit packet exists.
3. W036 successor beads exist in `.beads/`.

### Exit Gate

1. Every W035 open lane is mapped to W036 evidence, implementation work, handoff/watch rows, or explicit deferral.
2. TraceCalc coverage closure criteria are machine-readable and no full oracle claim is made unless all in-scope rows have deterministic replay evidence.
3. Optimized/core-engine conformance gaps are either resolved with evidence or carried as explicit blockers.
4. Lean/TLA artifacts distinguish checked proof/model evidence from assumptions, bounded exploration, and external seams.
5. Independent-evaluator diversity and cross-engine differential evidence state actual diversity limits and mismatch consequences.
6. Continuous-assurance evidence states whether an operated or simulated multi-run lane exists and keeps timing subordinate to semantic correctness.
7. Pack/Stage 2 decisions state exact evidence and no-promotion blockers or promotion rationale.
8. Closure audit includes a prompt-to-artifact objective checklist, OPERATIONS Section 7 checklist, Section 9 self-audit, semantic-equivalence statement, and three-axis report.

## Bead Rollout

Parent:

1. `calc-rqq` - W036 core formalization verification closure expansion.

Child path:

1. `calc-rqq.1` - W036 residual coverage and promotion-blocker ledger.
2. `calc-rqq.2` - W036 TraceCalc coverage closure criteria and matrix expansion.
3. `calc-rqq.3` - W036 optimized/core-engine conformance closure plan and first fixes.
4. `calc-rqq.4` - W036 Lean theorem coverage expansion.
5. `calc-rqq.5` - W036 TLA Stage 2 partition and scheduler equivalence model.
6. `calc-rqq.6` - W036 independent evaluator diversity and cross-engine differential harness.
7. `calc-rqq.7` - W036 continuous assurance operation and history window.
8. `calc-rqq.8` - W036 pack-grade replay and capability promotion gate reassessment.
9. `calc-rqq.9` - W036 closure audit and successor/full-verification decision.

The first W036 path is sequential to keep proof, replay, implementation, and promotion consequences auditable. Later W036 work may split only after the W036 ledger identifies disjoint evidence scopes.

## Initial Guardrails

1. TraceCalc remains the correctness oracle only for covered reference behavior.
2. Coverage matrices are evidence maps, not proof of totality unless every declared in-scope row has deterministic replay artifacts.
3. TreeCalc/CoreEngine comparison rows must not count declared gaps as matches.
4. Lean/TLA artifacts must state which obligations are proved locally, assumed, bounded by model size, or external to OxCalc.
5. Continuous assurance requires recurring or clearly simulated multi-run evidence and cross-engine differential criteria; timing remains measurement evidence.
6. OxFml-owned semantic or FEC/F3E changes require handoff packets rather than direct sibling-repo edits.
7. Any W036 artifact that constructs W073 conditional-formatting aggregate or visualization payloads must emit `VerificationConditionalFormattingRule.typed_rule`; `thresholds` is retained only for scalar/operator/expression rule families where threshold text is the actual input.

## Current Status

- execution_state: `calc-rqq.9_closure_audit_and_successor_packet`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - W037 successor tranche remains open
  - full Lean verification remains partial because W036 adds checked proof-inventory slices, not total Rust-engine proof
  - full TLA verification remains partial because W036 adds bounded model-check evidence, not total proof
  - full TraceCalc oracle coverage remains unpromoted because the multi-reader row is now bounded TLA model evidence rather than TraceCalc replay and the OxFunc-kernel exclusion remains external
  - full optimized/core-engine verification remains open because the W036 implementation-conformance closure run promotes zero W035 declared gaps as matches
  - fully independent evaluator diversity remains partial because W036 classifies 0 fully independent evaluator rows
  - direct OxFml evaluator re-execution and the narrow `LET`/`LAMBDA` seam evidence remain open
  - operated continuous-assurance service and continuous cross-engine differential service remain open because W036 emits simulated multi-run history and deterministic harness artifacts, not an operated service
  - alert/quarantine execution remains open because W036 defines policy, not an enforcing alert dispatcher
  - bounded Stage 2 partition ownership modeling exists, while production partitioning and deterministic scheduler replay equivalence remain open
  - pack C5, pack-grade replay, continuous-scale service operation, continuous cross-engine differential service, and Stage 2 policy remain unpromoted

Latest W036 evidence:

1. `docs/spec/core-engine/w036-formalization/W036_RESIDUAL_COVERAGE_AND_PROMOTION_BLOCKER_LEDGER.md` records the W036 residual coverage ledger for `calc-rqq.1`, mapping W035 no-promotion blockers to 20 W036 obligations, owners, evidence roots, and promotion consequences.
2. `docs/spec/core-engine/w036-formalization/W036_TRACECALC_COVERAGE_CLOSURE_CRITERIA_AND_MATRIX_EXPANSION.md` records the `calc-rqq.2` W036 TraceCalc coverage criteria and matrix expansion. The checked run `w036-tracecalc-coverage-closure-001` emits 32 matrix rows, 30 covered rows, 1 classified uncovered row, 1 excluded row, 0 failed/missing rows, 0 no-loss crosswalk gaps, and no full oracle claim.
3. `docs/spec/core-engine/w036-formalization/W036_OPTIMIZED_CORE_ENGINE_CONFORMANCE_CLOSURE_PLAN_AND_FIRST_FIXES.md` records the `calc-rqq.3` W036 optimized/core-engine conformance closure plan and first-fix harness evidence. The checked run `w036-implementation-conformance-closure-001` emits 6 closure action rows, 2 harness first-fix rows, 4 blocker-routed rows, 0 match-promoted rows, and 0 failed rows.
4. `docs/spec/core-engine/w036-formalization/W036_LEAN_THEOREM_COVERAGE_EXPANSION.md` records the `calc-rqq.4` W036 Lean theorem coverage expansion. It adds checked Lean artifacts for W036 coverage inventory and callable boundary inventory, with zero explicit axioms, zero match-promoted rows, zero full Lean promotion, and explicit routing for callable/OxFunc, TLA, and conformance/harness boundaries.
5. `docs/spec/core-engine/w036-formalization/W036_TLA_STAGE2_PARTITION_AND_SCHEDULER_EQUIVALENCE_MODEL.md` records the `calc-rqq.5` W036 TLA Stage 2 partition and scheduler-equivalence model. The checked TLC run `w036-stage2-partition-001` covers five configs, 0 failed configs, bounded partition ownership, snapshot/capability fence rejection, multi-reader overlay release ordering, scheduler-readiness criteria, and no Stage 2 policy promotion.
6. `docs/spec/core-engine/w036-formalization/W036_INDEPENDENT_EVALUATOR_DIVERSITY_AND_CROSS_ENGINE_DIFFERENTIAL_HARNESS.md` records the `calc-rqq.6` W036 independent evaluator diversity and cross-engine differential harness. The checked run `w036-independent-diversity-differential-001` emits 15 base comparison rows, 5 diversity rows, 6 cross-engine differential rows, 6 promotion blockers, 0 unexpected mismatches, 0 missing artifacts, 0 fully independent evaluator rows, and no continuous cross-engine service promotion.
7. `docs/spec/core-engine/w036-formalization/W036_CONTINUOUS_ASSURANCE_OPERATION_AND_HISTORY_WINDOW.md` records the `calc-rqq.7` W036 continuous-assurance operation/history packet. The checked run `w036-continuous-assurance-operation-001` emits 11 source rows, 4 scheduled lanes, 6 differential rows, 6 simulated history rows, 7 regression threshold rules, 7 quarantine/alert rules, 0 missing artifacts, 0 unexpected mismatches, 11 no-promotion reasons, and no operated continuous service promotion.
8. `docs/spec/core-engine/w036-formalization/W036_PACK_GRADE_REPLAY_AND_CAPABILITY_PROMOTION_GATE_REASSESSMENT.md` records the `calc-rqq.8` W036 pack-grade replay and capability reassessment. The checked run `w036-pack-capability-reassessment-001` emits 12 satisfied inputs, 22 no-promotion blockers, 0 missing artifacts, `cap.C4.distill_valid` as highest honest capability, no C5 promotion, and no Stage 2 policy promotion.
9. `docs/spec/core-engine/w036-formalization/W036_CLOSURE_AUDIT_AND_SUCCESSOR_PACKET.md` records the `calc-rqq.9` W036 closure audit and successor/full-verification decision. It maps the active formalization objective to W036 evidence, keeps the broader objective partial, and registers W037 as the next direct-evidence tranche.
