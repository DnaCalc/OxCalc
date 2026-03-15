# W013: Execution Sequence A - TreeCalc Stage 1 First Implementation Sequence

## Purpose
Operationalize the first serious OxCalc implementation path as a dependency-ordered execution packet for the TreeCalc-first Stage 1 engine.

This packet exists to:
1. turn the current workset graph into an executable sequence,
2. identify the true critical path for first implementation,
3. define concrete sub-phase gates for implementation kickoff,
4. prevent the first execution wave from drifting back into open-ended planning.

## Position and Dependencies
- **Depends on**: W001, W005
- **Blocks**: none
- **Cross-repo**: assumes accepted shared seam direction from `HANDOFF-CALC-001`; any narrower replay- or trace-facing seam pressure discovered during execution must be handed back through OxCalc -> OxFml handoff discipline

## Scope
### In scope
1. The first execution sequence for the TreeCalc-first Stage 1 implementation wave.
2. The concrete gate criteria for sub-phases 1 through 10.
3. The critical-path ordering and the side-lane ordering.
4. The final gate for the whole `Execution Sequence A` wave.

### Out of scope
1. Replacing the numbered worksets themselves.
2. Stage 2 concurrency realization.
3. Grid or later substrate introduction.
4. Declaring any workset or feature lane complete without its own evidence gates.

## Sequence Preconditions
Execution Sequence A assumes the following preconditions already hold:
1. the rewritten OxCalc canonical spec set exists,
2. the TreeCalc-first target is locked,
3. the Stage 1 coordinator, recalc, overlay, formalization, harness, and oracle planning packets exist,
4. the OxFml seam has acknowledged `HANDOFF-CALC-001`,
5. W001 and W005 remain open as governance/integration lanes, but no longer block the first local implementation sequence.

## Critical-Path Doctrine
The critical path for the first implementation wave is:
1. W002 structural kernel,
2. W003 coordinator and publication baseline,
3. W004 recalc and overlay baseline,
4. W006 gate binding,
5. W007/W008/W009 first assurance artifacts,
6. W010 instrumentation schema,
7. W011 harness implementation,
8. W012 reference-machine implementation.

W001 and W005 are carried conditions across the sequence.
They must stay aligned, but they are not the main implementation pacing items.

## Execution Sequence A

### Sequence 1. W002 Structural Kernel Foundation
Primary workset:
- `W002_TREECALC_STRUCTURAL_STATE_AND_SNAPSHOT_KERNEL.md`

Entry gate:
- W001 canonical spec set is present and stable enough to drive implementation naming.
- The TreeCalc-first target remains unchanged.
- The accepted candidate-result versus publication split is already acknowledged across the seam.

Execution objective:
- realize the immutable structural kernel, snapshot root, stable identity policy, pinned-view basis, and projection/facade boundary.

Exit gate:
- the first implementation-facing structural state objects are explicit enough to code without reopening architecture choices,
- immutable structural truth, mutable derived runtime state, and observer-facing projections are separated in the implementation plan,
- the snapshot identity and pin-protection basis are concrete enough for W003 and W004 to depend on,
- no hidden mutable structure remains inside the Stage 1 structural boundary.

### Sequence 2. W003 Coordinator and Publication Baseline
Primary workset:
- `W003_STAGE1_COORDINATOR_AND_PUBLICATION_BASELINE.md`

Entry gate:
- Sequence 1 exit gate has passed.
- structural snapshot handles and reader-protection basis are stable enough to reference from coordinator state.

Execution objective:
- realize the Stage 1 coordinator state variables and `C1..C6` transition packet over the W002 kernel.

Exit gate:
- the coordinator state variables exist in implementation-facing form,
- the Stage 1 candidate-result intake and publication bundle path is realizable against the structural kernel,
- only `C4 AcceptAndPublish` can create stable publication,
- reject-is-no-publish is enforceable in implementation-facing logic,
- pinned-reader protection is concrete enough for W004 overlay retention and W008/W009 artifact authoring.

### Sequence 3. W004 Recalc and Overlay Baseline
Primary workset:
- `W004_INCREMENTAL_RECALC_AND_OVERLAY_BASELINE.md`

Entry gate:
- Sequence 2 exit gate has passed.
- the coordinator can accept, reject, publish, pin, and unpin against the W002 kernel.

Execution objective:
- realize the Stage 1 invalidation vocabulary, `I1..I8` transition packet, topo/SCC baseline, overlay key floor, retention matrix, and fallback floor.

Exit gate:
- the Stage 1 invalidation-state transitions are realizable end-to-end,
- recalc can drive candidate-result production into the coordinator path,
- overlay reuse, fallback, and eviction eligibility are explicit enough to instrument,
- the implementation-facing Stage 1 engine slice exists conceptually from structure through publication,
- W006 can now bind real implementation artifacts rather than only prose packets.

### Sequence 4. W006 Gate-Binding Execution Kickoff
Primary workset:
- `W006_CORE_FORMALIZATION_AND_GATE_BINDING.md`

Entry gate:
- Sequences 1 through 3 have passed.
- Stage 1 implementation-facing state and transition packets are concrete enough to bind into artifacts.

Execution objective:
- shift W006 from planning-only coordination into concrete artifact-binding authority for Lean, TLA+, replay, packs, and instrumentation.

Exit gate:
- the first actual artifact backlog is bound to the concrete Stage 1 state and transition set,
- W007 through W010 have declared authored-artifact targets rather than only planning obligations,
- gate ownership for the first Stage 1 implementation slice is explicit.

### Sequence 5. W007 Lean-Facing Artifact Baseline
Primary workset:
- `W007_LEAN_FACING_STATE_OBJECTS_AND_TRANSITION_BOUNDARY_PLAN.md`

Entry gate:
- Sequence 4 has passed.
- the W002/W003/W004 object names are stable enough to freeze into a first Lean-facing vocabulary.

Execution objective:
- author the first real Lean-facing state-object and transition-boundary skeleton corresponding to the implemented Stage 1 slice.

Exit gate:
- the first Lean-facing artifact exists,
- W002/W003/W004 state objects are mapped into that artifact without unresolved naming drift,
- at least the core structural state, coordinator state, and invalidation-state objects are represented,
- the theorem backlog can now reference authored objects instead of only prose names.

### Sequence 6. W008 TLA+ Skeleton
Primary workset:
- `W008_TLA_COORDINATOR_PUBLICATION_AND_FENCE_SAFETY_MODEL_PLAN.md`

Entry gate:
- Sequence 4 has passed.
- the Stage 1 transition bindings `C1..C6` and `I1..I8` are concrete enough to model.

Execution objective:
- author the first TLA+ skeleton over actions `A1..A10`, including the coordinator/publication/fence-safety state boundary.

Exit gate:
- the first TLA+ artifact exists,
- the Stage 1 transition-binding matrix is represented in the artifact,
- the first safety properties `S1..S6` are attached to explicit state variables and actions,
- the artifact is strict enough for W009 replay classes to reference it directly.

### Sequence 7. W009 Replay Artifact Baseline
Primary workset:
- `W009_REPLAY_AND_PACK_BINDING_FOR_STAGE1_SEAM_AND_COORDINATOR_BEHAVIOR.md`

Entry gate:
- Sequence 6 has passed.
- the Stage 1 transitions have both implementation-facing names and TLA+-facing names.

Execution objective:
- author the first replay artifact family for the Stage 1 coordinator and recalc chain.

Minimum first artifact order:
1. `R1` candidate-result versus publication separation,
2. `R2` reject-is-no-publish,
3. `R7` verification-clean without publication,
4. `R4`/`R5` pinned-reader and overlay protection,
5. `R3`/`R8` fence split and fallback or re-entry,
6. `R6` broader reject taxonomy coverage.

Exit gate:
- at least the first replay artifact set exists for `R1`, `R2`, and `R7`,
- Stage 1 transition labels are present in replay output,
- the replay artifacts reference the W008 action surface coherently,
- the first pack hooks can now bind to real artifacts rather than reserved classes.

### Sequence 8. W010 Measurement and Instrumentation Schema
Primary workset:
- `W010_EXPERIMENT_REGISTER_AND_MEASUREMENT_SCHEMA_PLANNING.md`

Entry gate:
- Sequences 3 and 7 have passed.
- replay labels and overlay/recalc transition points are stable enough to instrument.

Execution objective:
- author the first concrete counter schema and experiment-register binding for the Stage 1 implementation slice.

Exit gate:
- the first counter schema exists,
- the schema covers fallback, overlay reuse/miss, eviction, and recalc work-volume signatures,
- decisive experiments can reference actual counter families instead of only conceptual ones,
- W011 and W012 can consume a fixed measurement surface.

### Sequence 9. W011 Harness and Validator/Runner Implementation
Primary workset:
- `W011_CORE_ENGINE_TEST_HARNESS_AND_SELF_CONTAINED_FIXTURE_PLAN.md`

Entry gate:
- Sequences 7 and 8 have passed.
- replay classes, trace labels, and counter schema are concrete enough for a real harness boundary.

Execution objective:
- implement the first self-contained validator, runner, fixture lifecycle, and scriptable host for `TraceCalc` scenarios.

Exit gate:
- a runnable self-contained harness exists,
- the harness can execute the initial `TraceCalc` corpus,
- emitted artifacts use the canonical artifact root and naming policy,
- the harness can surface candidate-result, reject, publish, pinned-view, and counter outcomes in the declared shape.

### Sequence 10. W012 Reference Machine and Conformance Oracle
Primary workset:
- `W012_TRACECALC_REFERENCE_MACHINE_AND_CONFORMANCE_ORACLE.md`

Entry gate:
- Sequences 5 through 9 have passed.
- the harness can execute scenarios and emit canonical artifacts.

Execution objective:
- implement the first deterministic `TraceCalc Reference Machine` and use it as the semantic oracle for the Stage 1 engine slice.

Exit gate:
- the first reference-machine implementation exists,
- the reference machine can execute the seeded corpus through the harness boundary,
- oracle outputs are emitted at the canonical artifact root,
- the conformance surface for published view, pinned views, rejects, traces, and counters is executable rather than only specified.

## Parallel Side-Lane Rules
Execution Sequence A is primarily critical-path driven, but the following side-lane rules apply:
1. W003 may begin in late Sequence 1 once structural state names and snapshot handles stop moving.
2. W007 may begin in late Sequence 3 once the implementation-facing names stop drifting.
3. W008 and W009 should be developed in lockstep once Sequence 4 has passed.
4. W010 may begin once W004 and W009 expose stable transition points and labels.
5. W011 must wait for W009 and W010 artifact surfaces; it should not invent them.
6. W012 must wait for W011; it should not bypass the harness boundary.

## Carried Conditions Outside The 1-10 Sequence
The following remain active across the sequence but are not numbered sub-phases:
1. W001 remains the canonical-spec and repo-integration maintenance lane.
2. W005 remains the seam-alignment and follow-on-handoff lane.
3. If implementation work discovers narrower replay or trace pressure against OxFml, W005 must be re-opened at that specific seam point rather than letting local divergence accumulate.

## Final Gate For Execution Sequence A
Execution Sequence A reaches its final gate only when all of the following hold:
1. Sequences 1 through 10 have each met their declared exit gate.
2. A self-contained TreeCalc-first Stage 1 engine slice exists from immutable structure through coordinator publication, recalc/overlay behavior, harness execution, and oracle execution.
3. The first Lean-facing artifact, first TLA+ artifact, first replay artifact family, first counter schema, first harness, and first reference-machine artifact all exist and are mutually aligned to the same Stage 1 transition and state vocabulary.
4. At least the first declared replay classes `R1`, `R2`, `R7`, `R4`, and `R5` are authored and exercised.
5. Candidate-result versus publication, reject-is-no-publish, pinned-reader stability, verification-without-publication, and overlay-retention safety are no longer only spec claims; they are represented in emitted deterministic artifacts.
6. No unresolved local seam ambiguity remains that would block the Stage 1 implementation slice from being judged against its own oracle and replay artifacts.

This final gate is not the end of OxCalc Stage 1 in full.
It is the end of the first implementation wave that makes Stage 1 executable, measurable, and judgeable.

## Pre-Closure Verification Checklist
| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes |
| 2 | Pack expectations updated for affected packs? | no |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | no |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes |
| 6 | All required tests pass? | yes |
| 7 | No known semantic gaps remain in declared scope? | no |
| 8 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | yes |
| 9 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | yes |
| 10 | CURRENT_BLOCKERS.md updated (new/resolved)? | no |

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - Sequences 1 through 3 are now scaffolded into local implementation code with passing tests, but the gate criteria remain only partially discharged because replay, assurance, and broader scheduling artifacts do not exist yet
  - W001 and W005 remain carried governance/integration lanes across the sequence
  - Sequences 4 through 10 still depend on authored Lean, TLA+, replay, measurement, harness, and oracle artifacts that have not been realized yet
- claim_confidence: provisional
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
