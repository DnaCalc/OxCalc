# IN_PROGRESS_FEATURE_WORKLIST.md — OxCalc

Canonical repo-level register of feature areas that are in-progress under workset completion doctrine.

Status: active.
Last updated: 2026-03-16.

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
- **Current floor**: canonical recalc and overlay architecture is drafted, and the local Stage 1 floor now executes a widened planner-driven slice with deterministic multi-node DAG scheduling, first SCC-oriented handling, fallback re-entry, emitted per-scenario counters, and passing tests plus a checked-in widened baseline run.
- **Remaining gaps**: richer runtime-effect handling, broader overlay-economics reporting, replay-appliance bundle projection, and later concurrency-facing widening.
- **Why still open**: the widened Stage 1 slice now exists, but later evidence, replay projection, and Stage 2-facing lanes remain partial.
- **Canonical owner**: W004.

### IP-05: OxFml Seam Hardening and Handoff Closure

- **Status**: in-progress
- **Current floor**: OxCalc-local seam requirements are drafted; `HANDOFF-CALC-001` is filed, acknowledged by OxFml, reflected in OxFml canonical seam updates, and now tightened locally with an explicit Stage 1 candidate-result, runtime-effect, and typed-reject floor.
- **Remaining gaps**: replay artifacts for candidate-result versus publication boundaries, broader runtime-derived effect taxonomy beyond the Stage 1 subset, exact trace-schema mapping, and any narrower follow-on handoff if required.
- **Why still open**: acknowledgment and local tightening exist, but evidence and full downstream alignment are still partial.
- **Canonical owner**: W005.

### IP-06: Core Formalization and Gate Binding

- **Status**: in-progress
- **Current floor**: formalization and assurance direction is drafted, W006 is active, W007 contains the first Lean-facing object inventory and transition-boundary packet, W008 contains the first TLA+-oriented coordinator-state and safety-boundary packet plus explicit Stage 1 transition bindings, W009 contains the replay-class and pack-binding matrix through `R8`, W010 contains the experiment-register and measurement-schema packet, and the repo now includes Lean, TLA+, replay-seed, measurement-schema, emitted counter, and widened baseline run artifacts; the Lean state file has been typechecked locally and the TLA+ smoke model has been checked once with TLC.
- **Remaining gaps**: theorem authoring, richer TLA+ model exploration, replay-appliance bundle projection, pack artifact creation, and later retained-witness evidence.
- **Why still open**: the assurance lane now has real widened Stage 1 evidence, but its later proof, pack, and replay-appliance lanes remain partial.
- **Canonical owner**: W006.

### IP-07: Self-Contained Test Harness Planning

- **Status**: in-progress
- **Current floor**: `W011` now includes an exercised 12-scenario `TraceCalc` corpus, validator and runner paths under `src/OxCalc.Core/TraceCalc/`, a small tool host under `src/OxCalc.TraceCalc.Tool/`, planner tests, runner tests, and checked-in emitted baseline runs at `w013-sequence-a-baseline` and `w014-stage1-widening-baseline`.
- **Remaining gaps**: replay-appliance bundle export, richer retained-failure handling, larger generated-corpus lanes, and later OxFml-integrated harness coverage.
- **Why still open**: the widened self-contained harness slice is exercised, but later replay, retained-witness, and integrated-host lanes remain partial.
- **Canonical owner**: W011.

### IP-08: TraceCalc Reference Machine and Conformance Oracle

- **Status**: in-progress
- **Current floor**: `W012` now includes an executable `TraceCalc Reference Machine`, an engine adapter, conformance comparison logic, planner-driven DAG and SCC coverage, and checked-in emitted oracle/conformance baseline runs for both the original and widened 12-scenario corpora.
- **Remaining gaps**: richer trace comparison policy beyond the first conformance surface, replay-appliance bundle projection, reduced-witness flows, and later continuous engine-versus-oracle series beyond the current baseline runs.
- **Why still open**: the oracle is exercised over the widened Stage 1 slice, but later replay-appliance and retained-witness lanes remain partial.
- **Canonical owner**: W012.

### IP-09: Replay Appliance Adapter Rollout

- **Status**: in-progress
- **Current floor**: `W015` now has an execution-ready packet, local replay-coherence refactor direction, explicit adapter and capability-manifest docs, normalized event-family projection embodied in runner output, typed local mismatch projection fields, and replay-facing scenario metadata carried in the checked-in corpus and active widened baseline run.
- **Remaining gaps**: normalized replay-appliance bundle emission, bundle-validator conformance artifacts, explain records, capability promotion beyond the current conservative floor, and later retained-witness rollout.
- **Why still open**: local replay coherence and adapter doctrine are now in place, but the actual bundle-projection and retained-witness lanes are still later work.
- **Canonical owner**: W015.

### IP-10: Rust-First Reimplementation of Current Realized Scope

- **Status**: planned
- **Current floor**: OxCalc has an exercised .NET realization for the current TreeCalc Stage 1 scope, widened TraceCalc corpus, reference-machine comparisons, replay-facing adapter doctrine through W015, and an execution-sequenced Rust-first transition packet in W017.
- **Remaining gaps**: Rust crate layout, Rust implementation of the current structural/coordinator/recalc/TraceCalc surfaces, Rust test and artifact tooling, and parity/conformance evidence against the current exercised baseline.
- **Why still open**: the repo direction has shifted to Rust-first realization, but the current exercised implementation surface is still .NET-based.
- **Canonical owner**: W017.




