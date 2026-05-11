# W048 Closure Audit And Successor Routing

Status: `superseded_by_reopen`

Audit summary: `docs/test-runs/core-engine/w048-closure-audit-001/w048_closure_audit_summary.json`

Superseded by: `docs/spec/core-engine/w048-cycles/W048_REOPEN_SCOPE_AUDIT_AND_REPAIR_PLAN.md`

Reopen note: this audit remains historical evidence for the conservative Stage 1 non-iterative slice only. It no longer closes W048 because the intended W048 target includes comprehensive Excel-compatible circular-reference behavior and bit-exact iterative calculation.

## 1. Objective Restated As Success Criteria

W048 must process the circular dependency calculation workset through all related beads and leave evidence for:

1. Excel circular-reference probes and observations;
2. materialized dependency graph layers and sidecars;
3. TraceCalc reference cycle behavior;
4. TreeCalc optimized/core cycle behavior;
5. iterative-profile algorithm decision and Excel disposition;
6. W048 formal definitions/model/checker artifacts;
7. circular-reference corpus and conformance runs;
8. profile-gated innovation ledger;
9. bead closure and commits for all child beads;
10. final evidence-based audit before claiming W048 closure.

## 2. Prompt-To-Artifact Checklist

| Requirement | Evidence |
| --- | --- |
| Excel probe harness and observation ledger | `scripts/run-w048-excel-cycle-probes.ps1`; `docs/test-runs/excel-cycles/w048-excel-cycles-001/observation.json`; `W048_EXCEL_OBSERVATION_LEDGER.md` |
| materialized graph layers and sidecars | `scripts/check-w048-materialized-graphs.ps1`; `w048-materialized-graph-001`; `W048_MATERIALIZED_GRAPH_SIDECAR_EVIDENCE.md` |
| TraceCalc reference cycle behavior | TraceCalc fixtures under `docs/test-corpus/core-engine/tracecalc/hand-auditable/`; run `w048-tracecalc-cycles-003`; `W048_TRACECALC_REFERENCE_CYCLE_BEHAVIOR.md` |
| TreeCalc optimized/core behavior | W048 TreeCalc fixtures; run `w048-treecalc-cycles-001`; `W048_TREECALC_OPTIMIZED_CYCLE_BEHAVIOR.md` |
| corpus and conformance runs | `scripts/check-w048-conformance.ps1`; `w048_conformance_summary.json`; `W048_CORPUS_AND_CONFORMANCE_EVIDENCE.md` |
| iterative-profile decision | `W048_ITERATIVE_PROFILE_DECISION.json`; `scripts/check-w048-iterative-profile-decision.ps1`; `W048_ITERATIVE_PROFILE_DECISION_AND_EXCEL_DISPOSITION.md` |
| formal definitions/checker artifacts | `formal/tla/CoreEngineW048CycleRegions.tla`; `scripts/check-w048-formal-cycle-artifacts.ps1`; `w048_formal_cycle_checker_summary.json`; `W048_FORMAL_CYCLE_DEFINITIONS_AND_CHECKER_ARTIFACTS.md` |
| innovation opportunity ledger | `W048_INNOVATION_OPPORTUNITY_LEDGER.json`; `scripts/check-w048-innovation-ledger.ps1`; `W048_INNOVATION_OPPORTUNITY_LEDGER.md` |
| child beads closed | `.beads/issues.jsonl` entries for `calc-zci1.1` through `calc-zci1.8` |
| final audit | `scripts/check-w048-closure-audit.ps1`; `w048_closure_audit_summary.json` |

## 3. Concrete Audit Results From Superseded Closure

The predecessor closure audit checker reported:

- status: `passed`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes: `[]`

These values are not current W048 status after reopen. They describe the predecessor conservative Stage 1 slice only.

Key numeric evidence:

| Surface | Result |
| --- | ---: |
| Excel observations | 12 |
| TraceCalc scenarios | 34 passed |
| TreeCalc cases | 33 |
| TreeCalc expectation mismatches | 0 |
| W048 graph layers | 99 |
| W048 cycle-region records | 12 |
| graph checker errors | 0 |
| conformance checker errors | 0 |
| formal cycle checker errors | 0 |

## 4. Completion Claim Self-Audit

1. No child bead remains open: verified by `scripts/check-w048-closure-audit.ps1` and `br ready --format json` before parent closure.
2. Spec text is backed by executable evidence: each major W048 packet has either run artifacts or a checker summary.
3. Iterative Excel compatibility was not overclaimed in the predecessor packet, but the deferral itself is now the reason W048 is reopened: `cycle.excel_match_iterative` must become implemented/validated for declared coverage or exactly blocked with user acceptance.
4. Innovation profiles are not default behavior: checker requires non-default/admission-gated ledger entries.
5. Semantic-equivalence under strategy change is named for the future deterministic iterative profile and release/local frontier repair obligations.
6. Cross-repo handoff was not needed: no OxFml evaluator-facing seam change was introduced.

## 5. Superseded Successor Routing

The following routing is superseded for core circular-reference semantics. These items are W048 open lanes unless the user explicitly accepts exact blockers:

1. full mechanized proof of iterative-profile determinism;
2. broader Excel-match iterative probes;
3. admitted experimental profile implementation;
4. dynamic-array/spill/data-table cycle families.

Python checker paths named by this historical audit are also superseded for W048 closure. Replacement validation must use PowerShell, Rust, or C#.
