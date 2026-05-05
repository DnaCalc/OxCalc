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

- execution_state: `calc-rqq.2_tracecalc_coverage_closure_matrix_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-rqq.3` through `calc-rqq.9` remain blocked by the sequential W036 path
  - full Lean/TLA verification remains open
  - full TraceCalc oracle coverage remains unpromoted because the W036 matrix still has one uncovered TLA-owned row and one OxFunc-kernel exclusion
  - full optimized/core-engine verification and fully independent evaluator diversity remain open
  - concrete Stage 2 partition modeling and replay equivalence remain open
  - pack-grade replay, continuous-scale service operation, continuous cross-engine differential service, and Stage 2 policy remain unpromoted

Latest W036 evidence:

1. `docs/spec/core-engine/w036-formalization/W036_RESIDUAL_COVERAGE_AND_PROMOTION_BLOCKER_LEDGER.md` records the W036 residual coverage ledger for `calc-rqq.1`, mapping W035 no-promotion blockers to 20 W036 obligations, owners, evidence roots, and promotion consequences.
2. `docs/spec/core-engine/w036-formalization/W036_TRACECALC_COVERAGE_CLOSURE_CRITERIA_AND_MATRIX_EXPANSION.md` records the `calc-rqq.2` W036 TraceCalc coverage criteria and matrix expansion. The checked run `w036-tracecalc-coverage-closure-001` emits 32 matrix rows, 30 covered rows, 1 classified uncovered row, 1 excluded row, 0 failed/missing rows, 0 no-loss crosswalk gaps, and no full oracle claim.
