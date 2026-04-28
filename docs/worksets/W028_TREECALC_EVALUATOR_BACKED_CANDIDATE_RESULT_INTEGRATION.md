# W028: TreeCalc Evaluator-Backed Candidate Result Integration

## Purpose
Move the live engine path from synthetic or proving-lane candidate intake to real OxFml-backed evaluator outputs for the first TreeCalc-ready formula families.
This packet widens execution truth beneath the existing `OxCalcTree` host-facing consumer contract rather than changing host ownership of the runtime contract.

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
- the `OxCalcTree` consumer contract remains the preferred host-facing entry surface while evaluator-backed candidate flow widens underneath it

### Exit gate
- the coordinator consumes real seam-produced candidate results and typed rejects for the covered TreeCalc scope
- reject-is-no-publish is exercised over real formula-driven candidate intake
- verified-clean semantics are explicit and evidenced for the live path

## Current Executed Floor
The first W028 floor is now exercised in live OxCalc code against the normal OxFml consumer baseline at `../OxFml` HEAD `487a5cfedc342f3983576d553cfc798941ab96bd`.
The earlier W028 guard that used pinned OxFml baseline `9aca95a8598124d8fc2125bd264469daeda5185f` and temporary worktree `../OxFml_W028_9aca95a` was historical quarantine evidence; OxFml W066 resolved that guard, and the ordinary validated baseline is now normal `../OxFml` HEAD `487a5cf`.

Current realized evaluator-backed candidate packet:
1. W028 validation now uses the normal clean `../OxFml` workspace at HEAD `487a5cfedc342f3983576d553cfc798941ab96bd`; no `../OxFml_W028_9aca95a` Cargo redirect is required.
2. `RuntimeFormulaResult.candidate_result` is consumed before local coordinator publication.
3. OxCalc records deterministic candidate diagnostics:
   - `oxfml_candidate_result_id`
   - `oxfml_candidate_formula_stable_id`
   - `oxfml_candidate_trace_correlation_id`
   - candidate value-delta identity fields.
4. OxCalc records deterministic accepted commit diagnostics:
   - `oxfml_commit_candidate_result_id`
   - `oxfml_commit_attempt_id`
   - `oxfml_commit_formula_stable_id`
   - commit value-delta candidate identity.
5. OxCalc validates the accepted OxFml `CommitBundle` remains compatible with the evaluator candidate before local coordinator publication.
6. OxCalc records deterministic reject/no-publish diagnostics from OxFml `RejectRecord` where present:
   - reject code
   - formula id
   - optional commit id
   - trace correlation id
   - explicit `oxfml_reject_no_publish:true` marker.
7. Coordinator publication authority remains OxCalc-owned; evaluator success is not treated as local coordinator publication.
8. Verified-clean semantics are explicit over the evaluator-backed path: when the OxFml-backed candidate value equals the seeded published value, the node is marked `VerifiedClean`, candidate diagnostics are preserved, `verified_clean_publication_suppressed:<node>` is emitted, and no local `candidate_result` or `publication_bundle` is produced.

Current non-overclaim:
1. full runtime-derived dynamic dependency overlay closure belongs to `W029`.
2. broader retained-failure widening and full oracle/replay promotion remain successor work.
3. the W028 floor is scoped to the first TreeCalc-ready formula families covered by the current local engine and validated normal OxFml baseline.

## Pre-Closure Verification Checklist
Audit bead: `calc-uns`.

1. Spec text and realization notes updated for all in-scope items: yes — this packet records the current executed W028 floor and non-overclaim boundaries.
2. Pack expectations updated for affected packs: yes — no new pack family was introduced; W028 evidence is bound to the existing TreeCalc local runner, OxFml consumer facade, and validation surfaces.
3. At least one deterministic replay artifact exists per in-scope behavior: yes — checked-in TreeCalc local artifacts and runner tests preserve candidate, publication, reject, and verified-clean diagnostics for the current floor; the historical audit validated the pinned OxFml clean-baseline path, and the current post-W066 guard validates normal `../OxFml` HEAD `487a5cf`.
4. Semantic-equivalence statement provided for policy or strategy changes: yes / not applicable — W028 does not promote a scheduling strategy change; observable publication remains under OxCalc coordinator authority, while evaluator candidate/commit/reject diagnostics are made explicit.
5. FEC/F3E cross-repo impact assessed and handoff filed if needed: yes — no new OxFml canonical seam change is required; W028 now targets the validated normal OxFml baseline `487a5cfedc342f3983576d553cfc798941ab96bd` and does not require the historical quarantine worktree.
6. All required tests pass: yes — `cargo test --workspace`, scoped OxCalc `cargo fmt -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `scripts/check-worksets.ps1` pass against normal `../OxFml`; clippy reports only a non-fatal warning in sibling `OxFunc`.
7. No known semantic gaps remain in declared scope: yes — remaining runtime-derived overlay and broader oracle/replay lanes are explicitly outside W028 and belong to successor worksets.
8. Completion language audit passed: yes — closure language is limited to the declared W028 phase scope and does not claim W029-W031 semantics.
9. `IN_PROGRESS_FEATURE_WORKLIST.md` updated: yes / not applicable — feature-map truth did not change for this W028 closure audit.
10. `CURRENT_BLOCKERS.md` updated if needed: yes / not applicable — ordinary execution state is in `.beads/`; no prose blocker update is needed.

## Status
- execution_state: closure_recommended
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: integrated
- open_lanes: []
- closure_audit_result: pass for declared W028 phase scope
- next_ready_if_closed: `calc-k5i` / `W029 TreeCalc runtime-derived effects and overlay closure`
- non_scope_successors:
  - runtime-derived dynamic dependency and overlay closure belongs to `W029`
  - broader sequential corpus/oracle baseline evidence belongs to `W030`
  - assurance refresh and residual packetization belongs to `W031`
- claim_confidence: high for W028 declared phase scope
- reviewed_inbound_observations: W020 remains the carried seam-intake baseline; OxFml W066 resolved the historical W028 baseline quarantine guard, and normal clean `../OxFml` HEAD `487a5cfedc342f3983576d553cfc798941ab96bd` is the current validated baseline.
