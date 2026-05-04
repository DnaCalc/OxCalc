# W033 Authority And Claim Matrix

Status: `calc-uri.5_entry_matrix`
Workset: `W033`
Parent epic: `calc-uri`
Bead: `calc-uri.5`
Created: 2026-05-04

## 1. Purpose

This matrix maps first-pass W033 claims to ownership, source specs, formal targets, replay/conformance evidence, pack or capability obligations, and handoff/deferred state.

The matrix is a routing artifact. It does not promote a claim beyond its current evidence. A row marked `planned` or `source_only` remains partial until the linked proof, model, replay, conformance, pack, or handoff artifact exists.

## 2. Ownership Classes

| Class | Meaning | Mutation rule |
|---|---|---|
| `OxCalc-owned` | OxCalc owns the spec and artifact surface. | W033 may patch in this repo after decision-ledger classification. |
| `OxFml-owned` | OxFml owns the semantic or seam source. | W033 may cite/model read-only and must hand off normative changes. |
| `shared-seam` | OxFml owns evaluator-facing truth and OxCalc owns coordinator-facing consumption requirements. | W033 may patch OxCalc companion text and must hand off upstream changes. |
| `OxFunc-opaque` | OxFunc owns general function semantics. | W033 may state opaque assumptions only. |
| `LET-LAMBDA-carrier` | Narrow OxFml/OxFunc boundary facts that affect OxCalc-visible dependency, trace, replay, or runtime-effect behavior. | W033 may model carrier facts and hand off if upstream surfaces are insufficient. |
| `Foundation-sensitive` | Foundation doctrine or replay/capability policy constrains promotion. | W033 may cite local source freeze and must avoid claiming Foundation-owned promotion. |

## 3. Evidence-State Vocabulary

| State | Meaning |
|---|---|
| `existing_floor` | Some current spec, formal, replay, or run artifact exists, but W033 still has to bind it explicitly. |
| `planned` | W033 has a declared downstream artifact obligation but no first-pass artifact yet. |
| `source_only` | Current source/spec text exists without exercised W033 evidence. |
| `deferred` | W033 intentionally leaves this as a later lane with rationale. |
| `handoff_watch` | Concrete seam pressure may require an OxFml handoff/watch row. |

## 4. First-Pass Claim Matrix

| Claim ID | Claim | Owner class | Primary source(s) | Modeled object or transition | Lean target | TLA+ target | TraceCalc / replay / conformance target | Pack / capability target | Handoff or deferral state | Evidence state | Driving beads |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `W033-CLM-001` | Structural truth is separate from runtime, overlay, candidate, and publication state. | `OxCalc-owned` | state/snapshot spec; architecture spec | structural snapshot, runtime view, overlay attachment | `CoreSnapshots`; structural-runtime non-mutation invariant | state snapshot transition safety | replay projection distinguishes structural and runtime surfaces | none first pass | none | `source_only` | `calc-uri.6`, `calc-uri.11`, `calc-uri.13` |
| `W033-CLM-002` | Candidate result is not public publication. | `shared-seam` | coordinator spec; OxFml seam companion; FEC/F3E design input | candidate, accepted candidate, public snapshot | accepted-candidate-is-not-publication theorem | candidate/commit/publication state separation | TraceCalc observed artifact surface; production conformance comparison | `PACK.fec.commit_atomicity` | handoff only if OxFml candidate wording is insufficient | `existing_floor` | `calc-uri.7`, `calc-uri.8`, `calc-uri.9`, `calc-uri.11`, `calc-uri.12`, `calc-uri.14` |
| `W033-CLM-003` | Reject is no-publish. | `shared-seam` | coordinator spec; OxFml reject taxonomy; TraceCalc docs | reject transition, public snapshot, reject artifact | reject-is-no-publish theorem | no transition from reject to publication | TraceCalc reject scenario self-check; production comparison | `PACK.fec.reject_detail_replay` | handoff only for upstream reject meaning drift | `existing_floor` | `calc-uri.7`, `calc-uri.8`, `calc-uri.9`, `calc-uri.11`, `calc-uri.12`, `calc-uri.13`, `calc-uri.14` |
| `W033-CLM-004` | Accepted publication is atomic and not torn. | `OxCalc-owned` with `shared-seam` inputs | coordinator spec; state/snapshot spec; FEC/F3E design input | accept-and-publish transition | publication atomicity theorem | no torn publication property | TraceCalc published view and production comparison | `PACK.fec.commit_atomicity`; `PACK.concurrent.epochs` | Stage 2 concurrency deferred | `source_only` | `calc-uri.7`, `calc-uri.9`, `calc-uri.11`, `calc-uri.12`, `calc-uri.14` |
| `W033-CLM-005` | Stale or incompatible fence cannot publish. | `shared-seam` | coordinator spec; OxFml seam companion; FEC/F3E specs | fence compatibility, commit bundle, publish transition | fence-preservation theorem backlog | stale-fence-no-publish property | replay/witness bridge should include fence mismatch | `PACK.fec.commit_atomicity`; `PACK.concurrent.epochs` | `handoff_watch` if OxFml fence facts are too weak | `planned` | `calc-uri.12`, `calc-uri.13`, `calc-uri.15` |
| `W033-CLM-006` | Dynamic dependency and runtime-effect facts must not be silently lost. | `shared-seam`; `LET-LAMBDA-carrier` where applicable | recalc spec; overlay spec; OxFml delta/effect docs; LET/LAMBDA prep input | static graph, runtime graph, effect facts, invalidation closure | invalidation-closure coverage theorem backlog | dynamic dependency update/interleaving model | TreeCalc replay bridge; production conformance; metamorphic families | `PACK.dag.dynamic_dependency_bind_semantics`; `PACK.trace.forensic_plane` | handoff/watch for upstream carrier insufficiency | `existing_floor` | `calc-uri.6`, `calc-uri.9`, `calc-uri.10`, `calc-uri.11`, `calc-uri.12`, `calc-uri.13`, `calc-uri.14`, `calc-uri.15` |
| `W033-CLM-007` | Conservative over-invalidation is admissible; under-invalidation is a correctness fault. | `OxCalc-owned` | recalc spec; historical theory crosswalk | invalidation closure, affected set, dirty set | invalidation closure contains affected nodes | invalidation interleaving safety | TreeCalc local/scale evidence classified via conformance | `PACK.dag.dynamic_dependency_bind_semantics` | none | `planned` | `calc-uri.10`, `calc-uri.11`, `calc-uri.12`, `calc-uri.13`, `calc-uri.14` |
| `W033-CLM-008` | Overlay eviction must not remove protected or pinned-compatible state. | `OxCalc-owned` | overlay spec; state/snapshot spec; historical MVCC rows | overlay lifecycle, pin, eviction predicate | protected-overlay-retention theorem backlog | pinned-reader overlay retention model | replay/witness bridge for retention cases | `PACK.fec.overlay_lifecycle`; `PACK.overlay.fallback_economics` | Stage 2 contention deferred | `source_only` | `calc-uri.11`, `calc-uri.12`, `calc-uri.13`, `calc-uri.14` |
| `W033-CLM-009` | Pinned readers keep a compatible view across publication and overlay lifecycle events. | `OxCalc-owned` with Foundation-sensitive replay policy | state/snapshot spec; coordinator spec; overlay spec | pinned reader, epoch, publication view | pinned-view compatibility backlog | pinned-reader safety property | TraceCalc pinned views; replay appliance projection | `PACK.concurrent.epochs`; `PACK.replay.appliance` | concurrency promotion deferred | `existing_floor` | `calc-uri.7`, `calc-uri.12`, `calc-uri.13`, `calc-uri.14` |
| `W033-CLM-010` | TraceCalc is the executable correctness oracle only for covered behavior. | `OxCalc-owned` | TraceCalc reference-machine spec; test scenario schema; runner contract | reference state, transition, observed artifacts | optional reference-state correspondence lemmas | optional sequential reference transition model | TraceCalc oracle self-check first slice | `PACK.replay.appliance`; `PACK.diff.cross_engine.continuous` | uncovered behavior becomes oracle gap | `existing_floor` | `calc-uri.7`, `calc-uri.8`, `calc-uri.13`, `calc-uri.14` |
| `W033-CLM-011` | Production/core-engine behavior must refine TraceCalc over a declared observable surface for covered behavior. | `OxCalc-owned` | TraceCalc spec; coordinator/recalc specs; W033 plan | observable surface, compatibility relation, mismatch taxonomy | refinement relation backlog | strategy-equivalence model where needed | production conformance first slice | `PACK.diff.cross_engine.continuous`; `PACK.scaling.signature` where tied to semantics | internal strategy differences require semantic-equivalence statement | `planned` | `calc-uri.7`, `calc-uri.9`, `calc-uri.10`, `calc-uri.14` |
| `W033-CLM-012` | LET/LAMBDA carrier facts cross the OxFml/OxFunc boundary and can affect OxCalc-visible dependency, trace, replay, and runtime-effect behavior. | `LET-LAMBDA-carrier` | OxFml LET/LAMBDA prep docs; OxFunc semantic-boundary docs; OxCalc seam companion | local binding, lambda value, call identity, prepared-call carrier, runtime-effect visibility | `OxfmlOxfuncBoundary` abstract ADTs | optional carrier fact visibility model | fixture/witness selection for higher-order callable cases | `PACK.dag.dynamic_dependency_bind_semantics`; `PACK.trace.forensic_plane` | handoff/watch if upstream carrier facts are not enough | `source_only` | `calc-uri.6`, `calc-uri.10`, `calc-uri.13`, `calc-uri.15` |
| `W033-CLM-013` | Ordinary OxFunc function kernels remain opaque to W033. | `OxFunc-opaque` | W033 plan; OxFml OxFunc semantic boundary | function outcome as imported packet fact | opaque assumption only | none first pass | no direct semantic replay claim beyond consumed packet facts | none first pass | `non_carry_forward` for general kernels | `source_only` | `calc-uri.6`, `calc-uri.15`, `calc-uri.16` |
| `W033-CLM-014` | Scale/performance measurements do not prove semantic correctness unless tied to conformance, replay, or pack obligations. | `OxCalc-owned`; Foundation-sensitive | realization roadmap; formalization/assurance spec; historical crosswalk | timing counters, phase split, run manifest | none first pass | none first pass | conformance packet may consume selected measurements | `PACK.scaling.signature` only with semantic binding | measurement-only rows remain deferred | `existing_floor` | `calc-uri.9`, `calc-uri.10`, `calc-uri.14`, `calc-uri.16` |
| `W033-CLM-015` | Pack/capability claims must not exceed proof/model/replay/conformance evidence. | Foundation-sensitive; `OxCalc-owned` local mapping | formalization/assurance spec; replay appliance adapter; capability manifest | pack row, capability level, evidence pointer | theorem-backed rows where available | model-check-backed rows where available | replay/conformance backed rows where available | all W033 pack rows | none; deferred rows explicit | `existing_floor` | `calc-uri.14`, `calc-uri.16` |
| `W033-CLM-016` | Stage 2 concurrency promotion is outside first-pass W033 unless later model/replay gates are explicit. | `OxCalc-owned`; Foundation-sensitive | coordinator spec; realization roadmap; historical crosswalk | contention, async work, pinned readers, single publisher | future theorem backlog | future Stage 2 model | no first-pass promotion | `PACK.concurrent.epochs` deferred | `deferred` | `source_only` | `calc-uri.12`, `calc-uri.14`, `calc-uri.16` |
| `W033-CLM-017` | Replay and witness identity must correlate OxFml candidate/commit/reject facts to OxCalc coordinator outcomes without losing provenance. | `shared-seam`; Foundation-sensitive | replay adapter; OxFml fixtures/witnesses; runner contract | trace id, replay id, witness lifecycle, reduction manifest | optional replay-equivalent history theorem | optional replay transition model | replay/witness bridge across OxCalc and OxFml | `PACK.replay.appliance`; `PACK.trace.forensic_plane` | handoff/watch for upstream identity insufficiency | `existing_floor` | `calc-uri.13`, `calc-uri.14`, `calc-uri.15` |
| `W033-CLM-018` | OxFml reject, fence, trace, and runtime-effect meanings are imported, not reinterpreted by OxCalc. | `OxFml-owned`; `shared-seam` consumption | OxFml consumer/facade contract; FEC/F3E specs; OxCalc seam companion | imported packet facts and coordinator consumption | OxFml seam ADTs as abstract imports | FEC bridge model | cross-lane witness bridge | `PACK.fec.reject_detail_replay`; `PACK.trace.forensic_plane` | `handoff_watch` for normative changes | `source_only` | `calc-uri.6`, `calc-uri.12`, `calc-uri.13`, `calc-uri.15` |
| `W033-CLM-019` | Metamorphic and differential transformations can multiply coverage but must preserve the declared observable surface. | `OxCalc-owned` | test harness; scenario schema; recalc spec; TraceCalc spec | scenario transformation, observed artifacts, compatibility relation | optional transformation-preservation lemmas | scheduling-policy equivalence where needed | metamorphic/differential test-family packet | `PACK.diff.cross_engine.continuous` | strategy changes need semantic-equivalence statement | `planned` | `calc-uri.10`, `calc-uri.14`, `calc-uri.16` |
| `W033-CLM-020` | Historical/bootstrap ideas influence W033 only through explicit crosswalk disposition. | `OxCalc-owned` | historical no-loss crosswalk; spec-evolution ledger | historical idea, disposition, owner/rationale | none first pass | none first pass | no replay effect unless promoted | none first pass | out-of-scope and non-carry-forward rows checked at closure | `existing_floor` | `calc-uri.14`, `calc-uri.16` |

## 5. Claim-To-Bead Dependency Reading

The first downstream use of this matrix is:

1. `calc-uri.6` consumes owner classes, object vocabulary, OxFml-owned imported facts, OxFunc-opaque assumptions, and the LET/LAMBDA carrier rows.
2. `calc-uri.7` consumes TraceCalc oracle, observable-surface, publication, reject, and conformance rows.
3. `calc-uri.11` and `calc-uri.12` consume the Lean/TLA target columns.
4. `calc-uri.13` consumes replay, witness, trace identity, and OxFml fixture rows.
5. `calc-uri.14` consumes pack/capability rows and evidence states.
6. `calc-uri.15` consumes every row marked `handoff_watch` or upstream-sensitive.
7. `calc-uri.16` checks that no row remains promoted beyond evidence and packetizes uncovered rows.

## 6. Status

- execution_state: `authority_and_claim_matrix_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - downstream W033 packets have not yet consumed the matrix
  - most claim rows are `source_only`, `planned`, or `existing_floor` rather than evidence-bound W033 promotions
  - no OxFml handoff is filed by this matrix
  - Lean, TLA+, TraceCalc, production conformance, replay/witness, metamorphic, pack, handoff/watch, and closure packets remain later W033 lanes
