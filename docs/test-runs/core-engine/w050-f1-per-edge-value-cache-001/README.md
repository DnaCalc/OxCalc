# W050 F1 Per-Edge Value Cache

Run id: `w050-f1-per-edge-value-cache-001`

Purpose: pin the first OxCalc-local per-edge value cache behavior for
differential evaluation.

Artifact:
- `run_artifact.json` records the cache key fields, retention class, eviction
  policy, and validation cases for hit, miss, volatile exclusion, and
  effectful exclusion.

Validation commands:
- `cargo test -p oxcalc-core per_edge_value_cache -- --nocapture`
- `cargo test -p oxcalc-core`

Result: same `(call_site_id, hole_binding_fingerprint)` lookups hit, changed
hole-binding fingerprints miss, volatile/effectful paths are excluded, and the
cache is bounded by deterministic oldest-first eviction pending W054 retention
policy.
