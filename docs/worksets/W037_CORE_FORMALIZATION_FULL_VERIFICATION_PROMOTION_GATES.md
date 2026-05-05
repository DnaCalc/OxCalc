# W037: Core Formalization Full-Verification Promotion Gates

## Purpose

W037 continues the formalization path after W036.

W036 produced a strong verification-closure expansion tranche and kept the remaining no-promotion blockers exact. W037 turns those blockers into direct evidence work where possible: full-verification residual mapping, TraceCalc observable closure, optimized/core-engine conformance closure, direct OxFml evaluator evidence, narrow `LET`/`LAMBDA` seam evidence, proof/model closure inventory, Stage 2 deterministic replay criteria, operated assurance, and pack/C5 reassessment.

W037 is not a fixed-spec test pass. It treats current specs, current implementations, TraceCalc behavior, optimized/core behavior, and OxFml seam behavior as evidence surfaces that may reveal spec corrections, implementation faults, promotion blockers, or successor scope. Promotion claims may only be made from direct artifacts.

## Position And Dependencies

- depends_on: `W036`
- parent epic: `calc-ubd`
- predecessor epic: `calc-rqq`
- upstream dependencies: `OxFml`
- canonical predecessor packet: `docs/spec/core-engine/w036-formalization/W036_CLOSURE_AUDIT_AND_SUCCESSOR_PACKET.md`

## Scope

### In Scope

1. Convert W036 closure residuals and no-promotion blockers into W037 obligations with owners, evidence roots, and promotion consequences.
2. Move TraceCalc toward observable closure for remaining in-scope rows, including multi-reader/overlay release ordering where OxCalc owns the behavior.
3. Convert W036 optimized/core-engine conformance blockers into implementation fixes, direct differential matches, spec corrections, or explicit residual blockers.
4. Exercise or explicitly block direct OxFml evaluator re-execution needed for pack-grade replay evidence.
5. Include the narrow `LET`/`LAMBDA` OxFml/OxFunc carrier fragment that threads through OxCalc and OxFml core-engine work while keeping general OxFunc kernels outside scope.
6. Preserve OxFml formatting watch/input-contract guardrails for any direct OxFml evaluator path: W073 typed conditional-formatting metadata, distinct `format_delta` versus `display_delta`, and no broad display-facing closure claim from semantic-format evidence.
7. Convert W036 Lean/TLA inventories into stronger proof/model closure claims only where runnable artifacts and assumption ledgers support them.
8. Define Stage 2 deterministic replay and partition promotion criteria, including observable-result invariance under scheduler or partition strategy changes.
9. Move simulated continuous assurance toward operated multi-run evidence and, where feasible, a cross-engine service pilot.
10. Reassess pack-grade replay, C5 capability, and Stage 2 policy only after direct evidence is bound.

### Out Of Scope

1. General OxFunc semantic kernels beyond the narrow `LET`/`LAMBDA` carrier boundary consumed by OxCalc.
2. Direct edits to OxFml from this repo.
3. Production Stage 2 policy promotion without direct replay, partition, differential, and semantic-equivalence evidence.
4. C5 or pack-grade replay promotion from coverage matrices, bounded proof/model slices, simulated continuous assurance, declared-gap classifications, or timing evidence alone.
5. UI, host, or file-adapter work unless directly required by a W037 evidence artifact.

## Gate Model

### Entry Gate

1. `calc-rqq` W036 parent epic has closed.
2. W036 closure audit packet exists.
3. W037 successor beads exist in `.beads/`.

### Exit Gate

1. Every W036 open lane is mapped to W037 evidence, implementation work, handoff/watch rows, explicit deferral, or successor scope.
2. TraceCalc observable closure is either supported by deterministic replay for every in-scope row or exact residual rows remain.
3. Optimized/core-engine conformance gaps are either replay/diff-promoted, fixed, spec-evolved, or carried as blockers.
4. Direct OxFml evaluator and `LET`/`LAMBDA` seam evidence is exercised or remains a named blocker for pack-grade replay.
5. Lean/TLA proof/model claims distinguish checked artifacts, assumptions, bounds, external seams, and proof gaps.
6. Stage 2 deterministic replay and partition promotion criteria state semantic-equivalence obligations before any scheduler-policy claim.
7. Operated continuous-assurance and cross-engine service claims require operated multi-run artifacts; otherwise simulation remains explicitly non-promoting.
8. Pack/C5 decisions state exact evidence and no-promotion blockers or direct promotion rationale.
9. Closure audit includes a prompt-to-artifact objective checklist, OPERATIONS Section 7 checklist, Section 9 self-audit, semantic-equivalence statement, and three-axis report.

## Bead Rollout

Parent:

1. `calc-ubd` - W037 core formalization full-verification promotion gates.

Child path:

1. `calc-ubd.2` - W037 residual full-verification and promotion-gate ledger.
2. `calc-ubd.1` - W037 TraceCalc observable closure and multi-reader replay.
3. `calc-ubd.3` - W037 optimized/core-engine conformance implementation closure.
4. `calc-ubd.4` - W037 direct OxFml evaluator and `LET`/`LAMBDA` seam evidence.
5. `calc-ubd.5` - W037 Lean/TLA proof and model closure inventory.
6. `calc-ubd.6` - W037 Stage 2 deterministic replay and partition promotion criteria.
7. `calc-ubd.7` - W037 operated continuous assurance and cross-engine service pilot.
8. `calc-ubd.8` - W037 pack-grade replay governance and C5 candidate decision.
9. `calc-ubd.9` - W037 closure audit and full-verification release decision.

The bead suffix order reflects creation timing for the first W037 children. Readiness is governed by the dependency chain.

## Initial Guardrails

1. TraceCalc remains the correctness oracle only for covered reference behavior.
2. A coverage row is not a proof of totality unless every declared in-scope observable row has deterministic replay evidence or authority-exclusion.
3. TreeCalc/CoreEngine comparison rows must not count declared gaps as matches.
4. Lean/TLA artifacts must state which obligations are proved locally, assumed, bounded, external, or blocked.
5. Direct OxFml evaluator evidence is required before pack-grade replay can rely on OxFml semantics.
6. `LET`/`LAMBDA` is a narrow carrier seam, not permission to formalize general OxFunc kernels inside OxCalc.
7. Continuous assurance requires operated recurring evidence for service claims; simulated history remains non-promoting.
8. Any strategy or scheduler change must include a semantic-equivalence statement that observable results are invariant for the affected profile.
9. W073 conditional-formatting aggregate or visualization payloads must emit `VerificationConditionalFormattingRule.typed_rule`; `thresholds` remains only for scalar/operator/expression rule families where threshold text is the actual input.
10. `format_delta` and `display_delta` remain distinct consequence categories; format dependency tokens, locale/date-system inputs, and replayable format-sensitive outcomes are hooks to carry when exercised, not implicit publication obligations for every W037 artifact.

## Current Status

- execution_state: `calc-ubd.4_direct_oxfml_evaluator_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-ubd.5` is the next ready W037 bead
  - `calc-ubd.6` through `calc-ubd.9` remain blocked by the sequential W037 path
  - full Lean/TLA verification remains open
  - full TraceCalc oracle promotion remains unclaimed because authority exclusions and non-TraceCalc gates remain
  - full optimized/core-engine verification remains open with five explicit W037 residual conformance blockers
  - fully independent evaluator diversity remains open
  - direct OxFml evaluator re-execution is now exercised for the upstream-host fixture slice, but pack-grade replay remains unpromoted
  - the narrow `LET`/`LAMBDA` seam is exercised for two direct OxFml rows, while general OxFunc callable kernels remain outside OxCalc scope
  - Stage 2 deterministic replay and partition promotion criteria remain open
  - pack-grade replay, C5, operated continuous-assurance service, operated continuous cross-engine differential service, and enforcing alert/quarantine service remain unpromoted

Latest W037 evidence:

1. `docs/spec/core-engine/w037-formalization/W037_RESIDUAL_FULL_VERIFICATION_AND_PROMOTION_GATE_LEDGER.md` records the `calc-ubd.2` W037 residual full-verification and promotion-gate ledger, mapping W036 no-promotion blockers to W037 owners, evidence roots, and promotion consequences.
2. `docs/spec/core-engine/w037-formalization/W037_TRACECALC_OBSERVABLE_CLOSURE_AND_MULTI_READER_REPLAY.md` records the `calc-ubd.1` TraceCalc observable-closure slice and the W037 oracle matrix run with 32 rows, 31 covered rows, 0 uncovered rows, 1 authority-excluded row, 0 failed/missing rows, and no full oracle claim.
3. `docs/spec/core-engine/w037-formalization/W037_OPTIMIZED_CORE_ENGINE_CONFORMANCE_IMPLEMENTATION_CLOSURE.md` records the `calc-ubd.3` optimized/core-engine conformance decision slice. The TreeCalc run `w037-optimized-core-conformance-treecalc-001` emits 24 cases with 0 expectation mismatches and adds a resolved dynamic dependency publication case. The implementation-conformance run `w037-implementation-conformance-closure-001` emits 6 decision rows, 1 fixed/promoted row, 5 residual blockers, 1 match-promoted row, 0 failed rows, and no full optimized/core-engine verification claim.
4. `docs/spec/core-engine/w037-formalization/W037_DIRECT_OXFML_EVALUATOR_AND_LET_LAMBDA_SEAM_EVIDENCE.md` records the `calc-ubd.4` direct OxFml evaluator slice. The upstream-host run `w037-direct-oxfml-evaluator-001` emits 12 cases with 0 expectation mismatches, including 3 direct-OxFml rows, 2 `LET`/`LAMBDA` rows, and 1 W073 typed conditional-formatting guard row. Pack-grade replay and C5 remain unpromoted.
