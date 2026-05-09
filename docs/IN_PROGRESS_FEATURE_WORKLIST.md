# OxCalc In-Progress Feature Worklist

This file is a compact orientation surface. It is not live execution state; `.beads/` owns that. Older detailed W033-W045 status material was archived during the W046 cleanup prelude at `archive/IN_PROGRESS_FEATURE_WORKLIST.pre-w046-cleanup.md`.

## Current Focus
1. `W046_CORE_FORMALIZATION_ENGINE_SEMANTIC_PROOF_SPINE.md`
2. Epic: `calc-gucd`
3. Cleanup prelude: `calc-gucd.12`
4. Generated evidence isolation follow-up: `calc-gucd.13`
5. Replay-tooling source cleanup follow-up: `calc-gucd.14`
6. Redirect and semantic catalog bead: `calc-gucd.1`
7. Current semantic bead: `calc-gucd.4`

## Active Aim
W046 redirects formalization toward the calculation engine's semantic proof spine:
1. dependency graph build,
2. reverse/forward edge consistency,
3. SCC and cycle classification,
4. invalidation seed and closure semantics,
5. soft-reference and dynamic-reference rebind,
6. recalc tracker transitions,
7. evaluation order and working-value read discipline,
8. TraceCalc refinement for selected formulas,
9. OxFml seam behavior where `LET`, `LAMBDA`, and dynamic references thread through the core engine.

## Active Truth Surfaces
1. `docs/worksets/W046_CORE_FORMALIZATION_ENGINE_SEMANTIC_PROOF_SPINE.md`
2. `docs/spec/core-engine/w046-formalization/`
3. `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`
4. `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`
5. `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
6. `formal/tla/` for current engine-state models
7. `formal/lean/` for active proof/model files
8. `src/oxcalc-core/` and `src/oxcalc-tracecalc/` for implementation and reference machinery

## Archived Predecessor Material
W038-W045 packet docs, workset docs, late row-classifier Lean files, generated evidence roots, and superseded showcase material now live under top-level `archive/`.

The archive is retained as historical evidence and predecessor context. It is not the active direction for W046.

## Open Cleanup Lanes
1. Distill any useful W033-W045 semantic fragments into W046 specs before further archiving.
