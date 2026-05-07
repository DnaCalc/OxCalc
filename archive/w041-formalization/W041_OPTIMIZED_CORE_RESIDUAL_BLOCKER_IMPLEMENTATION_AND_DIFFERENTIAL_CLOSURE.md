# W041 Optimized/Core Residual Blocker Implementation And Differential Closure

Status: `spec_drafted_with_replay_evidence`

Bead: `calc-sui.2`

Run id: `w041-optimized-core-residual-blocker-differentials-001`

Supporting TreeCalc run id: `w041-optimized-core-automatic-dynamic-transition-001`

## Purpose

This packet attacks the W040 optimized/core exact blockers assigned to `calc-sui.2`.

The narrow new engine evidence is automatic dynamic dependency-set transition handling for a resolved dynamic reference that becomes dynamic-potential on rerun. W040 required manual `DependencyRemoved` and `DependencyReclassified` invalidation seeds for that scenario. W041.2 adds a successor formula-catalog fixture path and derives those invalidation reasons from predecessor/successor dependency descriptors without manual seed injection.

The snapshot-fence counterpart, capability-view fence counterpart, and callable metadata projection lanes are retained as exact blockers for later W041 beads. They are not match-promoted by this packet.

## Changed Surfaces

| Path | Change |
| --- | --- |
| `src/oxcalc-core/src/treecalc.rs` | adds formula-catalog-aware structural invalidation seed derivation and descriptor-transition classification |
| `src/oxcalc-core/src/treecalc_fixture.rs` | allows a post-edit fixture to supply a successor formula catalog |
| `src/oxcalc-core/src/treecalc_runner.rs` | adds runner assertions for the automatic dynamic transition artifact |
| `docs/test-fixtures/core-engine/treecalc/cases/tc_local_dynamic_release_reclassification_auto_post_edit_001.json` | new W041 fixture with no manual invalidation seeds |
| `docs/test-fixtures/core-engine/treecalc/MANIFEST.json` | registers the W041 fixture |

## Evidence Packet

| Artifact | Purpose |
| --- | --- |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w041-optimized-core-residual-blocker-differentials-001/run_summary.json` | records W041.2 counts, remaining blockers, no-promotion claims, and next bead |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w041-optimized-core-residual-blocker-differentials-001/dynamic_release_reclassification_auto_transition_evidence.json` | binds the automatic dynamic transition fixture to predecessor/successor descriptor evidence |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w041-optimized-core-residual-blocker-differentials-001/w041_exact_blocker_disposition_register.json` | records disposition rows for W040 optimized/core exact blockers |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w041-optimized-core-residual-blocker-differentials-001/w041_exact_remaining_blocker_register.json` | retains three exact blockers for later W041 lanes |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w041-optimized-core-residual-blocker-differentials-001/w041_match_promotion_guard.json` | records zero match promotion |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w041-optimized-core-residual-blocker-differentials-001/source_evidence_index.json` | indexes predecessor, code, replay, and OxFml formatting intake sources |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w041-optimized-core-residual-blocker-differentials-001/validation.json` | records validation commands and artifact checks |

The direct replay evidence is:

1. `archive/test-runs-core-engine-w038-w045/treecalc-local/w041-optimized-core-automatic-dynamic-transition-001/run_summary.json`
2. `archive/test-runs-core-engine-w038-w045/treecalc-local/w041-optimized-core-automatic-dynamic-transition-001/cases/tc_local_dynamic_release_reclassification_auto_post_edit_001/post_edit/invalidation_seeds.json`
3. `archive/test-runs-core-engine-w038-w045/treecalc-local/w041-optimized-core-automatic-dynamic-transition-001/cases/tc_local_dynamic_release_reclassification_auto_post_edit_001/post_edit/invalidation_closure.json`
4. `archive/test-runs-core-engine-w038-w045/treecalc-local/w041-optimized-core-automatic-dynamic-transition-001/cases/tc_local_dynamic_release_reclassification_auto_post_edit_001/post_edit/result.json`

Observed replay facts:

1. TreeCalc emitted 26 cases with 0 expectation mismatches.
2. The W041 automatic transition case supplies no manual invalidation seed overrides.
3. The predecessor formula catalog contains `DynamicResolved(target_node_id=2)`.
4. The successor formula catalog contains `DynamicPotential` for the same formula artifact.
5. The derived post-edit seeds are `DependencyRemoved` and `DependencyReclassified`.
6. The invalidation closure marks node 3 as `requires_rebind: true`.
7. The post-edit rerun rejects with `HostInjectedFailure` before new publication.

## Remaining Exact Blockers

| Blocker | Owner |
| --- | --- |
| `w041_snapshot_fence_counterpart_exact_blocker` | `calc-sui.5` |
| `w041_capability_view_fence_counterpart_exact_blocker` | `calc-sui.5` |
| `w041_callable_metadata_projection_exact_blocker` | `calc-sui.8` |

## OxFml Formatting Intake

The current W073 formatting update does not require a W041.2 optimized/core code change.

The typed-only rule remains active for W041 later lanes: `VerificationConditionalFormattingRule.typed_rule` is the accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`; W072 bounded `thresholds` strings are not fallback metadata for those aggregate and visualization families.

## Semantic Equivalence Statement

For existing fixtures that do not supply a successor formula catalog, post-edit seed derivation is behavior-preserving: the old structural-context derivation path is still used.

For fixtures that do supply a successor formula catalog, observable behavior changes only when predecessor/successor dependency descriptors differ. In that case the rerun records explicit dependency-transition reasons and uses the existing rebind gate. This does not change published values for accepted runs; it rejects transition cases that require rebind before reevaluation.

## Validation

| Command | Result |
| --- | --- |
| `cargo test -p oxcalc-core structural_invalidation_seeds_mark_formula_catalog_dynamic_release_reclassification` | passed |
| `cargo test -p oxcalc-core treecalc_fixture` | passed |
| `cargo test -p oxcalc-core treecalc_runner` | passed |
| `cargo run -p oxcalc-tracecalc-cli -- treecalc w041-optimized-core-automatic-dynamic-transition-001` | passed; emitted 26 cases with 0 expectation mismatches |
| `cargo test -p oxcalc-core` | passed |
| `scripts/check-worksets.ps1` | passed |
| `br dep cycles --json` | passed; 0 cycles |
| `git diff --check` | passed; CRLF warnings only |

## Status Report

- execution_state: `calc-sui.2_optimized_core_dynamic_transition_evidence_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-sui.3` Rust totality/refinement and panic-boundary discharge
  - `calc-sui.5` snapshot-fence and capability-view fence counterparts
  - `calc-sui.8` callable metadata projection and callable carrier sufficiency
  - full optimized/core verification remains unpromoted
  - release-grade verification, C5, pack-grade replay, Stage 2 policy, operated services, independent evaluator breadth, broad OxFml display/publication, public migration, callable carrier sufficiency, and general OxFunc kernels remain unpromoted

## Pre-Closure Verification Checklist

| Check | Result |
| --- | --- |
| Workset and bead ids are explicit | yes: `W041`, `calc-sui.2` |
| Required artifacts exist | yes: W041.2 packet artifacts and TreeCalc replay artifacts are present |
| Direct replay evidence exists for changed behavior | yes: automatic dynamic transition fixture and replay run |
| No declared gap is match-promoted | yes: `w041_match_promotion_guard.json` records zero match promotion |
| Residual blockers are explicit | yes: three exact remaining blocker rows |
| Semantic equivalence statement is present | yes |

## Completion Claim Self-Audit

| Audit Item | Result |
| --- | --- |
| Claim is limited to `calc-sui.2` target | yes |
| Scaffolding is not reported as implementation | yes |
| Spec text has replay evidence | yes |
| Cross-repo handoff is not treated as closure | yes; OxFml W073 remains a later seam-watch lane |
| Uncertain lanes default to in-progress | yes; remaining exact blockers are retained |
| Strategy-change equivalence statement is present | yes |
