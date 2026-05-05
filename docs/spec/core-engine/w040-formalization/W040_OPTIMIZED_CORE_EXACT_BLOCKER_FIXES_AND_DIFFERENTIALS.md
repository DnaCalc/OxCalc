# W040 Optimized Core Exact Blocker Fixes And Differentials

Status: `calc-tv5.2_optimized_core_exact_blockers_narrowed_no_promotion`
Workset: `W040`
Parent epic: `calc-tv5`
Bead: `calc-tv5.2`

## 1. Purpose

This packet attacks the W040 optimized/core exact-blocker target.

The target is not to promote full optimized/core verification. The target is to bind direct executable evidence where available, retain exact blockers where direct evidence is still absent, and keep declared gaps out of match promotion.

The new W040 code and fixture work focuses on dynamic dependency release and reclassification: the local TreeCalc runner can now exercise explicit `DependencyRemoved` and `DependencyReclassified` invalidation seeds in a checked post-edit fixture, and emits post-edit invalidation-closure artifacts for inspection.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/direct_verification_obligation_map.json` | W040 obligations and promotion guards |
| `docs/test-runs/core-engine/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/w039_exact_remaining_blocker_register.json` | four source W039 optimized/core exact blockers |
| `docs/spec/core-engine/w039-formalization/W039_OPTIMIZED_CORE_EXACT_BLOCKER_IMPLEMENTATION_CLOSURE.md` | predecessor optimized/core disposition |
| `docs/test-fixtures/core-engine/treecalc/cases/tc_local_dynamic_release_reclassification_post_edit_001.json` | new W040 fixture for dependency release/reclassification seeds |
| `docs/test-runs/core-engine/treecalc-local/w040-optimized-core-dynamic-release-reclassification-001/` | W040 TreeCalc direct evidence run |

## 3. Artifact Surface

Run id: `w040-optimized-core-exact-blocker-fixes-differentials-001`

| Artifact | Result |
|---|---|
| `docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/run_summary.json` | records W040 disposition counts, direct evidence, blockers, match guard, and no-promotion claims |
| `docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/dynamic_release_reclassification_evidence.json` | binds the new dynamic release/reclassification TreeCalc evidence |
| `docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/w040_exact_blocker_disposition_register.json` | machine-readable W040 blocker dispositions |
| `docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/w040_exact_remaining_blocker_register.json` | exact blockers retained after this slice |
| `docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/w040_match_promotion_guard.json` | zero match promotions; declared-gap guard holds |
| `docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/source_evidence_index.json` | predecessor and new evidence paths |
| `docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/validation.json` | validation status for this packet |

The supporting TreeCalc run id is `w040-optimized-core-dynamic-release-reclassification-001`.

## 4. Direct Implementation Changes

`src/oxcalc-core/src/treecalc_fixture.rs` now supports explicit post-edit invalidation seed overrides in checked fixture cases. The new fixture field is `post_edit.invalidation_seeds`, with reasons such as `DependencyRemoved` and `DependencyReclassified`.

`src/oxcalc-core/src/treecalc_runner.rs` now emits `post_edit/invalidation_closure.json` so dependency-change closure evidence can be inspected and cited directly. Runner tests assert the W040 dynamic release/reclassification case emits:

1. `DependencyRemoved` and `DependencyReclassified` post-edit seeds,
2. a closure record for node `3` with `requires_rebind: true`,
3. a rejected rerun with `HostInjectedFailure`,
4. no new publication after dependency release/reclassification seed handling.

## 5. Disposition Summary

| W040 row | Source blocker | W040 disposition | Evidence consequence |
|---|---|---|---|
| `w040_dynamic_release_reclassification_narrowed_blocker` | W039 dynamic release/reclassification exact blocker | direct invalidation evidence bound; exact transition blocker retained | explicit dependency release/reclassification seeds force rebind/no-publication; automatic dependency-set transition differential remains absent |
| `w040_snapshot_fence_counterpart_exact_blocker` | W039 snapshot-fence counterpart exact blocker | retained exact Stage 2/coordinator blocker | stale accepted-candidate counterpart remains absent |
| `w040_capability_view_fence_counterpart_exact_blocker` | W039 capability-view fence counterpart exact blocker | retained exact Stage 2/coordinator blocker | compatibility-fenced capability-view mismatch counterpart remains absent |
| `w040_callable_metadata_projection_exact_blocker` | W039 callable metadata projection exact blocker | retained exact proof/seam blocker | value-only TreeCalc and direct OxFml carrier evidence remain insufficient for metadata projection promotion |
| `w040_declared_gap_match_promotion_guard` | W039 match guard | retained zero match promotion | no exact blocker or declared gap is counted as an optimized/core match |

## 6. Dynamic Evidence Boundary

The W040 fixture provides direct optimized-lane evidence for dependency-change invalidation consequences:

1. initial resolved dynamic dependency publishes value `11` and emits `runtime_effect.dynamic_reference`,
2. post-edit manual seeds are `DependencyRemoved` and `DependencyReclassified`,
3. the invalidation closure records `requires_rebind: true`,
4. the rerun rejects with `HostInjectedFailure`,
5. the runner preserves the previously published value and emits no new publication.

This narrows the W039 blocker but does not remove it. The remaining exact blocker is automatic dynamic dependency-set transition detection: a future slice must compare predecessor and successor optimized/core dependency descriptors against the TraceCalc release row without manual dependency-change seed injection.

## 7. Semantic-Equivalence Statement

This bead adds fixture input support for explicit post-edit invalidation seeds, a post-edit invalidation-closure artifact, one checked W040 fixture, W040 TreeCalc evidence artifacts, and W040 disposition/audit artifacts.

It does not change coordinator scheduling, publication semantics, reject taxonomy, dependency graph construction semantics, ordinary structural invalidation derivation, formula evaluation semantics, OxFml evaluator behavior, OxFunc kernels, Stage 2 scheduler policy, pack/C5 capability policy, operated-service behavior, alert dispatch behavior, or retained-history behavior.

Observable runtime behavior for existing fixtures is invariant. The new behavior is opt-in through checked fixture input and exposes already-modeled `DependencyRemoved` and `DependencyReclassified` invalidation reasons as deterministic evidence.

## 8. Verification

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `cargo test -p oxcalc-core treecalc -- --nocapture` | passed; 29 tests |
| `cargo run -p oxcalc-tracecalc-cli -- treecalc w040-optimized-core-dynamic-release-reclassification-001` | passed; emitted 25 cases with 0 expectation mismatches |
| JSON parse for W040 implementation-conformance and TreeCalc run artifacts | passed |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-tv5.3` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

## 9. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W040 status surfaces, fixture, runner artifacts, and machine-readable disposition artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-tv5.9` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; the W040 TreeCalc run emits deterministic dynamic release/reclassification seed evidence |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 7 states no runtime strategy changed for existing behavior |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; this bead changes local fixture/runner evidence only and no OxFml handoff is triggered |
| 6 | All required tests pass? | yes; see Section 8 |
| 7 | No known semantic gaps remain in declared scope? | yes for the `calc-tv5.2` target; broader optimized/core gaps remain exact blockers with successor owners |
| 8 | Completion language audit passed? | yes; no full optimized/core verification, release-grade, C5, pack-grade replay, Stage 2 policy, operated service, independent-diversity, broad OxFml, callable metadata projection, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W040 optimized/core disposition |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-tv5.2` closure and `calc-tv5.3` readiness |

## 10. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-tv5.2` asks for optimized/core exact blocker fixes and differentials |
| Gate criteria re-read | pass; dynamic, snapshot, capability-fence, callable metadata, and match-promotion rows are classified with direct evidence or exact blockers |
| Silent scope reduction check | pass; the dynamic row is narrowed but not promoted; all four exact blockers remain visible |
| "Looks done but is not" pattern check | pass; fixture support and direct seed evidence are not represented as automatic dependency-set transition closure |
| Result | pass for the `calc-tv5.2` target after final validation |

## 11. Three-Axis Report

- execution_state: `calc-tv5.2_optimized_core_exact_blockers_narrowed_no_promotion`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - automatic dynamic dependency-set release/reclassification transition differential remains an exact blocker
  - snapshot-fence counterpart remains a Stage 2/coordinator replay blocker
  - capability-view fence counterpart remains a Stage 2/coordinator replay blocker
  - callable metadata projection remains an OxFml/callable seam blocker
  - `calc-tv5.3` Rust totality and refinement proof tranche is next
  - Lean/TLA, Stage 2 policy, operated services, independent evaluator diversity, OxFml seam breadth, pack/C5, and release-grade decision remain open
