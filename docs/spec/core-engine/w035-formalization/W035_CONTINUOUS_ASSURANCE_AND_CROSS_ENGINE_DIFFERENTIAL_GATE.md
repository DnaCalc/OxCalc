# W035 Continuous Assurance And Cross-Engine Differential Gate

Status: `calc-tkq.6_continuous_assurance_gate_validated`
Workset: `W035`
Parent epic: `calc-tkq`
Bead: `calc-tkq.6`

## 1. Purpose

This packet moves W035 beyond single-run scale evidence by defining a reproducible continuous-assurance gate and cross-engine differential criteria.

The target is not to rerun million-node scale inside this bead. The target is to bind existing W034/W035 evidence into an explicit gate that says what recurring evidence must exist before continuous-scale, pack, or Stage 2 promotion can be considered.

## 2. Authority Inputs Reviewed

| Input | Role in this bead |
|---|---|
| `docs/worksets/W035_CORE_FORMALIZATION_PROOF_AND_ASSURANCE_HARDENING.md` | W035 scope and `calc-tkq.6` gate |
| `docs/spec/core-engine/w035-formalization/W035_RESIDUAL_PROOF_OBLIGATION_AND_SPEC_EVOLUTION_LEDGER.md` | W035 obligations for independent diversity and continuous assurance |
| `docs/spec/core-engine/w035-formalization/W035_TRACECALC_ORACLE_MATRIX_EXPANSION.md` | W035 TraceCalc oracle matrix source |
| `docs/spec/core-engine/w035-formalization/W035_IMPLEMENTATION_CONFORMANCE_HARDENING.md` | W035 implementation-conformance source |
| `docs/spec/core-engine/w035-formalization/W035_LEAN_ASSUMPTION_DISCHARGE_AND_SEAM_PROOF_MAP.md` | W035 formal proof-map source |
| `docs/spec/core-engine/w035-formalization/W035_TLA_NON_ROUTINE_EXPLORATION_AND_SCHEDULER_PRECONDITIONS.md` | W035 TLA/scheduler-precondition source |
| `docs/spec/core-engine/w034-formalization/W034_PACK_CAPABILITY_AND_CONTINUOUS_SCALE_GATE_BINDING.md` | predecessor pack/scale gate |
| `docs/test-runs/core-engine/metamorphic-scale-semantic-binding/w034-continuous-scale-gate-binding-001/` | W034 scale semantic binding source |
| `docs/test-runs/core-engine/pack-capability/w034-pack-capability-gate-binding-001/` | W034 pack capability source |

## 3. Implementation Surface

| Surface | Change |
|---|---|
| `src/oxcalc-tracecalc/src/continuous_assurance.rs` | adds `ContinuousAssuranceRunner` and machine-readable W035 gate packet emission |
| `src/oxcalc-tracecalc/src/lib.rs` | exports the continuous-assurance module |
| `src/oxcalc-tracecalc-cli/src/main.rs` | adds `continuous-assurance <run-id>` command |
| `docs/test-runs/core-engine/continuous-assurance/w035-continuous-assurance-gate-001/` | checked-in deterministic gate artifact root |

## 4. Gate Run Summary

Run id: `w035-continuous-assurance-gate-001`

| Metric | Value |
|---|---:|
| Source evidence rows | 5 |
| Scheduled lanes | 3 |
| Cross-engine differential rows | 4 |
| Missing artifacts | 0 |
| Unexpected mismatches | 0 |
| No-promotion reasons | 9 |
| Bundle validation | `bundle_valid` |
| Required artifacts validated | 21 |
| Decision status | `continuous_assurance_gate_defined_without_promotion` |

Primary artifacts:

1. `docs/test-runs/core-engine/continuous-assurance/w035-continuous-assurance-gate-001/run_summary.json`
2. `docs/test-runs/core-engine/continuous-assurance/w035-continuous-assurance-gate-001/evidence/source_evidence_index.json`
3. `docs/test-runs/core-engine/continuous-assurance/w035-continuous-assurance-gate-001/schedule/continuous_assurance_schedule.json`
4. `docs/test-runs/core-engine/continuous-assurance/w035-continuous-assurance-gate-001/differentials/cross_engine_differential_gate.json`
5. `docs/test-runs/core-engine/continuous-assurance/w035-continuous-assurance-gate-001/decision/continuous_assurance_decision.json`
6. `docs/test-runs/core-engine/continuous-assurance/w035-continuous-assurance-gate-001/replay-appliance/bundle_manifest.json`
7. `docs/test-runs/core-engine/continuous-assurance/w035-continuous-assurance-gate-001/replay-appliance/validation/bundle_validation.json`

## 5. Source Evidence Rows

| Source row | Evidence state |
|---|---|
| W034 scale semantic binding | semantic scale binding present, no continuous promotion |
| W034 pack capability binding | pack binding present, C5 not promoted |
| W035 TraceCalc oracle matrix | oracle matrix present with classified uncovered rows |
| W035 implementation conformance | gap dispositions present without promotion |
| W035 Lean/TLA formal packets | bounded formal packets present, no full-verification promotion |

The source rows are inputs to the gate. They do not become promotion proof merely because they are present.

## 6. Scheduled Lanes

The gate defines three scheduled lanes:

| Lane | Cadence | Minimum acceptance |
|---|---|---|
| `continuous.semantic.smoke` | per-change or nightly | zero failed oracle-matrix rows, zero failed implementation-conformance rows, zero source missing artifacts |
| `continuous.scale.regression` | scheduled weekly or release-candidate | closed-form scale validation passes, metamorphic signature rows remain matched, timing changes reported as measurement |
| `continuous.cross_engine.diff` | release-candidate and before pack promotion | no unexpected mismatches, declared gaps not counted as matches, C5 only after blockers are cleared |

The packet defines these lanes but does not claim a running scheduler, alerting system, or history window exists yet.

## 7. Cross-Engine Differential Gate

| Row | Current state | Limit |
|---|---|---|
| TraceCalc oracle matrix to engine projection | bounded semantic match with classified uncovered rows | TraceCalc covers W035 matrix rows, not the full engine universe |
| implementation conformance gap dispositions | valid dispositions without match promotion | implementation-work and spec-evolution deferrals remain |
| scale semantic signatures | scale signatures matched without continuous service | single checked-in W034 binding is not recurring assurance |
| independent evaluator diversity | not fully independent | TreeCalc/CoreEngine projection is useful but not full independent evaluator implementation diversity |

The differential gate is intentionally stricter than the current evidence. It turns the missing continuous and diversity surfaces into explicit blockers rather than letting them remain implicit.

## 8. No-Promotion Decision

The W035 gate decision keeps these surfaces unpromoted:

1. continuous scale assurance,
2. cross-engine differential service,
3. pack capability `cap.C5.pack_valid`,
4. Stage 2 scheduler policy,
5. performance-derived correctness claims.

No-promotion reason ids:

1. `continuous.no_scheduled_regression_runner`
2. `continuous.no_cross_engine_diff_service`
3. `continuous.no_history_window_for_regression_thresholds`
4. `continuous.no_alerting_or_quarantine_policy`
5. `continuous.performance_not_correctness_proof`
6. `continuous.independent_evaluator_diversity_not_full`
7. `continuous.pack_c5_not_promoted`
8. `continuous.stage2_scheduler_not_promoted`
9. `continuous.formal_evidence_bounded_not_full_verification`

## 9. Obligation Disposition

| Obligation | `calc-tkq.6` disposition |
|---|---|
| `W035-OBL-006` | fully independent evaluator diversity remains unpromoted and is now an explicit cross-engine gate blocker |
| `W035-OBL-011` | continuous assurance packet shape, schedule lanes, cross-engine differential criteria, and no-promotion decision are machine-readable and checked in |

## 10. Semantic-Equivalence Statement

This bead adds a gate runner, CLI command, emitted gate artifacts, and spec text. It reads existing W034/W035 evidence and emits criteria. It does not change coordinator scheduling, dependency graph construction, soft-reference resolution, recalc semantics, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc semantics, TreeCalc semantics, Lean/TLA artifacts, pack-decision logic, or OxFml evaluator behavior.

Observable runtime behavior is invariant under this bead. The new runner classifies evidence and writes assurance-gate artifacts only.

## 11. Verification

| Command | Result |
|---|---|
| `cargo test -p oxcalc-tracecalc continuous_assurance` | passed; 1 test |
| `cargo run -p oxcalc-tracecalc-cli -- continuous-assurance w035-continuous-assurance-gate-001` | passed; emitted 5 source rows, 3 scheduled lanes, 4 differential rows, 9 no-promotion reasons |
| `cargo test -p oxcalc-tracecalc` | passed |
| `cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `cargo fmt --all -- --check` | passed |
| `scripts/check-worksets.ps1` | passed |
| `br dep cycles --json` | passed; `count=0` |
| `git diff --check` | passed; CRLF normalization warnings only |

## 12. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W035 workset status, W035 ledger, spec index, and feature-map surfaces record the continuous-assurance gate |
| 2 | Pack expectations updated for affected packs? | yes; pack C5 remains unpromoted and `calc-tkq.7` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; this bead emits deterministic checked-in gate artifacts that bind existing W034/W035 replay/conformance/scale inputs |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 10 states no runtime policy or strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no OxFml-owned seam contradiction or handoff trigger appeared |
| 6 | All required tests pass? | yes; focused runner, CLI emission, package tests, CLI tests, format, workset, bead graph, and diff checks passed |
| 7 | No known semantic gaps remain in declared scope? | yes for this target; missing continuous service, history, alerting, independent diversity, C5, and Stage 2 surfaces are explicit blockers |
| 8 | Completion language audit passed? | yes; no continuous-scale, cross-engine service, C5, Stage 2, or performance-correctness promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 points at this continuous-assurance gate evidence |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-tkq.6` execution state and later closure evidence |

## 13. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-tkq.6` asks for scheduled/continuous assurance packet shape and cross-engine differential gate criteria |
| Gate criteria re-read | pass; performance data remains semantic-bound, and promotion is blocked without recurring evidence and differential criteria |
| Silent scope reduction check | pass; this bead defines the gate and explicitly does not claim a running continuous service |
| "Looks done but is not" pattern check | pass; single-run scale evidence is not represented as continuous assurance or correctness proof |
| Result | pass for the `calc-tkq.6` continuous-assurance gate target |

## 14. Three-Axis Report

- execution_state: `calc-tkq.6_continuous_assurance_gate_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-tkq.7` and `calc-tkq.8` remain open
  - full Lean/TLA verification remains open
  - full TraceCalc oracle coverage remains open
  - full optimized/core-engine verification and fully independent evaluator diversity remain open beyond W035 conformance-hardening dispositions
  - concrete Stage 2 partition modeling, pack-grade replay, continuous-scale service operation, and Stage 2 policy remain unpromoted
