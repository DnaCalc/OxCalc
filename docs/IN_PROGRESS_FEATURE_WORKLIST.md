# IN_PROGRESS_FEATURE_WORKLIST.md — OxCalc

Canonical repo-level register of feature areas that are in-progress under workset completion doctrine.

Status: active.
Last updated: 2026-03-15.

## Status Vocabulary

- `in-progress`: partial implementation or canonical spec work exists, parity/completeness not yet achieved.
- `blocked`: in-progress with active blocker (see CURRENT_BLOCKERS.md).
- `planned`: explicitly accepted into scope, no shipped work yet.

## Active Feature Register

### IP-01: Core Rewrite and Canonicalization

- **Status**: in-progress
- **Current floor**: rewritten canonical core-engine spec set drafted; bootstrap set archived; repo integration now includes first OxFml seam handoff and receiving-side acknowledgment tracking.
- **Remaining gaps**: final integration tightening, follow-on seam alignment wording, workset closure discipline, operational execution of W013, and later replay-backed evidence.
- **Why still open**: the canonical set is established, but realization and assurance closure are still outstanding.
- **Canonical owner**: W001.

### IP-02: TreeCalc Structural State and Snapshot Kernel

- **Status**: in-progress
- **Current floor**: immutable structural snapshot, builder, projection lookup, and pinned structural view are now scaffolded in code with passing tests over snapshot construction, successor identity stability, and pinning behavior.
- **Remaining gaps**: richer structural edit operations, formula-artifact integration depth, replay artifacts, and formal or assurance bindings.
- **Why still open**: the kernel now exists as executable code, but only at the initial TreeCalc floor and without downstream assurance artifacts.
- **Canonical owner**: W002.

### IP-03: Stage 1 Coordinator and Publication Baseline

- **Status**: in-progress
- **Current floor**: canonical coordinator and publication architecture is drafted, and the local Stage 1 floor is now scaffolded in code with candidate intake, accepted-candidate recording, typed reject handling, atomic publish, pinned publication views, and passing tests for candidate-versus-publication separation and reject-is-no-publish behavior.
- **Remaining gaps**: richer publication artifact emission, replay-oriented reject-detail binding, concurrency-facing safety realization, and emitted publication diagnostics.
- **Why still open**: the sequential coordinator floor exists, but assurance, artifact emission, and broader realization remain partial.
- **Canonical owner**: W003.

### IP-04: Incremental Recalc and Overlay Baseline

- **Status**: in-progress
- **Current floor**: canonical recalc and overlay architecture is drafted, and the local Stage 1 floor is now scaffolded in code with named invalidation states, overlay keys and entries, fallback transitions, release and eviction eligibility, and passing tests for publish, verify-clean, reject, and overlay-retention paths.
- **Remaining gaps**: multi-node scheduling, topo or SCC execution, counter instrumentation, replay binding, richer runtime-effect handling, and broader overlay economics evidence.
- **Why still open**: the recalc floor now exists as an executable single-node baseline, but it is not yet the broader Stage 1 engine slice the workset targets.
- **Canonical owner**: W004.

### IP-05: OxFml Seam Hardening and Handoff Closure

- **Status**: in-progress
- **Current floor**: OxCalc-local seam requirements are drafted; `HANDOFF-CALC-001` is filed, acknowledged by OxFml, reflected in OxFml canonical seam updates, and now tightened locally with an explicit Stage 1 candidate-result, runtime-effect, and typed-reject floor.
- **Remaining gaps**: replay artifacts for candidate-result versus publication boundaries, broader runtime-derived effect taxonomy beyond the Stage 1 subset, exact trace-schema mapping, and any narrower follow-on handoff if required.
- **Why still open**: acknowledgment and local tightening exist, but evidence and full downstream alignment are still partial.
- **Canonical owner**: W005.

### IP-06: Core Formalization and Gate Binding

- **Status**: in-progress
- **Current floor**: formalization and assurance direction is drafted, W006 is active, W007 contains the first Lean-facing object inventory and transition-boundary packet, W008 contains the first TLA+-oriented coordinator-state and safety-boundary packet plus explicit Stage 1 transition bindings, W009 contains the first replay-class and pack-binding packet plus transition-coverage mapping, W010 contains the first experiment-register and measurement-schema packet, and the repo now includes the first Lean and TLA+ skeleton artifacts under `formal/`, with the Lean state file typechecked once locally.
- **Remaining gaps**: theorem authoring, TLA+ model checking, replay and pack artifact creation, initial counter-schema drafting, and execution of the remaining assurance-planning sequence.
- **Why still open**: the assurance lane now has initial state, concurrency-model, replay-binding, and measurement-planning packets plus first artifact skeletons, but not yet exercised replay, instrumentation, or TLA+ evidence.
- **Canonical owner**: W006.

### IP-07: Self-Contained Test Harness Planning

- **Status**: in-progress
- **Current floor**: `W011` now has a planning packet, three supporting spec companions, and a first checked-in hand-auditable `TraceCalc` corpus covering the minimal OxFml test-double surface, fixture lifecycle direction, canonical JSON scenario shape, validator-runner consumption contract, and the alternate calculation space for engine-only testing.
- **Remaining gaps**: final test-double payload alignment, validator and scriptable host realization, replay-pack binding, generated-corpus lanes, and later fixture implementation.
- **Why still open**: harness, scenario, and consumption guidance are now spec_drafted and seeded with an initial corpus, but no validator, fixture, host, or replay artifacts exist yet.
- **Canonical owner**: W011.

### IP-08: TraceCalc Reference Machine and Conformance Oracle

- **Status**: in-progress
- **Current floor**: `W012` now defines the first canonical oracle lane for a `TraceCalc Reference Machine`, including the reference state model, transition set, canonical artifact root, first diff policy, conformance surface, and later-engine comparison doctrine.
- **Remaining gaps**: reference-machine implementation, richer trace comparison policy beyond the first conformance surface, emitted artifact production, and the first actual engine-versus-oracle conformance run.
- **Why still open**: oracle doctrine is now spec_drafted, but no executable reference machine or conformance artifact exists yet.
- **Canonical owner**: W012.


