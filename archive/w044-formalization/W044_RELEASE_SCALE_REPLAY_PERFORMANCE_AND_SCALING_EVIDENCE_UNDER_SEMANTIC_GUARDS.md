# W044 Release-Scale Replay Performance And Scaling Evidence Under Semantic Guards

Status: `spec_drafted_with_checked_evidence`

Bead: `calc-b1t.9`

Run id: `w044-release-scale-replay-performance-scaling-001`

## Purpose

This packet records the W044 release-scale replay/performance tranche after the W044 OxFml seam packet.

The narrow new evidence is a W044 scale semantic-binding profile plus regenerated million-node TreeCalc scale artifacts. The profile binds timing and scale measurements to semantic guards before any interpretation: closed-form validation, metamorphic scale signatures, TraceCalc scale-seed replay, independent conformance, predecessor pack no-promotion, W044 optimized/core evidence, W044 Rust and Lean/TLA evidence, W044 Stage 2 scheduler/pack equivalence evidence, W044 operated-assurance evidence, W044 diversity/mismatch evidence, and W044 OxFml typed-formatting evidence.

The packet does not promote performance-derived correctness, continuous scale assurance, pack-grade replay, C5, Stage 2 production policy, release-grade verification, operated services, broad OxFml closure, full Lean/TLA verification, Rust-engine totality, Rust refinement, or general OxFunc kernels.

## Evidence Surfaces

| Artifact | Purpose |
|---|---|
| `docs/test-runs/core-engine/treecalc-scale/million_grid_r1/run_summary.json` | million-node grid cross-sum run, closed-form oracle, dependency-build timing, pure-recalc timing |
| `docs/test-runs/core-engine/treecalc-scale/million_grid_r2/run_summary.json` | repeat grid run for semantic signature invariance |
| `docs/test-runs/core-engine/treecalc-scale/million_indirect_r1/run_summary.json` | million-node dynamic `INDIRECT`-shaped potential carrier run with 1,000,000 dynamic diagnostics |
| `docs/test-runs/core-engine/treecalc-scale/million_fanout_f8_r1/run_summary.json` | million-node fanout-8 run with 7,999,928 descriptors and edges |
| `docs/test-runs/core-engine/treecalc-scale/million_fanout_f8_calc1024_r1/run_summary.json` | fanout-8 run with 1024 closed-form recalc rounds and 8,191,926,272 reference visits |
| `docs/test-runs/core-engine/treecalc-scale/million_relative_rebind_f8_r1/run_summary.json` | million-node relative rebind run with 999,991 rebind seeds and explicit soft-reference timing |
| `docs/test-runs/core-engine/treecalc-scale/million_fanout_f16_r1/run_summary.json` | fanout-16 widening run with 15,999,728 descriptors and edges |
| `archive/test-runs-core-engine-w038-w045/metamorphic-scale-semantic-binding/w044-release-scale-replay-performance-scaling-001/run_summary.json` | W044 scale semantic-binding summary |
| `archive/test-runs-core-engine-w038-w045/metamorphic-scale-semantic-binding/w044-release-scale-replay-performance-scaling-001/evidence/scale_semantic_evidence_index.json` | 7 validated scale rows with model shape, semantic surfaces, timing surfaces, and no timing-only claim |
| `archive/test-runs-core-engine-w038-w045/metamorphic-scale-semantic-binding/w044-release-scale-replay-performance-scaling-001/differentials/scale_signature_differential.json` | 6 signature rows, including phase timing split guard |
| `archive/test-runs-core-engine-w038-w045/metamorphic-scale-semantic-binding/w044-release-scale-replay-performance-scaling-001/replay_conformance_bindings.json` | 10 replay/conformance/W044 semantic guard binding rows |
| `archive/test-runs-core-engine-w038-w045/metamorphic-scale-semantic-binding/w044-release-scale-replay-performance-scaling-001/decision/continuous_scale_assurance_criteria.json` | W044 criteria states and no-promotion consequences |
| `archive/test-runs-core-engine-w038-w045/metamorphic-scale-semantic-binding/w044-release-scale-replay-performance-scaling-001/decision/scale_no_promotion_decision.json` | explicit no-promotion decision |
| `archive/test-runs-core-engine-w038-w045/metamorphic-scale-semantic-binding/w044-release-scale-replay-performance-scaling-001/replay-appliance/validation/bundle_validation.json` | bundle validation: 27 required artifacts, 0 missing, 0 mismatches |

## Result

The W044 scale semantic-binding run records:

1. 7 scale rows.
2. 7 validated scale rows.
3. 6 scale signature rows.
4. 10 replay/conformance/W044 guard binding rows.
5. 0 missing artifacts.
6. 0 unexpected mismatches.
7. 9 no-promotion reasons.
8. `bundle_valid` with 27 required artifacts validated.

The W044 criteria states:

1. `scale.semantic.closed_form_validation`: `satisfied`.
2. `scale.semantic.metamorphic_signature_binding`: `satisfied`.
3. `scale.semantic.replay_conformance_pack_binding`: `satisfied`.
4. `scale.w044.phase_timing_split`: `satisfied`.
5. `scale.w044.semantic_guard_binding`: `satisfied`.
6. `scale.continuous.scheduled_regression_floor`: `missing`.
7. `scale.continuous.cross_engine_diff_service`: `missing`.

## Model Shapes And Oracles

The release-scale rows use deterministic closed-form oracles rather than timing as correctness evidence:

1. `grid-cross-sum`: 1,002,001 nodes, 1,000,000 formulas, 2,000,000 static direct descriptors, `left_row + top_col`, expected sum after edit `92,299,000`, expected delta `18,000`.
2. `dynamic-indirect-stripes`: 1,002,001 nodes, 1,000,000 formulas, 2,000,000 static direct edges, 1,000,000 dynamic-potential descriptors/diagnostics, same static closed-form result as the grid baseline.
3. `fanout-bands` with fanout 8: 1,000,000 nodes, 999,991 formulas, 7,999,928 static descriptors/edges, expected after sum `42,999,613`, expected delta `6,999,937`.
4. `fanout-bands` with fanout 8 and 1024 recalc rounds: same model shape as fanout 8, 1,023,990,784 formula evaluations, 8,191,926,272 reference visits, expected after sum `44,031,603,712`, expected delta `7,167,935,488`.
5. `relative-rebind-churn` with fanout 8: 1,000,000 nodes, 999,991 formulas, 7,999,928 relative-bound descriptors/edges, 999,991 expected rebind seeds.
6. `fanout-bands` with fanout 16: 1,000,000 nodes, 999,983 formulas, 15,999,728 descriptors/edges, expected after sum `142,997,569`, expected delta `6,999,881`.

The relative-rebind scale profile uses `closed_form_scale_owner_scan` for the million-owner soft-reference update phase. Small/smoke `treecalc_scale` tests still exercise the general structural rebind derivation. The large-scale owner scan is a scale-model timing strategy for this synthetic profile, not proof of the full general structural rebind algorithm.

## Phase Timing Split

Every W044 scale row records these timing phases:

1. `model_build_structural_snapshot_and_formula_catalog`
2. `dependency_descriptor_lowering`
3. `dependency_graph_build_and_cycle_scan`
4. `soft_reference_update_rebind_seed_derivation`
5. `invalidation_closure_derivation`
6. `synthetic_closed_form_recalc`
7. `validation_checks`

The phase split is measurement evidence. It distinguishes dependency lowering, dependency graph build, soft-reference update, invalidation closure, and pure recalc, but it does not prove correctness independently of the closed-form validation and W044 semantic guard rows.

## W044 Semantic Guard Binding

The W044 profile requires these guard rows before interpreting scale evidence:

1. optimized/core dynamic-transition evidence: 6 disposition rows, 2 direct-evidence rows, 0 failed rows, 0 match-promoted rows.
2. Rust totality/refinement evidence: 11 local proof rows, 9 refinement rows, 0 failed rows, no Rust totality/refinement/pack promotion.
3. Lean/TLA evidence: 10 local proof rows, 4 bounded model rows, 0 failed rows, no full Lean/TLA or unbounded model promotion.
4. Stage 2 evidence: 25 policy rows, 5 observable-invariance rows, scheduler and pack equivalence evidenced for declared profiles, no Stage 2 or pack promotion.
5. operated-assurance evidence: 25 readiness criteria, 40 retained-history rows, file-backed envelope present, no operated-service or retention-SLO promotion.
6. diversity/mismatch evidence: 8 independent reference-model cases, 8 matches, 25 accepted boundaries, no independent-evaluator/mismatch-quarantine/operated-differential service promotion.
7. OxFml evidence: W073 OxCalc fixture request construction verified for the `typed_rule` direct-replacement path, old W072 bounded threshold strings intentionally ignored for aggregate/visualization families, downstream DNA OneCalc uptake unverified, no broad OxFml or callable metadata promotion.

## No-Promotion Reasons

The W044 scale packet retains these no-promotion reasons:

1. `scale.performance.measurement_not_a_correctness_proof`
2. `scale.performance.single_day_baseline_not_continuous_assurance`
3. `scale.performance.semantic_binding_not_scheduler_policy_promotion`
4. `scale.performance.not_pack_grade_replay`
5. `scale.performance.stage2_contention_not_promoted`
6. `scale.w044.phase_timing_split_is_measurement_only`
7. `scale.w044.semantic_guard_binding_required_before_pack_reassessment`
8. `scale.w044.no_operated_continuous_scale_service`
9. `scale.w044.no_release_grade_correctness_from_performance`

## Semantic-Equivalence Statement

The W044 scale runner changes only the emitted scale-model evidence shape and W044 semantic-binding classification. The large synthetic rebind profile uses a closed-form owner scan for million-owner soft-reference timing while retaining exact rebind-seed validation and retaining the general derivation in smoke tests.

This packet does not change coordinator scheduling, dependency graph semantics, invalidation semantics, soft-reference semantics, recalc semantics, publication, reject behavior, TraceCalc semantics, OxFml semantics, OxFunc semantics, Lean proof semantics, TLA model semantics, Stage 2 policy, operated services, pack decisions, or release-grade capability. Observable runtime behavior is invariant under this packet.

## Validation

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `cargo test -p oxcalc-core treecalc_scale -- --nocapture` | passed; 4 tests |
| `cargo test -p oxcalc-tracecalc scale_semantic_binding -- --nocapture` | passed; 3 tests |
| 7 `treecalc-scale` release-scale source runs | passed; regenerated W044 scale-source artifacts |
| `cargo run -p oxcalc-tracecalc-cli -- scale-semantic-binding w044-release-scale-replay-performance-scaling-001` | passed; emitted 7 scale rows, 6 signature rows, 10 replay binding rows |
| JSON parse for W044 scale artifacts | passed; 36 JSON artifacts parsed |
| `cargo test -p oxcalc-tracecalc` | passed; 74 tests plus doctests |
| `cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `scripts/check-worksets.ps1` | passed post-close; worksets=22, beads total=175, open=3, in_progress=0, ready=1, blocked=1, closed=172 |
| `br ready --json` | passed post-close; `calc-b1t.10` ready |
| `br dep cycles --json` | passed; 0 cycles |
| `git diff --check` | passed with CRLF normalization warnings only |

## Status Report

- execution_state: `calc-b1t.9_release_scale_replay_performance_scaling_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - continuous scale assurance remains blocked by absent recurring scheduled regression service
  - operated cross-engine differential service remains blocked
  - performance-derived correctness remains unpromoted
  - pack-grade replay governance and C5 reassessment remain owned by `calc-b1t.10`
  - release-grade verification decision remains owned by `calc-b1t.11`
  - full Lean/TLA verification, Rust-engine totality, Rust refinement, Stage 2 production policy, broad OxFml closure, public migration verification, callable metadata/carrier, registered-external projection, provider-publication, and general OxFunc kernels remain unpromoted

## Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W044 README/status surfaces, feature map, runner, tests, TreeCalc scale artifacts, and W044 scale semantic-binding artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack-grade replay and C5 remain unpromoted and `calc-b1t.10` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; W044.9 emits scale-source run summaries, signature differentials, replay/conformance bindings, W044 guard rows, criteria, no-promotion decision, and bundle validation |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no runtime strategy changed, and the scale-runner synthetic owner-scan boundary is explicit |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no OxFml handoff is triggered by scale/performance evidence |
| 6 | All required tests pass? | yes; see Validation |
| 7 | No known semantic gaps remain in declared scope? | yes for the W044.9 evidence target; broader exact blockers are explicit |
| 8 | Completion language audit passed? | yes; no performance correctness, continuous scale assurance, pack/C5, Stage 2, release-grade, broad OxFml, callable, Rust totality/refinement, full Lean/TLA, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; no ordered workset-truth change in this bead |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W044 scale update |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-b1t.9` closed and `calc-b1t.10` ready |

## Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-b1t.9` asks for release-scale replay/performance evidence under semantic guards, not performance-derived correctness promotion |
| Gate criteria re-read | pass; model shape, validation oracle, phase timing split, semantic guards, and no-promotion consequences are named |
| Silent scope reduction check | pass; continuous service, cross-engine service, pack/C5, Stage 2, release-grade, broad OxFml, callable, Rust, Lean/TLA, and general OxFunc blockers remain explicit |
| "Looks done but is not" pattern check | pass; timing data is subordinate to closed-form validation, replay/conformance binding, and W044 semantic guard rows |
| Result | pass for the `calc-b1t.9` target |
