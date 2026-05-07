# W038 Independent Evaluator Diversity And OxFml Seam Watch Closure

Status: `calc-zsr.7_independent_diversity_seam_watch_validated`
Workset: `W038`
Parent epic: `calc-zsr`
Bead: `calc-zsr.7`

## 1. Purpose

This packet binds the W038 independent-evaluator diversity and OxFml seam-watch slice.

The target is not to promote a fully independent evaluator, operated cross-engine service, broad OxFml display/publication closure, general OxFunc kernel, pack-grade replay, C5, Stage 2 policy, or release-grade verification. The target is to state exactly what diversity evidence exists, what still fails the independent-implementation standard, and how current OxFml formatting/seam updates are carried without direct OxFml edits.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W038_CORE_FORMALIZATION_RELEASE_GRADE_CLOSURE_HARDENING.md` | W038 gate model and `calc-zsr.7` target |
| `docs/spec/core-engine/w038-formalization/W038_RESIDUAL_RELEASE_GRADE_OBLIGATION_LEDGER_AND_OBJECTIVE_MAP.md` | W038 obligations `W038-OBL-004`, `W038-OBL-007`, `W038-OBL-010`, `W038-OBL-017`, and `W038-OBL-018` |
| `docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/run_summary.json` | W036 diversity and projection evidence |
| `docs/test-runs/core-engine/cross-engine-differential/w036-independent-diversity-differential-001/run_summary.json` | W036 file-backed cross-engine differential evidence |
| `docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/run_summary.json` | W037 direct OxFml evaluator, narrow `LET`/`LAMBDA`, and W073 guard evidence |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w038-optimized-core-conformance-disposition-001/` | W038 conformance blockers, including callable metadata projection |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w038-proof-model-assumption-discharge-001/run_summary.json` | W038 general OxFunc-kernel no-promotion source row |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w038-stage2-partition-replay-001/run_summary.json` | W038 W073 formatting watch and Stage 2 no-promotion source row |
| `archive/test-runs-core-engine-w038-w045/operated-assurance/w038-operated-assurance-alert-quarantine-001/run_summary.json` | W038 operated-service no-promotion source row |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | inbound OxFml seam observations, including formatting and public consumer surface updates |

## 3. Artifact Surface

Run id: `w038-diversity-seam-watch-001`

| Artifact | Result |
|---|---|
| `src/oxcalc-tracecalc/src/diversity_seam.rs` | W038 diversity/seam-watch runner and validator |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w038-diversity-seam-watch-001/run_summary.json` | 7 source rows, 5 diversity rows, 8 OxFml seam-watch rows, 4 exact blockers, 0 failed rows |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w038-diversity-seam-watch-001/implementation_diversity_disposition.json` | diversity evidence classified without independent-evaluator promotion |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w038-diversity-seam-watch-001/oxfml_seam_watch_packet.json` | current OxFml seam-watch rows, including W073 typed-formatting intake |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w038-diversity-seam-watch-001/exact_diversity_seam_blocker_register.json` | 4 exact diversity/seam blockers |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w038-diversity-seam-watch-001/promotion_decision.json` | independent evaluator, general OxFunc kernel, callable metadata projection, pack/C5, and Stage 2 remain unpromoted |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w038-diversity-seam-watch-001/source_evidence_index.json` | source artifact index |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w038-diversity-seam-watch-001/validation.json` | validation status `w038_diversity_seam_packet_valid` |

## 4. Diversity Disposition

| Row | W038 disposition | Evidence consequence |
|---|---|---|
| TraceCalc reference machine | accepted boundary | correctness oracle for covered reference behavior, not independent optimized implementation |
| TreeCalc/CoreEngine projection | projection evidence only | useful differential signal, not independent implementation authority |
| file-backed cross-engine differential | accepted boundary | assurance input, not an operated service or independent evaluator |
| direct OxFml external evaluator slice | accepted external slice | strengthens formula seam confidence, not full OxCalc coordinator diversity |
| fully independent evaluator row set | exact remaining blocker | no independently implemented OxCalc evaluator row set exists |

## 5. OxFml Seam Watch

| Row | W038 disposition | Current read |
|---|---|---|
| W073 typed conditional formatting | aligned watch input | aggregate/visualization conditional-formatting families use `typed_rule`; legacy `thresholds` are not interpreted there |
| `format_delta` versus `display_delta` | aligned watch input | consequence categories remain distinct; broad display-facing closure is not inferred |
| direct OxFml runtime facade | direct external slice bound | current fixture slice drives OxFml through the runtime facade without reopening ownership |
| narrow `LET`/`LAMBDA` carrier | accepted boundary | included as a carrier seam through OxCalc/OxFml/OxFunc; general OxFunc kernels remain external |
| callable metadata projection | exact remaining blocker | value-carrier evidence exists, but callable metadata projection remains unpromoted |
| host/runtime and public consumer surface | note-level watch | current ordinary downstream use points at `consumer::runtime`, `consumer::editor`, and `consumer::replay` |
| structured-reference table packet | note-level watch | `table_catalog`, `enclosing_table_ref`, and `caller_table_region` remain the aligned first packet direction |
| registered-external packet | note-level watch | direct packet naming and seven-field descriptor read are converged notes, not coordinator API freeze |

No OxFml handoff is filed by this bead. The current update is compatible with the existing W037/W038 guarded evidence and does not expose an exercised OxCalc contract defect.

## 6. Exact Blockers

| Blocker | Owner | Consequence |
|---|---|---|
| `diversity.fully_independent_evaluator_absent` | `calc-zsr.7`; `calc-zsr.9` | fully independent evaluator diversity remains unpromoted |
| `diversity.operated_cross_engine_service_absent` | `calc-zsr.6`; `calc-zsr.7` | continuous cross-engine diversity claims remain unpromoted |
| `seam.callable_metadata_projection_absent` | `calc-zsr.7`; external `OxFunc` | callable metadata projection remains unpromoted |
| `seam.broad_oxfml_display_and_publication_breadth_unfrozen` | `calc-zsr.7`; OxFml watch lane | broad OxFml display/publication closure remains unpromoted until exercised evidence requires and supports it |

## 7. Semantic-Equivalence Statement

This bead adds a diversity/seam-watch runner path, emitted evidence artifacts, and status/spec text.

It does not change evaluator kernels, coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc semantics, TreeCalc runtime behavior, OxFml evaluator behavior, OxFunc kernels, Lean/TLA model semantics, pack/C5 capability policy, Stage 2 scheduler policy, service operation, or alert-dispatch behavior.

Observable runtime behavior is invariant under this bead. The packet classifies evidence and exact blockers only.

## 8. Verification

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `cargo test -p oxcalc-tracecalc diversity_seam -- --nocapture` | passed; 1 test |
| `cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `cargo run -p oxcalc-tracecalc-cli -- diversity-seam w038-diversity-seam-watch-001` | passed; emitted W038 diversity/seam-watch artifacts |
| JSON parse for `archive/test-runs-core-engine-w038-w045/diversity-seam/w038-diversity-seam-watch-001/*.json` | passed |
| `cargo test -p oxcalc-tracecalc` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-zsr.8` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

## 9. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W038 README/status surfaces, machine-readable artifacts, and feature map record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack-grade replay and C5 remain unpromoted and `calc-zsr.8` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; this bead binds deterministic source evidence, diversity disposition, seam-watch, blocker, decision, and validation artifacts |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 7 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; current OxFml formatting and seam notes are carried as watch evidence, and no handoff trigger exists |
| 6 | All required tests pass? | yes; see Section 8 |
| 7 | No known semantic gaps remain in declared scope? | yes for the local evidence/disposition target; independent evaluator, operated cross-engine service, callable metadata, and broad display/publication breadth remain exact blockers |
| 8 | Completion language audit passed? | yes; the packet limits claims to evidence binding and keeps independent evaluator, service, broad OxFml, general OxFunc, pack/C5, Stage 2, and release-grade claims unpromoted |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W038 diversity/seam-watch state |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-zsr.7` closure and `calc-zsr.8` readiness |

## 10. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-zsr.7` asks to strengthen implementation diversity and OxFml seam evidence, including current formatting watch rows, direct evaluator boundaries, and the narrow `LET`/`LAMBDA` carrier |
| Gate criteria re-read | pass; independent evaluator claims are blocked because no independently implemented row set exists |
| Silent scope reduction check | pass; the packet does not count shared projections, file-backed differentials, or external evaluator slices as full independent OxCalc evaluator evidence |
| "Looks done but is not" pattern check | pass; watch rows and direct external slices are not represented as broad OxFml closure, general OxFunc kernels, operated service, pack-grade replay, C5, Stage 2 policy, or release-grade verification |
| Result | pass for the `calc-zsr.7` diversity/seam-watch disposition target |

## 11. Three-Axis Report

- execution_state: `calc-zsr.7_independent_diversity_seam_watch_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - fully independent evaluator diversity remains unpromoted
  - operated cross-engine differential service remains unpromoted
  - callable metadata projection remains unpromoted
  - broad OxFml display/publication closure remains unpromoted
  - general OxFunc kernels remain outside OxCalc scope except the narrow `LET`/`LAMBDA` carrier seam
  - pack-grade replay governance, C5, and W038 release decision remain open under `calc-zsr.8` and `calc-zsr.9`
  - full Lean/TLA verification and release-grade verification remain unpromoted
