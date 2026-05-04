# W033 Core Spec Review Ledger

Status: `calc-uri.2_entry_ledger`
Workset: `W033`
Parent epic: `calc-uri`
Bead: `calc-uri.2`
Created: 2026-05-04

## 1. Purpose

This ledger defines the ordered W033 review sweep for OxCalc core-engine specs and their W033-facing supporting companions.

The ledger is not a claim that the listed specs are already verified. It records the entry classification, ownership, expected evidence, and next sweep lane so later W033 artifacts can patch, hand off, defer, or reject findings without treating the current prose set as frozen truth.

## 2. Review Status Vocabulary

| Status | Meaning |
|---|---|
| `triaged_for_sweep` | Included in the W033 review surface and assigned a sweep lane. |
| `authority_anchor` | Used as an authority or classification input before deeper clause review. |
| `evidence_anchor` | Used as an existing replay, formal, pack, or realization evidence input. |
| `handoff_sensitive` | Contains seam clauses where normative change may require OxFml handoff. |
| `historical_input` | Used only through the historical no-loss crosswalk. |
| `review_pending` | Detailed clause review still has to happen in later W033 work. |

## 3. Evidence Expectation Vocabulary

| Evidence expectation | Meaning |
|---|---|
| `proof_or_model` | Claim should map to Lean, TLA+, or both. |
| `tracecalc_oracle` | Claim should map to TraceCalc reference-machine behavior. |
| `production_conformance` | Claim should compare optimized/current engine behavior to TraceCalc or declared observable semantics. |
| `replay_or_witness` | Claim should map to deterministic replay artifacts, reduced witnesses, retained witnesses, or fixture cases. |
| `pack_or_capability` | Claim should map to pack rows or capability levels. |
| `handoff_or_watch` | Claim may require OxFml handoff or watch-lane packetization. |
| `deferred_rationale` | Claim is allowed to remain outside first-pass evidence only with explicit rationale. |

## 4. Ordered Sweep Plan

1. **Kernel vocabulary sweep**
   - Primary docs: architecture, state/snapshot, recalc/incremental, overlay/runtime, coordinator/publication.
   - Output pressure: object vocabulary packet, authority matrix, Lean/TLA+ first-slice claims.
2. **OxFml seam and boundary sweep**
   - Primary docs: OxFml seam, minimal upstream host packet, TreeCalc seam negotiation matrix, upstream OxFml sources cited by the source freeze.
   - Output pressure: OxCalc/OxFml ownership split, LET/LAMBDA carrier boundary, handoff/watch candidates.
3. **Oracle, harness, replay, and conformance sweep**
   - Primary docs: TraceCalc reference machine, test harness, scenario schema, runner contract, replay appliance adapter.
   - Output pressure: observable surface, refinement relation, oracle self-checks, production conformance, witness bridge.
4. **TreeCalc and capability sweep**
   - Primary docs: TreeCalc semantic completion plan, TreeCalc assurance authority map, OxCalcTree contract, realization roadmap, formalization/assurance.
   - Output pressure: pack/capability binding, performance/scaling evidence classification, successor packets.
5. **Downstream and non-primary companion sweep**
   - Primary docs: downstream seam reference, OpLog/undo/redo/collaboration plan, spec index, feature map.
   - Output pressure: classification consistency, no unintended host/API promotion, replay/export vocabulary checks.
6. **Historical no-loss sweep**
   - Primary docs: bootstrap formal/theory docs and rewrite-control archives.
   - Output pressure: historical crosswalk rows marked current, deferred, guardrail-only, out-of-scope, or non-carry-forward.

## 5. Core Spec Review Ledger

| Sweep | Source | Owner class | Entry review status | Evidence expectation | Next W033 lane |
|---:|---|---|---|---|---|
| 1 | `docs/spec/core-engine/CORE_ENGINE_ARCHITECTURE.md` | OxCalc canonical | `triaged_for_sweep`; `review_pending` | `proof_or_model`, `tracecalc_oracle`, `deferred_rationale` | `calc-uri.5`, `calc-uri.6`, `calc-uri.11`, `calc-uri.12` |
| 1 | `docs/spec/core-engine/CORE_ENGINE_STATE_AND_SNAPSHOTS.md` | OxCalc canonical | `triaged_for_sweep`; `review_pending` | `proof_or_model`, `replay_or_witness`, `deferred_rationale` | `calc-uri.5`, `calc-uri.6`, `calc-uri.11`, `calc-uri.12`, `calc-uri.13` |
| 1 | `docs/spec/core-engine/CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md` | OxCalc canonical | `triaged_for_sweep`; `review_pending` | `proof_or_model`, `tracecalc_oracle`, `production_conformance`, `replay_or_witness` | `calc-uri.5`, `calc-uri.7`, `calc-uri.8`, `calc-uri.9`, `calc-uri.10`, `calc-uri.11`, `calc-uri.12` |
| 1 | `docs/spec/core-engine/CORE_ENGINE_OVERLAY_AND_DERIVED_RUNTIME.md` | OxCalc canonical | `triaged_for_sweep`; `review_pending` | `proof_or_model`, `replay_or_witness`, `pack_or_capability`, `deferred_rationale` | `calc-uri.5`, `calc-uri.11`, `calc-uri.12`, `calc-uri.13`, `calc-uri.14` |
| 1 | `docs/spec/core-engine/CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md` | OxCalc canonical | `triaged_for_sweep`; `review_pending` | `proof_or_model`, `tracecalc_oracle`, `production_conformance`, `replay_or_witness` | `calc-uri.5`, `calc-uri.7`, `calc-uri.8`, `calc-uri.9`, `calc-uri.11`, `calc-uri.12`, `calc-uri.13` |
| 2 | `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md` | OxCalc canonical seam companion; OxFml-sensitive | `triaged_for_sweep`; `handoff_sensitive`; `review_pending` | `handoff_or_watch`, `replay_or_witness`, `proof_or_model`, `deferred_rationale` | `calc-uri.5`, `calc-uri.6`, `calc-uri.13`, `calc-uri.15` |
| 2 | `docs/spec/core-engine/CORE_ENGINE_OXFML_MINIMAL_UPSTREAM_HOST_INTERFACES.md` | OxCalc supporting companion; OxFml-sensitive | `triaged_for_sweep`; `handoff_sensitive`; `review_pending` | `replay_or_witness`, `production_conformance`, `handoff_or_watch` | `calc-uri.5`, `calc-uri.6`, `calc-uri.13`, `calc-uri.15` |
| 2 | `docs/spec/core-engine/CORE_ENGINE_TREECALC_OXFML_SEAM_NEGOTIATION_MATRIX.md` | temporary planning companion | `triaged_for_sweep`; `handoff_sensitive`; `review_pending` | `handoff_or_watch`, `deferred_rationale` | `calc-uri.5`, `calc-uri.6`, `calc-uri.15` |
| 3 | `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md` | OxCalc supporting companion; oracle surface | `triaged_for_sweep`; `evidence_anchor`; `review_pending` | `tracecalc_oracle`, `production_conformance`, `replay_or_witness` | `calc-uri.7`, `calc-uri.8`, `calc-uri.9`, `calc-uri.13` |
| 3 | `docs/spec/core-engine/CORE_ENGINE_TEST_HARNESS_AND_FIXTURES.md` | OxCalc supporting companion | `triaged_for_sweep`; `evidence_anchor`; `review_pending` | `tracecalc_oracle`, `replay_or_witness`, `production_conformance` | `calc-uri.7`, `calc-uri.8`, `calc-uri.9`, `calc-uri.10`, `calc-uri.13` |
| 3 | `docs/spec/core-engine/CORE_ENGINE_TEST_SCENARIO_SCHEMA_AND_TRACECALC.md` | OxCalc supporting companion | `triaged_for_sweep`; `evidence_anchor`; `review_pending` | `tracecalc_oracle`, `replay_or_witness`, `pack_or_capability` | `calc-uri.7`, `calc-uri.8`, `calc-uri.10`, `calc-uri.13`, `calc-uri.14` |
| 3 | `docs/spec/core-engine/CORE_ENGINE_TEST_VALIDATOR_AND_RUNNER_CONTRACT.md` | OxCalc supporting companion | `triaged_for_sweep`; `evidence_anchor`; `review_pending` | `replay_or_witness`, `pack_or_capability`, `production_conformance` | `calc-uri.8`, `calc-uri.9`, `calc-uri.13`, `calc-uri.14` |
| 3 | `docs/spec/core-engine/CORE_ENGINE_REPLAY_APPLIANCE_ADAPTER.md` | OxCalc supporting companion; Foundation-sensitive | `triaged_for_sweep`; `evidence_anchor`; `review_pending` | `replay_or_witness`, `pack_or_capability`, `deferred_rationale` | `calc-uri.13`, `calc-uri.14`, `calc-uri.16` |
| 4 | `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md` | OxCalc canonical | `triaged_for_sweep`; `authority_anchor`; `review_pending` | `proof_or_model`, `replay_or_witness`, `pack_or_capability` | `calc-uri.5`, `calc-uri.11`, `calc-uri.12`, `calc-uri.14` |
| 4 | `docs/spec/core-engine/CORE_ENGINE_REALIZATION_ROADMAP.md` | OxCalc canonical | `triaged_for_sweep`; `authority_anchor`; `review_pending` | `pack_or_capability`, `deferred_rationale`, `handoff_or_watch` | `calc-uri.5`, `calc-uri.14`, `calc-uri.15`, `calc-uri.16` |
| 4 | `docs/spec/core-engine/CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md` | OxCalc canonical TreeCalc-first plan | `triaged_for_sweep`; `evidence_anchor`; `review_pending` | `production_conformance`, `replay_or_witness`, `pack_or_capability`, `deferred_rationale` | `calc-uri.5`, `calc-uri.9`, `calc-uri.10`, `calc-uri.13`, `calc-uri.14` |
| 4 | `docs/spec/core-engine/CORE_ENGINE_TREECALC_ASSURANCE_AUTHORITY_MAP.md` | OxCalc supporting companion | `triaged_for_sweep`; `evidence_anchor`; `review_pending` | `pack_or_capability`, `replay_or_witness`, `deferred_rationale` | `calc-uri.5`, `calc-uri.13`, `calc-uri.14`, `calc-uri.16` |
| 4 | `docs/spec/core-engine/CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md` | OxCalc host-facing contract | `triaged_for_sweep`; `authority_anchor`; `review_pending` | `production_conformance`, `replay_or_witness`, `deferred_rationale` | `calc-uri.5`, `calc-uri.9`, `calc-uri.13`, `calc-uri.14` |
| 5 | `docs/spec/core-engine/CORE_ENGINE_DOWNSTREAM_HOST_SEAM_REFERENCE.md` | OxCalc downstream seam-reference filter | `triaged_for_sweep`; `authority_anchor`; `review_pending` | `handoff_or_watch`, `deferred_rationale` | `calc-uri.5`, `calc-uri.15`, `calc-uri.16` |
| 5 | `docs/spec/core-engine/CORE_ENGINE_OPLOG_UNDO_REDO_AND_COLLAB_ARCHITECTURE_PLAN.md` | OxCalc supporting companion | `triaged_for_sweep`; `review_pending` | `deferred_rationale`, `replay_or_witness` where replay/export vocabulary intersects W033 | `calc-uri.5`, `calc-uri.13`, `calc-uri.16` |
| 5 | `docs/spec/README.md` | OxCalc spec index | `authority_anchor`; `review_pending` | `deferred_rationale` for classification drift | `calc-uri.16` |
| 5 | `docs/WORKSET_REGISTER.md` | OxCalc workset authority | `authority_anchor`; `review_pending` | `deferred_rationale`; bead closure evidence | `calc-uri.16` |
| 5 | `docs/IN_PROGRESS_FEATURE_WORKLIST.md` | OxCalc feature map | `authority_anchor`; `review_pending` | `deferred_rationale`; status consistency | `calc-uri.16` |
| 6 | `docs/spec/core-engine/CORE_ENGINE_FORMAL_MODEL.md` | historical redirect/reference | `historical_input`; `review_pending` | `deferred_rationale` or promotion through current specs | `calc-uri.4` |
| 6 | `docs/spec/core-engine/CORE_ENGINE_THEORY_AND_ALTERNATIVE_PATHS.md` | historical redirect/reference | `historical_input`; `review_pending` | `deferred_rationale` or promotion through current specs | `calc-uri.4` |
| 6 | `docs/spec/core-engine/archive/bootstrap-2026-03/CORE_ENGINE_FORMAL_MODEL.md` | historical archive | `historical_input`; `review_pending` | `deferred_rationale` or promotion through current specs | `calc-uri.4` |
| 6 | `docs/spec/core-engine/archive/bootstrap-2026-03/CORE_ENGINE_THEORY_AND_ALTERNATIVE_PATHS.md` | historical archive | `historical_input`; `review_pending` | `deferred_rationale` or promotion through current specs | `calc-uri.4` |
| 6 | `docs/spec/core-engine/archive/rewrite-control-2026-03/SPEC_REWRITE_DOCUMENT_MAP.md` | historical archive | `historical_input`; `review_pending` | `deferred_rationale` or promotion through current specs | `calc-uri.4` |
| 6 | `docs/spec/core-engine/archive/rewrite-control-2026-03/REWRITE_PROMOTION_LEDGER.md` | historical archive | `historical_input`; `review_pending` | `deferred_rationale` or promotion through current specs | `calc-uri.4` |
| 6 | `docs/spec/core-engine/archive/rewrite-control-2026-03/SPEC_REWRITE_PLAN.md` | historical archive | `historical_input`; `review_pending` | `deferred_rationale` or promotion through current specs | `calc-uri.4` |

## 6. First Detailed Review Questions

These questions are the initial sweep prompts. They are intentionally framed as classification checks rather than conclusions.

| Question | Primary lane |
|---|---|
| Does the doc distinguish structural truth, runtime overlays, candidates, commits, rejects, and publication state with enough precision for formal state transitions? | Kernel vocabulary sweep |
| Does the doc accidentally promote current implementation behavior without evidence, or does it classify behavior as an evidence input? | Kernel vocabulary sweep |
| Does the doc keep candidate result, accepted commit, and public publication separate? | Coordinator/publication sweep |
| Does the doc state reject-is-no-publish in a way that can be expressed in TraceCalc, Lean, and TLA+? | Coordinator/publication sweep |
| Does the doc preserve OxFml-owned reject, fence, trace, and runtime-effect meanings without OxCalc reinterpretation? | OxFml seam sweep |
| Does the doc expose every dynamic dependency or runtime-effect fact needed to avoid under-invalidation? | Recalc and seam sweep |
| Does the doc keep TraceCalc oracle authority separate from production/core-engine conformance claims? | Oracle/conformance sweep |
| Does the doc tie pack/capability claims to proof, model, replay, or explicit deferral? | Capability sweep |
| Does the doc preserve historical intent or explicitly explain why an older idea is not carried forward? | Historical no-loss sweep |

## 7. Mutation And Handoff Rule

W033 may patch OxCalc-owned specs after a finding is classified in the spec-evolution decision ledger.

W033 must not patch OxFml directly. If a review finding requires a normative OxFml, FEC/F3E, formula-language, or LET/LAMBDA carrier change, the finding moves to the W033 handoff/watch packet and, where appropriate, a handoff entry under `docs/handoffs/`.

## 8. Status

- execution_state: `core_spec_review_ledger_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - detailed clause review is still pending for all rows marked `review_pending`
  - spec-evolution decisions are not yet entered in `W033_SPEC_EVOLUTION_DECISION_LEDGER.md`
  - no OxCalc-owned spec patch is made by this ledger
  - no OxFml handoff is filed by this ledger
  - authority matrix, vocabulary packet, refinement packet, formal slices, replay bridge, pack binding, and closure audit remain later W033 lanes
