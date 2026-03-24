# W028: TreeCalc Evaluator-Backed Candidate Result Integration

## Purpose
Move the live engine path from synthetic or proving-lane candidate intake to real OxFml-backed evaluator outputs for the first TreeCalc-ready formula families.

## Position and Dependencies
- **Depends on**: W026, W027
- **Blocks**: W029, W030, W031
- **Cross-repo**: may require a narrower handoff only if the real evaluator-backed candidate or reject payloads prove insufficient for the coordinator obligations already declared

## Scope
### In scope
1. evaluator-backed candidate-result intake for in-scope TreeCalc formula families
2. typed reject and no-publish intake for first TreeCalc-ready scope
3. coordinator accept/reject/publication driven by real seam-produced candidate objects
4. verified-clean semantics in the real formula-driven path
5. deterministic diagnostics for candidate, reject, and publish consequences over the live TreeCalc path

### Out of scope
1. full runtime-derived dynamic dependency overlay closure
2. broader retained-failure widening beyond what is needed for the live TreeCalc path
3. concurrency or async realization

## Deliverables
1. a real evaluator-backed candidate intake path wired into the Rust coordinator
2. typed reject handling over the live TreeCalc path for first-phase families
3. deterministic publication diagnostics and artifacts for real formula-driven candidate intake
4. explicit verified-clean behavior over the real formula path

## Gate Model
### Entry gate
- W027 has produced the real dependency and invalidation substrate
- W026 has defined the candidate and reject seam floor for TreeCalc scope

### Exit gate
- the coordinator consumes real seam-produced candidate results and typed rejects for the covered TreeCalc scope
- reject-is-no-publish is exercised over real formula-driven candidate intake
- verified-clean semantics are explicit and evidenced for the live path

## Pre-Closure Verification Checklist
1. Spec text and realization notes updated for all in-scope items: no
2. Pack expectations updated for affected packs: no
3. At least one deterministic replay artifact exists per in-scope behavior: no
4. Semantic-equivalence statement provided for policy or strategy changes: no
5. FEC/F3E cross-repo impact assessed and handoff filed if needed: no
6. All required tests pass: no
7. No known semantic gaps remain in declared scope: no
8. Completion language audit passed: no
9. `IN_PROGRESS_FEATURE_WORKLIST.md` updated: no
10. `CURRENT_BLOCKERS.md` updated if needed: no

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - the first direct-host OxFml slice now drives local candidate adaptation and typed reject handling, but broader W026 bind/reference intake is still open
  - verified-clean semantics are evidenced only for the current local TreeCalc subset, not yet for the broader first TreeCalc-ready family set
  - publication, reject, and candidate artifacts are still local-floor TreeCalc evidence rather than the later live oracle/replay lane
- claim_confidence: draft
- reviewed_inbound_observations: W020 remains the carried seam-intake baseline
