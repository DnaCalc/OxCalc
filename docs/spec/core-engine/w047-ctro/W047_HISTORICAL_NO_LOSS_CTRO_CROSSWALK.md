# W047 Historical No-Loss CTRO Crosswalk

Status: `calc-aylq.1_crosswalk_validated`

Parent workset: `W047 Calc-Time Rebinding Overlay Design Sweep`

Parent bead: `calc-aylq.1`

## 1. Purpose

This packet records the no-loss review required before later W047 design and implementation work proceeds. Its job is to preserve the older Calc-Time Rebinding Overlay precursors without importing obsolete scope, shallow proof patterns, or hidden evaluator mutation.

The crosswalk classifies predecessor material into four buckets:

| Bucket | Meaning |
| --- | --- |
| retained | The idea remains part of W047 CTRO doctrine or the implementation target. |
| revised | The idea remains useful, but the W047 landing changes its scope or evidence standard. |
| rejected | The idea is explicitly not used by W047, with reason. |
| routed | The idea is deferred to W048/W049 or a later workset rather than handled inside W047. |

W047 uses this packet as the input gate for `calc-aylq.2` through `calc-aylq.4` and `calc-aylq.7`. It does not discharge W049 formal, checker, sidecar, pack, C5, operated-service, or release-readiness work.

## 2. Intended CTRO Pipeline

The predecessor material converges on this W047 engine pipeline:

1. Start from immutable structural snapshot `S`.
2. Build structural graph `G_struct` from dependencies known before formula evaluation.
3. Attach accepted runtime overlay `O_published`.
4. Define effective graph `G_eff = G_struct + O_published`.
5. On edit or upstream value change, derive invalidation over `G_eff`.
6. Evaluate the deterministic needed frontier.
7. If evaluation discovers dependency-shape, region, spill, capability, or execution-restriction effects, stage them as `O_candidate`.
8. Classify overlay deltas against `O_published`: activation, release, reclassification, region/spill resize, unsupported/unresolved, or cycle introduction.
9. Compose provisional graph `G_eff_prime = G_struct + O_candidate`.
10. Run the same SCC/cycle classifier on `G_eff_prime` that the structural path uses on `G_struct` and current `G_eff`.
11. For the current non-iterative Stage 1 profile, any cycle group in `G_eff_prime` yields `CycleBlocked` / `SyntheticCycleReject`, no value publication, and no overlay commit.
12. Repair invalidation frontier and order only for the acyclic remainder when repair is deterministic and supported.
13. Reject or fallback when graph repair, dependency classification, region/spill support, or evaluator data is insufficient. The fallback/reject must be replay-visible.
14. Candidate publication is atomic: values and overlay consequences publish together, or neither publishes.
15. Retain and evict `O_published` under explicit reader/replay compatibility rules.

## 3. Source Crosswalk

| Source | Classification | CTRO material recovered | W047 landing | Later route or rejection reason |
| --- | --- | --- | --- | --- |
| `W003_STAGE1_COORDINATOR_AND_PUBLICATION_BASELINE.md` | retained and revised | Candidate result can carry `dependency_shape_updates`, `runtime_effects`, and `published_runtime_effects`; reject taxonomy includes `dynamic_dependency_failure` and `synthetic_cycle_reject`. | Candidate bundles must include value deltas plus dependency-shape/runtime-effect consequences. CTRO cycle rejection uses the existing synthetic-cycle reject family rather than a CTRO-only code. | Revised because W047 requires candidate overlay classification before publication, not just passive fields in a Stage 1 result shape. |
| `W004_INCREMENTAL_RECALC_AND_OVERLAY_BASELINE.md` | retained | Stage 1 runtime effects include dynamic dependency activation/release and region-shape activation/release. Dependency-shape update kinds include `activate_dynamic_dep`, `release_dynamic_dep`, `change_region_membership`, and `synthetic_spill_shape`. Overlay keys, retention, fallback, and re-entry are first-class. | This becomes the main CTRO state model: `O_published`, `O_candidate`, overlay compatibility keys, release/activation deltas, region/spill resizing, fallback, and overlay retention. | Economics and optimization measurement from this lane are routed to W049 or later; W047 records counters only if the implementation needs them to stay diagnosable. |
| `W007_LEAN_FACING_STATE_OBJECTS_AND_TRANSITION_BOUNDARY_PLAN.md` | retained and revised | Runtime-derived state and `OverlayEntry` are separate from structural truth. | W047 keeps structural snapshots immutable and represents runtime-derived dependency facts as explicit effective-graph overlay state. | Revised because W047 will not add new Lean state objects; formal state-object work is routed to W049. |
| `W008_TLA_COORDINATOR_PUBLICATION_AND_FENCE_SAFETY_MODEL_PLAN.md` | retained and routed | `overlayState` participates in candidate, reject, publish, and eviction actions; cycle-blocked states are represented. | W047 keeps candidate/reject/publish/evict overlay transitions in the design and Rust-facing semantics. | TLA modeling is routed to W049. W047 does not extend smoke-level TLA artifacts. |
| `W009_REPLAY_AND_PACK_BINDING_FOR_STAGE1_SEAM_AND_COORDINATOR_BEHAVIOR.md` | retained and routed | Replay classes cover fallback and overlay re-entry; dynamic dependency bind semantics are named. | W047 requires TraceCalc/TreeCalc scenario rows to expose overlay activation, release, fallback, and re-entry instead of hiding them inside evaluator behavior. | Proof-carrying trace schema additions, pack binding, and sidecar enrichment are routed to W049. |
| `W010_EXPERIMENT_REGISTER_AND_MEASUREMENT_SCHEMA_PLANNING.md` | retained and routed | Dynamic-topological maintenance, deterministic rebuild, overlay reuse/miss, fallback rates, and reuse-after-retention economics are measurable concerns. | W047 uses these concerns to keep fallback explicit and to avoid claiming dynamic-topo wins without data. | Experimental economics and promotion thresholds are routed to W049 or later operated-evidence work. |
| `W012_TRACECALC_REFERENCE_MACHINE_AND_CONFORMANCE_ORACLE.md` | retained | TraceCalc state includes runtime overlay state and conformance surfaces. | W047 scenario design must include reference rows for graph-stable recalc, dynamic target switch, unresolved dynamic reject, downstream invalidation, spill/region change, cycle introduction, cycle release, and fallback. | Native proof sidecars are routed to W049. W047 can use existing TraceCalc/TreeCalc surfaces only to the extent they are already available. |
| `W027_TREECALC_DEPENDENCY_GRAPH_AND_INVALIDATION_CLOSURE.md` | retained and revised | Current Rust floor has explicit graph, reverse edges, diagnostics, cycle groups, and invalidation closure with cycle-blocked nodes. | W047 reuses this graph discipline for both structural and effective graphs. The same SCC/cycle classifier must classify candidate `G_eff_prime`. | Revised because W027 handled runtime-derived overlay closure as a successor item; W047 makes it central. |
| `W029_TREECALC_RUNTIME_DERIVED_EFFECTS_AND_OVERLAY_CLOSURE.md` | retained | Runtime-derived effects and overlays are live, replay-visible, deterministic projections rather than hidden mutable truth. Covered effect families include dynamic dependency, capability, execution restriction, and shape/topology. | W047 treats runtime effects as CTRO inputs and requires no-publish/no-overlay-commit on reject. Positive dynamic dependency publication becomes the key implementation refinement lane. | Any unsupported syntax, full grid semantics, or broad spill substrate remains exact-blocked rather than silently generalized. |
| `CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md` | retained and revised | Distinguishes structural graph from effective runtime graph; includes `HoldCycleBoundary`, `cycle_blocked`, deterministic topological/SCC baseline, runtime-observed dependencies, and explicit fallback. | This provides the policy baseline for `G_struct`, `G_eff`, invalidation over effective graph, CTRO frontier repair, and shared structural/CTRO cycle handling. | Revised to add candidate overlay graph `G_eff_prime` and to defer iterative-cycle semantics beyond the current Stage 1 floor. |
| `CORE_ENGINE_OXFML_SEAM.md` | retained and revised | Candidate/publication distinction; accepted candidate carries value plus topology/dependency consequences; runtime-observed dependency effects, shape/topology effects, and reject categories. | W047 preserves OxCalc-owned commit authority. OxFml may expose observed runtime facts, but OxCalc coordinates acceptance, rejection, publication, and overlay retention. | Any seam clause that would require OxFml contract change must become a handoff/watch item before promotion. |
| W046 semantic-spine packets | revised and routed | Useful vocabulary: graph construction, reverse edges, SCC/cycle classification, invalidation, order/read discipline, reject/no-publish, publication/refinement language, OxFml seam effects. Known defect pattern: record-projection proofs, tiny smoke TLA, silent-degrade checker, unbound evidence roots, terminology drift. | W047 reuses vocabulary and Rust-facing behavior where it constrains real calculation, especially graph/SCC and reject/no-publish. | Formal/checker/sidecar/readiness work is routed to W049. W047 rejects adding new decorative Lean/TLA/checker artifacts. |
| INDIRECT and spill scenario matrix | retained and revised | Stress cases: static comparator, dynamic target switch, unresolved target, downstream dependent, spill expansion, spill contraction, structural cycle, CTRO self-cycle, CTRO multi-node cycle, CTRO cycle release, fallback. | W047 uses these as the minimum scenario classes for `calc-aylq.3` and implementation evidence planning. Dynamic dependency positive publication is the narrow implementation proving ground. | Full Excel spill/grid behavior and direct Excel observation packets are routed to W048 unless existing TreeCalc support is sufficient for a deterministic local scenario. |

## 4. Decisions Preserved For Later W047 Beads

Retained decisions:

1. CTRO is a third change class, distinct from value-only recalc and structural/model change.
2. Effective graph state is explicit: `G_struct`, `O_published`, `O_candidate`, `G_eff`, and `G_eff_prime`.
3. Runtime-derived dependency facts are separate from structural truth.
4. Candidate results carry value, dependency-shape, runtime-effect, diagnostic, and publication consequences together.
5. Publication is atomic across values and overlay consequences.
6. Reject means no value publication and no overlay commit.
7. Structural and CTRO-created cycles share the same SCC/cycle policy; CTRO provenance is diagnostic, not a separate semantic class.
8. Fallback is allowed only as a visible, instrumented safety path, not as a way to hide a known cycle or unsupported dynamic dependency.
9. Overlay retention and eviction are compatibility-keyed and reader/replay aware.

Revised decisions:

1. W047 is implementation-first. It records formal obligations but does not produce new Lean, TLA, checker, or sidecar artifacts.
2. W046 terminology is retained only when it names concrete engine behavior; broad "refinement" or "proof" labels do not carry authority without non-vacuous evidence.
3. Existing structural graph/SCC machinery is the anchor; CTRO extends it to effective and candidate graphs rather than replacing it.
4. Iterative circular calculation is not part of the current Stage 1 floor; W048 owns the Excel-compatible investigation and later deterministic iteration lane.

Rejected decisions:

1. No hidden evaluator mutation. Formula evaluation can observe runtime dependency facts; it cannot commit graph truth directly.
2. No value-only publication when accepted overlay consequences were dropped or rejected.
3. No CTRO-only cycle semantics separate from structural SCC/cycle handling.
4. No generic fallback when `G_eff_prime` has already proved a cycle and the current profile requires cycle rejection.
5. No W047 formal/checker/sidecar artifacts that repeat the W046 shallow-proof failure mode.

Routed decisions:

1. Lean/TLA model repair, non-vacuous theorem work, checker hardening, and proof-carrying sidecars route to W049.
2. Pack, C5, operated-service, release, and readiness gates route through W049 evidence.
3. Dynamic-topology economics and overlay performance thresholds route to W049 or later operated evidence.
4. Direct Excel circular-reference observation packets and iterative-cycle determinacy route to W048.
5. Broad grid/spill substrate work routes to a later host/grid workset unless the W047 TreeCalc subset already supports a narrow deterministic case.

## 5. Prompt-To-Artifact Checklist

| Requirement from `calc-aylq.1` | Evidence in this packet |
| --- | --- |
| Cover `W003` | Section 3 row for `W003_STAGE1_COORDINATOR_AND_PUBLICATION_BASELINE.md`. |
| Cover `W004` | Section 3 row for `W004_INCREMENTAL_RECALC_AND_OVERLAY_BASELINE.md`. |
| Cover `W007` | Section 3 row for `W007_LEAN_FACING_STATE_OBJECTS_AND_TRANSITION_BOUNDARY_PLAN.md`. |
| Cover `W008` | Section 3 row for `W008_TLA_COORDINATOR_PUBLICATION_AND_FENCE_SAFETY_MODEL_PLAN.md`. |
| Cover `W009` | Section 3 row for `W009_REPLAY_AND_PACK_BINDING_FOR_STAGE1_SEAM_AND_COORDINATOR_BEHAVIOR.md`. |
| Cover `W010` | Section 3 row for `W010_EXPERIMENT_REGISTER_AND_MEASUREMENT_SCHEMA_PLANNING.md`. |
| Cover `W012` | Section 3 row for `W012_TRACECALC_REFERENCE_MACHINE_AND_CONFORMANCE_ORACLE.md`. |
| Cover `W027` | Section 3 row for `W027_TREECALC_DEPENDENCY_GRAPH_AND_INVALIDATION_CLOSURE.md`. |
| Cover `W029` | Section 3 row for `W029_TREECALC_RUNTIME_DERIVED_EFFECTS_AND_OVERLAY_CLOSURE.md`. |
| Cover `CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL` | Section 3 row for that spec. |
| Cover `CORE_ENGINE_OXFML_SEAM` | Section 3 row for that spec. |
| Cover W046 packets | Section 3 W046 row and Sections 4 revised/rejected/routed decisions. |
| Cover INDIRECT/spill scenario matrix | Section 3 scenario-matrix row and Section 6. |
| Identify intended CTRO pipeline before later design work | Section 2 pipeline and Section 4 decisions. |
| Classify predecessor ideas as retained, revised, rejected, or routed | Sections 3 and 4. |

## 6. Scenario Classes Carried Forward

These scenario classes are carried to `calc-aylq.3` as the minimum W047 matrix:

| Scenario class | CTRO question carried forward |
| --- | --- |
| static value edit | Baseline graph-stable recalc comparator. |
| dynamic target switch | Dependency release/activation and deterministic frontier repair. |
| unresolved dynamic target | Reject/no-publish/no-overlay-commit boundary. |
| dynamic target with downstream dependent | Downstream invalidation after overlay change. |
| spill expansion | Region activation and new dependent exposure. |
| spill contraction | Region release, stale outputs, and consumers. |
| structural direct cycle | Baseline SCC/cycle policy. |
| CTRO self-cycle | Candidate overlay points owner to itself and must follow the same cycle policy. |
| CTRO multi-node cycle | Candidate overlay creates an SCC across formulas. |
| CTRO cycle release | Previously blocked dynamic target becomes acyclic and needs deterministic re-entry. |
| conservative fallback/rebuild | Safety path when repair cannot be classified or supported. |

## 7. Current Status

- execution_state: `calc-aylq.1_crosswalk_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - `calc-aylq.3` scenario matrix and evidence plan
  - `calc-aylq.4` implementation remodeling and readiness-gate routing
  - `calc-aylq.7` dynamic dependency positive publication implementation refinement
  - W049 formal/checker/sidecar/readiness successor work
