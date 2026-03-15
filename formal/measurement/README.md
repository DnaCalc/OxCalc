# Stage 1 Measurement Artifacts

This directory contains the first explicit counter schema and experiment register artifacts for the TreeCalc-first Stage 1 implementation wave.

## Current Artifacts
1. `stage1_counter_schema.json`
2. `stage1_experiment_register.json`

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - artifacts define schemas and experiment records only; no runtime instrumentation emits them yet
  - replay-linked summaries, regression thresholds, and scenario-level counter snapshots are still absent
  - Stage 2 reserved counter families remain declarative only
