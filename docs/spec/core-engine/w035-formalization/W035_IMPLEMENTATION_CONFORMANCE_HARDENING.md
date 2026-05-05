# W035 Implementation Conformance Hardening

Status: `calc-tkq.3_implementation_conformance_hardening_validated`
Workset: `W035`
Parent epic: `calc-tkq`
Bead: `calc-tkq.3`

## 1. Purpose

This packet hardens the W034 implementation-conformance gap surface without converting proxy evidence into false matches.

The bead target is to take the six W034 declared local gaps and classify each as a match, implementation-work deferral, or spec-evolution deferral with authority. The result keeps the spec evolutionary: gaps can become later implementation work, proof assumptions, seam clarifications, or explicit out-of-scope rows as the domain model deepens.

## 2. Authority Inputs Reviewed

| Input | Role in this bead |
|---|---|
| `docs/spec/core-engine/w035-formalization/W035_RESIDUAL_PROOF_OBLIGATION_AND_SPEC_EVOLUTION_LEDGER.md` | W035 obligation map for `W035-OBL-004`, `W035-OBL-005`, and `W035-OBL-006` |
| `docs/spec/core-engine/w035-formalization/W035_TRACECALC_ORACLE_MATRIX_EXPANSION.md` | W035 oracle-matrix rows used as backing evidence |
| `docs/spec/core-engine/w034-formalization/W034_INDEPENDENT_CONFORMANCE_WIDENING.md` | predecessor declared-gap surface |
| `docs/test-runs/core-engine/independent-conformance/w034-independent-conformance-001/` | source independent-conformance run |
| `docs/test-runs/core-engine/treecalc-local/w034-independent-conformance-treecalc-001/` | TreeCalc-local source run |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/` | W035 TraceCalc matrix evidence |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | current inbound OxFml observation ledger |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | W073 formatting watch input |

## 3. Implementation Surface

| Surface | Change |
|---|---|
| `src/oxcalc-tracecalc/src/implementation_conformance.rs` | adds `ImplementationConformanceRunner` and deterministic W035 gap-disposition artifact emission |
| `src/oxcalc-tracecalc-cli/src/main.rs` | adds `implementation-conformance <run-id>` command |
| `docs/test-runs/core-engine/implementation-conformance/w035-implementation-conformance-hardening-001/` | checked-in conformance-hardening evidence root |

## 4. Evidence Summary

Run id: `w035-implementation-conformance-hardening-001`

| Metric | Value |
|---|---:|
| Source W034 comparison rows | 15 |
| Source W034 declared gaps | 6 |
| Source W034 missing artifacts | 0 |
| Source W034 unexpected mismatches | 0 |
| TreeCalc-local cases | 23 |
| TreeCalc-local expectation mismatches | 0 |
| W035 TraceCalc matrix rows | 17 |
| W035 TraceCalc covered rows | 15 |
| W035 TraceCalc failed/missing rows | 0 |
| W035 gap disposition rows | 6 |
| Implementation-work deferrals | 5 |
| Spec-evolution deferrals | 1 |
| Failed disposition rows | 0 |
| Validation status | `implementation_conformance_hardening_valid` |

Primary artifacts:

1. `docs/test-runs/core-engine/implementation-conformance/w035-implementation-conformance-hardening-001/run_summary.json`
2. `docs/test-runs/core-engine/implementation-conformance/w035-implementation-conformance-hardening-001/gap_disposition_register.json`
3. `docs/test-runs/core-engine/implementation-conformance/w035-implementation-conformance-hardening-001/evidence_summary.json`
4. `docs/test-runs/core-engine/implementation-conformance/w035-implementation-conformance-hardening-001/validation.json`

## 5. Gap Disposition

| W034 row | Disposition | Authority owner | Backing W035 evidence |
|---|---|---|---|
| `ic_gap_dynamic_dependency_001` | dynamic bind projection remains implementation-work deferral | `calc-tkq.3`, carried by `calc-tkq.8` | `w035_dependency_dynamic_switch_publish` |
| `ic_gap_lambda_host_effect_001` | host-sensitive lambda effect is a spec-evolution deferral into callable seam mapping | `calc-tkq.4` | `w035_callable_full_oxfunc_semantics` classified out of W035 OxCalc/OxFml carrier scope |
| `ic_gap_w034_dynamic_dependency_negative_001` | dynamic negative shape-update projection remains implementation-work deferral | `calc-tkq.3`, carried by `calc-tkq.8` | `w035_dependency_dynamic_negative`, `w035_dependency_dynamic_release_publish` |
| `ic_gap_w034_snapshot_fence_projection_001` | snapshot-fence projection remains coordinator implementation/harness deferral | `calc-tkq.5` | `w035_stale_snapshot_fence_reject` |
| `ic_gap_w034_capability_view_fence_projection_001` | capability-view fence projection remains coordinator implementation/harness deferral | `calc-tkq.5` | `w035_stale_capability_view_fence_reject` |
| `ic_gap_w034_higher_order_callable_metadata_001` | callable metadata projection remains implementation-work deferral | `calc-tkq.4` | `w035_callable_higher_order_publish` |

No row is promoted as a conformance match. The W035 runner validates that each source gap remains failure-free, artifact-present, and backed by either covered W035 oracle evidence or an explicit W035 out-of-scope classification.

## 6. OxFml Formatting Watch

The OxFml W073 formatting update remains watch/input-contract evidence only. This bead does not construct conditional-formatting payloads and does not exercise `VerificationConditionalFormattingRule`.

No OxFml handoff is filed by this bead.

## 7. Semantic-Equivalence Statement

This bead adds a conformance-hardening artifact runner, a CLI command, and generated disposition artifacts. It does not change coordinator scheduling, dirty marking, dependency graph construction, soft-reference resolution, recalc semantics, publication fences, reject policy, overlay lifecycle semantics, TreeCalc evaluation semantics, TraceCalc semantics, Lean/TLA artifacts, pack-decision logic, or OxFml evaluator behavior.

Observable core-engine runtime behavior is invariant under this bead. Existing W034 gaps remain explicitly non-matching until later implementation or spec work changes their exercised surface.

## 8. Verification

| Command | Result |
|---|---|
| `cargo test -p oxcalc-tracecalc implementation_conformance` | passed; 1 test |
| `cargo run -p oxcalc-tracecalc-cli -- implementation-conformance w035-implementation-conformance-hardening-001` | passed; emitted 6 gap dispositions, 5 implementation-work deferrals, 1 spec-evolution deferral, 0 failed rows |
| `cargo test -p oxcalc-tracecalc` | passed; 14 tests |
| `cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `cargo fmt --all -- --check` | passed |
| `scripts/check-worksets.ps1` | passed; `worksets=13`, `beads total=79`, `open=6`, `in_progress=1`, `ready=0`, `blocked=5`, `closed=72` |
| `br dep cycles --json` | passed; `count=0` |
| `git diff --check` | passed; CRLF normalization warnings only |

## 9. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet and W035 status surfaces record the six-row disposition |
| 2 | Pack expectations updated for affected packs? | yes; no pack promotion is claimed and `calc-tkq.7` still owns pack/Stage 2 reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; W034 conformance artifacts and W035 implementation-conformance artifacts are checked in |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 7 states no runtime policy or strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; W073 formatting remains watch/input-contract evidence and no concrete handoff trigger exists |
| 6 | All required tests pass? | yes; see Section 8 and the bead closure note |
| 7 | No known semantic gaps remain in declared scope? | yes for this target; all six W034 gap rows have explicit disposition |
| 8 | Completion language audit passed? | yes; no full implementation conformance or independent evaluator promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 points at this conformance-hardening evidence |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-tkq.3` execution state and later closure evidence |

## 10. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-tkq.3` asks for W034 declared gaps to become matches, implementation work, or authoritative deferrals |
| Gate criteria re-read | pass; each of the six W034 declared gaps has a generated W035 disposition row with validation |
| Silent scope reduction check | pass; zero gaps are hidden or counted as matches without evidence |
| "Looks done but is not" pattern check | pass; deferrals are explicit and no proxy row is promoted as implementation conformance |
| Result | pass for the `calc-tkq.3` conformance-hardening target |

## 11. Three-Axis Report

- execution_state: `calc-tkq.3_implementation_conformance_hardening_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-tkq.4` through `calc-tkq.8` remain open
  - full formalization, full Lean/TLA verification, full TraceCalc oracle coverage, full optimized/core-engine conformance, fully independent evaluator diversity, pack-grade replay, continuous scale assurance, and Stage 2 policy remain partial
