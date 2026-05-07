# W044 Optimized Core Dynamic Transition And Callable Metadata Implementation

Status: `calc-b1t.2_optimized_core_dynamic_transition_callable_metadata_tranche_validated`
Workset: `W044`
Parent epic: `calc-b1t`
Bead: `calc-b1t.2`

## 1. Purpose

This packet attacks the W044 optimized/core tranche with direct implementation-conformance evidence for a mixed dynamic soft-reference transition and with explicit retained blockers for the lanes that remain under-evidenced.

The new evidence is a TreeCalc fixture where one formula owner simultaneously releases one dynamic dependency and adds another. The successor formula catalog derives `DependencyAdded`, `DependencyRemoved`, and `DependencyReclassified` invalidation seeds automatically, marks the owner rebind-required, and rejects the post-edit run without publication.

This narrows the broader dynamic-transition blocker. It does not promote broad dynamic coverage, full optimized/core verification, callable metadata projection, Stage 2 production policy, pack-grade replay, C5, release-grade verification, or general OxFunc kernels.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W044_CORE_FORMALIZATION_RELEASE_GRADE_BLOCKER_BURN_DOWN_AND_SERVICE_PROOF_CLOSURE.md` | W044 gate and `calc-b1t.2` target |
| `docs/spec/core-engine/w044-formalization/W044_RESIDUAL_RELEASE_GRADE_BLOCKER_RECLASSIFICATION_AND_PROMOTION_CONTRACT_MAP.md` | W044 residual lane map and no-proxy guard |
| `archive/test-runs-core-engine-w038-w045/release-grade-ledger/w044-residual-release-grade-blocker-reclassification-map-001/` | W044.1 machine-readable residual blocker and promotion-contract inputs |
| `docs/spec/core-engine/w043-formalization/W043_OPTIMIZED_CORE_BROAD_CONFORMANCE_AND_CALLABLE_METADATA_CLOSURE.md` | predecessor optimized/core and callable metadata packet |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w043-optimized-core-broad-conformance-callable-metadata-closure-001/` | W043 counterpart, exact blocker, callable metadata, match guard, and formatting-intake inputs |
| `docs/test-fixtures/core-engine/treecalc/MANIFEST.json` | checked-in TreeCalc fixture corpus |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | current W073 typed-only formatting guard |
| `../OxFml/docs/handoffs/HANDOFF-DNAONECALC-012_W073_TYPED_CF_PAYLOAD_FIRST_SLICE.md` | downstream typed-rule request-construction handoff |

## 3. Artifact Surface

Run ids:

1. `w044-optimized-core-dynamic-transition-treecalc-001`
2. `w044-optimized-core-dynamic-transition-callable-metadata-001`

| Artifact | Result |
|---|---|
| `archive/test-runs-core-engine-w038-w045/treecalc-local/w044-optimized-core-dynamic-transition-treecalc-001/run_summary.json` | 28 cases, 0 expectation mismatches |
| `archive/test-runs-core-engine-w038-w045/treecalc-local/w044-optimized-core-dynamic-transition-treecalc-001/cases/tc_local_dynamic_mixed_add_release_auto_post_edit_001/post_edit/invalidation_seeds.json` | emits `DependencyAdded`, `DependencyRemoved`, and `DependencyReclassified` |
| `archive/test-runs-core-engine-w038-w045/treecalc-local/w044-optimized-core-dynamic-transition-treecalc-001/cases/tc_local_dynamic_mixed_add_release_auto_post_edit_001/post_edit/invalidation_closure.json` | records owner `3` as rebind-required with all three reasons |
| `archive/test-runs-core-engine-w038-w045/treecalc-local/w044-optimized-core-dynamic-transition-treecalc-001/cases/tc_local_dynamic_mixed_add_release_auto_post_edit_001/post_edit/result.json` | rejects as `HostInjectedFailure` without publication |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w044-optimized-core-dynamic-transition-callable-metadata-001/run_summary.json` | records 6 W044 dispositions, 2 direct-evidence rows, 4 exact remaining blockers, 0 match-promoted rows, and 0 failed rows |
| `w044_dynamic_transition_evidence.json` | binds the mixed dynamic transition evidence |
| `w044_optimized_core_disposition_register.json` | records W044.2 disposition rows |
| `w044_exact_remaining_blocker_register.json` | records 4 retained exact blockers |
| `w044_callable_metadata_projection_register.json` | records callable value/carrier evidence as non-metadata projection evidence |
| `w044_match_promotion_guard.json` | preserves zero match/proxy promotion |
| `validation.json` | validates the W044.2 implementation-conformance packet |

## 4. Implementation Delta

Changed files:

1. `docs/test-fixtures/core-engine/treecalc/MANIFEST.json`
2. `docs/test-fixtures/core-engine/treecalc/cases/tc_local_dynamic_mixed_add_release_auto_post_edit_001.json`
3. `src/oxcalc-core/src/treecalc.rs`
4. `src/oxcalc-core/src/treecalc_fixture.rs`
5. `src/oxcalc-core/src/treecalc_runner.rs`
6. `src/oxcalc-tracecalc/src/implementation_conformance.rs`
7. `src/oxcalc-tracecalc-cli/src/main.rs`

The new fixture proves:

1. predecessor formula descriptors include one resolved dynamic dependency and one potential dynamic dependency,
2. successor formula descriptors release the first carrier and resolve the second carrier,
3. automatic post-edit seed reasons are `DependencyAdded`, `DependencyRemoved`, and `DependencyReclassified`,
4. invalidation closure requires rebind for owner `3`,
5. post-edit execution rejects as `HostInjectedFailure`,
6. no publication is committed for the rebind-required transition.

## 5. Disposition

| Area | W044.2 result |
|---|---|
| Mixed dynamic soft-reference transition | direct evidence bound |
| Broader dynamic transition coverage | exact blocker retained after narrowing evidence |
| Snapshot-fence counterpart breadth | exact blocker retained; W043 declared-profile evidence is not broad production evidence |
| Capability-view counterpart breadth | exact blocker retained; W043 declared-profile evidence is not broad production evidence |
| Callable metadata projection | exact blocker retained; value-carrier evidence is not metadata projection evidence |
| W073 formatting | current typed-rule-only guard carried; downstream request construction remains unverified |
| Match/proxy promotion | zero promoted matches |

## 6. Remaining Exact Blockers

1. `w044_broader_dynamic_transition_remaining_exact_blocker`
   - The new evidence covers a mixed formula-catalog add/release/reclassification transition. It does not prove all dynamic reference families, production host-resolution behavior, structural-plus-formula edit mixtures, or a sufficiency theorem for broad dynamic transition coverage.
2. `w044_snapshot_fence_counterpart_breadth_exact_blocker`
   - W043 snapshot counterpart evidence remains declared-profile evidence, not broad Stage 2 production policy evidence.
3. `w044_capability_view_counterpart_breadth_exact_blocker`
   - W043 capability-view counterpart evidence remains declared-profile evidence, not broad capability-fence production evidence.
4. `w044_callable_metadata_projection_exact_blocker`
   - LET/LAMBDA carrier and value publication evidence remains separate from callable identity metadata projection.

## 7. OxFml Formatting Intake

Current W044.2 intake:

1. W073 remains a direct typed-rule replacement path for aggregate and visualization conditional-formatting metadata.
2. `VerificationConditionalFormattingRule.typed_rule` is the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
3. W072 bounded `thresholds` strings are intentionally ignored for those families.
4. `thresholds` remains meaningful for scalar/operator/expression rule families only.
5. The current OxFml working-copy evidence reports 21 focused conditional-formatting tests, including old bounded-string non-interpretation rows.
6. Downstream W073 typed-rule request construction remains required but unverified in OxCalc.
7. No OxFml handoff is filed by this bead because no exercised OxCalc/OxFml mismatch is exposed.

## 8. Semantic-Equivalence Statement

This bead adds one checked-in fixture, one unit assertion for mixed dynamic add/release/reclassification seed derivation, TreeCalc runner assertions for emitted artifacts, and a W044 implementation-conformance runner/CLI branch.

For existing fixtures and existing runtime profiles, observable behavior is invariant: the fresh TreeCalc run has 0 expectation mismatches across the expanded 28-case corpus. The new fixture adds a previously unexercised transition and records the current runtime consequence for that transition: rebind-required rejection with no publication.

The implementation-conformance runner emits evidence and disposition artifacts only. It does not change coordinator scheduling policy, dependency graph construction, invalidation semantics, soft-reference update semantics, pure recalc semantics, publication semantics, reject policy, overlay lifecycle behavior, Stage 2 partition policy, OxFml evaluator behavior, or OxFunc behavior.

## 9. Verification

| Command | Result |
|---|---|
| `cargo test -p oxcalc-core treecalc -- --nocapture` | passed; 32 tests |
| `cargo run -p oxcalc-tracecalc-cli -- treecalc w044-optimized-core-dynamic-transition-treecalc-001` | passed; 28 cases, 0 expectation mismatches |
| `cargo run -p oxcalc-tracecalc-cli -- implementation-conformance w044-optimized-core-dynamic-transition-callable-metadata-001` | passed; 6 W044 dispositions, 2 direct-evidence rows, 4 exact remaining blockers, 0 match-promoted rows, 0 failed rows |
| `cargo test -p oxcalc-tracecalc implementation_conformance -- --nocapture` | passed; 6 tests |
| JSON parse for W044.2 TreeCalc and implementation-conformance artifacts | passed; generated JSON artifacts parsed |
| `cargo fmt --all -- --check` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure: `worksets=22; beads total=175; open=10; in_progress=0; ready=1; blocked=8; deferred=0; closed=165` |
| `br ready --json` | passed after bead closure; next ready bead is `calc-b1t.3` |
| `br dep cycles --json` | passed with `count=0` |
| `git diff --check` | passed after bead closure; CRLF normalization warnings only |

## 10. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W044 README/status surfaces, feature map, fixture corpus, runner code, and machine-readable artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-b1t.10` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; W044 TreeCalc run `w044-optimized-core-dynamic-transition-treecalc-001` records the mixed dynamic transition fixture |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 8 states no policy or runtime strategy change |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; current W073 typed-only formatting intake is carried and no handoff trigger exists |
| 6 | All required tests pass? | yes; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for the `calc-b1t.2` target; broader gaps are retained as exact blockers |
| 8 | Completion language audit passed? | yes; no full optimized/core, release-grade, C5, pack, Stage 2, callable metadata, callable carrier sufficiency, broad OxFml, public migration, W073 downstream uptake, registered-external projection, provider-publication, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; no workset ordering changed |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-b1t.2` closure and `calc-b1t.3` readiness |

## 11. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-b1t.2` asks for optimized/core dynamic transition and callable metadata implementation evidence or exact blocker retention |
| Gate criteria re-read | pass; no declared-gap match promotion and no full optimized/core verification claim appear |
| Silent scope reduction check | pass; mixed dynamic, broader dynamic, snapshot, capability, callable metadata, W073, and match-guard lanes are all classified |
| "Looks done but is not" pattern check | pass; the new dynamic evidence narrows one blocker but does not promote broad dynamic coverage |
| Result | pass for the `calc-b1t.2` target |

## 12. Three-Axis Report

- execution_state: `calc-b1t.2_optimized_core_dynamic_transition_callable_metadata_tranche_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - broader dynamic dependency-transition coverage remains partial beyond the mixed fixture family
  - snapshot-fence and capability-view counterpart breadth remain exact blockers for production promotion
  - callable metadata projection remains an exact blocker
  - callable carrier sufficiency proof remains blocked
  - Rust totality/refinement and panic-free core proof frontier remains open
  - Lean/TLA full-verification and unbounded fairness discharge remains open
  - Stage 2 production partition analyzer and scheduler equivalence remains open
  - operated services, retained-history, retained-witness lifecycle, retention SLO, alert/quarantine dispatch, independent evaluator breadth, mismatch quarantine, OxFml seam breadth, public migration, pack/C5, and release-grade decision remain open
