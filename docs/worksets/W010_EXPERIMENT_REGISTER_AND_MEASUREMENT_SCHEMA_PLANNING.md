# W010: Experiment Register and Measurement-Schema Planning

## Purpose
Convert the roadmap's decisive experiments and early-counter requirements into explicit planning packets for later implementation and assurance work.

## Position and Dependencies
- **Depends on**: W006, W009
- **Blocks**: none
- **Cross-repo**: none required initially

## Scope
### In scope
1. Early-cutoff experiment planning.
2. Dynamic-topo versus rebuild experiment planning.
3. Dynamic-dependency and overlay economics measurement planning.
4. Counter-schema planning for Stage 1 and Stage 2 promotion evidence.

### Out of scope
1. Running the experiments.
2. Final promotion-threshold decisions.
3. Production instrumentation implementation.

## Deliverables
1. An experiment-register and measurement-schema planning packet aligned to the realization roadmap.

## Gate Model
### Entry gate
- W006 has bound the roadmap to the assurance-planning sequence.
- W009 has defined the first replay classes and pack hooks that the counters must support.

### Exit gate
- The decisive experiments and counter schemas are explicit enough to inform later implementation and assurance work.
- Measurement planning is explicitly connected to replay classes, pack hooks, and later promotion gates.

## Counter Schema Families
The first measurement schema should group counters by decision type rather than by implementation module.

### C1. Candidate And Publication Counters
Purpose:
1. support replay-backed understanding of candidate-result versus publication behavior
2. detect silent drift between evaluator success and committed publication

Initial counters:
1. candidate admissions
2. accepted candidate results recorded
3. accepted publications committed
4. rejects by typed class
5. candidate results abandoned before publication

Primary workset or replay anchors:
1. W008 actions `A5`, `A6`, and `A7`
2. W009 replay classes `R1`, `R2`, and `R3`

### C2. Pinned-Reader And Retention Counters
Purpose:
1. measure reader protection pressure and retention consequences
2. support pinned-view and overlay-retention replay classes

Initial counters:
1. pinned-reader count
2. average pin lifetime bucketed by scenario
3. release events
4. retention-blocked cleanup attempts
5. post-release eviction-eligibility transitions

Primary workset or replay anchors:
1. W008 actions `A8` and `A9`
2. W009 replay classes `R4` and `R5`

### C3. Invalidation And Fallback Counters
Purpose:
1. measure how often Stage 1 relies on fallback rather than stronger incremental discrimination
2. support later economics-driven promotion decisions

Initial counters:
1. nodes marked stale
2. nodes marked necessary
3. nodes verified clean without further propagation
4. fallback-trigger count by reason
5. fallback-affected work volume

Primary workset or replay anchors:
1. W004 invalidation-state packet
2. roadmap early-cutoff and dynamic-dependency experiments

### C4. Overlay Economics Counters
Purpose:
1. measure whether explicit overlay retention is paying for itself
2. inform eviction and retention policy tuning

Initial counters:
1. overlay lookups
2. overlay hits
3. overlay misses
4. overlay creations
5. overlay evictions
6. overlay reuse after protected retention

Primary workset or replay anchors:
1. W004 overlay packet
2. W009 replay class `R5`

### C5. Stage 2 Promotion Counters
Purpose:
1. reserve the counter families that will later justify Stage 2 promotion
2. keep early instrumentation compatible with later concurrency evidence

Initial reserved families:
1. contention count by class
2. retry count by class
3. overlapping in-flight work count
4. publish-order replay signature count

These remain reserved in Stage 1 and should not be over-claimed yet.

## Decisive Experiment Register
The first experiment register should keep one row per promotion question.

### E1. Early-Cutoff Benefit Experiment
Question:
1. does verification or early-cutoff materially reduce propagation without semantic drift for intended TreeCalc workloads?

Inputs:
1. `C3` counters
2. replay classes that distinguish verification from fallback where later available

Required outputs:
1. hit rate
2. avoided work estimate
3. mismatch or drift count if any

### E2. Dynamic-Topo Versus Rebuild Experiment
Question:
1. is dynamic-topological maintenance justified relative to deterministic rebuild for the expected workload shapes?

Inputs:
1. work-volume signatures
2. structural change counts
3. replay-stable scenario families

Required outputs:
1. crossover region estimate
2. correctness parity status
3. replay compatibility status

### E3. Dynamic-Dependency Economics Experiment
Question:
1. when does explicit runtime-tracked dynamic dependency handling outperform conservative fallback?

Inputs:
1. `C3` counters
2. `C4` counters
3. W009 replay class `R5` and later W004-derived cases

Required outputs:
1. fallback rate by scenario
2. overlay reuse benefit estimate
3. observed high-cost cases requiring narrower seam pressure if any

### E4. Overlay Retention Experiment
Question:
1. what retention window or policy shape gives useful reuse without causing protection-heavy buildup?

Inputs:
1. `C2` counters
2. `C4` counters
3. replay classes `R4` and `R5`

Required outputs:
1. protected-retention hit rate
2. blocked-cleanup count
3. post-release cleanup effectiveness

### E5. Stage 2 Replay Readiness Experiment
Question:
1. is the Stage 1 model instrumented enough to detect concurrency-sensitive publication or retention drift before Stage 2 promotion?

Inputs:
1. reserved `C5` families
2. W008 safety properties `S1` through `S6`
3. W009 replay class coverage map

Required outputs:
1. instrumentation coverage gap list
2. missing counter families
3. missing replay classes for Stage 2 entry

## Counter-to-Pack Binding
The measurement schema should also say which counters inform which packs.

### P1. `PACK.fec.commit_atomicity`
Inform with:
1. `C1` candidate and publication counters
2. replay class `R1`

### P2. `PACK.fec.reject_detail_replay`
Inform with:
1. `C1` reject-by-class counters
2. replay classes `R2` and `R6`

### P3. `PACK.concurrent.epochs`
Inform with:
1. `C2` pinned-reader and retention counters
2. reserved `C5` Stage 2 counters later

### P4. `PACK.fec.overlay_lifecycle`
Inform with:
1. `C2` retention counters
2. `C4` overlay economics counters

## Measurement Artifact Shape
The first planning packet should assume these artifact forms later:
1. per-scenario counter snapshots
2. replay-linked counter summaries
3. promotion-question summary tables
4. regression-threshold notes where later justified

The planning rule is simple: counters must be explainable by replay class and promotion question.

## Immediate Authoring Order After This Packet
1. define minimal `C1` and `C2` counter names with stable semantics
2. define per-replay-class measurement expectations for `R1` through `R5`
3. define experiment record schema for `E1` through `E5`
4. reserve `C5` without implementing Stage 2 counters prematurely

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
  - experiment register and counter families now exist, but no instrumentation artifacts have been authored yet
  - W004 still needs to provide the tighter dynamic-dependency and overlay packet that several counters depend on
  - Stage 2 reserved counters are intentionally not yet realized
  - no promotion-threshold evidence rules are closed yet
- claim_confidence: provisional
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
