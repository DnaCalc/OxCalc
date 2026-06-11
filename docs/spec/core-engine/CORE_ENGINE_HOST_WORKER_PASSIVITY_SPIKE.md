# Core Engine Host-Worker Passivity Spike

Status: investigation_complete — architecture GO / performance finding raised;
performance workstream round 1 (bead `calc-ekq3`) landed 2026-06-11 — see
"Performance workstream round 1" below
Owner: OxCalc
Consumer pressure: DNA TreeCalc stack-requirements W5 `host-worker-calc`
(ROADMAP open question 5: can the synchronous calc be sliced/resumed
cooperatively, or does bounded-slice pumping require engine reentrancy work?)

## Product Status

Two findings, one per audience:

1. **Architecture (the question asked): GO for run-to-completion in a host
   worker, with zero engine changes.** Cooperative slice/resume is NOT
   warranted. A per-node cancellation hook is a small bounded extension if a
   host ever needs intra-run abort.
2. **Performance (the question the spike exposed): synchronous recalculation
   cost is quadratic-or-worse in model size and is the real blocker for the
   program's scale goals** — a 1,000-node model takes ~4 minutes per cold
   recalc in release mode, and a *no-op* warm run is dramatically slower than
   a cold one. Worker hosting moves this off the UI thread; it does not make
   models usable. A dedicated performance workstream is the true prerequisite
   for DNA TreeCalc's "responsive at large N" goals, ahead of any worker
   plumbing.

## Architecture finding

`LocalTreeCalcEngine::execute` (`src/oxcalc-core/src/treecalc.rs:788`) is a
**pure function of an owned `LocalTreeCalcInput`**:

- `&self` on a stateless engine struct; all run state (coordinator, recalc
  tracker, working values, phase timer) is local to the call.
- The consumer (`OxCalcTreeContext::recalculate`,
  `src/oxcalc-core/src/consumer.rs:2888`) builds the input from cloned
  snapshots and applies the outcome to workspace state afterwards.
- No interior mutability or shared globals on the path (no `Rc`/`RefCell` in
  `consumer.rs`), so the context is movable to a worker thread on native, and
  on wasm a web worker can own the whole session outright.

Consequences:

- **Run-to-completion in a worker needs nothing from the engine.** The
  passivity doctrine ("host drives every tick") is preserved: the worker is
  host code calling the same synchronous API.
- **Cooperative slice/resume would be genuine rework** — `execute` would have
  to become a resumable state machine over ~10 locals spanning its phase
  pipeline (prepare → lower → graph build → invalidation closure → runtime
  setup → evaluation loop → finalize/publish). Cost M/L. Not justified while
  run-to-completion plus run-versioning (discard superseded results) covers
  the host's cancellation story.
- **Cancellation hook (optional, S):** the evaluation loop iterates
  `evaluation_order` in one function; a `should_cancel: &dyn Fn() -> bool`
  checked per node, returning a typed `Cancelled` run state, is a contained
  change if a host ever needs to abandon long runs mid-flight.

The host-side prerequisites (serializable Skin-IR surface for the postMessage
boundary, delta-only resync at scale) are DnaTreeCalc work and tracked there.

## Performance finding (release-mode timings)

Fixture: flat root-level nodes via the public consumer API
(`tests/host_worker_passivity_spike.rs`; run with
`cargo test -p oxcalc-core --release --test host_worker_passivity_spike -- --ignored --nocapture`).
"Chain" = `N{i} = N{i-1}+1` (depth stress); "fan" = independent `=1` leaves
(breadth stress). "Warm" = recalculate again with no changes
(`VerifiedClean`). Engine phase timings from the artifact's own
`phase_timings_micros`.

| scenario | wall | TotalEngineExecute | dominant phases |
|---|---|---|---|
| chain n=100 cold | 0.66 s | 0.45 s | eval 0.28 s · prepare 0.14 s |
| chain n=200 cold | 3.04 s | 1.93 s | eval 1.00 s · prepare 0.79 s |
| chain n=1000 cold | **244 s** | — | (first run; truncated batch) |
| fan n=1000 cold | **253 s** | — | shape-independent ⇒ per-formula cost |
| chain n=100 warm | **6.9 s** | 5.7 s | DiagnosticSeedCollection 3.2 s · EdgeValueCacheLookup 2.0 s |
| chain n=200 warm | **246 s** | 124.9 s | EdgeValueCacheLookup 88.7 s · DiagnosticSeedCollection 29.5 s |
| chain n=100 incremental (1 mid-edit) | **31.6 s** | 8.7 s | publication 2.6 s · eval 2.4 s · cache 2.0 s — and **23 s outside the engine timer** (consumer prelude/postlude) |

Observations:

1. **Cold runs scale ~quadratically** (2× nodes → ~4.6× time), with
   `OxfmlPrepareFormulas` + `OxfmlFormulaEvaluation` dominating: every run
   re-prepares every formula, and per-formula cost grows with model size.
2. **Warm runs invert the cache's purpose**: a no-change re-verification is
   ~10× (n=100) to ~80× (n=200) *slower* than the cold run, scaling ~35× for
   2× nodes. `EdgeValueCacheLookup` and `DiagnosticSeedCollection` dominate —
   the cache lookup path appears to do per-edge work proportional to whole-
   model state.
3. **Consumer-side overhead is its own problem**: the incremental case spends
   3× the engine's own time outside `TotalEngineExecute` — snapshot cloning
   and outcome application in `OxCalcTreeContext::recalculate`.
4. Fan ≈ chain at the same N ⇒ the cost is per-formula bookkeeping, not
   dependency depth.

None of this is visible at walking-skeleton scale (≤ ~25 nodes ⇒ tens of
milliseconds, feels instant in the browser); it bites from a few hundred
formulas up.

## Decision

- `host-worker-calc` (DNA TreeCalc W5): **unblocked architecturally** —
  proceed with run-to-completion worker hosting whenever the host schedules
  it; no OxCalc changes required. Engine-side cancellation hook available as
  an S-cost follow-up on request.
- **Raise an OxCalc performance workstream** as the successor to this spike,
  seeded with the phase evidence above. Candidate targets, in expected-payoff
  order: (a) `EdgeValueCacheLookup` per-edge cost on warm/verify runs,
  (b) `DiagnosticSeedCollection` scaling, (c) per-run `OxfmlPrepareFormulas`
  re-preparation (retain prepared formulas across runs keyed by binding
  snapshot), (d) consumer `recalculate` prelude/postlude cloning.
  DNA TreeCalc's "60fps at 100k nodes" W5 goal is unreachable by orders of
  magnitude until this lands; virtualization and delta channels only protect
  the frame, not time-to-result.

## Performance workstream round 1 (2026-06-11, bead `calc-ekq3`)

Four diagnoses, four semantics-preserving fixes, all merged to main with the
full `oxcalc-core` suite green (399 passed):

| commit | fix |
|---|---|
| `1955c8d` | Digest the edge-value-cache basis to a fixed-width 128-bit token before it reaches any per-node cache key (the raw basis embedded whole-model snapshot identity strings, Θ(N²) and copied into every key). |
| `6a2cca0` | `TreeNameResolutionIndex`: per-run memoized name resolution (meta-effectiveness, per-scope symbol maps, visible-symbol sweep) instead of whole-snapshot rescans per formula; plus `PreparedFormulaRetention` — prepared formulas retained across runs, reuse equality-gated on the full prepare basis and per-binding equality. |
| `aa8eb26` | Arc-share the heavy workspace payloads between live state, retained revisions, and outcomes; copy-on-write at mutation sites. `retain_current_workspace_revision` went from deep copies to reference bumps (~100× on its component timer). |
| `64e144f` | Digest layer snapshot-id bases the same way — the warm-recalc memory explosion (snapshot "IDs" were concatenations that folded each run's trace into the next revision's ids) is gone. |

Note: the digests use `std` `DefaultHasher` lanes — deterministic within a
build, **not stable across Rust releases**. The ids only participate in
equality comparisons against ids minted by the same process, so this is fine;
do not persist them across toolchains.

### Acceptance numbers (release, harness `ab86126`, 2026-06-11)

| scenario | wall | TotalEngineExecute | dominant phases |
|---|---|---|---|
| chain n=1000 cold | 69.9 s | 52.8 s | eval 46.8 s · publication 2.6 s |
| chain n=1000 warm | **5.6 s** | 2.7 s | VerifiedClean; cache lookup 8 ms · diag seeds 1.1 s |
| chain n=1000 incremental (1 mid-edit) | 14.4 s | 7.9 s | publication 4.1 s · **actual formula eval 55 ms** |
| chain n=2000 cold | 277.3 s | 174.4 s | eval 137.0 s · publication 21.8 s |
| chain n=2000 warm | 63.3 s | 28.5 s | VerifiedClean; diag seeds 13.0 s · prepare 12.4 s · cache lookup 36 ms |
| chain n=2000 incremental (1 mid-edit) | 171.5 s | 107.1 s | publication 67.6 s · actual formula eval 247 ms |

(Baseline at the same sizes was unmeasurable: n=1000 warm aborted on a 783 MB
allocation before `64e144f`; the 2026-06-10 table above shows n=200 warm at
246 s.)

### B.2.0 acceptance criteria (DNA TreeCalc PHASE_B): 1 of 3 met

1. **chain n=5k cold ≤ ~1 s: FAIL** (~3 orders of magnitude). Cold remains
   ~quadratic (2× nodes → 4.0× wall); n=5000 extrapolates to ~30–60 min.
   Dominant residual: per-formula `OxfmlFormulaEvaluation` cost growing with
   N — consistent with the known w056 O(n²) `host_name_bind_results`
   diagnostics, which need explicit sign-off to change.
2. **Warm strictly cheaper than cold: PASS** at both measured sizes (12.5× at
   n=1000, 4.4× at n=2000). This criterion flipped from pathological-fail
   (warm 10–80× *slower*) to pass; the margin shrinks with N because
   `DiagnosticSeedCollection` and `OxfmlPrepareFormulas` remain superlinear on
   the warm path.
3. **Incremental ∝ dirty set: FAIL.** Re-evaluation itself *is* proportional
   (55–247 ms), but wall is dominated by full-N `CandidatePublication`
   (4.1 s → 67.6 s for 2× nodes, worse than quadratic) and consumer-side
   overhead outside the engine timer.

### Residual targets for round 2

- `OxfmlFormulaEvaluation` per-formula cost growth (w056 diagnostics — gate:
  sign-off on changing their O(n²) shape).
- `CandidatePublication` full-N cost on every run, including incrementals.
- Warm-path `DiagnosticSeedCollection` / `OxfmlPrepareFormulas` superlinearity.
- Consumer `recalculate` outside-engine overhead beyond the retention copies
  (already ~99% reduced).

**Consequence for DNA TreeCalc:** B.2.2 worker hosting remains gated — the
warm/no-op path is now sane, but time-to-result at thousands of nodes is
still the blocker the worker cannot fix.

## Evidence

- `src/oxcalc-core/tests/host_worker_passivity_spike.rs` (`#[ignore]`d timing
  harness; 2026-06-10 numbers from the original release run, 2026-06-11
  numbers from the acceptance run at `ab86126`, both on the development
  machine — note ±25% run-to-run wall variance observed on this box).
- Code-reading trail: `treecalc.rs:788` (`execute` phase pipeline),
  `treecalc.rs:993` (topological order + evaluation loop),
  `consumer.rs:2888` (`recalculate` prelude/postlude).
