# W048 Materialized Graph Sidecar Evidence

Status: `active_execution_evidence`

## 1. Purpose

This packet records the first W048 materialized graph sidecar widening for TreeCalc local artifacts. It binds the W048 graph-layer contract to an executable checker/normalizer and a deterministic run root.

## 2. Executable Surface

| Surface | Path |
| --- | --- |
| checker/normalizer | `scripts/check-w048-materialized-graphs.ps1` |
| TreeCalc run root | `docs/test-runs/core-engine/treecalc-local/w048-materialized-graph-001/` |
| W048 summary | `docs/test-runs/core-engine/treecalc-local/w048-materialized-graph-001/w048_materialized_graph_check_summary.json` |
| per-case sidecars | `docs/test-runs/core-engine/treecalc-local/w048-materialized-graph-001/cases/*/w048_materialized_graph_layers.json` |

Commands used:

```powershell
cargo run -p oxcalc-tracecalc-cli -- treecalc w048-materialized-graph-001
scripts/check-w048-materialized-graphs.ps1 docs/test-runs/core-engine/treecalc-local/w048-materialized-graph-001
```

Review commands used before bead closure:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-materialized-graphs.ps1 docs/test-runs/core-engine/treecalc-local/w048-materialized-graph-001
powershell -NoProfile -ExecutionPolicy Bypass -Command "$root='docs/test-runs/core-engine/treecalc-local/w048-materialized-graph-001'; $s = Get-Content (Join-Path $root 'w048_materialized_graph_check_summary.json') -Raw | ConvertFrom-Json; if ($s.case_count -ne 29 -or $s.layer_count -ne 87 -or $s.check_error_count -ne 0) { throw 'w048 materialized sidecar summary mismatch' }; foreach ($case in $s.case_summaries) { $sidecar = Get-Content $case.materialized_graph_layers_path -Raw | ConvertFrom-Json; if (@($sidecar.layers).Count -ne 3) { throw 'expected three graph layers' }; foreach ($layer in $sidecar.layers) { if (($layer.graph_hash -as [string]) -notmatch '^sha256:') { throw 'missing graph hash' } } }"
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-worksets.ps1
```

## 3. Evidence Summary

From `w048_materialized_graph_check_summary.json`:

| Field | Value |
| --- | ---: |
| cases checked | 29 |
| materialized graph layers | 87 |
| reverse edge records | 111 |
| cycle region records | 0 |
| check errors | 0 |

The checker emits three graph-layer records for each TreeCalc local case:

1. `structural`;
2. `published_effective`;
3. `candidate_effective`.

Each layer carries:

1. graph-layer and basis metadata;
2. node projections;
3. normalized forward edges;
4. materialized reverse edges;
5. edge provenance and stability class;
6. overlay delta records where candidate dependency-shape updates exist;
7. cycle-region schema slots;
8. topological order or blocked reason;
9. stable `sha256:` graph hash.

## 4. Invariants Exercised

The checker validates:

1. every normalized forward edge has a matching reverse edge tuple;
2. every reverse edge has a matching forward edge tuple;
3. graph hashes are present and stable under canonical JSON encoding;
4. cyclic layers do not also claim a topological order;
5. acyclic layers do not claim a blocked reason.

## 5. Current Limits And Follow-Up Binding

This bead establishes the materialized graph sidecar surface and reverse-edge replay visibility. It does not yet provide non-empty cycle-region evidence because the current checked TreeCalc corpus has no circular-reference fixture. That is deliberately routed to the W048 implementation and corpus beads:

1. `calc-zci1.3` / `calc-zci1.6`: add TraceCalc and TreeCalc structural/CTRO cycle behavior and fixtures;
2. `calc-zci1.7`: rerun this checker over the circular-reference corpus and require non-empty cycle-region sidecars for cycle cases;
3. `calc-zci1.5`: consume these sidecars for reverse-edge converse and graph-layer formal/checker targets.

Until those beads land, W048 can rely on this packet for graph-layer sidecar shape, reverse-edge materialization, provenance fields, overlay-delta projection, and hash/checker plumbing, not for cycle-behavior closure.
