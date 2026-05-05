# W043 Optimized Core Broad Conformance And Callable Metadata Closure

Status: `calc-2p3.2_optimized_core_broad_conformance_and_callable_metadata_closure_validated`
Workset: `W043`
Parent epic: `calc-2p3`
Bead: `calc-2p3.2`

## 1. Purpose

This packet attacks the W043 optimized/core conformance lane by adding direct dynamic-transition evidence and by retaining exact blockers where direct evidence is still absent.

The new implementation evidence is a TreeCalc fixture and run for automatic dynamic dependency addition. A dynamic-potential formula is rerun with a successor formula catalog that resolves the dynamic dependency. The runtime derives `DependencyAdded` and `DependencyReclassified` invalidation seeds without manual seed overrides, marks the node rebind-required, and rejects with no publication.

This narrows the W042 dynamic-transition blocker, but it does not promote broad dynamic transition coverage or full optimized/core verification. Callable metadata projection also remains an exact blocker: the current evidence still proves ordinary LET/LAMBDA value-carrier behavior, not callable identity metadata projection.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W043_CORE_FORMALIZATION_RELEASE_GRADE_PROOF_AND_OPERATED_SERVICE_INTEGRATION.md` | W043 gate and `calc-2p3.2` target |
| `docs/spec/core-engine/w043-formalization/W043_RESIDUAL_RELEASE_GRADE_PROOF_SERVICE_OBLIGATION_MAP.md` | W043 obligations and no-proxy guard |
| `docs/test-runs/core-engine/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/` | predecessor optimized/core counterpart and blocker packet |
| `docs/test-fixtures/core-engine/treecalc/MANIFEST.json` | checked-in TreeCalc fixture corpus |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | current W073 typed-only formatting guard |
| `../OxFml/docs/handoffs/HANDOFF-DNAONECALC-012_W073_TYPED_CF_PAYLOAD_FIRST_SLICE.md` | downstream typed request-construction handoff |

## 3. Artifact Surface

Run id: `w043-optimized-core-broad-conformance-callable-metadata-closure-001`

| Artifact | Result |
|---|---|
| `docs/test-runs/core-engine/treecalc-local/w043-optimized-core-broad-conformance-treecalc-001/run_summary.json` | 27 cases, 0 expectation mismatches |
| `docs/test-runs/core-engine/implementation-conformance/w043-optimized-core-broad-conformance-callable-metadata-closure-001/run_summary.json` | W043.2 summary and no-promotion claims |
| `w043_dynamic_transition_evidence.json` | new automatic dependency-addition/reclassification row and carried release/reclassification row |
| `w043_counterpart_conformance_register.json` | conformance disposition rows |
| `w043_callable_metadata_projection_register.json` | callable value versus metadata rows |
| `w043_exact_remaining_blocker_register.json` | exact blockers retained |
| `w043_match_promotion_guard.json` | no declared-gap, proxy, or match promotion guard |
| `w073_formatting_intake.json` | W073 typed-only formatting intake, no OxCalc code change |
| `validation.json` | validation results for this packet |

## 4. Implementation Delta

Changed files:

1. `docs/test-fixtures/core-engine/treecalc/MANIFEST.json`
2. `docs/test-fixtures/core-engine/treecalc/cases/tc_local_dynamic_addition_auto_post_edit_001.json`
3. `src/oxcalc-core/src/treecalc.rs`
4. `src/oxcalc-core/src/treecalc_fixture.rs`
5. `src/oxcalc-core/src/treecalc_runner.rs`

The new fixture proves:

1. predecessor formula descriptor: dynamic potential, no target,
2. successor formula descriptor: dynamic resolved, target `node:2`,
3. automatic post-edit seed reasons: `DependencyAdded`, `DependencyReclassified`,
4. invalidation closure requires rebind,
5. post-edit run rejects as `HostInjectedFailure`,
6. no publication is committed for the rebind-required transition.

## 5. Disposition

| Area | W043.2 result |
|---|---|
| Dynamic dependency transition coverage | narrowed with direct automatic `DependencyAdded` + `DependencyReclassified` evidence |
| Snapshot-fence counterpart | carried as declared-profile counterpart evidence |
| Capability-view counterpart | carried as declared-profile counterpart evidence |
| Callable value carrier | carried with fresh TreeCalc LET/LAMBDA value row |
| Callable metadata projection | exact blocker retained |
| W073 formatting | current typed-only guard carried; no OxCalc code change or handoff |
| Full optimized/core verification | exact blocker retained |

## 6. Remaining Exact Blockers

1. `w043_broader_dynamic_transition_coverage_remaining_exact_blocker`
   - The new evidence covers one automatic addition/reclassification pattern and carries the removal/reclassification pattern. It does not prove all dynamic reference families, multi-descriptor mixtures, structural-plus-formula edits, or production host-resolution behavior.
2. `w043_callable_metadata_projection_exact_blocker`
   - Current evidence proves ordinary value-carrier behavior but not callable identity metadata projection.
3. `w043_full_optimized_core_release_grade_conformance_exact_blocker`
   - Full optimized/core release-grade conformance still depends on broader dynamic coverage, callable metadata, Rust/proof, Stage 2, service, diversity, OxFml, pack/C5, and release-grade gates.

## 7. OxFml Formatting Intake

The latest local OxFml W073 formatting update is already aligned with W043 intake:

1. `VerificationConditionalFormattingRule.typed_rule` is the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are intentionally ignored for those aggregate and visualization families.
3. `thresholds` remains meaningful only for scalar/operator/expression rules.
4. W043.2 does not exercise conditional-formatting metadata and does not require an OxCalc code change.
5. No OxFml handoff is required by this bead.

## 8. Semantic-Equivalence Statement

This bead adds one checked-in fixture, one unit assertion for formula-catalog dynamic addition/reclassification seed derivation, runner assertions for the emitted artifacts, and evidence documentation.

For existing fixtures and existing runtime profiles, observable behavior is invariant: the fresh TreeCalc run has 0 expectation mismatches across the expanded 27-case corpus. The new fixture adds a previously unexercised transition and records the current runtime consequence for that transition: rebind-required rejection with no publication. It does not change scheduling policy, publication semantics, reject policy, overlay lifecycle semantics, Stage 2 partition policy, OxFml evaluator behavior, or OxFunc behavior.

## 9. Verification

| Command | Result |
|---|---|
| `cargo test -p oxcalc-core treecalc -- --nocapture` | passed; 31 tests |
| `cargo run -p oxcalc-tracecalc-cli -- treecalc w043-optimized-core-broad-conformance-treecalc-001` | passed; 27 cases, 0 expectation mismatches |
| JSON parse for W043.2 implementation-conformance artifacts | passed; 9 JSON artifacts parsed |
| `cargo fmt --all -- --check` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure: worksets=21; beads total=163, open=9, in_progress=0, ready=1, blocked=7, deferred=0, closed=154 |
| `br ready --json` | passed after bead closure; next ready bead is `calc-2p3.3` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed after bead closure; CRLF normalization warnings only |

## 10. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W043 README/status surfaces, feature map, fixture corpus, and machine-readable artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-2p3.9` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; W043 TreeCalc run `w043-optimized-core-broad-conformance-treecalc-001` records the new dynamic-addition fixture |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 8 states no policy/strategy change and the new fixture consequence |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; W073 typed-only formatting intake is carried and no handoff trigger exists |
| 6 | All required tests pass? | yes; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for the `calc-2p3.2` target; broader gaps are retained as exact blockers |
| 8 | Completion language audit passed? | yes; no full optimized/core, release-grade, C5, pack, Stage 2, callable metadata, callable carrier sufficiency, broad OxFml, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; no workset ordering changed |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-2p3.2` closure and `calc-2p3.3` readiness |

## 11. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-2p3.2` asks for optimized/core blocker attack with implementation work, direct differentials, or exact blocker retention |
| Gate criteria re-read | pass; no declared-gap match promotion and no full optimized/core verification claim appear |
| Silent scope reduction check | pass; dynamic, snapshot, capability, callable metadata, W073, and full conformance lanes are all classified |
| "Looks done but is not" pattern check | pass; the new dynamic evidence narrows one blocker but does not promote broad dynamic coverage |
| Result | pass for the `calc-2p3.2` target |

## 12. Three-Axis Report

- execution_state: `calc-2p3.2_optimized_core_broad_conformance_and_callable_metadata_closure_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - broader dynamic dependency-transition coverage remains partial beyond the new addition/reclassification and carried removal/reclassification fixture families
  - callable metadata projection remains an exact blocker
  - callable carrier sufficiency proof remains blocked
  - Rust totality/refinement and panic-free core proof frontier remains open
  - Lean/TLA full-verification and unbounded fairness discharge remains open
  - Stage 2 production partition analyzer and scheduler equivalence remains open
  - operated services, retained-history, retained-witness lifecycle, retention SLO, alert/quarantine dispatch, independent evaluator breadth, mismatch quarantine, OxFml seam breadth, pack/C5, and release-grade decision remain open
