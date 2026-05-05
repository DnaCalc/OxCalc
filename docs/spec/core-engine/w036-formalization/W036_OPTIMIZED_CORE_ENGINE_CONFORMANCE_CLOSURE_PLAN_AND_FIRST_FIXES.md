# W036 Optimized/Core-Engine Conformance Closure Plan And First Fixes

Status: `calc-rqq.3_optimized_core_engine_conformance_closure_plan_validated`
Workset: `W036`
Parent epic: `calc-rqq`
Bead: `calc-rqq.3`

## 1. Purpose

This packet converts the six W035 implementation-conformance deferrals into W036 action rows without treating any declared gap as a match.

The target is narrower than full optimized/core-engine verification. It records which W035 rows now have first-fix harness evidence, which rows remain routed to blocker beads, and which promotion consequences remain open for later W036 work.

## 2. Authority Inputs Reviewed

| Input | Role in this bead |
|---|---|
| `docs/spec/core-engine/w035-formalization/W035_IMPLEMENTATION_CONFORMANCE_HARDENING.md` | source W035 six-gap disposition packet |
| `docs/test-runs/core-engine/implementation-conformance/w035-implementation-conformance-hardening-001/` | deterministic W035 gap-disposition evidence |
| `docs/spec/core-engine/w036-formalization/W036_RESIDUAL_COVERAGE_AND_PROMOTION_BLOCKER_LEDGER.md` | W036 obligations `W036-OBL-003` through `W036-OBL-008` |
| `docs/spec/core-engine/w036-formalization/W036_TRACECALC_COVERAGE_CLOSURE_CRITERIA_AND_MATRIX_EXPANSION.md` | W036 TraceCalc coverage matrix source |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/` | deterministic W036 oracle-matrix evidence |
| `docs/test-runs/core-engine/independent-conformance/w034-independent-conformance-001/` | TreeCalc/CoreEngine projection and differential source |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | current inbound OxFml observation ledger |

## 3. Implementation Surface

| Surface | Change |
|---|---|
| `src/oxcalc-tracecalc/src/implementation_conformance.rs` | adds a W036 run profile for closure action, blocker, and match-promotion guard artifacts |
| `src/oxcalc-tracecalc-cli/src/main.rs` | reports W036 action counts for `implementation-conformance <w036-run-id>` |
| `docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/` | checked-in W036 implementation-conformance closure evidence root |

No runtime evaluator, coordinator, dependency graph, recalc, publication, or OxFml behavior changes in this bead.

## 4. Evidence Summary

Run id: `w036-implementation-conformance-closure-001`

| Metric | Value |
|---|---:|
| Source W035 gap disposition rows | 6 |
| W036 closure action rows | 6 |
| Harness first-fix rows | 2 |
| Blocker-routed rows | 4 |
| Promoted match rows | 0 |
| Validated action rows | 6 |
| Failed action rows | 0 |
| Source W035 implementation-work deferrals | 5 |
| Source W035 spec-evolution deferrals | 1 |
| Validation status | `implementation_conformance_w036_closure_plan_valid` |

Primary artifacts:

1. `docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/run_summary.json`
2. `docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/w036_closure_action_register.json`
3. `docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/w036_blocker_register.json`
4. `docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/w036_match_promotion_guard.json`
5. `docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/evidence_summary.json`
6. `docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/validation.json`

## 5. W036 Disposition Register

| W035 gap row | W036 obligation | W036 disposition | Evidence or blocker | Promotion consequence |
|---|---|---|---|---|
| `ic_gap_dynamic_dependency_001` | `W036-OBL-003` | harness first-fix row binds positive dynamic-switch replay to current TreeCalc runtime-effect boundary | evidence: `w035_dependency_dynamic_switch_publish`, `w036_dynamic_dependency_switch_seed`, W034 core projection | not a match; optimized/core-engine conformance remains blocked until positive dynamic-bind update differential evidence exists |
| `ic_gap_lambda_host_effect_001` | `W036-OBL-004` | formal deferral to LET/LAMBDA boundary inventory | blocker bead: `calc-rqq.4` | LET/LAMBDA carrier conformance remains bounded until callable/OxFunc-opaque boundary inventory is recorded |
| `ic_gap_w034_dynamic_dependency_negative_001` | `W036-OBL-005` | harness first-fix row binds dynamic negative and release/reclassification replay to shape-update harness requirements | evidence: `w035_dependency_dynamic_negative`, `w035_dependency_dynamic_release_publish`, W034 differential | not a match; optimized/core-engine conformance remains blocked until optimized-lane differential evidence exists |
| `ic_gap_w034_snapshot_fence_projection_001` | `W036-OBL-006` | coordinator harness blocker | blocker bead: `calc-rqq.5` | snapshot-fence conformance remains blocked until TLA/coordinator replay supplies a local counterpart or explicit external ownership |
| `ic_gap_w034_capability_view_fence_projection_001` | `W036-OBL-007` | coordinator harness blocker | blocker bead: `calc-rqq.5` | capability-view fence conformance remains blocked until TLA/coordinator replay supplies a local counterpart or explicit external ownership |
| `ic_gap_w034_higher_order_callable_metadata_001` | `W036-OBL-008` | formal deferral to callable boundary inventory | blocker bead: `calc-rqq.4` | callable metadata projection remains blocked until carrier sufficiency is proven or a concrete metadata projection fixture exists |

## 6. Match-Promotion Guard

`w036_match_promotion_guard.json` records:

| Field | Value |
|---|---:|
| Source declared gap rows | 6 |
| Promoted match rows | 0 |
| Non-promoted rows | 6 |
| Guard status | `no_declared_gap_promoted_as_match` |

The guard is intentionally strict: W036 may add harness evidence and blocker routing, but no W035 declared gap is counted as an optimized/core-engine conformance match in this bead.

## 7. OxFml Watch

No OxFml handoff is filed by this bead.

The LET/LAMBDA rows remain within the OxCalc/OxFml carrier fragment. The full OxFunc LAMBDA semantic kernel remains excluded from OxCalc-local TraceCalc oracle coverage and routed to the `calc-rqq.4` boundary inventory. W073 typed conditional-formatting metadata remains watch/input-contract evidence only; this bead does not construct conditional-formatting payloads.

## 8. Semantic-Equivalence Statement

This bead adds evidence classification, W036 conformance-action artifact emission, CLI reporting, checked artifacts, and documentation only.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, TraceCalc execution semantics, TreeCalc/CoreEngine runtime behavior, Lean/TLA model semantics, pack-decision logic, continuous-assurance runner semantics, or OxFml/OxFunc evaluator behavior.

Observable runtime behavior is invariant under this bead. The two harness first-fix rows are evidence-binding actions, not runtime semantic changes.

## 9. Verification

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `$env:CARGO_TARGET_DIR='target\w036-test-target'; cargo test -p oxcalc-tracecalc implementation_conformance` | passed; 2 tests |
| `$env:CARGO_TARGET_DIR='target\w036-test-target'; cargo run -p oxcalc-tracecalc-cli -- implementation-conformance w036-implementation-conformance-closure-001` | passed; emitted 6 W036 closure actions, 2 first-fix harness rows, 4 blocker-routed rows, 0 match-promoted rows, 0 failed rows |
| `$env:CARGO_TARGET_DIR='target\w036-test-target'; cargo test -p oxcalc-core treecalc_runtime_facade_exposes_dynamic_dependency_family_directly` | passed; 1 test |
| `$env:CARGO_TARGET_DIR='target\w036-test-target'; cargo test -p oxcalc-tracecalc` | passed; 18 tests |
| `$env:CARGO_TARGET_DIR='target\w036-test-target'; cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `scripts/check-worksets.ps1` | passed; `worksets=14`, `beads total=89`, `open=7`, `in_progress=1`, `ready=0`, `blocked=6`, `closed=81` |
| `br dep cycles --json` | passed; `count=0` |
| `git diff --check` | passed; CRLF normalization warnings only |

The default shared `target` directory produced `LINK : fatal error LNK1201` while writing the `oxcalc_tracecalc` PDB during the first focused test attempt. The same focused test passed in isolated target directory `target\w036-test-target`, so the recorded validation uses the isolated target path for Rust test/run commands.

## 10. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W036 status surfaces, and generated artifacts record the six-row W036 disposition |
| 2 | Pack expectations updated for affected packs? | yes; pack remains unpromoted and `calc-rqq.8` owns reassessment after later W036 evidence |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; W036 action rows reference deterministic W036 TraceCalc matrix evidence, W034 conformance evidence, and W035 gap-disposition evidence |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 8 states no runtime policy or strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no OxFml/FEC/F3E change or handoff trigger exists for this bead |
| 6 | All required tests pass? | yes; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for this target; all six W035 gap rows have W036 action disposition, evidence, or blocker-bead routing, and none are promoted as matches |
| 8 | Completion language audit passed? | yes; no full optimized/core-engine verification, pack promotion, or full formalization claim is made |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 points at this W036 conformance evidence |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-rqq.3` execution state and later closure evidence |

## 11. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-rqq.3` asks for W035 implementation deferrals to become optimized/core-engine fixes, harness work, or formal deferrals |
| Gate criteria re-read | pass; every W035 declared gap has a W036 disposition with implementation evidence or a blocker bead |
| Silent scope reduction check | pass; dynamic rows are harness-evidence actions, not conformance matches; other rows are routed to `calc-rqq.4` or `calc-rqq.5` |
| "Looks done but is not" pattern check | pass; no full optimized/core-engine conformance, full TraceCalc oracle, pack, Stage 2, Lean/TLA, or continuous assurance promotion is claimed |
| Result | pass for the `calc-rqq.3` conformance-closure target |

## 12. Three-Axis Report

- execution_state: `calc-rqq.3_optimized_core_engine_conformance_closure_plan_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-rqq.4` through `calc-rqq.9` remain open
  - full optimized/core-engine verification remains partial because zero W035 declared gaps are promoted as matches
  - callable/OxFunc boundary inventory remains routed to `calc-rqq.4`
  - snapshot/capability fence coordinator modeling remains routed to `calc-rqq.5`
  - full Lean/TLA verification, concrete Stage 2 partition/replay equivalence, independent evaluator diversity, cross-engine differential service, continuous assurance operation/history, pack-grade replay, and Stage 2 policy remain partial
