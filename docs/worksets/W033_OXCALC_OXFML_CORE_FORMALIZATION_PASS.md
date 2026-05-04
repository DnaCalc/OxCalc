# W033: OxCalc + OxFml Core Formalization Pass

## Purpose

Plan and execute a comprehensive OxCalc-owned formalization pass that covers OxCalc core-engine semantics plus the OxFml evaluator/FEC-F3E surfaces consumed by OxCalc.

The workset is repo-local to OxCalc. It may read OxFml specs, formal artifacts, fixtures, and observation ledgers as authoritative inputs, but it does not patch OxFml directly and does not transfer canonical OxFml ownership into OxCalc.

## Highest-Level Intent

The point of this workset is to make the OxCalc + OxFml engine/seam contract trustworthy under change.

This means producing checkable artifacts for the parts where informal reasoning is too weak: stale publishes, torn publication, missed invalidation, unsafe overlay reuse, reject paths that accidentally publish, and optimization or scheduling changes that alter stabilized observable results.

The workset does not try to prove every formula function. OxFunc semantic kernels stay out of scope. The core target is the interaction between OxFml evaluator facts and OxCalc coordinator behavior: candidate is not publication, reject is no-publish, accepted publication is atomic, dependency/runtime-effect facts are not silently lost, overlays are epoch/fence safe, and strategy changes preserve semantics.

The deliberate exception is `LET` and `LAMBDA` at the OxFml/OxFunc boundary. Their local binding, lambda value, call identity, prepared-call shape, and dependency/runtime-effect visibility have to be modeled as carrier facts because they can affect OxCalc-visible behavior. This is a narrow boundary fragment, not ownership of general OxFunc semantics.

TraceCalc is the executable correctness oracle for the spec surfaces it realizes. W033 may formally check both the TraceCalc reference-machine surface and the production/core-engine surface, but the authority relationship is asymmetric: TraceCalc establishes covered reference behavior, and optimized or production implementations prove conformance to it for covered observable semantics.

The workset also acts as a quality-tempering pass over the core-engine spec set. It should reread the initial core-engine specification documents, make their state/transition vocabulary sharper, remove drift against realized evidence, and keep the prose specs aligned with the formal, replay, and pack artifacts created by this pass.

The review surface includes both current canonical docs and the superseded bootstrap formal/theory documents as historical no-loss inputs. Historical inputs may recover intent, vocabulary, or deferred lanes, but they do not override current charter, operation, or canonical spec authority.

W033 is not only a verification pass over the current implementation, and it is not a test against a frozen initial spec. Current implementation behavior, current specs, historical intent, TraceCalc, formal models, replay evidence, and scale evidence are all inputs to a controlled discovery loop. When that loop exposes a better domain model, a missing invariant, an incorrect assumption, or a scope boundary that should move, the expected outcome is an explicit spec-evolution decision: patch OxCalc-owned specs, file an OxFml handoff where ownership requires it, or record a deferred/non-carry-forward rationale.

## Formal Leverage Model

W033 should deliberately use several complementary formal tools rather than treating "formalization" as one technique:

1. Operational semantics and refinement:
   - treat TraceCalc as the small-step executable reference semantics for covered behavior,
   - state production/core-engine behavior as a refinement of TraceCalc over a declared observable surface.
2. TLA+ and temporal/state modeling:
   - use TLA+ for publication, fences, pinned readers, overlays, scheduling, dependency invalidation interleavings, and later concurrency pressure.
3. Lean and transition invariants:
   - use Lean where the invariant is crisp over state transitions, such as reject-is-no-publish, publication atomicity, invalidation closure, protected overlay retention, and replay-equivalent sequential histories.
4. Graph theory and incremental-computation vocabulary:
   - make DAG, SCC, dynamic-dependency graph, invalidation closure, topological order, cycle region, and fixed-point vocabulary explicit where the engine already depends on it.
5. Abstract interpretation and dataflow conservatism:
   - model uncertain references and runtime-derived dependencies as conservative approximations where over-invalidation is admissible but under-invalidation is not.
6. Metamorphic and differential testing:
   - use transformations such as independent-node reorderings, from-scratch versus incremental recalc, `LET` inlining, `LAMBDA` call refactoring, and scheduling-policy changes to multiply coverage without relying only on example fixtures.

## Position and Dependencies

- **Depends on**: W031, W032, W020, W026
- **Blocks**: later Stage 2 concurrency promotion, formal pack promotion, and any optimization lane that needs a stronger cross-lane semantic authority
- **Cross-repo**: OxFml is an upstream input and seam owner; handoff is required for any normative OxFml/FEC-F3E change

## Scope

### In scope

1. OxCalc core state, recalc, dependency, overlay, coordinator, publication, replay, and TreeCalc runtime formalization.
2. OxFml evaluator-facing and FEC/F3E seam surfaces needed to state OxCalc correctness:
   - evaluator session lifecycle,
   - candidate result,
   - commit bundle,
   - typed reject,
   - fence and capability facts,
   - runtime-derived effect facts,
   - trace and replay correlation.
3. A cross-lane authority and claim-to-artifact matrix.
4. A narrow `LET`/`LAMBDA` OxFml/OxFunc boundary fragment covering carrier facts needed by OxCalc-visible dependency, trace, replay, and runtime-effect behavior.
5. TraceCalc expansion as a reference-machine and correctness-oracle surface for covered spec behavior.
6. A review-and-correction pass through the initial OxCalc core-engine spec documents.
7. A historical no-loss crosswalk against the superseded bootstrap formal model, theory, and rewrite-control materials.
8. Lean and TLA+ widening plans and first execution packets.
9. Replay, witness, pack, and capability-claim mapping.
10. Explicit handoff candidates where OxCalc formalization exposes an OxFml seam insufficiency.

### Out of scope

1. OxFunc semantic kernels, coercion rules, and function catalog truth, except for the narrow `LET`/`LAMBDA` boundary carrier assumptions named in scope.
2. Direct edits to the OxFml repo from this OxCalc workset.
3. Host/UI/file-adapter semantics.
4. Broad grid semantics beyond the current TreeCalc-first floor.
5. Stage 2 concurrency policy promotion.
6. Pack-grade claims without proof/model/replay evidence.

## Deliverables

1. `docs/spec/core-engine/CORE_ENGINE_OXCALC_OXFML_FORMALIZATION_PASS_PLAN.md`.
2. A cross-lane formalization matrix mapping claims to specs, proofs, TLA+ properties, replay witnesses, packs, and handoff state.
3. A core-engine spec review ledger identifying corrected, confirmed, provisional, and deferred clauses.
4. A historical no-loss crosswalk for original formal/theory ideas that are confirmed, promoted, deferred, or intentionally not carried forward.
5. A spec-evolution decision ledger for discoveries that change, widen, narrow, or defer current scope/spec claims.
6. Spec updates where W033 exposes drift, ambiguity, missing invariants, stale realization notes, or a better domain model.
7. An observable-surface and refinement-relation packet for TraceCalc-to-production/core-engine conformance.
8. A metamorphic and differential test-family packet tied to TraceCalc, TreeCalc, or later replay evidence.
9. Lean module-family rollout plan and first artifact-widening beads.
10. TLA+ model-family rollout plan and first model-check beads.
11. Replay/witness bridge plan over OxCalc and read-only OxFml evidence.
12. Pack/capability mapping with explicit deferred rows.
13. Handoff packet candidates only where needed by concrete seam pressure.

## Gate Model

### Entry gate

- OxCalc canonical core-engine specs exist and identify formalization as an active partial lane.
- OxFml current consumer, formalization, and FEC/F3E assurance surfaces are readable as upstream inputs.
- TreeCalc local evidence and scale evidence exist as current OxCalc runtime evidence inputs.

### Exit gate

- The W033 formalization scope is decomposed into explicit epics and beads.
- The initial core-engine spec documents have been reviewed against current evidence, and any W033-discovered correction is either patched or recorded as a deferred lane with rationale.
- Current implementation behavior and current spec text have been treated as evidence inputs rather than as immutable targets.
- Any W033-discovered scope/spec evolution decision is patched, handed off, deferred, or explicitly rejected with rationale.
- Historical formal/theory inputs have been checked for no-loss intent coverage, with any recovered lane mapped to current docs, a deferred item, or an explicit non-carry-forward rationale.
- The authority matrix identifies OxCalc-owned, OxFml-owned, shared, OxFunc-opaque, and `LET`/`LAMBDA` boundary-carrier clauses.
- Each first-pass claim has at least one planned proof/model/replay/pack obligation or an explicit deferred rationale.
- TraceCalc oracle claims are separated from production/core-engine conformance claims.
- The first observable-surface/refinement-relation shape is explicit enough to drive artifacts.
- Initial Lean, TLA+, replay, pack, and handoff lanes have reviewable next beads.
- No claim is promoted beyond the evidence available in the declared scope.

## Execution Packet

### Environment Preconditions

- Rust workspace tooling is available for OxCalc validation where code or artifact generators change.
- PowerShell is available for repo-local validation scripts and formal runners.
- Lean and TLC are optional until a specific formal-artifact bead declares them required.
- OxFml is read-only for this OxCalc workset.

### Evidence Layout

- Canonical planning artifact: `docs/spec/core-engine/CORE_ENGINE_OXCALC_OXFML_FORMALIZATION_PASS_PLAN.md`
- W033 spec-evidence artifact root: `docs/spec/core-engine/w033-formalization/`
- W033 source freeze and layout packet: `docs/spec/core-engine/w033-formalization/W033_SOURCE_AUTHORITY_AND_ARTIFACT_LAYOUT.md`
- W033 core spec review ledger: `docs/spec/core-engine/w033-formalization/W033_CORE_SPEC_REVIEW_LEDGER.md`
- W033 spec-evolution decision ledger: `docs/spec/core-engine/w033-formalization/W033_SPEC_EVOLUTION_DECISION_LEDGER.md`
- W033 historical no-loss crosswalk: `docs/spec/core-engine/w033-formalization/W033_HISTORICAL_NO_LOSS_CROSSWALK.md`
- W033 authority and claim matrix: `docs/spec/core-engine/w033-formalization/W033_AUTHORITY_AND_CLAIM_MATRIX.md`
- W033 object vocabulary and `LET`/`LAMBDA` boundary packet: `docs/spec/core-engine/w033-formalization/W033_OBJECT_VOCABULARY_AND_LET_LAMBDA_BOUNDARY.md`
- W033 TraceCalc refinement packet: `docs/spec/core-engine/w033-formalization/W033_TRACECALC_REFINEMENT_PACKET.md`
- W033 TraceCalc oracle self-check first slice: `docs/spec/core-engine/w033-formalization/W033_TRACECALC_ORACLE_SELF_CHECK_FIRST_SLICE.md`
- W033 production/core-engine conformance first slice: `docs/spec/core-engine/w033-formalization/W033_PRODUCTION_CONFORMANCE_FIRST_SLICE.md`
- W033 metamorphic and differential test-family packet: `docs/spec/core-engine/w033-formalization/W033_METAMORPHIC_DIFFERENTIAL_TEST_FAMILIES.md`
- W033 Lean first-slice packet: `docs/spec/core-engine/w033-formalization/W033_LEAN_MODULE_FAMILY_FIRST_SLICE.md`
- W033 TLA bridge first-slice packet: `docs/spec/core-engine/w033-formalization/W033_TLA_BRIDGE_FIRST_SLICE.md`
- Existing OxCalc formal root: `formal/`
- Existing OxCalc Lean root with W033 first slice: `formal/lean/OxCalc/CoreEngine/W033FirstSlice.lean`
- Existing OxCalc replay/test roots:
  - `formal/replay/`
  - `docs/test-corpus/core-engine/tracecalc/`
  - `docs/test-runs/core-engine/`
- OxFml read-only evidence roots:
  - `../OxFml/formal/`
  - `../OxFml/crates/oxfml_core/tests/fixtures/`
- Historical orientation roots:
  - `docs/spec/core-engine/CORE_ENGINE_FORMAL_MODEL.md`
  - `docs/spec/core-engine/CORE_ENGINE_THEORY_AND_ALTERNATIVE_PATHS.md`
  - `docs/spec/core-engine/archive/bootstrap-2026-03/`
  - `docs/spec/core-engine/archive/rewrite-control-2026-03/`

Any new checked-in evidence root must be declared before it is emitted.

### Initial Epic Lanes

1. Core spec review and correction ledger.
2. Historical no-loss crosswalk against original formal/theory materials.
3. Authority and source inventory.
4. Cross-lane vocabulary and object-model alignment.
5. Formal leverage mapping: refinement, temporal modeling, graph/dataflow theory, and metamorphic testing.
6. Observable-surface and refinement-relation packetization.
7. Lean model-family widening.
8. TLA+ model-family widening.
9. Replay and witness bridge.
10. Pack and capability binding.
11. OxFml handoff/watch lane.
12. Closure audit and successor packetization.

## Bead Rollout

W033 is rolled into `.beads/` under parent epic `calc-uri`.

Parent:
1. `calc-uri` - W033 OxCalc + OxFml core formalization and spec evolution.

Parent dependency anchors:
1. `calc-ukb` - W031 TreeCalc assurance refresh and residual packetization.
2. `calc-fbx` - W032 OxCalc beads migration and light doctrine reorientation.
3. `calc-cmj` - W026 TreeCalc OxFml bind/reference and seam intake.
4. `W020` remains a source/spec dependency without a current local bead anchor.

Child bead path:
1. `calc-uri.1` - freeze source authority and artifact layout.
2. `calc-uri.2` - author core spec review ledger and sweep order.
3. `calc-uri.3` - author spec-evolution decision taxonomy and first ledger.
4. `calc-uri.4` - build historical no-loss crosswalk.
5. `calc-uri.5` - build cross-lane authority and claim matrix.
6. `calc-uri.6` - align object vocabulary and `LET`/`LAMBDA` carrier boundary.
7. `calc-uri.7` - define observable surface and TraceCalc refinement relation.
8. `calc-uri.8` - widen TraceCalc oracle self-check first slice.
9. `calc-uri.9` - add production conformance comparison first slice.
10. `calc-uri.10` - define metamorphic and differential test families.
11. `calc-uri.11` - widen Lean module family first slice.
12. `calc-uri.12` - widen TLA bridge first slice.
13. `calc-uri.13` - map replay and witness bridge across OxCalc and OxFml.
14. `calc-uri.14` - bind packs and capability claims to current evidence.
15. `calc-uri.15` - packetize OxFml handoff and watch candidates.
16. `calc-uri.16` - closure audit and successor packetization.

Current ready path:
1. `calc-uri.1` is the first ready bead.
2. `calc-uri.2`, `calc-uri.3`, and `calc-uri.4` depend on `calc-uri.1`.
3. `calc-uri.5` depends on the first three inventory/ledger lanes.
4. Formal, replay, conformance, pack, handoff, and closure beads remain blocked by their declared predecessor evidence.

## Open Guardrails

1. OxFml formal artifacts are inputs, not OxCalc-owned canonical truth.
2. OxFml note-level residuals remain residual unless exercised evidence exposes a concrete insufficiency.
3. OxFunc appears only through opaque assumptions and already surfaced OxFml packet facts, with `LET`/`LAMBDA` admitted only as a narrow carrier fragment.
4. Stage 2 concurrency remains blocked on explicit model/replay gates, not on prose planning alone.
5. Performance/scaling evidence remains measurement input unless tied to semantic or pack obligations.
6. TraceCalc oracle authority applies only to covered behavior with reference-machine and comparison evidence.
7. Spec text changed by W033 must remain mapped to an authority source, formal/replay obligation, or explicit deferred rationale.
8. Historical/bootstrap docs are no-loss review inputs only; they do not become normative unless promoted into current specs through W033 artifacts.
9. The current implementation is evidence, not normative authority; discovered implementation behavior must be classified as intended behavior, implementation fault, spec gap, or unresolved evidence before it affects scope.
10. Current spec text is a starting authority surface, not an immutable final target; changes require explicit authority mapping and evidence/deferred rationale.

## Pre-Closure Verification Checklist

1. Spec text and realization notes updated for all in-scope items: no
2. Pack expectations updated for affected packs: no
3. At least one deterministic replay artifact exists per in-scope behavior: no
4. Semantic-equivalence statement provided for policy or strategy changes: no
5. FEC/F3E cross-repo impact assessed and handoff filed if needed: no
6. All required tests pass: no
7. No known semantic gaps remain in declared scope: no
8. Completion language audit passed: no
9. `WORKSET_REGISTER.md` updated when ordered workset truth changed: no
10. `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed: no
11. execution-state blocker surface updated if needed: no

## Status

- execution_state: bead_rollout_created
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - parent epic `calc-uri` is open
  - source authority and artifact layout packet exists under `docs/spec/core-engine/w033-formalization/`
  - core-engine spec review ledger exists under `docs/spec/core-engine/w033-formalization/`
  - spec-evolution decision ledger exists under `docs/spec/core-engine/w033-formalization/`
  - historical no-loss crosswalk exists under `docs/spec/core-engine/w033-formalization/`
  - authority and claim matrix exists under `docs/spec/core-engine/w033-formalization/`
  - object vocabulary and `LET`/`LAMBDA` boundary packet exists under `docs/spec/core-engine/w033-formalization/`
  - TraceCalc observable-surface/refinement-relation packet exists under `docs/spec/core-engine/w033-formalization/`
  - TraceCalc oracle self-check first slice exists under `docs/spec/core-engine/w033-formalization/`
  - production/core-engine conformance first slice exists under `docs/spec/core-engine/w033-formalization/`
  - metamorphic and differential test-family packet exists under `docs/spec/core-engine/w033-formalization/`
  - Lean first-slice packet and checked Lean artifact exist
  - TLA bridge first-slice packet exists and Stage 1 smoke model was checked
  - no new pack or handoff artifacts exist for W033
  - broad independent production conformance remains open beyond the first W033 artifact
  - OxFml is in formalization scope as a read-only upstream/seam input
  - OxFunc semantic kernels remain out of scope except for the narrow `LET`/`LAMBDA` boundary carrier fragment
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`
