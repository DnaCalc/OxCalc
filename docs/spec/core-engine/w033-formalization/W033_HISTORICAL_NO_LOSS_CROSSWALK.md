# W033 Historical No-Loss Crosswalk

Status: `calc-uri.4_entry_crosswalk`
Workset: `W033`
Parent epic: `calc-uri`
Bead: `calc-uri.4`
Created: 2026-05-04

## 1. Purpose

This crosswalk maps the original bootstrap formal/theory material and rewrite-control material to current W033 handling.

The archived materials are no-loss inputs. They are not current authority by default. A historical idea affects W033 only when this crosswalk maps it to a current spec, a W033 packet, a deferred lane, a guardrail, an out-of-scope boundary, or a non-carry-forward rationale.

## 2. Historical Source Set

| Source | Role |
|---|---|
| `docs/spec/core-engine/CORE_ENGINE_FORMAL_MODEL.md` | Current redirect/reference surface for the archived formal model. |
| `docs/spec/core-engine/CORE_ENGINE_THEORY_AND_ALTERNATIVE_PATHS.md` | Current redirect/reference surface for the archived theory and alternatives. |
| `docs/spec/core-engine/archive/bootstrap-2026-03/CORE_ENGINE_FORMAL_MODEL.md` | Original consolidated formal-model story. |
| `docs/spec/core-engine/archive/bootstrap-2026-03/CORE_ENGINE_THEORY_AND_ALTERNATIVE_PATHS.md` | Original theory exposition and alternative paths. |
| `docs/spec/core-engine/archive/rewrite-control-2026-03/SPEC_REWRITE_DOCUMENT_MAP.md` | Rewrite target-map and topic-family source. |
| `docs/spec/core-engine/archive/rewrite-control-2026-03/REWRITE_PROMOTION_LEDGER.md` | Rewrite promotion/defer/non-promotion source. |
| `docs/spec/core-engine/archive/rewrite-control-2026-03/SPEC_REWRITE_PLAN.md` | Rewrite intent, quality bar, and sequencing source. |

## 3. Disposition Vocabulary

| Disposition | Meaning |
|---|---|
| `current_scope` | W033 actively carries the idea as part of the first formalization/spec-evolution pass. |
| `promoted_current_spec` | The idea is already represented in current canonical OxCalc specs and must be checked against evidence. |
| `deferred_open_lane` | The idea remains valid but needs later proof, model, replay, implementation, or handoff work. |
| `guardrail_only` | The idea constrains choices but is not itself a first-pass W033 claim. |
| `out_of_scope` | The idea is outside W033 scope. |
| `non_carry_forward` | W033 intentionally does not carry the idea forward because it conflicts with current scope, architecture, or evidence discipline. |

## 4. Bootstrap Formal-Model Crosswalk

| Historical topic | Historical source | Current W033 disposition | Current owner or artifact | Rationale |
|---|---|---|---|---|
| Protocol surface and contracts | bootstrap formal model Section 5.1 | `current_scope` | `W033_AUTHORITY_AND_CLAIM_MATRIX.md`; `CORE_ENGINE_OXFML_SEAM.md`; OxFml read-only FEC/F3E docs | The core engine cannot be formalized without explicit imported evaluator/session/candidate/commit/reject/fence facts. |
| Profiles, gates, and compatibility | bootstrap formal model Section 5.2 | `current_scope` | `W033_PACK_CAPABILITY_BINDING.md`; `CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md` | W033 needs capability honesty and pack binding before any stronger claim is promoted. |
| Epoch and CalcDelta semantics | bootstrap formal model Section 5.3 | `promoted_current_spec` | `CORE_ENGINE_STATE_AND_SNAPSHOTS.md`; `CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`; Lean/TLA first-slice packets | Epoch/fence/publication vocabulary is already core spec material and needs proof/model mapping. |
| Calculation semantics, stabilization, and fixed points | bootstrap formal model Section 5.4; theory Section 3.1 | `current_scope` | `CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`; `W033_TRACECALC_REFINEMENT_PACKET.md` | W033 needs stabilized observable-result semantics for TraceCalc and production conformance. |
| External invalidation and UDF/provider semantics | bootstrap formal model Section 5.5 | `deferred_open_lane` | OxFml/OxFunc handoff/watch lane; later provider/callable publication work | W033 may model consumed runtime-effect facts but does not own broad provider, UDF, or OxFunc kernel semantics. |
| Controls and charts as core entities | bootstrap formal model Section 5.6 | `out_of_scope` | none for W033 | Host/UI/chart semantics are outside this OxCalc + OxFml core formalization pass. |
| Tree-grid hybrid kernel | bootstrap formal model Section 6.1 | `non_carry_forward` for grid ownership; `current_scope` for TreeCalc tree substrate | `CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md`; `CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md` | W033 stays TreeCalc-first and explicitly excludes broad grid semantics beyond current floor. |
| Layered formal model | bootstrap formal model Section 6.2 | `current_scope` | W033 Lean/TLA/replay/refinement packets | W033 keeps layered specs, formal models, replay witnesses, and pack claims synchronized. |
| Executable reference model | bootstrap formal model Section 6.3 | `current_scope` | `CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`; `W033_TRACECALC_REFINEMENT_PACKET.md` | TraceCalc is now the executable correctness oracle for covered behavior. |
| Operations and replay model | bootstrap formal model Section 6.4 | `current_scope` | `CORE_ENGINE_REPLAY_APPLIANCE_ADAPTER.md`; `W033_REPLAY_WITNESS_BRIDGE.md` | Replay/witness identity is a first-pass W033 evidence surface. |
| Structural rewrite model | bootstrap formal model Section 6.5 | `guardrail_only` | `CORE_ENGINE_OPLOG_UNDO_REDO_AND_COLLAB_ARCHITECTURE_PLAN.md`; closure successor packet | Structural rewrite and op-log concerns inform state-transition vocabulary but are not the main first-pass artifact lane. |
| Cycles and iteration | bootstrap formal model Section 6.6 | `deferred_open_lane` | `CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`; successor packet if first-pass review finds pressure | Cycle/SCC vocabulary stays visible, but deep iterative calc semantics are not first-pass W033 closure evidence. |
| Formalization seams | bootstrap formal model Section 6.7 | `current_scope` | `W033_AUTHORITY_AND_CLAIM_MATRIX.md`; `W033_OBJECT_VOCABULARY_AND_LET_LAMBDA_BOUNDARY.md` | W033 is largely about turning seam assumptions into explicit imported contracts and obligations. |
| Coordinator and staged realization | bootstrap formal model Section 6.8 | `current_scope` for Stage 1; `deferred_open_lane` for Stage 2 concurrency promotion | `CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`; TLA first-slice packet; closure successor packet | Stage 1 publication safety is current scope; Stage 2 concurrency remains gated. |
| Idea funnel and representation strategies | bootstrap formal model Section 7 | `guardrail_only` | historical crosswalk; authority matrix; successor packet | Useful design options must be reclassified before affecting current scope. |
| Deferred/open decision register | bootstrap formal model Section 7.3 | `current_scope` as decision discipline | `W033_SPEC_EVOLUTION_DECISION_LEDGER.md` | W033 restores the decision-ledger discipline with current ownership and evidence categories. |

## 5. Theory And Alternative-Paths Crosswalk

| Historical theory or alternative | Historical source | Current W033 disposition | Current owner or artifact | Rationale |
|---|---|---|---|---|
| Fixed points and stabilization | theory Section 3.1 | `current_scope` | recalc spec; TraceCalc refinement; Lean/TLA slices | Needed for comparing incremental and from-scratch stabilized observable results. |
| DAG, SCC, topo baseline | theory Section 3.2 | `current_scope` | recalc spec; TreeCalc evidence; metamorphic test families | Dependency and cycle-region vocabulary underpins invalidation and topological ordering. |
| Invalidation as first-class state | theory Section 3.3 | `promoted_current_spec` | recalc spec; W027/W031 evidence; Lean/TLA slices | Already a current core concern and needs formal/dataflow conservatism treatment. |
| Dynamic dependency graph plus calc-time overlay | theory Section 3.4 | `current_scope` | recalc spec; overlay spec; TreeCalc evidence; replay bridge | This is central to soft/runtime-derived dependency correctness. |
| Spill and virtual region theory | theory Section 3.5 | `out_of_scope` for W033 first pass; `deferred_open_lane` for future grid/spill work | closure successor packet if needed | W033 excludes broad grid/spill semantics. |
| FEC/F3E transactional seam discipline | theory Section 3.6 | `current_scope` | OxFml read-only specs; OxCalc seam companion; authority matrix | Transactional candidate/commit/reject boundaries are core W033 seam material. |
| MVCC, single publisher, pinned readers, epochs | theory Section 3.7 | `current_scope` | state/snapshot, coordinator, overlay specs; TLA first-slice packet | Publication safety and reader compatibility are first-pass formal candidates. |
| Formatting/display boundary | theory Section 3.8 | `out_of_scope` with `guardrail_only` boundary | downstream host seam reference if touched | W033 avoids UI/display semantics but keeps boundary awareness. |
| Visibility as scheduling metadata, not semantic truth | theory Section 3.9 | `guardrail_only`; possible pack row | recalc spec; pack binding | It constrains optimization and scheduling-policy equivalence without becoming a broad first-pass behavior claim. |
| External streams and time models | theory Section 3.10 | `deferred_open_lane` | handoff/watch or successor packet | Volatile inputs and external streams require later host/provider semantics. |
| Dynamic topological maintenance as default | theory Section 4.1 | `deferred_open_lane` | performance/scaling successor, not first-pass correctness foundation | Current baseline remains deterministic topo/SCC; dynamic maintenance is an optimization lane. |
| Full self-adjusting computation as primary runtime | theory Section 4.2 | `non_carry_forward` as primary architecture; `guardrail_only` as inspiration | closure successor packet if research reopened | Too broad and premature for current OxCalc formal/evidence floor. |
| Whole-language incremental lambda calculus or derivative engine | theory Section 4.3 | `non_carry_forward`; narrow LET/LAMBDA carrier fragment remains `current_scope` | LET/LAMBDA boundary packet | W033 does not own general OxFunc semantics or full language derivatives. |
| Differential/timely model as general recalc backbone | theory Section 4.4 | `guardrail_only`; `deferred_open_lane` for future scale architecture | performance/scaling and successor packet | Useful as a comparison model but not the first-pass runtime foundation. |
| Semiring provenance as default explainability substrate | theory Section 4.5 | `non_carry_forward` as default; `deferred_open_lane` for later explainability | replay/explain successor if needed | Explainability should not become a default semantic substrate without evidence. |
| Full lock-free speculative coordinator from Phase 1 | theory Section 4.6 | `non_carry_forward` | Stage 2 concurrency remains gated | Current doctrine keeps Stage 2 concurrency blocked on explicit model/replay gates. |

## 6. Rewrite-Control Crosswalk

| Rewrite row or topic | Historical source | Current W033 disposition | Current owner or artifact | Rationale |
|---|---|---|---|---|
| `RW-001` immutable structural truth and stable identity | rewrite promotion ledger | `promoted_current_spec` | architecture and state/snapshot specs; vocabulary packet | Still a core state-kernel invariant. |
| `RW-002` TreeCalc-first tree-only scope | rewrite promotion ledger | `promoted_current_spec` | TreeCalc semantic plan; OxCalcTree contract | Matches W033 grid boundary and TreeCalc-first floor. |
| `RW-003` versioned snapshots, epochs, stable observer views | rewrite promotion ledger | `current_scope` | state/snapshot, coordinator, overlay specs; TLA packet | Core W033 publication/fence/reader material. |
| `RW-004` runtime overlay taxonomy and lifecycle | rewrite promotion ledger | `current_scope` | overlay spec; replay bridge; pack binding | Overlay retention and protected eviction are W033 safety obligations. |
| `RW-005` single-publisher coordinator and atomic publish | rewrite promotion ledger | `current_scope` | coordinator spec; Lean/TLA packets; TraceCalc refinement | One of the main formalization targets. |
| `RW-006` OxFml evaluator seam boundary and ownership split | rewrite promotion ledger | `current_scope` | OxFml seam companion; authority matrix; handoff/watch packet | Matches W033 OxCalc + read-only OxFml scope. |
| `RW-007` explicit invalidation-state model | rewrite promotion ledger | `current_scope` | recalc spec; Lean/TLA/dataflow rows | Under-invalidation is a correctness fault, not a performance detail. |
| `RW-008` deterministic topo/SCC recalc baseline | rewrite promotion ledger | `current_scope` | recalc spec; metamorphic/differential packet | Baseline needed before dynamic maintenance optimization. |
| `RW-009` verification and early-cutoff incremental strategy | rewrite promotion ledger | `current_scope` with evidence gate | TraceCalc refinement; production conformance | Requires semantic-equivalence and conformance evidence. |
| `RW-010` dynamic dependency handling through runtime overlay | rewrite promotion ledger | `current_scope` | recalc/overlay specs; replay bridge | Aligns with runtime-derived dependency facts. |
| `RW-011` dynamic topological maintenance | rewrite promotion ledger | `deferred_open_lane` | performance/scaling successor | Optimization lane after correctness foundation. |
| `RW-012` staged concurrent and async evaluation | rewrite promotion ledger | `deferred_open_lane` | Stage 2 successor; TLA bridge guardrail | Not promoted by W033 first-pass prose alone. |
| `RW-013` performance instrumentation and decisive experiments | rewrite promotion ledger | `current_scope` as measurement input; semantic promotion deferred | pack/capability binding; conformance packet | Measurement matters but does not substitute for correctness evidence. |
| `RW-014` Lean-facing semantic model and theorem backlog | rewrite promotion ledger | `current_scope` | Lean first-slice packet | W033 widens Lean from a single floor into a module-family plan. |
| `RW-015` TLA+ model of coordinator, fences, and overlay GC safety | rewrite promotion ledger | `current_scope` | TLA first-slice packet | Direct W033 formal leverage lane. |
| `RW-016` Roslyn-inspired persistence principles and structure design space | rewrite promotion ledger | `guardrail_only` | architecture/state successor packet if needed | Useful design inspiration, not first-pass W033 authority. |
| `RW-017` concrete grid persistence strategy alternatives | rewrite promotion ledger | `out_of_scope` | future grid work only | W033 excludes broad grid ownership. |
| `RW-018` grid/spill-heavy future semantics in TreeCalc rewrite | rewrite promotion ledger | `out_of_scope` with deferred watch | future grid/spill work | Not part of current TreeCalc-first formalization floor. |
| `RW-019` over-normative design draft details | rewrite promotion ledger | `guardrail_only`; possible `non_carry_forward` per clause | spec-evolution ledger if rediscovered | W033 should not resurrect over-specified design text without evidence. |
| Rewrite document map target canonical set | rewrite document map | `current_scope` as review surface | core spec review ledger | W033 uses the current canonical set but may evolve it. |
| Rewrite quality bar | rewrite plan Section 11 | `current_scope` | W033 closure audit | W033 inherits the expectation that specs stay precise, scoped, and tied to evidence. |

## 7. First Crosswalk Decisions

| Decision ID | Historical input | Decision | Outcome class | Next action |
|---|---|---|---|---|
| `W033-HIST-001` | Bootstrap reference-machine and layered formal model ideas | Carry forward as TraceCalc oracle plus formal/replay/refinement stack. | `current_scope` | Feed `calc-uri.7`, `calc-uri.8`, `calc-uri.9`, `calc-uri.11`, `calc-uri.12`, and `calc-uri.13`. |
| `W033-HIST-002` | Self-adjusting computation, differential/timely, and dynamic topo alternatives | Keep as optimization/research guardrails, not first-pass engine foundations. | `guardrail_only`; `deferred_open_lane` | Packetize only if performance/scaling or conformance evidence creates pressure. |
| `W033-HIST-003` | Broad grid, spill, controls, charts, and display semantics | Keep outside W033. | `out_of_scope` | Record in closure audit; do not widen W033 silently. |
| `W033-HIST-004` | FEC/F3E transactionality and single-publisher/MVCC discipline | Carry forward as current W033 formal seam and publication-safety scope. | `current_scope` | Feed authority matrix, TLA, Lean, replay, and handoff/watch packets. |
| `W033-HIST-005` | Whole-language incremental lambda calculus | Do not carry forward as a general engine model; preserve only the narrow LET/LAMBDA carrier fragment. | `non_carry_forward` plus `current_scope` boundary fragment | Feed `calc-uri.6`; avoid OxFunc semantic-kernel ownership. |

## 8. Status

- execution_state: `historical_no_loss_crosswalk_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - detailed clause-by-clause historical review may still expose additional rows during later W033 sweeps
  - authority matrix has not yet consumed this crosswalk
  - no OxCalc-owned spec patch is made by this crosswalk
  - no OxFml handoff is filed by this crosswalk
  - deferred and out-of-scope historical topics must be rechecked in the W033 closure audit
