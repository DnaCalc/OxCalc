# OxCalc

OxCalc is the multi-node core calculation engine lane for DNA Calc.

## Core Responsibilities
1. Structural dependency graph management and invalidation policy.
2. Calc-time overlay lifecycle (dynamic references, spill overlays, visibility metadata).
3. Coordinator scheduling and publication semantics.
4. Deterministic staged realization (Stage 1 sequential -> Stage 2 partitioned parallel -> Stage 3 advanced lanes).

## Startup Docs
- `CHARTER.md`
- `OPERATIONS.md`
- `docs/spec/README.md`

## Dependency Constitution
- May depend on: `OxFml`, `OxFunc`.
- Must not depend on: host/UI/file-adapter layers.

## Foundation Alignment
Precedence and constitutional constraints are inherited from:
1. `../Foundation/CHARTER.md`
2. `../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md`
3. `../Foundation/OPERATIONS.md`
