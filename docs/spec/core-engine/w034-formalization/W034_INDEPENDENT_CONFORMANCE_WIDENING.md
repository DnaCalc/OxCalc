# W034 Independent Conformance Widening

Status: `calc-e77.3_evidence_packet`
Workset: `W034`
Parent epic: `calc-e77`
Bead: `calc-e77.3`
Created: 2026-05-05

## 1. Purpose

This packet records the W034 TreeCalc-local and TraceCalc comparison slice.

The target is to widen implementation-facing conformance evidence against the W034 TraceCalc oracle run without treating proxy pass signals as proof of unsupported local surfaces.

This slice updates the independent conformance runner from the W033 references to:

1. TraceCalc reference run: `w034-tracecalc-oracle-deepening-001`
2. TreeCalc-local run: `w034-independent-conformance-treecalc-001`
3. comparison run: `w034-independent-conformance-001`

The comparison remains an observable-surface differential between TraceCalc artifacts and TreeCalc-local artifacts. It is not a fully independent evaluator implementation, pack-grade replay, continuous differential service, Stage 2 promotion, or full OxFml/OxFunc semantic proof.

## 2. Corpus And Runner Changes

TreeCalc fixture corpus:

| Fixture | Purpose |
|---|---|
| `tc_local_w034_higher_order_let_lambda_publish_001` | value counterpart for the W034 higher-order `LET`/`LAMBDA` oracle scenario |
| `tc_local_w034_independent_order_equiv_001` | value counterpart for the W034 replay-equivalent independent-order oracle scenario |

Independent conformance row families:

| Family | Count | Meaning |
|---|---:|---|
| exact value rows | 5 | published values match between TraceCalc and TreeCalc-local artifacts |
| no-publication rows | 3 | verified-clean or reject surfaces do not publish |
| lifecycle rows | 1 | overlay retain/release lifecycle matches TreeCalc-local retention guardrail evidence |
| declared gap rows | 6 | TraceCalc covers a behavior that TreeCalc-local does not yet project as the same surface |
| unexpected mismatch rows | 0 | no observed implementation/spec mismatch in this packet |

The W034 rows deliberately separate value/no-publication matches from local capability gaps. For example, the capability-fence scenario has a no-publication match, but its capability-view fence semantics remain a declared local projection gap.

## 3. Evidence

Generated TreeCalc-local run:

```powershell
cargo run -p oxcalc-tracecalc-cli -- treecalc w034-independent-conformance-treecalc-001
```

Generated comparison run:

```powershell
cargo run -p oxcalc-tracecalc-cli -- independent-conformance w034-independent-conformance-001
```

Evidence roots:

1. `docs/test-runs/core-engine/treecalc-local/w034-independent-conformance-treecalc-001/run_summary.json`
2. `docs/test-runs/core-engine/treecalc-local/w034-independent-conformance-treecalc-001/cases/tc_local_w034_higher_order_let_lambda_publish_001/result.json`
3. `docs/test-runs/core-engine/treecalc-local/w034-independent-conformance-treecalc-001/cases/tc_local_w034_independent_order_equiv_001/result.json`
4. `docs/test-runs/core-engine/treecalc-local/w034-independent-conformance-treecalc-001/retention_guardrail.json`
5. `docs/test-runs/core-engine/independent-conformance/w034-independent-conformance-001/run_summary.json`
6. `docs/test-runs/core-engine/independent-conformance/w034-independent-conformance-001/surface_mapping.json`
7. `docs/test-runs/core-engine/independent-conformance/w034-independent-conformance-001/comparisons/treecalc_tracecalc_differential.json`
8. `docs/test-runs/core-engine/independent-conformance/w034-independent-conformance-001/comparisons/core_engine_projection_differential.json`
9. `docs/test-runs/core-engine/independent-conformance/w034-independent-conformance-001/replay-appliance/validation/bundle_validation.json`

Summary:

| Measure | Value |
|---|---:|
| TreeCalc-local fixture cases | 23 |
| TreeCalc-local expectation mismatches | 0 |
| comparison rows | 15 |
| exact value matches | 5 |
| no-publication matches | 3 |
| lifecycle matches | 1 |
| declared local gaps | 6 |
| missing artifacts | 0 |
| unexpected mismatches | 0 |
| bundle validation | `bundle_valid` |

## 4. Gap Classification

Declared local gaps in this packet:

| Row | Classification |
|---|---|
| `ic_gap_dynamic_dependency_001` | existing dynamic dependency projection gap retained |
| `ic_gap_lambda_host_effect_001` | existing host-sensitive `LET`/`LAMBDA` runtime-effect projection gap retained |
| `ic_gap_w034_dynamic_dependency_negative_001` | W034 dynamic dependency shape-update projection gap |
| `ic_gap_w034_snapshot_fence_projection_001` | snapshot-fence admission mismatch has no TreeCalc-local fixture counterpart yet |
| `ic_gap_w034_capability_view_fence_projection_001` | capability-view fence mismatch has no TreeCalc-local fence counterpart yet |
| `ic_gap_w034_higher_order_callable_metadata_001` | TreeCalc-local compares the value but not returned callable identity metadata |

These rows are not counted as conformance matches. They classify current local scope limits and feed later W034 proof/model/pack decisions.

## 5. Obligation Mapping

| W034 obligation | Evidence in this bead | Carry after this bead |
|---|---|---|
| `W034-OBL-004` `LET`/`LAMBDA` carrier breadth | higher-order value counterpart is exercised; callable metadata remains a declared local gap | Lean proof-family work and broader callable metadata modeling remain later W034 lanes |
| `W034-OBL-005` optimized/CoreEngine conformance | comparison packet widens W033 from 7 rows to 15 rows, with 0 unexpected mismatches | continuous differential and broader implementation diversity remain later lanes |
| `W034-OBL-006` independent evaluator diversity | TraceCalc and TreeCalc-local comparison is widened and still explicitly limited | fully independent evaluator implementation diversity remains unpromoted |
| `W034-OBL-012` direct OxFml fixture depth | TreeCalc-local adds W034 higher-order value counterpart through Raw OxFml | direct OxFml fixture bridge breadth remains carried |

## 6. Semantic-Equivalence Statement

This bead widens fixtures, conformance comparison row types, run references, and checked evidence roots. It does not change coordinator scheduling policy, candidate admission semantics, publication fence semantics, reject policy, dependency invalidation policy, TraceCalc transition semantics, OxFml-owned fixture content, Lean models, TLA models, pack decisions, or formatting/display seam meaning.

Existing TraceCalc scenarios remain referenced through the W034 21-scenario run, and existing W033 comparison surfaces remain present as value/no-publication/gap rows. New rows classify additional W034 surfaces without promoting unsupported local projection gaps as matches.

## 7. Verification

Commands run:

| Command | Result |
|---|---|
| `cargo test -p oxcalc-core treecalc_fixture::tests::checked_in_treecalc_fixtures_execute_against_local_runtime` | passed |
| `cargo test -p oxcalc-tracecalc independent_conformance::tests::independent_conformance_runner_writes_clean_diff_packet` | passed |
| `cargo run -p oxcalc-tracecalc-cli -- treecalc w034-independent-conformance-treecalc-001` | passed; 23 TreeCalc-local cases |
| `cargo run -p oxcalc-tracecalc-cli -- independent-conformance w034-independent-conformance-001` | passed; 15 comparison rows |
| `cargo test -p oxcalc-core` | passed; 49 unit tests, 5 integration tests |
| `cargo test -p oxcalc-tracecalc` | passed; 10 tests |
| `cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `cargo fmt --all -- --check` | passed |
| `scripts/check-worksets.ps1` | passed |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

## 8. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for in-scope items? | yes; this packet records row families, evidence roots, limits, and obligation mapping |
| 2 | Pack expectations updated for affected packs? | not promoted; this packet feeds later `calc-e77.6` pack/capability binding |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; TreeCalc-local and independent-conformance run roots are deterministic checked artifacts |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 6 states no runtime policy or strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no OxFml-owned seam change or concrete formatting/CF mismatch is introduced |
| 6 | All required tests pass? | yes; see Section 7 |
| 7 | No known semantic gaps remain in declared target? | yes for classifying matches, declared gaps, missing artifacts, and mismatches in this W034 comparison packet |
| 8 | Completion language audit passed? | yes; this packet does not claim full formalization, full independent evaluator diversity, pack-grade replay, Stage 2 promotion, or full OxFml/OxFunc semantics |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth did not change |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; W034 current-state text records this conformance widening slice |
| 11 | execution-state blocker surface updated? | yes; `calc-e77.3` is represented in `.beads/` |

## 9. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-e77.3` asks for widened optimized/core-engine and TreeCalc-to-TraceCalc comparison evidence against W034 oracle surfaces |
| Gate criteria re-read | pass; comparison artifacts classify exact value matches, no-publication matches, lifecycle matches, declared gaps, missing artifacts, and unexpected mismatches |
| Silent scope reduction check | pass; fully independent evaluator diversity, broad OxFml fixture depth, Lean/TLA widening, pack-grade replay, Stage 2 promotion, and continuous scale gates are explicitly carried |
| "Looks done but is not" pattern check | pass; declared gaps are not counted as conformance matches and proxy pass signals are not used as proof |
| Result | pass for the `calc-e77.3` declared conformance-widening target |

## 10. Three-Axis Report

- execution_state: `calc-e77.3_independent_conformance_widening_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-e77.4` Lean proof-family deepening
  - `calc-e77.5` TLA model-family and contention precondition slice
  - `calc-e77.6` pack capability and continuous scale gate binding
  - `calc-e77.7` W034 closure audit and successor packetization
