# W033 Spec-Evolution Decision Ledger

Status: `calc-uri.3_entry_ledger`
Workset: `W033`
Parent epic: `calc-uri`
Bead: `calc-uri.3`
Created: 2026-05-04

## 1. Purpose

This ledger defines how W033 records discoveries that may change OxCalc-owned specs, create OxFml handoff pressure, expose implementation mismatch, identify TraceCalc oracle gaps, defer open lanes, or reject older ideas.

The point is to make the formalization pass a controlled spec-evolution loop. W033 is not a fixed-spec compliance pass and is not an approval of current implementation behavior by default.

## 2. Decision Input Classes

| Input class | Meaning | Required handling |
|---|---|---|
| `current_spec_clause` | Current OxCalc-owned spec text or supporting companion text. | Confirm, patch, defer, or narrow with evidence/rationale. |
| `current_implementation_behavior` | Behavior observed in current OxCalc code or emitted artifacts. | Classify as intended, fault, oracle gap, spec gap, or unresolved before it affects specs. |
| `tracecalc_behavior` | Behavior observed in TraceCalc reference-machine artifacts. | Classify as oracle authority only for covered behavior; otherwise classify as oracle gap or pending coverage. |
| `production_conformance_signal` | Comparison between production/core-engine behavior and TraceCalc or declared observable semantics. | Classify mismatch severity and whether it is implementation fault, spec gap, oracle gap, or intended strategy difference. |
| `formal_model_pressure` | Lean/TLA+/mathematical modeling exposes an invariant, contradiction, or missing state distinction. | Patch OxCalc-owned specs, add formal obligation, hand off, or defer. |
| `replay_or_witness_pressure` | Replay fixture, reduced witness, retained witness, or run artifact exposes a semantic issue. | Bind to replay/witness bridge and classify outcome. |
| `scale_or_measurement_signal` | Performance/scaling run exposes phase timing or structural stress. | Treat as measurement input unless tied to semantic or pack obligation. |
| `historical_no_loss_input` | Bootstrap or rewrite-control docs preserve older intent. | Promote, map to current scope, defer, mark guardrail-only, or mark non-carry-forward. |
| `oxfml_upstream_source` | OxFml-owned spec, fixture, formal artifact, or upstream note. | Cite as upstream input; file handoff only for normative change pressure. |

## 3. Outcome Classes

| Outcome class | Meaning | Evidence minimum |
|---|---|---|
| `oxcalc_spec_patch` | OxCalc-owned spec should change. | Source finding plus rationale, and either artifact evidence or explicit deferred evidence obligation. |
| `oxcalc_artifact_obligation` | A proof/model/replay/test/pack artifact is needed before a stronger claim can be made. | Owning W033 bead or successor bead. |
| `oxfml_handoff_candidate` | Normative OxFml, FEC/F3E, formula-language, or LET/LAMBDA carrier change may be needed. | Handoff/watch row; no direct OxFml edit. |
| `implementation_fault_or_mismatch` | Current OxCalc behavior appears wrong against declared or newly clarified semantics. | Repro path, artifact, or concrete inspection note; successor implementation bead. |
| `tracecalc_oracle_gap` | TraceCalc does not cover or cannot yet express the behavior needed for oracle authority. | Oracle self-check or corpus widening obligation. |
| `intended_strategy_difference` | Two strategies differ internally but should preserve observable semantics. | Semantic-equivalence statement and comparison surface. |
| `deferred_open_lane` | The issue is valid but outside first-pass W033 evidence. | Rationale, owner, and blocker/successor packet path. |
| `guardrail_only` | The idea informs constraints but is not a first-pass claim or artifact obligation. | Rationale and affected guardrail. |
| `non_carry_forward` | Historical or proposed idea is intentionally not carried into current scope. | Explicit reason and conflict/non-goal citation. |
| `unresolved_evidence` | The finding is not yet strong enough to classify. | Required next evidence and owner. |

## 4. Evidence Strength Classes

| Strength | Meaning |
|---|---|
| `proof_checked` | Lean proof or equivalent checked formal proof exists for the stated claim. |
| `model_checked` | TLA+ or equivalent model run exists with declared bounds/config. |
| `tracecalc_oracle_checked` | TraceCalc reference-machine self-check exists for covered behavior. |
| `production_compared` | Production/core-engine output is compared to the declared observable surface. |
| `replay_backed` | Deterministic replay, fixture, or witness artifact exists. |
| `spec_only_deferred` | Spec text exists but evidence is explicitly deferred. |
| `measurement_only` | Timing/scale/counter evidence exists but is not semantic evidence by itself. |
| `source_only` | Current source/prose/historical text exists without exercised artifact evidence. |

## 5. Decision Record Schema

Every W033 decision row should contain:

1. `id`
2. `source`
3. `input_class`
4. `finding`
5. `decision`
6. `outcome_class`
7. `evidence_strength`
8. `owner`
9. `linked_artifacts`
10. `next_action`
11. `status`

Allowed statuses:

| Status | Meaning |
|---|---|
| `recorded` | Decision is captured for W033 and linked to a next action. |
| `patched_in_oxcalc` | OxCalc-owned spec text was patched in this repo. |
| `handoff_packet_needed` | Handoff/watch packet must be authored before promotion. |
| `artifact_needed` | Formal/replay/test/pack evidence is needed before claim promotion. |
| `deferred` | Valid lane remains outside first-pass execution with rationale. |
| `rejected_for_scope` | Idea is intentionally not carried into W033 scope. |

## 6. First Decision Ledger

| ID | Source | Input class | Finding | Decision | Outcome class | Evidence strength | Owner | Linked artifacts | Next action | Status |
|---|---|---|---|---|---|---|---|---|---|---|
| `W033-DEC-001` | User W033 direction; `CORE_ENGINE_OXCALC_OXFML_FORMALIZATION_PASS_PLAN.md`; `W033_OXCALC_OXFML_CORE_FORMALIZATION_PASS.md` | `current_spec_clause` | W033 must make the OxCalc + OxFml engine/seam contract trustworthy under change, not merely test current behavior. | Treat current specs and current implementation as evidence surfaces inside a controlled spec-evolution loop. | `oxcalc_spec_patch` | `spec_only_deferred` | OxCalc | plan/workset W033 intent sections | Use this ledger before later W033 spec patches. | `recorded` |
| `W033-DEC-002` | W033 scope plan | `current_spec_clause` | OxFml evaluator/FEC/F3E surfaces are in W033 scope as consumed seam/formal inputs, but OxFml ownership remains upstream. | Model and cite OxFml surfaces read-only; use handoff/watch packet for normative pressure. | `oxfml_handoff_candidate` | `spec_only_deferred` | OxCalc for W033 packet; OxFml for upstream truth | source-freeze packet; later handoff/watch packet | Feed OxFml-sensitive rows into `calc-uri.15`. | `recorded` |
| `W033-DEC-003` | W033 scope plan; OxFml formula-language docs | `oxfml_upstream_source` | General OxFunc semantic kernels are out of W033 scope, but LET/LAMBDA carrier facts affect OxCalc-visible dependency, trace, replay, and runtime-effect behavior. | Admit a narrow LET/LAMBDA carrier-boundary fragment and keep other OxFunc semantics opaque. | `oxcalc_artifact_obligation` | `spec_only_deferred` | OxCalc with OxFml/OxFunc boundary inputs | later `W033_OBJECT_VOCABULARY_AND_LET_LAMBDA_BOUNDARY.md` | Define carrier facts in `calc-uri.6`; hand off only if normative OxFml text is insufficient. | `artifact_needed` |
| `W033-DEC-004` | TraceCalc docs and W033 user direction | `tracecalc_behavior` | TraceCalc is expected to expand as the executable correctness oracle, but only for behavior it covers. | Separate TraceCalc oracle self-checks from production/core-engine conformance checks. | `tracecalc_oracle_gap` | `source_only` | OxCalc | later refinement, oracle self-check, conformance packets | Drive `calc-uri.7`, `calc-uri.8`, and `calc-uri.9`. | `artifact_needed` |
| `W033-DEC-005` | Source freeze packet | `current_spec_clause` | W033 needs a stable checked-in location for review ledgers and formalization packets. | Declare `docs/spec/core-engine/w033-formalization/` as the W033 spec-evidence root. | `oxcalc_spec_patch` | `source_only` | OxCalc | `W033_SOURCE_AUTHORITY_AND_ARTIFACT_LAYOUT.md` | Use the root for subsequent W033 packets. | `recorded` |
| `W033-DEC-006` | Source freeze packet; Foundation repository state | `current_spec_clause` | Foundation doctrine is higher-precedence but the Foundation worktree is dirty outside OxCalc. | Freeze the Foundation doctrine files by SHA-256 in the W033 source-freeze packet instead of pretending the Foundation commit alone identifies the inputs. | `deferred_open_lane` | `source_only` | OxCalc for local freeze; Foundation owns doctrine repo | `W033_SOURCE_AUTHORITY_AND_ARTIFACT_LAYOUT.md` | Recheck hashes during W033 closure audit if doctrine-sensitive claims are promoted. | `recorded` |
| `W033-DEC-007` | W033 plan; historical bootstrap/archive docs | `historical_no_loss_input` | Historical formal/theory and rewrite-control docs may preserve early intent but are not current authority. | Route historical ideas through the no-loss crosswalk before any promotion. | `guardrail_only` | `source_only` | OxCalc | later `W033_HISTORICAL_NO_LOSS_CROSSWALK.md` | Execute `calc-uri.4`. | `recorded` |
| `W033-DEC-008` | Current scale/performance lane and W033 plan | `scale_or_measurement_signal` | Performance and scaling measurements are useful but do not prove semantic correctness unless tied to observable semantics or pack obligations. | Classify scale evidence as measurement input by default; promote only through conformance, replay, or pack rows. | `deferred_open_lane` | `measurement_only` | OxCalc | later pack/capability binding and conformance packets | Feed into `calc-uri.9`, `calc-uri.10`, and `calc-uri.14`. | `recorded` |
| `W033-DEC-009` | Core spec review ledger | `current_spec_clause` | Later W033 spec edits need a stable review order and evidence expectation before mutations. | Use `W033_CORE_SPEC_REVIEW_LEDGER.md` as the first-pass sweep plan. | `oxcalc_artifact_obligation` | `source_only` | OxCalc | `W033_CORE_SPEC_REVIEW_LEDGER.md` | Authority matrix and vocabulary lanes consume this ledger. | `recorded` |
| `W033-DEC-010` | Historical no-loss crosswalk | `historical_no_loss_input` | Bootstrap and rewrite-control ideas should not be silently lost, but they also must not override current authority by nostalgia. | Use `W033_HISTORICAL_NO_LOSS_CROSSWALK.md` to classify historical ideas as current scope, promoted current spec, deferred open lane, guardrail-only, out-of-scope, or non-carry-forward. | `oxcalc_artifact_obligation` | `source_only` | OxCalc | `W033_HISTORICAL_NO_LOSS_CROSSWALK.md` | Authority matrix and closure audit must consume the crosswalk. | `recorded` |

## 7. Promotion Rule

A W033 decision does not promote a claim by being listed here. Promotion requires the linked evidence level named by the row:

1. OxCalc-owned spec patch, if the decision changes OxCalc-owned text.
2. OxFml handoff/watch row, if upstream ownership is involved.
3. Proof, model-check, TraceCalc oracle, production conformance, replay/witness, or pack/capability artifact, if the claim needs exercised evidence.
4. Deferred rationale, if W033 intentionally keeps the claim outside first-pass evidence.

## 8. Status

- execution_state: `spec_evolution_decision_ledger_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - decisions from detailed spec review are not yet recorded
  - historical no-loss decisions have an entry crosswalk but may widen during detailed sweeps
  - no OxCalc-owned semantic spec patch is made by this ledger
  - no OxFml handoff is filed by this ledger
  - TraceCalc, production conformance, Lean, TLA+, replay, pack, and closure evidence remain later W033 lanes
