# W046 Cleanup Manifest

Beads: `calc-gucd.12`, `calc-gucd.13`

Date: 2026-05-07

## Intent
Reset the active W046 surface around the engine semantic proof spine. The cleanup moves historical W038-W045 material into a shallow archive, keeps the useful predecessor context reachable, and reduces the amount of non-semantic status material loaded by default.

## Archived In This Pass
1. `archive/w038-formalization/` through `archive/w045-formalization/`
   - 88 packet docs moved from `docs/spec/core-engine/`.
2. `archive/worksets-w038-w045/`
   - 8 historical workset docs moved from `docs/worksets/`.
3. `archive/lean-w038-w045/`
   - 21 late W038-W045 Lean row-classifier files moved from `formal/lean/OxCalc/CoreEngine/`.
4. `archive/IN_PROGRESS_FEATURE_WORKLIST.pre-w046-cleanup.md`
   - previous detailed worklist moved out of the default context-loading path.
5. `archive/oxcalc-deep-technical-brochure/`
   - superseded showcase draft moved out of `docs/showcase/`.
6. `archive/formal-verification-lab.png`
   - generated showcase image not used by the current HTML deck.
7. `archive/bootstrap-2026-03/` and `archive/rewrite-control-2026-03/`
   - older nested archives moved up from `docs/spec/core-engine/archive/` so archive material has one obvious top-level home.

## Archived In Follow-Up
1. `archive/test-runs-core-engine-w038-w045/`
   - 92 W038-W045 generated run directories moved from `docs/test-runs/core-engine/`.
   - 3,304 tracked generated evidence files moved out of the active test-run root.
   - literal W038-W045 evidence references were rewritten to the archive path; generic future-output runner roots under `docs/test-runs/core-engine/` were left in place.
2. Historical TraceCalc runner path rules
   - W038-W045 predecessor-evidence reads assembled through local artifact-path helpers now resolve to `archive/test-runs-core-engine-w038-w045/`.
   - Test-only roots such as `test-w038-*` still use transient `docs/test-runs/core-engine/` locations and clean themselves up.

## Active Surfaces Kept
1. `docs/worksets/W046_CORE_FORMALIZATION_ENGINE_SEMANTIC_PROOF_SPINE.md`
2. `docs/spec/core-engine/w046-formalization/`
3. `docs/showcase/oxcalc_w033_w045_engine_formalization_review_catalog.md`
4. `docs/showcase/oxcalc_w033_w045_engine_formalization_storyboard.md`
5. `docs/showcase/oxcalc_w033_w045_formalization_showcase.html`
6. W033-W037 formalization specs and current formal/TLA models that still carry useful semantic foundation material.

## Deferred Cleanup
1. `src/oxcalc-tracecalc/` still contains W038-W045 runner branches and classifiers as historical replay tools. The next cleanup slice can delete or compress them after their useful semantics are distilled into W046.
2. W033-W037 specs should be reviewed next for semantic content worth promoting into W046, then further archived if they no longer guide engine behavior.

## Active-Path Rule
The active W046 path should cite this archive only as predecessor context. New W046 proof or implementation work should cite active semantic specs, current tests, current models, or freshly generated replay evidence.
