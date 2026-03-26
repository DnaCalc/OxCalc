# Upstream Host Fixture Corpus

This directory contains the checked-in deterministic fixture corpus for OxCalc's minimal OxFml upstream-host packet.

Current purpose:
1. exercise the public `MinimalUpstreamHostPacket` scaffolding surface through data-driven cases,
2. keep host-info, RTD, runtime-catalog, capture-packet, and bind-context expectations explicit,
3. provide a small reusable corpus for later OxCalc/OxFml host-seam widening.

Current corpus:
1. `uh_info_directory_capture_001`
2. `uh_info_unsupported_query_001`
3. `uh_rtd_provider_error_001`
4. `uh_sum_defined_name_bind_001`
5. `uh_table_context_bind_001`
6. `uh_structured_reference_eval_001`

Current executable coverage:
1. loader and execution support in `src/oxcalc-core/src/upstream_host_fixture.rs`
2. crate-local fixture validation in `src/oxcalc-core/src/upstream_host_fixture.rs`
3. public integration use in `src/oxcalc-core/tests/upstream_host_scaffolding.rs`

Status:
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - broader W026 bind/reference intake remains outside this first corpus
  - first table-context carriage and one bounded evaluator-facing structured-reference family are now fixture-covered, but richer structured-reference evaluator families are not yet covered here
  - broader execution-restriction and publication/topology breadth remain later seam lanes
