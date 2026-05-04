# W033 Source Authority And Artifact Layout

Status: `calc-uri.1_entry_freeze`
Workset: `W033`
Parent epic: `calc-uri`
Bead: `calc-uri.1`
Created: 2026-05-04

## 1. Purpose

This packet freezes the source authority set and declares the W033 artifact layout before the later formalization ledgers, models, replay bridges, and closure audits are emitted.

The freeze is an evidence index, not a semantic promotion. Current specs, current implementation behavior, historical documents, and upstream OxFml artifacts remain evidence surfaces that W033 will review, classify, patch, hand off, defer, or reject with rationale.

## 2. Repository Snapshot

| Repository | Role in W033 | Entry revision or worktree basis | Worktree state at freeze | Rule |
|---|---|---|---|---|
| `OxCalc` | Owner of W033, core-engine specs, coordinator/runtime/formal/replay artifacts, and bead execution state. | `263355e2bdaf0be69acaa31c88b31fab7ca68932` | clean before this packet was authored; branch `main` was ahead of `origin/main` by one local commit | OxCalc-owned specs and artifacts may be patched by W033. |
| `OxFml` | Read-only upstream owner for evaluator, formula-language, FEC/F3E, trace, replay, and seam surfaces consumed by OxCalc. | `7f5cd5f8af8c7b0bc5d8a82d5036b4fcd32e0240` | clean | W033 may cite and model OxFml surfaces, but must not patch OxFml directly. |
| `Foundation` | Higher-precedence doctrine and architecture authority. | `d079b66446493373d57cff39b47b7d54b6a94066` plus current working-tree doctrine files | dirty worktree outside this repo | W033 reads Foundation doctrine as authority. Dirty Foundation files are frozen here by file hash for auditability. |

Foundation doctrine file hashes used by this freeze:

| File | SHA-256 |
|---|---|
| `../Foundation/CHARTER.md` | `AE6E8ADF34C35954A5CC2B3E663825281D19AB7B9107D340A9B475DDBF9E919A` |
| `../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md` | `CD6828A870A4175D8F17006296D389BCE363A0EBCF3D5E96F22ED082B05744EC` |
| `../Foundation/OPERATIONS.md` | `6A882E65D68AC95F416FB8493C62B2B0210D01D3174723D6DEBE035F47364A5E` |
| `AGENTS.md` | `A3B3B709682E87E12EA6EE7F0CF505EFAA438BEE7030F94544D8ED44FEA755DE` |

## 3. Source Authority Inventory

### 3.1 Doctrine And Execution Sources

| Source | Authority role | W033 handling |
|---|---|---|
| `README.md` | Repo orientation and startup context. | Entry context. |
| `CHARTER.md` | OxCalc-local charter under Foundation precedence. | Authority input. |
| `OPERATIONS.md` | OxCalc-local operating model, gates, completion doctrine, and handoff process. | Authority input. |
| `docs/WORKSET_REGISTER.md` | Canonical ordered workset surface. | W033 register entry and closure condition source. |
| `docs/BEADS.md` | Canonical local bead method. | Bead mutation and closure rule source. |
| `docs/worksets/README.md` | Workset index and provenance entry point. | Index input. |
| `docs/spec/README.md` | OxCalc spec index and source classification surface. | Link target for this W033 artifact root. |
| `docs/IN_PROGRESS_FEATURE_WORKLIST.md` | Compact live feature map. | High-level status input. |
| `.beads/` via `br` | Live execution-state truth. | Mutated only through `br`; never edited directly. |

### 3.2 OxCalc Core-Engine Spec Sources

| Source | W033 role |
|---|---|
| `docs/spec/core-engine/CORE_ENGINE_ARCHITECTURE.md` | Core architecture and object model review surface. |
| `docs/spec/core-engine/CORE_ENGINE_STATE_AND_SNAPSHOTS.md` | State/snapshot vocabulary and structural/runtime separation review surface. |
| `docs/spec/core-engine/CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md` | Incremental recalc, invalidation, stabilization, and dependency review surface. |
| `docs/spec/core-engine/CORE_ENGINE_OVERLAY_AND_DERIVED_RUNTIME.md` | Overlay, derived runtime state, pinning, retention, and runtime-effect review surface. |
| `docs/spec/core-engine/CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md` | Candidate, reject, accept, publication, fence, and coordinator transition review surface. |
| `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md` | OxCalc-owned companion for consumed OxFml seam facts. |
| `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md` | Existing formalization and assurance authority surface to be sharpened by W033. |
| `docs/spec/core-engine/CORE_ENGINE_REALIZATION_ROADMAP.md` | Realization sequencing and capability positioning review surface. |
| `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md` | TraceCalc reference-machine and correctness-oracle authority surface. |
| `docs/spec/core-engine/CORE_ENGINE_TEST_HARNESS_AND_FIXTURES.md` | Fixture and harness review surface. |
| `docs/spec/core-engine/CORE_ENGINE_TEST_SCENARIO_SCHEMA_AND_TRACECALC.md` | TraceCalc scenario-schema review surface. |
| `docs/spec/core-engine/CORE_ENGINE_TEST_VALIDATOR_AND_RUNNER_CONTRACT.md` | Validator/runner contract review surface. |
| `docs/spec/core-engine/CORE_ENGINE_REPLAY_APPLIANCE_ADAPTER.md` | Replay-appliance projection and witness-lifecycle review surface. |
| `docs/spec/core-engine/CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md` | TreeCalc-first runtime and residual planning review surface. |
| `docs/spec/core-engine/CORE_ENGINE_TREECALC_ASSURANCE_AUTHORITY_MAP.md` | W031 assurance/residual mapping input. |
| `docs/spec/core-engine/CORE_ENGINE_TREECALC_OXFML_SEAM_NEGOTIATION_MATRIX.md` | Temporary seam-negotiation tracker to be treated as supporting input, not seam authority. |
| `docs/spec/core-engine/CORE_ENGINE_OXFML_MINIMAL_UPSTREAM_HOST_INTERFACES.md` | Exercised upstream-host packet companion. |
| `docs/spec/core-engine/CORE_ENGINE_DOWNSTREAM_HOST_SEAM_REFERENCE.md` | Downstream seam-reference classification input. |
| `docs/spec/core-engine/CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md` | Host-facing TreeCalc runtime contract input. |
| `docs/spec/core-engine/CORE_ENGINE_OPLOG_UNDO_REDO_AND_COLLAB_ARCHITECTURE_PLAN.md` | Operation-model companion; included only where replay/export or state-transition vocabulary intersects W033. |

### 3.3 Historical No-Loss Inputs

These are historical review inputs, not current authority unless W033 promotes a recovered idea into current OxCalc-owned specs or records it as a deferred lane.

| Source | W033 role |
|---|---|
| `docs/spec/core-engine/CORE_ENGINE_FORMAL_MODEL.md` | Current redirect/reference surface for older formal model ideas. |
| `docs/spec/core-engine/CORE_ENGINE_THEORY_AND_ALTERNATIVE_PATHS.md` | Current redirect/reference surface for older theory and alternative-path ideas. |
| `docs/spec/core-engine/archive/bootstrap-2026-03/CORE_ENGINE_FORMAL_MODEL.md` | Original formal-model input for no-loss crosswalk. |
| `docs/spec/core-engine/archive/bootstrap-2026-03/CORE_ENGINE_THEORY_AND_ALTERNATIVE_PATHS.md` | Original theory/alternatives input for no-loss crosswalk. |
| `docs/spec/core-engine/archive/rewrite-control-2026-03/SPEC_REWRITE_DOCUMENT_MAP.md` | Rewrite-control provenance input. |
| `docs/spec/core-engine/archive/rewrite-control-2026-03/REWRITE_PROMOTION_LEDGER.md` | Promotion/defer/reject provenance input. |
| `docs/spec/core-engine/archive/rewrite-control-2026-03/SPEC_REWRITE_PLAN.md` | Rewrite sequencing provenance input. |

### 3.4 OxCalc Formal, Replay, And Measurement Floors

| Source | W033 role |
|---|---|
| `formal/README.md` | Existing formal root index. |
| `formal/lean/OxCalc/CoreEngine/Stage1State.lean` | Lean Stage 1 floor to widen. |
| `formal/tla/CoreEngineStage1.tla` | TLA+ Stage 1 floor to widen. |
| `formal/tla/CoreEngineStage1.cfg` | Non-smoke TLA+ Stage 1 config. |
| `formal/tla/CoreEngineStage1.smoke.cfg` | Smoke TLA+ Stage 1 config. |
| `formal/replay/stage1-hand-authored/` | Hand-authored replay floor for Stage 1 invariants. |
| `formal/measurement/` | Measurement and counter-schema floor. |
| `docs/test-corpus/core-engine/tracecalc/` | Checked-in TraceCalc scenario corpus. |
| `docs/test-runs/core-engine/tracecalc-reference-machine/` | Existing TraceCalc run evidence. |
| `docs/test-runs/core-engine/treecalc-local/` | Existing TreeCalc local evidence. |
| `docs/test-runs/core-engine/treecalc-scale/` | Existing TreeCalc scale evidence. |

### 3.5 OxFml Read-Only Inputs

| Source | W033 role |
|---|---|
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | Inbound OxFml observation ledger for OxCalc. |
| `../OxFml/docs/spec/OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md` | Public consumer facade contract input. |
| `../OxFml/docs/spec/OXFML_FORMALIZATION_AND_VERIFICATION.md` | OxFml formalization plan input. |
| `../OxFml/docs/spec/OXFML_FORMAL_ARTIFACT_REGISTER.md` | OxFml formal artifact register input. |
| `../OxFml/docs/spec/OXFML_DELTA_EFFECT_TRACE_AND_REJECT_TAXONOMIES.md` | Delta/effect/trace/reject taxonomy input. |
| `../OxFml/docs/spec/OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md` | Host runtime and external-requirement input. |
| `../OxFml/docs/spec/OXFML_FIXTURE_HOST_AND_COORDINATOR_STANDIN_PACKET.md` | Fixture-host and coordinator-standin input. |
| `../OxFml/docs/spec/fec-f3e/FEC_F3E_DESIGN_SPEC.md` | FEC/F3E design authority input. |
| `../OxFml/docs/spec/fec-f3e/FEC_F3E_FORMAL_AND_ASSURANCE_MAP.md` | FEC/F3E assurance input. |
| `../OxFml/docs/spec/fec-f3e/FEC_F3E_TESTING_AND_REPLAY.md` | FEC/F3E testing/replay input. |
| `../OxFml/docs/spec/formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md` | OxFunc-opaque boundary input. |
| `../OxFml/docs/spec/formula-language/OXFML_OXFUNC_LET_LAMBDA_PIN_DOWN_PREP.md` | Narrow LET/LAMBDA carrier-fragment input. |
| `../OxFml/docs/spec/formula-language/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md` | Prepared-call and shared-interface input. |
| `../OxFml/formal/lean/*.lean` | Read-only Lean input. |
| `../OxFml/formal/tla/*.tla` and `*.cfg` | Read-only TLA+ input. |
| `../OxFml/crates/oxfml_core/tests/fixtures/*` | Read-only replay and witness fixture input, including retained witness indexes and LET/LAMBDA/higher-order callable cases. |

## 4. Declared W033 Artifact Layout

### 4.1 New W033 Spec-Evidence Root

W033 declares this new checked-in artifact root:

`docs/spec/core-engine/w033-formalization/`

This root is for W033's review ledgers, authority matrices, refinement packets, handoff/watch packets, closure audit, and other spec-evidence artifacts. It is OxCalc-owned and may cite OxFml by path and revision, but it must not copy OxFml canonical documents as if they were OxCalc-owned truth.

Planned W033 packet names under this root:

| Planned packet | Owning bead |
|---|---|
| `W033_SOURCE_AUTHORITY_AND_ARTIFACT_LAYOUT.md` | `calc-uri.1` |
| `W033_CORE_SPEC_REVIEW_LEDGER.md` | `calc-uri.2` |
| `W033_SPEC_EVOLUTION_DECISION_LEDGER.md` | `calc-uri.3` |
| `W033_HISTORICAL_NO_LOSS_CROSSWALK.md` | `calc-uri.4` |
| `W033_AUTHORITY_AND_CLAIM_MATRIX.md` | `calc-uri.5` |
| `W033_OBJECT_VOCABULARY_AND_LET_LAMBDA_BOUNDARY.md` | `calc-uri.6` |
| `W033_TRACECALC_REFINEMENT_PACKET.md` | `calc-uri.7` |
| `W033_TRACECALC_ORACLE_SELF_CHECK_FIRST_SLICE.md` | `calc-uri.8` |
| `W033_PRODUCTION_CONFORMANCE_FIRST_SLICE.md` | `calc-uri.9` |
| `W033_METAMORPHIC_DIFFERENTIAL_TEST_FAMILIES.md` | `calc-uri.10` |
| `W033_LEAN_MODULE_FAMILY_FIRST_SLICE.md` | `calc-uri.11` |
| `W033_TLA_BRIDGE_FIRST_SLICE.md` | `calc-uri.12` |
| `W033_REPLAY_WITNESS_BRIDGE.md` | `calc-uri.13` |
| `W033_PACK_CAPABILITY_BINDING.md` | `calc-uri.14` |
| `W033_OXFML_HANDOFF_WATCH_PACKET.md` | `calc-uri.15` |
| `W033_CLOSURE_AUDIT_AND_SUCCESSOR_PACKET.md` | `calc-uri.16` |

### 4.2 Existing Artifact Roots Retained

W033 does not move existing formal or replay roots. Later beads may add artifacts to these existing roots when the artifact type belongs there:

| Existing root | W033 use |
|---|---|
| `formal/lean/` | OxCalc Lean module-family widening. |
| `formal/tla/` | OxCalc TLA+ bridge widening and model-check configs. |
| `formal/replay/` | Small hand-authored replay facts tied to formal invariants. |
| `formal/measurement/` | Measurement schemas and experiment registers. |
| `docs/test-corpus/core-engine/tracecalc/` | TraceCalc scenario corpus. |
| `docs/test-runs/core-engine/tracecalc-reference-machine/` | TraceCalc reference-machine run evidence. |
| `docs/test-runs/core-engine/treecalc-local/` | TreeCalc local run evidence. |
| `docs/test-runs/core-engine/treecalc-scale/` | TreeCalc scale/performance run evidence. |
| `docs/handoffs/` and `docs/handoffs/HANDOFF_REGISTER.csv` | OxFml handoff packets if W033 finds concrete seam pressure requiring upstream acknowledgement. |

Any new root beyond these requires an explicit W033 artifact-layout update before artifacts are emitted there.

## 5. Validation And Tooling Baseline

### 5.1 Commands Used For This Freeze

| Command | Result at freeze |
|---|---|
| `git rev-parse HEAD` | `263355e2bdaf0be69acaa31c88b31fab7ca68932` |
| `git status --short` | clean before this packet was authored |
| `git -C ..\OxFml rev-parse HEAD` | `7f5cd5f8af8c7b0bc5d8a82d5036b4fcd32e0240` |
| `git -C ..\OxFml status --short` | clean |
| `git -C ..\Foundation rev-parse HEAD` | `d079b66446493373d57cff39b47b7d54b6a94066` |
| `git -C ..\Foundation status --short` | dirty; doctrine files frozen by hash in Section 2 |
| `rg --files formal` | existing OxCalc formal/replay/measurement roots listed successfully |
| `rg --files ..\OxFml\formal ..\OxFml\docs\spec ..\OxFml\docs\upstream ..\OxFml\crates\oxfml_core\tests\fixtures` | OxFml read-only formal/spec/fixture roots listed successfully |

### 5.2 Available Local Tooling

| Tool | Observed version or availability | W033 use |
|---|---|---|
| PowerShell | `7.6.1` | Repository validation and script execution. |
| `br` | `0.1.34` | Bead mutation. |
| `bv` | `v0.15.2`; `--version` also reports an option-handling message | Graph inspection. |
| `cargo` | `1.94.1 (29ea6fb6a 2026-03-24)` | Rust build and test validation. |
| `rustc` | `1.94.1 (e408947bf 2026-03-25)` | Rust compiler baseline. |
| `lean` | `4.29.1` | Lean formal artifacts. |
| `lake` | `5.0.0-src+f72c35b` with Lean `4.29.1` | Lean project runner if a later bead introduces/uses Lake. |
| `java` | OpenJDK `21.0.10` LTS | TLA+ TLC runner. |
| TLC jar | `../OxFml/formal/tools/tla2tools.jar`, reports `TLC2 Version 2.19 of 08 August 2024` before rejecting unsupported `-version` | TLA+ model checking through repo scripts or direct invocation. |

### 5.3 Validation Commands For Later W033 Gates

Later beads should use the subset relevant to their touched surfaces and record actual outcomes in their packet:

```powershell
git status --short --branch
br ready --json
br dep cycles --json
powershell -ExecutionPolicy Bypass -File scripts/check-worksets.ps1
cargo test --workspace
lean formal/lean/OxCalc/CoreEngine/Stage1State.lean
powershell -ExecutionPolicy Bypass -File scripts/run-tlc.ps1 -Spec formal/tla/CoreEngineStage1.tla -Config formal/tla/CoreEngineStage1.smoke.cfg
powershell -ExecutionPolicy Bypass -File scripts/compare-tracecalc-run.ps1
powershell -ExecutionPolicy Bypass -File scripts/compare-treecalc-local-run.ps1
```

W033 does not require every command on every bead. The closure audit must state which commands were relevant, which were run, which were skipped, and why.

## 6. Status

- execution_state: `source_authority_and_layout_packet_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - W033 source/link validation still needs to run after this packet is linked from W033 truth surfaces
  - W033 ledger, matrix, formal, replay, conformance, pack, handoff, and closure packets are not yet authored
  - no OxFml handoff is filed by this packet
  - no new Lean, TLA+, Rust, or replay artifact is emitted by this packet
