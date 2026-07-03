# CTRO recalc-integration design — the grid-driver contract

**Status:** parked implementation checkpoint, grounded against live code and **hardened by an
adversarial fresh-eyes review** (3 blockers, 4 majors folded in — June 2026). The live model is now
structural graph plus calc-time overlay graph; the older claim/consequence wording below is retained
only where it still explains the review pressure that produced the current implementation.

**Companion documents:**
- [`ctro-parked-rework-status.md`](ctro-parked-rework-status.md) — concise product status, readiness checklist, use guide, and remaining refinements for parking/resuming the rework.
- [`ctro-implementation-spec.md`](ctro-implementation-spec.md) — historical CTRO-1/2/3 claim-lifecycle spec, superseded by the ideal graph model in this document.
- [`calc-phase-dynamic-scenarios.md`](calc-phase-dynamic-scenarios.md) — 85-scenario corpus, gap analysis, Excel-behaviour register, review agenda.
- [`ctro-reference-overlay-architecture-review.html`](ctro-reference-overlay-architecture-review.html) — the review aid this design answers.

## Implementation checkpoint: ideal graph model now in flight

This document began in the older CTRO claim/consequence vocabulary. The active implementation direction has since shifted to the ideal model:

```text
effective calculation dependency graph
= structural/formula dependency graph
+ calc-time realized overlay graph
```

There is no compatibility requirement to preserve the old claim lifecycle. Useful pieces may survive only where they directly serve the clearer model: structural graph, overlay graph, runtime dependency trace, publication delta, and effective-graph worklist recalc.

Current implementation slices now landed in the Rust grid engines:

1. `GridInvalidationRef` has explicit `Structural` and `CalcOverlay` layers. Direct references such as `B1=A5` install structurally; runtime discoveries such as `B1=INDIRECT(C1)` install only in the overlay layer.
2. `GridRuntimeDependencyTrace` records realized reference-system effects during evaluation, filters out structurally known dependencies, and replaces the evaluated formula's overlay dependency set.
3. `GridValuePublicationDelta` turns changed/vacated effective spill cells, spill facts, and spill blockers into ordinary dirty seeds.
4. Reference and optimized dirty recalc paths close over the effective graph, evaluate a worklist, replace overlay dependencies, publish values, and feed publication-delta seeds back into the worklist.
5. Overlay replacement now returns a `GridOverlayDependencyUpdate` report with old/new dependency sets and dirty seeds for changed realized dependency identities. Dirty recalc uses that report to requeue from the effective graph when a runtime reference retargets, so overlay-edge changes participate in scheduling even when the authored structural graph did not change.
6. Runtime text resolution now records defined-name and structured-table identity dependencies for `INDIRECT("InputRange")` and `INDIRECT("Table1[Amount]")`, allowing `GridDirtySeed::Name` and `GridDirtySeed::Table` to reach dynamic consumers through the overlay layer.
7. OxFml semantic-plan volatility now feeds the runtime trace. `GridInvalidationRef` tracks volatile roots, `GridDirtySeed::Volatile` closes over those roots through the effective graph, and the optimized warm-no-op cache refuses reuse when the cached valuation contains volatile roots.
8. External/non-terminal state now has graph-level semantics: `GridRuntimeDependencyTrace.external_pending` and `GridRuntimeDependencyTrace.external_subscriptions` maintain external-pending roots and their topic descriptors, `GridDirtySeed::External` closes over those roots through the effective graph, axis edits transform the roots, and the optimized warm-no-op cache refuses reuse when the cached valuation contains external-pending roots. `GridExternalAvailabilityEventReport` exposes the broad producer-facing pending roots and dirty closure, while `GridExternalAvailabilityTopicRegistry`, `GridExternalAvailabilityTopicEnvelopeUpdate`, and `GridExternalAvailabilityTopicDispatchReport` provide ordered/deduped topic-envelope dispatch into precise formula-root and dynamic defined-name dirty closures. Ordinary formula evaluation projects OxFml semantic-plan external-provider dependence such as RTD into trace subscriptions (`topic:rtd`), recalc reports emit `GridExternalAvailabilitySubscriptionUpdate` replacements for evaluated formula roots and refreshed dynamic names, and the registry applies those replacements so stale topics are cleared. Exact provider/workspace topic-argument extraction from richer runtime effects remains deferred.
9. Reference and optimized dirty recalc now select the next pending formula by effective-graph readiness instead of raw address order. Pending formulas whose pending formula precedents are not yet evaluated wait behind those precedents, fixing forward formula chains such as `A1=B1+1`, `B1=C1+1` after a `C1` edit.
10. Dirty recalc now reports effective dependency cycles as `GridRefError::EffectiveDependencyCycleDetected`, including calc-time cycles discovered from runtime overlay edges such as `INDIRECT("A1")`. A stale overlay cycle can still be released by evaluating an overlay-bearing formula first; Excel-style iterative convergence remains a separate unimplemented engine.
11. Spill publication now installs a publication-owned spill-blocker watch in the calc-overlay layer for the current spill extent, and table resize/rename/delete lifecycle reports emit paired table and spill-blocker dirty seeds. This lets a table growth into a previously unrelated dynamic-array spill extent wake the spill anchor through the effective graph instead of relying on advance knowledge of dynamic targets.
12. Branch-sensitive runtime references are covered in both engines: when an `IF` branch flips from `INDIRECT("A5")` to `INDIRECT("A7")`, dirty recalc replaces the overlay edge rather than appending history, and the released target no longer reaches the formula through the effective graph.
13. Runtime resolution errors are covered in both engines: when `INDIRECT(C1)` retargets from a valid cell to invalid reference text, dirty recalc publishes `#REF!` and clears the stale overlay edge so the old target no longer dirties the formula.
14. Runtime reference transforms are covered in both engines: when `OFFSET(A1,C1,0)` retargets from `A5` to `A2`, the base and offset inputs remain structural while the realized target cell is replaced in the calc-overlay graph. The released target no longer dirties the formula.
15. Axis edits now treat overlay dependencies as stale realization state, not as authored references to transform. `GridInvalidationRef::apply_axis_edit` transforms the structural layer and formula-root sets, clears the calc-overlay layer, and lets evaluation rebuild realized dependencies. The reference sheet applies this rule during `apply_axis_edit`, so `INDIRECT("A5")` after an inserted row re-realizes textual `A5` rather than following old `A5` to `A6`.
16. Optimized valuations now have an explicit `apply_axis_edit` operation for carried snapshots. It transforms sparse and dense computed state, spill/name/table shape state, and the runtime dependency graph with the same structural-transform/overlay-clear rule, allowing dirty recalc to continue from a valuation that has been deliberately carried across a row/column edit.
17. Parsed spill-anchor references such as `A2#` are now structural spill-fact dependencies. The feeder walks the bound expression tree, recognizes `ReferenceExpr::Spill { anchor }`, installs `GridDependency::SpillFact(GridSpillDependency::anchor(A2))`, and leaves ordinary direct dependencies in the structural layer. Publication deltas seed spill-fact changes, so both reference and optimized dirty recalc now update `SUM(A2#)` when `SEQUENCE(A1)` grows or shrinks without treating the consumer as a calc-time overlay rediscovery.
18. External availability seeds now have value-engine coverage. The reference sheet and optimized valuation expose an explicit way to mark/clear external-pending formula roots and to produce the corresponding `GridDirtySeed::External`; dirty recalc then evaluates the pending root and its effective dependents, and a successful ordinary evaluation clears the pending root. `external_availability_event_report()` now reports the pending formula roots, pending dynamic defined names, dirty seeds, and effective dirty cells a host-side availability producer should hand to dirty recalc for broad availability flips. `external_availability_topic_event_report(...)` filters a recognized topic through `GridExternalAvailabilityTopicRegistry`, emits precise cell/name dirty seeds for currently pending subscribed roots, and leaves unrelated pending roots untouched. `dispatch_external_availability_topic_updates(...)` now applies ordered/deduped topic envelopes, retains topic-envelope state, and returns a union dirty closure ready for dirty recalc. Fresh runtime traces flow into recalc-report subscription replacements for formula roots and dynamic names, and applying those replacements to the registry clears stale topic roots on retarget/resolution. Ordinary RTD/external-provider semantic classification now creates trace subscriptions; the remaining external boundary is exact provider/workspace topic descriptor production from real runtime effects, not the grid topic-event, dirty-closure, or recalc consumer path.
19. The old grid `rebind.rs` claim/consequence module has been removed from the live machine. `DynamicRebind*`, `ResolvedReferenceIdentity`, and `resolve_dynamic_rebind_claim` are no longer re-exported grid APIs. The old lifecycle remains useful only as historical design material; the live model is structural dependencies plus calc-overlay dependencies plus runtime traces plus publication deltas consumed by effective-graph recalc.
20. Direct defined-name deletion now has dirty-recalc coverage in both grid engines. Deleting `InputRange` transforms direct consumers to the explicit `#NAME?` formula surface, `GridDirtySeed::Name("InputRange")` reaches the affected formula through the retained graph, and evaluation releases the stale structural name edge.
21. A graph-model dirty-vs-mark-all differential oracle now exists. `GridDirtyRecalcDifferentialRunReport` compares sampled values, spill facts, the post-run runtime dependency graph, and the dynamic defined-name runtime snapshot after dirty recalc against a fresh mark-all rebuild. The dynamic-name snapshot includes active keys, realized extents, the namespace formula-input dependency ledger, volatile dynamic-name roots, and external-pending dynamic-name roots. The corpus uses the oracle for clean equivalence checks on name release, overlay retarget, table resize, volatility, external roots, reference-engine spill growth into plain referenced cells, reference-engine forward formula chains, and dynamic defined-name retarget in both engines. Deliberately divergent dependency-graph and dynamic-name-state fixtures prove the oracle goes red even when sampled values and spill facts still match.
22. Defined-name lifecycle reports now produce dirty seeds just like table lifecycle reports. Rename emits old-name and new-name seeds; delete emits the old-name seed. Dirty recalc tests now consume those report-provided seeds instead of hand-building the name invalidation input.
23. Defined-name create/redefine now uses the same lifecycle report surface. `set_defined_name` reports `Create` for a new namespace entry and `Redefine` for a changed existing entry, both with a `GridDirtySeed::Name` payload. Name-resize dirty recalc tests now consume the redefine report directly for direct and `INDIRECT` consumers in both engines.
24. Unresolved name consumers now retain a namespace-only dependency. `GridDependency::NameIdentity` indexes a name key without pretending a target extent exists. Direct formulas against a missing name keep a structural name-identity edge; failed `INDIRECT("Name")` resolution keeps an overlay name-identity edge. Creating the name then dirties and heals both consumers through `set_defined_name(...).dirty_seeds`, after which successful evaluation replaces the identity-only edge with the resolved name/range dependency.
25. Dynamic defined names now have an implementation slice in both grid engines. `set_dynamic_defined_name(...)` stores an authored formula separately from static `defined_names`, recalc evaluates that formula through the runtime tracing provider, and the realized target extent is exposed to ordinary formula evaluation through the provider's defined-name namespace. Dirty recalc refreshes dynamic names from the namespace-side dependency ledger described below; any extent change emits the existing `GridDirtySeed::Name` surface, so direct consumers and `INDIRECT("Name")` consumers are reached through the effective graph. The key model correction is in place: selector inputs such as `C1` in `InputRange = INDIRECT(C1)` are not treated as the target; the calc-time realized target range is.
26. Volatile dynamic defined names now participate in the recalc and warm-no-op safety model. Dynamic-name evaluation records `volatile` and `external_pending` status alongside the realized extent. A `GridDirtySeed::Volatile` pass forces dirty seeds for volatile dynamic-name keys, so consumers such as `SUM(InputRange)` are reached even when `InputRange = OFFSET(...)` resolves to the same extent. Optimized cached valuations also retain volatile/external dynamic-name status and refuse warm-no-op reuse while such namespace roots are present. This closes the first volatile dynamic-name warm-no-op risk without inventing fake cell roots for names.
27. External-pending dynamic defined names now have the same explicit availability-root surface as external-pending formula cells. The reference sheet and optimized valuation expose `set_dynamic_defined_name_external_pending(...)`, `external_pending_dynamic_defined_names()`, and `external_availability_dirty_seeds()` now emits `GridDirtySeed::Name(...)` for pending dynamic names. Dirty recalc consumes that name seed through the effective graph, evaluates the consumer, clears the pending-name state after successful ordinary evaluation, and optimized warm-no-op refuses a cached valuation while a dynamic-name external root is pending. Optimized valuations also retain dynamic-name identity keys separately from realized extents, so an unresolved dynamic name with no current target can still be marked as an external-availability root. `GridExternalAvailabilityEventReport` includes those pending dynamic-name keys and the consumer dirty cells, and `GridExternalAvailabilityTopicRegistry` can map recognized topic-envelope updates to only the pending dynamic-name roots subscribed to each topic. Dynamic-name refresh now returns the same subscription-replacement updates as formula recalc, so the host registry can keep dynamic-name topic roots fresh. Ordinary formula evaluation now covers the first semantic-plan RTD/external-provider subscription projection; exact host/provider/workspace topic-argument production remains deferred.
28. Dynamic defined names now keep a namespace-side formula-input dependency ledger instead of relying on all-name refresh for every dirty recalc step. Each dynamic-name evaluation persists the dependencies that can affect the name formula's realized target (structural inputs plus runtime-traced inputs), while the existing realized extent remains the consumer-facing name dependency. Dirty recalc now selects dynamic names to refresh from dirty seed intersection, direct name seeds, volatile/external root sets, and conservative first-discovery cases. Reference and optimized reports expose `dynamic_defined_name_evaluations`, and both engines now have regression coverage proving a `C1` selector edit refreshes only `InputRange = INDIRECT(C1)`, not an unrelated `OtherRange = INDIRECT(D1)`. This moves dynamic names closer to the ideal graph model without pretending namespace entries are grid cells.
29. Dynamic defined-name refresh is now a bounded namespace worklist rather than a single pass. When a refreshed dynamic name changes realized extent, the refresh worklist enqueues other dynamic names whose formula-input ledger depends on that name, then emits the ordinary `GridDirtySeed::Name(...)` for cell/formula consumers. This fixes dynamic-name-to-dynamic-name ordering without relying on map iteration order: `AlphaRange = INDIRECT("ZuluRange")`, `ZuluRange = INDIRECT(C1)` now cascades on mark-all and dirty recalc in both engines. The same ledger now detects dynamic defined-name cycles and reports `GridRefError::DynamicDefinedNameCycleDetected { cycle }`, with reference and optimized tests covering `AlphaRange <-> ZuluRange`.
30. Spill growth into an existing table extent is now covered as a first-class grid publication/blocker scenario in both engines. A baseline unblocked `SEQUENCE(A1)` spill feeding `SUM(A2#)` can grow into a table overlay, publish `#SPILL!` at the anchor, dirty the spill-anchor consumer through the spill-fact dependency, and then shrink back out of the table and release the consumer to the restored value. This verifies the corrected grid rule: table/spill overlap is handled by effective publication deltas and blocker watches, not by advance enumeration of possible spill target cells.
31. Structured-table namespace identity now matches the missing-name model. Runtime traces record `GridDependency::TableIdentity(...)` when `INDIRECT("Table1[Amount]")` cannot resolve because the table namespace entry is missing, and table dirty seeds match both resolved table dependencies and table-identity dependencies. Reference and optimized dirty-recalc tests now cover direct structured refs plus `INDIRECT` through table rename, delete, and recreate: the direct ref follows rewrite/delete semantics, while the indirect text consumer errors on rename/delete and heals when `Table1` is recreated and a `GridDirtySeed::Table("Table1")` is produced.
32. Table creation/replacement now has the same report-produced dirty seed surface as the rest of table lifecycle. `set_table_overlay(...)` returns a `GridTableLifecycleReport` with `GridTableLifecycleOperation::Set`, a table dirty seed, and spill-blocker dirty seeds for the new table extent (plus the old extent when replacing an existing overlay). The table rename/delete/recreate tests now consume `recreate.dirty_seeds` directly, removing the last hand-built table-create seed from that scenario.
33. Runtime-realized range overlays are now covered directly, not just scalar `INDIRECT`/`OFFSET` targets. Reference and optimized dirty-recalc tests retarget `SUM(INDIRECT(C1))` from `A2:A4` to `A6:A8`, assert that the calc-overlay layer holds the realized `GridDependency::Range`, prove the old range no longer reaches the formula, and then edit an interior cell of the new realized range (`A7`) to force the consumer through the effective graph.
34. Formula-derived axis dependencies now feed the real structural graph. Whole-row/whole-column references such as `SUM(A:B)` install compressed `GridDependency::AxisValue(...)` edges instead of large scalar ranges, and hidden-sensitive aggregate calls such as `SUBTOTAL(109,A1:A3)` install `GridDependency::AxisVisibility(...)` for the consumed row span. Reference and optimized dirty-recalc tests now prove `GridDirtySeed::AxisVisibility(...)` updates hidden-sensitive consumers after row visibility changes, and `GridDirtySeed::AxisValue(...)` reaches whole-column consumers when a bulk/effective axis publication has already committed changed grid values. This closes the gap between the existing axis dependency index tests and real recalculation through the effective structural-plus-overlay graph.
35. Effective-graph readiness now uses non-scalar value-producing dependencies, not just direct scalar cell edges. The worklist treats pending formulas inside compressed range dependencies, whole-row/whole-column value dependencies, spill-anchor dependencies, and resolved name/table extents as pending precedents for the consumer. Spill-blocker watches remain invalidation triggers rather than value precedents, so a spill anchor does not wait on its own published/blocker extent. Reference and optimized tests cover a deliberately address-misordered case where `B3=A1*2` feeds `D2=SUM(A:B)` and `E2=D2+1`; after an `A1` edit, dirty recalc evaluates exactly the three formulas in dependency order.
36. The reference OxFml mark-all path now uses the effective-graph worklist instead of a single address-order formula pass. It installs structural dependencies for every formula before evaluation, selects formulas by graph readiness, and feeds publication-delta and dynamic-name dirty seeds back into the same worklist. Overlay-change and spill-blocker-watch dirty seeds remain incremental dirty-recalc feedback, not cold-rebuild feedback. Reference dirty-vs-mark-all differentials for forward formula chains and spill growth into a plain referenced cell are now clean. The optimized OxFml mark-all path now uses the same worklist for guarded sparse formula sets and for small repeated-region formula sets when a dependency probe shows address order is not effective-graph order. Metadata-only compiled reference functions such as `ROW`, `COLUMN`, `ROWS`, and `COLUMNS` remain on the compact path even when their structural references are address-forward, because they do not read precedent values. Ordinary repeated-region scale cases still use the existing compact fast path, and tests cover both the promoted repeated forward-chain case and the preserved scale counter case.
37. Stable blocked-spill publication no longer re-emits blocker dirty seeds forever. `GridValuePublicationDelta` still treats unblocked spill publications as value-epoch changes, but blocked spill blocker extents are emitted only when the blocked fact changes. Clearing an old blocked spill fact now removes only the anchor cell's effective output, not the whole blocked extent. This lets mutually blocking spill anchors settle to paired `#SPILL!` results instead of one anchor clearing the other's blocked value or requeueing indefinitely.
38. The optimized visible-first projection path no longer has a trace-blind formula evaluator. For every sparse or fallback repeated formula it actually evaluates, it installs structural dependencies, captures `GridRuntimeDependencyTrace`, replaces overlay dependencies, checks for effective dependency cycles, publishes through `GridValuePublicationDelta`, and refreshes publication-owned spill-blocker watches. This does not make visible-first a full semantic worklist recalc; its upstream cone remains the visible projection contract. It does mean a visible-first valuation that evaluates `INDIRECT` now carries the same structural/overlay graph evidence as the production recalc paths.
39. Reference-metadata functions now distinguish invalidation evidence from value-ordering evidence. `ROW`, `COLUMN`, `ROWS`, and `COLUMNS` arguments are recorded as `GridDependency::ReferenceMetadata(...)`, so the semantic graph still explains the authored reference while `next_ready_dirty_formula` does not treat the referenced cells/ranges as value precedents. Metadata namespace, table, and spill dependencies are indexed as invalidation-only edges: `GridDirtySeed::Name(...)` reaches `ROWS(InputRange)` after the named extent changes, `GridDirtySeed::Table(...)` reaches `ROWS(Table1[Amount])` after a table resize, and a spill-fact seed reaches `ROWS(A2#)` after the anchor extent changes. None of those metadata consumers wait on the referenced extent's cell values. This fixes the false-cycle case `A1=ROW(B1)`, `B1=A1+1` in both engines. A paired guardrail proves `A1=ROW(B1)+B1`, `B1=A1+1` still reports a real effective cycle because the ordinary `B1` value read remains in the value dependency graph.
40. Spill shrink/vacated effective cells now have explicit direct-range coverage in both engines. With `A2=SEQUENCE(A1)` and `B1=SUM(A5:A6)`, shrinking `A1` from `5` to `1` routes vacated `A5:A6` through ordinary structural range dependencies, dirties `B1`, and matches the mark-all oracle. This complements the earlier plain-cell vacate and range-growth coverage, and keeps the core grid rule visible: spill publication deltas dirty the effective cells themselves, then the effective graph reaches structural and overlay consumers.
41. Overlay dependencies now have explicit repeated-retarget guardrails in both engines. `B1=SUM(INDIRECT(C1))` retargets `C1` from `A5` to `A4` to `A6`; after the second dirty recalc the calc-overlay layer contains only `A6`, old targets no longer dirty `B1`, and the current target still reaches it through the effective graph. This pins the replacement-state invariant separately from the single-retarget tests.
42. Defined-name release is now explicitly distinguished from target-value release in both engines. `InputRange` pointing at `A1` keeps its structural name dependency when `A1` is cleared and `SUM(InputRange)` recalculates to `0`; deleting `InputRange` then transforms the direct consumer to `#NAME?` and releases the name edge. This keeps the old `NameOrValue` concern grounded in the new model: value changes flow through the extent, namespace deletion flows through the name lifecycle.
43. Defined-name keying now has a scoped-name slice for static and dynamic names. Workbook/global names keep the legacy bare key, while sheet-scoped names use a canonical `excel.grid.v1:scoped-name:...` key. Unresolved name identity dependencies keep both the global candidate and the current sheet-scoped candidate, so either kind of later name creation can heal a missing direct or `INDIRECT` consumer without a mark-all escape hatch. Reference and optimized dirty recalc both prove static `InputRange` sheet-scope create, shadowing, and fallback: direct `SUM(InputRange)` and runtime `SUM(INDIRECT("InputRange"))` can be healed by a sheet-scoped create, first bind to the sheet-scoped key when both scopes exist, then a sheet-scoped delete emits that scoped name seed, dirty recalc reaches the consumer through the structural or calc-overlay graph, and evaluation rebinds to the remaining workbook/global name. The same scoped key now covers dynamic defined names: `set_sheet_dynamic_defined_name(...)` realizes and retargets a scoped `InputRange = INDIRECT(C1)`, shadows the global name, deletes back to the global fallback through the scoped name seed, and accepts external-pending root marking by canonical scoped key. This closes the first namespace/scope keying gap without pretending multi-sheet workbook storage is complete.
44. External availability now has producer-facing grid reports for both broad availability flips and recognized topic updates. `GridExternalAvailabilityEventReport` is available from the reference sheet and optimized valuation, carrying pending formula roots, pending dynamic defined-name keys, the dirty seeds, and the effective dirty cell closure. `GridRuntimeDependencyTrace` can carry `GridRuntimeExternalSubscription` descriptors, and recalc reports now carry `GridExternalAvailabilitySubscriptionUpdate` replacement records for evaluated formula roots and refreshed dynamic names. `GridExternalAvailabilityTopicRegistry` applies those updates, maps topic IDs to formula roots or dynamic defined-name keys, stores topic envelope state with ordering/dedupe semantics, and `GridExternalAvailabilityTopicDispatchReport` returns the union dirty closure for applied topic-envelope updates. Reference and optimized tests prove a pending formula root reports `GridDirtySeed::External` and reaches its downstream formula, a pending dynamic defined name reports the name seed and reaches the consumer, topic-specific reports dirty only the subscribed pending formula/name closure while leaving unrelated external-pending roots in place, ordered/deduped topic-envelope dispatch feeds dirty recalc directly, and a recalc-produced RTD trace installs then clears `topic:rtd` registry roots through report updates. This narrows the earlier external-event gap: ordinary RTD/external-provider formulas now become external-pending roots with semantic-plan topic subscriptions during normal evaluation; the remaining work is extracting exact provider/workspace topic descriptors from richer runtime effects.
45. The dirty-vs-mark-all differential oracle itself is now hardened against the blind spots a prior review pass raised. `GridDirtyRecalcDifferentialRunReport` gained three new axes folded into `is_clean()`: an opt-in full-computed-state comparison over a caller-supplied address set (`compare_grid_dirty_recalc_full_state`, intended for small fixtures rather than sampled probes); spill-epoch-ledger equality (`spill_epoch_ledger_content_equal`, which compares ledger *content* — anchor, extent, blocked, value fingerprint — rather than the raw monotonic `value_epoch` counter, because that counter's absolute value legitimately depends on how many prior recalcs the seeding valuation carried, not on whether the two runs computed the same spill state; comparing the raw counter produced false positives against the existing `run_dirty_recalc_differential_with_oxfml` harness, which seeds dirty recalc from an already-once-recalced baseline valuation while mark-all cold-starts from the sheet's own committed ledger); and a registry-effect comparison (`compare_grid_dirty_recalc_registry_effect`) that applies both runs' `external_subscription_updates` to two clones of a caller-seeded `GridExternalAvailabilityTopicRegistry` and compares the resulting `roots_by_topic` maps. Both grid engines have a RED-fixture regression test that injects a bogus overlay edge via a test-only `runtime_dependency_graph_mut_for_test()` mutator (added to `GridOptimizedValuation` to match the reference sheet's existing one) on an unrelated bystander cell the dirty seed does not touch, and asserts the differential goes `!is_clean()` with `!dependency_graphs_equal` specifically — proving the graph-equality axis is load-bearing, not vacuous. `run_dirty_recalc_differential_with_oxfml` on both engines now threads the new comparison inputs through `build_grid_dirty_recalc_differential_report` (reusing the caller's probe addresses as the full-state address set, so existing small-fixture callers get the new axis for free without API churn); the registry-effect axis stays opt-in (`None` seed) for callers that do not pass one.
46. The volatile- and external-seed blast radius now has direct regression coverage, not just correctness-on-the-covered-cells coverage. The existing volatile-seed and broad-external-seed tests in both engines gained an unrelated non-volatile/non-external sibling formula with no dependency on the seeded root, and the tests now assert that sibling is not visited (reference engine `visited_cells`) and that `formula_evaluations` stays at its previously-asserted bound — closing the gap where a regression that dirtied every formula on a volatile/external seed would have passed the pre-existing fixtures. The optimized volatile+external differential test's clean assertion is no longer vacuous either: it now additionally asserts `formula_evaluations >= 1` on the dirty-recalc side, pinning that the volatile root was actually re-evaluated rather than merely happening to already match mark-all's cached value.
47. RTD's non-terminal "stays pending" behaviour now has explicit dirty-recalc coverage in both engines, beyond the existing baseline-mark-all-only RTD tests. A baseline mark-all over `=RTD(...)` plus a plain dependent, followed by `recalculate_dirty_*` seeded from `external_availability_dirty_seeds()`, terminates normally, leaves the RTD root in `external_pending_roots()` (RTD has no live provider in these tests, so it never stabilizes), still carries a `topic:rtd` subscription update for that root in the recalc report, and leaves the pending cell's published placeholder value unchanged across the re-evaluation.
48. Spill value-change and OFFSET out-of-bounds coverage closed two named gaps in both engines. A same-extent spill VALUE change (`A2=SEQUENCE(3,1,A1)`, editing `A1`'s start value with the row count fixed at 3) now has direct dirty-recalc coverage proving a `SUM(A2#)` consumer updates through the spill-fact value-epoch path with the spill extent provably unchanged — distinct from the pre-existing extent-change coverage. `OFFSET`'s out-of-bounds retarget error leg (`B1=SUM(OFFSET(A1,C1,0))` retargeting `C1` far enough to push the resolved target past `max_rows`) now mirrors the existing `INDIRECT` error-retarget tests: the consumer publishes `#REF!`, the stale in-bounds overlay target is cleared from the calc-overlay layer, and the old target no longer reaches the formula through the effective graph.
49. The external-availability topic registry now has conflict-semantics tests matching the pruning window `apply_topic_envelope_update` already implements (prune dedupe identities recorded at strictly older sequences once the topic advances past them; retain identities at the current sequence). An equal-sequence, distinct-dedupe-identity pair is not a duplicate: both envelopes apply in processing order (the later one becomes the retained envelope) and both identities remain tracked in-window. A redelivery that reuses the *current* (already-applied) sequence's own dedupe identity — simulating an at-least-once transport re-sending a message it believes was already applied — is dropped rather than re-applied, and does not overwrite the retained envelope. These tests are engine-agnostic: `GridExternalAvailabilityTopicRegistry` is a shared host-owned type with no reference/optimized split, so one set of tests covers both engines' usage of it.

The remaining design sections below are useful historical analysis. They intentionally preserve the old claim/consequence vocabulary, but they are no longer the live API plan. Future refactors should prefer the ideal model over preserving old CTRO type names or lifecycle boundaries.

The pre-refactor text below was the **revised CTRO + recalc-integration specification** for the older
claim lifecycle. It is retained because it names the original wiring gap and review pressure points,
not because the removed `rebind.rs` API should be restored.

> **What the review changed.** The first draft claimed the existing spill-repair fixpoint could serve
> as the convergence/ordering layer for a seeded recalc. That is **false**: the fixpoint is gated off
> for non-spill sheets and converges on `spill_facts`-map equality, *not* value convergence
> (`calc_ref_sheet.rs:726-731,747`). Today's recalc reads precedents from the partial shared valuation
> with **no recursion** (`with_borrowed_cells(&self.computed)`, `calc_ref_sheet.rs:968`), so a
> formula→formula forward reference in a non-spill sheet can read a stale precedent (`DOR-FORWARD-REF`).
> The corrected design makes the **dependency graph the ordering authority** (topological evaluation of
> the seed) and keeps the fixpoint only for its real job: spill-extent re-convergence. Two more blockers
> (where `request_key` is derived; the error enum cannot express the matrix) and several majors are
> folded in below.

---

## 1. The problem, stated precisely (grounded)

CTRO is split across three subsystems that **share no data path at value-recalc time** — that
disjointness *is* the wiring gap.

| Subsystem | What it is | State |
|---|---|---|
| **Producer** | `GridInvalidationRef` — the semantic dependency graph + `dirty_closure_for_*` machinery (`grid/machine/invalidation.rs:196`). | Built. Constructed **only by closure/scale test harnesses** (e.g. `run_invalidation_scenario`, `grid/runner.rs:1051`), **never by a value engine** (verified zero matches in `optimized_sheet.rs`/`calc_ref_sheet.rs`). |
| **Consumer** | The value engines `GridOptimizedSheet` / `GridCalcRefSheet`. | Recalc is **mark-all-dirty**: one address-order pass + a bounded spill-repair fixpoint (`calc_ref_sheet.rs:656,718`; `optimized_sheet.rs` mirror). They hold **no** `GridInvalidationRef`, consult **no** dirty closure, are **volatility-blind**, and have **no forward-reference re-queue**. |
| **Resolver** | Historical `grid/machine/rebind.rs` claim resolver — `resolve_dynamic_rebind_claim(refs, claim) -> Result<Consequence, GridRefError>`. | Removed from the live grid machine after the ideal graph-model implementation began. It had no production caller; the live replacement is runtime traces, overlay dependency replacement, publication deltas, and effective-graph recalc. |

Four concrete things are missing, in dependency order: **(1) no feeder** (nothing turns bound
formulas into `GridDependency` edges — OxFml's `normalized_references` is never installed); **(2) no
claim producer** (the resolution sites in `grid/reference_engine.rs` produce values with zero claim
emission); **(3) no consumer seam** (nothing applies a `dirty_closure` as recalc work); **(4) no
oracle** (no `GridDynamicRebindMismatch` / `compare_dynamic_rebind_consequences` /
`run_dynamic_rebind_differential` / `GridDifferentialRunReport.dynamic_rebind`). This is why **CTRO-3
was paused**: its planned home is unreachable until the feeder exists.

### Two review questions, now answered by the code

- **`UNC-SINGLE-PASS-RECURSE` — does a recalc read stale precedents? RESOLVED: yes, it can.**
  `evaluate_formula_with_oxfml` reads precedent values from the in-flight `self.computed` map via
  `with_borrowed_cells(&self.computed)` (`calc_ref_sheet.rs:1009,968`) — **no on-demand recursion**.
  Literals are pre-loaded before the formula pass (`calc_ref_sheet.rs:679-683`), so *literal*
  precedents are always correct. But a **formula→formula forward reference** (e.g. `A1=B1+1` where
  `B1` is a formula later in address order) reads a **stale** precedent: the single pass visits in
  BTreeMap address order once, and the spill-repair fixpoint is **gated off for non-spill sheets**
  (`calc_ref_sheet.rs:726-731`) and converges only on `spill_facts`-map equality
  (`calc_ref_sheet.rs:747`), never on value change. This is the corpus's `DOR-FORWARD-REF` gap
  (corpus §DOR-FORWARD-REF). **Design consequence: mark-all-dirty is not order-correct for forward
  formula chains; the recalc-driver must supply ordering itself — which is exactly what a dependency
  graph gives us.**
- **`UNC-IF-BRANCH-UNION` — are both IF/CHOOSE arms tracked? Yes, by construction.** `bind_call`
  binds every argument uniformly (only `LET`/`LAMBDA` are special-cased); every reference atom lands
  in `normalized_references` (OxFml `binding/mod.rs:966-973`). `IF` only adds a `BranchLazy`
  *evaluation* requirement — it never prunes edges. The static edge set over-approximates, which is
  Excel-correct (no under-dirtying). CTRO can trust `normalized_references`. *(The corpus still lists
  this as an integration-verification item; the binder evidence settles the edge-membership half.)*

### Excel observation: UDF text dereference can run before the target is ready

We ran a local Excel/VBA probe for the question:

```excel
A1: =MyFunc("A2")
A2: =1+1
```

where `MyFunc` internally evaluates the text reference `A2`, logs the value it sees, and returns that
value. The probe was a clean-room black-box observation through Excel COM/VBA, not inspection of
Excel internals. On Excel `16.0 (build 20026)`, a full rebuild produced the critical call sequence:

```text
call 4, A1=MyFunc("A2"):
  A2 formula: =1+1
  A2.Value2 observed inside MyFunc: <Empty>
  ws.Evaluate("A2"): <Empty>
  MyFunc return: <Empty>

call 5, A1=MyFunc("A2") again:
  A2 formula: =1+1
  A2.Value2 observed inside MyFunc: 2
  ws.Evaluate("A2"): 2
  MyFunc return: 2
```

Interpretation for CTRO:

1. Excel does not necessarily suspend the caller before invoking the UDF. The first UDF invocation
   can observe an empty/stale value for the dynamically dereferenced target.
2. Excel later calls the UDF again after the target has calculated, and the final visible value is
   the later value.
3. The probe does **not** prove whether Excel internally commits then overwrites, suppresses the first
   publication, or uses another private calc-chain state. It only proves the visible UDF call order
   and the values available to the UDF body.
4. This supports the OxCalc model of evaluation-owned overlay discovery plus effective-graph requeue:
   a formula may run early, discover a runtime dependency, and then be requeued when the discovered
   precedent publishes.
5. A useful future refinement is an explicit "provisional evaluation / not stabilization-worthy yet"
   state for formulas whose runtime dereferences touched not-yet-ready formula precedents. That would
   be a scheduler/publication refinement, not a replacement for the structural-plus-overlay graph.

---

## 2. Scope boundary — what these seams cover, and what is deferred

This design is the recalc-integration contract for the **address-dynamic and spill/table** dynamic
behaviours. It is **not** a complete dynamic-calc engine. Stated explicitly so the phasing is not
mistaken for full corpus coverage:

| Covered by the four seams | Deferred (named, with reason) |
|---|---|
| Address-dynamic **cell** refs: `INDIRECT`, `OFFSET` (the `CellDynamicRequest` family). | **Rich objects / `FIELDVALUE`** — `ResolvedReferenceIdentity` has no field/entity variant; needs a new family. Belongs with the rich-object surface (unbuilt). |
| **Spill anchors** `A1#` (`SpillAnchorRef`). | **External / non-terminal event production**: RTD push, workspace-availability flips, `#BUSY!`/`ExternallyInvalidated` re-enqueue — the graph now has an external-pending root set, `GridDirtySeed::External`, broad `GridExternalAvailabilityEventReport`, topic-specific registry/envelope dispatch through `GridExternalAvailabilityTopicRegistry` / `GridExternalAvailabilityTopicDispatchReport`, fresh-trace subscription replacement via recalc-report `GridExternalAvailabilitySubscriptionUpdate`, and semantic-plan RTD/external-provider subscription projection during ordinary formula evaluation. Exact descriptor production from real provider/workspace runtime effects is still deferred. |
| **Structured tables** (`StructuredTableRebind`). | **Cross-sheet / 3D references** — `apply_axis_edit` handles only Row/Column on one sheet (`invalidation.rs:564`); 3D spans and sheet-axis edits are out of scope here. |
| **Volatility roots**: `NOW`/`TODAY`/`RAND`/`RANDBETWEEN`/`RANDARRAY`/`OFFSET`/`INDIRECT`/`WEBSERVICE` (whole-function). | **Name scope / per-name volatility / redefine** beyond the static-vs-dynamic name split in §4.1; **arg-conditional `CELL`/`INFO`** (format-sensitivity is a non-value precedent axis a whole-function flag can't capture). |
| The **spill extent-delta** plain-cell case (`B1=A5`). | **Iterative calc convergence** — dirty recalc now reports effective dependency cycles, including runtime overlay cycles, but it does not implement Excel's `MaxIterations`/`MaxChange` iterative engine. That remains distinct from the spill-repair fixpoint (`DOR-CYCLE`, `DOR-MIX-02`). |
| | **Legacy CSE arrays** — separate from dynamic-array spill. |

**The seeded recalc reports effective dependency cycles but provides no iterative-calc convergence.**
A calc-time cycle (`DOR-MIX-02`: a dynamic ref resolving to its own owner) is now surfaced as an
effective dependency cycle in dirty recalc. Excel-style iterative evaluation with `MaxIterations` /
`MaxChange` is still out of scope here and called out in §8.

---

## 3. Design tenets

Each is forced by a grounded fact, not a preference.

1. **The dependency graph is the ordering authority — not address order, not the spill fixpoint.**
   `close_over_dependents` (`invalidation.rs:1402`) returns an *unordered* `BTreeSet`. Today's engine
   evaluates in address order with no forward-ref re-queue, so it mis-orders forward formula chains
   (§1). Because the feeder (§4.1) builds the precedent relation (`semantic_dependencies_by_cell`),
   the consumer **topologically orders the dirty seed** (Kahn over precedent edges) before evaluating.
   This fixes `DOR-FORWARD-REF` *as a byproduct of wiring CTRO*. The spill-repair fixpoint is retained
   **only** for spill-extent re-convergence and dynamic-cascade re-resolution (re-seed + re-order),
   with a generalized continue-condition (below) — it is not the ordering mechanism.
2. **Resolve-structural-edges-before-purge; discover-claims-iteratively.** Split into two:
   - **(a)** Table/name **delete/rename** must resolve the old-name dependents *before* the edges are
     purged (the CTRO-2 fix mirrors `delete_table` dirtying the old key first, `invalidation.rs:744`).
     Achievable: defer the structural edge purge until after the per-epoch resolution that reads it.
   - **(b)** Dynamic **claim discovery is interleaved with evaluation**: a claim's `after` identity is
     only known *after* the owner formula evaluates, and evaluation reads precedents from the in-flight
     valuation. So the orchestrator is a **re-collect loop**, not a single collect-then-resolve phase.
     An `after`-identity is valid only relative to the partial valuation of the pass that minted it.
3. **Request keys are stable across target movement *and derivable where they're needed*.** The cell
   feeder seeds from one `request_key` that holds *old ∪ new* dependents (`rebind.rs:279`). The key
   must be `(owner_address, OxFml-call-site-handle, a1_mode)` — derived in the **bind/feeder layer**
   (where the owner and the OxFml call-site identity are both in scope), **not** at the resolution
   site, which sees only evaluated text (§4.1, §4.2). Same key at edge-install and claim-mint time.
4. **Volatility is a separate always-dirty root set, not an identity transition.** A `=NOW()` cell has
   *no edge that changes* between recalcs, so CTRO's `IdentityPreserved ⇒ clean` model is the wrong
   shape. Volatile roots are an orthogonal seed (§6), and — per tenet 1 — they must be evaluated in
   topological order so a volatile root's non-volatile dependents read the recomputed root in one run.
5. **OxCalc never parses formulas.** Edges, dynamic-call-site identity, and the `request_key` come
   *through OxFml's reference-system provider and `normalized_references`*, never by inspecting formula
   text (core project doctrine).
6. **Anchor-only seeding is correct for *declared* dependents; the engine still writes the spill body,
   and the extent delta is a separate seed.** CTRO's spill closure seeds by anchor, never extent
   (`rebind.rs:340`); the `B1=A5`-enters-a-spill case needs a consumer-side extent-delta seed (§4.4).
7. **Permanent-pair discipline for the oracle, and an independent Excel oracle beyond it.** A
   reference arm must be built from the *same committed state* (mirroring `project_authored_to_reference`,
   `optimized_sheet.rs:2245`). But "incremental == mark-all-dirty" only proves *value-neutrality*, not
   Excel-faithfulness — so the gate is paired with corpus expected-value fixtures (§5).

---

## 4. The contract — four seams

### 4.1 FEEDER — bound formulas → `GridDependency` edges (and the `request_key`)

**Goal:** make `GridInvalidationRef` reflect the live sheet, and own the stable `request_key`.

**Where:** at bind (where a `SemanticPlan` is obtained), call `set_cell_dependencies(owner, edges)`
with edges derived from OxFml's `normalized_references` + the per-reference descriptor kind:

| OxFml reference descriptor | `GridDependency` variant |
|---|---|
| direct cell | `Cell(address)` |
| direct range / area | `Range(rect)` |
| **static** (rect-bound) defined-name | `Name(GridNameDependency)` |
| **dynamic** name (`OFFSET`/`INDIRECT`/`INDEX`-based, e.g. `Data=OFFSET(...)`) | `DynamicRequest(request_key)` with per-name volatility (OFFSET volatile, INDEX not) — *not* a static `Name` edge, or **deferred to a later bead** if dynamic names are out of this tranche |
| structured table reference | `Table(GridTableDependency::new(name, …))` (keyed by **human name**) |
| spill-anchor reference `A1#` | `SpillFact(GridSpillDependency::anchor(anchor))` |
| dynamic call site (`OFFSET`/`INDIRECT`, `Contextual`) | `DynamicRequest(request_key)` **and** the static both-arms edges for its arguments |

Constraints:

- `set_cell_dependencies` is **full-replace per cell** (`invalidation.rs:359`) — re-binding re-authors
  the cell's whole edge list; there is no `add_edge`/`remove_edge`. A per-cell bind step matches this;
  a structural (axis) edit pays the `apply_axis_edit` rebuild (`invalidation.rs:564`), which exists.
- **The `request_key` is derived here**, once, as `(owner_address, OxFml-call-site-handle, a1_mode)`
  and threaded *down* to the resolution site via the provider/request context. It must be stable while
  the resolved target moves (the normalized call **site**, not the evaluated text/target).
  **FEED-1 precondition (verify, don't assume):** confirm OxFml's `normalized_references` exposes a
  stable normalized dynamic-call-site identity usable as that handle.
- **Cost:** the feeder runs at bind, closures at recalc, both bounded by edges — net cheaper than the
  `formula_count × formula_count` worst case of the all-formula fixpoint. Confirm on the scale harness.

This feeder is the **shared prerequisite** for both production wiring (§4.3) and the CTRO-3 oracle (§5):
the oracle's reference arm rebuilds a `GridInvalidationRef` *from authored cells*, which is what the
feeder does.

### 4.2 CLAIM PRODUCTION — resolution sites → `DynamicRebindClaim`

**Where:** the resolution sites in `grid/reference_engine.rs` that today return values with no claim:
`resolve_text` (INDIRECT, `:657`), `offset_reference` (`:783`), `spill_anchor_dereference_report`
(`:263`), `structured_reference_rects` (`:306`). These sites receive the **precomputed `request_key`**
from the feeder via the provider context (tenet 3) — they do **not** derive it (they only see
evaluated text, not the owner or the call-site AST).

**The rebind ledger.** Add `rebind_ledger: BTreeMap<request_key, ResolvedReferenceIdentity>` to the
sheet, beside the existing `spill_epoch_ledger`. Per recalc epoch, at each resolution site:

1. Compute the `after` `ResolvedReferenceIdentity` (`Cell{request_key, target}`; `Spill(snapshot)`;
   `Table{table_key=name, resolved_identity}`).
2. Look up `before` from the ledger.
3. `before == after` → no claim (reference-preserving short-circuit, `rebind.rs:265`).
4. Else emit a `DynamicRebindClaim { … }` and update the ledger.

Because claim discovery is **interleaved** with evaluation (tenet 2b), this is a re-collect loop, not
a one-shot collection: a claim minted in pass *k* may change a precedent that mints another claim in
pass *k+1*. Spill `before`/`after` snapshots come from `spill_epoch_snapshots()`
(`optimized_sheet.rs:258`, `calc_ref_sheet.rs:242`). The table feeder carries the **human table name**
in `ResolvedReferenceIdentity::Table.table_key` (the load-bearing CTRO-2 convention, `rebind.rs:411`).

**Pure anchor-move caveat (`DAS-SPILL-ANCHORMOVE`).** `spill_epoch_change_kind` keys on
extent/value_epoch/blocked and **ignores anchor identity** (`optimized_sheet.rs:4156`; comment at
`rebind.rs:1213`). A spill whose anchor relocates with identical extent/value/blocked yields `None` →
*no claim minted* → §4.4's extent-delta never runs *and* the moved-anchor dependents are missed by
both paths. Fix at this seam: **mint a claim whenever the before/after snapshots differ in anchor**
(or add anchor identity to the change-kind), so the spill feeder's existing before∪after-anchor union
fires.

### 4.3 CONSUMER — the recalc-driver loop

The **orchestrator** sits at the recalc entry and, per edit/epoch, runs a re-collect loop (tenet 2b):

1. **Collect** the claims produced this pass — a **claim fan with ordering**, not a single "paired
   claim": one claim per *distinct table/dynamic key a cell references* (cross-table cells emit
   several), plus the multi-effect cases (table-grows-into-spill → a `StructuredTableRebind` claim
   *and* a separate `SpillAnchorRef(BlockedChanged)` claim; PQ refresh → external seed + table rebind +
   downstream spill). Where an axis edit and a table/dynamic closure both apply, order is
   **transform-then-resolve** (`apply_axis_edit` re-addresses, *then* resolve against the transformed
   graph) — `IOV-OFF-06`/`STBL-TABLEMOVE`/`STBL-AUTOEXPAND`.
2. **Resolve** each claim via `resolve_dynamic_rebind_claim(refs, claim)` against the
   currently-maintained `GridInvalidationRef`, **before** any structural edge purge (tenet 2a).
3. **Union the seed:** `seed = ⋃ consequence.dirty_closure ∪ volatile_roots ∪ external_pending_roots ∪ extent_delta_seeds`.
   Dedup/key by `claim_id` or `(family, request_key)` — **never** by `ResolvedReferenceIdentity`
   (deliberately not `Ord`/`Hash`, `rebind.rs:32`).
4. **Topologically order the seed** (tenet 1) using `semantic_dependencies_by_cell`, then evaluate in
   that order. **This is the fix for `DOR-FORWARD-REF`** — it is what makes the seeded recalc
   *more* correct than mark-all-dirty, not less. Order must be specified independently of the
   optimized engine's three source-buckets (sparse / repeated-region / dense,
   `optimized_sheet.rs:1912,1936`).
5. **Re-converge** the spill-repair fixpoint **only for spill**, with its continue-condition
   **generalized** from `spill_facts == spill_facts_before` to *"re-loop while any seeded cell's
   published value OR the rebind ledger changed this pass"*, and the spill-presence gate **dropped in
   seeded mode**. Each re-pass re-collects newly-minted claims (step 1), re-seeds, re-orders. Bounded
   by `formula_count`; effective dependency cycles report as terminal cycle errors, while iterative
   convergence remains out of scope (§2).
6. **Error effects → terminal values:** after evaluation, a consequence's `error_effect` maps the
   dirtied cells' result where the reference layer hasn't already produced it (§7).

**Warm-no-op soundness.** The P19 warm-no-op token has no volatility or external-pending field, so it
would unsoundly skip a volatile or non-terminal sheet. Inject the `volatile_roots`/
`external_pending_roots`-non-empty (and per-epoch claim-set non-empty) fact into the token so the fast
path never fires for sheets that must be re-polled, or always re-dirties the relevant roots.

### 4.4 SPILL EXTENT-DELTA seam (the `B1=A5` case)

The anchor-only closure is correct for cells that *declared* a spill-fact dependency on the anchor, but
does **not** dirty an ordinary `B1=A5` when `A5` enters/leaves `A2`'s extent. Resolution:

- The engine's spill-**write** path materializes/clears the spill body cells (CTRO does not own writing
  the extent). On **shrink**, vacated cells must be **cleared before** their plain-cell dependents
  re-evaluate within the seeded pass — an ordering constraint the topological order (tenet 1) must
  honor (clear-then-recompute).
- When a spill epoch's extent changes, seed the **symmetric difference of the old and new extents** as
  ordinary `Cell` dirty roots; `close_over_dependents` then flows from those addresses along `B1`'s
  ordinary `Cell(A5)` edge (installed by the feeder). The delta is derivable from the before/after
  `GridSpillEpochSnapshot`s already in hand.
- This extent-delta seed **composes with** (does not replace) the anchor spill-fact closure and any
  compressed-range-overlap closure. Fixtures: `DAS-SPILL-SHRINK`, `DAS-SPILL-ANCHORMOVE`,
  `DOR-RANGE-OVERLAP`.

---

## 5. CTRO-3 — the differential oracle (re-scoped, with an honest ground-truth limit)

**Insight:** `resolve_dynamic_rebind_claim(refs, claim)` is **pure over `(refs, claim)`**
(`rebind.rs:567`) — the incremental and brute-force arms differ in exactly one input: *which
`GridInvalidationRef`*.

- **Incremental arm:** the live, incrementally-maintained ref (`apply_axis_edit`-transformed).
- **Reference arm:** a `GridInvalidationRef` **rebuilt from scratch** from the same committed state via
  the feeder (§4.1) using the `run_invalidation_scenario` idiom (`runner.rs:1051`). It re-resolves
  identity; the incremental ref only re-addresses — which is exactly where `AxisEditDuringRebind`
  divergence shows up (the strict-closure gate the taxonomy calls out).

**The compare:** add `compare_dynamic_rebind_consequences(incremental, brute_force) ->
GridDynamicRebindMismatch` (per-field `*_equal` flags + both consequences,
[`ctro-implementation-spec.md:98`](ctro-implementation-spec.md)), aggregated into a
`DynamicRebindReport`, surfaced as a new `GridDifferentialRunReport.dynamic_rebind` field, wired as a
**third compare** in the `Both` arm (`optimized_sheet.rs:1152`).

**Two layers, reported separately:** the *consequence* compare catches closure/reason/kind/error-effect
regressions; the existing *value + `spill_facts`* compares catch value regressions. Conflating them
muddies attribution.

**Harden the foundation first:** add a `spill_epoch_snapshots()`-equality compare to the `Both` arm
*before* trusting those snapshots as claim identities (they are not cross-checked between engines today).

**Error-divergence is a mismatch class, not an abort:** `resolve_dynamic_rebind_claim` returns
`Result<_, GridRefError>`; define "error on one arm / value on the other" as an explicit variant of
`GridDynamicRebindMismatch`.

**The oracle's honest limit (do not over-claim).** "incremental == mark-all-dirty" proves the wiring is
**value-neutral**, *not* Excel-faithful. Mark-all-dirty re-evaluates every cell every pass, so it
*accidentally* gets volatiles right (everything dirty) — **masking VOL under-dirtying bugs** — and it
is non-faithful for `DOR-FORWARD-REF`, cycles (`DOR-CYCLE`/`DOR-MIX-02`), and volatile-size `#SPILL!`
(`DAS-SPILL-NOCONVERGE`). And CTRO-3 is **blind to a feeder/binder error common to both arms** (a
wrong-key or under-complete edge set yields two identical-but-wrong arms reporting green). Therefore:
- Pair the differential with **independent corpus expected-value fixtures** (the scenarios' `Excel:`
  lines) for the cases where mark-all-dirty is known non-faithful.
- Give **FEED-1 its own ground-truth check** (hand-authored corpus closures), not the differential
  alone.

**Sequence:** build CTRO-3 *before* shipping CTRO-driven production recalc.

---

## 6. Volatility roots (VOL-1)

At bind, read `SemanticPlan.execution_profile.volatility` (OxFml — the field at `semantics/mod.rs:46`
`execution_profile` / `:179` `volatility`; the value is computed by `promote_volatility` at `:874`) and
reuse the value-cache predicate `volatility != Stable || determinism != Deterministic`
(`treecalc.rs:3389`) to populate `volatile_roots: BTreeSet<address>` — an **unconditional per-recalc
dirty obligation**, independent of any identity transition, forcing recalc regardless of `IF` branch.

- **Volatile set (full corpus list):** `NOW`/`TODAY`/`RAND`/`RANDBETWEEN`/`RANDARRAY`/`OFFSET`/
  `INDIRECT`/`WEBSERVICE` + arg-conditional `CELL`/`INFO`.
- **Deferred:** per-arg (not per-function) volatility and **format-sensitivity** — `CELL("format",D1)`
  is sensitive to `D1`'s *display format*, a non-value precedent the whole-function flag over-fires for
  and the value axis misses. Out of this tranche.
- **Interaction with tenet 1:** a volatile root's non-volatile dependents must read the recomputed root
  in one converged run — guaranteed by the topological-order evaluation (§4.3 step 4), *not* by the
  spill fixpoint (which doesn't run for non-spill chains).

`WEBSERVICE` also belongs to the deferred external/non-terminal domain (§2) for its `#BUSY!`/async
behaviour; only its volatile-leaf classification is in scope here.

---

## 7. Error routing (ERR-1) — a type-widening task, not just a spec gap

The corpus wants a `family × cause × target_kind → terminal value` matrix. The current types **cannot
express it**:

- `DynamicRebindErrorEffect` is a **4-value enum** `{None, Ref, Spill, NameOrValue}` (`rebind.rs:92`)
  and `error_effect` is a **single scalar field** (`rebind.rs:141`), not a function of target kind.
- The cell resolver **hardcodes `Released → Ref`** (`rebind.rs:282`) with no family/cause/target-kind
  discrimination — so a deleted **name** (must be `#NAME?`) and a deleted **cell** (must be `#REF!`)
  are **indistinguishable** (`UNC-NAME-RELEASE-EFFECT`).

ERR-1 is therefore: **(a)** widen `DynamicRebindErrorEffect` to carry the full `WorksheetErrorCode`
family (the engine already produces `WorksheetErrorCode::{Ref,Spill}`, `calc_ref_sheet.rs:766,832`) or
route through it; **(b)** thread `target_kind` into the `Released` arm of
`resolve_cell_dynamic_request_claim` / `resolve_structured_table_claim` so deleted-name vs
deleted-target diverge. `#FIELD!`/`#CALC!`/`#BUSY!`/`#CONNECT!`/`#BLOCKED!`/`#N/A` are tied to the
**deferred rich/external domains** (§2), not ERR-1.

This also forces the §9 decision on whether `ResolvedReferenceIdentity` gains a `Name` family (the only
way `#NAME?` routes *through* CTRO rather than the name-resolution path).

---

## 8. Phasing — proposed bead set

Each bead carries the **fresh-eyes review cycle before commit**, the per-bead invariant, and the gate
(`cargo build/test/fmt -p oxcalc-core` clean, trunk commit on `main`).

| Bead | Scope | Unblocks | Invariant |
|---|---|---|---|
| **FEED-1** | The OxFml-`normalized_references` → `GridDependency` feeder; build a `GridInvalidationRef` from a sheet's bound formulas + committed overlays. Define + enforce the `request_key = (owner, call-site-handle, a1_mode)` derivation **in the bind layer**; verify OxFml exposes a stable call-site handle. | CTRO-3 **and** all production wiring | A fed ref's closures equal the **hand-authored corpus closures** (independent ground truth, not the differential); `request_key` stable across an INDIRECT retarget. |
| **CTRO-3** | `compare_dynamic_rebind_consequences` + `GridDynamicRebindMismatch` (incl. error-divergence variant) + `DynamicRebindReport` + `GridDifferentialRunReport.dynamic_rebind`; `Both`-arm hook (incremental vs FEED-1 reference ref). Add the `spill_epoch_snapshots` equality compare first. **Pair with corpus expected-value fixtures** for the non-faithful cases. | Proof gate for production wiring | Incremental consequence == brute-force consequence field-by-field on the corpus; a deliberately-broken fixture proves the gate *detects* divergence. |
| **VOL-1** | Full volatile-root set from `execution_profile.volatility`; inject into the warm-no-op token; topological-order interaction. | Sound warm-no-op | A sheet with any volatile never warm-skips; volatile root's dependents read the recomputed root in one run; Stable sheet still warm-skips. |
| **WIRE-1** | Claim production at the four `reference_engine.rs` sites + the `rebind_ledger`; the **re-collect** discovery loop; the pure-anchor-move claim. | The consumer seam | Reference-preserving → no claim; retarget/spill-change/table-change → right family + before/after identity; anchor-move mints a claim. |
| **WIRE-2** | The consumer orchestrator: collect (claim fan + transform-then-resolve order) → resolve-before-purge → union seed → **topological evaluation** → generalized spill fixpoint → extent-delta (incl. shrink clear-then-recompute). | CTRO-driven recalc | Seeded recalc == mark-all-dirty final values/`spill_facts` on the corpus **and** == corpus `Excel:` fixtures for `DOR-FORWARD-REF` (where they differ — proving the topological fix). |
| **ERR-1** | Widen `DynamicRebindErrorEffect` to `WorksheetErrorCode`; thread `target_kind` into the `Released` arms; decide the `Name`-family question (§9). | Excel error parity | Deleted target → `#REF!`; blocked spill → `#SPILL!`; deleted **name** → `#NAME?`. |

Build order: **FEED-1 → CTRO-3 → (VOL-1 ∥ WIRE-1) → WIRE-2 → ERR-1.** The oracle (FEED-1, CTRO-3)
lands first and is value-neutral; it proves the model before any recalc behaviour changes — and note
WIRE-2 is the first bead that *intentionally diverges* from mark-all-dirty (it fixes `DOR-FORWARD-REF`),
so its invariant is anchored to the corpus `Excel:` fixtures, not to mark-all-dirty.

---

## 9. Decisions that are genuinely yours

1. **`NameOrValue` / a `Name` family.** `ResolvedReferenceIdentity` has no `Name` variant, so CTRO
   cannot currently model a defined-name release; a deleted name surfaces `#NAME?` through the
   name-resolution path. **Option A (recommended):** keep name-release in the name-resolution path;
   CTRO stays about *reference-target* movement; `NameOrValue` documented as reserved. **Option B:** add
   `ResolvedReferenceIdentity::Name` ("CTRO-4") so name lifecycle becomes a first-class claim with
   `#NAME?` routing — more uniform, wider resolver surface. ERR-1 needs this decided.
2. **Oracle-first, or wire production straight away?** FEED-1 + CTRO-3 prove the model with **zero
   behaviour change**. Given how delicate this area is, I lean **oracle-first**: land FEED-1/CTRO-3,
   confirm green on the corpus, *then* WIRE-1/2.
3. **How far to chase Excel-faithfulness now.** The seeded recalc *fixes* `DOR-FORWARD-REF` and now
   reports effective dependency cycles (strict improvements). But iterative-calc convergence and the
   external/rich event-production domains are explicitly deferred (§2). Confirm that boundary is where
   you want it, or pull a domain forward.
4. **The 14 open Excel-behaviour calls** from the corpus register — especially version targets (`@` on
   2D refs, INDIRECT-under-`@`, legacy CSE vs dynamic arrays, cross-workbook update-links gate,
   closed-workbook structured-ref values). Product/version decisions; they bear on WIRE-2/ERR-1, not
   on FEED-1/CTRO-3.

---

## 10. Residual uncertainties (re-verify at implementation)

- **Line-number drift.** A few spans were cited from a single read (`repair_optimized_spills_with_oxfml`,
  the warm-no-op token, the `recalculate_mark_all_dirty_compact_with_oxfml` family line, the
  `spill_epoch_change_kind` anchor-ignore at `optimized_sheet.rs:4156`). Structural claims are
  cross-confirmed; re-check exact lines before citing in code comments.
- **Topological order vs the optimized engine's bucket evaluation.** The optimized engine evaluates in
  three source-buckets (`optimized_sheet.rs:1912,1936`); the seed's topological order must be applied
  independently of bucket layout — confirm the optimized path can honor an externally-supplied order.
- **`request_key` call-site handle.** FEED-1 assumes OxFml's `normalized_references` exposes a stable
  normalized dynamic-call-site identity; **verify** before relying on it (BLOCKER-2 fix).
- **Dead-branch volatile re-fire.** Whether OxFml's `BranchLazy` executes a `NOW()`/`RAND()` in an
  untaken `IF` arm is unresolved; Excel re-evaluates the whole volatile formula regardless, so VOL-1
  treats any volatile-classified formula as recalc-forced — confirm the runtime path.
- **Generalized fixpoint termination.** Dropping the spill-presence gate and broadening the
  continue-condition to value/ledger change must still terminate; bound it by `formula_count` as today
  and add `DOR-FORWARD-REF` + a chained-INDIRECT cascade as termination fixtures.
