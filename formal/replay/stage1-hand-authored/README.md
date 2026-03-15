# Stage 1 Hand-Authored Replay Artifacts

This directory holds the first deterministic replay artifacts authored directly from the current Stage 1 OxCalc semantics.

## Purpose
1. seed W009 with real artifacts before the validator, harness, and oracle exist,
2. preserve a stable replay surface for the earliest Stage 1 coordinator and recalc behaviors,
3. provide concrete examples for later emitted harness and oracle artifacts to match or supersede.

## Current Artifacts
1. `r1_candidate_result_vs_publication_001.json`
2. `r2_reject_is_no_publish_001.json`
3. `r7_verify_clean_without_publication_001.json`

These seed artifacts are now paired with the emitted baseline harness and oracle run at:
1. `docs/test-runs/core-engine/tracecalc-reference-machine/w013-sequence-a-baseline/`

The emitted baseline run exercises:
1. `R1` through `tc_accept_publish_001`
2. `R2` through `tc_reject_no_publish_001`
3. `R7` through `tc_verify_clean_no_publish_001`
4. `R4` through `tc_pinned_view_stability_001`
5. `R5` through `tc_overlay_retention_001`

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - the seed artifacts remain hand-authored even though equivalent behaviors are now exercised through emitted harness and oracle artifacts
  - pack binding and generated replay corpus lanes remain open
  - `R3`, `R6`, and `R8` still need authored and exercised artifacts
