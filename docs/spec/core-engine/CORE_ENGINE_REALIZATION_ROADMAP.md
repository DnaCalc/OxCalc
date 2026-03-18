# CORE_ENGINE_REALIZATION_ROADMAP.md

## 1. Purpose and Status
This document defines the staged realization roadmap for the rewritten OxCalc core-engine spec set.

Status:
1. active rewrite baseline,
2. intended canonical execution roadmap companion,
3. TreeCalc-first in immediate target scope,
4. gate-driven rather than time-driven.

This document binds the architectural and assurance docs to:
1. the Stage 1 realization subset,
2. staged-later promotion lanes,
3. decisive experiments,
4. pack and evidence gates,
5. seam hardening state and follow-on pressure with OxFml.

## 2. Roadmap Purpose
The roadmap exists to answer three questions explicitly:
1. what is the first serious engine we are actually building,
2. what belongs in that first realization versus later staged promotion,
3. what evidence is required before a staged-later lane can be promoted.

## 3. Immediate Realization Target
The immediate realization target is the DNA TreeCalc engine baseline.

This means:
1. tree-based substrate,
2. no grid baseline,
3. no hidden grid assumptions in required Stage 1 semantics,
4. explicit OxFml seam discipline,
5. deterministic single-publisher coordinator,
6. strong incremental architecture direction from the start.

## 4. Stage Structure

### 4.1 Stage 1
Stage 1 is the first realized baseline.

Its role is to prove the architecture on TreeCalc-first scope.

### 4.2 Stage 2
Stage 2 introduces staged concurrent and async realization behind the same architectural contracts.

### 4.3 Stage 3+
Stage 3 and later phases promote more ambitious runtime strategy lanes and later substrate expansion under explicit gates.

## 5. Stage 1 Realization Subset
Stage 1 is intentionally conservative in realization, but not weak in architecture.

### 5.1 Stage 1 Must Realize
1. immutable structural snapshots,
2. stable identity and projection discipline,
3. pinned stable reader and observer views,
4. deterministic topo/SCC scheduling baseline,
5. explicit invalidation-state model,
6. explicit runtime overlay model at architecture-relevant minimum,
7. single-publisher coordinator authority,
8. accepted-commit atomic publication,
9. reject-is-no-publish behavior,
10. explicit coordinator-facing seam boundary with OxFml,
11. replay-oriented diagnostics for accept/reject and structural/recalc behavior,
12. initial assurance scaffolding for Lean, TLA+, and pack mapping.

### 5.2 Stage 1 May Realize Conservatively
1. verification-oriented incremental behavior through a conservative subset,
2. dynamic dependency handling through explicit runtime discipline plus bounded fallback where needed,
3. overlay retention and reuse through a safe initial policy before economics tuning,
4. cycle or iteration support through the declared profile subset.

### 5.3 Stage 1 Does Not Need To Realize Yet
1. default dynamic-topological maintenance,
2. full SAC-style repair,
3. advanced visibility-priority scheduling,
4. full staged concurrent or async throughput realization,
5. later grid-native substrate semantics.

## 6. Stage 1 Assurance and Evidence Minimum
Stage 1 should not be declared as architecturally sound without evidence across these axes:

### 6.1 Spec Axis
1. the new canonical docs are internally aligned for the Stage 1 subset,
2. Stage 1 subset wording is explicit and not implied.

### 6.2 Replay Axis
1. deterministic replay exists for declared Stage 1 behaviors,
2. accept/reject consequences are observable and diagnosable,
3. pinned-view and publication invariants can be exercised through artifacts.

### 6.3 Assurance Axis
1. initial Lean-facing model objects are defined,
2. initial TLA+ coordinator safety model is defined,
3. Stage 1 pack obligations are identified and exercised where possible.

### 6.4 Measurement Axis
1. baseline counters for reuse, fallback, and eviction behavior are planned,
2. baseline timing and signature measurement for recalc behavior is planned,
3. economics questions are captured as explicit experiments rather than guesswork.

## 7. Stage 2 Promotion Gate
Stage 2 is the first concurrency-bearing promotion.

It should require explicit closure of at least:
1. deterministic contention replay,
2. coordinator fence safety under concurrent or async work,
3. pinned-reader safety under overlapping work,
4. reject-detail adequacy for concurrency diagnosis,
5. stable publication behavior under staged concurrency.

Stage 2 is not allowed to reinterpret the publication or observer model.
It must preserve the Stage 1 contracts while extending realization strategy.

## 8. Staged-Later Optimization Lanes
The following lanes are explicitly retained as intended design space but are not baseline Stage 1 commitments.

### 8.1 Dynamic Topological Maintenance
Promotion requires:
1. correctness parity with the baseline,
2. deterministic replay compatibility,
3. demonstrated economics crossover against rebuild-based strategy.

### 8.2 SAC-Inspired Repair or More Ambitious Trace Repair
Promotion requires:
1. correctness evidence at least matching the baseline,
2. replay and diagnostics compatibility,
3. clear economics value for targeted workloads.

### 8.3 Later Scheduling Policies
Promotion requires:
1. semantic-equivalence evidence,
2. deterministic policy definition,
3. fairness or starvation evidence where relevant.

### 8.4 Later Grid Expansion
Promotion requires:
1. explicit substrate-extension spec work,
2. semantic-gap closure against TreeCalc baseline assumptions,
3. preservation of immutable-structure and publication discipline.

## 9. Decisive Experiments
The roadmap requires explicit decisive experiments rather than optional benchmarks.

### 9.1 Early-Cutoff Experiment
Purpose:
1. determine correctness and practical value of verification or early-cutoff behavior for the intended workloads.

### 9.2 Dynamic-Topo Versus Rebuild Experiment
Purpose:
1. determine the crossover point, if any, where dynamic-topological maintenance is justified.

### 9.3 Dynamic-Dependency Economics Experiment
Purpose:
1. determine when explicit runtime-tracked dynamic dependency behavior outperforms conservative fallback.

### 9.4 Overlay Economics Experiment
Purpose:
1. measure reuse, miss, fallback, and eviction behavior,
2. calibrate retention policy and future thresholds.

### 9.5 Staged Concurrency Replay Experiment
Purpose:
1. validate that concurrent or async realization preserves deterministic replay and publication safety.

## 10. Required Counter and Measurement Direction
The roadmap expects early instrumentation for at least:
1. overlay reuse and miss rates,
2. fallback rates,
3. eviction counts,
4. recalc work-volume signatures,
5. staged concurrency replay signatures once Stage 2 begins.

The exact counter schema is refined elsewhere, but the roadmap locks the requirement that such counters are part of the realization plan from early on.

## 11. OxFml Handoff State In The Roadmap
Seam hardening remains an active lane, but it is no longer only a local request set.

Current state:
1. `HANDOFF-CALC-001` has been filed by OxCalc,
2. OxFml has acknowledged and adapted the shared seam in its canonical docs,
3. OxCalc must now align local realization and assurance planning to the accepted candidate-result versus publication split,
4. OxFml has now also sent `HANDOFF-FML-001` and a stronger downstream observation note, both of which are local intake inputs for the next OxCalc round.

Remaining follow-on pressure includes:
1. replay artifact binding for the candidate-result versus publication boundary,
2. pack and trace binding for typed fence and capability rejects,
3. surfaced execution-restriction and capability-sensitive effect intake where coordinator replay or publication interpretation depends on them,
4. direct-binding-sensitive retained-witness and host-boundary handling where replay truth depends on concrete cell resolution,
5. narrower follow-on handoff only if exercised W019 evidence needs stronger shared obligations.

## 12. Workset Execution Sequence
The roadmap now binds to the following execution sequence.
This is dependency-ordered, not date-ordered.

1. `W001`: core-engine spec rewrite and canonicalization.
2. `W002`: TreeCalc structural state and snapshot kernel.
3. `W003`: Stage 1 coordinator and publication baseline.
4. `W004`: incremental recalc and overlay baseline.
5. `W005`: OxFml seam hardening and handoff tracking.
6. `W006`: core formalization and gate binding.
7. `W007`: Lean-facing state objects and transition boundary plan.
8. `W008`: TLA+ coordinator, publication, and fence-safety model plan.
9. `W009`: replay and pack binding for Stage 1 seam and coordinator behavior.
10. `W010`: experiment register and measurement-schema planning for Stage 1 and Stage 2 promotion lanes.
11. `W011`: core-engine test harness and self-contained fixture plan.
12. `W012`: TraceCalc reference machine and conformance oracle.
13. `W013`: Execution Sequence A for the first TreeCalc-first Stage 1 implementation wave.
14. `W014`: Execution Sequence B for Stage 1 widening and evidence hardening.
15. `W015`: Replay appliance adapter and bundle-validator rollout.
16. `W017`: Execution Sequence C for Rust-first reimplementation of the current realized TreeCalc and TraceCalc scope.
17. `W016`: witness distillation and retained failure packs.
18. `W018`: replay-appliance bundle and capability promotion.
19. `W020`: OxFml downstream integration round 01.
20. `W019`: replay distill and pack promotion.

The purpose of `W007` through `W012` is to turn `W006` from a generic formalization placeholder into executable assurance, harness, and conformance-oracle packets.
`W013` is the operational coordination packet that turns that dependency graph into the first concrete implementation sequence.
`W014` is the follow-on operational packet that widens the Stage 1 baseline, hardens emitted evidence, and closes the largest remaining pre-concurrency gaps.
`W015` is the replay-facing integration packet that adds local replay coherence, adapter doctrine, capability-floor governance, and normalized-bundle expectations without weakening OxCalc-owned semantics.
`W017` is the implementation-direction shift packet and Execution Sequence C: it re-establishes the current realized scope as a Rust-first ab initio implementation, using carried historical baseline artifacts as comparison and parity evidence.
`W016` is the first retained-witness and retained-failure baseline lane over the active Rust runner.
`W018` is the next replay-facing realization lane: it turns current replay-aware local artifacts into emitted replay-appliance bundle artifacts and capability evidence.
`W020` is the first post-W018 downstream OxFml integration lane: it records inbound schema and replay observations, aligns local ownership, and decides whether any narrower follow-on handoff is required.
`W019` is the explicit replay successor lane after W020 for `cap.C4.distill_valid`, `cap.C5.pack_valid`, and broader replay-appliance widening.

## 13. Relationship To Assurance
No staged promotion is complete without coupling to the assurance surfaces.

This means every promotion lane must name:
1. proof or model-check expectations where applicable,
2. replay artifact expectations,
3. pack obligations,
4. performance or economics evidence requirements.

## 14. Explicit Non-Promotion Rule
A lane is not promoted merely because:
1. an implementation exists,
2. it appears faster locally,
3. it matches intuition,
4. it was proposed in a research note.

Promotion requires the evidence class named for that lane.

## 15. Open Detailed Questions
These remain roadmap-level follow-on questions within the now-locked staged structure:
1. exact Stage 2 concurrency sub-phases,
2. exact thresholds for economics-based promotion,
3. exact replay corpus shape for multi-stage coordinator traces,
4. exact first realized pack set for W009 execution,
5. exact replay-pack binding layout and generated-scenario retention policy after the now-closed first artifact-root and diff-policy decisions.

## 16. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - W013 has reached its final gate for the first TreeCalc Stage 1 implementation wave
  - W014 has now reached its final gate for the widening wave: the Stage 1 slice executes deterministic multi-node DAG handling, first SCC-oriented handling, widened replay classes, emitted per-scenario counters, and the checked-in `w014-stage1-widening-baseline` run
  - replay artifacts now include the widened 12-scenario corpus and checked-in baseline runs for both `w013-sequence-a-baseline` and `w014-stage1-widening-baseline`
  - W015 now covers the local replay-coherence refactor and adapter-rollout doctrine, but normalized replay-appliance bundle emission remains a later roadmap lane
  - W017 has reached its final gate for the current Rust-first realization scope
  - W016 has reached its declared retained-witness baseline gate
  - W018 has now reached its declared gate: emitted replay-appliance bundle roots, validator artifacts, explain records, and checked-in replay-appliance-aware baselines exist for the current realized scope
  - W020 is now the immediate OxFml integration continuation: inbound schema, replay, retained-local, and host-boundary observations must be consumed locally without reopening W005
  - W019 is the next replay-facing continuation after that intake round: `cap.C4.distill_valid`, `cap.C5.pack_valid`, and broader replay-appliance widening remain later execution work
  - Stage 2 concurrency realization and broader replay-pack export remain later roadmap lanes
