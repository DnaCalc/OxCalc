# CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md

## 1. Purpose and Status
This document defines the OxCalc recalc and incremental-execution model for the rewritten
core-engine spec set.

Status:
1. active rewrite baseline,
2. intended canonical recalc companion to the architecture and state-kernel docs,
3. TreeCalc-first in immediate execution scope,
4. staged for later concurrency and substrate expansion.

This document defines:
1. invalidation-state semantics,
2. deterministic recalc baseline,
3. incremental direction and optimization policy,
4. dynamic dependency handling direction,
5. staged-later optimization lanes and deciding experiments.

## 2. Recalc Mission
The recalc engine must satisfy five simultaneous goals:
1. semantic determinism,
2. strong replayability,
3. explicit invalidation discipline,
4. scalable incremental execution direction,
5. compatibility with staged concurrent and async realization.

The architecture is conservative in substrate scope, but not conservative in quality ambition.
The recalc model should therefore be explicit about the intended high-quality incremental engine,
not only the minimum sequential baseline.

## 3. Baseline Semantic Rule
Semantic truth is independent of runtime strategy.

This implies:
1. the stabilized result is the semantic target,
2. scheduler shape is not semantic truth,
3. optimization lanes may change how work is reached, not what correct stabilized output means,
4. any strategy change must preserve deterministic replay and semantic equivalence for the affected profile.

## 4. Recalc Pipeline
The recalc pipeline remains conceptually staged.

At the architecture level it consists of:
1. structural-change intake,
2. dependency/invalidation impact derivation,
3. scheduling,
4. evaluation,
5. commit publication or reject,
6. observer-visible stabilization updates.

The detailed seam between OxCalc and OxFml is defined elsewhere, but this document fixes the
engine's responsibility for invalidation, scheduling, and stabilization logic.

## 5. Invalidation-State Model

### 5.1 Principle
Invalidation state is a first-class engine model, not a vague scheduler convenience.

The engine must explicitly model whether a node is:
1. clean,
2. stale or dirty,
3. necessary or demanded for the current stabilization objective,
4. under evaluation,
5. ready for publication,
6. blocked by cycle or reject conditions as defined by companion documents.

Exact names may evolve, but the model must preserve these distinctions.

### 5.2 Why It Is First-Class
This is required because:
1. most incremental correctness failures live in invalidation logic,
2. replay and diagnostics need explicit state transitions,
3. verification-oriented recalculation cannot be expressed cleanly without explicit state,
4. later concurrency requires strong state-machine discipline.

### 5.3 Minimum State Requirements
The recalc model must support:
1. marking stale/affected work after structural or upstream change,
2. distinguishing stale from actually necessary work,
3. distinguishing stale from verified-clean work,
4. deterministic transition ordering,
5. replay-visible transition evidence where required by packs.

### 5.4 Stage 1 Local State Vocabulary
For Stage 1, the first local invalidation-state vocabulary should be:
1. `clean`
2. `dirty_pending`
3. `needed`
4. `evaluating`
5. `verified_clean`
6. `publish_ready`
7. `rejected_pending_repair`
8. `cycle_blocked`

These names are the first OxCalc-local floor for implementation-facing work.
They may later map into richer formal names, but the distinctions are now fixed for Stage 1 planning.

### 5.5 Stage 1 Transition Packet
The first Stage 1 transition packet should include at least:
1. `R1 MarkDirty`: `clean | verified_clean -> dirty_pending`
2. `R2 MarkNeeded`: `dirty_pending -> needed`
3. `R3 BeginEvaluate`: `needed -> evaluating`
4. `R4 VerifyClean`: `evaluating -> verified_clean`
5. `R5 ProduceCandidateResult`: `evaluating -> publish_ready`
6. `R6 RejectAndFallback`: `evaluating | publish_ready -> rejected_pending_repair`
7. `R7 PublishAcceptedResult`: `publish_ready -> clean`
8. `R8 HoldCycleBoundary`: `needed | evaluating -> cycle_blocked` where the active profile forbids immediate publish
9. `R9 ReleaseNeed`: `verified_clean | clean -> clean` with demanded-frontier pressure removed

The minimum transition intent is:
1. `MarkDirty` records possible semantic impact without yet forcing evaluation,
2. `MarkNeeded` marks work required for the current stabilization objective,
3. `BeginEvaluate` begins deterministic evaluator work on the current snapshot and fence basis,
4. `VerifyClean` records an early-cutoff or verification success without producing a new publication candidate,
5. `ProduceCandidateResult` hands control to the coordinator through the local `AcceptedCandidateResult` floor,
6. `RejectAndFallback` records no-publish failure and any required conservative re-entry into the stale frontier,
7. `PublishAcceptedResult` clears the node back to `clean` only through accepted publication,
8. `HoldCycleBoundary` makes cycle-region staging explicit instead of treating it as scheduler folklore,
9. `ReleaseNeed` keeps demanded-frontier pressure explicit and replay-visible.

These transitions are the Stage 1 semantic bridge into W008 action naming and W009 replay classes.

## 6. Deterministic Topo/SCC Baseline

### 6.1 Baseline Scheduling Skeleton
The baseline scheduling skeleton is deterministic topo/SCC execution.

This means:
1. acyclic regions are evaluated in deterministic topological order,
2. cycle regions are isolated explicitly,
3. cycle-region behavior is profile-governed,
4. tie-breaking must be deterministic.

### 6.2 Why This Remains Baseline
Topo/SCC remains the baseline because it offers:
1. strong determinism,
2. straightforward replay semantics,
3. an explicit cycle boundary,
4. a clear proving substrate for TreeCalc-first realization,
5. compatibility with later incremental and concurrent refinements.

### 6.3 Baseline Does Not Mean Naive
The baseline is not permission to keep the engine conceptually shallow.

Topo/SCC is the deterministic structural skeleton.
It does not exclude stronger invalidation-state discipline, verification, early cutoff,
or explicit runtime-observed dependencies.

## 7. Verification-Oriented Incremental Direction

### 7.1 Architectural Direction
The intended engine direction is verification-oriented incremental recalculation.

This means the engine should aim to distinguish:
1. work that is stale because it may be affected,
2. work that must actually be re-evaluated,
3. work that can be verified clean without propagating full recomputation.

### 7.2 Early-Cutoff Direction
Early cutoff is promoted into the rewritten design as intended architecture.

At a high level:
1. if the relevant input facts for a node do not imply a changed observable result,
2. that node may be verified clean,
3. downstream propagation may be reduced accordingly,
4. provided replay and semantic-equivalence obligations still hold.

The exact realization mechanism may vary by stage, but the design intent is explicit.

### 7.3 Stage-1 Interpretation
Stage 1 realization may still use conservative mechanisms where required.

But the Stage 1 spec should not erase the intended verification-oriented model.
Instead:
1. the model is the target architecture,
2. Stage 1 may realize a conservative subset,
3. required instrumentation must begin early so later promotion is evidence-driven.

## 8. Necessary/Demanded Work
The engine should treat demanded or necessary work as an explicit concept.

For TreeCalc-first scope this means:
1. the engine may distinguish the full stale region from the demanded stabilization frontier,
2. observer-facing or publication-facing goals can shape what work is necessary,
3. this must not alter semantic truth, only work selection and stabilization timing.

The exact observer model is defined elsewhere, but the recalc architecture reserves this as a baseline concern rather than a late optimization surprise.

## 9. Dynamic Dependency Discipline

### 9.1 Runtime-Observed Dependencies Are First-Class
Dynamic dependency behavior must be handled through explicit runtime-observed dependency state.

This is necessary because:
1. some dependency facts are not fully knowable from structural binding alone,
2. pretending otherwise forces hidden mutation or brute-force fallback,
3. both are architecturally inferior to explicit runtime discipline.

### 9.2 Structural Graph Versus Effective Runtime Graph
The engine distinguishes:
1. structural dependency relations derivable from stable artifacts,
2. runtime-observed dependency effects attached through explicit derived state.

The effective work graph for recalculation may therefore include runtime-derived edges or edge effects,
but the architectural rule remains:
1. structural truth stays immutable,
2. runtime discovery is modeled explicitly,
3. replay and proof obligations must cover the resulting behavior.

### 9.3 Conservative Fallback Policy
The architecture permits conservative fallback when dynamic dependency conditions are not yet safely optimized.

But fallback must be:
1. explicit,
2. bounded by policy,
3. instrumented,
4. treated as a known optimization gap rather than as invisible baseline behavior.

### 9.4 External Stream Semantics Selector
External invalidation and RTD-like update handling are selected by
`StreamSemanticsProfile = (profile_version, stream_semantics_version)`.

The `stream_semantics_version` selector has three values:
1. `ExternalInvalidationV0`: pathfinder dirty-seed behavior. Topic-update
   events route to the external invalidation dirty-seed hook, but the
   repository does not mutate replay-visible topic envelopes under this
   selector.
2. `TopicEnvelopeV1`: topic updates mutate replay-visible topic envelopes
   through deterministic ordering and event-identity dedupe.
3. `RtdLifecycleV2`: topic updates use the `TopicEnvelopeV1` envelope path
   and additionally expose an RTD lifecycle tracking hook for later lifecycle
   state and replay corpus evidence.

Selector dispatch is semantic profile state, not a scheduler optimization.
Replay artifacts that include topic-update events must identify the active
`profile_version` and `stream_semantics_version`, and the update batch must be
validated against the selected behavior.

External update dispatch emits replay-visible dirty seeds as
`(topic_id, topic_sequence, formula_stable_id, node_id)`. Those seeds enter the
ordinary dependency-closure path with reason `ExternallyInvalidated`; they do
not publish values or runtime effects. Publication remains exclusively under
the coordinator commit path after ordinary session invocation.

The first checked OxCalc-local corpus for this path is
`docs/test-runs/core-engine/w050-d4-rtd-external-replay-corpus-001`. It records
the topic fixture, normalized per-profile publication artifact, and validation
commands for all three stream selector values.

## 10. Recalc Modes
The engine architecture permits distinct recalc modes, provided they preserve semantic truth.

At a high level these modes may include:
1. full rebuild,
2. incremental reuse,
3. hybrid incremental with explicit fallback.

The architecture requires:
1. deterministic mode selection policy,
2. explicit fallback triggers,
3. replay-visible consequences where required,
4. performance instrumentation for evaluating whether a mode remains justified.

## 11. Cycle Regions

### 11.1 Cycle Boundary
Cycle regions must be explicit, not accidental side effects of scheduler behavior.

### 11.2 Profile-Governed Semantics
Cycle behavior remains profile-governed.

The engine architecture must support:
1. deterministic cycle error behavior,
2. deterministic iterative cycle behavior when enabled,
3. explicit terminal-state signaling,
4. replay and diagnostic visibility for cycle progression.

### 11.3 Iterative Mode Is Not Freeform
If iterative behavior is enabled, it must remain:
1. explicitly bounded or convergence-governed,
2. deterministic under the declared profile,
3. suitable for replay and evidence capture.

### 11.4 W048 Cycle Execution Packet
Circular dependency calculation processing is owned by W048 as an execution workset, not only as a planning packet.

The active W048 packet root is:

1. `docs/spec/core-engine/w048-cycles/`

W048 refines this section by requiring:

1. explicit graph-layer classification over `G_struct`, `G_eff`, and `G_eff_candidate`,
2. materialized forward and reverse graph artifacts,
3. cycle-region records with source, members, root/order, boundary edges, and terminal policy,
4. Excel probe packets before Excel-match claims,
5. TraceCalc reference behavior and TreeCalc optimized/core behavior for declared cycle cases,
6. a deterministic circular-reference test corpus,
7. W048-owned formal definitions/models/checker targets for cycle semantics,
8. a declared iterative profile before any iterative cycle behavior is admitted,
9. profile-gated innovation opportunities that remain separate from default Excel-match behavior.

## 12. Runtime Work Selection and Scheduling

### 12.1 Deterministic Ordering Rule
Any scheduler used for realized work selection must be deterministic for the declared mode.

### 12.2 No Hidden Semantic Priority
Priority policies may influence when work is performed, but not what stabilized result means.

### 12.3 Visibility and Similar Policies
Visibility-priority and similar policies are later scheduling concerns.
They may alter work ordering or publish timing, but not semantic truth.

## 13. Advanced Optimization Lanes

### 13.1 Dynamic Topological Maintenance
Dynamic topological maintenance is retained as an intended later optimization lane.

It is not baseline realization commitment for TreeCalc Stage 1.
It is promoted as staged-later design because:
1. the research value is high,
2. the likely payoff is real,
3. crossover economics still need explicit evidence.

### 13.2 SAC-Inspired Repair
SAC-inspired or more ambitious trace-repair styles are also retained as staged-later lanes.

They are architecturally anticipated, but not baseline realization requirements.

### 13.3 Why These Lanes Remain Explicit
They remain explicit so that:
1. the architecture does not forget them,
2. later promotions have a named target,
3. the baseline does not silently ossify around a weaker model than intended.

## 14. Decisive Experiments and Instrumentation
The architecture requires decisive experiments rather than taste-based promotion.

At minimum, the roadmap must include experiments for:
1. early-cutoff hit rate and correctness,
2. dynamic-topo versus rebuild crossover,
3. dynamic-dependency tracking economics versus conservative fallback,
4. replay determinism under staged concurrency,
5. fallback-rate and reuse-economics counters.

Instrumentation is required from early realization because the architecture intends optimization by evidence, not by anecdote.

## 15. Relationship To Coordinator and Publication
The recalc engine determines work and candidate results.
It does not by itself define committed visibility.

The coordinator remains the single authority for:
1. accept/reject,
2. publication,
3. stable observer-visible state transition.

This boundary is critical.
Without it, recalc strategy and publication semantics collapse into one mutable mechanism,
which would weaken replay, proof, and concurrency discipline.

## 16. Formalization Direction
This document is intended to map into both theorem-oriented and model-oriented assurance work.

Expected assurance consequences:
1. state-transition definitions for invalidation and verification progress,
2. theorem targets for replay determinism and early-cutoff soundness,
3. theorem or strong evidence targets for dynamic-dependency from-scratch consistency,
4. pack obligations for cycle iteration, fallback behavior, and incremental equivalence,
5. TLA+ interaction points for staged concurrent scheduling and publication fences.

## 17. Open Detailed Questions
The following remain detailed follow-on questions within the architecture:
1. exact TLA+ and replay field mapping for the now-locked Stage 1 invalidation states and transitions,
2. exact Stage 1 realization subset for verification-oriented recalculation,
3. exact thresholding and policy for conservative fallback,
4. exact observer-demand integration rules,
5. exact criteria for promoting dynamic-topo from staged-later to realized baseline lane.

## 18. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - the Stage 1 invalidation-state vocabulary and first transition packet are now explicit, but replay and TLA+ field binding still need W008 and W009 realization,
  - conservative fallback thresholds and observer-demand integration remain open,
  - concurrency-specific scheduler clauses still need tighter coupling to coordinator publication behavior


