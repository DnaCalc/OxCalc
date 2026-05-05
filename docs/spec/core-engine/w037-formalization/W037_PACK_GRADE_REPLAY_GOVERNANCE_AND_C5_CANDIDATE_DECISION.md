# W037 Pack-Grade Replay Governance And C5 Candidate Decision

Status: `calc-ubd.8_pack_c5_candidate_decision_validated`
Workset: `W037`
Parent epic: `calc-ubd`
Bead: `calc-ubd.8`

## 1. Purpose

This packet records the W037 pack-grade replay governance and C5 candidate decision.

The target is to reassess `cap.C5.pack_valid` after the W037 TraceCalc observable closure, optimized/core-engine conformance decision, direct OxFml evaluator slice, proof/model inventory, Stage 2 criteria, and operated-assurance service pilot. The decision remains no promotion: direct OxFml evaluator absence is removed for the exercised upstream-host slice, but pack-grade replay governance, full verification, operated service, Stage 2 replay equivalence, and fully independent evaluator diversity are still not satisfied.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W037_CORE_FORMALIZATION_FULL_VERIFICATION_PROMOTION_GATES.md` | W037 scope and pack/C5 exit-gate guardrails |
| `docs/spec/core-engine/w037-formalization/W037_RESIDUAL_FULL_VERIFICATION_AND_PROMOTION_GATE_LEDGER.md` | W037 obligations `W037-OBL-009`, `W037-OBL-013`, `W037-OBL-015`, `W037-OBL-016`, and `W037-OBL-017` |
| `docs/spec/core-engine/w037-formalization/W037_TRACECALC_OBSERVABLE_CLOSURE_AND_MULTI_READER_REPLAY.md` | TraceCalc observable closure input |
| `docs/spec/core-engine/w037-formalization/W037_OPTIMIZED_CORE_ENGINE_CONFORMANCE_IMPLEMENTATION_CLOSURE.md` | optimized/core-engine conformance input |
| `docs/spec/core-engine/w037-formalization/W037_DIRECT_OXFML_EVALUATOR_AND_LET_LAMBDA_SEAM_EVIDENCE.md` | direct OxFml and narrow callable seam input |
| `docs/spec/core-engine/w037-formalization/W037_LEAN_TLA_PROOF_MODEL_CLOSURE_INVENTORY.md` | proof/model inventory input |
| `docs/spec/core-engine/w037-formalization/W037_STAGE2_DETERMINISTIC_REPLAY_AND_PARTITION_PROMOTION_CRITERIA.md` | Stage 2 deterministic replay criteria input |
| `docs/spec/core-engine/w037-formalization/W037_OPERATED_CONTINUOUS_ASSURANCE_AND_CROSS_ENGINE_SERVICE_PILOT.md` | operated assurance/service pilot input |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | inbound OxFml watch ledger, including current formatting guardrails |

## 3. Implementation Changes

The existing pack-capability runner now has a W037 evidence profile.

1. The W037 profile consumes W037 TraceCalc, TreeCalc/CoreEngine, direct OxFml, proof/model, Stage 2, and continuous-assurance/service-pilot artifacts.
2. The W037 profile removes the predecessor `direct_oxfml_evaluator_reexecution_absent` blocker for the exercised direct upstream-host slice.
3. The W037 profile keeps C5 blocked by full-verification, optimized/core-engine, operated-service, Stage 2 replay, and pack-governance gaps.
4. The runner emits the decision root under `docs/test-runs/core-engine/pack-capability/w037-pack-c5-candidate-decision-001/`.

No runtime evaluator, coordinator, dependency graph, soft-reference, recalc, publication, formal proof/model, continuous-service, or OxFml behavior changes in this bead.

## 4. Evidence Summary

Run id: `w037-pack-c5-candidate-decision-001`

| Metric | Value |
|---|---:|
| Satisfied inputs | 13 |
| No-promotion blockers | 22 |
| Missing artifacts | 0 |
| Highest honest capability | `cap.C4.distill_valid` |
| Target capability | `cap.C5.pack_valid` |
| Capability promoted | no |
| Stage 2 scheduler promoted | no |

Primary artifacts:

1. `docs/test-runs/core-engine/pack-capability/w037-pack-c5-candidate-decision-001/run_summary.json`
2. `docs/test-runs/core-engine/pack-capability/w037-pack-c5-candidate-decision-001/evidence/evidence_index.json`
3. `docs/test-runs/core-engine/pack-capability/w037-pack-c5-candidate-decision-001/decision/pack_capability_decision.json`
4. `docs/test-runs/core-engine/pack-capability/w037-pack-c5-candidate-decision-001/replay-appliance/bundle_manifest.json`
5. `docs/test-runs/core-engine/pack-capability/w037-pack-c5-candidate-decision-001/replay-appliance/validation/bundle_validation.json`

## 5. C5 Candidate Decision

The W037 decision keeps `highest_honest_capability=cap.C4.distill_valid`.

The C5 gate remains blocked because W037 still lacks:

1. operated pack-grade replay governance,
2. full TraceCalc oracle authority,
3. full optimized/core-engine verification,
4. full Lean/TLA verification,
5. Stage 2 deterministic partition replay and pack-grade replay equivalence,
6. an operated continuous-assurance service,
7. an operated continuous cross-engine differential service,
8. an enforcing alert/quarantine service,
9. fully independent evaluator diversity,
10. general OxFunc kernel verification inside the appropriate owner boundary.

## 6. Promotion Guard

The W037 pack decision forbids these conversions:

| Evidence type | Forbidden promotion |
|---|---|
| Direct OxFml upstream-host slice | pack-grade replay or C5 |
| W037 TraceCalc observable closure | full oracle authority |
| W037 conformance decision | full optimized/core-engine verification |
| W037 proof/model inventory | full formal verification |
| W037 Stage 2 criteria | Stage 2 scheduler policy |
| W037 service-readiness pilot | operated assurance or continuous differential service |
| Timing measurements | semantic correctness proof |

## 7. OxFml Watch

No OxFml handoff is filed by this bead.

The latest OxFml formatting update remains compatible with the W037 direct upstream-host slice: W073 aggregate/visualization conditional-formatting metadata is carried through `typed_rule`, while `thresholds` remains scalar/operator/expression input text. The pack decision consumes that evidence as a satisfied input for the exercised slice only.

## 8. Semantic-Equivalence Statement

This bead adds W037 pack/capability evidence binding, decision emission, tests, checked artifacts, and documentation only.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc execution semantics, TreeCalc/CoreEngine runtime behavior, Lean/TLA model semantics, continuous-assurance runner semantics, pack runtime behavior, service behavior, alert dispatch behavior, or OxFml/OxFunc evaluator behavior.

Observable runtime behavior is invariant under this bead. The W037 pack decision reads existing evidence and writes a no-promotion decision packet; it does not change executable calculator behavior.

## 9. Verification

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `cargo test -p oxcalc-tracecalc pack_capability -- --nocapture` | passed; 5 tests |
| `cargo run -p oxcalc-tracecalc-cli -- pack-capability w037-pack-c5-candidate-decision-001` | passed; emitted 13 satisfied inputs, 22 blockers, and highest honest capability `cap.C4.distill_valid` |
| JSON parse for `docs/test-runs/core-engine/pack-capability/w037-pack-c5-candidate-decision-001/**/*.json` | passed; 5 JSON files parsed |
| `cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `scripts/check-worksets.ps1` | passed after bead closure; worksets=15, beads total=99, open=2, in_progress=0, ready=1, blocked=0, closed=97 |
| `br ready --json` | passed; next ready bead is `calc-ubd.9` |
| `br dep cycles --json` | passed; `count: 0` |
| `git diff --check` | passed; line-ending warnings only |

## 10. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet and generated W037 pack artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; this packet is the W037 pack/C5 candidate decision and keeps C5 unpromoted |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for decision emission; pack-grade replay governance remains absent and is explicitly blocked |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 8 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no OxFml-owned seam contradiction or handoff trigger appeared |
| 6 | All required tests pass? | yes; commands in Section 9 passed |
| 7 | No known semantic gaps remain in declared scope? | yes for this target after artifact generation; remaining C5 blockers are explicit |
| 8 | Completion language audit passed? | yes; no C5, Stage 2, full verification, operated service, or pack-grade replay promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W037 pack/C5 decision |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-ubd.8` closed and `calc-ubd.9` ready |

## 11. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-ubd.8` asks for W037 pack-grade replay governance and C5 candidate reassessment |
| Gate criteria re-read | pass; the decision is machine-readable and does not promote from proxy, bounded, or file-backed-only evidence |
| Silent scope reduction check | pass; remaining blockers are explicit |
| "Looks done but is not" pattern check | pass; direct OxFml evidence, service-readiness evidence, and bounded proof/model evidence are not represented as C5 |
| Result | pass for the `calc-ubd.8` target after final validation; W037 remains scope-partial |

## 12. Three-Axis Report

- execution_state: `calc-ubd.8_pack_c5_candidate_decision_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-ubd.9` closure audit and full-verification release decision remains open
  - `cap.C5.pack_valid` remains unpromoted
  - pack-grade replay governance remains absent
  - full TraceCalc oracle authority, full optimized/core-engine verification, full independent evaluator diversity, operated continuous-assurance service, operated cross-engine differential service, full Lean/TLA verification, pack-grade Stage 2 replay equivalence, and Stage 2 policy remain partial
