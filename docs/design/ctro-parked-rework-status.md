# CTRO parked rework status — structural graph plus calc-time overlay

Date: June 30, 2026

## Product status

The CTRO rework is parked at a usable implementation checkpoint. The live grid model is:

```text
effective calculation dependency graph
= structural/formula dependency graph
+ calc-time realized overlay graph
```

The old `rebind.rs` claim lifecycle is no longer the live API. It remains historical design material only.

The reference and optimized grid engines now have enough shared machinery to begin testing and putting the richer calculation model into use for the covered domains below. The main risk is no longer "the graph is unwired"; the risk is making sure hosts call the right report surfaces and do not assume deferred domains are complete.

## What is implemented

| Area | Parked status |
|---|---|
| Layered dependency graph | `GridInvalidationRef` carries explicit `Structural` and `CalcOverlay` layers. Effective dirty closure walks both layers. |
| Structural dependencies | Direct cell/range/name/table/spill-anchor/reference-metadata/axis dependencies install structurally. Direct formulas such as `B1=A5` do not create runtime overlay edges. |
| Runtime overlay dependencies | `INDIRECT`, `OFFSET`, dynamic text resolution, realized ranges, name identity, table identity, and runtime reference transforms install replacement-state overlay dependencies after evaluation. |
| Overlay replacement | Each evaluated formula replaces its previous overlay dependencies. Retargeting removes stale targets; runtime resolution errors clear stale overlay state. |
| Publication deltas | Formula publication returns changed/vacated effective cells, spill-fact anchors, blocker extents, and current blocker watches. Spill growth/shrink dirties ordinary cell/range consumers through the effective graph. |
| Effective-graph recalc | Reference and optimized dirty recalc close over structural plus overlay dependencies, choose ready formulas from the effective graph, replace overlays, publish values, and feed new dirty seeds back into the worklist. |
| Mark-all rebuilds | Reference mark-all uses the effective graph worklist. Optimized mark-all uses it for guarded sparse/small repeated cases and preserves compact fast paths where safe. |
| Volatility | OxFml semantic-plan volatility feeds runtime traces; `GridDirtySeed::Volatile` reaches volatile formula roots and volatile dynamic names. Warm-no-op rejects volatile cached roots. |
| External availability | Formula and dynamic-name traces can carry external-pending state and topic subscriptions. Recalc reports emit `GridExternalAvailabilitySubscriptionUpdate`; hosts can apply these to `GridExternalAvailabilityTopicRegistry`. |
| Dynamic defined names | Dynamic names evaluate through the same tracing path, keep a namespace-side formula-input dependency ledger, detect name cycles, participate in volatile/external roots, and emit name dirty seeds when realized extents change. |
| Names and tables | Static/dynamic defined-name create/redefine/rename/delete and table set/resize/rename/delete/recreate produce dirty seed reports consumed by dirty recalc. Missing name/table identity dependencies heal on later creation. |
| Axis edits | Structural dependencies and formula-root sets transform; calc-overlay realization state clears and is rebuilt by evaluation. Optimized valuations can be carried across axis edits with the same rule. |
| Cycles | Dirty recalc reports effective dependency cycles, including cycles discovered through runtime overlay edges. Stale overlay cycles can release when an overlay-bearing formula reevaluates first. |
| Differential oracle | `GridDirtyRecalcDifferentialRunReport` compares dirty recalc with mark-all on sampled values, spill facts, dependency graph state, dynamic-name runtime state, spill-epoch-ledger content, and a registry-effect projection (both runs' `external_subscription_updates` applied to seeded `GridExternalAvailabilityTopicRegistry` clones, compared by topic-root map; both production wrappers seed a real, non-vacuous empty registry rather than opting out). `is_clean()` folds in every axis. Each axis has at least one engine-run RED fixture that corrupts live state and proves the axis actually goes red. The prior opt-in full-computed-state address-set axis was removed: at both call sites it was always fed the exact same probe-derived readout the ordinary `mismatches` axis already compares, so it added no independent coverage. |

## Excel observation note: dynamic UDF dereference before target readiness

A local Excel/VBA probe checked:

```excel
A1: =MyFunc("A2")
A2: =1+1
```

where `MyFunc` evaluated the text reference `A2` internally and logged its observed value. Excel
`16.0 (build 20026)` called `MyFunc` once while `A2.Value2` / `Evaluate("A2")` was `<Empty>`, then
called it again after `A2` evaluated to `2`; the final visible value was `2`.

This supports the current effective-graph requeue shape: a caller can run early, discover a runtime
dependency, and be rerun after the discovered precedent publishes. It does not prove Excel's internal
commit/reject mechanics for the first invocation. A possible future refinement is an explicit
"provisional evaluation result" state for formulas that touched not-yet-ready formula precedents, so
the scheduler/publication layer can distinguish "UDF was invoked" from "cell is stabilized".

## Covered scenario families

These are ready for directed host testing:

| Family | Examples currently covered |
|---|---|
| Plain spill publication into ordinary refs | `A2=SEQUENCE(A1)`, `B1=A5`, `SUM(A5:A6)` growth and shrink. |
| Spill anchors | `SUM(A2#)` dirtying on shape/value/blocker changes, including a same-extent value-only change (`A2=SEQUENCE(3,1,A1)` editing `A1`'s start with the extent held fixed). |
| Runtime scalar refs | `INDIRECT(C1)` and `OFFSET(A1,N1,0)` retarget, release, and error, including `OFFSET` retargeting out of grid bounds to `#REF!` with the stale overlay edge cleared. |
| Runtime ranges | `SUM(INDIRECT(C1))` retarget from one realized range to another. |
| Branch-sensitive runtime refs | `IF` branch flips replace overlay dependencies rather than accumulating both realized targets. |
| Forward formula ordering | Formula chains whose precedent appears later in address order evaluate by dependency readiness. |
| Names | Static and dynamic names, scoped names, missing-name identity healing, name deletion vs empty target value. |
| Tables | Structured refs, missing table identity healing, table resize/rename/delete/recreate, table/spill blocker interaction. |
| Axis value/visibility | Whole-row/whole-column value dependencies and hidden-row-sensitive aggregates. |
| External roots | Manual broad external availability, topic-specific availability, RTD semantic topic projection, stale topic clearing, RTD's non-terminal stays-pending behaviour across an external-seed recalc (root remains pending, the topic:rtd subscription is re-carried, the published placeholder value stays pinned). |
| Warm-no-op safety | Volatile and external-pending formula/name roots reject optimized warm-no-op reuse. |
| Blast-radius bounding | Volatile and external dirty seeds are proven not to over-dirty an unrelated non-volatile/non-external formula (bounded `formula_evaluations`, and `visited_cells` exclusion on the reference engine). |
| Topic-registry conflict semantics | Equal-sequence/distinct-dedupe-identity envelopes both apply (last-applied wins the retained envelope); a redelivery reusing the current sequence's already-consumed dedupe identity is dropped, not re-applied. |

## How to put it into use

1. Build structural dependencies when formulas are installed or transformed.
2. Run dirty recalc with report-produced dirty seeds from edits, publication deltas, name/table lifecycle reports, axis edits, volatility, or external availability.
3. Treat runtime overlay dependencies as replacement state owned by evaluation, not as authored formula dependencies.
4. Apply `GridExternalAvailabilitySubscriptionUpdate` records from recalc reports to the host-owned `GridExternalAvailabilityTopicRegistry`.
5. Use `GridExternalAvailabilityEventReport` for broad availability flips and topic event/dispatch reports for recognized provider topics.
6. Use dirty-vs-mark-all differential runs as the first safety gate for new scenario families.

## Readiness checklist

Before enabling this model in a broader host path, verify each item explicitly:

| Check | Status |
|---|---|
| `cargo test -p oxcalc-core --lib` is green. | Current checkpoint: 915 passed, 2 ignored. |
| Host uses lifecycle-report dirty seeds instead of hand-built seeds for name/table mutations. | Required at integration boundary. |
| Host stores and applies external subscription updates after recalc. | Required for topic-specific external events. |
| Host sends broad external availability seeds for unknown/coarse availability flips. | Required when exact topic is unavailable. |
| Host treats optimized warm-no-op `None` as "must recalc", not as an error. | Required for volatile/external safety. |
| New scenario families are first compared dirty-vs-mark-all before relying on incremental results. | Required for confidence, and the oracle itself now has RED-fixture coverage proving each comparison axis actually detects a corrupted run rather than passing vacuously. |
| Volatile/external dirty seeds are blast-radius-bounded, not "dirty everything". | Verified directly: the volatile and external seed test fixtures include an unrelated non-volatile/non-external formula and assert it is not visited and the evaluation count stays bounded. |
| Cross-sheet, rich object, external exact-topic, and iterative-calc cases are not silently advertised as complete. | Required to avoid surprises. |

## Still open / refinements

| Refinement | Why it is still open |
|---|---|
| Exact external provider/workspace topic descriptors | Current RTD/external-provider projection is semantic-plan coarse (`topic:rtd`). Exact provider/topic argument extraction needs richer runtime effect data from the value/provider boundary. |
| Cross-sheet and 3D references | Current grid machinery is sheet-local for axis transforms and dependency graph addressing. Multi-sheet/workbook-level graph semantics need a separate design pass. |
| Rich objects and `FIELDVALUE` | The overlay identity model has no entity/schema/field dependency surface yet. This belongs with rich-object support. |
| Iterative calculation | Effective cycles are reported; Excel-style iterative convergence with max iterations/change is not implemented. |
| Provisional publication semantics | Excel UDF observation shows a dynamic text dereference can run once against an empty/stale formula precedent and then rerun after the precedent calculates. Current OxCalc requeues correctly for the covered cell-precedent class; a first-class provisional/non-stabilized publication state remains a possible scheduler refinement. |
| Legacy CSE arrays | Separate from dynamic-array spills and not covered by this checkpoint. |
| Exact Excel version switches | Some behaviours remain version/config sensitive and should be encoded in scenario fixtures before claiming conformance. |
| Performance hardening | Whole-row/column and large spill paths are compressed/guarded in key places, but broader workload profiling should precede scale claims. |
| API polish | The public host-facing surfaces may want naming/placement cleanup after first real integration use. The model should drive the cleanup, not backward compatibility with the removed claim lifecycle. |
| Per-call-site metadata-consumption intent | The metadata-vs-value overlay classification (`ROW`/`COLUMN`/`ROWS`/`COLUMNS` over a runtime-realizing call such as `INDIRECT`) is a formula-wide bound-tree approximation (`grid_formula_runtime_realized_dependencies_are_metadata_only` in `runtime_trace.rs`), conservative by construction. The precise fix needs OxFml to surface which argument position of which function actually triggered a given realized dependency. |
| R1C1/sheet-qualified spill anchors (`A1#`) | `OP_SPILL_REF` builds the spill-anchor reference from the anchor operand's flattened display text rather than routing back through the reference-system provider's compose path, so an R1C1-channel or sheet-qualified spill anchor can get a structural edge whose runtime deref can never succeed. The real fix needs a new OxFml `ReferenceComposeOperation` variant for spill anchors, analogous to `Range`/`Union` — a cross-crate OxFml + OxFunc protocol change. |
| Manual-vs-filter hidden-row visibility split | `GridAxisVisibilityDependency` dirties hidden-row-sensitive consumers uniformly on any row-visibility change; it does not distinguish manually-hidden rows from filtered-hidden rows at the dependency-graph level (the row-context aggregate report does track `manually_hidden_rows`/`filtered_hidden_rows` separately at read time, but that split does not yet feed dirty-seed granularity). |

## Verification commands

Use these as the basic parking gate:

```powershell
cargo test -p oxcalc-core --lib
git diff --check
```

Focused checks useful for this slice:

```powershell
cargo test -p oxcalc-core --lib reconciles_rtd_subscription_registry -- --nocapture
cargo test -p oxcalc-core --lib runtime_result_external_subscription_projects_rtd_topic -- --nocapture
cargo test -p oxcalc-core --lib rtd_formula_marks_external_pending_root -- --nocapture
```

Focused checks for the differential-oracle hardening (RED-fixture proofs that each comparison axis actually detects a corrupted run, not just that clean fixtures stay clean):

```powershell
cargo test -p oxcalc-core --lib goes_red_on_injected_bogus_overlay_edge -- --nocapture
cargo test -p oxcalc-core --lib rtd_root_stays_pending_after_external_seed_recalc -- --nocapture
cargo test -p oxcalc-core --lib reaches_spill_anchor_consumer_on_same_extent_value_change -- --nocapture
cargo test -p oxcalc-core --lib clears_overlay_when_offset_retargets_out_of_bounds -- --nocapture
cargo test -p oxcalc-core --lib grid_external_topic_registry_equal_sequence_distinct_dedupe_identity_both_apply -- --nocapture
cargo test -p oxcalc-core --lib grid_external_topic_registry_stale_consumed_identity_redelivery_is_dropped -- --nocapture
```

## Source map

| Surface | File |
|---|---|
| Runtime trace and external subscription projection | `src/oxcalc-core/src/grid/machine/runtime_trace.rs` |
| Dependency graph, dirty seeds, external registry, lifecycle reports | `src/oxcalc-core/src/grid/machine.rs` and `src/oxcalc-core/src/grid/machine/invalidation.rs` |
| Reference engine recalc path | `src/oxcalc-core/src/grid/machine/calc_ref_sheet.rs` |
| Optimized engine recalc path | `src/oxcalc-core/src/grid/machine/optimized_sheet.rs` |
| Optimized valuation carried state | `src/oxcalc-core/src/grid/machine/optimized_valuation.rs` |
| Dynamic-name and host-context support | `src/oxcalc-core/src/grid/machine/host_info.rs` |
| Active design narrative | `docs/design/ctro-recalc-integration-design.md` |
| Historical claim-lifecycle material | `docs/design/ctro-implementation-spec.md` |
| Pre-refactor scenario catalogue | `docs/design/calc-phase-dynamic-scenarios.md` |

## Parking note

This is a good stopping point for the rework: the old CTRO claim machinery has been removed from the live grid path, the corrected graph model has behavioral coverage in both engines, and the remaining items are named refinements rather than hidden wiring gaps. The differential oracle itself has now been hardened with spill-epoch-ledger content equality, a registry-effect projection wired to a real (non-vacuous) seed at both production call sites, and RED fixtures proving each axis actually detects a corrupted run rather than passing vacuously; the volatile/external blast-radius and the vacuous volatile-differential assertion identified in review have direct regression coverage. A subsequent fresh-eyes pass found and closed a same-extent spill-value-only convergence gap in the optimized engine's spill-repair loop, fixed an E3 dynamic-name realized-extent bug on failed INDIRECT text resolution, fixed a mark-all torn-state hole in the reference engine, and removed a full-computed-state comparison axis that had been wired identically to the existing sampled-mismatches axis at every call site and so added no independent coverage. The next round should either run host-level integration through these report surfaces or pick one deferred domain — per-call-site metadata intent, R1C1/sheet-qualified spill anchors, or manual-vs-filter visibility — and extend the same structural-plus-overlay model deliberately.
