# W014: Execution Sequence B - Stage 1 Widening and Evidence Hardening

## Purpose
Operationalize the second serious OxCalc implementation wave after `W013`.

This packet exists to:
1. widen the current Stage 1 engine slice from a single-node baseline into a broader multi-node engine floor,
2. harden replay, measurement, and pack evidence around that widened floor,
3. prepare the Stage 1 baseline for later Stage 2 concurrency work without promoting concurrency early,
4. keep the next execution wave focused on the largest remaining Stage 1 gaps rather than reopening architecture choices.

## Position and Dependencies
- **Depends on**: W004, W009, W010, W011, W012, W013
- **Blocks**: none
- **Cross-repo**: assumes accepted shared seam direction from `HANDOFF-CALC-001`; any narrower trace-, reject-, or runtime-effect pressure discovered during this wave must route through W005 rather than being normalized locally

## Scope
### In scope
1. Multi-node Stage 1 recalc widening beyond the current single-node floor.
2. Deterministic topo scheduling for TreeCalc Stage 1 DAG cases.
3. First SCC-oriented Stage 1 handling for bounded cycle-region or iterative-profile scenarios.
4. Emitted runtime counters from running code, not schema-only definitions.
5. Replay widening for the remaining priority Stage 1 classes `R3`, `R6`, and `R8`.
6. Harness and corpus widening to cover multi-node DAG and first SCC-oriented scenarios.
7. Pack-evidence hardening for the Stage 1 surfaces exercised by the widened run.
8. A checked-in widened baseline run with deterministic emitted artifacts.

### Out of scope
1. Stage 2 concurrency or async realization.
2. Dynamic-topological maintenance promotion.
3. Full SAC-style repair.
4. Grid substrate introduction.
5. Closing all Stage 1 feature areas in full.

## Sequence Preconditions
Execution Sequence B assumes the following preconditions already hold:
1. `W013` has reached its final gate and its emitted baseline run is checked in,
2. the `TraceCalc` validator, runner, and reference machine exist and are exercised,
3. the first Lean-facing and TLA+-facing Stage 1 artifacts exist,
4. the current replay corpus already covers `R1`, `R2`, `R7`, `R4`, and `R5`,
5. the next-wave doctrine from `docs/LOCAL_EXECUTION_DOCTRINE.md` has been adopted.

## Execution Environment
Required tools:
1. `cargo`
2. `lean`

Optional tools:
1. `tlc`

Fallback rules:
1. if `tlc` is unavailable, the TLA+ lane remains authored and updated, but the report must state that no model-check run was executed,
2. if generated corpus tooling is not yet separate from the checked-in host, the wave may start with checked-in generated samples as long as they remain deterministic and traceable.

## Evidence Layout
Canonical emitted artifact root:
1. `docs/test-runs/core-engine/tracecalc-reference-machine/`

Checked-in baseline run for this sequence:
1. `w014-stage1-widening-baseline`

Checked-in policy:
1. the widened baseline run is tracked,
2. ad hoc validation or comparison runs must use distinct run ids and remain untracked unless intentionally promoted,
3. tracked emitted artifacts must use repo-relative paths only.

## Replay-Corpus Readiness
Required replay classes for this sequence and their minimum scenario ids:
1. `R3` -> one compatible publish branch and one incompatible reject branch
2. `R6` -> typed reject taxonomy scenario set covering at least snapshot or fence mismatch, token or artifact mismatch, and capability mismatch
3. `R8` -> fallback and overlay re-entry scenario
4. widened DAG scheduling coverage -> one multi-node fan-out or fan-in DAG scenario
5. first SCC-oriented coverage -> one bounded cycle-region or iterative-profile scenario

Already-covered replay classes carried into this sequence:
1. `R1`
2. `R2`
3. `R4`
4. `R5`
5. `R7`

Reserve replay classes for later waves:
1. any Stage 2 concurrency-sensitive replay classes beyond the current Stage 1 set

## Critical-Path Doctrine
The critical path for this widening wave is:
1. W004 widening for multi-node scheduling and first SCC handling,
2. W010 promotion from schema-only counters to emitted counter artifacts,
3. W009 widening for `R3`, `R6`, and `R8`,
4. W011 harness and corpus widening for multi-node and SCC cases,
5. W012 oracle and conformance widening over the broadened corpus,
6. final checked-in widened baseline run.

W001 and W005 remain carried conditions across the sequence.
W006 also remains active because the widened implementation must still bind back into assurance and pack evidence.

## Execution Sequence B

### Sequence 1. Multi-Node DAG Recalc Widening
Primary workset:
- `W004_INCREMENTAL_RECALC_AND_OVERLAY_BASELINE.md`

Entry gate:
- `W013` final gate has passed.
- the current Stage 1 single-node baseline is stable enough to widen rather than refactor.

Execution objective:
- realize deterministic multi-node invalidation propagation and topo-ordered scheduling for non-cyclic TreeCalc graphs.

Exit gate:
- the engine can process non-trivial multi-node DAG scenarios end-to-end,
- demand propagation, candidate-result production, verify-clean, reject, and publish consequences work across dependent node sets,
- topo order is deterministic and replay-visible,
- the widened implementation still preserves candidate-versus-publication and reject-is-no-publish semantics.

### Sequence 2. First SCC-Oriented Stage 1 Handling
Primary worksets:
- `W004_INCREMENTAL_RECALC_AND_OVERLAY_BASELINE.md`
- `W011_CORE_ENGINE_TEST_HARNESS_AND_SELF_CONTAINED_FIXTURE_PLAN.md`

Entry gate:
- Sequence 1 exit gate has passed.
- the corpus has at least one authored SCC-oriented or iterative-profile scenario.

Execution objective:
- realize the first bounded Stage 1 handling for cycle-region or iterative-profile scenarios without promoting full Stage 2 or SAC-style repair.

Exit gate:
- at least one SCC-oriented scenario executes through the real engine,
- the handling mode is deterministic and artifact-visible,
- the sequence clearly states what Stage 1 does and does not support for such cases.

### Sequence 3. Publication and Overlay Widening Over Multi-Node Sets
Primary worksets:
- `W003_STAGE1_COORDINATOR_AND_PUBLICATION_BASELINE.md`
- `W004_INCREMENTAL_RECALC_AND_OVERLAY_BASELINE.md`

Entry gate:
- Sequences 1 and 2 have passed.

Execution objective:
- ensure publication, reject, overlay retention, eviction eligibility, and verify-clean semantics remain sound when the workset spans multiple nodes and first SCC-oriented cases.

Exit gate:
- multi-node publication bundles are represented coherently,
- overlay protection and release semantics remain deterministic under widened graph cases,
- no hidden single-node assumptions remain in the widened Stage 1 artifact surface.

### Sequence 4. Runtime Counter Emission
Primary worksets:
- `W010_EXPERIMENT_REGISTER_AND_MEASUREMENT_SCHEMA_PLANNING.md`
- `W004_INCREMENTAL_RECALC_AND_OVERLAY_BASELINE.md`

Entry gate:
- Sequences 1 through 3 have passed.
- the widened implementation exposes stable instrumentation points.

Execution objective:
- promote the current Stage 1 counter families from schema-only artifacts into emitted runtime artifacts.

Exit gate:
- running code emits per-scenario counter snapshots,
- emitted counters include candidate/publication, pinned-reader/retention, invalidation/fallback, and overlay economics surfaces where the widened implementation supports them,
- counter names and meanings remain aligned to W010 and W009.

### Sequence 5. Replay Widening for `R3`, `R6`, and `R8`
Primary workset:
- `W009_REPLAY_AND_PACK_BINDING_FOR_STAGE1_SEAM_AND_COORDINATOR_BEHAVIOR.md`

Entry gate:
- Sequences 1 through 4 have passed.
- the widened harness can execute the required scenarios.

Execution objective:
- author and exercise the remaining priority Stage 1 replay classes.

Exit gate:
- `R3`, `R6`, and `R8` have authored scenario ids,
- those scenarios execute through the widened runner and oracle path,
- the emitted artifacts are pack-traceable to their declared replay classes.

### Sequence 6. Harness and Corpus Widening
Primary workset:
- `W011_CORE_ENGINE_TEST_HARNESS_AND_SELF_CONTAINED_FIXTURE_PLAN.md`

Entry gate:
- Sequences 1 through 5 have passed enough to stabilize the widened corpus needs.

Execution objective:
- extend the harness and corpus from the current hand-auditable floor into:
1. wider DAG scheduling cases,
2. first SCC-oriented cases,
3. typed reject taxonomy cases,
4. fallback and overlay re-entry cases,
5. first generated-but-traceable graph samples.

Exit gate:
- the widened corpus is executable and deterministic,
- hand-auditable versus generated scenario lanes are explicit,
- no required replay class for this wave lacks a scenario id.

### Sequence 7. Formal and Pack Hardening
Primary worksets:
- `W008_TLA_COORDINATOR_PUBLICATION_AND_FENCE_SAFETY_MODEL_PLAN.md`
- `W009_REPLAY_AND_PACK_BINDING_FOR_STAGE1_SEAM_AND_COORDINATOR_BEHAVIOR.md`
- `W010_EXPERIMENT_REGISTER_AND_MEASUREMENT_SCHEMA_PLANNING.md`

Entry gate:
- Sequences 1 through 6 have passed.

Execution objective:
- align the formal and pack surfaces to the widened implementation and emitted evidence.

Exit gate:
- the widened transition and replay surfaces are reflected in W008, W009, and W010 artifacts,
- the pack-evidence matrix points to current scenario ids and artifact roots,
- any unavailable formal tool execution is explicitly reported with fallback evidence.

### Sequence 8. Widened Baseline Run and Closure Evidence
Primary worksets:
- `W011_CORE_ENGINE_TEST_HARNESS_AND_SELF_CONTAINED_FIXTURE_PLAN.md`
- `W012_TRACECALC_REFERENCE_MACHINE_AND_CONFORMANCE_ORACLE.md`

Entry gate:
- Sequences 1 through 7 have passed.

Execution objective:
- emit one stable widened baseline run and use it as the closure evidence anchor for this packet.

Exit gate:
- `w014-stage1-widening-baseline` exists under the canonical artifact root,
- the widened baseline run exercises the required replay classes and widened graph cases,
- the engine-versus-oracle comparison is present for the widened corpus,
- the emitted counter artifacts are present in the checked-in baseline run.

## Parallel Side-Lane Rules
Execution Sequence B remains critical-path driven, but these side-lane rules apply:
1. the widened corpus scenarios should be authored as early as possible once the replay-class needs are stable,
2. counter emission may begin during late Sequence 2 if the instrumentation points stop moving,
3. W008, W009, and W010 should tighten in lockstep once the widened scenario set is stable,
4. W005 follow-on seam pressure should be filed immediately if widened reject or runtime-effect taxonomy exceeds the current accepted seam floor.

## Carried Conditions Outside The 1-8 Sequence
The following remain active across the sequence but are not numbered sub-phases:
1. W001 remains the canonical-spec and repo-integration maintenance lane.
2. W005 remains the seam-alignment and follow-on-handoff lane.
3. W006 remains the assurance-binding lane that ties widened implementation back into formal and pack artifacts.
4. if a widening step reveals a need to narrow Stage 1 support instead of widening it, that scope change must be documented explicitly rather than silently absorbed.

## Final Gate For Execution Sequence B
Execution Sequence B reaches its final gate only when all of the following hold:
1. Sequences 1 through 8 have each met their declared exit gate.
2. The Stage 1 engine slice is widened beyond the single-node floor into deterministic multi-node DAG handling and first SCC-oriented handling.
3. Running code emits counter artifacts aligned to the declared Stage 1 measurement families.
4. `R3`, `R6`, and `R8` are authored and exercised, while previously covered replay classes remain valid under the widened implementation.
5. The pack-evidence matrix references current scenario ids and emitted artifact roots rather than planning-only pack names.
6. A checked-in widened baseline run exists and is sufficient to judge the widened Stage 1 slice against its oracle.
7. No unresolved local seam ambiguity remains that would block the widened Stage 1 slice from being judged on replay, counters, and oracle artifacts.

This final gate is not the end of Stage 1 in full.
It is the end of the widening wave that makes the Stage 1 baseline materially broader, better instrumented, and better evidenced before concurrency promotion begins.

## Pre-Closure Verification Checklist
| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes |
| 2 | Pack expectations updated for affected packs? | yes |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes |
| 6 | All required tests pass? | yes |
| 7 | No known semantic gaps remain in declared scope? | yes |
| 8 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | yes |
| 9 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | yes |
| 10 | CURRENT_BLOCKERS.md updated (new/resolved)? | yes |

## Status
- execution_state: complete
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: integrated
- open_lanes: none
- claim_confidence: validated
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
