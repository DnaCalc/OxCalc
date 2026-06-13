# CORE_ENGINE_GRID_PERF_REGISTER

Status: **Promoted active planning register** (2026-06-13). Canonical OxCalc register for grid-lane performance claims and retained counter gates. Companion to the Invariant Register in CORE_ENGINE_GRID_REFINEMENT_AND_EQUIVALENCE.md.

## 1. Doctrine

1. **Every optimization ships as a permanent pair.** The reference twin (GridCalc-Ref, the
   scalarizer lowering, eager renumbering, per-cell reverse edges, per-cell prepare, map-based
   storage) is product code selected at runtime (`--engine reference|optimized|both`), never
   scaffolding. A differential run is one process.
2. **Merge rule:** no optimization lands without (a) an Invariant Register row (equivalence
   oracle, expand-and-compare against its twin) and (b) a row here with a falsifiable claim.
3. **Gates are deterministic counters, never wall-clock.** Bytes/cell, prepare count, cells
   evaluated, edges visited, slots visited, bytes/frame. This reconciles per-PR gating with
   DnaTreeCalc TECHNICAL.md §7.6 ("no clock-time success gates") and the documented ±25%
   wall variance. Wall-clock is recorded evidence in retained baselines only, always with
   ref/optimized ratios alongside absolutes.
4. **Counter fidelity:** every counter is asserted against a closed-form expectation on at
   least one workload, so a silently broken counter fails validation, not just gating.
5. **The reference twin is gated too:** naive in structure, not constants — it shares the leaf
   evaluation stack (OxFml/OxFunc) with the optimized engine and differs only in machinery.
   Its own row (P-00) keeps it usable as an oracle.

## 2. Row schema

| Field | Content |
|---|---|
| id / invariant | `P-xx` ↔ `I-xx` cross-link (mandatory both ways) |
| claim | complexity + constant, falsifiable (e.g. "≤17 B/cell on boring-1Mx10") |
| ref twin | which paired implementation expands/checks it |
| workload(s) | named ids from §5 |
| measured | ref number, optimized number, ratio, retained run id |
| budget | counted regression bound (never wall-clock) |
| status | claimed → measured → **bound** (counter gate live) → retired |

## 3. Seeded rows — calc-perf round 1 (tree lane, merged to main)

> To absorb verbatim from `CORE_ENGINE_HOST_WORKER_PASSIVITY_SPIKE.md` (fix table and
> measured numbers) on owner confirmation of the one-register decision. Skeleton:

| id | claim (summary) | evidence | status |
|---|---|---|---|
| P-01 | edge-cache warm-path cost fix (round-1 fix 1, commit `1955c8d`) | spike doc fix table | measured |
| P-02 | diagnostic-seed scaling fix (round-1 fix 2, commit `6a2cca0`) | spike doc fix table | measured |
| P-03 | consumer clone/retention fix (round-1 fix 3, commit `aa8eb26`) | spike doc fix table | measured |
| P-04 | layer-snapshot digest fix — warm-recalc OOM root cause (commit `64e144f`) | spike doc + OOM analysis | measured |

Round-2 residuals (incl. `CandidatePublication` growth and w056 O(n²) diagnostics) enter as
open rows when round 2 is scoped; P-22 below sidesteps the publication residual structurally
for the grid lane.

## 4. Open rows — grid lane catalogue

Invariant cross-links (I-xx) pend the equivalence doc; the proposed sets for spill
(I-SP1..I-SP5: bodies never authored; ledger ⇔ computed support; arbitration determinism;
quiesced no-edit extent stability; scalarizer equivalence) and visibility (I-VIS-1..3: exact
toggle closure; toggle storms change no insensitive value; manual/filter provenance
separation) were distilled from the archived DnaTreeCalc grid recon notes and are now owned by `CORE_ENGINE_GRID_REFINEMENT_AND_EQUIVALENCE.md`.

| id | claim | workload(s) | counter (vs ref twin) | status |
|---|---|---|---|---|
| P-00 | GridCalc-Ref evaluates each occupied cell exactly once per recalc (visited/uptodate discipline); ref scaling curve retained as the first baseline | all | cells-evaluated == occupied | claimed |
| P-10 | blocks: ≤17 B/cell dense, ≤85 B/cell adversarial singletons, blank cells = 0 bytes | boring-1Mx10, zig-zag-1M, full-column-1M | bytes-by-layer vs map-of-CalcValue (>200 B/cell) | claimed |
| P-11 | template prepare-once: prepare_count == distinct templates, not cells | fill-down-1M, enron-mix | prepare counter vs per-cell prepare | claimed |
| P-12 | rect propagation + interval index: invalidation O(dirty-rects·log n + consumers) | edit-storm, sum-pyramid-N | edges visited + seeds vs per-cell reverse-edge expansion | claimed |
| P-13 | compressed reverse edges (FAP/TACO-style): support bytes O(regions); queries ≡ expanded graph | fill-down-1M, sum-pyramid-N | graph bytes + expand-and-compare | claimed |
| P-14 | plan-cache hit rate ≥ (cells − templates)/cells at steady state | recalc-rounds amplification | hit/miss counters | claimed |
| P-15 | tile streaming: bytes/frame ≤ k·subscribed-cells, independent of model size and change count | doom-320x200 | bytes/frame | claimed (reserved until tile protocol exists) |
| P-16 | visible-first: cells evaluated before P0-complete ≤ upstream closure of visible rects | viewport-64k-of-1M | evaluation counter; time-to-visible-clean recorded | claimed |
| P-17 | insert/delete: blocks touched ≤ boundary + O(log n) via positional mapping (guards the 337 s rebind-churn pathology) | insert-storm-1M | blocks-touched vs eager renumber O(n) | claimed |
| P-18 | partition witness: same-level regions have disjoint read/write rects (witness validity only; parallel execution deferred) | boring-1Mx10, zig-zag-1M | witness validity + max-parallelism bound recorded | claimed |
| P-19 | warm no-op visits 0 cells on non-volatile sheets (permanent tripwire for the 10–80× warm pathology) | all, warm pass | cells-visited == 0 | claimed |
| P-20 | occupancy-proportional aggregates: `SUM(A:A)` slots visited == occupied, never 2^20 | full-column-1M, zig-zag-1M | reader slots-visited | claimed |
| P-21 | COW retention bytes ∝ touched blocks per revision | edit-storm + retention | retained-bytes growth | claimed |
| P-22 | grid publication entries ∝ delta, never full-N | edit-storm | publication-entries | claimed |
| P-23 | re-spill cost ∝ \|old ∪ new extent\| (cells written + rects propagated), never sheet size | filter-spill-1M | cells written + rects propagated | claimed |
| P-24 | hidden-toggle: cells_evaluated == \|affected visibility consumers\|, edges visited O(log intervals + k), independent of sheet size | hide-storm | seeds + cells-evaluated | claimed |
| P-25 | `A1#` dereference = O(1) spill-ledger probes, no extent scan | spill-storm | ledger-probe counter | claimed |
| P-26 | spill blockage check ∝ occupied blocks ∩ intended extent (never empty-slot iteration) | spill intent over empty 1M-row column | blocks-probed counter | claimed |
| P-27 | spill-extent epoch precision: `A1#` consumers re-evaluate only on extent-epoch or value change, never on unrelated anchor-sheet churn | spill-storm | consumer-evaluations counter | claimed |
| P-28 | aggregate-context query slots visited ∝ axis runs ∩ referenced span (provider side); per-cell seam expansion bounded by range extent (run-compressed seam = registered deferral) | hide-storm | provider slots-visited | claimed |

## 5. Workloads

Descriptors retained, fixtures generated (treecalc-scale precedent — no million-cell fixtures
in git). Engine workloads under `OxCalc/docs/test-corpus/grid-perf/`; host/session workloads
(doom, viewport, intent replays) under `DnaTreeCalc/docs/test-corpus/perf/` (directory named
in doctrine; to be created).

| id | shape | external comparison |
|---|---|---|
| boring-1Mx10 | 1M rows × 10 cols dense values+formulas | Excel ~37 B/cell, 0.57 s full recalc (candid-startup benchmark) |
| zig-zag-1M | 1M isolated diagonal singletons (worst block fragmentation) | — |
| full-column-1M | one column fully occupied, `SUM(A:A)` consumers | — |
| sum-pyramid-N | aggregation pyramid (Sestoft §3.3 O(N²) support blowup case) | — |
| deep-chain-N | sequential dependency chain (same shape as the calc-ekq3 spike chain — keeps rounds comparable) | spike baselines |
| fill-down-1M | one template region, `=R[-1]C+1` style | — |
| flash-fill-region | host-declared never-materialized region (owner decision: second authorship mode) | — |
| enron-mix | generator parameterized to ~4.5% unique-formula ratio | corpus statistics |
| insert-storm-1M | repeated row insert/delete across regions and block boundaries | — |
| viewport-64k-of-1M | visible rect + halo over large dirty model | — |
| doom-320x200 | cell=pixel, 64k cells changing per tick at 30–60 Hz | — |
| filter-spill-1M | FILTER() anchor whose result size changes per edit (re-spill old∪new cost) | — |
| spill-storm | many anchors + `A1#` consumers; extent churn + blockage/clearance cycles | — |
| hide-storm | boring-1Mx10 + k SUBTOTAL footers + running `SUBTOTAL(103,…)` probes; random hide/unhide/filter toggles | — |

## 6. Cadence and gates

- **Per PR:** counter gates at smoke scale for rows the PR touches, plus the ≤1e5-cell
  differential (`--engine both`). Engine rows gate in OxCalc CI; host rows in DnaTreeCalc CI.
- **Per round** (calc-ekq3 model): retained full-scale acceptance run →
  `docs/test-runs/core-engine/grid-scale/BASELINE_<date>.md` in the established format
  (commit, machine, phase split, variance caveat, ref/optimized ratios).
- **Run artifacts:** the scale runner gains `counter_summary.json` and
  `register_assertions.json` (per touched row: bound, measured, pass/fail) beside the existing
  summaries; phase vocabulary extends with `block_build`, `template_prepare`,
  `rect_propagation`, `tile_publication`.

## 7. Open questions (owner)

1. One register spanning lanes (this file) vs per-lane registers — confirm; this moves
   calc-perf record-keeping here.
2. Named baseline machine + pinned toolchain for retained wall-clock runs, or ratio-only?
3. `CalcRunCounters` compiled into release permanently (recommended) vs feature-gated?
4. Host-row gates (tiles, replay) in DnaTreeCalc CI even when the code is in OxCalc?
5. Hand-author the boring-1Mx10 Excel workbook now for the §7.6 Excel-timing leg, or wait for
   the OxDoc export path?
