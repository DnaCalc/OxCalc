# W039 Optimized Core Exact Blocker Implementation Closure

Status: `calc-f7o.2_optimized_core_exact_blocker_disposition_validated`
Workset: `W039`
Parent epic: `calc-f7o`
Bead: `calc-f7o.2`

## 1. Purpose

This packet attacks the W039 optimized/core exact-blocker target.

The result is deliberately non-promoting. W039 adds a real implementation-conformance runner profile and CLI path for the post-W038 exact-blocker packet, binds the W039 obligation ledger to the W038 exact blocker register, and keeps exact blockers where direct optimized/core, coordinator, fixture, or proof evidence is still absent.

The target is not to claim full optimized/core verification. The target is to ensure that `calc-f7o.3` starts from machine-readable exact blockers rather than prose memory.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/spec/core-engine/w039-formalization/W039_RESIDUAL_SUCCESSOR_OBLIGATION_LEDGER_AND_PROMOTION_READINESS_MAP.md` | W039 obligations `W039-OBL-002` through `W039-OBL-005` and `W039-OBL-008` |
| `docs/test-runs/core-engine/release-grade-ledger/w039-residual-successor-obligation-ledger-001/successor_obligation_ledger.json` | machine-readable W039 obligation owners and promotion consequences |
| `docs/spec/core-engine/w038-formalization/W038_OPTIMIZED_CORE_ENGINE_CONFORMANCE_BLOCKER_CLOSURE_AND_FIXES.md` | predecessor optimized/core disposition packet |
| `docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/w038_exact_remaining_blocker_register.json` | four W038 exact remaining blockers |
| `docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/w038_match_promotion_guard.json` | zero W038 match promotions |
| `docs/test-runs/core-engine/release-grade-ledger/w039-residual-successor-obligation-ledger-001/w073_formatting_intake.json` | W073 typed-only formatting guard retained as seam context |

## 3. Artifact Surface

Run id: `w039-optimized-core-exact-blocker-disposition-001`

| Artifact | Result |
|---|---|
| `docs/test-runs/core-engine/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/run_summary.json` | records 5 W039 disposition rows, 2 direct-evidence rows, 4 exact blockers, 0 match-promoted rows, and 0 failed rows |
| `docs/test-runs/core-engine/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/w039_exact_blocker_disposition_register.json` | machine-readable disposition rows binding W039 obligations to W038 exact blockers |
| `docs/test-runs/core-engine/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/w039_exact_remaining_blocker_register.json` | four retained exact blockers and owner lanes |
| `docs/test-runs/core-engine/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/w039_match_promotion_guard.json` | zero match-promoted rows; W039 declared-gap guard holds |
| `docs/test-runs/core-engine/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/evidence_summary.json` | source evidence index, W039 obligation bindings, W038 inputs, and W073 formatting-intake guard |
| `docs/test-runs/core-engine/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/validation.json` | runner validation status `implementation_conformance_w039_exact_blockers_valid` |

## 4. Disposition Summary

| W039 row | Source blocker | W039 disposition | Evidence consequence |
|---|---|---|---|
| `w039_dynamic_release_reclassification_exact_blocker` | W038 dynamic negative/release/reclassification | retained exact blocker after W038 direct evidence | W038 bound dynamic rejection, resolved publication, and retention-release evidence; release/reclassification differential remains absent |
| `w039_snapshot_fence_counterpart_exact_blocker` | W038 snapshot-fence projection | retained Stage 2/coordinator blocker | stale accepted-candidate counterpart remains absent |
| `w039_capability_view_fence_counterpart_exact_blocker` | W038 capability-view fence projection | retained Stage 2/coordinator blocker | compatibility-fenced capability-view mismatch counterpart remains absent |
| `w039_callable_metadata_projection_exact_blocker` | W038 callable metadata projection | retained callable metadata blocker | value-only TreeCalc and direct OxFml carrier evidence exists; metadata projection fixture or carrier sufficiency proof remains absent |
| `w039_declared_gap_match_promotion_guard` | W038 match guard | retained no-proxy guard | no exact blocker or declared gap is counted as an optimized/core match |

## 5. Exact Remaining Blockers

| Blocker | Required successor evidence |
|---|---|
| dynamic release/reclassification differential | direct optimized/core differential proving dependency release and reclassification against the TraceCalc release row |
| snapshot-fence counterpart | deterministic coordinator or partition replay for stale accepted-candidate admission and rejection |
| capability-view fence counterpart | deterministic coordinator or partition replay for compatibility-fenced capability-view mismatch |
| callable metadata projection | concrete metadata projection fixture or carrier sufficiency proof for consumed callable rows |

These blockers route to `calc-f7o.3`, `calc-f7o.4`, `calc-f7o.7`, `calc-f7o.8`, and `calc-f7o.9` as promotion constraints.

## 6. Runner Changes

`src/oxcalc-tracecalc/src/implementation_conformance.rs` now has a W039 implementation-conformance profile.

The W039 profile:

1. reads the W039 successor obligation ledger,
2. reads the W038 exact remaining blocker register,
3. reads the W038 match-promotion guard,
4. reads the W039 W073 formatting-intake guard,
5. emits W039 disposition, blocker, match-guard, evidence-summary, validation, and run-summary artifacts,
6. validates that W039 retains four exact blockers and promotes zero matches.

`src/oxcalc-tracecalc-cli/src/main.rs` now reports W039 implementation-conformance summaries separately from W038 summaries.

## 7. OxFml Formatting Intake

This bead does not construct a new OxFml conditional-formatting request payload.

It still carries the W039 W073 guard as a source input:

1. `typed_rule` remains the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are not a fallback for those families.
3. No OxFml handoff is filed by this bead because the W039 runner consumes the guard but exposes no concrete exercised OxCalc mismatch.

## 8. Semantic-Equivalence Statement

This bead adds a W039 implementation-conformance runner profile, CLI reporting branch, emitted artifacts, and status/spec text.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TreeCalc runtime behavior, TraceCalc reference semantics, OxFml evaluator behavior, OxFunc kernels, Stage 2 scheduler policy, pack/C5 capability policy, operated-service behavior, alert dispatch behavior, or retained-history behavior.

Observable runtime behavior is invariant under this bead. The W039 runner classifies evidence and exact blockers only.

## 9. Verification

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `cargo test -p oxcalc-tracecalc implementation_conformance -- --nocapture` | passed; 5 tests |
| `cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `cargo run -p oxcalc-tracecalc-cli -- implementation-conformance w039-optimized-core-exact-blocker-disposition-001` | passed; emitted W039 disposition artifacts |
| JSON parse for `docs/test-runs/core-engine/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/*.json` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-f7o.3` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

## 10. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W039 workset/status surfaces, feature map, runner, CLI branch, and machine-readable artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-f7o.8` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for predecessor evidence cited by the W039 runner; this bead emits a deterministic implementation-conformance disposition packet |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 8 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; W073 is retained as a guard and no handoff trigger exists |
| 6 | All required tests pass? | yes; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for this exact-blocker disposition target; broader gaps remain exact blockers with owner lanes |
| 8 | Completion language audit passed? | yes; no full optimized/core verification, release-grade, C5, pack-grade replay, Stage 2 policy, operated service, independent-diversity, broad OxFml, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W039 optimized/core disposition state |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-f7o.2` closure and `calc-f7o.3` readiness |

## 11. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-f7o.2` asks for W039 optimized/core exact blocker implementation closure |
| Gate criteria re-read | pass; callable metadata projection, dynamic, snapshot, capability-fence counterparts, and match-promotion guards have deterministic evidence or exact blockers |
| Silent scope reduction check | pass; all four exact blockers remain visible and no row is promoted as a match |
| "Looks done but is not" pattern check | pass; runner support and retained evidence are not represented as full optimized/core verification |
| Result | pass for the `calc-f7o.2` target |

## 12. Three-Axis Report

- execution_state: `calc-f7o.2_optimized_core_exact_blocker_disposition_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-f7o.3` Lean/TLA totality and proof-model closure tranche is next
  - dynamic release/reclassification differential remains an exact optimized/core blocker
  - snapshot-fence and capability-view counterparts remain exact Stage 2/coordinator replay blockers
  - callable metadata projection remains an exact proof/seam blocker
  - operated assurance service, retained history, alert/quarantine dispatcher, and cross-engine differential service remain open
  - independent evaluator row set and diversity evidence remain open
  - OxFml seam breadth, W073 typed-only conditional-formatting metadata, public consumer surfaces, and callable metadata closure remain open
  - pack-grade replay, C5, and release-grade decision remain open
