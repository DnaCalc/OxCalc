# W033 OxFml Handoff Watch Packet

Status: `calc-uri.15_evidence_packet`
Workset: `W033`
Parent epic: `calc-uri`
Bead: `calc-uri.15`
Created: 2026-05-04

## 1. Purpose

This packet classifies W033 seam pressure into:

1. `no_new_handoff_required`,
2. `watch`,
3. `deferred_successor`,
4. `handoff_required_now`.

The packet makes no direct OxFml repo patch. A handoff register row is required only if W033 has concrete normative OxFml/FEC/F3E/formula-language pressure that cannot be handled by the current upstream source, note-level watch lane, or OxCalc-local successor work.

## 2. Inputs Reviewed

| Input | W033 reading |
|---|---|
| `docs/spec/core-engine/w033-formalization/W033_OBJECT_VOCABULARY_AND_LET_LAMBDA_BOUNDARY.md` | Defines the narrow `LET`/`LAMBDA` carrier fragment and watch rows. |
| `docs/spec/core-engine/w033-formalization/W033_REPLAY_WITNESS_BRIDGE.md` | Bridges OxFml FEC/witness inputs to OxCalc TraceCalc/TreeCalc evidence and identifies fixture gaps. |
| `docs/spec/core-engine/w033-formalization/W033_PACK_CAPABILITY_BINDING.md` | Caps all pack/capability claims below pack-grade and marks LET/LAMBDA and direct OxFml fixture replay as open. |
| `docs/spec/core-engine/w033-formalization/W033_SPEC_EVOLUTION_DECISION_LEDGER.md` | Requires handoff for normative OxFml changes, not for routine observation exchange. |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | Current OxFml-owned observation ledger. It repeatedly classifies current residuals as note-level/watch unless live evidence exposes a concrete insufficiency. |
| `docs/handoffs/HANDOFF_REGISTER.csv` | Existing local register has only `HANDOFF-CALC-001` and `HANDOFF-FML-001`; no W033 handoff row is added by this bead. |

## 3. Handoff Decision

Current W033 decision: `no_new_handoff_required`.

Rationale:

1. Candidate, commit, reject, trace, fence, and capability-sensitive facts are already available as OxFml-owned upstream surfaces for current W033 first-slice modeling.
2. The W033 replay bridge found gaps in OxCalc-local replay coverage, but not a concrete upstream contradiction requiring an OxFml normative change.
3. `LET`/`LAMBDA` carrier facts are important to W033 scope, but W033 has not yet exercised TraceCalc or TreeCalc carrier witnesses that prove the current OxFml surface is insufficient.
4. OxFml's inbound note keeps provider-failure, callable-publication, execution-restriction transport breadth, and publication/topology breadth in note-level/watch lanes unless concrete implementation evidence forces a narrower handoff.
5. Pack/capability limitations are local promotion constraints, not OxFml handoff requirements.

No new row is added to `docs/handoffs/HANDOFF_REGISTER.csv`.

## 4. Watch And Handoff Classification

| Watch ID | Surface | W033 evidence | Classification | Trigger for later handoff |
|---|---|---|---|---|
| `W033-HW-001` | FEC candidate/commit/reject/fence facts | `W033-WIT-001` to `W033-WIT-004`; OxFml FEC fixtures; TraceCalc and TreeCalc no-publish/publication evidence | `no_new_handoff_required` | upstream FEC fixture or canonical text contradicts OxCalc coordinator consumption, or direct OxFml fixture replay exposes mismatch |
| `W033-HW-002` | `LET`/`LAMBDA` minimum callable carrier | `W033-WATCH-LL-001`; Lean abstract `LetLambdaCarrierFact`; no TraceCalc/TreeCalc carrier witness yet | `watch` | carrier origin, capture, arity, invocation contract, dependency, runtime-effect, or replay identity is unrecoverable in exercised carrier evidence |
| `W033-HW-003` | `LET`/`LAMBDA` provenance detail | `W033-WATCH-LL-002`; no W033 carrier scenario yet | `watch` | provenance becomes required for replay/witness identity or mismatch triage and upstream surface cannot carry it |
| `W033-HW-004` | Callable publication policy | `W033-WATCH-LL-003`; OxFml note keeps callable-publication as watch until coordinator-visible evidence | `watch` | callable publication or callable-result reject behavior becomes coordinator-visible in exercised W033 evidence |
| `W033-HW-005` | Callable/provider/gating typed reject and retry behavior | `W033-WATCH-LL-004`; OxFml note flags provider failure as likely future trigger only if coordinator-visible | `watch` | provider failure changes typed reject, retry, or publication consequences visible to OxCalc |
| `W033-HW-006` | LET/LAMBDA dependency visibility | `W033-WATCH-LL-005`; TraceCalc/TreeCalc dynamic dependency evidence exists, but not callable carrier evidence | `watch` | OxCalc cannot conservatively invalidate without additional OxFml surfaced carrier facts |
| `W033-HW-007` | Runtime-derived effects and execution-restriction transport breadth | OxFml note says current candidate-result, commit-bundle, topology/effect refs, and capability/execution observations are consumable, while final carrier shape is narrower | `watch` | live TreeCalc or TraceCalc evidence shows the current carried family cannot preserve coordinator-visible consequence or trace identity |
| `W033-HW-008` | Publication and topology consequence breadth | OxFml note calls current breadth canonical but narrower; TreeCalc current topology/overlay evidence remains first-slice | `watch` | broader publication/topology cases require consequence categories not present in OxFml current surface |
| `W033-HW-009` | Direct OxFml fixture replay inside OxCalc | replay bridge gap: no OxFml fixture is replayed inside OxCalc by W033 first slice | `deferred_successor` | successor work chooses direct fixture replay and finds mismatch requiring upstream normative change |
| `W033-HW-010` | Pack-grade replay or witness promotion | pack binding caps current evidence below `cap.C5.pack_valid` | `deferred_successor` | pack governance requires an OxFml-owned replay/witness promotion decision |
| `W033-HW-011` | Structured references, table context, immutable edit, stand-in fixture host, registered external packets | inbound OxFml note records these as converged or bounded note-level lanes | `no_action_for_W033` | only if a later TreeCalc/formal evidence lane brings one of these into W033 core-engine semantics |

## 5. No-Action Rows

The following are not W033 handoff triggers:

1. Existing `HANDOFF-CALC-001` and `HANDOFF-FML-001` remain historical register rows and are not reopened by W033.
2. OxFml current local replay capability and retained-local witness floor are not a request for OxCalc to claim pack-grade replay.
3. General OxFunc kernel semantics remain out of W033 scope.
4. Structured-reference/table/context/editor/registered-external packet topics remain outside this W033 formalization pass unless later evidence pulls them into the core-engine contract.

## 6. Handoff Register State

`docs/handoffs/HANDOFF_REGISTER.csv` is not modified by this packet.

Current local register rows:

| Handoff | Direction | Status | W033 effect |
|---|---|---|---|
| `HANDOFF-CALC-001` | outbound | acknowledged | historical seam-hardening input; not reopened |
| `HANDOFF-FML-001` | inbound | acknowledged | historical intake input; not reopened |

## 7. Downstream Obligations

1. `calc-uri.16` must carry the watch rows into the W033 closure/successor packet.
2. Later direct OxFml fixture replay, LET/LAMBDA carrier witnesses, or broader TreeCalc consequence cases must revisit this classification if they expose a concrete upstream insufficiency.
3. Any future public or cross-repo handoff message must follow the public attribution doctrine.

## 8. Status

- execution_state: `oxfml_handoff_watch_packet_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - W033 parent epic remains open
  - closure audit has not yet consumed this packet
  - watch rows remain active successor inputs
  - no W033 handoff register row is filed because no handoff is required by current evidence
