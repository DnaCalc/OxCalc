# TreeCalc Local Runs

This root is for local sequential TreeCalc run artifacts emitted from the checked-in fixture
corpus under `docs/test-fixtures/core-engine/treecalc/`.

Current role:
- deterministic local run artifacts for the first seam-backed TreeCalc local floor
- first local oracle and conformance sidecars against fixture-declared expectations
- first local trace and explain sidecars against the emitted local runtime artifacts
- first local run-comparison discipline through `scripts/compare-treecalc-local-run.ps1`
- precursor to the first sequential TreeCalc baseline lane in `W030`

Current emitted roots:
- `w025-treecalc-local-baseline` — carried local baseline from the W025/W029 widening floor
- `w030-treecalc-oracle-baseline` — first broader W030 TreeCalc oracle baseline over the 17-case runtime-derived corpus

Current local corpus floor:
- 17 cases
- direct publish
- verified-clean
- ancestor-relative dependency ordering
- sibling-offset dependency ordering
- host-sensitive rejection with emitted runtime effects
- capability-sensitive rejection with emitted runtime effects
- shape/topology-sensitive rejection with emitted runtime effects
- dynamic-reference rejection with emitted runtime effects
- rename-triggered rebind consequence with post-edit rerun artifacts
- recalc-only constant edit consequence with post-edit rerun artifacts
- recalc-only dependency-chain consequence with deterministic downstream rerun order
- recalc-only post-edit host-sensitive runtime-effect and overlay consequence
- recalc-only post-edit capability-sensitive runtime-effect and overlay consequence
- recalc-only post-edit shape/topology-sensitive runtime-effect and overlay consequence
- mixed publication-then-post-edit overlay consequence with preserved published view and successor-snapshot runtime effects
- move-triggered rebind consequence with preserved publication on the successor snapshot
- removal consequence with typed post-edit rejection on missing direct dependency

Current limits:
- the current lane is seam-backed for the first direct-host slice, but still has W029/W030 successor breadth beyond the currently exercised dynamic-dependency, execution-restriction, capability-sensitive, and shape/topology floors
- oracle and conformance are local fixture-expectation floors only, not the later live TreeCalc oracle lane

Current compare commands:
- `pwsh ./scripts/compare-treecalc-local-run.ps1 -CandidateRunId w030-treecalc-oracle-baseline`
- `pwsh ./scripts/compare-treecalc-local-run.ps1 -CandidateRunId w030-treecalc-oracle-baseline -BaselineRunId w030-treecalc-oracle-baseline`
