# W033 Metamorphic Scale Semantic Binding

Status: `calc-8lg_evidence_packet`
Workset: `W033`
Successor bead: `calc-8lg`
Created: 2026-05-04

## 1. Purpose

This packet records the post-W033 binding slice for metamorphic, differential, and scale evidence.

The target is not a performance promotion. The target is to make scale evidence usable as semantic evidence only where it is tied to deterministic checks and replay/conformance artifacts.

The slice binds:

1. checked TreeCalc scale runs,
2. scale-signature metamorphic comparisons,
3. the TraceCalc scale seed replay artifact,
4. independent TreeCalc/TraceCalc conformance,
5. the pack-capability no-promotion decision.

## 2. Executable Surface

The binding runner is:

```powershell
cargo run -p oxcalc-tracecalc-cli -- scale-semantic-binding post-w033-metamorphic-scale-semantic-binding-001
```

The runner is `ScaleSemanticBindingRunner` in `src/oxcalc-tracecalc/src/scale_semantic_binding.rs`.

The runner reads existing scale/replay/conformance artifacts and emits a binding packet. It does not re-run million-node scale jobs during routine validation.

## 3. Evidence Artifacts

Artifact root:

`docs/test-runs/core-engine/metamorphic-scale-semantic-binding/post-w033-metamorphic-scale-semantic-binding-001/`

Generated files:

| Artifact | Role |
|---|---|
| `run_summary.json` | Machine-readable run summary. |
| `evidence/scale_semantic_evidence_index.json` | Per-scale-run closed-form semantic validation index. |
| `differentials/scale_signature_differential.json` | Metamorphic scale-signature comparisons. |
| `replay_conformance_bindings.json` | Replay/conformance/pack binding rows. |
| `decision/scale_no_promotion_decision.json` | Explicit no-promotion decision for performance, pack, continuous scale, and Stage 2 claims. |
| `replay-appliance/bundle_manifest.json` | Required-artifact manifest. |
| `replay-appliance/validation/bundle_validation.json` | Bundle validator result. |

Run summary:

| Measure | Value |
|---|---:|
| scale run rows | 7 |
| validated scale rows | 7 |
| scale signature rows | 5 |
| replay binding rows | 3 |
| missing artifacts | 0 |
| unexpected mismatches | 0 |
| no-promotion reasons | 5 |
| bundle validation | `bundle_valid`, `missing_paths: []` |

## 4. Scale Runs Bound

The binding consumes these checked scale roots:

| Run | Profile | Semantic role |
|---|---|---|
| `million_grid_r1` | `grid-cross-sum` | Static closed-form grid floor. |
| `million_grid_r2` | `grid-cross-sum` | Repeat run for semantic-signature invariance. |
| `million_indirect_r1` | `dynamic-indirect-stripes` | Dynamic potential carrier diagnostics while preserving static base result. |
| `million_fanout_f8_r1` | `fanout-bands` | Fanout edge-volume and invalidation floor. |
| `million_fanout_f8_calc1024_r1` | `fanout-bands` | Synthetic recalc amplification over the same dependency model. |
| `million_relative_rebind_f8_r1` | `relative-rebind-churn` | Soft-reference update/rebind seed derivation. |
| `million_fanout_f16_r1` | `fanout-bands` | Wider fanout edge-volume differential. |

Each row must have:

1. `validation.passed = true`,
2. every validation check passed,
3. node count at or above the million-node floor,
4. observed closed-form after-sum equals expected after-sum,
5. observed closed-form delta-sum equals expected delta-sum.

## 5. Metamorphic Rows

`scale_signature_differential.json` records five semantic-binding rows:

| Row | W033 family | Relation checked |
|---|---|---|
| `meta_scale_grid_repeat_invariance` | `W033-META-013` | Repeat grid run keeps model counts and closed-form outputs stable while timings may vary. |
| `meta_scale_calc_amplification_binding` | `W033-META-013` | 1024x synthetic recalc amplification keeps model/dependency surfaces unchanged and scales reference visits by 1024. |
| `meta_dynamic_indirect_semantic_binding` | `W033-META-006` | Dynamic potential carriers add 1,000,000 diagnostics/descriptors while preserving the static grid closed-form output. |
| `meta_relative_rebind_churn_binding` | `W033-META-001/W033-META-007/W033-META-013` | Relative anchor rename produces one rebind seed per formula owner. |
| `meta_fanout_edge_widening_binding` | `W033-META-013` | Fanout 16 widens edge volume relative to fanout 8 while closed-form validation still passes. |

All five rows have `comparison_state = semantic_binding_matched`.

## 6. Replay And Conformance Binding

The binding also checks three replay/conformance rows:

| Row | Evidence |
|---|---|
| `tracecalc_scale_seed_replay_binding` | `tc_scale_chain_seed_001` passes with no assertion failures, validation failures, or conformance mismatches, and its replay bundle is valid. |
| `independent_conformance_projection_binding` | `post-w033-independent-conformance-001` has 0 unexpected mismatches, 0 missing artifacts, and valid bundle status. |
| `pack_capability_no_promotion_binding` | `post-w033-pack-capability-decision-001` keeps `decision_status = capability_not_promoted` and `highest_honest_capability = cap.C4.distill_valid`. |

## 7. No-Promotion Decision

The decision artifact records:

1. `performance_claim_promoted = false`,
2. `pack_capability_promoted = false`,
3. `continuous_scale_assurance_promoted = false`,
4. `scale_semantic_evidence_recorded = true`,
5. `decision_status = semantic_binding_recorded_without_performance_promotion`.

No-promotion reason ids:

1. `scale.performance.measurement_not_a_correctness_proof`,
2. `scale.performance.single_day_baseline_not_continuous_assurance`,
3. `scale.performance.semantic_binding_not_scheduler_policy_promotion`,
4. `scale.performance.not_pack_grade_replay`,
5. `scale.performance.stage2_contention_not_promoted`.

## 8. Handoff Decision

Current decision: `no_new_handoff_required`.

Rationale:

1. scale semantic rows have no unexpected mismatch,
2. replay/conformance rows have no unexpected mismatch,
3. no OxFml-owned semantic contradiction appears,
4. no performance or pack claim is promoted,
5. `docs/handoffs/HANDOFF_REGISTER.csv` does not need a new row for this bead.

## 9. Semantic-Equivalence Statement

This bead adds an additive runner, CLI wiring, documentation, and generated binding artifacts.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update logic, recalculation logic, publication semantics, reject policy, TraceCalc behavior, TreeCalc behavior, OxFml fixture content, or pack capability decision logic.

Observable runtime behavior is invariant under this bead because the runner reads existing artifacts and emits a binding packet only.

## 10. Verification

Commands run:

| Command | Result |
|---|---|
| `cargo fmt -p oxcalc-tracecalc -p oxcalc-tracecalc-cli -- --check` | passed |
| `cargo test -p oxcalc-tracecalc scale_semantic_binding` | passed |
| `cargo run -p oxcalc-tracecalc-cli -- scale-semantic-binding post-w033-metamorphic-scale-semantic-binding-001` | passed; 7 scale rows, 5 signature rows, 3 replay binding rows |
| `cargo fmt --all -- --check` | passed |
| `cargo test --workspace` | passed: 49 `oxcalc_core` tests, 5 upstream-host tests, 10 `oxcalc_tracecalc` tests, 0 CLI tests, 0 doctest failures |
| `cargo clippy --workspace --all-targets -- -D warnings` | passed |
| `git diff --check` | passed; CRLF normalization warnings only |
| `scripts/check-worksets.ps1` | passed |
| `br dep cycles --json` | passed with `count: 0` |

## 11. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for in-scope items? | yes; this packet records runner, evidence inputs, artifact outputs, semantic rows, replay bindings, no-promotion decision, and handoff decision |
| 2 | Pack expectations updated for affected packs? | yes; the binding consumes the pack no-promotion decision and makes no C5 or performance promotion |
| 3 | At least one deterministic replay/projection artifact exists per in-scope behavior? | yes; the binding root records deterministic scale, replay, conformance, and pack artifacts |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 9 records that no runtime policy or strategy change was made |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no upstream mismatch was found, and no new handoff row is required |
| 6 | All required tests pass? | yes; see Section 10 |
| 7 | No known semantic gaps remain in declared target? | yes for this first semantic-binding target; continuous scale assurance and Stage 2 remain unpromoted |
| 8 | Completion language audit passed? | yes; this packet keeps scale evidence bound to deterministic checks and does not promote performance correctness, pack-grade replay, continuous scale assurance, or Stage 2 contention |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth did not change |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | not applicable; feature-map truth did not change |
| 11 | execution-state blocker surface updated? | yes; `calc-8lg` is represented in `.beads/` |

## 12. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-8lg` asks to turn W033 metamorphic, differential, and scaling families into exercised semantic evidence by binding scale/performance runs to replay, conformance, and semantic-equivalence criteria |
| Gate criteria re-read | pass; the target is a semantic-binding artifact and no-promotion decision, not a continuous scale suite or performance proof |
| Silent scope reduction check | pass; performance correctness, continuous assurance, pack-grade replay, and Stage 2 contention remain explicit non-promotions |
| "Looks done but is not" pattern check | pass; timing evidence is not reported as correctness evidence unless the deterministic closed-form and replay/conformance bindings also pass |
| Result | pass for the `calc-8lg` declared semantic-binding target |

## 13. Three-Axis Report

- execution_state: `calc-8lg_evidence_packet_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - continuous scale assurance remains unpromoted
  - performance correctness proof remains unpromoted
  - Stage 2 contention remains unpromoted
