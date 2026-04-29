# W009: Replay and Pack Binding for Stage 1 Seam and Coordinator Behavior

## Purpose
Bind the accepted seam direction and Stage 1 coordinator architecture to explicit replay artifact classes, first-pack expectations, and the newly named Stage 1 coordinator and recalc transitions.

## Position and Dependencies
- **Depends on**: W006, W007, W008
- **Blocks**: none
- **Cross-repo**: must remain aligned with OxFml replay and seam vocabulary

## Scope
### In scope
1. Candidate-result versus publication replay cases.
2. Typed reject replay cases for fence and capability mismatch.
3. First-pack binding for Stage 1 seam and coordinator obligations.
4. Trace-schema pressure that may require narrower follow-on handoff.
5. Explicit replay coverage for the named Stage 1 recalc and coordinator transitions.

### Out of scope
1. Implementing the replay harness.
2. Running the packs.
3. Full Stage 2 concurrency replay coverage.

## Deliverables
1. A replay and pack-binding planning packet for first assurance artifact creation.
2. A first transition-coverage matrix tying W003/W004 transitions to replay classes and trace fields.

## Gate Model
### Entry gate
- W007 and W008 have established the formal model boundary and coordinator safety scope.

### Exit gate
- Stage 1 replay artifact classes and first-pack bindings are explicit enough to drive artifact authoring.
- Replay classes are aligned to W008 actions `A1` through `A10` where relevant and safety classes `S1` through `S6`.
- The named W003 and W004 Stage 1 transitions are covered by at least one replay class or explicit reserve lane.

## Stage 1 Replay Artifact Classes
The first replay corpus should be organized by behavior class rather than by ad hoc scenario naming.

### R1. Candidate-Result Versus Publication Separation
Purpose:
1. show that evaluator success and accepted publication are distinct events
2. preserve replay visibility between `A5` and `A7`

Minimum trace shape:
1. candidate work admission
2. accepted candidate-result recording
3. no publication yet visible
4. later accept-and-publish step

Primary W008 anchors:
1. `A5`
2. `A7`
3. `S3`

### R2. Reject-Is-No-Publish
Purpose:
1. show that rejection emits typed no-publish detail
2. prove publication state is unchanged after rejection

Minimum trace shape:
1. candidate work admission
2. reject decision with typed detail
3. unchanged published view

Primary W008 anchors:
1. `A6`
2. `S2`

### R3. Fence-Compatibility Accept And Reject Split
Purpose:
1. show that compatible work can publish
2. show that incompatible work routes to typed rejection rather than partial publication

Minimum trace shape:
1. one compatible case reaching `A7`
2. one incompatible case reaching `A6`
3. explicit difference in compatibility basis

Primary W008 anchors:
1. `A6`
2. `A7`
3. `S4`

### R4. Pinned-Reader Stability During Later Work
Purpose:
1. show that a pinned reader continues to see a stable snapshot-compatible view while later work proceeds

Minimum trace shape:
1. pin reader on a stable published view
2. admit and process later work
3. preserve earlier pinned view
4. optionally publish later state for non-pinned readers

Primary W008 anchors:
1. `A8`
2. `A7`
3. `S5`

### R5. Overlay Retention And Release Safety
Purpose:
1. show that protected overlay state is not evicted before pin release
2. show that release can make eviction eligible later

Minimum trace shape:
1. overlay state protected by pinned reader or stable view
2. attempted later work or cleanup does not evict prematurely
3. unpin or release
4. later eligibility change

Primary W008 anchors:
1. `A8`
2. `A9`
3. `A10`
4. `S6`

### R6. Typed Reject Taxonomy Coverage
Purpose:
1. cover the first typed reject families that Stage 1 coordinator behavior depends on

Initial reject families:
1. snapshot or fence mismatch
2. token or artifact mismatch
3. capability mismatch
4. other coordinator-declared no-publish class if introduced locally

Primary W008 anchors:
1. `A6`
2. `rejectLog`

### R7. Verification-Clean Without Publication
Purpose:
1. show that demanded work can resolve through verification without emitting a publication bundle
2. preserve replay visibility for `I4 VerifyClean` and `A3b VerifyClean`

Minimum trace shape:
1. node marked dirty and needed
2. evaluation begins
3. verification-clean resolution occurs
4. published view remains unchanged
5. demanded frontier can release without synthetic publication

Primary W008 anchors:
1. `A2`
2. `A3`
3. `A3b`

### R8. Fallback And Overlay Re-entry
Purpose:
1. show that incompatible overlay basis or insufficient runtime-derived effect detail routes to explicit fallback
2. prove the fallback path is no-publish and replay-visible

Minimum trace shape:
1. node reaches evaluation on a compatible structural basis
2. dynamic-dependency or overlay condition fails the optimized path
3. typed reject or fallback label is emitted
4. node re-enters the stale or needed frontier under conservative policy
5. overlay reuse is withheld until compatible basis is restored

Primary W008 anchors:
1. `A3`
2. `A5`
3. `A6`
4. `A10`

## Stage 1 Transition Coverage Matrix
The first replay corpus should make the Stage 1 transition coverage explicit.

| Stage 1 transition | Minimum replay class | Minimum trace labels |
|---|---|---|
| `I1 MarkDirty` | `R7` or `R8` depending on path | `node_marked_dirty` |
| `I2 MarkNeeded` | `R7`, `R8`, or any publish-path class | `node_marked_needed` |
| `I3 BeginEvaluate` | `R1`, `R2`, `R3`, `R7`, `R8` | `evaluation_started` |
| `I4 VerifyClean` | `R7` | `node_verified_clean` |
| `I5 ProduceDependencyShapeUpdate` | `R1`, `R3`, `R8` | `candidate_shape_update_produced` |
| `I6 RejectOrFallback` | `R2`, `R6`, `R8` | `candidate_rejected` or `fallback_forced` |
| `I7 PublishAndClear` | `R1`, `R3` | `publication_committed`, `node_cleared` |
| `I8 ReleaseAndEvictEligible` | `R5`, `R8` | `eviction_eligibility_opened` |
| `C1 AdmitCandidateWork` | `R1`, `R2`, `R3` | `candidate_admitted` |
| `C2 RecordAcceptedCandidateResult` | `R1`, `R3` | `candidate_recorded` |
| `C3 RejectCandidateWork` | `R2`, `R6`, `R8` | `candidate_rejected` |
| `C4 AcceptAndPublish` | `R1`, `R3` | `publication_committed` |
| `C5 PinReader` | `R4`, `R5` | `reader_pinned` |
| `C6 UnpinAndReleaseProtection` | `R5` | `reader_unpinned`, `eviction_eligibility_opened` |

## First Pack Binding Matrix
The first pack set should be anchored directly to replay classes and safety properties.

### P1. `PACK.fec.commit_atomicity`
Should bind to:
1. replay class `R1`
2. replay class `R3` compatible branch
3. safety property `S1`

### P2. `PACK.fec.reject_detail_replay`
Should bind to:
1. replay class `R2`
2. replay class `R6`
3. safety property `S2`

### P3. `PACK.concurrent.epochs`
Stage 1 subset should bind to:
1. replay class `R4`
2. replay class `R5`
3. safety properties `S5` and `S6`

### P4. `PACK.fec.overlay_lifecycle`
Stage 1 subset should bind to:
1. replay class `R5`
2. replay class `R8`
3. any overlay retention or release counters referenced later by W010

### P5. `PACK.dag.dynamic_dependency_bind_semantics`
Stage 1 subset should bind first to:
1. replay class `R8`
2. the dynamic-dependency branches of `R3`
3. the relevant fallback and reuse labels from the transition coverage matrix

Current status:
1. the Stage 1 effect and dependency-shape subset is now explicit enough to name the first replay classes
2. broader pack breadth still remains reserved until artifacts exist

## Current Evidence Traceability Matrix
This matrix should be extended as new replay artifacts and emitted runs are promoted.

| Pack | Replay classes | Current scenario ids | Current artifact root |
|---|---|---|---|
| `PACK.fec.commit_atomicity` | `R1`, `R3` compatible branch | `tc_accept_publish_001`, `tc_multinode_dag_publish_001` | `docs/test-runs/core-engine/tracecalc-reference-machine/w014-stage1-widening-baseline/` |
| `PACK.fec.reject_detail_replay` | `R2`, `R3` incompatible branch, `R6` | `tc_reject_no_publish_001`, `tc_publication_fence_reject_001`, `tc_artifact_token_reject_001` | `docs/test-runs/core-engine/tracecalc-reference-machine/w014-stage1-widening-baseline/` |
| `PACK.concurrent.epochs` | `R4`, `R5` | `tc_pinned_view_stability_001`, `tc_overlay_retention_001` | `docs/test-runs/core-engine/tracecalc-reference-machine/w014-stage1-widening-baseline/` |
| `PACK.fec.overlay_lifecycle` | `R5`, `R8` | `tc_overlay_retention_001`, `tc_fallback_reentry_001` | `docs/test-runs/core-engine/tracecalc-reference-machine/w014-stage1-widening-baseline/` |
| `PACK.dag.dynamic_dependency_bind_semantics` | `R8` plus dynamic-shape publish path | `tc_dynamic_dep_switch_001`, `tc_fallback_reentry_001` | `docs/test-runs/core-engine/tracecalc-reference-machine/w014-stage1-widening-baseline/` |
| `PACK.stage1.scc_bounded_handling` | local W014 SCC lane | `tc_cycle_region_reject_001` | `docs/test-runs/core-engine/tracecalc-reference-machine/w014-stage1-widening-baseline/` |

## W031 TreeCalc Replay And Pack Guardrail Refresh

The W030 TreeCalc local baseline refreshes the replay/pack guardrails for the declared local sequential TreeCalc corpus without replacing the older `TraceCalc` reference-machine lane or promoting concurrency/economics claims.

Authority anchors:
1. authority inventory: `docs/spec/core-engine/CORE_ENGINE_TREECALC_ASSURANCE_AUTHORITY_MAP.md`
2. checked-in TreeCalc baseline: `docs/test-runs/core-engine/treecalc-local/w030-treecalc-oracle-baseline/`
3. replay inventory: `docs/test-runs/core-engine/treecalc-local/w030-treecalc-oracle-baseline/replay_artifact_manifest.json`
4. conformance summary: `docs/test-runs/core-engine/treecalc-local/w030-treecalc-oracle-baseline/conformance/conformance_summary.json`

Guardrail rule:
1. W030 TreeCalc artifacts are additive authority for the `OxCalcTree` local sequential floor.
2. They do not silently supersede the `TraceCalc` reference-machine artifacts for older Stage 1 proof obligations.
3. They do not widen `PACK.concurrent.epochs`, pinned-reader retention, overlay economics, or Stage 2 promotion evidence.
4. W029 invariants remain preserved: candidate acceptance, reject/no-publish, and coordinator publication authority are unchanged by replay/pack guardrail refresh.

| Pack or replay class | W031 TreeCalc refresh | Current guardrail |
|---|---|---|
| `R1` candidate-result versus publication / `PACK.fec.commit_atomicity` | W030 published TreeCalc cases carry result, trace, explain, candidate/publication fields, and published values under the `OxCalcTree` contract. | Refreshed for local sequential TreeCalc corpus only; multi-node atomic bundle stress remains successor assurance work. |
| `R2` reject-is-no-publish / `PACK.fec.reject_detail_replay` | W030 rejected TreeCalc cases pass conformance with no expectation mismatches and preserve no-publish behavior. | Broader provider-failure, callable-publication, and future OxFml reject taxonomy breadth remain residual/watch lanes. |
| `R3` fence-compatible accept/reject split | Compatibility and artifact-token basis are preserved through `OxCalcTreeRecalcRequest` and result diagnostics. | Explicit incompatible-fence TreeCalc corpus breadth remains future if promotion pressure requires it. |
| `R4` pinned-reader stability / `PACK.concurrent.epochs` | No W030 widening. | Stage 2 or retention-policy promotion remains blocked on explicit TreeCalc pinned-reader/epoch replay. |
| `R5` overlay retention and release / `PACK.fec.overlay_lifecycle` | Runtime-effect overlay sidecars and replay manifest make covered overlay projections visible. | Retention/release safety and economics counters remain future W031 residual packetization candidates. |
| `R6` typed reject taxonomy | Current local TreeCalc reject/runtime-effect families are covered by W030 rejected cases. | Taxonomy is not globally closed; handoff only if a concrete seam insufficiency appears. |
| `R7` verified-clean without publication | W030 includes a verified-clean case with zero expectation mismatch. | Equality breadth for richer future value/format surfaces remains future widening. |
| `R8` fallback and overlay re-entry / `PACK.dag.dynamic_dependency_bind_semantics` | Dependency graph, invalidation closure, runtime effects, and overlays are emitted for covered dynamic/runtime-derived cases. | Fallback economics and conservative re-entry measurement remain W010-linked residuals. |

## First Trace Schema Pressure List
W009 should make explicit which trace fields are now required by the accepted seam and W008 model shape.

### Required Event Distinctions
1. candidate work admission event
2. accepted candidate-result event
3. reject event with typed reject detail
4. accept-and-publish event
5. pin-reader event
6. unpin or release-protection event
7. verification-clean event
8. fallback-forced event
9. eviction-eligibility-opened event

### Required Trace Fields
1. structural snapshot identity
2. compatibility basis or fence-class summary
3. candidate work identity
4. accepted candidate-result identity or reference
5. published view identity or commit-bundle reference
6. typed reject detail where rejection occurs
7. pinned-reader identity where reader protection matters
8. overlay protection or release markers where retention safety matters
9. node invalidation state before and after the transition where W004 transitions are under test
10. transition label drawn from the Stage 1 transition coverage matrix
11. overlay key or compatibility-basis fragment where fallback or eviction eligibility is being asserted

## Cross-Repo Seam Pressure Assessment
At the planning level, W009 should assume the current OxFml seam is sufficient for:
1. accepted candidate-result versus publication separation
2. typed no-publish rejection at the general-rule level

Potential narrower follow-on handoff pressure exists only if:
1. replay traces need a more explicit shared reject taxonomy than currently stated
2. runtime-derived effect families required for W004 exceed the current shared canonical surface

## Immediate Artifact Authoring Order
The first actual replay or pack authoring should proceed in this order:
1. `R1` candidate-result versus publication separation
2. `R2` reject-is-no-publish
3. `R7` verification-clean without publication
4. `R4` pinned-reader stability
5. `R5` overlay retention or release safety
6. `R3` fence-compatibility split
7. `R8` fallback and overlay re-entry
8. `R6` broader reject taxonomy coverage

This order prioritizes the coordinator and publication contract before broader replay breadth.

## Pre-Closure Verification Checklist
| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes |
| 2 | Pack expectations updated for affected packs? | no |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | no |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes |
| 6 | All required tests pass? | no |
| 7 | No known semantic gaps remain in declared scope? | no |
| 8 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | yes |
| 9 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | yes |
| 10 | CURRENT_BLOCKERS.md updated (new/resolved)? | no |

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - `R1` through `R8` are now exercised by the widened checked-in corpus and the emitted `w014-stage1-widening-baseline` run
  - dedicated replay-appliance bundle projection, validator output, and explain artifacts now exist in the W018 replay-appliance-aware baselines, but pack-grade promotion remains later than the current OxCalc-native artifact surface
  - trace-schema ownership split with OxFml is still only partially explicit
  - W010 still needs promotion-threshold reporting and replay-linked summaries over the now-emitted counters
- claim_confidence: provisional
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
