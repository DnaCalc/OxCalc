# CORE_ENGINE_OPLOG_UNDO_REDO_AND_COLLAB_ARCHITECTURE_PLAN.md

## 1. Purpose and Status
This document defines the OxCalc-centered architecture, ownership split, design direction, and staged work plan for:
1. the operation model (`OpLog` / operations),
2. undo and redo,
3. live and later concurrent editing,
4. replay export and `OxReplay` integration for operation-derived evidence.

Status:
1. active OxCalc-owned architecture and planning companion,
2. intended to clarify positioning and implementation ownership before broad code movement begins,
3. not a frozen wire protocol or storage-format commitment,
4. broad enough to cover required cross-repo update areas.

## 2. Why This Exists
Foundation doctrine already says:
1. all persistent state changes flow through the operation model,
2. no hidden mutation pathways are allowed,
3. collaboration is modeled as replication of the `OpLog`,
4. replay must remain deterministic and adapter-governed.

But the current repo topology does not yet contain an implemented `OpLog` realization or a clear code-owner packet for undo/redo and live-editing foundations.

This document answers that gap from the OxCalc perspective.

## 3. Resulting Rule
The resulting architecture rule is:
1. Foundation owns the `OpLog` doctrine and constraints,
2. OxCalc owns the first executable realization of the operation model, sequencing, publication, and collaboration substrate,
3. undo and redo are OxCalc features built on the same operation path rather than host-local hidden mutation stacks,
4. `OxReplay` consumes operation-derived replay artifacts through declared adapters and normalized bundle contracts; it does not become the mutation authority,
5. spreadsheet hosts submit intent through OxCalc-owned operation pathways rather than mutating persistent state directly.

## 4. Scope
This document covers:
1. repo ownership and topology for the operation model,
2. target runtime architecture,
3. proposed OxCalc-internal crate and module decomposition,
4. undo/redo design direction,
5. live/collaborative editing design direction,
6. replay/export implications,
7. staged work planning and cross-repo update obligations.

It does not:
1. redefine Foundation doctrine,
2. redefine OxFml evaluator semantics,
3. freeze a final network protocol,
4. claim implemented capability by itself.

## 5. Ownership Split By Repo

### 5.1 Foundation
Foundation owns:
1. the doctrine that all persistent mutation is operation-based,
2. architectural constraints on replayability, explicit mutation, and collaboration semantics,
3. cross-program protocol and conformance framing,
4. pack and promotion policy.

Foundation does not own the executable Rust realization.

### 5.2 OxCalc
OxCalc owns:
1. operation application to structural truth,
2. transaction sequencing and coordinator authority,
3. acceptance, rejection, fence checks, and publication,
4. snapshot advancement and observer-visible stability,
5. undo/redo semantics over accepted operations,
6. collaboration/replication substrate for core-engine mutation,
7. export of operation-derived replay artifacts for downstream Replay use.

This follows OxCalc's existing ownership of coordinator behavior, publication lifecycle, and staged concurrency.

### 5.3 OxReplay
`OxReplay` owns:
1. replay bundle validation and indexing,
2. normalized replay runtime and host surfaces,
3. diff, explain, witness, and pack export machinery,
4. adapter SDK and conformance harnesses.

`OxReplay` does not own:
1. the primary `OpLog`,
2. core mutation semantics,
3. coordinator publication policy,
4. undo/redo truth,
5. live-editing authority.

### 5.4 Other Lane Repos
Other repos keep their existing roles:
1. `OxFml` remains evaluator-side semantic and protocol authority for formula-language and single-node evaluator products,
2. `OxFunc` remains function semantic-kernel owner,
3. `OxVba` remains VBA runtime/compiler and mutation-intent source for VBA-driven editing,
4. `OxXlPlay` remains observation or driving evidence source rather than the owner of persistent document mutation semantics.

## 6. Target Architecture

### 6.1 Mutation Spine
The intended core mutation pipeline is:
1. host emits user or automation intent,
2. intent becomes one or more typed operations,
3. operations are wrapped in a transaction or operation envelope with basis/fence metadata,
4. OxCalc validates and applies the operations against immutable structural truth,
5. OxCalc derives invalidation, dependency, evaluation, and publication consequences,
6. the coordinator accepts or rejects the candidate outcome,
7. accepted work publishes atomically as observer-visible stable state,
8. rejected work produces diagnostics and replay evidence but no published state,
9. checkpoints and replay/export artifacts are emitted from accepted and retained outcomes.

### 6.2 Architectural Layers
The intended OxCalc layers are:
1. **Intent layer**
   - host-local commands, UI actions, automation calls, VBA-originated requests, file-import deltas.
2. **Operation layer**
   - typed persistent operations, transaction grouping, basis/fence metadata, idempotency and actor metadata.
3. **Apply layer**
   - deterministic transformation from operation payloads into changed immutable structural snapshots plus derived invalidation seeds.
4. **Coordinator layer**
   - compatibility checks, candidate acceptance/rejection, publication, pinned-reader safety, and epoch advancement.
5. **Derived runtime layer**
   - overlays, recalculation work, observer views, lifecycle counters, and retained diagnostics.
6. **Export layer**
   - checkpoints, audit traces, replay projection artifacts, and later collaboration replication envelopes.

### 6.3 Why This Must Stay In OxCalc
This architecture belongs in OxCalc because:
1. operation application changes persistent structural truth,
2. acceptance and publication are coordinator authority,
3. undo/redo must be judged against the same snapshot, fence, and publication rules as ordinary edits,
4. collaboration ordering is inseparable from commit acceptance and visibility rules,
5. replay is downstream of the mutation model rather than the owner of it.

## 7. Core Data Structures

### 7.1 Operation Families
OxCalc should own typed operation families for persistent document mutation, for example:
1. document-structure operations,
2. formula or value assignment operations,
3. name/binding-definition operations,
4. host-setting operations when those settings are persistent and semantics-relevant,
5. external update operations where profile rules require them to be persistent inputs,
6. VBA-driven persistent mutation operations.

### 7.2 Envelope Model
The first realized model should distinguish:
1. **operation payload**
   - the semantic mutation request,
2. **operation envelope**
   - op id, actor/session metadata, causality/idempotency fields, basis epoch, and optional provenance,
3. **transaction envelope**
   - grouped operations with atomic commit semantics,
4. **accepted publication record**
   - the committed outcome tied to snapshot/fence basis and publication epoch.

### 7.3 Snapshot and Checkpoint Relation
The stable relation should be:
1. immutable structural snapshots remain truth,
2. the `OpLog` is the mutation history and audit spine,
3. checkpoints allow bounded startup and recovery cost,
4. replay can start from checkpoint plus operation suffix, or from retained exported replay bundles where appropriate.

The `OpLog` should not be treated as a substitute for immutable published snapshots.

## 8. Undo and Redo Design

### 8.1 Rule
Undo and redo must use the same operation pathway as ordinary edits.

They must not:
1. mutate persistent state behind the coordinator,
2. bypass publication rules,
3. keep a host-only private mutation stack that cannot be replayed.

### 8.2 Design Direction
The first honest model is:
1. accepted user-intent groups are recorded as undoable transaction groups,
2. undo produces compensating operations or an equivalent coordinator-approved inverse transaction,
3. redo replays the corresponding operation group under the same sequencing model,
4. both undo and redo produce ordinary accepted or rejected publication outcomes.

### 8.3 Consequences
This gives:
1. deterministic undo/redo replay,
2. shared auditability,
3. clean interaction with collaboration,
4. no second mutation authority in the host.

## 9. Live and Concurrent Editing Design

### 9.1 Rule
The first live-editing model should be server-sequenced or coordinator-sequenced operations, not CRDT-first.

This is the safer fit because:
1. OxCalc already owns single-publisher coordinator authority,
2. deterministic publication ordering is already required,
3. replay and pack evidence depend on explicit accepted/rejected outcomes,
4. spreadsheet structural edits carry non-trivial rewrite and invalidation semantics.

### 9.2 First Collaboration Model
The first collaboration-capable model should include:
1. actor/session identity,
2. transaction grouping,
3. basis snapshot or basis epoch,
4. idempotency key,
5. causality/order metadata,
6. deterministic reject or stale-drop behavior,
7. explicit publication and visibility consequences.

### 9.3 Staged Concurrency Path
The staged path should be:
1. local single-writer operation log,
2. local undo/redo over accepted operation groups,
3. durable checkpoint plus replay/export,
4. remote or multi-client replication of operation envelopes,
5. staged concurrent evaluator widening behind the same coordinator authority.

## 10. OxCalc Internal Realization Direction

### 10.1 Preferred First Layout
The preferred first realization is inside OxCalc, likely as new or widened Rust crates/modules under the existing workspace:
1. `oxcalc-oplog`
   - operation ids, transaction ids, envelope types, sequencing metadata, checkpoint metadata.
2. `oxcalc-document`
   - operation payload definitions and deterministic `apply_op` / `apply_tx` logic over structural truth.
3. `oxcalc-coordinator`
   - acceptance, rejection, publication, fence checks, and observer-visible epoch advancement.
4. `oxcalc-undo`
   - undo-group policy, compensating-operation generation, redo cursor semantics.
5. `oxcalc-collab`
   - replication envelope, dedupe, causality checks, and staged remote-ingestion helpers.
6. existing runtime/evaluator integration crates
   - remain responsible for evaluation, bind/effect carriage, and publication consequence widening.

### 10.2 Refactor Rule
This is a cross-cutting refactor, not a new semantic lane.

The refactor should:
1. preserve OxCalc ownership of coordinator and publication semantics,
2. avoid hiding mutation semantics inside host shells,
3. keep evaluator and function semantics in `OxFml` and `OxFunc`,
4. keep replay hosting and normalized cross-lane tooling in `OxReplay`.

## 11. Replay and OxReplay Integration

### 11.1 Resulting Split
Once the operation model exists:
1. OxCalc remains the owner of operation meaning and publication truth,
2. OxReplay should consume exported replay-facing artifacts derived from that operation stream,
3. OxReplay must not read internal coordinator state as if it were the public mutation model.

### 11.2 Likely Replay-Facing OxCalc Artifacts
The likely exported families are:
1. retained operation-run bundles,
2. accepted/rejected publication records,
3. operation-derived trace/projection packets,
4. witness bundles and reduction inputs tied to operation sequences,
5. checkpoint-plus-suffix replay descriptors where needed.

The exact packet names should be frozen later by OxCalc and then adapted by OxReplay.

### 11.3 Design Constraint
Replay-facing export should be:
1. deterministic,
2. provenance-rich,
3. explicit about lossy versus lossless projection,
4. separate from internal coordinator-only ephemeral state.

## 12. Host Positioning

### 12.1 DNA TreeCalc and Later Hosts
Runtime hosts such as `DNA TreeCalc`, later `DNA PreCalc`, and later full hosts should:
1. submit edits and persistent commands through OxCalc operation pathways,
2. read stable snapshots, publication state, and observer-visible views from OxCalc,
3. treat undo/redo as OxCalc-backed engine features, not host-local document hacks.

### 12.2 DNA OneCalc
`DNA OneCalc` is a downstream proving host and should not become the primary owner of the multi-node operation model.

It may:
1. mirror smaller host-local intent models for its own proving scope,
2. consume replay surfaces from `OxReplay`,
3. later align with the same operation doctrine where it grows into persistent multi-node editing.

### 12.3 DNA ReCalc
`DNA ReCalc` remains the replay host over `OxReplay`.

It should:
1. validate and replay OxCalc-exported operation-derived artifacts,
2. provide diff/explain/distill over those artifacts,
3. not become the owner of live mutation or collaboration semantics.

## 13. Repo Update Areas

### 13.1 Foundation
Foundation should later absorb a clarification that:
1. `OpLog` doctrine is Foundation-owned,
2. the first executable realization is OxCalc-owned,
3. `OxReplay` consumes operation-derived replay artifacts rather than owning the primary log.

### 13.2 OxCalc
OxCalc needs:
1. this architecture/planning packet,
2. updates to core-engine architecture, state, coordinator, and realization docs as execution begins,
3. new worksets for operation-model realization, undo/redo, and staged collaboration,
4. code restructuring when implementation starts.

### 13.3 OxReplay
OxReplay later needs:
1. an OxCalc adapter/intake note for operation-derived replay artifacts,
2. bundle and normalized projection support for the chosen OxCalc export families,
3. explicit boundaries preventing replay tooling from becoming mutation authority.

### 13.4 OxFml and OxFunc
OxFml and OxFunc later need:
1. seam checks that operation-derived replay exports still preserve evaluator and function provenance,
2. no transfer of evaluator/function semantic ownership into OxCalc's operation layer.

### 13.5 OxVba and OxXlPlay
These repos likely need:
1. mutation-intent mapping into OxCalc operation pathways where they drive persistent edits,
2. replay-facing provenance fields where observation or automation should later be correlated with accepted operations.

## 14. Staged Work Plan

### 14.1 Phase A: Ownership and Architecture Alignment
1. clarify doctrine and repo positioning in Foundation and OxCalc docs,
2. state that OxCalc is the code owner for the first `OpLog` realization,
3. state that OxReplay is downstream replay infrastructure for operation-derived evidence.

### 14.2 Phase B: Local OpLog Baseline In OxCalc
1. define operation and transaction envelope types,
2. define basis/fence and publication relation,
3. implement deterministic operation application over immutable structural truth,
4. retain local accepted and rejected operation evidence.

### 14.3 Phase C: Undo/Redo Baseline
1. define undo-group and redo-group semantics,
2. implement compensating or inverse transaction generation for the first supported operation families,
3. retain deterministic replay evidence for undo and redo outcomes.

### 14.4 Phase D: Replay Export Baseline
1. define OxCalc-owned replay-facing export packets for operation-derived runs,
2. produce retained artifacts that OxReplay can ingest through a declared adapter or canonical bundle surface,
3. widen conformance and explanation around accepted/rejected publication outcomes.

### 14.5 Phase E: Collaboration Baseline
1. add actor/session/idempotency metadata,
2. define replicated transaction envelope rules,
3. define stale-drop, reject, and publication ordering behavior for multi-client mutation intake,
4. retain deterministic replay artifacts for contention and conflict scenarios.

### 14.6 Phase F: Extraction Decision
Only after Phases B through E are implementation-backed should OxCalc decide whether any narrow part deserves extraction into a new shared repo.

## 15. Extraction and Repo-Repurposing Rule
Do not create a new repo now.

A new repo is justified only if:
1. the operation-envelope and replication protocol surface is stable,
2. multiple repos genuinely need to implement that same stable protocol as code,
3. extraction can occur without moving coordinator, publication, or semantic mutation authority out of OxCalc.

If extraction ever happens, the extractable slice should be narrow:
1. wire-level envelope schema,
2. protocol ids and versioning,
3. maybe checkpoint or replication transport helpers.

It should not extract:
1. operation-application semantics,
2. undo policy,
3. coordinator publication logic,
4. lane-specific mutation meaning.

## 16. Initial Workset Packetization Direction
The likely future OxCalc workset line should be:
1. `W0xx` operation-envelope and transaction baseline,
2. `W0xx` operation application and checkpoint relation,
3. `W0xx` undo/redo baseline,
4. `W0xx` replay export and OxReplay handoff,
5. `W0xx` collaboration and replicated transaction baseline.

Final workset naming should follow the active OxCalc register when packetization begins.

## 17. Open Questions
The main remaining design questions are:
1. which first operation families are small enough to realize without premature grid-wide complexity,
2. whether undo should be compensating-operation first or inverse-transaction first for the earliest baseline,
3. which operation-derived artifact family should be the first public OxReplay intake packet,
4. how much checkpointing should be in the first realization versus later optimization,
5. when host-setting and external-update operations become persistent rather than transient policy inputs.

## 18. Resulting Rule
From the OxCalc perspective, the correct architecture is:
1. realize the operation model in OxCalc,
2. build undo/redo and live-editing foundations on that same model,
3. export replay-facing artifacts from OxCalc into OxReplay,
4. keep Foundation as doctrine owner and OxReplay as replay host,
5. delay any new shared repo until a real implemented stable protocol slice exists.
