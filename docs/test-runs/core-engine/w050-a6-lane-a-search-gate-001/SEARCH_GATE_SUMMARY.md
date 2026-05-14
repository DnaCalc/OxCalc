# W050 A6 Lane A Search Gate Summary

Date: 2026-05-14
Bead: `calc-cwpl.8.2`
Evidence root: `docs/test-runs/core-engine/w050-a6-lane-a-search-gate-001`

## Scope

This search gate covers production Rust, Rust test/scaffolding modules,
TreeCalc and upstream-host fixture corpora, active showcase/slide docs, and
active W050/status docs. Generated test-run archives and retired workset/spec
surfaces are classified as archive/provenance hits when encountered.

## No-Hit Proofs

Production and Rust test search:

```powershell
rg -n "\btranslate_formula\b|\bTranslationState\b|evaluate_via_oxfml|build_upstream_host_packet|formula_allows_lazy_residual_publication|RuntimeEnvironment::new\(\)\.execute|TreeFormula::(Literal|Binary|FunctionCall|Reference)" src/oxcalc-core/src src/oxcalc-core/tests -g "*.rs"
```

Result: exit code 1, no output. The retired production names and
`TreeFormula::{Literal,Binary,FunctionCall,Reference}` semantic constructors
are absent from the searched Rust surfaces.

Active showcase/slide stale-wording search:

```powershell
rg -n "evaluate_via_oxfml|build_upstream_host_packet|formula_allows_lazy_residual_publication|TreeFormulaCatalog</code> carries formulas|translates TreeCalc formulas|Evaluate locally|builds a minimal upstream host packet|locally computes dynamic arrays|array arithmetic" docs/slides/oxcalc_xyz_call_trace.html docs/showcase/oxcalc_w047_w048_core_engine_showcase.html docs/showcase/oxcalc_w033_w045_formalization_showcase.html docs/showcase/oxcalc_w033_w045_engine_formalization_storyboard.md docs/showcase/oxcalc_w033_w045_engine_formalization_review_catalog.md
```

Result: exit code 1, no output. The active showcase and slide surfaces no
longer present the retired packet path or local formula-semantics wording.

## Remaining Allowed References

| Surface | Search result | Classification | Open lane |
| --- | --- | --- | --- |
| `src/oxcalc-core/src/formula.rs` | `TreeFormula`, `TreeFormulaReferenceCarrier`, `TreeFormulaCatalog`, `TreeReference`, `FixtureFormulaAst`, and `FixtureFormulaBinaryOp` remain. Constructor-count proof over searched Rust fixture/quarantine code: `FixtureFormulaAst::Binary` 15, `FunctionCall` 12, `Literal` 11, `Reference` 57, `FixtureFormulaBinaryOp::` 17. | `TreeFormula` is opaque OxFml source plus explicit carriers. `TreeReference` is dependency/evaluator-fact projection. `FixtureFormulaAst`/`FixtureFormulaBinaryOp` are quarantine scaffolding for fixtures/tests/scale profiles. | Fixture corpus conversion/deletion after canonical OxFml source transport is available. |
| `src/oxcalc-core/src/treecalc.rs` | `synthetic_cell_row` at lines 1706, 2244, 2610, 2615; `synthetic_cell_target` at lines 2230, 2237, 2614. Count: `synthetic_cell_row` 4, `synthetic_cell_target` 3. | Current V1 reference/input compatibility residue used by the session path, not the retired packet path. | CALC-002/H1 canonical reference/input transport, then helper deletion. |
| `src/oxcalc-core/src/upstream_host.rs`, `src/oxcalc-core/src/upstream_host_fixture.rs`, `src/oxcalc-core/tests/upstream_host_scaffolding.rs`, `docs/test-fixtures/core-engine/upstream-host/README.md` | Minimal-family count proof: `MinimalUpstreamHostPacket` 12, `MinimalFormulaSlotFacts` 7, `MinimalBindingWorld` 4, `MinimalTypedQueryFacts` 8, `MinimalRuntimeCatalogFacts` 8. | Upstream-host fixture/scaffolding packet surface. It is no longer the TreeCalc production invocation path after B7. | Keep fixture-only until the session API covers the same intake evidence, then delete or quarantine. |
| `src/oxcalc-core/src/consumer.rs` and `src/oxcalc-core/src/treecalc.rs` test modules | Fixture constructors appear under `#[cfg(test)]`. | Unit-test scaffolding; not production formula semantics. | Convert/delete only when fixture corpus migration provides equivalent evidence. |
| `src/oxcalc-core/src/treecalc_scale.rs` | Procedural `FixtureFormulaAst` builders remain for scale/demo profiles. | Scale/demo scaffolding; not production formula semantics. | Convert/delete with fixture corpus migration. |
| `docs/test-fixtures/core-engine/treecalc/cases/*.json` | Fixture expression-key counts: `"Binary":` 29, `"FunctionCall":` 6, `"Literal":` 25, `"Reference":` 67, `"RawOxfml":` 3. Representative policy tags: two `fixture-policy:opaque-oxfml-source`, one `fixture-policy:legacy-structured-quarantine`. | Checked-in fixture corpus still has legacy structured quarantine plus three RawOxfml cases. | Broader fixture conversion/deletion remains open after canonical CALC-002 transport. |
| `docs/IN_PROGRESS_FEATURE_WORKLIST.md` | Two hits: retired pre-W050 `evaluate_via_oxfml` bridge audit line, and a negative instruction to remove/quarantine docs that imply local dynamic-array or function semantics. | Active status wording only; no active product claim. | Keep status wording aligned as W050 lands. |
| W050/status/spec/handoff docs | Count proof over active docs found hits concentrated in W050/status, CALC-002 handoff, OxFml seam docs, upstream notes, and historical W026/W033/W034/W037/W046/W047 spec references. | Status, handoff, current seam inventory, or historical/provenance references. | `calc-cwpl.16.4` and later closure audit must preserve the active/historical distinction. |

## Current Lane A Reading

The A6 search gate proves that the retired production names are absent from
the Rust production/test search scope and that active showcase wording no
longer makes the targeted local formula-semantics claims. Lane A remains
partial at aggregate W050 level because synthetic A1 compatibility helpers,
Minimal* fixture scaffolding, and legacy structured fixture/scale quarantine
surfaces still exist by design until CALC-002 and fixture migration lanes
provide replacement evidence.
