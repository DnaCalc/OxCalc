# W033 LET/LAMBDA Carrier Witness Widening

Status: `calc-688_evidence_packet`
Workset: `W033`
Successor bead: `calc-688`
Created: 2026-05-04

## 1. Purpose

This packet records the first post-W033 exercised witness widening for the narrow `LET`/`LAMBDA` OxFml/OxFunc boundary fragment.

The target is not general OxFunc function semantics. The target is OxCalc-visible carrier behavior:

1. helper-lambda origin,
2. lexical capture,
3. arity and invocation contract,
4. dependency visibility,
5. runtime-effect visibility,
6. replay identity and witness anchors.

## 2. Runtime Surface

TreeCalc now has a narrow fixture-facing `TreeFormula::RawOxfml` source carriage for helper-carrier witnesses whose syntax cannot yet be represented by the structured `TreeFormula` subset. It carries:

1. raw OxFml source text,
2. explicit `reference_carriers` used by OxCalc to preserve dependency and runtime-effect visibility.

This is a witness-carriage surface, not a promotion of arbitrary raw formula text as the normal TreeCalc substrate.

## 3. TraceCalc Witnesses

New TraceCalc scenarios:

| Scenario | Coverage |
|---|---|
| `tc_let_lambda_carrier_publish_001` | helper-lambda origin, lexical capture, arity, invocation contract, dependency visibility, replay identity, publication |
| `tc_let_lambda_invocation_reject_001` | arity mismatch, invocation contract reject, reject/no-publish replay identity |
| `tc_let_lambda_runtime_effect_001` | runtime-effect visibility through callable invocation and replay identity |

Run:

```powershell
cargo run -p oxcalc-tracecalc-cli -- post-w033-let-lambda-carrier-witness-001
```

Evidence:

1. `docs/test-runs/core-engine/tracecalc-reference-machine/post-w033-let-lambda-carrier-witness-001/run_summary.json`
2. `docs/test-runs/core-engine/tracecalc-reference-machine/post-w033-let-lambda-carrier-witness-001/scenarios/tc_let_lambda_carrier_publish_001/result.json`
3. `docs/test-runs/core-engine/tracecalc-reference-machine/post-w033-let-lambda-carrier-witness-001/scenarios/tc_let_lambda_invocation_reject_001/result.json`
4. `docs/test-runs/core-engine/tracecalc-reference-machine/post-w033-let-lambda-carrier-witness-001/scenarios/tc_let_lambda_runtime_effect_001/result.json`
5. `docs/test-runs/core-engine/tracecalc-reference-machine/post-w033-let-lambda-carrier-witness-001/replay-appliance/validation/bundle_validation.json`

Summary:

| Measure | Value |
|---|---:|
| scenarios | 15 |
| passed scenarios | 15 |
| LET/LAMBDA scenarios | 3 |
| validation failures in LET/LAMBDA scenarios | 0 |
| conformance mismatches in LET/LAMBDA scenarios | 0 |
| bundle validation | `bundle_valid`, `missing_paths: []` |

## 4. TreeCalc Witnesses

New TreeCalc fixtures:

| Fixture | Coverage |
|---|---|
| `tc_local_let_lambda_capture_publish_001` | raw OxFml `LET`/`LAMBDA` invocation, lexical capture, direct dependency visibility, publication |
| `tc_local_lambda_host_sensitive_reject_001` | callable invocation with host-sensitive argument, runtime-effect overlay, local reject/no-publish |

Run:

```powershell
cargo run -p oxcalc-tracecalc-cli -- treecalc post-w033-let-lambda-treecalc-witness-001
```

Evidence:

1. `docs/test-runs/core-engine/treecalc-local/post-w033-let-lambda-treecalc-witness-001/run_summary.json`
2. `docs/test-runs/core-engine/treecalc-local/post-w033-let-lambda-treecalc-witness-001/cases/tc_local_let_lambda_capture_publish_001/result.json`
3. `docs/test-runs/core-engine/treecalc-local/post-w033-let-lambda-treecalc-witness-001/cases/tc_local_let_lambda_capture_publish_001/dependency_graph.json`
4. `docs/test-runs/core-engine/treecalc-local/post-w033-let-lambda-treecalc-witness-001/cases/tc_local_lambda_host_sensitive_reject_001/result.json`
5. `docs/test-runs/core-engine/treecalc-local/post-w033-let-lambda-treecalc-witness-001/cases/tc_local_lambda_host_sensitive_reject_001/runtime_effects.json`
6. `docs/test-runs/core-engine/treecalc-local/post-w033-let-lambda-treecalc-witness-001/cases/tc_local_lambda_host_sensitive_reject_001/runtime_effect_overlays.json`
7. `docs/test-runs/core-engine/treecalc-local/post-w033-let-lambda-treecalc-witness-001/replay-appliance/validation/bundle_validation.json`

Summary:

| Measure | Value |
|---|---:|
| cases | 19 |
| published | 10 |
| rejected | 8 |
| verified clean | 1 |
| expectation mismatches | 0 |
| LET/LAMBDA TreeCalc fixtures | 2 |
| bundle validation | `bundle_valid`, `missing_paths: []` |

## 5. Handoff Decision

Current decision: `no_new_handoff_required`.

Rationale:

1. OxFml helper-carrier facts were sufficient for the exercised TraceCalc and TreeCalc witness floor.
2. TreeCalc raw carrier witnesses preserved explicit dependency and runtime-effect visibility without inventing OxFunc kernel semantics.
3. The callable host-sensitive fixture rejected conservatively with replay-visible runtime effect and overlay sidecars.
4. No exercised result exposed a contradiction in OxFml-owned carrier facts.
5. `docs/handoffs/HANDOFF_REGISTER.csv` does not need a new row for this bead.

## 6. Limits And Successor Carry

This bead does not claim:

1. general OxFunc function-kernel coverage,
2. full callable publication policy closure,
3. pack-grade replay,
4. all higher-order callable families,
5. final structured TreeFormula syntax for every Excel helper construct.

Successor pressure remains:

1. `calc-y0r`: independent conformance beyond shared TraceCalc/TreeCalc evidence,
2. `calc-lwh`: pack-grade replay/capability promotion,
3. `calc-rcr`: Lean/TLA widening over callable carriers,
4. `calc-8lg`: metamorphic and scale semantic binding.

## 7. Semantic-Equivalence Statement

This bead widens fixture and witness coverage and adds a narrow TreeCalc fixture source-carriage path. It does not change coordinator scheduling, publication policy, recalc invalidation policy, or OxFml/OxFunc semantics.

For existing non-LET/LAMBDA TraceCalc and TreeCalc fixture families, observable behavior remains covered by the same manifests and full workspace tests. The new raw-carrier path is exercised only by checked-in LET/LAMBDA TreeCalc fixtures and remains explicit in emitted input artifacts.

## 8. Verification

Commands run:

| Command | Result |
|---|---|
| `cargo test -p oxcalc-core treecalc_fixture::tests::checked_in_treecalc_fixtures_execute_against_local_runtime` | passed |
| `cargo test -p oxcalc-tracecalc runner::tests::execute_manifest_produces_passing_conformance_artifacts_for_seed_corpus` | passed |
| `cargo run -p oxcalc-tracecalc-cli -- post-w033-let-lambda-carrier-witness-001` | passed; 15 TraceCalc scenarios |
| `cargo run -p oxcalc-tracecalc-cli -- treecalc post-w033-let-lambda-treecalc-witness-001` | passed; 19 TreeCalc cases |
| `cargo fmt --all -- --check` | passed |
| `cargo test --workspace` | passed: 49 `oxcalc_core` tests, 5 upstream-host tests, 7 `oxcalc_tracecalc` tests, 0 CLI tests, 0 doctest failures |
| `cargo clippy --workspace --all-targets -- -D warnings` | passed |

## 9. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for in-scope items? | yes; this packet records runtime surface, witness cases, run evidence, limits, and handoff decision |
| 2 | Pack expectations updated for affected packs? | yes; new TraceCalc scenarios name callable pack bindings without promoting pack-grade replay |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; TraceCalc and TreeCalc run artifacts exist for carrier publication, invocation reject, dependency visibility, and runtime-effect visibility |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 7 states no coordinator/recalc/publication policy change |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no concrete OxFml mismatch appeared, and no new handoff row is required |
| 6 | All required tests pass? | yes; see Section 8 |
| 7 | No known semantic gaps remain in declared target? | yes; remaining broader callable/pack/formal lanes are explicit successors |
| 8 | Completion language audit passed? | yes; this packet separates exercised carrier witness coverage from general OxFunc and pack-grade claims |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth did not change |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | not applicable; feature-map truth did not change |
| 11 | execution-state blocker surface updated? | yes; `calc-688` is represented in `.beads/` and successor lanes remain ordinary bead graph entries |

## 10. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-688` asked for TraceCalc/TreeCalc LET/LAMBDA carrier witnesses covering carrier origin, capture, arity, invocation contract, dependency/runtime-effect visibility, and replay identity |
| Gate criteria re-read | pass; scenarios, fixtures, emitted runs, replay bundle validation, and docs now cover the declared target |
| Silent scope reduction check | pass; broader callable publication, pack-grade replay, and formal widening are not hidden and remain successor-scoped |
| "Looks done but is not" pattern check | pass; raw source carriage is reported as a witness surface, not a general TreeFormula substrate closure |
| Result | pass for the `calc-688` declared witness-widening target |

## 11. Three-Axis Report

- execution_state: `calc-688_evidence_packet_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - independent conformance widening remains successor-scoped
  - pack-grade replay remains unpromoted
  - formal callable-carrier model widening remains successor-scoped
  - metamorphic and scale semantic binding remains successor-scoped
