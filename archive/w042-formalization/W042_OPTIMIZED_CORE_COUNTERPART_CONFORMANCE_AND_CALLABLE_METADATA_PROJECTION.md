# W042 Optimized/Core Counterpart Conformance And Callable Metadata Projection

Status: `evidence_packet_validated`

Bead: `calc-czd.2`

Run id: `w042-optimized-core-counterpart-conformance-callable-metadata-001`

Supporting TreeCalc run id: `w042-optimized-core-counterpart-conformance-treecalc-001`

## Purpose

This packet attacks the W042 optimized/core obligations assigned to `calc-czd.2`.

The new replay evidence is a fresh W042 TreeCalc run over the current 26-case local corpus. It rebinds the W041 automatic dynamic dependency-transition fixture, capability-sensitive reject evidence, and LET/LAMBDA value-carrier fixture under a W042 artifact root. The conformance packet also binds W041 Stage 2 declared-profile snapshot-fence and capability-view counterpart rows into the W042 optimized/core ledger without treating those rows as production Stage 2 policy or full optimized/core verification.

Callable metadata projection remains an exact blocker. Current TreeCalc and upstream-host rows exercise ordinary value publication and narrow callable carrier behavior; they do not project callable identity metadata as an optimized conformance surface.

## Changed Surfaces

| Path | Change |
|---|---|
| `archive/test-runs-core-engine-w038-w045/treecalc-local/w042-optimized-core-counterpart-conformance-treecalc-001/` | fresh W042 TreeCalc replay over the current local corpus |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/` | W042 implementation-conformance packet |
| `docs/spec/core-engine/w042-formalization/W042_OPTIMIZED_CORE_COUNTERPART_CONFORMANCE_AND_CALLABLE_METADATA_PROJECTION.md` | this evidence packet |
| `docs/spec/core-engine/w042-formalization/README.md` | W042 packet index update |
| `docs/worksets/W042_CORE_FORMALIZATION_RELEASE_GRADE_EVIDENCE_CLOSURE_EXPANSION.md` | W042 current-status update |
| `docs/IN_PROGRESS_FEATURE_WORKLIST.md` | feature-map update |

No Rust source, fixture definition, formal model, or OxFml source file changes are made by this bead.

## Evidence Packet

| Artifact | Purpose |
|---|---|
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/run_summary.json` | records W042.2 counts, remaining blockers, no-promotion claims, and next bead |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/w042_counterpart_conformance_register.json` | records dynamic, snapshot, capability, callable, no-proxy, and full-conformance disposition rows |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/w042_callable_metadata_projection_register.json` | separates callable value/carrier evidence from callable metadata projection |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/w042_exact_remaining_blocker_register.json` | retains three exact W042 optimized/core blockers |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/w042_match_promotion_guard.json` | records zero match promotion |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/w073_formatting_intake.json` | records the current OxFml W073 typed-only formatting working-tree intake |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/source_evidence_index.json` | indexes W042 TreeCalc, W041 Stage 2, W041 OxFml seam, W041 upstream-host, W042 ledger, and current OxFml W073 working-tree sources |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/validation.json` | records validation commands and artifact checks |

The direct replay evidence is:

1. `archive/test-runs-core-engine-w038-w045/treecalc-local/w042-optimized-core-counterpart-conformance-treecalc-001/run_summary.json`,
2. `archive/test-runs-core-engine-w038-w045/treecalc-local/w042-optimized-core-counterpart-conformance-treecalc-001/cases/tc_local_dynamic_release_reclassification_auto_post_edit_001/post_edit/invalidation_seeds.json`,
3. `archive/test-runs-core-engine-w038-w045/treecalc-local/w042-optimized-core-counterpart-conformance-treecalc-001/cases/tc_local_dynamic_release_reclassification_auto_post_edit_001/post_edit/invalidation_closure.json`,
4. `archive/test-runs-core-engine-w038-w045/treecalc-local/w042-optimized-core-counterpart-conformance-treecalc-001/cases/tc_local_capability_sensitive_reject_001/result.json`,
5. `archive/test-runs-core-engine-w038-w045/treecalc-local/w042-optimized-core-counterpart-conformance-treecalc-001/cases/tc_local_w034_higher_order_let_lambda_publish_001/result.json`.

Observed replay facts:

1. TreeCalc emitted 26 cases with 0 expectation mismatches.
2. The automatic dynamic transition post-edit case derives `DependencyRemoved` and `DependencyReclassified` seeds.
3. The capability-sensitive case rejects without publication and emits a capability-sensitive runtime effect/overlay surface.
4. The higher-order LET/LAMBDA TreeCalc case publishes ordinary value `17`.
5. The LET/LAMBDA TreeCalc case does not project callable identity metadata.

## Counterpart Conformance Result

| Row | Result | Consequence |
|---|---|---|
| automatic dynamic transition | fresh W042 replay binds the W041 resolved-to-potential transition evidence | exercised pattern remains evidenced; broader transition coverage remains blocked |
| snapshot-fence counterpart | W041 Stage 2 declared-profile snapshot reject/no-publish counterpart is bound | evidenced for declared profiles; not production policy or full optimized/core promotion |
| capability-view counterpart | W041 Stage 2 declared-profile capability-view reject/no-publish counterpart is bound | evidenced for declared profiles; broader capability semantics remain blocked |
| callable value carrier | W042 TreeCalc and W041 upstream-host evidence bind ordinary value/carrier behavior | callable metadata projection and carrier sufficiency remain unpromoted |
| no-proxy guard | zero match promotion | bounded, declared-profile, and value-only rows are not counted as full promotion |

## Remaining Exact Blockers

| Blocker | Owner |
|---|---|
| `w042_broader_dynamic_transition_coverage_exact_blocker` | `calc-czd.3`, `calc-czd.4`, `calc-czd.5` |
| `w042_callable_metadata_projection_exact_blocker` | `calc-czd.4`, `calc-czd.8` |
| `w042_full_optimized_core_release_grade_conformance_exact_blocker` | `calc-czd.10` |

## OxFml Formatting Intake

The current OxFml W073 working-tree update does not require a W042.2 optimized/core code change.

The typed-only rule remains active for W042 later lanes: `VerificationConditionalFormattingRule.typed_rule` is the accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`; W072 bounded `thresholds` strings are not fallback metadata for those aggregate and visualization families. The updated OxFml evidence now also includes old-string non-interpretation tests for visualization strings and aggregate option strings.

The W071/W072 output carrier remains stable for `effective_fill_color`, `data_bar`, and `icon`. OxCalc records the input-contract guard here and leaves any downstream request-construction uptake or mismatch check to the W042 OxFml/public migration lane `calc-czd.8`.

## Semantic-Equivalence Statement

This bead adds a fresh TreeCalc replay run, conformance artifacts, documentation, and bead state only. It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc semantics, TreeCalc/CoreEngine runtime behavior, Lean/TLA model semantics, pack-decision logic, service behavior, retained-history behavior, alert dispatch behavior, or OxFml/OxFunc evaluator behavior.

Observable runtime behavior is invariant under this bead because the only executable action reruns the existing TreeCalc local corpus and emits artifacts for the current behavior.

## Validation

| Command | Result |
|---|---|
| `cargo run -p oxcalc-tracecalc-cli -- treecalc w042-optimized-core-counterpart-conformance-treecalc-001` | passed; emitted 26 cases with 0 expectation mismatches |
| JSON parse for `archive/test-runs-core-engine-w038-w045/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/*.json` | passed |
| OxFml W073 working-tree intake review | passed; typed-only aggregate/visualization input contract recorded with no `calc-czd.2` core-engine code change |
| JSON parse for W042 TreeCalc run summary and selected case artifacts | passed |
| `cargo test -p oxcalc-core treecalc_runner` | passed; 3 tests passed |
| `scripts/check-worksets.ps1` | passed; `ready=1`, `closed=143` |
| `br ready --json` | passed; `calc-czd.3` ready |
| `br dep cycles --json` | passed; `cycles=0` |
| `git diff --check` | passed; CRLF normalization warnings only |

No Lean or TLC command is required for this bead because proof/model successor work is owned by later W042 beads.

## Status Report

- execution_state: `calc-czd.2_optimized_core_counterpart_conformance_callable_metadata_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-czd.3` Rust totality/refinement and core panic-boundary closure
  - broader dynamic dependency-transition coverage remains partial
  - callable metadata projection remains an exact blocker
  - W073 typed-only conditional-formatting request-construction uptake remains a later OxFml/public migration lane
  - full optimized/core verification remains unpromoted
  - release-grade verification, C5, pack-grade replay, Stage 2 policy, operated services, independent evaluator breadth, mismatch quarantine, broad OxFml display/publication, public migration, callable carrier sufficiency, registered-external callable projection, provider-failure/callable-publication semantics, and general OxFunc kernels remain unpromoted

## Pre-Closure Verification Checklist

| Check | Result |
|---|---|
| Workset and bead ids are explicit | yes: `W042`, `calc-czd.2` |
| Required artifacts exist | yes: W042.2 packet artifacts and W042 TreeCalc replay artifacts are present |
| Direct replay evidence exists for changed evidence surface | yes: fresh W042 TreeCalc run with selected dynamic, capability, and LET/LAMBDA case artifacts |
| No declared gap is match-promoted | yes: `w042_match_promotion_guard.json` records zero match promotion |
| Residual blockers are explicit | yes: three exact remaining blocker rows |
| Semantic-equivalence statement is present | yes |

## Completion Claim Self-Audit

| Audit Item | Result |
|---|---|
| Claim is limited to `calc-czd.2` target | yes |
| Scaffolding is not reported as implementation | yes |
| Spec text has replay evidence | yes |
| Cross-repo handoff is not treated as closure | yes; OxFml W073 remains a later seam-watch lane |
| Uncertain lanes default to in-progress | yes; remaining exact blockers are retained |
| Strategy-change equivalence statement is present | yes |
