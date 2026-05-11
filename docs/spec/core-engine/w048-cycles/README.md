# W048 Cycles Packet Root

Status: `closure_audit_passed`

Parent workset: `docs/worksets/W048_CIRCULAR_DEPENDENCY_CALCULATION_PROCESSING.md`

Parent bead epic: `calc-zci1`

## Purpose

This directory is the W048 packet root for circular dependency calculation processing.

W048 treats cycle behavior as calculation semantics, not as an incidental scheduler artifact. The workset owns the full circular-reference lane:

1. what public Excel documentation establishes and what still requires black-box observation;
2. how OxCalc must materialize dependency graphs explicitly for structural, published-effective, and candidate-effective layers;
3. which deterministic choices can affect a cycle result, including cycle root, iteration order, initial values, terminal policy, and publication boundary;
4. how structural cycles and Calc-Time Rebinding Overlay-created cycles share one policy surface;
5. reference implementation in TraceCalc;
6. optimized/core implementation in TreeCalc;
7. test corpus and conformance execution;
8. W048-owned formal definitions, proof/model artifacts, and checker targets for cycle behavior;
9. opportunities for OxCalc-specific improvements beyond Excel compatibility, explicitly separated from the Excel-match profile.

## Packet Map

1. `W048_CYCLE_LITERATURE_AND_DECISION_MAP.md`
   - public Excel evidence, Incremental/SAC/build-system literature, Foundation research intake, and W048 decision axes.
2. `W048_GRAPH_MATERIALIZATION_AND_CTRO_LAYERS.md`
   - materialized graph model for `G_struct`, `G_eff`, and `G_eff_candidate`, including forward/reverse edges and overlay provenance.
3. `W048_EXCEL_PROBE_CATALOG_AND_OBSERVATION_SCHEMA.md`
   - black-box Excel probe catalog and normalized observation schema.
4. `W048_ENGINE_AND_FORMALIZATION_ROADMAP.md`
   - TraceCalc, TreeCalc, test, formal-artifact, and successor-consumption route.
5. `W048_ITERATIVE_PROFILE_DECISION_AND_EXCEL_DISPOSITION.md`
   - explicit non-default iterative profile decision plus Excel-match disposition.
6. `W048_FORMAL_CYCLE_DEFINITIONS_AND_CHECKER_ARTIFACTS.md`
   - formal definitions, TLA model sketch, and executable checker projection over W048 artifacts.
7. `W048_TEST_CORPUS_AND_CONFORMANCE_PLAN.md`
   - circular-reference fixture corpus, Excel/OxCalc differential observations, and run-artifact expectations.
8. `W048_CORPUS_AND_CONFORMANCE_EVIDENCE.md`
   - cross-corpus evidence summary and conformance checker binding.
9. `W048_INNOVATION_OPPORTUNITY_LEDGER.md`
   - candidate OxCalc innovations for cycle handling, kept profile-gated and evidence-driven.
10. `W048_CLOSURE_AUDIT_AND_SUCCESSOR_ROUTING.md`
   - final W048 audit checklist, evidence summary, and W049 routing.

## Current Conclusions

1. Excel has deterministic machinery, but public documentation shows that determinism is stateful: Excel builds a dependency tree, creates and revises a calculation chain, and can save calculation-chain metadata to the workbook.
2. Excel-compatible cycle behavior therefore cannot be inferred from formula text alone. The observation unit must include workbook history, calculation-chain state, recalc command, calculation mode, iteration settings, and version/platform metadata.
3. Non-iterative OxCalc Stage 1 should keep the current no-publication cycle boundary until Excel observations and explicit policy justify any narrower publication rule.
4. Iterative cycle support needs a declared profile, not hidden scheduler behavior. The profile must name its root/order policy, initial-value policy, update model, convergence metric, max-iteration rule, terminal state, and publication rule.
5. The existing Rust floor already has structural forward edges, reverse edges, and cycle groups. W048 must extend that floor into materialized graph artifacts, executable cycle policies, tests, and formal definitions/models.

## Status Surface

- execution_state: `closure_audit_passed`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- closure_audit: `W048_CLOSURE_AUDIT_AND_SUCCESSOR_ROUTING.md`
- open_lanes: []
