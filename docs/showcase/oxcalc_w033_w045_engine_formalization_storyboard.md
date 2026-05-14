# OxCalc W033-W045 Engine Formalization Showcase Storyboard

Revision note: this storyboard has been superseded for the densified review pass by
`docs/showcase/oxcalc_w033_w045_engine_formalization_review_catalog.md`.
The current HTML deck uses the catalog as its governing script: every major engine
phase is now presented with the color-coded lanes `Engine Design`, `Formal Target`,
`Artifacts Today`, and `Gap / Direction`.

The original 50-slide storyboard remains here as historical planning context for the
first showcase rebuild. The active review deck intentionally shifts emphasis from
"what artifacts exist" to "which engine transitions and invariants are modeled,
which are proved fragments, which are bounded/replay-backed, and which remain
formal targets."

Audience: internal/external technical review of the OxCalc + OxFml formalization path through W045.

Intent: tell an engine-first story. The deck should show how a calculation request moves through OxFml formula preparation, dependency descriptor lowering, graph construction, SCC/cycle grouping, invalidation closure, recalc ordering, candidate evaluation, reject/no-publish, and publication. It should then map those implementation phases to TraceCalc, TreeCalc, Lean, TLA+, and machine-readable evidence artifacts.

Evidence weight scan used to set slide emphasis:

| Surface | Local scan result | Showcase implication |
| --- | ---: | --- |
| `docs/test-runs/core-engine` | 10,848 files, 21.07 MB, 23 roots | Evidence is mostly engine/replay/scale artifacts, not closure prose. |
| largest run roots | `treecalc-local` 5,992 files; `tracecalc-reference-machine` 2,739 files; `tracecalc-retained-failures` 1,097 files | Early deck weight goes to runtime traces and replay. |
| formal artifacts | `formal/lean` 34 files; `formal/tla` 2,626 files including TLC output | Formal material is broad but includes bounded outputs and classification theorems, not full proof promotion. |
| implementation source | `src/oxcalc-core/src` 14 files; `src/oxcalc-tracecalc/src` 21 files | Use code snippets as phase anchors and show how replay services surround the engine. |
| current W045 closure audit | 36 obligations, 18 promotion contracts, 28 TreeCalc cases with 0 expectation mismatches | Present W045 as strong C4-grade evidence with explicit residual boundaries. |

Visual assets generated for the HTML deck:

| Asset | Use |
| --- | --- |
| `assets/engine-pipeline-hero.png` | opening and phase overview |
| `assets/dependency-graph-scale.png` | dependency graph, SCC, invalidation, scale section |
| `archive/formal-verification-lab.png` | Lean/TLA/proof section |
| `assets/evidence-instrument-finale.png` | closing/future section |

## 50-Slide Storyboard

1. Title: OxCalc Formalization Showcase Through W045
   - Point: this is a calculation-engine formalization story, not a workset-process story.
   - Evidence: local artifact scan, W045 closure audit, generated hero visual.
   - Visual: full-bleed engine pipeline raster plus metrics strip.

2. The Evidence Surface Is Engine Heavy
   - Point: quantify closure/process versus engine/evidence weight.
   - Evidence: 10,848 core-engine run artifacts, 21.07 MB; largest roots are TreeCalc/TraceCalc.
   - Visual: treemap/table with roots by file count.

3. What The Formalization Is Trying To Buy
   - Point: quality, correctness, evolution freedom, not freezing an initial spec.
   - Evidence: spec evolution guard, W045 non-promotion reasons, successor W046.
   - Visual: three lanes: implementation, formal model, replay oracle.

4. Core Objects At The Engine Boundary
   - Point: `LocalTreeCalcInput` defines the execution problem.
   - Implementation: `treecalc.rs` lines 47-57.
   - Formal counterpart: TLA variables over snapshots, runtime view, candidates, publications.
   - Visual: contract card with input fields and state variables.

5. What A Run Emits
   - Point: `LocalTreeCalcRunArtifacts` is the evidence-producing runtime envelope.
   - Implementation: `treecalc.rs` lines 89-103.
   - Evidence: TreeCalc local run directories and `phase_timings_micros`.
   - Visual: artifact bundle fan-out.

6. Environment Effects Are Explicit
   - Point: dynamic, execution restriction, capability, shape topology, and overlay effects are first-class.
   - Implementation: `LocalTreeCalcEnvironmentContext`.
   - Formal counterpart: TLA snapshot/capability fences and Lean external boundary rows.
   - Visual: runtime context toggles feeding reject/publication gates.

7. The Concrete Engine Call Trace
   - Point: `LocalTreeCalcEngine::execute` is an ordered phase trace.
   - Implementation: `record_duration(...)` phases in `treecalc.rs`.
   - Evidence: scale runner phase split.
   - Visual: timeline from OxFml prepare to evaluation loop.

8. Phase 1: OxFml Preparation
   - Point: formula bindings are translated/bound before graph construction.
   - Implementation: `prepare_oxfml_formula`.
   - OxFml seam: W045 direct OxFml cases, LET/LAMBDA carrier boundary.
   - Visual: formula text into bound formula plus bind diagnostics.

9. Phase 2: Descriptor Lowering
   - Point: formula references become dependency descriptors.
   - Implementation: `oxfml_dependency_descriptors`.
   - Formal counterpart: descriptor-kind taxonomy maps to explicit proof/model rows.
   - Visual: reference token to descriptor record.

10. Descriptor Taxonomy
   - Point: static, relative, dynamic, host, capability, shape, unresolved references are separated.
   - Implementation: `DependencyDescriptorKind`.
   - Evidence: W045 exact blockers preserve dynamic/soft-reference breadth gaps.
   - Visual: seven descriptor chips with consequences.

11. Edge Construction And Diagnostics
   - Point: targetful descriptors become edges; untargeted descriptors become diagnostics.
   - Implementation: `DependencyGraph::build`.
   - Formal counterpart: reject path for incompatible dependencies.
   - Visual: split flow: `target_node_id` -> edge, `None` -> diagnostic.

12. Reverse Edge Index
   - Point: invalidation runs over reverse edges from edited/upstream nodes to dependents.
   - Implementation: `reverse_edges` in `DependencyGraph`.
   - Evidence: TreeCalc local post-edit cases.
   - Visual: edit wave over reverse adjacency.

13. Cycle Groups As SCCs
   - Point: Tarjan-style SCC detection gives explicit cycle groups.
   - Implementation: `find_cycle_groups`, `TarjanState`, `strong_connect`.
   - Formal counterpart: node state includes `cycle_blocked`.
   - Visual: SCC boxes over graph raster.

14. Invalidation Reasons
   - Point: the reason enum is a small domain-specific algebra for edit/upstream/dependency changes.
   - Implementation: `InvalidationReasonKind`.
   - Evidence: dynamic dependency add/remove/reclassify registers.
   - Visual: reason table with `requires_rebind` consequence.

15. Closure Derivation
   - Point: invalidation closure is BFS over reverse edges, with cycle/block and sorted reasons.
   - Implementation: `derive_invalidation_closure`.
   - Formal counterpart: monotone dirty/needed/blocked state evolution.
   - Visual: queue, records map, impacted order.

16. Rebind Gate
   - Point: structural rebind-needed nodes block local sequential recalc.
   - Implementation: `rebind_gate_scan`.
   - Evidence: relative rebind churn and soft-reference update scale profile.
   - Visual: gate before topological evaluation.

17. Topological Evaluation Order
   - Point: dependency graph becomes an evaluation order or a reject path.
   - Implementation: `topological_formula_order`.
   - Formal counterpart: TLA action sequence A1-A7 and stage partition ordering.
   - Visual: DAG layer ordering.

18. Recalc Tracker States
   - Point: state transitions model dirty, needed, evaluating, verified, publish-ready, rejected, cycle-blocked.
   - Implementation: `Stage1RecalcTracker`.
   - Formal counterpart: `NodeStates` in `CoreEngineStage1.tla`.
   - Visual: state machine.

19. Evaluation Loop
   - Point: each node enters evaluate, delegates to OxFml/OxFunc-backed backend, and updates working values.
   - Implementation: `begin_evaluate`, `evaluate_with_oxfml_session`, `value_updates`.
   - Evidence: TraceCalc run steps and TreeCalc local witness bundles.
   - Visual: per-node mini-pipeline repeated over ordered nodes.

20. OxFml Runtime Candidate Adaptation
   - Point: accepted/rejected OxFml results are adapted into TreeCalc success/failure.
   - Implementation: `evaluate_with_oxfml_session`, `invoke_prepared_formula_via_session`, `adapt_oxfml_runtime_candidate`.
   - OxFunc peek: LET/LAMBDA carriers cross OxFml/OxFunc interaction boundary.
   - Visual: OxFml result surface into accept/reject.

21. Residual And Host Outcomes
   - Point: unresolved residuals and typed host-provider outcomes reject unless lazy residual publication is allowed.
   - Implementation: residual check in `evaluate_with_oxfml_session` using `prepared.lazy_residual_publication`.
   - Evidence: host/dynamic/capability/shape reject fixtures.
   - Visual: residual gate with diagnostics.

22. Runtime Effects And Overlays
   - Point: failure effects are annotated with environment and projected into overlays.
   - Implementation: `annotate_runtime_effects_with_environment`, `build_runtime_effect_overlays`.
   - Formal counterpart: TLA overlay state, protected overlays.
   - Visual: overlay cards pinned to nodes/readers.

23. Candidate, Reject, Publication
   - Point: candidate values are not publication; reject means no publish.
   - Implementation: `local_candidate`, `candidate_result`, `publication_bundle`, `reject_detail`.
   - Formal counterpart: `RejectIsNoPublish`, `CandidateIsNotPublication`.
   - Visual: three mutually exclusive outcome lanes.

24. Publication Fence Semantics
   - Point: accepted candidate requires compatible snapshot/capability basis before publication.
   - Implementation: coordinator/publication bundle paths.
   - Formal counterpart: `AcceptedCandidateRequiresFences`.
   - Visual: snapshot/capability fence lock.

25. TraceCalc As Correctness Oracle
   - Point: TraceCalc is a reference implementation and evidence generator for spec purposes.
   - Implementation: `src/oxcalc-tracecalc/src/machine.rs`, replay runners.
   - Evidence: `tracecalc-reference-machine` 2,739 files.
   - Visual: independent oracle beside TreeCalc.

26. TraceCalc State Machine
   - Point: TraceCalc expands deterministic run traces into comparison surfaces.
   - Implementation: `execute_step`, replay manifests, retained failures.
   - Formal counterpart: TLA action machine.
   - Visual: machine trace tape.

27. TreeCalc Fixture Corpus
   - Point: local TreeCalc fixtures cover publish/reject/post-edit/overlay paths.
   - Evidence: W045 28 TreeCalc cases, 0 expectation mismatches.
   - Visual: fixture matrix grouped by behavior.

28. W033 Baseline Closure Of The First Formalization Slice
   - Point: W033 established initial replay/formal bridge evidence.
   - Evidence: 12 TraceCalc scenarios, 17 TreeCalc cases, TLA Stage1 smoke, cargo test/clippy.
   - Visual: baseline milestone card.

29. W034-W036: From Stage1 To Stage2 Partitioning
   - Point: bounded model and partition replay semantics were added.
   - Formal counterpart: `CoreEngineW036Stage2Partition.tla`.
   - Evidence: Stage2 replay roots and TLA model output.
   - Visual: two partitions with cross-dependency guard.

30. W037-W039: Policy Predicates And Promotion Guards
   - Point: proof predicates encode what would be required for promotion.
   - Lean: `CanPromoteStage2ProductionPolicy`.
   - Evidence: no-proxy/match promotion guards.
   - Visual: boolean lattice from evidence inputs to policy result.

31. W040-W042: Operated Assurance And Diversity
   - Point: formalization widened toward independent evaluator, differential, retained history.
   - Evidence: continuous-assurance, diversity-seam, operated-assurance roots.
   - Visual: multi-engine comparison harness.

32. W043-W045: Hardened Boundaries
   - Point: W045 binds optimized core, Rust totality/refinement, Lean/TLA, Stage2, OxFml seam, scale, closure audit.
   - Evidence: W045 run summaries.
   - Visual: W045 evidence map.

33. Lean: Row Models As Checked Evidence Classifiers
   - Point: current Lean files classify evidence and blockers with checked theorems.
   - Lean: `W045RustRow`, `W045ProofModelRow`.
   - Honesty: this is not full functional equivalence proof.
   - Visual: proof row table with predicates.

34. Lean: Stage2 Promotion Predicate
   - Point: promotion requires declared replay plus dynamic, soft-reference, analyzer, fairness, operated, pack governance inputs.
   - Lean: `CanPromoteStage2ProductionPolicy`.
   - Evidence: current W045 false fields.
   - Visual: AND-gate dependency graph.

35. Lean: Current No-Promotion Theorems
   - Point: checked statements prove current evidence does not overclaim.
   - Lean: `currentW045Stage2Evidence_doesNotPromoteStage2Policy`, `w045LeanTlaSummary_noFullLeanPromotion`.
   - Visual: proof cards with blocker badges.

36. TLA+: Stage1 Publication Safety
   - Point: TLA Stage1 models candidate/admit/reject/publish state.
   - TLA: `CoreEngineStage1.tla`.
   - Checked properties: type invariant, no torn publication, reject/no-publish, candidate not publication.
   - Visual: state transition swimlane.

37. TLA+: Stage2 Partition Safety
   - Point: bounded partition model checks fences, no policy promotion, reject/no-publish.
   - TLA: `CoreEngineW036Stage2Partition.tla`.
   - Checked properties: `NoStage2PolicyPromotion`, `AcceptedCandidateRequiresFences`.
   - Visual: partition scheduler state space.

38. Formal Checks Actually Run
   - Point: separate checked commands from claimed properties.
   - Evidence: Lean invocations, cargo tests, CLI replay runners, JSON validation, TLA smoke/model configs.
   - Visual: command/output table.

39. Rust Totality And Refinement Surface
   - Point: W045 Rust evidence has 17 rows, 11 local checked-proof classifications, 5 totality boundaries, 7 exact blockers.
   - Evidence: W045 Rust run summary.
   - Visual: row-count dashboard.

40. Panic Surface Audit
   - Point: panic markers are counted and carried as a boundary, not erased.
   - Evidence: W045 panic marker count 158.
   - Formal counterpart: panic-free core promotion false.
   - Visual: code heat map with boundary marker.

41. OxFml Seam And LET/LAMBDA
   - Point: LET/LAMBDA value/carrier behavior threads OxCalc, OxFml, and a small OxFunc peek.
   - Evidence: W045 direct OxFml 7 cases, LET/LAMBDA 2 cases.
   - Formal counterpart: callable carrier accepted boundary, callable metadata projection still unpromoted.
   - Visual: seam bridge.

42. OxFml Formatting Guard
   - Point: W073 typed-rule formatting intake is carried as a guard, not broad display publication closure.
   - Evidence: W045 W073 formatting guard 5 cases; downstream typed-rule construction not verified.
   - Visual: payload schema guard.

43. Scale Profiles
   - Point: performance/scaling tests define grid, fanout, dynamic indirect stripes, relative rebind churn.
   - Implementation: `TreeCalcScaleProfile`, defaults near one million nodes.
   - Visual: four scale scenario panels.

44. Scale Phase Split
   - Point: scale runner separates model build, descriptor lowering, graph/SCC, soft-reference update, invalidation closure, recalc, validation.
   - Implementation: `execute_scale_model`.
   - Evidence: scale run summaries and phase timings.
   - Visual: timing bars by phase.

45. Closed-Form Correctness Checks
   - Point: scale recalc is checkable by deterministic formulas, not only elapsed time.
   - Implementation: `synthetic_grid_recalc`.
   - Evidence: validation checks and scale required artifacts.
   - Visual: left/top delta equations and observed/expected equality.

46. Dynamic/Soft-Reference Stress
   - Point: soft-reference update and INDIRECT-like churn remain first-class future hardening targets.
   - Implementation: `execute_soft_reference_update`.
   - Evidence: W045 soft-reference/INDIRECT blocker remains exact.
   - Visual: selector stripes and rebind seeds.

47. Evidence-To-Property Matrix
   - Point: show which properties have direct evidence, bounded formal evidence, classification proof, or exact blockers.
   - Evidence: W045 closure audit and release decision.
   - Visual: dense matrix.

48. Highest Honest Capability At W045
   - Point: strongest honest label is `cap.C4.distill_valid`; release/C5/full formalization are not promoted.
   - Evidence: W045 closure audit.
   - Visual: capability ladder with W045 position.

49. W046+ Engine-First Follow-Up Scope
   - Point: future work should deepen engine formalization: dependency graph laws, SCC correctness, invalidation closure, soft-reference totality, recalc determinism, scheduler fairness, operated differential.
   - Evidence: W045 successor fields and non-promotion reasons.
   - Visual: roadmap as proof obligations tied to phases.

50. Closing: A Calculation Tool With Traceable Semantics
   - Point: OxCalc now has an engine implementation, a reference oracle, bounded formal models, checked Lean classifiers, and extensive replay evidence. The next phase is to turn more classifiers into semantic proofs.
   - Evidence: file/MB counts, W045 summary metrics, residual boundaries.
   - Visual: finale raster plus property stack.
