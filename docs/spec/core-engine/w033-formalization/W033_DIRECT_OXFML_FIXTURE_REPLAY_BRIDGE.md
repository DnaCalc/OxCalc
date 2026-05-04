# W033 Direct OxFml Fixture Replay Bridge

Status: `calc-8pe_evidence_packet`
Workset: `W033`
Successor bead: `calc-8pe`
Created: 2026-05-04

## 1. Purpose

This packet records the first post-W033 direct OxFml fixture intake inside OxCalc.

The bridge reads OxFml-owned fixture JSON from `../OxFml/crates/oxfml_core/tests/fixtures/`, emits OxCalc-owned projection artifacts, compares the fixture cases that have current OxCalc counterparts against W033 TraceCalc/TreeCalc evidence, and reports whether a concrete OxFml handoff trigger appears.

## 2. Runner Surface

The executable surface is:

```powershell
cargo run -p oxcalc-tracecalc-cli -- oxfml-bridge <run-id>
```

The runner is `OxFmlFixtureBridgeRunner` in `src/oxcalc-tracecalc/src/oxfml_fixture_bridge.rs`.

It projects these fixture families:

| Family | Source fixture | Bridge treatment |
|---|---|---|
| `fec_commit` | `fec_commit_replay_cases.json` | direct projection plus current TraceCalc/TreeCalc comparison where mapped |
| `session_lifecycle` | `session_lifecycle_replay_cases.json` | direct projection plus current comparison for shared accept/reject/fence classes |
| `prepared_call` | `prepared_call_replay_cases.json` | direct projection; exercised prepared-call/carrier comparison remains successor-scoped |
| `higher_order_callable` | `higher_order_callable_cases.json` | direct projection; exercised callable-carrier widening remains successor-scoped |

## 3. Evidence Run

Run:

```powershell
cargo run -p oxcalc-tracecalc-cli -- oxfml-bridge post-w033-direct-oxfml-fixture-bridge-001
```

Artifacts:

1. `docs/test-runs/core-engine/oxfml-fixture-bridge/post-w033-direct-oxfml-fixture-bridge-001/run_summary.json`
2. `docs/test-runs/core-engine/oxfml-fixture-bridge/post-w033-direct-oxfml-fixture-bridge-001/fixture_index.json`
3. `docs/test-runs/core-engine/oxfml-fixture-bridge/post-w033-direct-oxfml-fixture-bridge-001/family-projections/*.json`
4. `docs/test-runs/core-engine/oxfml-fixture-bridge/post-w033-direct-oxfml-fixture-bridge-001/comparisons/comparison_summary.json`
5. `docs/test-runs/core-engine/oxfml-fixture-bridge/post-w033-direct-oxfml-fixture-bridge-001/handoff/handoff_watch.json`
6. `docs/test-runs/core-engine/oxfml-fixture-bridge/post-w033-direct-oxfml-fixture-bridge-001/replay-appliance/bundle_manifest.json`
7. `docs/test-runs/core-engine/oxfml-fixture-bridge/post-w033-direct-oxfml-fixture-bridge-001/replay-appliance/validation/bundle_validation.json`

Run summary:

| Measure | Value |
|---|---:|
| fixture families | 4 |
| fixture cases | 45 |
| comparison rows | 45 |
| current-evidence matches | 6 |
| deferred current counterparts | 39 |
| missing current evidence | 0 |
| current-evidence mismatches | 0 |
| handoff triggered | false |
| bundle validation | `bundle_valid`, `missing_paths: []` |

## 4. Comparison Classification

The matched rows are limited to cases with a current, specific TraceCalc or TreeCalc counterpart:

| OxFml fixture case | Current counterpart |
|---|---|
| `fec_001_accept` | `tc_accept_publish_001`; `tc_local_publish_001` |
| `fec_002_formula_token_reject` | `tc_artifact_token_reject_001` |
| `fec_003_capability_view_reject` | `tc_reject_no_publish_001`; `tc_local_capability_sensitive_reject_001` |
| `session_001_commit` | `tc_accept_publish_001`; `tc_local_publish_001` |
| `session_002_capability_denied` | `tc_reject_no_publish_001`; `tc_local_capability_sensitive_reject_001` |
| `session_007_commit_stale_fence` | `tc_publication_fence_reject_001` |

The remaining rows are not failures. They are explicit `deferred_no_current_counterpart` rows because current OxCalc evidence does not yet carry an exact session lifecycle, prepared-call, or higher-order callable counterpart for those fixture cases.

## 5. Handoff Decision

Current decision: `no_new_handoff_required`.

Rationale:

1. all six mapped current-evidence comparisons matched,
2. no mapped comparison reported missing evidence,
3. no mapped comparison reported a current-evidence mismatch,
4. deferred rows are OxCalc successor-scope coverage gaps rather than OxFml contradictions,
5. `docs/handoffs/HANDOFF_REGISTER.csv` does not need a new row for this bead.

## 6. Successor Carry

This bridge intentionally does not promote pack-grade replay or direct OxFml evaluator re-execution.

Successor pressure remains:

1. `calc-688`: exercise LET/LAMBDA and callable carrier witnesses directly,
2. `calc-y0r`: widen independent conformance beyond shared-executor/current-evidence comparisons,
3. `calc-lwh`: consider pack-grade promotion only after governance and replay-appliance evidence widen,
4. `calc-rcr`: widen formal model families over FEC/session/callable/dynamic surfaces,
5. `calc-8lg`: bind metamorphic and scale evidence semantically.

## 7. Semantic-Equivalence Statement

This bead adds a read-only fixture projection and comparison runner plus evidence artifacts. It does not change coordinator scheduling, recalc strategy, publication behavior, FEC/F3E semantics, TreeCalc execution, TraceCalc semantics, or OxFml fixtures.

Observable engine behavior for existing W033 TraceCalc and TreeCalc evidence remains invariant. The bridge only reads that evidence and classifies overlap with OxFml fixture cases.

## 8. Verification

Commands run:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `cargo test -p oxcalc-tracecalc -p oxcalc-tracecalc-cli` | passed: 7 tracecalc tests, 0 CLI tests |
| `cargo run -p oxcalc-tracecalc-cli -- oxfml-bridge post-w033-direct-oxfml-fixture-bridge-001` | passed; projected 45 fixture cases across 4 families |
| `cargo test --workspace` | passed: 48 `oxcalc_core` tests, 5 upstream-host tests, 7 `oxcalc_tracecalc` tests, 0 CLI tests, 0 doctest failures |
| `cargo clippy --workspace --all-targets -- -D warnings` | passed |

## 9. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for in-scope items? | yes; this packet records runner surface, fixture families, evidence run, comparison classification, handoff decision, and successor carry |
| 2 | Pack expectations updated for affected packs? | yes; no pack-grade replay promotion is made, and pack-grade work remains successor-scoped |
| 3 | At least one deterministic replay/projection artifact exists per in-scope behavior? | yes; the bridge emitted fixture index, family projections, comparison summary, handoff watch, bundle manifest, and bundle validation for `post-w033-direct-oxfml-fixture-bridge-001` |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 7 states the read-only bridge changes no engine scheduling, recalc, publication, FEC/F3E, TraceCalc, TreeCalc, or OxFml behavior |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; handoff watch reports `no_new_handoff_required`, `handoff_triggered: false`, and `concrete_upstream_mismatch_count: 0` |
| 6 | All required tests pass? | yes; see Section 8 plus `git diff --check`, `scripts/check-worksets.ps1`, and `br dep cycles --json` |
| 7 | No known semantic gaps remain in declared target? | yes; no known gap remains unpacketized for this bridge target, and broader callable/prepared/pack/formal lanes remain explicit successors |
| 8 | Completion language audit passed? | yes; this packet separates bridge evidence from successor-scoped coverage and does not claim direct evaluator re-execution or pack-grade replay |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth did not change |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | not applicable; feature-map truth did not change |
| 11 | execution-state blocker surface updated? | yes; `calc-8pe` is represented in `.beads/` and successor lanes remain ordinary bead graph entries |

## 10. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-8pe` asked for direct OxFml fixture replay/projection, comparison to current TraceCalc/TreeCalc outcomes where possible, and mismatch-driven handoff only |
| Gate criteria re-read | pass; bridge code, CLI surface, deterministic artifacts, comparison summary, handoff watch, and validation evidence exist for the declared target |
| Silent scope reduction check | pass; exact-current-counterpart gaps are not hidden and are carried as successor lanes rather than counted as matches |
| "Looks done but is not" pattern check | pass; projection-only families are reported as projection-only, not as exercised evaluator or callable carrier conformance |
| Result | pass for the `calc-8pe` declared bridge target |

## 11. Three-Axis Report

- execution_state: `calc-8pe_evidence_packet_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - prepared-call and higher-order callable direct exercised witnesses remain successor-scoped
  - pack-grade replay remains unpromoted
  - broader independent conformance and formal model widening remain successor-scoped
