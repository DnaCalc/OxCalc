# CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md

## 1. Purpose and Status
This document defines the OxCalc coordinator, publication, and staged concurrency model for the rewritten core-engine spec set.

Status:
1. active rewrite baseline,
2. intended canonical companion for commit and publication authority,
3. TreeCalc-first in immediate realization scope,
4. staged for later concurrent and async realization.

This document defines:
1. coordinator authority,
2. accepted and rejected work consequences,
3. publication boundaries,
4. observer-visible stability rules,
5. staged concurrency and async progression,
6. core assurance targets for coordinator behavior.

## 2. Coordinator Mission
The coordinator is the single authority that turns evaluator-produced candidate runtime work into stable engine-visible consequences.

Its mission is to:
1. govern snapshot and fence compatibility,
2. accept or reject candidate work,
3. publish accepted results atomically,
4. prevent rejected work from leaking into stable publication,
5. preserve stable views for readers and observers,
6. stage concurrency without semantic drift.

## 3. Single-Publisher Rule
The baseline engine has one publication authority: the coordinator.

This rule means:
1. no evaluator or runtime worker publishes committed state directly,
2. all observer-visible committed state transitions pass through coordinator logic,
3. accepted-commit atomicity is defined centrally,
4. reject semantics are enforced centrally.

This is a correctness rule first and a coordination rule second.

## 4. Coordinator Responsibilities
At the architecture level, the coordinator is responsible for:
1. structural snapshot and fence awareness,
2. runtime work admission and compatibility checks,
3. commit acceptance or rejection,
4. publication of stable observer-visible state,
5. retention constraints that affect pinned-reader safety,
6. staged concurrency arbitration,
7. replay-visible reject and contention behavior where required.

The coordinator may delegate computation, but it may not delegate final publication authority.

## 5. Fence and Compatibility Model

### 5.1 Fence Principle
Any candidate work intended for publication must be checked against the relevant compatibility boundaries.

These boundaries may include:
1. structural snapshot compatibility,
2. token or artifact compatibility,
3. profile or version compatibility,
4. capability or fence compatibility,
5. publication-state compatibility.

### 5.2 Why Fences Matter
Fences exist so that:
1. work computed against stale assumptions is not silently published,
2. replay can explain accept or reject outcomes,
3. staged concurrency does not weaken correctness,
4. stable observer views remain coherent.

### 5.3 Reject Consequence
When compatibility conditions do not hold, the coordinator rejects candidate work rather than publishing partially compatible state.

## 6. Accepted-Commit Publication Rule

### 6.1 Atomicity
An accepted commit publishes one atomic stable bundle of consequences for the accepted work unit.

Atomic here means:
1. the stable observer-visible consequences appear together,
2. partially visible accepted publication is forbidden,
3. publication is tied to a coherent snapshot and fence basis.

### 6.2 Publication Content
The exact bundle schema is defined by seam and supporting documents, but at architecture level the coordinator publishes a coherent derived result package rather than disconnected mutable fragments.

Where OxFml canonical seam language applies, that publication package is produced from an evaluator-side `AcceptedCandidateResult` that remains distinct from publication until coordinator acceptance.

### 6.3 Publication Ordering
Publication ordering must be deterministic for the declared mode.

The coordinator may expose progress or staged stabilization behavior where later documents allow it,
but it may not do so through semantically ambiguous partial publication.

## 7. Reject-Is-No-Publish Rule

### 7.1 Core Rule
Rejected work publishes no stable accepted state.

### 7.2 Consequences
This means:
1. no accepted result fragment is visible from rejected work,
2. no observer-visible publication state advances based on rejected work,
3. no internal optimization shortcut may treat rejected work as if it were accepted publication.

### 7.3 Diagnostics and Replay
Reject information may still be preserved for:
1. diagnostics,
2. replay,
3. pack evidence,
4. migration or seam analysis.

But diagnostic preservation is not publication.

## 8. Observer-Visible Stability Rules

### 8.1 Stable View Rule
The coordinator must preserve stable observer-visible views.

Readers and observers see:
1. a coherent structural snapshot,
2. a coherent published runtime view for that snapshot and fence basis,
3. status signaling that is valid for that view.

### 8.2 No Torn Publish Rule
The coordinator must prevent torn observer-visible states such as:
1. partially published accepted work,
2. publication from incompatible fence bases,
3. rejected work leaking into the stable view,
4. overlay consequences becoming visible without authorized publication.

### 8.3 Ongoing Recalc Rule
Ongoing work may continue while observers read stable prior state.

That ongoing work must not retroactively mutate what a pinned observer sees.

## 9. Stage 1 Baseline Coordinator

### 9.1 Scope
Stage 1 is the sequential single-publisher baseline.

### 9.2 Required Properties
Stage 1 must prove:
1. the coordinator owns publication,
2. deterministic accept or reject behavior,
3. stable observer-visible state,
4. explicit fence discipline,
5. replay-compatible rejection behavior,
6. TreeCalc-first realization on a simpler substrate.

### 9.3 What Stage 1 Does Not Need To Prove Yet
Stage 1 does not need to prove full parallel evaluation throughput.
It does need to prove the coordinator architecture that later concurrency will depend on.

## 10. Stage 2 Concurrent and Async Progression

### 10.1 Scope
Stage 2 introduces partitioned, concurrent, or asynchronous evaluator work behind the same coordinator authority.

### 10.2 Invariants That Must Not Change
Stage 2 must preserve:
1. single publication authority,
2. accept or reject fence discipline,
3. stable observer-visible state,
4. deterministic replay obligations,
5. no semantic drift from baseline truth.

### 10.3 New Obligations
Stage 2 introduces additional obligations such as:
1. deterministic contention handling,
2. replay-visible concurrency outcomes where required,
3. safe interaction with pinned readers,
4. publication discipline under overlapping runtime work.

## 11. Async and In-Flight Work
The architecture allows ongoing work to exist without becoming stable publication.

This distinction is essential.

In-flight work may:
1. produce an evaluator-side `AcceptedCandidateResult`,
2. produce internal runtime state,
3. wait on compatibility checks,
4. be rejected.

In-flight work may not:
1. redefine stable observer-visible truth by default,
2. bypass coordinator publication,
3. force torn state onto readers.

## 12. Coordinator and Overlay Interaction
The coordinator governs whether overlay-derived consequences matter to stable publication.

This means:
1. overlays may inform evaluator-produced candidate work,
2. overlays may influence accept or reject decisions,
3. overlays may be retained or evicted under coordinator-aware safety rules,
4. overlay consequences reach stable observers only through coordinator-controlled publication.

## 13. Coordinator and OxFml Interaction
The coordinator depends on OxFml evaluator work and seam discipline, but publication authority remains in OxCalc.

The coordinator therefore requires a seam that supports:
1. explicit candidate-work boundaries,
2. explicit compatibility and fence information,
3. explicit `AcceptedCandidateResult` payload structure,
4. explicit reject-detail structure suitable for replay and diagnostics.

Detailed ownership and handoff text belongs in the dedicated seam document.

## 14. Contention and Retry Direction
Later concurrent stages may require contention and retry policy.

The architecture locks only the high-level rule here:
1. contention behavior must be explicit,
2. retry behavior must be deterministic under the declared mode where replay requires it,
3. contention resolution must not bypass publication fences,
4. fallback or retry policy must be visible to assurance and evidence tooling where required.

## 15. Publication and Stabilization
The coordinator governs when evaluator-produced candidate work becomes stable published state.

The architecture distinguishes:
1. structural truth,
2. runtime in-flight work,
3. stable published view.

Stabilization and publication policy may evolve in detail, but it must remain true that:
1. stable publication is explicit,
2. accepted publication is atomic,
3. rejected work is no-publish,
4. observer-visible state remains coherent.

## 16. Stage 1 Local Candidate and Reject Packet

### 16.1 AcceptedCandidateResult Intake Minimum
The minimum Stage 1 OxCalc-local `AcceptedCandidateResult` intake packet should contain:
1. `candidate_result_id`
2. `struct_snapshot_id`
3. `artifact_token_basis`
4. `compatibility_basis`
5. `target_set`
6. `value_updates`
7. `dependency_shape_updates`
8. `runtime_effects`
9. `diagnostic_events`

This is the minimum local coordinator intake surface required to preserve candidate-versus-publication separation while still allowing atomic publish, typed reject, and replay-friendly diagnostics.

### 16.2 Publication Bundle Minimum
The minimum Stage 1 OxCalc-local publication bundle derived from an accepted candidate result should contain:
1. `publication_id`
2. `candidate_result_id`
3. `published_view_delta`
4. `published_runtime_effects`
5. `counter_deltas`
6. `trace_markers`

This is the minimum observer-facing stable publication surface for the Stage 1 coordinator.
The exact shared canonical field names may differ on the OxFml side, but the coordinator-local publication consequences are fixed to this minimum shape.

### 16.3 Stage 1 Reject Taxonomy
The minimum Stage 1 coordinator-local reject classes should be:
1. `snapshot_mismatch`
2. `artifact_token_mismatch`
3. `profile_version_mismatch`
4. `capability_mismatch`
5. `publication_fence_mismatch`
6. `dynamic_dependency_failure`
7. `synthetic_cycle_reject`
8. `host_injected_failure`

These classes are the local Stage 1 floor for coordinator reasoning, replay classification, and typed no-publish behavior.
They do not claim that the shared OxFml canonical taxonomy is closed.

## 17. Formalization Direction
Coordinator behavior is one of the highest-priority near-formal areas in the core engine.

Expected assurance consequences include:
1. TLA+ state and transition modeling for coordinator safety,
2. safety properties for no torn publication,
3. safety properties for reject-is-no-publish,
4. safety properties for pinned-reader stability,
5. liveness or progress analysis for staged concurrent and async execution where applicable,
6. replay and pack obligations for contention and reject behavior.

## 18. Open Detailed Questions
These remain detailed follow-on questions within the now-locked architecture:
1. exact in-flight progress publication policy if any,
2. exact contention and retry policies for later stages,
3. exact relationship between stabilized state markers and observer APIs,
4. exact pack and trace binding for the now-locked Stage 1 candidate and reject classes,
5. exact promotion path from the Stage 1 local packet shape to later richer comparison and replay payloads.

## 19. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - replay artifacts still needed for candidate-result versus publication behavior,
  - the Stage 1 local packet shape is now explicit, but pack and trace binding still need W009 realization,
  - no exercised coordinator implementation or emitted publication artifacts exist yet


