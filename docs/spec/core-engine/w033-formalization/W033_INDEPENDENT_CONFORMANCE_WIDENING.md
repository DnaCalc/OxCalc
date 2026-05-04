# W033 Independent Conformance Widening

Status: `calc-y0r_evidence_packet`
Workset: `W033`
Successor bead: `calc-y0r`
Created: 2026-05-04

## 1. Purpose

This packet records the first post-W033 widening beyond the shared TraceCalc reference-machine/engine-machine comparison.

The target is not a fully independent evaluator implementation. The target is narrower and explicit:

1. add TreeCalc-local fixtures that mirror selected TraceCalc observable value surfaces,
2. run those fixtures through the TreeCalc/CoreEngine fixture runner,
3. compare checked TraceCalc artifacts against checked TreeCalc artifacts over the declared W033 observable surface,
4. keep capability gaps visible as declared gaps rather than treating them as conformance passes,
5. emit deterministic artifacts that later pack, formal-model, and scale/metamorphic work can consume.

## 2. Executable Surface

The widened evidence uses two commands:

```powershell
cargo run -p oxcalc-tracecalc-cli -- treecalc post-w033-independent-conformance-treecalc-001
cargo run -p oxcalc-tracecalc-cli -- independent-conformance post-w033-independent-conformance-001
```

The `independent-conformance` runner is `IndependentConformanceRunner` in `src/oxcalc-tracecalc/src/independent_conformance.rs`.

The runner consumes:

| Evidence source | Run id |
|---|---|
| TraceCalc oracle/reference artifacts | `post-w033-let-lambda-carrier-witness-001` |
| TreeCalc/CoreEngine local artifacts | `post-w033-independent-conformance-treecalc-001` |

## 3. New TreeCalc Counterpart Fixtures

This bead adds two TreeCalc-local fixtures whose formulas intentionally mirror TraceCalc value surfaces:

| Fixture | TraceCalc counterpart | Checked value surface |
|---|---|---|
| `tc_local_tracecalc_accept_publish_equiv_001` | `tc_accept_publish_001` | TraceCalc `B=2` equals TreeCalc node `3=2` |
| `tc_local_tracecalc_multinode_dag_equiv_001` | `tc_multinode_dag_publish_001` | TraceCalc `B=3`, `C=3`, `D=6` equal TreeCalc nodes `3=3`, `4=3`, `5=6` |

The existing LET/LAMBDA fixture `tc_local_let_lambda_capture_publish_001` is reused as the third exact value comparison against `tc_let_lambda_carrier_publish_001`.

## 4. Evidence Run

TreeCalc source run:

| Measure | Value |
|---|---:|
| cases | 21 |
| published | 12 |
| rejected | 8 |
| verified clean | 1 |
| expectation mismatches | 0 |

Artifact root:

`docs/test-runs/core-engine/treecalc-local/post-w033-independent-conformance-treecalc-001/`

Independent conformance run:

| Measure | Value |
|---|---:|
| comparison rows | 7 |
| exact value matches | 3 |
| no-publication matches | 2 |
| declared capability gaps | 2 |
| missing artifacts | 0 |
| unexpected mismatches | 0 |
| handoff triggered | false |
| bundle validation | `bundle_valid`, `missing_paths: []` |

Artifact root:

`docs/test-runs/core-engine/independent-conformance/post-w033-independent-conformance-001/`

## 5. Comparison Rows

| Row | State | Surface |
|---|---|---|
| `ic_exact_accept_publish_001` | `matched_exact_value_surface` | accepted publication value delta |
| `ic_exact_multinode_dag_001` | `matched_exact_value_surface` | multi-node DAG value delta |
| `ic_exact_let_lambda_capture_001` | `matched_exact_value_surface` | LET/LAMBDA capture value delta |
| `ic_no_publish_verified_clean_001` | `matched_no_publication_surface` | verified-clean is no-publication |
| `ic_no_publish_reject_001` | `matched_no_publication_surface` | reject is no-publication |
| `ic_gap_dynamic_dependency_001` | `declared_capability_gap` | TreeCalc-local dynamic dependency projection gap |
| `ic_gap_lambda_host_effect_001` | `declared_capability_gap` | TreeCalc-local host-sensitive lambda projection gap |

The two gap rows are not counted as conformance matches. They are retained because they exercise important boundary facts:

1. TraceCalc can represent dynamic dependency switching as an oracle scenario, while the current local TreeCalc fixture path records a dynamic runtime-effect projection and rejects.
2. TraceCalc can represent a LET/LAMBDA runtime-effect witness, while the current local TreeCalc fixture path records a host-sensitive lambda runtime-effect projection and rejects.

## 6. CoreEngine Projection Differential

The run also emits:

`docs/test-runs/core-engine/independent-conformance/post-w033-independent-conformance-001/comparisons/core_engine_projection_differential.json`

This file records, for every mapped TreeCalc case:

1. result state,
2. publication-bundle presence,
3. reject kind when present,
4. dependency descriptors,
5. runtime-effect families,
6. artifact paths for the CoreEngine projection surfaces.

This is the current production/CoreEngine comparison floor for `calc-y0r`. It is stronger than W033's first shared-executor slice because it consumes TreeCalc/CoreEngine fixture artifacts, but weaker than a fully independent evaluator implementation.

## 7. Handoff Decision

Current decision: `no_new_handoff_required`.

Rationale:

1. no exact value row mismatched,
2. no no-publication row mismatched,
3. no artifact required by the bundle validator was missing,
4. the two non-matching semantic areas are already classified as OxCalc local capability gaps, not as OxFml upstream contradictions,
5. `docs/handoffs/HANDOFF_REGISTER.csv` does not need a new row for this bead.

## 8. Semantic-Equivalence Statement

This bead adds additive TreeCalc fixtures, an artifact comparison runner, CLI wiring, documentation, and generated evidence artifacts.

It does not change coordinator scheduling, invalidation strategy, publication semantics, reject policy, OxFml semantics, TraceCalc machine behavior, or TreeCalc evaluation semantics.

Observable behavior for the widened evidence surfaces is invariant under the comparison:

1. TreeCalc fixture execution for the 21-case source run reports 0 expectation mismatches.
2. Independent TraceCalc-to-TreeCalc differential comparison reports 0 unexpected mismatches.
3. Declared capability gaps are explicitly not promoted as conformance matches.

## 9. Verification

Commands run:

| Command | Result |
|---|---|
| `cargo fmt --all` | passed |
| `cargo test -p oxcalc-tracecalc independent_conformance` | passed |
| `cargo test -p oxcalc-core treecalc_fixture::tests::checked_in_treecalc_fixtures_execute_against_local_runtime` | passed |
| `cargo test -p oxcalc-core treecalc_runner_emits_local_run_artifacts` | passed |
| `cargo run -p oxcalc-tracecalc-cli -- treecalc post-w033-independent-conformance-treecalc-001` | passed; 21 TreeCalc cases, 0 expectation mismatches |
| `cargo run -p oxcalc-tracecalc-cli -- independent-conformance post-w033-independent-conformance-001` | passed; 7 comparison rows |
| `cargo test --workspace` | passed: 49 `oxcalc_core` tests, 5 upstream-host tests, 8 `oxcalc_tracecalc` tests, 0 CLI tests, 0 doctest failures |
| `cargo clippy --workspace --all-targets -- -D warnings` | passed |
| `git diff --check` | passed; CRLF normalization warnings only |
| `scripts/check-worksets.ps1` | passed |
| `br dep cycles --json` | passed with `count: 0` |

## 10. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for in-scope items? | yes; this packet records the runner, fixtures, comparison rows, evidence roots, gap treatment, and handoff decision |
| 2 | Pack expectations updated for affected packs? | yes; no pack-grade replay promotion is made, and `calc-lwh` remains successor-scoped |
| 3 | At least one deterministic replay/projection artifact exists per in-scope behavior? | yes; TreeCalc source run and independent conformance run are checked under `docs/test-runs/core-engine/` |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 8 states that no runtime policy or strategy change was made |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no upstream mismatch or handoff trigger was observed |
| 6 | All required tests pass? | yes; see Section 9 |
| 7 | No known semantic gaps remain in declared target? | yes for this widened comparison target; declared local capability gaps remain explicit and unpromoted |
| 8 | Completion language audit passed? | yes; this packet separates widened evidence from pack-grade replay, continuous differential coverage, and fully independent evaluator implementation |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth did not change |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | not applicable; feature-map truth did not change |
| 11 | execution-state blocker surface updated? | yes; `calc-y0r` is represented in `.beads/` |

## 11. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-y0r` asks for independent production/CoreEngine and TreeCalc-to-TraceCalc differential comparison beyond the shared-executor slice |
| Gate criteria re-read | pass; the target is a widened evidence packet, not a fully independent evaluator or pack-grade suite |
| Silent scope reduction check | pass; dynamic dependency and host-sensitive lambda surfaces are retained as declared gap rows rather than omitted |
| "Looks done but is not" pattern check | pass; the runner does not claim fully independent evaluator diversity, continuous differential coverage, or pack-grade replay |
| Result | pass for the `calc-y0r` declared independent conformance widening target |

## 12. Three-Axis Report

- execution_state: `calc-y0r_evidence_packet_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - fully independent evaluator implementation diversity remains open
  - continuous cross-engine differential suite remains open
  - pack-grade replay promotion remains successor-scoped to `calc-lwh`
  - formal model widening remains successor-scoped to `calc-rcr`
  - scale/metamorphic semantic binding remains successor-scoped to `calc-8lg`
  - TreeCalc-local dynamic dependency and host-sensitive lambda projection gaps remain explicit and unpromoted
