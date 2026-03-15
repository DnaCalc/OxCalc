# OPERATIONS.md — OxCalc Operations

## 1. Purpose
Define day-to-day execution rules for core multi-node recalc and coordinator policy.

## 2. Operating Principles
1. Semantic stability is invariant under strategy changes.
2. Coordinator is single publisher at baseline.
3. Overlay lifecycle must be deterministic and epoch-safe.
4. Visibility-priority optimization must preserve stabilized semantic equivalence.

## 3. Staged Realization
1. Stage 1:
   - sequential deterministic topo/SCC baseline,
   - atomic commit bundle handling,
   - conservative fallback allowed.
2. Stage 2:
   - partitioned parallel evaluators,
   - deterministic contention replay required,
   - snapshot/token fence hardening required.
3. Stage 3:
   - advanced policy lanes (dynamic-topo/SAC-like),
   - bounded experimental-lane policy,
   - parity evidence required before promotion.

## 4. Required Packs (baseline)
1. `PACK.visibility.policy_equivalence`
2. `PACK.visibility.starvation_bound`
3. `PACK.dag.dynamic_dependency_bind_semantics`
4. `PACK.dag.cycle_iterative_semantics`
5. `PACK.dag.external_stream_ordering`
6. `PACK.overlay.fallback_economics`
7. `PACK.concurrent.epochs` (including reject-fence and overlay-GC safety lanes)

## 5. Cross-Repo Handoff Rule
When OxCalc needs FEC/F3E protocol updates:
1. issue a handoff packet to OxFml,
2. include exact clause changes,
3. include evidence/replay artifacts,
4. include migration and fallback impact.

## 6. Promotion Gate
No scheduler/invalidation policy promotion without:
1. deterministic replay for affected classes,
2. updated pack expectations and matrix links,
3. explicit semantic-equivalence statement.

## 7. Pre-Closure Verification Checklist

Before claiming any workset or feature item as complete, answer each item yes or no.
All items must be "yes" for a completion claim. Any "no" means the item is `in_progress`.

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | |
| 2 | Pack expectations updated for affected packs? | |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | |
| 6 | All required tests pass? | |
| 7 | No known semantic gaps remain in declared scope? | |
| 8 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | |
| 9 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | |
| 10 | CURRENT_BLOCKERS.md updated (new/resolved)? | |

## 8. Expanded Definition of Done

A workset or feature item is done for its declared scope only when all of the following hold:

1. **Spec text**: all in-scope coordinator/policy spec text and realization notes are updated and internally consistent.
2. **Pack expectations**: all affected pack expectations are updated with evidence links.
3. **Replay evidence**: at least one deterministic replay artifact per in-scope behavior proves intended semantics.
4. **Semantic-equivalence**: for any policy or strategy change, a semantic-equivalence statement demonstrates that observable results are invariant under the change for affected profiles.
5. **FEC/F3E impact**: cross-repo impact on OxFml evaluator-facing clauses is assessed; handoff packet filed if shared protocol changes needed.
6. **Overlay invariants**: overlay lifecycle changes are demonstrated to be deterministic and epoch-safe.
7. **No semantic gaps**: no known semantic gap remains between spec and exercised behavior for declared scope.
8. **Three-axis report**: completion report includes `scope_completeness`, `target_completeness`, `integration_completeness`, and `open_lanes` per AGENTS.md Section 3.
9. **Checklist attached**: Pre-Closure Verification Checklist (Section 7) is filled in and all items are "yes".

## 9. Completion Claim Self-Audit

Before submitting a completion claim, the agent must perform this self-audit and include the results.

### Step 1: Scope Re-Read
Re-read the workset scope declaration. For each in-scope item, verify that exercised implementation (not scaffolding) matches. Any missing item = `in_progress`.

### Step 2: Gate Criteria Re-Read
Re-read the workset gate criteria. All pass criteria must be met. Any unmet criterion = gate open.

### Step 3: Silent Scope Reduction Check
Compare the original scope declaration with what was actually delivered. Any unreported narrowing of scope is a doctrine violation. If scope was intentionally narrowed, it must be explicitly documented with rationale.

### Step 4: "Looks Done But Is Not" Pattern Check
Check for these patterns:
- Stubs or placeholder implementations reported as real.
- Insufficient test coverage masking untested paths.
- Spec text that does not match exercised implementation.
- Handoffs filed but not acknowledged by receiving repo.

### Step 5: Include Result
Include the self-audit result in the completion report with explicit pass/fail for each step.

## 10. Report-Back Completeness Contract

Every completion report (status updates, workset closure notes, handoff summaries) must include:

1. `execution_state`: `planned` | `in_progress` | `blocked` | `complete`
2. `scope_completeness`: `scope_complete` | `scope_partial`
3. `target_completeness`: `target_complete` | `target_partial`
4. `integration_completeness`: `integrated` | `partial`
5. `open_lanes`: explicit list when any completeness axis is partial

Normative wording rules:
1. Use `complete for declared scope` only when the declared scope already represents full known semantics and only integration or external limits remain partial.
2. Do not use `complete for declared scope` for semantically bounded subsets that still carry known gaps; report those as `scope_partial`.
3. Do not claim `fully complete` unless all three completeness axes are complete and evidence links are present.

## 11. Carried-Forward Operating Lessons

These five lessons are derived from observed execution failures in OxVba (86+ worksets) and OxFunc (13 worksets). They are not speculative — each addresses a real failure mode.

OxCalc-local lessons discovered from exercised work now live in `docs/LOCAL_EXECUTION_DOCTRINE.md`.

### Lesson 1: Scaffold Determinism Is a Gate
Scaffolding (stubs, empty traits, compile-only code) must produce deterministic outputs or be explicitly marked non-functional. Non-deterministic scaffolding that silently passes tests is a gate failure.
*Source: OxVba Lesson 1.*

### Lesson 2: Spec Drift Checks Run Alongside Implementation
Do not defer spec-vs-implementation consistency checks to a separate phase. Run them as part of each workset execution. Spec drift discovered late is expensive to reconcile.
*Source: OxVba Lesson 3.*

### Lesson 3: Final Validation Must Not Mutate Tracked Evidence
Validation runs must not modify the artifacts they are validating. Evidence mutation during validation invalidates the evidence chain.
*Source: OxVba Lesson 9.*

### Lesson 4: Guard Artifact Scope Before Commit
Before committing, verify that only intended artifacts are staged. Accidental inclusion of generated files, temporary outputs, or out-of-scope changes pollutes the evidence record.
*Source: OxVba Lesson 12.*

### Lesson 5: Partial Semantics Are Not Implementation
A coordinator policy, scheduling algorithm, or protocol that covers a subset of its declared semantic space is work-in-progress, not an implementation. This applies even if the subset compiles, passes tests, and looks correct for the covered cases.
*Source: OxFunc doctrine decision.*

## 12. Upstream Observation Ledger Protocol

### 12.1 Purpose
Repos that depend on OxCalc discover interface and design constraints through their own implementation work. Those observations must flow back to OxCalc through a structured channel so they inform design before contracts solidify.

This is distinct from handoff packets (Section 5), which propose specific normative text changes. Observation ledgers are standing documents that accumulate design feedback over time.

### 12.2 Inbound Observation Sources
OxCalc must check for inbound observation ledgers from consumer repos at the start of any design or interface workset. Known source locations:

| Source repo | Ledger location | Relationship |
|-------------|----------------|--------------|
| OxFml | `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | Evaluator-facing coordinator constraints |

(Host repos may also contribute observations as they exercise OxCalc interfaces.)

### 12.3 Outbound Observations
When OxCalc implementation work reveals design constraints that affect a sibling repo, write them to `docs/upstream/NOTES_FOR_<REPO>.md` following this structure:

1. **Purpose**: what the consuming repo needs to know and why.
2. **Core message**: the essential design constraint in 2-3 sentences.
3. **Current evidence**: specific examples with concrete scenarios.
4. **Interface implications**: what the receiving repo must preserve, avoid, or expose.
5. **Minimum invariants**: binary testable statements.
6. **Open questions**: explicit questions the receiving repo should answer.

### 12.4 Lifecycle
1. Observation ledgers are living documents — updated as new evidence accumulates.
2. Entries are never silently removed; outdated observations are marked superseded with rationale.
3. When an observation is addressed by the receiving repo (through spec changes, interface decisions, or handoff packets), the originating entry is updated with a resolution reference.
4. Observation ledgers are not completion artifacts — they do not close worksets or satisfy gate criteria. They are design inputs.

### 12.5 Agent Obligation
Agents starting work on OxCalc interface or contract design must:
1. Check all listed inbound observation sources (Section 12.2).
2. Note any unresolved observations that are relevant to current scope.
3. Include a "reviewed inbound observations" line in the workset status report.
4. When a design decision addresses an inbound observation, reference the observation entry explicitly.

## 13. Emitted Artifact Protocol

### 13.1 Canonical Artifact Root Required
Any execution packet that expects emitted evidence must declare a canonical artifact root before implementation begins.

That declaration must state:
1. the canonical root path,
2. whether the artifacts are checked in or ephemeral,
3. whether emitted artifacts are runner-only, oracle-only, or comparison artifacts.

### 13.2 Path Normalization Rule
Tracked artifacts must use repo-relative paths only.

Absolute paths are allowed only in transient local diagnostics that are not tracked.

### 13.3 Validation Non-Mutation Rule
Validation runs must not mutate tracked evidence in place.

If a checked-in baseline run exists:
1. re-validation should run into a separate transient run id, or
2. the tracked baseline should be regenerated intentionally as a new evidence act, not accidentally during validation.

### 13.4 Artifact Root Reporting Rule
Completion and status reports for any emitted-evidence workset must name:
1. the canonical artifact root,
2. the checked-in baseline run if one exists,
3. the commands used to generate or validate it.

## 14. Execution Packet Minimums

Any workset that acts as an execution packet must include the following sections explicitly.

### 14.1 Environment Preconditions
1. required tools on PATH,
2. optional tools and their role,
3. fallback evidence rules if optional tools are unavailable.

### 14.2 Evidence Layout
1. canonical emitted artifact root,
2. checked-in versus ephemeral policy,
3. stable naming policy for baseline runs.

### 14.3 Replay-Corpus Readiness
If replay classes are part of the gate model, the packet must state:
1. which replay classes require corpus scenarios before implementation begins,
2. which scenario ids satisfy them,
3. which replay classes remain reserve or later lanes.

### 14.4 Pack-Evidence Traceability
Execution packets that mention packs must identify:
1. pack name,
2. replay classes,
3. scenario ids or artifact paths once they exist.

## 15. Local Doctrine Reference
OxCalc-local execution lessons now live at `docs/LOCAL_EXECUTION_DOCTRINE.md`.

Those lessons are additive to the carried-forward lessons in Section 11.
They should be updated when later execution waves reveal new recurring failure modes or stronger operating practices.
