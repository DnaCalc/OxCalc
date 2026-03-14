# ARCHITECTURE_AND_REQUIREMENTS.md — DNA Calc Architecture and Requirements

## 1. Overview
DNA Calc is a near-formal spreadsheet system with two independent engines (Rust and .NET) sharing identical protocol surfaces and validated against a Green-owned spec stack: Lean (semantics proofs), TLA+ (concurrency protocol checks), OCaml oracle (executable reference), and conformance packs.

### 1.1 Three Hard Boundaries (core architectural shape)
1. **OpLog (Operations)**
   - All persistent state changes are operations (including structural edits, external updates, macro edits).
2. **DocSnapshot (Versioned Document State)**
   - Immutable snapshots per epoch/meta-epoch. Inputs are truth; derived values are caches.
3. **CalcDeltas (Derived Outputs)**
   - Engine produces deltas tagged with version info (epoch/value_epoch) and explicit stale/pending status.

## 2. Requirements Taxonomy (how to write requirements)
### 2.1 Architecture-independent requirements (REQ-)
Observable behaviors and quality targets, independent of internal mechanisms.

### 2.2 Architecture-dependent constraints (CONSTR-)
Enforceable structural rules derived from Mission/Doctrine.

### 2.3 Architecture-anchored intents and realizations (INT-/REAL-)
- **Intent (INT-)**: desired outcome, mechanism-agnostic.
- **Realization (REAL-)**: precise, testable specification anchored to chosen architecture.

## 3. System Architecture (A1)
This section now includes a formal-core layer model intended to be shared by Green proofs/models, the OCaml oracle, and both delivery engines.
Detailed core formal/semantic model development is maintained in `CORE_ENGINE_FORMAL_MODEL.md`.
This document should keep architecture-level summaries and stable constraints, with minimal duplication of formal-core detail.

### 3.1 Protocol Surface (identical across Red/Blue)
- Dispatch ops/transactions.
- Query snapshots/state.
- Subscribe to deltas/events.
- Capability negotiation.

Detailed core semantics are in `CORE_ENGINE_FORMAL_MODEL.md`:
- protocol baseline: Section `5.1`,
- operation/replay semantics: Section `6.4`.

### 3.2 Profiles, Feature Gates, and Compatibility
- Profiles remain the semantic spine (`profile_id` + `profile_version`) for compatibility, feature gates, obligation packs, and degrade behavior.
- Required profile-governed semantic selectors include:
  - `CycleSemantics = PriorValueFallback | CycleError | Iterative`,
  - `StreamSemanticsVersion = ExternalInvalidationV0 | TopicEnvelopeV1 | RtdLifecycleV2`,
  - optional scheduler policy selectors (for example `VisibleFirst`) that must preserve stabilized semantic equivalence.

Detailed core profile semantics are in `CORE_ENGINE_FORMAL_MODEL.md` Section `5.2`.

### 3.3 Epoch Model (MVCC-style)
- Core epoch semantics (`committed/stabilized/value epoch`, stale/pending visibility, pinning/GC constraints) remain mandatory.
- Core calc-delta derived-output semantics remain mandatory.

#### 3.3.1 Derived Publication Contract
- An accepted node commit publishes exactly one atomic derived bundle for that node.
- The bundle must include `value_delta`, `topology_delta`, `shape_delta`, and optional display/format deltas when profile-gated features are enabled.
- Rejected commits publish no derived deltas and must provide structured rejection detail sufficient for deterministic replay diagnostics.

Detailed definitions are in `CORE_ENGINE_FORMAL_MODEL.md` Section `5.3`.

### 3.4 Calculation Engine Pipeline (conceptual)
- Core pipeline remains parse/bind/dependency/invalidation/schedule/evaluate/commit with deterministic and incremental requirements.
- `NodeId`-based dependency identity remains baseline.
- Both invariant-oriented and dirty-closure propagation models remain valid formal baselines.

Detailed definitions are in `CORE_ENGINE_FORMAL_MODEL.md` Section `5.4`.

### 3.5 External Streaming and RTD-like Behavior
- Pathfinder: `STREAM("topic")` is acceptable and deterministic (epoch-scoped external provider).
- Full system: RTD support (topic lifecycle, updates, invalidations) is a core interop feature.
- External updates must appear as explicit `OpExternalUpdate` ops (`topic_id`, `topic_seq`, payload ref/envelope) and be replayable for test harnesses where required.
- STREAM/external update semantics include explicit topic identity, dedupe rules, ordering guarantees, and coalescing policy.
- Profile policy defines whether oracle values are local-only or shared for collaboration scenarios.
- A stream replay bundle (topic declarations, updates, timing/order envelope) is a required artifact for conformance and minimization.

#### 3.5.0 Stream Semantics Versioning
- `StreamSemanticsVersion` is profile-governed with three explicit values:
  - `ExternalInvalidationV0`: pathfinder-style externally-driven invalidation behavior.
  - `TopicEnvelopeV1`: topic/sequence envelope with deterministic ordering and dedupe replay behavior.
  - `RtdLifecycleV2`: full RTD-style topic lifecycle semantics.
- Profile claims must state which stream version is active and replay artifacts must be validated against that version.

#### 3.5.1 External Invalidation vs Volatile Invalidation
- External functions (`STREAM`, RTD, and profile-marked externally-invalidated UDFs) recalculate on explicit external signal, not on every volatile cycle.
- Volatile functions recalculate on invalidation cycles triggered by host policy.
- Distinct invalidation pathways are required:
  - volatile invalidation scope (global or class-based),
  - external invalidation scope (targeted by provider/topic/UDF identity).
- Both pathways converge on the same dirty-closure propagation and deterministic evaluation pipeline.

### 3.6 External UDFs / XLL-like integration
Pathfinder scope includes:
- external UDF registration/unregistration with explicit volatility class,
- externally driven invalidation hooks (`invalidate_udf`) that feed normal dirty-closure propagation,
- deterministic caller-driven execution (no autonomous internal timers/threads in pathfinder engine boundary),
- UDFs treated as pure-oracle from Lean/TLA+ perspective (semantics parameterized by oracle results).

Volatility classification:
- `Standard`: recalculates when upstream dependencies change.
- `Volatile`: recalculates on host invalidation cycle.
- `ExternallyInvalidated`: recalculates on explicit external signal.

Built-in and UDF volatility classification is profile-governed and must be reflected in capability/profile artifacts.

Full system adds:
- full XLL compatibility including marshalling/lifetime contracts and RTD integration.
- XLL is in-process with the engine; boundary contracts are formally specified and validated by Green packs.

### 3.6.1 Controls and Charts as Engine Entities
- Controls/charts are core engine entities participating in dependency semantics and operation/replay lifecycle.

Detailed core semantics are in `CORE_ENGINE_FORMAL_MODEL.md` Section `5.6`.

### 3.7 VBA and Macros (outside core)
- VBA runtime and editor live outside the core engine.
- Core engine stores the VBA project as a document object:
  - opaque blob (e.g., `vbaProject.bin`) + minimal metadata.
- Application layer glues:
  - file I/O ↔ engine (store/retrieve blob),
  - VBA runtime ↔ engine (macros emit ops via protocol),
  - macro execution occurs in an exclusive mutation mode (serialized event stream).

Windows-only COM automation is a separate facade layered on top of the identical protocol surface.

### 3.8 File I/O (outside core, full fidelity)
- File adapters are external components that translate to/from the object model and ops.
- For Excel interop:
  - preserve unknown/unsupported OOXML parts where feasible (opaque attachments),
  - never silently drop meaning on round-trip,
  - lowering pipeline may translate internal constructs to Excel-safe constructs or explicit loss markers,
  - degrade outcomes must be surfaced through diagnostics with policy class (`Native` / `Lowered` / `Opaque` / `Rejected`).

### 3.9 Collaboration (designed-in seam)
- Collaboration modeled as replication of the OpLog.
- Initial design preference: server-sequenced ops (deterministic shared log).
- Identity under structural edits is considered early (stable IDs where needed).
- Derived calc is generally local; external oracles (RTD) may be local or shared depending on profile policy.
- Replication envelope requires operation idempotency, causal ordering metadata, and transaction grouping.

### 3.10 UI Architecture (intended stack)
- Tauri shell with web UI.
- Grid rendering:
  - Canvas/WebGL for the giant grid (virtualized; no DOM-per-cell).
  - DOM overlay editor for Excel-grade editing (IME, selection, clipboard).
- UI state machine (“reducer” style) with explicit modes (selecting, editing, formula ref picking, fill, resize).
- Geometry spec and hit-test invariants.
- RenderPlan IR used for deterministic testing (avoid screenshot brittleness).
- View state is partially document-backed (saved view settings) and partially session state.
- UI reliability requires property-level invariants for geometry/hit-test consistency and deterministic RenderPlan validation.

### 3.10.1 Core and Non-core Interaction Summary
- UI/TUI layers read snapshots/deltas and dispatch explicit operations; no hidden persistent core mutation path is allowed.
- File adapters translate external formats to/from core entities and operations, and surface degradation outcomes explicitly.
- VBA/macro runtimes execute outside core and interact through operation pathways.
- Collaboration adapters replicate operation envelopes and preserve causality/idempotency constraints.

Formal-model note:
- Detailed formal semantics for the core engine model are in `CORE_ENGINE_FORMAL_MODEL.md`.
- Section `3.11` in this document provides a consolidated architecture summary with cross-references to that detailed model.

### 3.11 Core Formal Semantics Summary
Detailed formal semantics for the core engine model are maintained in `CORE_ENGINE_FORMAL_MODEL.md`. This section provides architecture-level summaries; consult the detailed model for formal definitions, type sketches, and invariant specifications.

Key areas and their detailed-model locations:

| Area | Summary | CORE_ENGINE_FORMAL_MODEL Section |
|---|---|---|
| State kernel | Immutable green core + ephemeral red facades; ID-based identity | `6.1` |
| Layered semantics | Five-layer `S/R/D/V/O` model with explicit derivation contracts | `6.2` |
| OpLog transitions | `OpEnvelope` carrier, `apply_op` transition, replay equivalence | `6.4` |
| Structural rewrites | Deterministic rewrite functions, explicit invalidation traces | `6.5` |
| Reference resolution | Normalized reference forms, forward/reverse indices, no silent drops | `6.3` |
| Cycles and iteration | Deterministic SCC order, profile-governed cycle mode, terminal rules | `6.6` |
| Formalization seams | Lean/OCaml/trace schema integration, `CoreIds`..`CoreOps` module split | `6.7` |

### 3.18 FEC/F3E Lane Boundary (OxFml/OxFunc/FEC)
To reduce ownership drift during implementation spikes, lane ownership is explicitly split as follows:
- **OxFml** owns formula grammar, parse, and bind semantics.
- **OxFunc** owns value-type and function semantics consumed during evaluation.
- **FEC host/model lane** owns host protocol, capability policy, dependency lifecycle, scheduler interaction, and publication lifecycle.

FEC/F3E spec ownership transfers to OxFml when that repo is created (Wave A); Foundation retains a read-only conformance mirror of the FEC/F3E specification artifacts for cross-reference and assurance use.
- Formula-semantic formatting behavior (including format-string interpretation and conditional-format configuration evaluation lanes) is evaluator work and must cross the FEC/F3E seam; it is not a display-only concern.
- FEC/F3E protocol definition authority: co-defined. OxFml defines the evaluator-side contract (session lifecycle, commit deltas, trace schema). OxCalc co-defines the coordinator-facing parts (publication fences, scheduling interaction, rejection policy). The shared protocol specification lives in OxFml as the spec owner, with OxCalc contributing coordinator-facing requirements through the cross-repo handoff process.
- Visibility-priority scheduling policy is core-engine policy and must preserve stabilized semantic equivalence; FEC/F3E provides evidence and deltas, not global scheduling truth.

Current working references:
- `reference/conformance/excel-worksheet-engine/model/fec-f3e/FEC_F3E_REDESIGN_SPEC.md`
- `reference/conformance/excel-worksheet-engine/model/fec-f3e/FEC_F3E_REDESIGN_OBSERVATIONS.md`
- `reference/conformance/excel-worksheet-engine/model/fec-f3e/FEC_F3E_PROTOCOL_CONFORMANCE_MATRIX.csv`

#### 3.18.1 End-to-End Data Flow Narrative (OxFml -> FEC/F3E -> OxCalc)
The core architectural data flow connects the formula evaluator to the multi-node coordinator through the FEC/F3E transactional seam:

1. **Formula text enters OxFml** — parse and bind produce a normalized formula AST with resolved references.
2. **OxFml evaluator executes single-node evaluation within an FEC session** — the session lifecycle is `prepare -> open_session/capability_view -> execute -> commit`.
3. **Accepted commit publishes one atomic derived bundle** — containing `value_delta` + `topology_delta` + `shape_delta` + optional `display_delta`/`format_delta` when profile-gated features are enabled.
4. **OxCalc coordinator consumes bundles** — for multi-node scheduling, dependency closure maintenance, epoch management, and stabilization tracking.
5. **Rejected commits produce no deltas** — structured reject detail (`reject_code` + typed context) flows back for diagnostics and deterministic replay.

This flow is the primary seam contract between the evaluator lane (OxFml) and the coordinator lane (OxCalc). Detailed session/commit semantics are in `CORE_ENGINE_FORMAL_MODEL.md` Sections `5.3` and `6.8`.

### 3.19 Core Engine Realization Plan (Option-B Staged Baseline)
The baseline realization strategy is:
1. immutable structural truth (`DocSnapshot`) and deterministic structural dependency derivation,
2. epoch-scoped runtime overlays (dynamic deps/spill/format tokens/visibility metadata),
3. FEC/F3E transactional seam for node-local evaluation publication,
4. single-publisher coordinator semantics for accepted/rejected commit publication.

Substrate progression:
- **Tree-only first (DNA TreeCalc):** multi-node calculation without grid concerns — no spill, no coordinates, no structural rewrites. Multi-result uses explicit node/value constructs. Tree-only is a proving strategy for coordinator/dependency/epoch semantics, not a separate doctrine.
- **Tree-grid-hybrid second (DNA PreCalc):** adds grid layer, spill semantics, structural rewrites, coordinate projection. Full reference resolution and grid-aware scheduling enter scope here.

Staged adoption:
- **Stage 1 (sequential coordinator):**
  - deterministic topo/SCC scheduling,
  - atomic commit bundle publication (`value_delta` + `topology_delta` + `shape_delta`),
  - conservative fallback allowed for unresolved spill/topology complexity.
- **Stage 2 (partitioned parallel evaluation):**
  - concurrent evaluator partitions with coordinator-controlled publication fences,
  - deterministic parallel signature packs required before policy promotion.
  - Stage 2 entry requires closure of FEC/F3E concurrency-hardening gates: (a) deterministic contention replay, (b) structured reject-detail for cross-engine replay, (c) coordinator snapshot/token fencing under concurrent evaluators.
- **Stage 3 (advanced policy lanes):**
  - optional dynamic-topology/SAC-inspired lanes and stream-heavy advanced lanes,
  - promotion only through parity/equivalence evidence and synthesis decisions.

The staged model is normative for execution planning:
- do not skip Stage 1 correctness closure,
- do not promote advanced lanes directly into baseline semantics.

### 3.20 Program Repo and Host Mapping
Architecture ownership map:
- **Foundation:** doctrine, architecture framing, operations and conformance policy.
- **OxFunc:** value/function semantics.
- **OxFml:** formula grammar/bind and single-node evaluator seam contracts. FEC/F3E spec owner (seam specification, evaluator contract, trace schema).
- **OxCalc:** multi-node core engine policy and execution. Co-defines coordinator-facing FEC/F3E protocol parts.
- **OxVba:** VBA runtime/compiler and host integration lane.
- **DnaVisiCalc:** Round 0 pathfinder evidence source.

Host progression map:
- `DNA VbCalc` (OxVba proving host),
- `DNA OneCalc` (single-node formula/function proving host),
- `DNA TreeCalc` (first serious OxCalc host on tree substrate),
- `DNA PreCalc` (first full tree-grid-hybrid host),
- later `DNA SuperCalc` and `DNA Calc` progression.

Interpretation rule:
- lane repos own reusable semantics/implementation lanes,
- host repos prove and compose lanes,
- Foundation remains doctrine owner.

## 4. Architectural Constraints (A2 / CONSTR- examples)
- **CONSTR-001:** All persistent mutations are ops; direct document mutation is forbidden outside the coordinator.
- **CONSTR-002:** File and network I/O are adapters outside core; core engine has no socket/file dependencies.
- **CONSTR-003:** Unsupported constructs never crash; they preserve or degrade explicitly.
- **CONSTR-004:** Protocol surfaces are identical across Red/Blue; compatibility negotiation is mandatory.
- **CONSTR-005:** Deterministic mode must exist and be used for conformance and minimization runs.
- **CONSTR-006:** Spec stack/oracle/tool integration contracts are file/CLI-based with schema-versioned artifacts.
- **CONSTR-007:** Compatibility claims require linked clean-room evidence records tied to REQ/INT/REAL identifiers.
- **CONSTR-008:** Engine lock discipline forbids awaiting or user-callback execution while holding mutation-critical locks.
- **CONSTR-009:** Performance readiness uses deterministic phase counters and published scaling signatures per required profile.
- **CONSTR-010:** Snapshot identity is ID-based (`RowId`/`ColId`/`CellId`), not coordinate-string-based; address projection is derived.
- **CONSTR-011:** Every structural op must define deterministic axis rewrite functions and explicit rewrite classification for affected references.
- **CONSTR-012:** Reference layer must model region nodes and error references explicitly; unresolved references cannot be silently discarded.
- **CONSTR-013:** Cycle handling mode (`PriorValueFallback` / `CycleError` / `Iterative`) is profile-defined and deterministic with explicit terminal behavior.
- **CONSTR-014:** Op envelopes require idempotency/causality metadata sufficient for deterministic replay and replication safety.
- **CONSTR-015:** Green/Red persistence-facade split must preserve immutable core semantics and avoid hidden mutation in facade caches.
- **CONSTR-016:** Function invalidation classes (`Standard` / `Volatile` / `ExternallyInvalidated`) must have deterministic, non-ambiguous trigger semantics per profile.
- **CONSTR-017:** Control/chart lifecycle mutations must be represented as explicit operations and replay artifacts, never as UI-only hidden state.
- **CONSTR-018:** Accepted commit publication is atomic at node-bundle granularity; rejected commits publish no derived deltas.
- **CONSTR-019:** Rejected commits must carry structured rejection detail (`reject_code` + typed context) for deterministic replay and migration diagnostics.
- **CONSTR-020:** Runtime overlay reuse must be keyed by epoch/token/bind/profile fences; stale overlays must be evicted deterministically under epoch-safe GC rules.
- **CONSTR-021:** `VisibleFirst` scheduling is optional and must enforce deterministic ordering plus bounded non-visible starvation.
- **CONSTR-022:** Formatting semantics that influence formula evaluation must be modeled through evaluator/seam contracts, not inferred from renderer state.
- **CONSTR-023:** Core semantic truth is profile-defined and must remain invariant under runtime strategy choices (scheduler policy, parallelism shape, incremental algorithm, or optimization mode).
- **CONSTR-024:** Required profiles must emit portable replay bundles plus structured forensic traces sufficient for deterministic causality diagnosis and cross-engine differential triage.
- **CONSTR-025:** Core runtime overlays are epoch-scoped derived state; overlay lifecycle, retention, and eviction must be deterministic and pinned-epoch safe.
- **CONSTR-026:** Coordinator publication authority is single-publisher at baseline; concurrent evaluators may not bypass coordinator commit fences.
- **CONSTR-027:** Host/repo composition may evolve, but lane ownership boundaries (OxFunc/OxFml/OxCalc/OxVba) cannot be collapsed without synthesis-approved architecture edits.

## 5. Core Requirements (REQ- and INT-/REAL- examples)
### REQ (architecture-independent)
- Excel interop: load/save macro-enabled workbooks with no unexpected loss; preserve VBA project unless explicitly edited.
- Manual and auto recalc behaviors must match the profile definition.
- Streaming updates propagate to dependents; system exposes progress and staleness.
- UI remains responsive under defined workloads (scrolling/edit feedback targets per profile).
- System never crashes on unsupported features; must yield deterministic errors/warnings or preserve opaque.
- Structural edits must preserve or invalidate references deterministically with explicit diagnostics and replayable rewrite traces.
- OpLog replay of an accepted operation sequence must reproduce equivalent snapshot/value states across engines.
- Cycle behavior (prior-value fallback, error, or iterative) must be observable, deterministic, and profile-consistent.
- Reference-grid updates must be incrementally maintained and auditable after every structural or formula mutation.
- CalcDelta outputs must be epoch-tagged, typed, and observationally consistent with committed snapshot transitions.

### INT/REAL (architecture-anchored)
- **INT:** Users can trust what they see during recalculation.  
  **REAL:** Every value carries `value_epoch` and explicit stale/pending status in UI/API.
- **INT:** Custom features must not break other builds.  
  **REAL:** Unknown extension payloads round-trip; unsupported semantic extensions evaluate to explicit deterministic errors and emit diagnostics.
- **INT:** STREAM/external updates must be predictable and replayable across engines.  
  **REAL:** External updates are explicit OpLog operations with versioned stream semantics, deterministic replay bundles, and pack-validated ordering/dedupe behavior.
- **INT:** Volatile and externally-signaled recalculation must not be conflated.  
  **REAL:** Profiles classify functions as `Standard` / `Volatile` / `ExternallyInvalidated` with explicit invalidation triggers and deterministic dirty-scope behavior.
- **INT:** UI correctness must be testable without screenshot dependence.  
  **REAL:** Geometry/hit-test invariants and RenderPlan determinism are required and pack-gated.
- **INT:** Performance claims must be trend-checkable, not anecdotal.  
  **REAL:** Required profiles publish deterministic phase counters and slope-based scaling signatures with regression thresholds.
- **INT:** Cross-engine disagreements must be quickly explainable and reproducible.  
  **REAL:** Required profiles maintain differential execution lanes across Red/Blue/oracle surfaces and publish indexed divergence artifacts with replay handles.
- **INT:** Clean-room compatibility claims must be auditable.  
  **REAL:** Every compatibility claim links to admissible evidence records and review status.
- **INT:** Structural change semantics must be predictable and formally checkable.  
  **REAL:** Structural ops produce deterministic axis rewrite mappings plus per-reference rewrite classification artifacts.
- **INT:** Reference resolution ambiguity must be bounded and diagnosable.  
  **REAL:** Binder outputs normalized references (`CellRef`/`RegionRef`/`NameRef`/`ErrorRef`) and explicit unresolved diagnostics.
- **INT:** The formal core must be implementable consistently in Lean, OCaml, Rust, and .NET.  
  **REAL:** Shared algebraic data schemas and transition traces are normative artifacts for proofs, oracle runs, and engine conformance.
- **INT:** Cycles should not produce hidden nondeterminism.  
  **REAL:** SCC decomposition order, iteration bounds, convergence policy, and terminal-state rules are profile-governed and replayable.

## 6. Pathfinder Scope Anchor (DnaVisiCalc)
Round 0 pathfinder functional scope is authoritative in the DnaVisiCalc docs set:
- `..\\DnaVisiCalc\\docs\\SPEC_v0.md`
- `..\\DnaVisiCalc\\docs\\ENGINE_REQUIREMENTS.md`
- `..\\DnaVisiCalc\\docs\\ENGINE_API.md`

Foundation documents define doctrine/architecture/process and must remain consistent with that authoritative functional scope.

### 6.1 Round 0 Normative Contract (minimum)
- Required functional scope baseline (frozen for pathfinder v0) includes:
  - externally driven single-sheet engine contract with default `A1..BK254` bounds,
  - cell/name inputs, deterministic formula evaluation, dynamic arrays/spill behavior,
  - epoch model (`committed_epoch`, `stabilized_epoch`, `value_epoch`) and manual/auto recalc,
  - deterministic row/column structural rewrites with explicit invalidation behavior,
  - profile-governed cycle handling with explicit v0-compatible `PriorValueFallback` behavior and optional iterative mode configuration,
  - stream behavior declared by `StreamSemanticsVersion`, with pathfinder v0 mapping to `ExternalInvalidationV0`,
  - three-class invalidation model (`Standard`, `Volatile`, `ExternallyInvalidated`) with distinct trigger paths,
  - external UDF registration/invalidation, engine-managed controls/charts, typed change-tracking journal,
  - metadata-only formatting and deterministic bulk enumeration sufficient for current file-adapter handoff,
  - TUI interaction/testing scope (editing/commands plus deterministic replay/capture surfaces) as defined by upstream pathfinder docs.
- Required obligations: core semantics packs, epoch/concurrency invariants, oracle alignment, and basic scaling signature.
- Required artifacts: capability manifest, conformance report, minimized trace corpus, replay bundles for stream cases, and formal-core traces (structural rewrite + reference-grid delta + SCC iteration).

### 6.2 Explicit Non-goals for Round 0
- Multi-sheet workbook semantics.
- Full number-format mini-language and full date/time serial compatibility system.
- Full Excel coercion parity and implicit-intersection (`@`) compatibility breadth.
- Lambda-helper family completeness beyond the stabilized pathfinder subset.
- Full XLL marshalling/lifetime compatibility.
- Full RTD lifecycle parity.
- Full OOXML fidelity breadth outside the pathfinder subset.
- VBA runtime hosting.
- Multi-writer collaboration semantics beyond seam validation.

### 6.3 Round 0 Track Status Decomposition (Pathfinder Feedback Snapshot)
As of **February 27, 2026**, synthesis of DnaVisiCalc pathfinder feedback indicates:

- Track A — Engine implementation scope:
  - status: functional scope is stabilized and exercised against the authoritative v0 spec/requirements/API contract.
- Track B — Green formal artifacts and assurance packs:
  - status: remains the principal Round 0 exit blocker (Lean/TLA+/oracle/pack artifacts still required by doctrine).
- Track C — beyond-minimum artifacts:
  - status: design/API artifacts exist and should be treated as evidence inputs for Round 1 shaping, not as Round 0 gate substitutes.

Round 0 exit remains blocked until required Track B obligations are completed, regardless of Track A progress.

### 6.4 Deferred Functional Expansion Backlog (Retained From Pathfinder Gap Analysis)
The following areas are retained as explicit post-freeze expansion candidates, not pathfinder-v0 functional-scope requirements:
- comprehensive number-format code language behavior,
- full date/time serial compatibility system,
- full coercion-matrix parity and implicit-intersection behavior,
- multi-sheet references and workbook semantics,
- broader lambda-helper family coverage.

## 7. Rounds 1–3 Forward Compatibility
DnaVisiCalc must already validate the meta-architecture and the discipline that enables:
- DnaPreCalc to expand feature surface without abandoning proofs/packs,
- DnaSuperCalc to explore deeper refactors and extensibility,
- DnaCalc to synthesize a maintainable, optimized foundation for long-term evolution.

### 7.1 Forward Execution Vehicles
Round-compatible host progression for implementation planning:
1. `DNA OneCalc`: fast single-node proving host for OxFml/OxFunc (optional OxVba integration). Proves formula language completeness, OxFunc function semantics, and UDF/VBA host integration on a single-cell or defined-name substrate — no reference resolution, no multi-node scheduling. Clean-room evaluator proving ground separate from DnaVisiCalc pathfinder.
2. `DNA TreeCalc`: first serious multi-node proving host for OxCalc on tree substrate.
3. `DNA PreCalc`: first integrated tree-grid-hybrid host aligned with Round 1 scope.
4. `DNA SuperCalc`: later refinement/perfection host stage.
5. `DNA Calc`: final full host/product realization.

### 7.2 Dependency-Conscious Progression Rule
Architecture progression must respect dependency constitution:
1. OxFunc remains dependency-light semantic base.
2. OxFml consumes OxFunc and exposes evaluator contracts.
3. OxCalc consumes OxFml/OxFunc and owns multi-node execution policy.
4. Host repos compose lanes; they do not redefine lane ownership or doctrine.
