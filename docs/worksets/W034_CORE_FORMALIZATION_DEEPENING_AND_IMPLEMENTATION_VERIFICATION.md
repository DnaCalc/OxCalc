# W034: Core Formalization Deepening And Implementation Verification

## Purpose

W034 continues the formalization path after W033 and its post-W033 successor slices.

The purpose is to deepen the checked model family, executable oracle coverage, and implementation-conformance evidence until the core engine can be improved with stronger confidence. W034 is not a validation pass against a frozen old spec. Current specs, current implementation behavior, TraceCalc evidence, OxFml seam facts, formal models, and scale evidence remain inputs to a discovery loop that may patch OxCalc-owned specs, open implementation work, record a deferred lane, or file an OxFml handoff when ownership requires it.

## Position And Dependencies

- depends_on: `W033`
- parent epic: `calc-e77`
- upstream dependencies: `OxFml`
- canonical planning companion: `docs/spec/core-engine/w034-formalization/W034_FORMALIZATION_DEEPENING_PLAN.md`

## Scope

### In Scope

1. TraceCalc reference-machine widening for the W034 tranche.
2. Optimized/core-engine and TreeCalc-to-TraceCalc conformance widening against the declared observable surface.
3. Lean proof-family deepening beyond `W033PostSlice`.
4. TLA+ model-family deepening for stale fences, dependency interleavings, overlays, pinned readers, and Stage 2 contention preconditions.
5. Pack/capability gate binding, including honest no-promotion decisions where evidence is still below the claimed floor.
6. Continuous scale-assurance criteria where scale evidence is tied to deterministic semantic checks.
7. OxFml seam-watch intake, including the current formatting/display split and typed conditional-formatting payload direction, without directly patching OxFml.
8. The narrow `LET`/`LAMBDA` carrier fragment where it threads through OxFml, OxFunc-opaque callable assumptions, TraceCalc, and OxCalc-visible dependency/runtime-effect behavior.
9. Spec evolution when formalization exposes a better model, stale wording, missing invariant, or implementation/spec mismatch.

### Out Of Scope

1. General OxFunc semantic kernels.
2. Direct edits to OxFml from this OxCalc workset.
3. Host/UI/file-adapter implementation.
4. Stage 2 policy promotion without the required proof/model/replay gate.
5. Pack-grade or performance claims based only on proxy signals.

## Gate Model

### Entry Gate

1. W033 first-pass formalization and post-W033 successor slices have evidence packets.
2. No W033 successor bead remains ready in the bead graph.
3. Current inbound OxFml notes are readable, including formatting/display and typed conditional-formatting updates.

### Exit Gate

1. W034 residual obligations are mapped to artifacts, beads, handoff/watch rows, or explicit deferred rationale.
2. TraceCalc oracle coverage is widened for the declared tranche with deterministic evidence.
3. Optimized/core-engine conformance is checked against the widened oracle surface.
4. Lean and TLA model-family artifacts are checked and linked to replay/conformance evidence.
5. Pack/capability and continuous scale gates state the actual evidence consequence and avoid proxy promotion.
6. Any Stage 2 contention result remains a precondition finding unless promotion gates are actually satisfied.
7. OxFml seam pressure is classified as no-handoff, watch, or handoff candidate with concrete rationale.
8. OxCalc-owned specs touched by the tranche remain aligned with the formal/replay/conformance artifacts.
9. Closure audit includes OPERATIONS Section 7 checklist, Section 9 self-audit, semantic-equivalence statement, and three-axis report.

## Evidence Layout

1. W034 planning root: `docs/spec/core-engine/w034-formalization/`
2. W034 formal root: existing `formal/` tree, with W034-specific Lean/TLA artifacts named by the owning beads before emission.
3. W034 replay/test roots: existing `docs/test-runs/core-engine/` tree, with new run ids declared by each execution bead before emission.
4. OxFml evidence roots are read-only inputs:
   - `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`
   - `../OxFml/docs/spec/`
   - `../OxFml/crates/oxfml_core/tests/fixtures/`
   - `../OxFml/formal/`

## Bead Rollout

Parent:

1. `calc-e77` - W034 core formalization deepening and implementation verification.

Child path:

1. `calc-e77.1` - W034 residual obligation and authority ledger.
2. `calc-e77.2` - W034 TraceCalc oracle deepening slice.
3. `calc-e77.3` - W034 optimized core conformance widening.
4. `calc-e77.4` - W034 Lean proof-family deepening.
5. `calc-e77.5` - W034 TLA model-family and contention precondition slice.
6. `calc-e77.6` - W034 pack capability and continuous scale gate binding.
7. `calc-e77.7` - W034 closure audit and successor packetization.

The child path is intentionally sequential for the first tranche. Later W034 child work may split into parallel beads only after the residual obligation ledger shows disjoint write/evidence scopes.

## Initial Guardrails

1. TraceCalc remains the correctness oracle only for covered reference behavior.
2. Optimized/core-engine conformance claims must state the observable surface and comparison limits.
3. Lean/TLA smoke or bounded checks must not be reported as broad proof closure.
4. Formatting/display seam facts are coordinator-visible only where the OxFml seam makes them semantic or replay-significant; renderer-only display behavior remains out of scope.
5. The OxFml W073 typed conditional-formatting input direction is an upstream seam-watch input, not an OxCalc request-construction change unless a W034 artifact exercises that path.
6. Stage 2 contention work may define preconditions and failure cases, but no concurrency policy is promoted without the required evidence bundle.
7. Any change to OxFml-owned evaluator/FEC/F3E clauses must use a handoff packet rather than a direct sibling-repo patch.

## Current Status

- execution_state: `planned`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - W034 child beads are open
  - W034 residual obligation ledger has not yet reached its gate
  - TraceCalc, conformance, Lean, TLA, pack/scale, and audit evidence remain future W034 child work
