# W048 Test Corpus And Conformance Plan

Status: `active_execution_workset`

## 1. Purpose

W048 owns a circular-reference test corpus, not only a probe list. The corpus must connect Excel observations, TraceCalc reference behavior, TreeCalc optimized/core behavior, graph sidecars, and formal/checker projections.

## 2. Corpus Families

| Family | Purpose | First artifacts |
| --- | --- | --- |
| `excel_cycle_observation` | black-box Excel comparison evidence | `docs/test-runs/excel-cycles/<run_id>/` |
| `tracecalc_cycle_reference` | reference semantics for declared cycle profiles | `docs/test-corpus/core-engine/tracecalc/` fixtures and run roots |
| `treecalc_cycle_core` | optimized/core conformance to reference behavior | `docs/test-fixtures/core-engine/treecalc/` fixtures and run roots |
| `graph_materialization` | sidecar checks for forward/reverse/effective/candidate graphs | graph JSON sidecars plus checker projections |
| `formal_cycle_model` | Lean/TLA/checker examples that bind definitions to artifacts | `formal/lean/`, `formal/tla/`, and checker run roots |
| `innovation_profile` | opt-in non-default cycle handling experiments | profile-specific fixtures and diagnostic notes |

## 3. Required Case Set

Minimum case set:

1. structural direct self-cycle;
2. structural two-node SCC;
3. structural three-node SCC with deterministic member ordering;
4. guarded activation cycle with prior-value retention question;
5. CTRO dynamic self-cycle;
6. CTRO dynamic two-node SCC;
7. CTRO candidate cycle rejected with no overlay commit;
8. CTRO cycle release and re-entry;
9. downstream dependent blocked by cycle and recomputed after release;
10. iterative self-cycle after profile selection;
11. order-sensitive iterative SCC after profile selection;
12. graph materialization reverse-edge converse case;
13. candidate-effective graph cycle introduction case;
14. innovation profile example when a non-default profile is admitted.

## 4. Artifact Requirements

Each non-Excel engine case should emit:

1. fixture input;
2. materialized graph layer artifacts;
3. cycle-region records;
4. evaluation or reject result;
5. published values or no-publication evidence;
6. overlay commit/no-commit evidence where CTRO applies;
7. invalidation closure and downstream state;
8. run result summary;
9. checker or formal projection when available.

Each Excel observation case should emit the schema defined in `W048_EXCEL_PROBE_CATALOG_AND_OBSERVATION_SCHEMA.md`.

## 5. Cross-Engine Comparison

For each declared OxCalc behavior case:

1. TraceCalc is the reference comparison point;
2. TreeCalc must match TraceCalc for observable behavior under the selected profile;
3. Excel observations are mapped to a profile disposition:
   - `matched_by_profile`;
   - `matched_visible_surface_only`;
   - `intentionally_different_with_reason`;
   - `deferred_unobserved`;
   - `out_of_scope_data_table_or_host_specific`.

## 6. Checker Expectations

Checkers should be able to assert:

1. every reverse edge has a forward edge counterpart;
2. every forward edge has a reverse edge counterpart;
3. cycle regions correspond to SCCs or self-loops;
4. rejected candidate cycles publish no new cycle-region values;
5. rejected candidate cycles commit no candidate overlay;
6. release/re-entry invalidates cycle members and downstream dependents;
7. iterative profile traces respect declared root/order and terminal policy.

## 7. Status Surface

- execution_state: `corpus_conformance_slice_passed`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- evidence_packet: `W048_CORPUS_AND_CONFORMANCE_EVIDENCE.md`
- conformance_summary: `docs/test-runs/core-engine/w048-conformance-001/w048_conformance_summary.json`
- open_lanes:
  - iterative profile decisions and fixture promotion: `calc-zci1.4`
  - formal/checker projections beyond current graph checker: `calc-zci1.5`
  - innovation profile examples: `calc-zci1.8`

