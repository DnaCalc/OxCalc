# Calculation System Optimization — Grounding Map and Roadmap

Status: exploratory design/direction document (2026-07-11). Not a spec, not a workset.
Intended as the anchor for a future optimization workset family; candidate successor
worksets are named in §11. Companion registers: `CORE_ENGINE_GRID_PERF_REGISTER.md`
(counter gates P-00..P-28), `CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md` (staged lanes),
`W053_STAGED_CONCURRENCY_STAGE_2.md`, `CORE_ENGINE_HOST_WORKER_PASSIVITY_SPIKE.md`
(tree-lane perf rounds 1–2), `W062_D3_WORKBOOK_CALCULATION.md` §10 (concurrency prep).

Doctrine this document operates under, restated once:

1. **Correctness outranks speed.** Every optimization ships as a permanent pair with its
   reference twin, an Invariant Register row, and a falsifiable perf-register row
   (perf register §1). Nothing below proposes relaxing that.
2. **Gates are deterministic counters, never wall-clock** (perf register §1.3).
   Wall-clock is retained evidence with ref/optimized ratios. This document adds a
   *trend* layer on top (§9), not a wall-clock gate.
3. **OxCalc does not parse formulas to infer semantics.** OxFml owns parse/bind/eval;
   OxFunc owns kernels. Where the current code bends this (the R1C1 compiled-plan lane,
   §4.3), the long-term direction is to move compilation across the seam, not to widen
   in-engine parsing (§8.4).

---

## 1. Purpose

Map the calculation system end to end, record where time and memory actually go today,
and lay out the optimization program — execution-plan/template caching, multi-threading,
and workbook "compilation" to lowered representations — with the measurement
infrastructure and test scope each stage needs. The goal condition is an Excel
calculation rocket-ship: full-recalc and incremental-recalc performance competitive with
or better than desktop Excel at 1M–10M-cell scale, without giving up the determinism,
replay, and equivalence discipline that distinguishes this engine.

Reference point for "competitive": the perf register's external comparison row for
`boring-1Mx10` records Excel at ~37 B/cell and **0.57 s full recalc** of a 10M-cell
workbook. Our optimized lane already beats the memory number (~6.4 B/cell authored) but
has no wall-clock story yet, and the general (non-compiled-plan) evaluation path is
currently orders of magnitude away (§5.1).

---

## 2. System map

### 2.1 Crate topology and ownership

```
OxDoc  (xlsx file boundary; DocumentEvent stream; no calc semantics)
  └─► OxCalc oxdoc_ingest (sink; one-transaction Tier-A commit)
        └─► OxCalc oxcalc-core (dependency graph, invalidation, scheduling,
              publication, snapshots, TreeCalc + Grid engines)
              ├─ uses ─► OxFml oxfml_core (parse → bind → semantic plan →
              │            compiled plan → tree-walking evaluation; FEC/F3E seam)
              │            └─ uses ─► OxFunc oxfunc_core (525-function catalog,
              │                         kernels, CalcValue value model)
              └─ consumed by ─► DnaTreeCalc (WASM/Leptos host, optional web worker),
                                 DnaOneCalc, OxIde, ...
Evidence/measurement siblings: oxcalc-tracecalc(+cli), OxReplay (replay appliance),
OxXlPlay (real-Excel observation harness).
```

Key seam facts (verified in code, July 2026):

- **Consumer API is a synchronous pull facade** — `OxCalcDocumentContext`
  (consumer.rs:2414), `&mut self` verbs, no callbacks, no async. The engine is passive:
  all computation happens inside the verb call ("host worker passivity").
- **Values are `CalcValue { core: CoreValue, rich: Option<Rc<RichValue>> }`**
  (oxfunc_value_types/src/lib.rs:386). The `Rc` makes `CalcValue` `!Send`; this is the
  single named blocker for cross-thread evaluation (machine.rs:28128 Send audit).
- **Two calculation lanes** share OxFml/OxFunc leaves:
  - *Tree lane* (`LocalTreeCalcEngine::execute`, treecalc.rs:862): pure function of an
    owned input; prepare → lower → graph build → invalidation closure → sequential
    evaluation → publish. Pull-full-closure or push-visibility-bounded scheduling.
  - *Grid lane*: permanent engine pair — `GridCalcRefSheet` (reference oracle,
    calc_ref_sheet.rs:10) and `GridOptimizedSheet` (optimized_sheet.rs:11) — with a
    differential harness (`grid/machine/differential.rs`) comparing them.
- **Hosts pull deltas by epoch**: per-cell `value_epoch`, `poll_grid_changes(since)`
  (consumer.rs:5006), tree `published_value_epochs`. Publication is a monotonic
  per-grid sequence; unchanged values keep their epoch.

### 2.2 The grid recalc pipeline (optimized lane)

1. Edits mutate `Arc<GridInputState>` via copy-on-write and accumulate
   `GridDirtySeed`s (consumer.rs:1408, ~1475).
2. `recalc` mints one `WorkbookRecalcTick` per transaction (coherent NOW()/RAND) and
   asks for the incremental lane; it runs **only when** a retained valuation exists,
   basis stamps match, topology is preserved, coverage is full, and the graph is
   installed — otherwise full mark-all (`GridSeededLaneOutcome`,
   optimized_sheet.rs:2690–2717).
3. Dirty closure runs over the **effective graph = structural layer + calc-time
   realized overlay layer** (CTRO model; `GridInvalidationRef`, invalidation.rs:513).
   Ranges ≤100k cells are scalarized *and* kept as compressed edges; larger ranges are
   compressed-only, indexed by 1024×1024 blocks (invalidation.rs:739, machine.rs:253–254).
4. Scheduler repeatedly picks a ready dirty formula (`next_ready_dirty_formula`,
   invalidation.rs:2831), evaluates it, replaces its overlay edges from the runtime
   trace, publishes, and re-closes over publication-induced seeds until drained.
   Stalls trigger cycle DFS → error (no in-engine iterative calc yet; `IterationSettings`
   is ingested and SCC groundwork exists in workbook_coordinator.rs:1105).
5. Evaluation per formula: fast path first (compiled R1C1 plan, §4.3), else the general
   OxFml path (§5.1). Repeated-formula regions can evaluate as one region sweep
   publishing one dense payload (optimized_sheet.rs:3942).
6. Output is an immutable `GridOptimizedValuation` (sparse point map + dense computed
   regions + spill facts/epoch ledger + graph), retained for the next incremental pass.

Multi-sheet: cross-sheet closure is workbook-coordinated (`workbook_dirty_closure`,
workbook_coordinator.rs:564) but each affected sheet currently **escalates to
sheet-granular mark-all** per round (consumer.rs ~10455).

### 2.3 Storage model (optimized lane)

Hybrid sparse-point + region-run, not columnar, not chunked:

- Authored: `sparse_points: BTreeMap<GridCellCoord, …>` + `dense_value_regions`
  (packed `Vec<f64>` / `Vec<bool>` / RLE payloads behind `Arc`, COW-shared across
  revisions) + `repeated_formula_regions` (one `GridFormulaCell` per fill-down/CSE
  region) — optimized_storage.rs:12,105,303.
- Measured: ~8 B/cell dense numeric, 24 B/cell adversarial sparse singletons, 0 bytes
  for blanks, ~0.0003 B/cell amortized formula text for repeated regions (perf register
  §5). This layer is in good shape; the gap is CPU, not resident bytes.

---

## 3. What already exists on the performance front

Do not re-litigate these; build on them.

| Mechanism | Where | Status |
|---|---|---|
| Template prepare-once (P-11) | `normal_form_key` + `GridOptimizedFormulaPlanCache` | prepare_count == distinct templates on ~44 1M-cell workloads |
| Compiled R1C1 plans (P-14) | r1c1_plan.rs, 9 shape classes | hit-rate floors + persistent-cache round test |
| Compressed range edges (P-12/13) | invalidation.rs block index | 1M-range = 1 edge; indexed candidates |
| Warm no-op (P-19) | warm_no_op.rs | 0 cells visited on unchanged sheet — permanent tripwire |
| Occupancy-proportional enumeration (P-20) | optimized_storage.rs:470 | SUM(A:A) visits occupied, never 2^20 |
| COW retention (P-21), publication deltas (P-22), spill epochs (P-25/27), visibility index (P-24) | various | retained floors |
| Visible-first projection (P-16), tile streaming (P-15) | optimized_sheet.rs:3184 | upstream-cone-only viewport eval |
| Partition witness (P-18) | disjoint read/write rects per level + `partition_max_parallelism_bound` | **measured; parallel execution deferred** |
| Per-edge value cache + push/pull scheduling | tree lane, recalc spec §13.3–13.5 | O(k) trace-count evidence |
| Sparse range reader seam | sparse_reader.rs:148, OxFunc resolver `enumerate_values` | SUM sparse path proven non-dense |
| W075 evaluator optimization | OxFml eval hot loop | Mandelbrot 10 s → 1.41 s; scratch pooling, slot frames, constant folding |
| Concurrency prep | W062 D3 §10, Send audit machine.rs:28128 | 5 named constraints; `Rc` in CalcValue is the last blocker |

Tree-lane history (spike doc): chain n=1000 cold went 244 s → 2.80 s across rounds 1–2;
warm 0.54 s; incremental 0.48 s. Cold remains ~quadratic; n=5000 cold 64.6 s. The named
residuals in that doc are a ready seed list.

---

## 4. The calculation mechanism per formula — three tiers today

### 4.1 Tier 0: repeated-region sweep
`try_evaluate_repeated_formula_region_fast_path` (optimized_sheet.rs:3942) evaluates an
entire repeated-R1C1 region row-major against a growing accumulator and publishes one
dense payload. Covers fill-down recurrences without graph scheduling.

### 4.2/4.3 Tier 1: compiled R1C1 plans
`GridOptimizedCompiledFormulaPlan` (optimized_provider.rs:985) — hand-rolled compile of
a closed set of R1C1 shapes (scalar/binary/aggregate-range/IF/IFERROR/logical/
comparison/text/INDEX), no OxFml on the hot path, streaming aggregate state. Cached per
`normal_form_key` in `GridOptimizedFormulaPlanCache` (r1c1_plan.rs:2132).
Two caveats found in code:
- the live consumer path constructs the plan cache **fresh inside every recalc**
  (optimized_sheet.rs:2053/2075/2084) — cross-recalc persistence exists only in the
  harness entry point;
- the single-cell fast path recompiles the plan from text per call
  (optimized_sheet.rs:6083).

### 4.4 Tier 2: the general OxFml path — the dominating cost
For any formula outside the compiled-plan classes,
`evaluate_optimized_formula_with_oxfml` (optimized_sheet.rs:4473) builds per evaluation:
fresh providers, fresh `FormulaSourceRecord` (cloned text), fresh
`RuntimeEnvironment::new()` — and `RuntimeEnvironment::execute` builds a fresh
`SingleFormulaHost` whose caches are empty. Net effect, **per formula cell per recalc**:

- lex → parse → green-tree fingerprint → red projection → **bind ×2** (execute path
  computes prepared identity, then the host re-binds) → Debug-format-string bind hash →
  **semantic plan ×2** → compiled-plan build (`EvaluationContext::new`,
  eval/mod.rs:1709) → evaluation → candidate/commit surface construction;
- plus a second independent parse+bind for metadata dependency classification
  (`bind_grid_formula_for_transform`, optimized_sheet.rs:7829);
- plus a third parse+bind in structural dependency installation
  (`grid_structural_dependencies_for_formula`, runtime_trace.rs:333).

≈3 full front-end passes per cell per recalc, on both engines. OxFml has all the reuse
machinery (`CachedHostArtifacts`, `ArtifactReuseReport`, `RuntimeSessionFacade` keeping
a host alive) and the identity vocabulary for template sharing
(`FormulaTemplateIdentity` with caller-anchor-excluded fingerprints) — but **no template
store exists and the grid path can't reach the per-host caches**. This is the single
largest CPU multiplier in the system and the first target (§6, O-1).

---

## 5. Cost model — where time and memory go (ranked, with evidence)

### 5.1 CPU, steady-state recalc
1. **General-path OxFml re-preparation ×3 per cell per recalc** (§4.4). Everything else
   is noise until this is fixed for the non-R1C1-template population.
2. **Differential oracle on by default**: `GridDifferentialPolicy::EveryRecalc`
   (differential.rs:199) — every consumer recalc also runs a reference-lane mark-all
   and compares. Right default for this maturity phase; must become a dial with an
   explicit policy story before any wall-clock claim (§9.4).
3. **Ready-pick scheduling is O(P²)**: `next_ready_dirty_formula` linear-scans pending
   × `has_pending_precedent` per pick (invalidation.rs:2831–2876);
   `effective_precedents_for_layer` scans the candidate pool per compressed edge. No
   precomputed topological order, no in-degree counters.
4. **Per-evaluation provider/context rebuild**: reference-system provider re-clones
   cross-sheet cells, spill extents, names, tables per formula
   (calc_ref_sheet.rs:2599–2662); `format!` identity strings per evaluation.
5. **Per-call argument cloning in OxFunc**: every dispatch does `args.to_vec()`
   (surface_dispatch.rs:1396) — deep clones for text/arrays.
6. **Aggregate materialization**: SUM-family has a sparse path, but the dense fallback
   is 3 passes + 2 O(n) allocations (clone every cell → `Vec<f64>` → fold);
   SUMIF/COUNTIF-family **densifies** ranges via `materialize_resolved_reference_values`
   (criteria_family.rs:195) — a whole-column criteria range materializes 1,048,576
   `CalcValue`s regardless of occupancy.
7. **Wholesale clones per recalc**: incremental lane clones the entire previous
   valuation (optimized_sheet.rs:2085); reference lane clones `authored` per mark-all;
   block candidate buckets are cloned per query.
8. **Per-cell dependency install even for repeated regions** — full parse+bind and edge
   registration per member cell despite one shared template (optimized_sheet.rs:4013).
9. **Cross-sheet rounds escalate to per-sheet mark-all** (consumer.rs ~10455).
10. **Tree lane**: per-node overhead inside `EvaluationLoopTotal` dominates (4.6 s of
    5.0 s at n=5000); `CandidatePublication` still full-N; graph build 23.3 s of 33.6 s
    on the 1M-formula scale profile.

### 5.2 Memory / data layout
1. **String-pair address keys everywhere**: `ExcelGridCellAddress { workbook_id: String,
   sheet_id: String, row, col }` (coords.rs:52) is the key of every graph BTreeMap, edge,
   seed, publication map — two heap allocations + O(len) compares per node, cloned
   pervasively. (The authored store already uses compact `GridCellCoord`.)
2. **BTreeMap-everything**: ~25 parallel ordered maps per `GridDependencyIndex`; no
   compact adjacency, no arenas. Deterministic, but cache-hostile and allocation-heavy.
3. **Text values**: `ExcelText = Vec<u16>` per value, no interning. At xlsx ingest the
   shared-string table's dedup is **destroyed**: `SharedText(index)` → `.cloned()` →
   fresh UTF-8→UTF-16 re-encode **per cell** (oxdoc_ingest.rs:1250–1256). 1M cells
   sharing one string ⇒ 1M buffers.
4. **Load-time triple residency**: decompressed source package bytes (kept for
   round-trip) + full `DocumentEvent` vec + the calc model all live simultaneously
   after load (oxdoc-xlsx lib.rs:1780).
5. **`CalcArray` is `Vec<CalcValue>`** — boxed per cell, not packed; array/spill flows
   clone deep.
6. OxFml artifacts are string-heavy owned trees; all fingerprints are
   `format!("{:?}")` + DefaultHasher over the whole tree (binding/mod.rs:3474) — an
   allocation of the entire tree's debug rendering per bind.

---

## 6. Opportunity catalogue

Each row: what, expected effect, correctness posture, prerequisites. IDs `O-n` for
cross-reference; ordering within a theme is priority order. Every item lands under the
perf-register merge rule (twin + invariant row + counter gate).

### Theme A — Prepare-once / execution-plan caching (the biggest lever)

**O-1. Workbook-scoped prepared-formula store keyed by template identity.**
A persistent map `(FormulaTemplateIdentity, bind_context_fingerprint) → Arc<prepared
artifact>` (green + red + bound + semantic plan + compiled plan), owned at the
workbook/session level, consulted by both engines. OxFml already exposes the identity
vocabulary and per-host reuse; the work is (a) an OxFml consumer-interface entry that
accepts a template store or returns shareable prepared artifacts, (b) OxCalc holding it
in `GridDerivedState`/session, (c) eviction keyed to the snapshot-layer compatibility
rules (W057 vocabulary already defines the basis ids). Kills the ×1 of §4.4 for every
repeated template and makes single-cell edits O(dirty cone) in *evaluation* work, not
preparation work.
*Gate:* new counter `oxfml_front_end_passes` (parse/bind/plan counts) with floor
`== distinct (template, context) pairs`, mirroring P-11. *Risk:* low — values unchanged;
reuse must be equality-gated on prepare basis exactly as `PreparedFormulaRetention`
already does in the tree lane.

**O-2. Eliminate the redundant passes inside one evaluation.**
Three sub-items, independently landable: (a) make `RuntimeEnvironment::execute` reuse
the prepared identity computation for the host's bind instead of re-running it;
(b) derive structural dependencies and metadata classification from the **already
bound** artifact rather than re-parsing (`grid_structural_dependencies_for_formula`,
`bind_grid_formula_for_transform` should accept a `&BoundFormula`); (c) hoist
`compile_formula_for_evaluation` out of `EvaluationContext::new` into the cached
artifacts. Together with O-1 this takes 3 front-end passes → 0 on template hits.
*Gate:* same counter as O-1. *Risk:* low; pure plumbing, oracle-diffed.

**O-3. Persist the compiled-plan cache in the live path.**
Wire `GridOptimizedFormulaPlanCache` into `GridDerivedState` so the consumer path uses
the `_using_formula_plan_cache` entry instead of `default()` per recalc
(optimized_sheet.rs:2053–2084), and route the single-cell fast path through the cache
instead of recompiling (6083). The persistent-cache behavior is already tested in the
harness (P-14 rounds test); this is wiring, not new machinery.
*Gate:* P-14 extended to the consumer path: compiled-plan misses == first encounters
across recalcs. *Risk:* minimal; stale-fingerprint recompile + prune lifecycle already
has a unit floor.

**O-4. Region-granular dependency installation.**
For `GridRepeatedFormulaRegion`, install structural edges once per region template with
a relative-offset stamp instead of per-cell parse+bind+insert (optimized_sheet.rs:4013).
The compressed-range machinery already proves the graph can carry region-shaped facts.
*Gate:* new counter `structural_installs` floor == regions, not cells, on fill-down-1M.
*Risk:* medium — closure queries must expand region edges exactly like scalarized ones;
needs an expand-and-compare invariant row against per-cell installation.

**O-5. Prepared-artifact sharing for the tree lane.**
The tree lane already retains prepared formulas across runs; extend O-1's store to it
and attack the spike residuals (per-node overhead in `EvaluationLoopTotal`,
`CandidatePublication` full-N growth, quadratic cold build). The spike doc's residual
table is the work list.

### Theme B — Data layout and value model

**O-6. Compact address keys.**
Intern `(workbook_id, sheet_id)` to a `u32` pair (or one `u64` grid id) at the consumer
boundary; graph, seeds, valuation, and publication key on `(grid_id, row, col)` = 16
bytes, Copy, Ord. Strings live in one side table for rendering/replay. This touches a
lot of code mechanically but is semantically inert and oracle-diffable throughout.
*Gate:* bytes-per-edge counter; expand-and-compare between keyed forms. *Risk:* low
semantics / medium churn. Prerequisite for serious graph-scale work and for cheap
cross-thread message passing later.

**O-7. Graph structure evolution: BTreeMaps → indexed adjacency.**
Once keys are compact (O-6): dense `Vec`-indexed node table for occupied formula cells,
CSR-style adjacency for scalar edges, retaining BTree/interval structures only where
order queries matter (block index, rect queries). Determinism is preserved by explicit
sort-on-iterate at the few points order is observable (worklist selection, replay
artifacts) rather than by paying ordered-map costs on every insert.
*Gate:* edges-visited and bytes-per-edge counters vs the BTreeMap twin. *Risk:* medium;
this is exactly the kind of change the permanent-pair doctrine exists for.

**O-8. Text interning and shared-string preservation.**
(a) `ExcelText` gains a shared representation (`Arc<[u16]>` or interner id — note
`Arc` here is also the Send-migration direction); (b) xlsx ingest resolves
`SharedText(index)` to the *same* shared buffer instead of re-encoding per cell
(oxdoc_ingest.rs:1250); (c) OxDoc hands the string table across the seam once instead
of clone-per-event-stream + clone-per-sink.
*Gate:* P-10-style bytes/cell on a text-heavy 1M workload (new `text-heavy-1M` profile);
floor: unique-string bytes, not cell-count bytes. *Risk:* low-medium — equality/ordering
semantics of text values must be by content, already true.

**O-9. Packed arrays and spill payloads.**
`CalcArray` variants for homogeneous numeric/logical payloads (mirroring
`GridDenseValuePayload`) so SEQUENCE/FILTER/vector arithmetic don't box per element,
and spill publication moves one packed buffer end to end.
*Risk:* medium — touches OxFunc kernel expectations; needs the O-13 seam work to pay
off fully. Sequence: after O-13.

**O-10. Cheap fingerprints.**
Replace `format!("{:?}")`+DefaultHasher identity/hash construction in OxFml
(bind hash, semantic-plan key, lambda-body cache key) with structural hashing
(`Hash` impls or a dedicated visitor). Pure CPU win on every bind; also shrinks O-1's
miss cost. *Risk:* low; keys are internal.

### Theme C — Scheduling and invalidation

**O-11. Ready-queue scheduling (Kahn) instead of O(P²) scans.**
Maintain in-degree counters over the dirty cone; decrement on publish; pop from an
ordered ready set. Deterministic order preserved by ordered tie-break (address order),
which is exactly what the current linear scan yields. Compressed-edge precedents get
counted through the same block-index candidate query used today.
*Gate:* `edges_visited`/`ready_scan_steps` counter: O(dirty + edges), never O(P²);
permutation-pinned determinism test (already a W062 D3 constraint). *Risk:* low-medium;
scheduler equivalence is directly oracle-diffable.

**O-12. Early cutoff (verified-clean) in the grid lane.**
The recalc spec (§7.2) already commits to this direction and the tree lane has the
per-edge value cache. Grid realization: when a re-evaluated cell publishes an unchanged
value (epoch preserved — the machinery exists), do not seed its dependents. Requires
care with spill facts, volatiles, and overlay replacement, all of which have explicit
state today.
*Gate:* cells-evaluated == changed cone, not dirty cone, on a `diamond-cutoff-1M`
workload (new profile: wide fan-in where most inputs don't change the intermediate).
*Risk:* medium — this is semantically observable if done wrong; mark-all oracle catches
it, and the CTRO fresh-eyes suite (921 tests) is the regression net.

**O-13. Streaming/columnar aggregate seam (OxFunc).**
Extend `ReferenceSystemProvider` with a sanctioned bulk variant — e.g.
`enumerate_numeric_slices` yielding `&[f64]` runs (dense regions) + sparse remainder —
and teach the aggregate/criteria families to consume it. Fixes SUMIF densification
(1M+ `CalcValue`s for whole-column criteria) and SUM's clone-then-collect double pass.
This is charter-aligned: it generalizes the existing sparse seam rather than adding
per-function fast paths, and the provider already has the dense-region knowledge.
*Gate:* P-20-style `slots_visited` + new `values_materialized` counter (== occupied,
never extent; == 0 CalcValue clones on packed paths). *Risk:* low-medium; reduction
order must remain `SequentialLeftFold` (correctness floor) — slice iteration in
row-major order is identical by construction.

**O-14. Cross-sheet incremental closure.**
Replace per-sheet mark-all escalation with cross-sheet dirty cones through
`WorkbookCrossSheetEdges` (the reverse index exists). *Gate:* cells-evaluated across a
two-sheet chain == cross-sheet cone. *Risk:* medium; workbook oracle twin exists
(`GridCalcRefWorkbook`).

**O-15. Fix known closure inefficiencies.** Spill-blocker closure scalarizes up to 100k
cells per seed (invalidation.rs:3468); block candidate buckets cloned per query;
`visited_cells: Vec` per pass. Small, local, counter-gated.

### Theme D — Multi-threading (Stage 2 realization)

The spec commitments already exist (coordinator spec §10, W053, W062 D3 §10). What this
document adds is sequencing and the concrete blocker list. See §7.

### Theme E — Ingest/load and memory residency

**O-16. Streaming ingest.** OxDoc parses every sheet eagerly into one `Vec<DocumentEvent>`
before the sink sees anything. Stream chunks to the sink per sheet band instead;
peak residency drops from (source + events + model) toward (source + model).
*Risk:* low; the sink is already accumulate-then-commit.
**O-17. Parallel sheet parse in OxDoc.** OxDoc has no `!Send` entanglement — sheets are
independent XML parses; this can thread long before the engine can. Cold-load latency
win on multi-sheet books. *Risk:* low (OxDoc-local; deterministic merge by sheet order).
**O-18. Drop or demote `source_image` retention** behind a load profile for
calculation-only sessions (keep for round-trip sessions).
**O-19. Use `CalcChainHint`.** Currently Tier-X dropped. As a *hint* it can seed initial
evaluation order / partition layout for the first cold recalc without ever being trusted
for correctness (the graph remains truth). Cheap cold-start win on real corpus files.

### Theme F — Differential and host economics

**O-20. Differential policy dial with staged defaults** (§9.4): `EveryRecalc` in tests
and canary sessions; `Sampled{one_in}` in interactive sessions above a size threshold;
always-on for mark-all escalations. The oracle stays product code; only the *when*
changes, and it stays replay-visible.
**O-21. Host delta consumption.** The engine's epoch-delta machinery is ahead of the
shipping host (workbook dispatcher republishes full snapshots, ignores windowed
interest). Not OxCalc work, but the measurement story (§9) should include end-to-end
frame cost so engine wins aren't masked by host costs.

---

## 7. Multi-threading roadmap

Everything below preserves: single coordinator/publication authority, deterministic
replay, the passivity doctrine (all parallelism joins before the verb returns), and
byte-identical published artifacts vs the sequential twin.

### 7.1 Prerequisites (strict order)

1. **`Rc` → `Arc` in the value model** (`CalcValue.rich`, `CallableValue.handle`,
   OxFml `Rc<CompiledExpr>`/frame internals). This is the named last Send blocker
   (machine.rs:28128 audit doc). Precedent: the CalcValue migration itself was a
   successful big-bang mechanical change across ~250 files. Uncontended `Arc` clone is
   an atomic inc — measurable but small; measure on the W075 Mandelbrot lane before/after
   to bound the sequential regression (budget: <3%).
2. **Kill the remaining `RefCell`/`thread_local` on the eval path**: owned per-evaluation
   trace buffers (W062 D3 §10 item 1), tick passed explicitly instead of
   `with_treecalc_recalc_tick` thread_local (treecalc.rs:103) — the deterministic
   seeded-volatile design (tick + per-node RAND keying) already makes results
   order-independent, which is the hard part, done.
3. **Complete the Send audit** to cover providers: `ReferenceSystemProvider` and friends
   need `Send + Sync` bounds (or per-thread construction, which the tracing-provider
   docs already anticipate — runtime_trace.rs:2150).
4. **O-11 ready-queue scheduler** — the frontier structure is what workers consume.

### 7.2 Execution model (Stage 2, per W053, made concrete)

- **Unit of parallelism: partition, not cell.** P-18 already computes same-level
  partitions with disjoint read/write rects and a `partition_max_parallelism_bound`.
  Workers evaluate partitions against an immutable pre-recalc snapshot + published-value
  table (W062 D3 §10 item 4: pure providers); each worker fills a private candidate
  buffer; the single coordinator commits buffers in deterministic partition order.
  Determinism holds because commit order is data-independent of completion order.
- **Level-synchronous first** (barrier per topological level), then relax to
  frontier-work-stealing once contention evidence justifies it. Level-sync is trivially
  deterministic and already matches the witness structure.
- **Speculative lanes** (W053's input-binding fingerprints + commit-time checks) are
  *after* non-speculative partitioned parallelism proves out. Do not start there.
- **Volatiles/host-serialized formulas**: OxFml already computes
  `ExecutionContract { lane_class: ConcurrentSafe | Serialized, … }`
  (scheduler/mod.rs:49) per formula — the scheduler routes `Serialized` and
  caller-context formulas to the coordinator thread's own lane. This contract exists
  today and is unconsumed; Stage 2 is its first consumer.
- **Runtime targets**: native (Tauri desktop, server) gets real threads; wasm32 stays
  sequential until a wasm-threads (SharedArrayBuffer) build is justified — the
  browser host's worker split already moves work off the UI thread, which is the UX
  bar there. The engine API must not fork: same verbs, thread-count is engine config.

### 7.3 Determinism obligations (new invariant rows)

- I-PAR-1: published values, epochs, diagnostics, and replay digests byte-identical
  across thread counts {1, 2, 8} and across repeated runs at fixed count.
- I-PAR-2: same-level partitions' write sets disjoint (P-18 promoted from witness to
  enforced runtime check in debug/differential builds).
- I-PAR-3: reduction order unchanged — parallelism never splits a single formula's
  argument reduction (until a `PairwiseTree` profile is *deliberately* introduced as a
  separate declared profile with Excel-comparison evidence).
- Contention/retry outcomes replay-visible (coordinator spec §10 obligation).

### 7.4 What multi-threading is *not* for

Fixing §5.1 items 1–5 first is mandatory sequencing: parallelizing a path that spends
its time re-parsing text and cloning BTreeMaps would burn cores to hide waste, and the
speedup would evaporate as the single-thread path improves. Threads come after
prepare-once (Theme A) and scheduling (O-11) land; that is also when partition-level
work units become large enough relative to coordination overhead.

---

## 8. "Compiling" a workbook — lowered representations

Direction, in increasing ambition. Each tier is a separate declared execution lane with
its own equivalence row; the interpreter twin remains product code forever.

### 8.1 Tier A (exists): shape-class compiled plans
The R1C1 plan lane. Extend coverage along the perf-register workload matrix
(remaining classes: date/time scalars, more text functions, SUMIF/COUNTIF over runs,
bounded XLOOKUP/HLOOKUP, IS* predicates). Each new class is cheap and counter-gated;
this is steady background work, not the main thrust.

### 8.2 Tier B: region vector programs
For repeated-formula regions whose template compiles, lower the *region* (not the cell)
to a vector program over dense payloads: `out[i] = in_a[i] op in_b[i]` sweeps,
prefix-scan for running aggregates (`=R[-1]C+X` recurrences, SUM windows), masked
select for IF trees over packed logicals. This is where SIMD lives naturally
(`f64x4`/autovectorized slices), works per-region without any threading, and composes
with threading later (partition = region slice). The repeated-region sweep (Tier 0) is
the scaffold; the change is operating on `Vec<f64>` payloads directly instead of
per-cell `CalcValue` round-trips.
*Order-of-magnitude potential on dense models: memory-bandwidth-bound instead of
interpreter-bound.* Reduction-order discipline: within-formula reduction stays
sequential per I-PAR-3; element-wise maps have no reduction question.

### 8.3 Tier C: workbook dataflow programs
Whole-workbook lowering: dependency graph + templates + regions → a scheduled dataflow
program (topologically ordered list of region ops and scalar ops with buffer
assignments), cached against the snapshot basis tuple, re-lowered only on
structural/template change. An edit executes: patch input buffer → run the program's
dirty suffix. This is the "calc chain as compiled artifact" idea done honestly — the
graph remains truth; the program is a derived artifact with a declared compatibility
basis (exactly the W057 snapshot-layer pattern, and the same invalidation discipline as
the plan cache). Cold-load cost amortizes across the session; serialization of the
lowered program alongside the workbook is a *later* possibility once the basis-stamp
story is proven (it is a pure cache: safe to drop, never trusted).

### 8.4 Doctrinal note on who owns compilation
Tier A lives in OxCalc and parses normalized R1C1 text — sanctioned but in tension with
"OxCalc never parses formulas." The durable shape: **OxFml exposes the lowered form**
(it already has `CompiledExpr`, constant folding, and the metadata) — e.g. a
`LoweredTemplatePlan` handed across the seam per template identity, which OxCalc then
instantiates over regions/offsets. Tier B should be built on that seam from the start
rather than growing a second parser in OxCalc. This also gives the tree lane the same
lowering for free. Requires an OxFml handoff packet (FEC/F3E co-definition rule).

### 8.5 Explicitly not pursued (recorded so it isn't re-litigated)
- JIT to machine code: unjustified complexity/portability cost while Tier B headroom
  exists; revisit only with evidence Tier B is instruction-bound, not bandwidth-bound.
- Parallel/pairwise numeric reduction as default: changes Excel-visible float results;
  only ever as a separate declared profile (`NumericalReductionPolicy` variants are
  already reserved and deliberately deferred).
- GPU offload: wrong maturity stage; nothing in the model precludes it later for
  Tier B kernels.

---

## 9. Measurement infrastructure requirements

What exists: counter-gated scale harness (58 grid profiles, 1M-cell scale, 75 retained
runs), `counter_summary.json`/`register_assertions.json`, tree-lane phase timers, one
manual wall-clock spike harness, counters-not-wall-clock doctrine. What's missing, in
priority order:

**M-1. Wall-clock trend layer (not gates).** A bench runner (criterion or the existing
scale runner with a `--timed` flag) that runs a pinned subset of profiles
(boring-1Mx10, fill-down-1M, enron-mix, sum-pyramid-1M, plus a real-corpus set) on a
**named baseline machine + pinned toolchain**, N≥5 iterations, and appends
`{commit, profile, engine, phase, p50, p95, ratio_vs_ref}` to a checked-in
`docs/test-runs/core-engine/wall-trend/TREND.jsonl`. A tiny reporter renders the trend
and flags >15% p50 regressions as *advisory*. This answers the perf-register open
question 2 (named machine: yes) without violating the no-wall-clock-gates rule.

**M-2. Phase timers in the grid lane.** The tree lane has `phase_timings_micros`; the
grid lane has none. Add the same wasm-safe phase timer to both grid engines:
`seed_closure / schedule / prepare / evaluate / publish / differential`, reported in
run summaries and the spike harness. Without this, Theme A/C wins can't be attributed.

**M-3. Front-end work counters (the O-1/O-2 gates).** `CalcRunCounters` gains:
`oxfml_parse_count`, `oxfml_bind_count`, `oxfml_plan_compile_count`,
`provider_builds`, `values_materialized` (O-13), `structural_installs` (O-4),
`ready_scan_steps` (O-11). Counter-fidelity rule applies: each asserted against a
closed-form expectation on at least one workload. Recommend compiling counters into
release permanently (perf-register open question 3: yes — they are cheap adds and the
doctrine depends on them).

**M-4. Allocation and peak-memory evidence.** (a) A counting global allocator behind a
feature flag for scale runs (`alloc_count`, `alloc_bytes`, `peak_bytes` per phase);
(b) dhat-style heap profiling as a documented workflow for investigations. Bytes/cell
counters cover authored storage; they do not see transient churn, which is where §5.1
items 4–7 live.

**M-5. Real-corpus harness (the anchor the user asked for).** A `grid-scale`-style CLI
profile that ingests a directory of real `.xlsx` files through the OxDoc → ingest →
recalc path and emits per-file: load phase timings, cell/formula/template counts,
template-dedup ratio, counter summary, full-recalc + single-edit-incremental timings,
and differential verdicts. Corpus tiers: (T1) checked-in small redistributable files;
(T2) generated-from-statistics (enron-mix generator — build it; the register already
cites ~4.5% unique-formula ratio); (T3) local-only real corpora (Enron spreadsheet
corpus, EUSES/FUSE sets, owner's own books) referenced by path, never committed.
Corpus runs produce the same retained-run artifacts as scale runs.

**M-6. Excel comparison legs via OxXlPlay.** The pipeline of record exists. Need: the
hand-authored boring-1Mx10 workbook (perf-register open question 5 — do it now, don't
wait for the export path), timed F9 captures on the baseline machine, and value-parity
capture on the T1/T2 corpus so performance work and correctness evidence ride the same
artifacts.

**M-7. CI floor.** There is no CI. Minimum: fmt + clippy + workspace tests + smoke-scale
counter gates (`--engine both` at ≤1e5 cells) on push; the M-1 trend run nightly on the
baseline machine. Without this, the counter-gate doctrine is manually enforced.

**M-8. Profiling workflow doc.** One page: how to flamegraph a scale profile on Windows
(e.g. `cargo flamegraph`/ETW/superluminal), which profiles to use per subsystem, where
to file findings (perf-register row or bead). Keeps investigation results comparable.

---

## 10. Test scope requirements

Beyond what exists (1,264 unit tests, TraceCalc conformance, retained failures,
differential twins, CTRO suite at 921):

**T-1. Determinism/permutation suite (pre-threading).** Pin that recalc output digests
are invariant under: seed insertion order, BTreeMap→ready-queue scheduler swap (O-11),
thread counts {1,2,8} (post-threading), and repeated runs. W062 D3 already requires the
permutation-pinned worklist test; generalize it into a suite that every Theme C/D PR
must extend.

**T-2. Equivalence-at-scale for every new lane.** Each O-item's twin comparison must run
at 1M scale in its retained run, not only at unit scale — the perf register's
"production invalidation equivalence still open" rows show where unit-scale-only
evidence currently stands in for scale evidence.

**T-3. Corpus value-parity floor.** For T1/T2 corpus files: full-recalc values diffed
against (a) file-cached values on load, (b) reference lane, (c) Excel captures where
OxXlPlay evidence exists. Tolerance policy per the correctness floor / comparison views
(exact for exact lanes; documented ULP classes where Excel itself is imprecise —
KNOWN_EXACTNESS_DEVIATIONS.md is the vocabulary).

**T-4. Volatile/host-serialized semantics under new schedulers.** NOW/RAND coherence per
tick, single-flight lanes, RTD stream selectors — a focused suite that runs under
sequential, ready-queue, and (later) partitioned execution with identical published
artifacts.

**T-5. Cache-invalidation adversarial tests for O-1/O-3.** Template store poisoning
attempts: same text different bind context (sheet-scoped names, table renames, locale),
formula edit → stale plan, name/table identity churn, cross-workbook same-key
collisions. The stale-fingerprint recompile unit floor is the seed; grow it into a
generated matrix.

**T-6. Memory-bound regression tests.** Extend P-10-style byte assertions to (a) the
text-interning lane (O-8), (b) transient allocation counts per phase (M-4) with
closed-form floors on at least boring-1Mx10 and text-heavy-1M.

**T-7. Long-session soak.** Interactive-pattern replay (edit storms from recorded intent
logs via OxReplay) with retained-revision pressure, candidate open/close cycles, and
plan-cache eviction — asserting bounded memory (W054 vocabulary) and no epoch/basis
drift. This is where cache lifetime bugs live.

---

## 11. Sequenced roadmap

Phases gate on evidence, not calendar. Each phase's exit produces retained runs.

**Phase 0 — See clearly (measurement first).**
M-2 grid phase timers, M-3 front-end counters, M-1 trend layer, M-5 corpus harness
(T1 + enron-mix generator), M-6 boring-1Mx10 Excel workbook + timed capture, M-7 CI
floor. Exit: a baseline TREND.jsonl and corpus report on the named machine, with the
front-end pass counts made visible (they will be ugly; that's the point).

**Phase 1 — Stop re-doing work (Theme A + quick wins).**
O-3 (wire persistent plan cache — days, not weeks), O-2 (dedupe passes), O-1 (template
store), O-4 (region-granular installs), O-10 (cheap fingerprints), O-20 (differential
dial). Exit gates: `oxfml_front_end_passes == distinct templates` on corpus files;
corpus full-recalc and incremental wall-clock trend steps down; all equivalence suites
green. Expected effect: this phase alone likely changes general-path recalc by an order
of magnitude on template-heavy real workbooks (typical unique-formula ratios ~5%).

**Phase 2 — Right-shaped data and scheduling (Themes B/C).**
O-6 compact keys → O-11 ready queue → O-7 graph layout → O-8 text interning →
O-13 columnar aggregate seam → O-12 early cutoff → O-14 cross-sheet cones → O-15.
Ingest lane in parallel: O-16..O-19. Exit: ready_scan_steps linear; SUMIF whole-column
`values_materialized == occupied`; text-heavy-1M bytes floor; cutoff workload floor.

**Phase 3 — Threads (Theme D / W053 realization).**
Rc→Arc migration (with the ≤3% sequential-regression budget measured), provider bounds,
tick plumbing, then level-synchronous partitioned execution on native, I-PAR rows
enforced. Exit: boring-1Mx10 and corpus full-recalc scaling curves at {1,2,4,8} threads
with byte-identical artifacts; ratio targets set from Phase 2 baselines (provisional
ambition: ≥3× at 4 threads on dense corpus books).

**Phase 4 — Compile it (Theme 8 Tiers B/C).**
OxFml `LoweredTemplatePlan` seam → region vector programs over dense payloads →
workbook dataflow program with basis-stamped caching. Exit: dense-model full recalc
memory-bandwidth-bound (measure: bytes moved / wall time vs machine STREAM number);
boring-1Mx10 full recalc target: **beat Excel's 0.57 s on the baseline machine,
single-threaded; then again at 4 threads by ≥3×.**

Ambition anchors (provisional targets to confirm with the owner once Phase 0 baselines
exist — falsifiable numbers, deliberately aggressive):

| Scenario | Target |
|---|---|
| boring-1Mx10 full recalc, 1 thread | < 0.5 s (Excel ≈ 0.57 s) |
| boring-1Mx10 full recalc, 8 threads | < 150 ms |
| Single-cell edit → publish, 1M-cell book, warm | < 1 ms end-to-end in-engine |
| Real corpus (T2 enron-mix 1M cells) full recalc | < 1 s |
| Cold load 100 MB xlsx → first full calc | < 5 s, peak RSS < 3× file size |
| Warm no-op | stays exactly 0 cells visited (P-19, forever) |

---

## 12. Open questions (owner input wanted)

> **Decision log (2026-07-11, owner):**
> 1. **Differential dial (O-20): decided.** Staged dial, restructured so there is NO
>    silent default that runs the dual-engine oracle: the validation spend becomes an
>    explicit, mandatory profile choice with intent-revealing names
>    (`DualValidated` / `DualValidatedSampled` / `OptimizedOnly`), specified at
>    consumer-context construction — `Default` is removed. Dual validation stays the
>    explicit setting for suite/CI/corpus; interactive hosts choose consciously.
> 2. **Baseline machine (M-1): decided.** The current Windows machine is the named
>    Windows baseline for now. No nightly runs yet — corpus exploration outranks fine
>    timing at this stage. Noted for later: evaluate GitHub-hosted/other runners, and
>    the owner's Linux VM as a perf-load + nightly-CI host (no Excel oracle there);
>    revisit once the corpus harness (M-5) produces its first retained runs.
> 3. **Template-store scope (O-1): decided.** Per-workbook. The workbook is a calc
>    context boundary and stays one: defined names, and workbook-scoped VBA UDF
>    namespaces, share that boundary (normal .xll/.xlam add-in UDFs are global scope —
>    a future bind-context fingerprint concern, not a store-boundary one).
> 4. **Sequencing: delegated.** O-2 (pass deduplication) before O-1 (template store).

1. **Differential default trajectory (O-20):** confirm staged policy — `EveryRecalc`
   until Phase 1 exits, then sampled-by-default above a size threshold with always-on
   in CI/canary? The oracle-always-on posture is currently the de facto perf ceiling.
2. **Template store ownership:** per-workbook, per-session, or process-global with
   scope-keyed entries? (Interacts with candidate overlays and the W054 retention
   worksets; recommendation: session-owned, scope-keyed, W054-classed.)
3. **Rc→Arc timing:** land as its own big-bang workset early in Phase 3, or
   opportunistically during Phase 2's value-model touches (O-8/O-9)? Recommendation:
   own workset, one commit family, measured before/after — same playbook as the
   CalcValue migration.
4. **Tier B lowering seam:** agree that region vector programs are built on an OxFml
   lowered-plan handoff (§8.4) rather than extending the in-OxCalc R1C1 compiler beyond
   its current closed set — this needs an FEC/F3E handoff packet and OxFml buy-in.
5. **Corpus acquisition:** which real corpora are licensed/available for T3 (Enron
   spreadsheet set, EUSES/FUSE, internal books), and what redaction bar applies before
   any T1 check-in?
6. **Baseline machine:** nominate the named machine + pinned toolchain for M-1/M-6 now
   (perf-register open question 2 is resolved "yes, named machine" by this document
   pending owner confirmation).
7. **Wasm threads:** is a SharedArrayBuffer build in DnaTreeCalc's future, or is native
   (desktop/server) the only Stage-2 parallel target for now? Affects nothing before
   Phase 3.

---

## Appendix A — Evidence index (file:line anchors for §5 claims)

| Claim | Anchor |
|---|---|
| 3× parse/bind per general evaluation | optimized_sheet.rs:4473–4594, 7829; runtime_trace.rs:333; calc_ref_sheet.rs:2680–2745 |
| Fresh RuntimeEnvironment/host per cell | optimized_sheet.rs:4541; consumer/runtime/mod.rs:205–211, 555–612 (OxFml) |
| Plan compile inside EvaluationContext::new | oxfml eval/mod.rs:1709 |
| Plan cache fresh per recalc in live path | optimized_sheet.rs:2053, 2075, 2084 |
| Single-cell fast path recompiles | optimized_sheet.rs:6083 |
| Differential EveryRecalc default | grid/machine/differential.rs:191–201 |
| O(P²) ready pick | invalidation.rs:2831–2876, 2915–2970 |
| String-pair address keys | grid/coords.rs:52; grid/geometry.rs:70 |
| Valuation clone per incremental recalc | optimized_sheet.rs:2085 |
| args.to_vec() per dispatch | oxfunc surface_dispatch.rs:1396–1398 |
| SUMIF densification | oxfunc criteria_family.rs:195; resolver.rs:526 |
| SUM triple pass | oxfunc sum.rs:58–78; adapters.rs:68–284 |
| Shared-string dedup loss at ingest | oxdoc_ingest.rs:1250–1256 |
| Triple residency after load | oxdoc-xlsx lib.rs:1459–1780 |
| Debug-string fingerprints | oxfml binding/mod.rs:3474; semantics/mod.rs:277–294 |
| Send audit + Rc blocker | machine.rs:28128–28199; oxfunc_value_types lib.rs:386, 724 |
| thread_local recalc tick | treecalc.rs:103–127 |
| ExecutionContract (unconsumed scheduler metadata) | oxfml scheduler/mod.rs:49 |
| Partition witness, parallel deferred | perf register P-18; optimized_sheet.rs:1448 |
| Cross-sheet mark-all escalation | consumer.rs ~10455 |
| Spill-blocker scalarized closure | invalidation.rs:3468–3487 |
| Tree-lane spike numbers | CORE_ENGINE_HOST_WORKER_PASSIVITY_SPIKE.md rounds 1–2 |
| Excel boring-1Mx10 comparison | CORE_ENGINE_GRID_PERF_REGISTER.md §5 workload table |

Line numbers are July 2026 working-tree observations (W062 in flight) and will drift;
symbol names are the durable pointers.
