# IN_PROGRESS_FEATURE_WORKLIST.md — OxCalc

Canonical repo-level register of feature areas that are in-progress under workset completion doctrine.

Status: active.
Last updated: 2026-03-24.

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
- **Remaining gaps**: richer runtime-effect handling, broader overlay-economics reporting, replay-appliance bundle validation and explain emission, and later concurrency-facing widening.
- **Why still open**: the widened Stage 1 slice now exists, but later evidence, replay projection, and Stage 2-facing lanes remain partial.
- **Canonical owner**: W004.

### IP-05: OxFml Seam Hardening and Handoff Closure

- **Status**: in-progress
- **Current floor**: OxCalc-local seam requirements are drafted; `HANDOFF-CALC-001` is filed and acknowledged; the stronger OxFml downstream note and `HANDOFF-FML-001` are now also received locally, giving OxCalc a stronger minimum-schema, typed reject-context, runtime-effect, and host-boundary floor to consume.
- **Remaining gaps**: replay artifacts for candidate-result versus publication boundaries, broader runtime-derived effect taxonomy beyond the Stage 1 subset, exact trace-schema mapping, and any narrower follow-on handoff if exercised evidence later requires it.
- **Why still open**: the first bilateral seam round is incorporated, but the broader seam and replay-consumption area remains active and is now carried by a successor integration packet rather than by reopening W005.
- **Canonical owner**: W020.

### IP-06: Core Formalization and Gate Binding

- **Status**: in-progress
- **Current floor**: formalization and assurance direction is drafted, W006 is active, W007 contains the first Lean-facing object inventory and transition-boundary packet, W008 contains the first TLA+-oriented coordinator-state and safety-boundary packet plus explicit Stage 1 transition bindings, W009 contains the replay-class and pack-binding matrix through `R8`, W010 contains the experiment-register and measurement-schema packet, and the repo now includes Lean, TLA+, replay-seed, measurement-schema, emitted counter, and widened baseline run artifacts; the Lean state file has been typechecked locally and the TLA+ smoke model has been checked once with TLC.
- **Remaining gaps**: theorem authoring, richer TLA+ model exploration, replay-appliance bundle validation, pack artifact creation, and later retained-witness evidence.
- **Why still open**: the assurance lane now has real widened Stage 1 evidence, but its later proof, pack, and replay-appliance lanes remain partial.
- **Canonical owner**: W006.

### IP-07: Self-Contained Test Harness Planning

- **Status**: in-progress
- **Current floor**: `W011` now includes an exercised 12-scenario `TraceCalc` corpus, validator and runner paths under [src/oxcalc-tracecalc](/C:/Work/DnaCalc/OxCalc/src/oxcalc-tracecalc), a CLI host under [src/oxcalc-tracecalc-cli](/C:/Work/DnaCalc/OxCalc/src/oxcalc-tracecalc-cli), crate-local tests, and checked-in emitted baseline runs at `w013-sequence-a-baseline`, `w014-stage1-widening-baseline`, and `w017-rust-parity-baseline`.
- **Remaining gaps**: replay-appliance bundle export, richer retained-failure handling, larger generated-corpus lanes, and later OxFml-integrated harness coverage.
- **Why still open**: the widened self-contained harness slice is exercised, but later replay, retained-witness, and integrated-host lanes remain partial.
- **Canonical owner**: W011.

### IP-08: TraceCalc Reference Machine and Conformance Oracle

- **Status**: in-progress
- **Current floor**: `W012` now includes an executable `TraceCalc Reference Machine`, an engine adapter, conformance comparison logic, planner-driven DAG and SCC coverage, and checked-in emitted oracle/conformance baseline runs for both the original and widened 12-scenario corpora.
- **Remaining gaps**: richer trace comparison policy beyond the first conformance surface, replay-appliance bundle validation, reduced-witness flows, and later continuous engine-versus-oracle series beyond the current baseline runs.
- **Why still open**: the oracle is exercised over the widened Stage 1 slice, but later replay-appliance and retained-witness lanes remain partial.
- **Canonical owner**: W012.

### IP-09: Replay Appliance Adapter Rollout

- **Status**: in-progress
- **Current floor**: `W015` now has an execution-ready packet, local replay-coherence refactor direction, explicit adapter and capability-manifest docs, normalized event-family projection embodied in runner output, typed local mismatch projection fields, and replay-facing scenario metadata carried in the checked-in corpus and active widened baseline run.
- **Remaining gaps**: normalized replay-appliance bundle emission, bundle-validator conformance artifacts, explain records, capability promotion beyond the current conservative floor, and later pack-facing rollout.
- **Why still open**: local replay coherence and adapter doctrine are now in place, but the emitted-bundle and capability-promotion lane is now carried by `W018`.
- **Canonical owner**: W015.

### IP-10: Rust-First Reimplementation of Current Realized Scope

- **Status**: in-progress
- **Current floor**: the declared current realized scope now has a Rust execution path under `src/` covering the structural snapshot kernel, coordinator/publication baseline, recalc/overlay baseline, `TraceCalc` runner/reference-machine lane, and Rust CLI host, all validated under `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`, with a distinct emitted run at `w017-rust-parity-baseline` and a passing parity comparison against `w014-stage1-widening-baseline`.
- **Remaining gaps**: later Stage 2 and concurrency realization in Rust, replay-appliance bundle emission in Rust, retained-witness flows after W016, and later archival policy for superseded historical implementation artifacts.
- **Why still open**: W017 reached its final gate for the current declared scope, but the broader Rust-first feature area remains active for later widening, replay, and retained-witness lanes.
- **Canonical owner**: W017.

### IP-11: Witness Distillation and Retained Failure Packs

- **Status**: in-progress
- **Current floor**: `W016` has reached its declared gate with deterministic witness-seed artifacts, explicit lifecycle-state handling for `wit.generated_local`, `wit.explanatory_only`, `wit.quarantined`, one replay-valid retained-local witness family, a retained-failure fixture runner, and a checked-in retained-failure baseline run.
- **Remaining gaps**: candidate journals, richer witness bundles beyond scenario copies, additional retained-local mismatch families, later pack-facing promotion evidence, and any successor workset for broader retained-failure widening.
- **Why still open**: W016 has discharged its declared scope, but the broader retained-witness feature area remains active beyond this first retained-failure baseline.
- **Canonical owner**: W016.

### IP-12: Replay Appliance Bundle Emission and Capability Promotion

- **Status**: in-progress
- **Current floor**: `W019` has now reached its declared gate with additive replay-appliance bundle roots for ordinary and retained-failure runs, bundle-validator artifacts, explain records, checked-in ordinary and retained-failure baselines, replay-valid reduced-witness distillation artifacts, run-level `distill_validation.json`, and a refreshed capability claim through `cap.C4.distill_valid`. `W021` then converted the pack-grade gap into bounded emitted blockers, and `W022` added both a checked-in direct-binding-sensitive retained-local baseline in `w022-sequence1-direct-binding-family` and a checked-in retained-shared family baseline in `w022-sequence2-shared-lifecycle-family`, plus an explicit `pack_grade_decision.json` that keeps `cap.C5.pack_valid` unclaimed for the current semantic-only scope and defers any narrower handoff. `W023` then emitted explicit program-grade contract, validation, and decision sidecars across `w023-sequence1-program-scope-contract`, `w023-sequence2-host-sensitive-family`, and `w023-sequence3-program-decision`, keeping `cap.C5.pack_valid` unclaimed and packetizing the broader residual in `W024`.
- **Remaining gaps**: broader program-scope pack evidence beyond the current exercised host-sensitive `TraceCalc` family, broader mismatch-family explain coverage, any narrower handoff if later pack-grade promotion creates stronger seam pressure, and later pack-grade replay governance.
- **Why still open**: W023 reached its declared gate by making the broader program-grade capability and handoff decisions explicit, but the broader pack-promotion feature area remains active and is now carried by W024.
- **Canonical owner**: W024.

### IP-13: OxFml Downstream Integration Rounds

- **Status**: in-progress
- **Current floor**: OxCalc now has a local receipt for `HANDOFF-FML-001`, an outbound `NOTES_FOR_OXFML.md` reply, a returned OxFml topic-by-topic classification pass, the later OxFml clarification that the current OxFunc refinement adds no new OxCalc-facing seam trigger, a processed intake of `OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md` as the bounded host/runtime packet for the next coordinator-host round, and an explicit OxFml reply agreeing that the host/runtime packet is strong enough for the first implementation slice. W019 has now also exercised the previously bounded dependency-projection and semantic-display questions without producing a new formal seam trigger.
- **Remaining gaps**: later downstream rounds may still be needed if stronger coordinator pressure appears around execution-restriction transport, publication/topology consequence breadth, or caller-anchor/address-mode carriage for the first TreeCalc relative-reference subset. Those three residuals are now being carried as an explicit three-sequence W026 narrowing packet, and the latest OxFml reply keeps all three as `canonical but narrower` with no current handoff trigger. Availability/provider-failure and callable-publication remain watch lanes only.
- **Why still open**: round 01 is materially processed and the host/runtime packet is now consumed as a planning floor with explicit OxFml agreement, but the broader downstream integration feature area remains active for future OxFml/OxFunc pressure and any later narrower handoff.
- **Canonical owner**: W020.

### IP-14: TreeCalc Semantic Completion

- **Status**: in-progress
- **Current floor**: the target and execution line are now defined in `CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md`, packetized into `W025` through `W031`, and the first TreeCalc local implementation floor now exists in Rust: widened structural snapshots with formula/bind attachment points, relative-reference context, immutable structural edits, a TreeCalc-local formula and reference model, an OxFml-backed direct-host translation/bind/evaluate slice in the local sequential runtime, dependency graph build seeded from the translated OxFml bind preparation plus explicit residual carriers, verified-clean handling, coordinator-facing candidate adaptation and publication through the seam-backed path, a reusable widened minimal upstream host packet and adapter in `src/oxcalc-core/src/upstream_host.rs` for deterministic OxFml-facing automated scaffolding, richer typed host-info stand-ins, RTD stand-ins, in-memory runtime catalog snapshot carriage, first replay-capture packet projection, explicit crate-level scaffolding tests in `src/oxcalc-core/tests/upstream_host_scaffolding.rs`, a first checked-in upstream-host fixture corpus under `docs/test-fixtures/core-engine/upstream-host` that now also covers the agreed first table-context packet plus four bounded evaluator-facing structured-reference families, first local runtime-effect emission for host-sensitive and dynamic-reference families, a first local runtime-effect overlay carrier, a checked-in thirteen-case TreeCalc fixture corpus under `docs/test-fixtures/core-engine/treecalc`, a first emitted local run root at `docs/test-runs/core-engine/treecalc-local/w025-treecalc-local-baseline` with local oracle, conformance, trace, explain, and post-edit rebind, recalc-only, downstream dependency-chain, post-edit runtime-effect/overlay, mixed branch-sensitive seam behavior, move-triggered, and removal sidecars against fixture-declared expectations, and a first compare script at `scripts/compare-treecalc-local-run.ps1` so that local TreeCalc baselines are rerunnable and comparable rather than inspect-only.
- **Remaining gaps**: broader consumed OxFml bind/reference intake beyond the current direct-host slice, broader dependency graph build from richer bind products than the current translated name-backed carrier subset, runtime-derived effect closure beyond the first local rejection-sidecar and overlay floor, first TreeCalc corpus and baseline beyond the current local pre-oracle local-fixture shape, replay/diff/explain widening beyond the current local sidecars, broader structural-edit and successor-snapshot families beyond the current representative set, and assurance refresh.
- **Why still open**: the first seam-backed TreeCalc local pipeline now exists, but the broader first TreeCalc-ready engine scope and assurance lane are not realized yet.
- **Canonical owner**: W025.




