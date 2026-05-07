# W045 Optimized Core Counterpart Coverage And Callable Metadata Projection Closure

Status: `calc-zkio.2_optimized_core_counterpart_callable_metadata_tranche_validated`
Workset: `W045`
Parent epic: `calc-zkio`
Bead: `calc-zkio.2`

## 1. Purpose

This packet attacks the W045 optimized/core tranche with runner-backed implementation-conformance evidence.

The W045.2 runner consumes the W045 residual obligation map plus the W044 optimized/core artifacts. It carries the W044 mixed dynamic soft-reference evidence as direct evidence, classifies broader dynamic and soft-reference/`INDIRECT` breadth as retained exact blockers, carries snapshot/capability counterpart breadth as declared-profile-only blockers, and keeps callable metadata projection separate from `LET`/`LAMBDA` value-carrier evidence.

This narrows and classifies the optimized/core surface. It does not promote full optimized/core verification, broad dynamic-transition coverage, snapshot/capability counterpart breadth, callable metadata projection, callable carrier sufficiency, Stage 2 production policy, pack-grade replay, C5, release-grade verification, registered-external/provider callable publication, or general OxFunc kernels.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W045_CORE_FORMALIZATION_RELEASE_GRADE_SERVICE_AND_CROSS_REPO_UPTAKE_VERIFICATION.md` | W045 gate and `calc-zkio.2` target |
| `docs/spec/core-engine/w045-formalization/W045_RESIDUAL_RELEASE_GRADE_SUCCESSOR_OBLIGATION_AND_CURRENT_OXFML_INTAKE_MAP.md` | W045 residual lane map and current OxFml intake |
| `archive/test-runs-core-engine-w038-w045/release-grade-ledger/w045-residual-release-grade-successor-obligation-current-oxfml-intake-map-001/` | W045.1 machine-readable successor obligations, promotion contracts, and W073/public-surface intake |
| `docs/spec/core-engine/w044-formalization/W044_OPTIMIZED_CORE_DYNAMIC_TRANSITION_AND_CALLABLE_METADATA_IMPLEMENTATION.md` | predecessor optimized/core tranche |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w044-optimized-core-dynamic-transition-callable-metadata-001/` | W044 disposition, exact blocker, dynamic evidence, callable metadata, and match guard inputs |
| `archive/test-runs-core-engine-w038-w045/treecalc-local/w044-optimized-core-dynamic-transition-treecalc-001/run_summary.json` | W044 TreeCalc run, 28 cases and 0 expectation mismatches |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | current public consumer surface and note-level watch lanes |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | current W073 typed-only formatting guard |

## 3. Artifact Surface

Run id: `w045-optimized-core-counterpart-callable-metadata-001`

| Artifact | Result |
|---|---|
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w045-optimized-core-counterpart-callable-metadata-001/run_summary.json` | 7 W045 dispositions, 2 direct-evidence rows, 5 exact remaining blockers, 0 match-promoted rows, 0 failed rows |
| `w045_dynamic_transition_coverage_register.json` | carries W044 mixed add/remove/reclassify evidence and names the remaining dynamic/soft-reference/`INDIRECT` expansion targets |
| `w045_counterpart_coverage_register.json` | records snapshot-fence and capability-view counterpart breadth as retained blockers |
| `w045_callable_metadata_projection_register.json` | records `LET`/`LAMBDA` value/carrier evidence as non-metadata evidence and keeps registered-external/provider publication as a seam obligation |
| `w045_optimized_core_disposition_register.json` | records the 7 W045.2 disposition rows |
| `w045_exact_remaining_blocker_register.json` | records 5 retained exact blockers |
| `w045_match_promotion_guard.json` | preserves zero match/proxy promotion |
| `validation.json` | validates the W045.2 implementation-conformance packet |

## 4. Implementation Delta

Changed source files:

1. `src/oxcalc-tracecalc/src/implementation_conformance.rs`
2. `src/oxcalc-tracecalc-cli/src/main.rs`

The implementation-conformance runner now has a W045 branch that:

1. reads the W045.1 residual obligation map,
2. reads W044 optimized/core disposition, dynamic evidence, exact blockers, callable metadata, and match guard artifacts,
3. emits W045.2 disposition, dynamic, counterpart, callable metadata, exact blocker, match guard, evidence summary, validation, and run-summary artifacts,
4. exposes W045 disposition counts through the CLI summary path,
5. includes a focused unit test for the W045.2 runner branch.

No new TreeCalc fixture or runtime behavior is added in this bead. The W045.2 runner intentionally carries the W044 mixed dynamic fixture as direct evidence and keeps the broader W045 lanes blocked where current evidence is still insufficient.

## 5. Disposition

| Area | W045.2 result |
|---|---|
| Mixed dynamic soft-reference transition | W044 direct evidence carried |
| Broader dynamic transition coverage | exact blocker retained after carried direct evidence |
| Soft-reference/`INDIRECT` and late reference-resolution breadth | exact blocker retained; needs new direct fixtures or a sufficiency proof |
| Snapshot-fence counterpart breadth | exact blocker retained; declared-profile evidence is not broad production evidence |
| Capability-view counterpart breadth | exact blocker retained; declared-profile evidence is not broad capability-fence production evidence |
| Callable metadata projection | exact blocker retained; value-carrier evidence is not metadata projection evidence |
| Registered-external/provider callable publication | watch/seam obligation retained for `calc-zkio.8` |
| W073 formatting | current typed-rule-only guard carried; downstream request construction remains unverified |
| Match/proxy promotion | zero promoted matches |

## 6. Remaining Exact Blockers

1. `w045_broader_dynamic_transition_remaining_exact_blocker`
   - W044 mixed dynamic add/remove/reclassify evidence is carried, but broad dynamic coverage still needs additional descriptor transitions, structural-plus-formula edits, host-resolution surfaces, or a sufficiency proof.
2. `w045_soft_reference_indirect_resolution_exact_blocker`
   - W045 still lacks direct broader fixture evidence for `INDIRECT` selector churn, late reference resolution, and soft-reference update breadth beyond the W044 mixed case.
3. `w045_snapshot_fence_counterpart_breadth_exact_blocker`
   - Snapshot-fence counterpart evidence remains declared-profile evidence, not broad production scheduler evidence.
4. `w045_capability_view_counterpart_breadth_exact_blocker`
   - Capability-view counterpart evidence remains declared-profile evidence, not broad capability-fence production evidence.
5. `w045_callable_metadata_projection_exact_blocker`
   - Callable metadata projection remains absent; `LET`/`LAMBDA` carrier/value evidence is not metadata projection evidence.

## 7. OxFml Intake

Current W045.2 intake:

1. W073 remains a direct typed-rule replacement path for aggregate and visualization conditional-formatting metadata.
2. `VerificationConditionalFormattingRule.typed_rule` is the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
3. W072 bounded `thresholds` strings are intentionally ignored for those families.
4. Ordinary downstream OxFml use should target `oxfml_core::consumer::runtime`, `oxfml_core::consumer::editor`, and `oxfml_core::consumer::replay`.
5. Public `substrate::...` access is not an ordinary downstream integration contract.
6. Downstream W073 typed-rule request construction remains required but unverified in OxCalc.
7. No OxFml handoff is filed by this bead because no exercised OxCalc/OxFml mismatch is exposed.

## 8. Semantic-Equivalence Statement

This bead adds a W045 implementation-conformance runner branch, CLI reporting for W045 disposition counts, one focused unit test, and generated implementation-conformance artifacts.

It does not change coordinator scheduling policy, dependency graph construction, invalidation semantics, soft-reference update semantics, pure recalc semantics, publication semantics, reject policy, overlay lifecycle behavior, Stage 2 partition policy, OxFml evaluator behavior, or OxFunc behavior.

Observable runtime behavior is invariant under this bead. The runner reads existing evidence and emits classification artifacts only.

## 9. Verification

| Command | Result |
|---|---|
| `cargo test -p oxcalc-tracecalc implementation_conformance_runner_classifies_w045_optimized_core_tranche -- --nocapture` | passed |
| `cargo test -p oxcalc-tracecalc implementation_conformance -- --nocapture` | passed; 7 tests |
| `cargo run -p oxcalc-tracecalc-cli -- implementation-conformance w045-optimized-core-counterpart-callable-metadata-001` | passed; 7 W045 dispositions, 2 direct-evidence rows, 5 exact remaining blockers, 0 match-promoted rows, 0 failed rows |
| JSON parse for `archive/test-runs-core-engine-w038-w045/implementation-conformance/w045-optimized-core-counterpart-callable-metadata-001/*.json` | passed |
| `cargo fmt --all -- --check` | passed |
| `scripts/check-worksets.ps1` | passed pre-close; `worksets=23; beads total=187; open=10; in_progress=1; ready=0; blocked=9; deferred=0; closed=176` |
| `br ready --json` | passed pre-close; no ready beads while `calc-zkio.2` is in progress |
| `br dep cycles --json` | passed pre-close; `count=0` |
| `git diff --check` | passed pre-close; CRLF normalization warnings only |
| JSON parse for `archive/test-runs-core-engine-w038-w045/implementation-conformance/w045-optimized-core-counterpart-callable-metadata-001/*.json` | passed post-close |
| `scripts/check-worksets.ps1` | passed post-close; `worksets=23; beads total=187; open=10; in_progress=0; ready=1; blocked=8; deferred=0; closed=177` |
| `br ready --json` | passed post-close; next ready bead is `calc-zkio.3` |
| `br dep cycles --json` | passed post-close; `count=0` |
| `git diff --check` | passed post-close; CRLF normalization warnings only |

## 10. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W045 README/status surfaces, feature map, runner code, and machine-readable artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-zkio.10` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; W045.2 cites W044 TreeCalc direct evidence and emits generated implementation-conformance artifacts |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 8 states no policy or runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; current W073 typed-only formatting intake and public-surface notes are carried and no handoff trigger exists |
| 6 | All required tests pass? | yes; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for the `calc-zkio.2` target; broader gaps are retained as exact blockers |
| 8 | Completion language audit passed? | yes; no full optimized/core, release-grade, C5, pack, Stage 2, callable metadata, callable carrier sufficiency, broad OxFml, public migration, W073 downstream uptake, registered-external projection, provider-publication, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; no workset ordering changed |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-zkio.2` closed and `calc-zkio.3` ready |

## 11. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-zkio.2` asks for optimized/core counterpart coverage and callable metadata projection closure evidence or retained blockers |
| Gate criteria re-read | pass; no declared-gap match promotion and no full optimized/core verification claim appear |
| Silent scope reduction check | pass; dynamic, soft-reference/`INDIRECT`, snapshot, capability, callable metadata, W073, registered-external/provider watch, and match-guard lanes are classified |
| "Looks done but is not" pattern check | pass; carried W044 dynamic evidence narrows one lane but does not promote broad dynamic coverage |
| Result | pass for the `calc-zkio.2` target after final post-close validation |

## 12. Three-Axis Report

- execution_state: `calc-zkio.2_optimized_core_counterpart_callable_metadata_tranche_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - broader dynamic dependency-transition coverage remains partial beyond the mixed fixture family
  - soft-reference/`INDIRECT` and late reference-resolution breadth remains an exact blocker
  - snapshot-fence and capability-view counterpart breadth remain exact blockers for production promotion
  - callable metadata projection remains an exact blocker
  - callable carrier sufficiency proof remains blocked
  - Rust totality/refinement and panic-free core proof frontier remains open
  - Lean/TLA full-verification and unbounded fairness discharge remains open
  - Stage 2 production partition analyzer and scheduler equivalence remains open
  - operated services, retained-history, retained-witness lifecycle, retention SLO, alert/quarantine dispatch, independent evaluator breadth, mismatch quarantine, OxFml seam breadth, public migration, pack/C5, and release-grade decision remain open
