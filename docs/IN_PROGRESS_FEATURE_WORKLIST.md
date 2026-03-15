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
- **Remaining gaps**: multi-node scheduling, topo or SCC execution, emitted counter instrumentation, replay widening, richer runtime-effect handling, and broader overlay economics evidence.
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
- **Current floor**: formalization and assurance direction is drafted, W006 is active, W007 contains the first Lean-facing object inventory and transition-boundary packet, W008 contains the first TLA+-oriented coordinator-state and safety-boundary packet plus explicit Stage 1 transition bindings, W009 contains the first replay-class and pack-binding packet plus transition-coverage mapping, W010 contains the first experiment-register and measurement-schema packet, and the repo now includes first Lean, TLA+, replay-seed, and measurement-schema artifacts under `formal/`, with the Lean state file typechecked once locally and the JSON replay/measurement artifacts validated.
- **Remaining gaps**: theorem authoring, TLA+ model checking, broader replay artifact coverage, emitted counter snapshots, pack artifact creation, and W014 execution over the widened Stage 1 slice.
- **Why still open**: the assurance lane now has initial state, concurrency-model, replay-binding, and measurement-planning packets plus first artifact skeletons and seed artifacts, but not yet exercised pack, instrumentation, or TLA+ evidence.
- **Canonical owner**: W006.

### IP-07: Self-Contained Test Harness Planning

- **Status**: in-progress
- **Current floor**: `W011` has now progressed from planning into a first realized slice: the repo contains the `TraceCalc` corpus, a validator and runner path under `src/OxCalc.Core/TraceCalc/`, a small tool host under `src/OxCalc.TraceCalc.Tool/`, unit-test coverage for the runner, and a checked-in emitted baseline run at `docs/test-runs/core-engine/tracecalc-reference-machine/w013-sequence-a-baseline/`.
- **Remaining gaps**: broader replay-pack export, generated-corpus lanes, richer failure-surface coverage, widened Stage 1 corpus for `R3`, `R6`, `R8`, DAG, and SCC cases, and later OxFml-integrated harness coverage.
- **Why still open**: the first self-contained harness slice is exercised, but the broader harness feature area still has later replay, scale, and integrated-host lanes.
- **Canonical owner**: W011.

### IP-08: TraceCalc Reference Machine and Conformance Oracle

- **Status**: in-progress
- **Current floor**: `W012` now includes an executable `TraceCalc Reference Machine`, an engine adapter, conformance comparison logic, a checked-in emitted oracle/conformance baseline run for the seven-scenario corpus, and unit-test coverage for the runner path that hosts the oracle.
- **Remaining gaps**: richer trace comparison policy beyond the first conformance surface, broader corpus coverage, widened Stage 1 conformance over multi-node and SCC-oriented cases, and later continuous engine-versus-oracle series beyond the current baseline run.
- **Why still open**: the first oracle slice now exists and is exercised, but the broader oracle feature area still has later replay-pack and scale lanes.
- **Canonical owner**: W012.




