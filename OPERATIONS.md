# OPERATIONS.md — OxCalc Operations

## 1. Purpose
Define how OxCalc works day to day. The rule is deliberately lean: trust the spec, the workset register, and the bead graph; do not add status ceremony to compensate for unclear product direction.

OxCalc follows the DnaTreeCalc / Foundation slim execution model:
1. `CHARTER.md` owns mission and repo boundaries.
2. `docs/SPEC.md` indexes the mutable spec/design set.
3. `docs/WORKSET_REGISTER.md` owns the roadmap, large work areas, dependency shape, and coarse history.
4. `.beads/`, through `br`, owns live execution truth.
5. `docs/BEADS.md` is a pocket reference for local bead mechanics, not a second execution authority.

## 2. Operating Principles
1. Semantic stability is invariant under strategy changes.
2. Coordinator is single publisher at baseline.
3. Overlay lifecycle must be deterministic and epoch-safe.
4. Visibility-priority optimization must preserve stabilized semantic equivalence.

## 2A. Execution Model
OxCalc executes through `workset -> epic bead -> child beads`.

Execution-state rule:
1. worksets are large planned work areas, not atomic tasks,
2. every active non-bootstrap workset should have an epic bead once it enters execution,
3. beads are the atomic units of executable progress,
4. `.beads/` owns readiness, blockers, in-progress state, dependencies, and closure,
5. the register is a roadmap and work history, not a live board.

Transition rule:
1. `W032` established the bead-model doctrine shift in OxCalc,
2. `.beads/` is now bootstrapped as the live execution-state surface,
3. `CURRENT_BLOCKERS.md` no longer owns ordinary blocker truth,
4. `docs/worksets/README.md` and `docs/IN_PROGRESS_FEATURE_WORKLIST.md` are orientation surfaces, not execution-status surfaces,
5. use `br ready`, `br list --status in_progress`, and `br epic status` for live state.

## 2B. High-Signal Execution Doctrine
OxCalc execution should keep the active repo small enough to understand and strong enough to trust.

1. Product behavior is the center of gravity: engine state, dependency graph construction, invalidation, evaluation, replay, and host-facing semantics.
2. Formalization should create executable leverage: state machines, transition relations, invariants, pre/post conditions, refinement links, and falsifiable checks.
3. A new artifact must have a named consumer: code, tests, proof/model checks, replay validation, a durable spec, or a specific human decision.
4. A successor workset should first distill predecessor output. If the durable result fits in an active spec paragraph or table, update that surface and archive the rest.
5. Generated evidence stays active only while it is a normative baseline or an input to current validation. Historical snapshots belong under the top-level `archive/` with a short manifest.
6. Prefer direct engineering moves over taxonomy expansion: fix behavior, write the model, run the replay, tighten the invariant, or remove stale material.
7. Keep code free of workset-specific accretion unless the runner or fixture is still an active validation tool. Legacy runners should be isolated, retired, or archived through explicit cleanup beads.
8. Reports should identify what behavior changed, what check proved it, and the next concrete engineering move.

## 2C. Product Claims Versus Formal Claims
Feature-area status must be reported in plain product terms first.

Use this shape unless a workset explicitly asks for something else:
1. `Product status`: what is supported and for whom.
2. `Evidence`: the decisive tests, replay runs, Excel observations, model checks, or review observations.
3. `Still open`: concrete exclusions or gaps.
4. `Formal status`: Lean/TLA/model/proof status, separate from product implementation status.

Do not use broad `partial` wording as a substitute for naming the supported product scope. A feature can be implemented for a declared product scope while formal proof, broader Excel-version coverage, or a successor edge-case lane remains open.

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

## 7. Verification And Closure

Verification is layered and applied to the work actually touched. Do not run or report a blanket ceremony when a small useful check answers the question.

Gate tiers:
1. **Local checks**: Rust build/test/lint/format, scripts, read-through, link/register scan, or `br` sanity check as appropriate.
2. **Excel anchor**: required when the behavior is Excel-defined. The claim is "matches Excel for this declared scope", backed by reproducible observations or replay/comparison artifacts.
3. **Formal anchor**: Lean/TLA/model/proof checks where they are load-bearing. Formal anchoring is a standing aim, but formal proof is not automatically the product-implementation gate.

A bead closes when:
1. its one outcome exists,
2. the useful acceptance check or observation ran,
3. a fresh-eyes review found no material issue or filed follow-up beads,
4. touched spec/design surfaces are updated,
5. newly discovered work is back in `.beads/`.

A workset closes when:
1. its closure condition is satisfied for the declared scope,
2. its epic/child bead state supports that claim,
3. touched artifacts have a housekeeping pass: keep, mark, or delete,
4. product status, evidence, exclusions, and formal status can be stated plainly.

## 8. Completion Claims

Completion language is allowed when scoped. Say what is implemented and for which scope; do not replace the answer with abstract completeness axes.

A valid completion claim names:
1. the declared product or workset scope,
2. the implementation surface,
3. the decisive evidence/checks,
4. explicit exclusions and successor lanes,
5. formal/proof status if relevant.

Scaffolding, placeholder code, compile-only paths, unexercised switches, and unconsumed spec text are not implementation. Handoffs are dependencies; they close only the local request, not behavior that depends on the receiving repo.

## 9. Fresh-Eyes Review

Before closing a non-trivial bead or workset, re-read or exercise the touched area as if encountering it fresh. Look for:
1. contradictions between spec, code, tests, and artifacts,
2. unsupported product claims,
3. stale status wording,
4. hidden exclusions,
5. brittle fixture-only behavior presented as general behavior,
6. docs or generated outputs that no longer earn their place.

Fix issues in scope. File follow-up beads for real out-of-scope work. Close notes should name the decisive check or observation, not repeat boilerplate.

## 10. Reporting

Use the shortest report that answers the user's question.

Preferred shape:
1. `Product status`: supported scope in plain language.
2. `Evidence`: decisive checks, runs, observations, or review findings.
3. `Still open`: concrete gaps or exclusions.
4. `Formal status`: proof/model state when relevant.

Historical packet-local status fields may remain valid in older artifacts. Do not copy them into ordinary reports or completion notes unless the artifact being edited explicitly owns that format.

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

### 13.5 Baseline Run Retention Rule
Execution packets that check in emitted baseline runs must state:
1. which baseline run is the active normative one for that wave,
2. whether earlier baseline runs remain active references or historical snapshots,
3. how intentional regeneration is recorded.

Later waves must not silently replace an earlier checked-in baseline run.
They may supersede it only by naming the new active baseline explicitly.

### 13.6 Replay Projection Additivity Rule
Replay-appliance bundle roots, validator outputs, and explain artifacts are additive sidecars unless a spec explicitly says otherwise.

That means:
1. the native OxCalc artifact root remains the semantic authority,
2. replay-facing projections may normalize, enrich, or validate emitted evidence,
3. replay-facing projections may not silently replace the native artifact surface as the authoritative meaning layer.

### 13.7 Capability Snapshot Consistency Rule
If a replay-facing run emits a run-local capability snapshot, it must remain aligned with the canonical manifest.

That means:
1. the canonical manifest remains the authority for claimed capability levels,
2. run-local snapshots may narrow or annotate the claim, but may not silently widen it,
3. when the canonical manifest changes, checked-in replay-facing baselines and run-local snapshots must be intentionally regenerated or explicitly marked stale.

### 13.8 Projection-Validator-Explain Coupling Rule
Replay-facing capability-promotion waves must declare projection, validator, and explain outputs together.

That means:
1. an emitted replay bundle root without validator output is a partial wave,
2. validator output without explain output may still be below the intended capability floor,
3. a capability-promotion workset must state the capability consequence of each emitted replay-facing artifact family.

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

If semantic widening depends on an oracle or conformance lane, the packet must also state:
1. which oracle surfaces must widen in the same slice,
2. which engine-versus-oracle comparison artifact proves the widened behavior.

### 14.4 Pack-Evidence Traceability
Execution packets that mention packs must identify:
1. pack name,
2. replay classes,
3. scenario ids or artifact paths once they exist.

### 14.5 Capability-Ladder Continuation
If an execution packet is expected to advance a replay capability ladder and later levels are already known, the successor packet should be authored before the current packet closes.

This avoids:
1. smearing later capability work back into the closing packet,
2. implicitly reopening a packet that already reached its declared gate,
3. ambiguity about where the next capability promotion act belongs.

### 14.5 Workset Versus Feature-Area Rule
Worksets are planning containers. Feature areas are product capabilities.

Rule:
1. a workset may close for its declared scope without closing the broader feature area,
2. later widening should use a successor workset or explicit extension lane,
3. reports must say the product feature status plainly instead of implying that a scoped workset closure answers the whole feature-area question,
4. ordered workset truth belongs in `docs/WORKSET_REGISTER.md` while live execution state belongs in `.beads/`.

## 15. Local Doctrine Reference
OxCalc-local execution lessons now live at `docs/LOCAL_EXECUTION_DOCTRINE.md`.

Those lessons are additive to the carried-forward lessons in Section 11.
They should be updated when later execution waves reveal new recurring failure modes or stronger operating practices.

## 16. Rust-First Realization Doctrine

### 16.1 Direction
1. OxCalc implementation work for the core engine and `TraceCalc` executable host/tooling lane is Rust-first.
2. The active implementation lives under the Rust workspace in `src/`.
3. Historical baseline runs and checked-in emitted artifacts remain valid evidence and comparison references after implementation shifts.
4. New feature or behavior work must land in the Rust implementation lane unless a workset explicitly declares a docs-only historical preservation task.

### 16.2 Foundation Conflict Adaptation
1. Foundation doctrine still states that repository tooling follows a different default than OxCalc now does locally.
2. For OxCalc, this default is explicitly adapted locally for behavior-critical engine and `TraceCalc` runtime work: those lanes are now Rust-first by repo direction.
3. This adaptation does not rewrite Foundation doctrine globally; it is an OxCalc-local implementation direction for this repo.

### 16.3 Rust Quality Floor
1. `unsafe` is forbidden for OxCalc Rust realization. Crates must declare `#![forbid(unsafe_code)]`.
2. Rust work must be warning-clean under the declared toolchain. Validation should include `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and the relevant `cargo test` scope.
3. Error handling must use explicit typed error pathways where failures are part of behavior or tooling control flow; avoid stringly typed control flow as the primary contract.
4. Deterministic replay, artifact emission, and comparison behavior must remain first-class constraints in crate and module design.
5. Interior mutability, global state, or hidden concurrency primitives require explicit justification in the workset and must not be introduced casually.
6. Rust module and type design must follow Rust ownership and data-model strengths rather than imitating older service, inheritance, or mutable object-graph patterns.

### 16.4 Porting Discipline
1. Reimplementation work must treat OxCalc specs, replay artifacts, baseline runs, and conformance behavior as the authority.
2. Historical prior-language artifacts may be used as behavior and evidence references, but not as the design template to be copied mechanically.
3. Any semantic difference introduced by the Rust realization must be called out explicitly and justified through the normal semantic-equivalence and replay-evidence rules.
